use super::roblox::{Presence, get_json, headshots, post};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameCard {
    pub universe_id: u64,
    pub root_place_id: u64,
    pub name: String,
    pub player_count: u64,
    pub up_votes: u64,
    pub down_votes: u64,
    pub icon: Option<String>,
}

fn card(v: &Value) -> Option<GameCard> {
    Some(GameCard {
        universe_id: v["universeId"].as_u64()?,
        root_place_id: v["rootPlaceId"].as_u64()?,
        name: v["name"].as_str()?.to_string(),
        player_count: v["playerCount"].as_u64().unwrap_or(0),
        up_votes: v["totalUpVotes"].as_u64().unwrap_or(0),
        down_votes: v["totalDownVotes"].as_u64().unwrap_or(0),
        icon: None,
    })
}

fn session_id() -> String {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let h = format!(
        "{:032x}",
        n.wrapping_mul(0x9E37_79B9_7F4A_7C15_F39C_C060_5CED_C835)
    );
    format!(
        "{}-{}-4{}-a{}-{}",
        &h[0..8],
        &h[8..12],
        &h[13..16],
        &h[17..20],
        &h[20..32]
    )
}

pub(crate) async fn with_icons(mut games: Vec<GameCard>) -> Vec<GameCard> {
    let ids: Vec<u64> = games
        .iter()
        .map(|g| g.universe_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let mut icons = HashMap::new();
    for chunk in ids.chunks(50) {
        let list = chunk
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let url = format!(
            "https://thumbnails.roblox.com/v1/games/icons?universeIds={list}&returnPolicy=PlaceHolder&size=256x256&format=Png&isCircular=false"
        );
        if let Ok(v) = get_json(&url, None).await {
            for d in v["data"].as_array().into_iter().flatten() {
                if let (Some(id), Some(u)) = (d["targetId"].as_u64(), d["imageUrl"].as_str()) {
                    icons.insert(id, u.to_string());
                }
            }
        }
    }
    for g in &mut games {
        g.icon = icons.get(&g.universe_id).cloned();
    }
    games
}

pub async fn banners(ids: &[u64]) -> HashMap<u64, String> {
    let mut out = HashMap::new();
    for chunk in ids.chunks(25) {
        let list = chunk
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let url = format!(
            "https://thumbnails.roblox.com/v1/games/multiget/thumbnails?universeIds={list}&countPerUniverse=1&defaults=true&size=768x432&format=Png&isCircular=false"
        );
        if let Ok(v) = get_json(&url, None).await {
            for d in v["data"].as_array().into_iter().flatten() {
                if let (Some(id), Some(u)) = (
                    d["universeId"].as_u64(),
                    d["thumbnails"][0]["imageUrl"].as_str(),
                ) {
                    out.insert(id, u.to_string());
                }
            }
        }
    }
    out
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Chart {
    pub id: String,
    pub name: String,
    pub games: Vec<GameCard>,
}

pub async fn charts(per_chart: usize) -> Result<Vec<Chart>, String> {
    let url = format!(
        "https://apis.roblox.com/explore-api/v1/get-sorts?sessionId={}&device=computer&country=all",
        session_id()
    );
    let v = get_json(&url, None).await?;
    let mut charts: Vec<Chart> = v["sorts"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|s| s["contentType"] == "Games")
        .map(|s| Chart {
            id: s["sortId"].as_str().unwrap_or_default().to_string(),
            name: s["sortDisplayName"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            games: s["games"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|g| g["isSponsored"] != true)
                .filter_map(card)
                .take(per_chart)
                .collect(),
        })
        .filter(|c| !c.games.is_empty())
        .collect();

    let all = with_icons(charts.iter().flat_map(|c| c.games.clone()).collect()).await;
    let icons: HashMap<u64, Option<String>> =
        all.into_iter().map(|g| (g.universe_id, g.icon)).collect();
    for c in &mut charts {
        for g in &mut c.games {
            g.icon = icons.get(&g.universe_id).cloned().flatten();
        }
    }
    Ok(charts)
}

pub async fn search(query: &str) -> Result<Vec<GameCard>, String> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(vec![]);
    }
    let url = format!(
        "https://apis.roblox.com/search-api/omni-search?searchQuery={}&sessionId={}&pageType=all",
        urlencoding::encode(q),
        session_id()
    );
    let v = get_json(&url, None).await?;
    let games = v["searchResults"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|g| g["contentGroupType"] == "Game")
        .flat_map(|g| g["contents"].as_array().cloned().unwrap_or_default())
        .filter(|c| c["isSponsored"] != true)
        .filter_map(|c| card(&c))
        .take(40)
        .collect();
    Ok(with_icons(games).await)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDetail {
    pub universe_id: u64,
    pub root_place_id: u64,
    pub name: String,
    pub description: String,
    pub creator: String,
    pub playing: u64,
    pub visits: u64,
    pub max_players: u64,
    pub updated: String,
    pub genre: String,
    pub up_votes: u64,
    pub down_votes: u64,
    pub icon: Option<String>,
    pub banners: Vec<String>,
}

pub async fn game(universe_id: u64) -> Result<GameDetail, String> {
    let info = get_json(
        &format!("https://games.roblox.com/v1/games?universeIds={universe_id}"),
        None,
    )
    .await?;
    let g = info["data"].get(0).ok_or("Roblox didn't find that game.")?;
    let votes = get_json(
        &format!("https://games.roblox.com/v1/games/votes?universeIds={universe_id}"),
        None,
    )
    .await
    .unwrap_or_default();
    let banners = get_json(&format!("https://thumbnails.roblox.com/v1/games/multiget/thumbnails?universeIds={universe_id}&countPerUniverse=5&defaults=true&size=768x432&format=Png&isCircular=false"), None)
        .await.unwrap_or_default();
    let icon = with_icons(vec![GameCard {
        universe_id,
        root_place_id: 0,
        name: String::new(),
        player_count: 0,
        up_votes: 0,
        down_votes: 0,
        icon: None,
    }])
    .await
    .pop()
    .and_then(|c| c.icon);
    let s = |k: &str| g[k].as_str().unwrap_or_default().to_string();
    let n = |k: &str| g[k].as_u64().unwrap_or(0);
    Ok(GameDetail {
        universe_id,
        root_place_id: n("rootPlaceId"),
        name: s("name"),
        description: s("description"),
        creator: g["creator"]["name"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        playing: n("playing"),
        visits: n("visits"),
        max_players: n("maxPlayers"),
        updated: s("updated"),
        genre: s("genre"),
        up_votes: votes["data"][0]["upVotes"].as_u64().unwrap_or(0),
        down_votes: votes["data"][0]["downVotes"].as_u64().unwrap_or(0),
        icon,
        banners: banners["data"][0]["thumbnails"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|t| t["imageUrl"].as_str().map(str::to_string))
            .collect(),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub id: String,
    pub playing: u64,
    pub max_players: u64,
    pub ping: Option<u64>,
    pub fps: Option<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerPage {
    pub servers: Vec<Server>,
    pub next_cursor: Option<String>,
}

pub async fn servers(
    place_id: u64,
    most_players: bool,
    exclude_full: bool,
    cursor: Option<String>,
) -> Result<ServerPage, String> {
    let order = if most_players { "Desc" } else { "Asc" };
    let mut url = format!(
        "https://games.roblox.com/v1/games/{place_id}/servers/Public?sortOrder={order}&limit=100&excludeFullGames={exclude_full}"
    );
    if let Some(c) = cursor.filter(|c| !c.is_empty()) {
        url += &format!("&cursor={}", urlencoding::encode(&c));
    }
    let v = get_json(&url, None).await?;
    Ok(ServerPage {
        servers: v["data"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|s| {
                Some(Server {
                    id: s["id"].as_str()?.to_string(),
                    playing: s["playing"].as_u64().unwrap_or(0),
                    max_players: s["maxPlayers"].as_u64().unwrap_or(0),
                    ping: s["ping"].as_u64(),
                    fps: s["fps"].as_f64(),
                })
            })
            .collect(),
        next_cursor: v["nextPageCursor"].as_str().map(str::to_string),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendInGame {
    pub user_id: u64,
    pub display_name: String,
    pub avatar: Option<String>,
    pub place_id: u64,
    pub game_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendGame {
    pub game: GameCard,
    pub friends: Vec<FriendInGame>,
}

pub async fn friends_playing(user_id: u64, cookie: &str) -> Result<Vec<FriendGame>, String> {
    let f = get_json(
        &format!("https://friends.roblox.com/v1/users/{user_id}/friends"),
        Some(cookie),
    )
    .await?;
    let ids: Vec<u64> = f["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|u| u["id"].as_u64())
        .collect();

    let mut in_game: Vec<Presence> = vec![];
    for chunk in ids.chunks(50) {
        in_game.extend(
            super::roblox::presence(cookie, chunk)
                .await?
                .into_iter()
                .filter(|p| {
                    p.user_presence_type == 2 && p.universe_id.is_some() && p.place_id.is_some()
                }),
        );
    }
    if in_game.is_empty() {
        return Ok(vec![]);
    }

    let friend_ids: Vec<u64> = in_game.iter().map(|p| p.user_id).collect();
    let names: HashMap<u64, String> = match post(
        "https://users.roblox.com/v1/users",
        None,
        json!({ "userIds": friend_ids, "excludeBannedUsers": false }),
    )
    .await
    {
        Ok(r) => r.json::<Value>().await.unwrap_or_default()["data"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|u| Some((u["id"].as_u64()?, u["displayName"].as_str()?.to_string())))
            .collect(),
        Err(_) => HashMap::new(),
    };
    let avatars = headshots(&friend_ids).await;

    let universes: Vec<u64> = in_game
        .iter()
        .filter_map(|p| p.universe_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let list = universes
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let info = get_json(
        &format!("https://games.roblox.com/v1/games?universeIds={list}"),
        None,
    )
    .await?;
    let cards = with_icons(
        info["data"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|g| {
                Some(GameCard {
                    universe_id: g["id"].as_u64()?,
                    root_place_id: g["rootPlaceId"].as_u64()?,
                    name: g["name"].as_str()?.to_string(),
                    player_count: g["playing"].as_u64().unwrap_or(0),
                    up_votes: 0,
                    down_votes: 0,
                    icon: None,
                })
            })
            .collect(),
    )
    .await;

    let mut out: Vec<FriendGame> = cards
        .into_iter()
        .map(|game| {
            let friends = in_game
                .iter()
                .filter(|p| p.universe_id == Some(game.universe_id))
                .map(|p| FriendInGame {
                    user_id: p.user_id,
                    display_name: names
                        .get(&p.user_id)
                        .cloned()
                        .unwrap_or_else(|| format!("User {}", p.user_id)),
                    avatar: avatars.get(&p.user_id).cloned(),
                    place_id: p.place_id.unwrap_or(game.root_place_id),
                    game_id: p.game_id.clone(),
                })
                .collect();
            FriendGame { game, friends }
        })
        .collect();
    out.sort_by(|a, b| b.friends.len().cmp(&a.friends.len()));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_is_uuid_shaped() {
        let s = session_id();
        let parts: Vec<_> = s.split('-').map(str::len).collect();
        assert_eq!(parts, [8, 4, 4, 4, 12]);
    }

    #[test]
    fn parses_cards_and_skips_incomplete() {
        let ok = json!({ "universeId": 1, "rootPlaceId": 2, "name": "Doors", "playerCount": 50 });
        let c = card(&ok).unwrap();
        assert_eq!((c.universe_id, c.root_place_id, c.player_count), (1, 2, 50));
        assert!(card(&json!({ "name": "no ids" })).is_none());
    }
}
