use super::roblox::{get_json, headshots, post, presence};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::HashMap;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub user_id: u64,
    pub name: String,
    pub display_name: String,
    pub verified: bool,
    pub avatar: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Friend {
    pub user_id: u64,
    pub name: String,
    pub display_name: String,
    pub avatar: Option<String>,
    pub status: u8,
    pub last_location: Option<String>,
    pub place_id: Option<u64>,
    pub game_id: Option<String>,
    pub universe_id: Option<u64>,
}

async fn avatars(ids: &[u64]) -> HashMap<u64, String> {
    let mut out = HashMap::new();
    for chunk in ids.chunks(100) {
        out.extend(headshots(chunk).await);
    }
    out
}

async fn names(ids: &[u64]) -> HashMap<u64, (String, String, bool)> {
    let mut out = HashMap::new();
    for chunk in ids.chunks(100) {
        let Ok(r) = post(
            "https://users.roblox.com/v1/users",
            None,
            json!({ "userIds": chunk, "excludeBannedUsers": false }),
        )
        .await
        else {
            continue;
        };
        let v: Value = r.json().await.unwrap_or_default();
        for u in v["data"].as_array().into_iter().flatten() {
            if let Some(id) = u["id"].as_u64() {
                out.insert(
                    id,
                    (
                        u["name"].as_str().unwrap_or_default().to_string(),
                        u["displayName"].as_str().unwrap_or_default().to_string(),
                        u["hasVerifiedBadge"].as_bool().unwrap_or(false),
                    ),
                );
            }
        }
    }
    out
}

pub async fn search(keyword: &str) -> Result<Vec<Player>, String> {
    let k: String = keyword.trim().chars().take(20).collect();
    if k.chars().count() < 3 {
        return Ok(vec![]);
    }
    let v = get_json(
        &format!(
            "https://users.roblox.com/v1/users/search?keyword={}&limit=10",
            urlencoding::encode(&k)
        ),
        None,
    )
    .await
    .map_err(|e| {
        if e.contains("429") {
            "Roblox is limiting player searches. Wait a few seconds and try again.".into()
        } else {
            e
        }
    })?;
    let mut list: Vec<Player> = v["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|u| {
            Some(Player {
                user_id: u["id"].as_u64()?,
                name: u["name"].as_str()?.to_string(),
                display_name: u["displayName"].as_str().unwrap_or_default().to_string(),
                verified: u["hasVerifiedBadge"].as_bool().unwrap_or(false),
                avatar: None,
            })
        })
        .collect();
    let a = avatars(&list.iter().map(|p| p.user_id).collect::<Vec<_>>()).await;
    for p in &mut list {
        p.avatar = a.get(&p.user_id).cloned();
    }
    Ok(list)
}

pub async fn friends(user_id: u64, cookie: &str) -> Result<Vec<Friend>, String> {
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
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let mut states = HashMap::new();
    for chunk in ids.chunks(50) {
        for p in presence(cookie, chunk).await.unwrap_or_default() {
            states.insert(p.user_id, p);
        }
    }
    let n = names(&ids).await;
    let a = avatars(&ids).await;
    let mut list: Vec<Friend> = ids
        .iter()
        .map(|id| {
            let p = states.get(id);
            let (name, display_name, _) = n.get(id).cloned().unwrap_or_default();
            Friend {
                user_id: *id,
                display_name: if display_name.is_empty() {
                    name.clone()
                } else {
                    display_name
                },
                name,
                avatar: a.get(id).cloned(),
                status: p.map(|p| p.user_presence_type).unwrap_or(0),
                last_location: p
                    .and_then(|p| p.last_location.clone())
                    .filter(|l| !l.is_empty() && l != "Website"),
                place_id: p.and_then(|p| p.place_id),
                game_id: p.and_then(|p| p.game_id.clone()),
                universe_id: p.and_then(|p| p.universe_id),
            }
        })
        .collect();
    list.sort_by_key(|f| {
        (
            match f.status {
                2 => 0,
                1 => 1,
                3 => 2,
                _ => 3,
            },
            f.display_name.to_lowercase(),
        )
    });
    Ok(list)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Created {
    pub universe_id: u64,
    pub root_place_id: u64,
    pub name: String,
    pub visits: u64,
    pub icon: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub user_id: u64,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub created: String,
    pub verified: bool,
    pub banned: bool,
    pub friends: u64,
    pub followers: u64,
    pub following: u64,
    pub avatar: Option<String>,
    pub status: Option<u8>,
    pub last_location: Option<String>,
    pub place_id: Option<u64>,
    pub game_id: Option<String>,
    pub games: Vec<Created>,
}

async fn count(user_id: u64, what: &str) -> u64 {
    get_json(
        &format!("https://friends.roblox.com/v1/users/{user_id}/{what}/count"),
        None,
    )
    .await
    .ok()
    .and_then(|v| v["count"].as_u64())
    .unwrap_or(0)
}

pub async fn profile(user_id: u64, cookie: Option<String>) -> Result<Profile, String> {
    let u = get_json(
        &format!("https://users.roblox.com/v1/users/{user_id}"),
        None,
    )
    .await?;
    let (friends, followers, following) = tokio::join!(
        count(user_id, "friends"),
        count(user_id, "followers"),
        count(user_id, "followings")
    );
    let avatar = get_json(
        &format!("https://thumbnails.roblox.com/v1/users/avatar?userIds={user_id}&size=420x420&format=Png&isCircular=false"),
        None,
    )
    .await
    .ok()
    .and_then(|v| v["data"][0]["imageUrl"].as_str().map(str::to_string));
    let here = match &cookie {
        Some(c) => presence(c, &[user_id]).await.ok().and_then(|mut p| p.pop()),
        None => None,
    };
    let g = get_json(
        &format!("https://games.roblox.com/v2/users/{user_id}/games?accessFilter=2&limit=10&sortOrder=Desc"),
        None,
    )
    .await
    .unwrap_or_default();
    let mut games: Vec<Created> = g["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|x| {
            Some(Created {
                universe_id: x["id"].as_u64()?,
                root_place_id: x["rootPlace"]["id"].as_u64()?,
                name: x["name"].as_str().unwrap_or_default().to_string(),
                visits: x["placeVisits"].as_u64().unwrap_or(0),
                icon: None,
            })
        })
        .collect();
    if !games.is_empty() {
        let ids = games
            .iter()
            .map(|x| x.universe_id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        if let Ok(v) = get_json(
            &format!("https://thumbnails.roblox.com/v1/games/icons?universeIds={ids}&size=150x150&format=Png&isCircular=false"),
            None,
        )
        .await
        {
            for d in v["data"].as_array().into_iter().flatten() {
                if let (Some(id), Some(url)) = (d["targetId"].as_u64(), d["imageUrl"].as_str()) {
                    if let Some(x) = games.iter_mut().find(|x| x.universe_id == id) {
                        x.icon = Some(url.to_string());
                    }
                }
            }
        }
    }
    let s = |k: &str| u[k].as_str().unwrap_or_default().to_string();
    Ok(Profile {
        user_id,
        name: s("name"),
        display_name: s("displayName"),
        description: s("description"),
        created: s("created"),
        verified: u["hasVerifiedBadge"].as_bool().unwrap_or(false),
        banned: u["isBanned"].as_bool().unwrap_or(false),
        friends,
        followers,
        following,
        avatar,
        status: here.as_ref().map(|p| p.user_presence_type),
        last_location: here
            .as_ref()
            .and_then(|p| p.last_location.clone())
            .filter(|l| !l.is_empty() && l != "Website"),
        place_id: here.as_ref().and_then(|p| p.place_id),
        game_id: here.and_then(|p| p.game_id),
        games,
    })
}
