use super::discover::{GameCard, with_icons};
use super::logwatch::{Event, logs_dir, parse_line};
use super::roblox::{get_json, post};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::Mutex,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Visit {
    pub user_id: u64,
    pub universe_id: u64,
    pub place_id: u64,
    pub start: u64,
    pub secs: u64,
}

#[derive(Serialize, Deserialize, Default)]
struct HistoryFile {
    files: HashMap<String, (u64, Vec<Visit>)>,
}

static LOCK: Mutex<()> = Mutex::new(());

pub fn parse_ts(line: &str) -> Option<u64> {
    let b = line.as_bytes();
    if b.len() < 19 || b[4] != b'-' || b[10] != b'T' {
        return None;
    }
    let n = |r: std::ops::Range<usize>| line.get(r)?.parse::<i64>().ok();
    let (y, m, d, hh, mm, ss) = (
        n(0..4)?,
        n(5..7)?,
        n(8..10)?,
        n(11..13)?,
        n(14..16)?,
        n(17..19)?,
    );
    let y2 = if m <= 2 { y - 1 } else { y };
    let era = y2.div_euclid(400);
    let yoe = y2 - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    u64::try_from(days * 86400 + hh * 3600 + mm * 60 + ss).ok()
}

pub fn visits_in(text: &str) -> Vec<Visit> {
    let mut out = vec![];
    let mut open: Option<Visit> = None;
    let mut last_ts = 0;
    for line in text.lines() {
        let Some(ts) = parse_ts(line) else { continue };
        last_ts = ts;
        match parse_line(line) {
            Some(Event::Joined {
                place_id,
                universe_id,
                user_id,
            }) => {
                if let Some(mut v) = open.take() {
                    v.secs = ts.saturating_sub(v.start);
                    out.push(v);
                }
                open = Some(Visit {
                    user_id,
                    universe_id,
                    place_id,
                    start: ts,
                    secs: 0,
                });
            }
            Some(Event::Left) => {
                if let Some(mut v) = open.take() {
                    v.secs = ts.saturating_sub(v.start);
                    out.push(v);
                }
            }
            _ => {}
        }
    }
    if let Some(mut v) = open {
        v.secs = last_ts.saturating_sub(v.start);
        out.push(v);
    }
    out
}

pub fn history() -> Vec<Visit> {
    let _g = LOCK.lock().unwrap();
    let path = super::paths::base().join("history.json");
    let mut h: HistoryFile = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let mut changed = false;
    if let Some(dir) = logs_dir() {
        for e in fs::read_dir(dir).into_iter().flatten().flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if !name.contains("_Player_")
                || name.contains("CrashHandler")
                || !name.ends_with(".log")
            {
                continue;
            }
            let len = e.metadata().map(|m| m.len()).unwrap_or(0);
            if h.files.get(&name).is_some_and(|(l, _)| *l == len) {
                continue;
            }
            let Ok(bytes) = fs::read(e.path()) else {
                continue;
            };
            h.files
                .insert(name, (len, visits_in(&String::from_utf8_lossy(&bytes))));
            changed = true;
        }
    }
    if changed {
        let _ = fs::create_dir_all(super::paths::base());
        let _ = fs::write(&path, serde_json::to_string(&h).unwrap());
    }
    h.files.into_values().flat_map(|(_, v)| v).collect()
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Played {
    pub game: GameCard,
    pub visits: u64,
    pub secs: u64,
    pub last_played: u64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Pick {
    pub game: GameCard,
    pub reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForYou {
    pub user_id: u64,
    pub jump_back_in: Vec<Played>,
    pub picks: Vec<Pick>,
    pub based_on: usize,
}

struct Info {
    card: GameCard,
    l1: String,
    l2: String,
}

async fn infos(ids: &[u64]) -> HashMap<u64, Info> {
    let mut out = HashMap::new();
    for chunk in ids.chunks(50) {
        let list = chunk
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let Ok(g) = get_json(
            &format!("https://games.roblox.com/v1/games?universeIds={list}"),
            None,
        )
        .await
        else {
            continue;
        };
        let votes = get_json(
            &format!("https://games.roblox.com/v1/games/votes?universeIds={list}"),
            None,
        )
        .await
        .unwrap_or_default();
        let vote = |id: u64| {
            votes["data"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|v| v["id"].as_u64() == Some(id))
                .map(|v| {
                    (
                        v["upVotes"].as_u64().unwrap_or(0),
                        v["downVotes"].as_u64().unwrap_or(0),
                    )
                })
                .unwrap_or((0, 0))
        };
        for d in g["data"].as_array().into_iter().flatten() {
            let Some(id) = d["id"].as_u64() else { continue };
            let (up, down) = vote(id);
            out.insert(
                id,
                Info {
                    card: GameCard {
                        universe_id: id,
                        root_place_id: d["rootPlaceId"].as_u64().unwrap_or(0),
                        name: d["name"].as_str().unwrap_or_default().to_string(),
                        player_count: d["playing"].as_u64().unwrap_or(0),
                        up_votes: up,
                        down_votes: down,
                        icon: None,
                    },
                    l1: d["genre_l1"].as_str().unwrap_or_default().to_string(),
                    l2: d["genre_l2"].as_str().unwrap_or_default().to_string(),
                },
            );
        }
    }
    out
}

async fn roblox_feed(cookie: &str) -> (Vec<u64>, Vec<u64>) {
    let body = json!({ "pageType": "Home", "sessionId": format!("{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()) });
    let Ok(r) = post(
        "https://apis.roblox.com/discovery-api/omni-recommendation",
        Some(cookie),
        body,
    )
    .await
    else {
        return (vec![], vec![]);
    };
    let Ok(v) = r.json::<Value>().await else {
        return (vec![], vec![]);
    };
    parse_feed(&v)
}

pub fn parse_feed(v: &Value) -> (Vec<u64>, Vec<u64>) {
    let (mut recs, mut cont) = (vec![], vec![]);
    for s in v["sorts"].as_array().into_iter().flatten() {
        let topic = s["topic"]
            .as_str()
            .or(s["topicId"].as_str())
            .unwrap_or_default()
            .to_lowercase();
        let ids = s["recommendationList"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|c| c["contentType"] == "Game")
            .filter_map(|c| c["contentId"].as_u64());
        if topic.contains("continue") {
            cont.extend(ids);
        } else {
            recs.extend(ids);
        }
    }
    (recs, cont)
}

fn affinity(mine: &[(&Visit, &Info)], now: u64) -> (HashMap<String, f64>, HashMap<String, f64>) {
    let (mut l1, mut l2) = (HashMap::new(), HashMap::new());
    for (v, i) in mine {
        let age_days = now.saturating_sub(v.start) as f64 / 86400.0;
        let w = (1.0 + v.secs as f64 / 60.0).ln().max(0.3) * 0.5f64.powf(age_days / 14.0);
        if !i.l2.is_empty() {
            *l2.entry(i.l2.clone()).or_insert(0.0) += w;
        }
        if !i.l1.is_empty() {
            *l1.entry(i.l1.clone()).or_insert(0.0) += w;
        }
    }
    (l1, l2)
}

pub async fn for_you(user_id: u64, cookie: Option<String>) -> Result<ForYou, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let visits: Vec<Visit> = tokio::task::spawn_blocking(history)
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|v| v.user_id == user_id)
        .collect();
    let (rec_ids, continue_ids) = match &cookie {
        Some(c) => roblox_feed(c).await,
        None => (vec![], vec![]),
    };

    let played_ids: HashSet<u64> = visits
        .iter()
        .map(|v| v.universe_id)
        .chain(continue_ids.iter().copied())
        .collect();
    let mut info = infos(&played_ids.iter().copied().collect::<Vec<_>>()).await;
    let (pre1, pre2) = affinity(
        &visits
            .iter()
            .filter_map(|v| Some((v, info.get(&v.universe_id)?)))
            .collect::<Vec<_>>(),
        now,
    );
    let mut top_genres: Vec<(String, f64)> = pre2
        .into_iter()
        .chain(pre1.into_iter().map(|(g, w)| (g, w * 0.45)))
        .collect();
    top_genres.sort_by(|a, b| b.1.total_cmp(&a.1));

    let charts = super::discover::charts(30).await.unwrap_or_default();
    let mut pool: Vec<u64> = rec_ids.clone();
    for (genre, _) in top_genres.iter().take(3) {
        pool.extend(
            super::discover::search(genre)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|g| g.universe_id),
        );
    }
    pool.extend(
        charts
            .iter()
            .flat_map(|c| c.games.iter().map(|g| g.universe_id)),
    );
    let mut want: Vec<u64> = pool
        .iter()
        .filter(|id| !info.contains_key(id))
        .copied()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    want.sort();
    info.extend(infos(&want).await);

    let mut per: HashMap<u64, (u64, u64, u64)> = HashMap::new();
    for v in &visits {
        let e = per.entry(v.universe_id).or_insert((0, 0, 0));
        e.0 += 1;
        e.1 += v.secs;
        e.2 = e.2.max(v.start + v.secs);
    }
    for id in &continue_ids {
        per.entry(*id).or_insert((0, 0, 0));
    }
    let mut jump: Vec<Played> = per
        .iter()
        .filter_map(|(id, (n, s, l))| {
            Some(Played {
                game: info.get(id)?.card.clone(),
                visits: *n,
                secs: *s,
                last_played: *l,
            })
        })
        .collect();
    jump.sort_by(|a, b| b.last_played.cmp(&a.last_played).then(b.secs.cmp(&a.secs)));
    jump.truncate(12);

    let mine: Vec<(&Visit, &Info)> = visits
        .iter()
        .filter_map(|v| Some((v, info.get(&v.universe_id)?)))
        .collect();
    let (a1, a2) = affinity(&mine, now);
    let top = a2
        .values()
        .chain(a1.values())
        .fold(0.0f64, |m, x| m.max(*x))
        .max(1e-9);
    let fav_in = |genre: &str, l2: bool| -> Option<String> {
        mine.iter()
            .filter(|(_, i)| if l2 { i.l2 == genre } else { i.l1 == genre })
            .max_by_key(|(v, _)| v.secs)
            .map(|(_, i)| i.card.name.clone())
    };
    let recs: HashSet<u64> = rec_ids.iter().copied().collect();
    let mut seen = HashSet::new();
    let mut scored: Vec<(f64, Pick, String)> = pool
        .iter()
        .filter(|id| !played_ids.contains(id) && seen.insert(**id))
        .filter_map(|id| {
            let i = info.get(id)?;
            if i.card.player_count < 50 && !recs.contains(id) {
                return None;
            }
            let g2 = a2.get(&i.l2).copied().unwrap_or(0.0) / top;
            let g1 = a1.get(&i.l1).copied().unwrap_or(0.0) / top;
            let pop = ((i.card.player_count as f64 + 1.0).log10() / 6.0).min(1.0);
            let votes = (i.card.up_votes + i.card.down_votes) as f64;
            let rating = if votes > 0.0 {
                i.card.up_votes as f64 / votes
            } else {
                0.5
            };
            let score = g2 * 1.0
                + g1 * 0.45
                + pop * 0.25
                + rating * 0.15
                + if recs.contains(id) { 0.3 } else { 0.0 };
            let reason = if g2 > 0.05 {
                fav_in(&i.l2, true).map(|n| format!("Because you play {n}"))
            } else {
                None
            }
            .or_else(|| {
                if g1 > 0.05 {
                    Some(format!("You like {} games", i.l1))
                } else {
                    None
                }
            })
            .or_else(|| {
                recs.contains(id)
                    .then(|| "Roblox recommends it for you".to_string())
            })
            .unwrap_or_else(|| "Popular right now".to_string());
            Some((
                score,
                Pick {
                    game: i.card.clone(),
                    reason,
                },
                i.l2.clone(),
            ))
        })
        .collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    let fav = top_genres
        .first()
        .map(|(g, _)| g.clone())
        .unwrap_or_default();
    let mut per_genre: HashMap<String, u32> = HashMap::new();
    let mut picks: Vec<Pick> = scored
        .into_iter()
        .filter(|(_, _, g)| {
            let n = per_genre.entry(g.clone()).or_insert(0);
            *n += 1;
            *n <= if *g == fav { 6 } else { 3 }
        })
        .map(|(_, p, _)| p)
        .take(18)
        .collect();

    let icons = with_icons(
        jump.iter()
            .map(|p| p.game.clone())
            .chain(picks.iter().map(|p| p.game.clone()))
            .collect(),
    )
    .await;
    let icon = |id: u64| {
        icons
            .iter()
            .find(|g| g.universe_id == id)
            .and_then(|g| g.icon.clone())
    };
    for p in &mut jump {
        p.game.icon = icon(p.game.universe_id);
    }
    for p in &mut picks {
        p.game.icon = icon(p.game.universe_id);
    }
    Ok(ForYou {
        user_id,
        jump_back_in: jump,
        picks,
        based_on: per.len(),
    })
}

pub fn local_players() -> Vec<u64> {
    let mut last: HashMap<u64, u64> = HashMap::new();
    for v in history() {
        let e = last.entry(v.user_id).or_insert(0);
        *e = (*e).max(v.start);
    }
    let mut ids: Vec<(u64, u64)> = last.into_iter().collect();
    ids.sort_by(|a, b| b.1.cmp(&a.1));
    ids.into_iter().map(|(u, _)| u).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_timestamps() {
        assert_eq!(parse_ts("1970-01-01T00:00:00.000Z,0.1"), Some(0));
        assert_eq!(
            parse_ts("2026-11-07T20:39:31.748Z,1.7,9ac4,6 [x]"),
            Some(1794083971)
        );
        assert_eq!(parse_ts("[FLog::Output] All use of Roblox"), None);
    }

    #[test]
    fn summarizes_visits() {
        let log = "\
2026-11-07T20:39:31.748Z,1,a,6 [FLog::GameJoinLoadTime] Report game_join_loadtime: placeid:10, join_time:0.6, universeid:100, sid:x, userid:7, \n\
2026-11-07T20:41:07.762Z,9,a,6,Info [DFLog::RbxTransportDummyClient] Disconnected from server for reason: Player: 285\n\
2026-11-07T20:41:07.904Z,9,a,6 [FLog::GameJoinLoadTime] Report game_join_loadtime: placeid:20, join_time:0, universeid:200, sid:y, userid:7, \n\
2026-11-07T21:00:00.000Z,9,a,6 [FLog::Network] something later\n";
        let v = visits_in(log);
        assert_eq!(v.len(), 2);
        assert_eq!((v[0].universe_id, v[0].secs), (100, 96));
        assert_eq!((v[1].universe_id, v[1].secs), (200, 1133));
    }

    #[test]
    fn splits_continue_from_recommendations() {
        let feed = json!({ "sorts": [
            { "topic": "Continue", "recommendationList": [{ "contentType": "Game", "contentId": 1 }] },
            { "topic": "Recommended For You", "recommendationList": [{ "contentType": "Game", "contentId": 2 }, { "contentType": "Ad", "contentId": 3 }] },
        ] });
        assert_eq!(parse_feed(&feed), (vec![2], vec![1]));
    }
}

#[cfg(test)]
mod real {
    #[test]
    #[ignore]
    fn real_for_you() {
        let players = super::local_players();
        println!("players seen in logs: {}", players.len());
        let Some(id) = players.first().copied() else {
            return;
        };
        let r = tauri::async_runtime::block_on(super::for_you(id, None)).unwrap();
        println!("based on {} played games", r.based_on);
        for p in &r.jump_back_in {
            println!(
                "  jump back in: {} ({} visits, {} min)",
                p.game.name,
                p.visits,
                p.secs / 60
            );
        }
        for p in &r.picks {
            println!("  pick: {} -- {}", p.game.name, p.reason);
        }
        assert!(!r.picks.is_empty());
    }
}
