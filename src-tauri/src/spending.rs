use super::roblox::get_json;
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

const PAGES: usize = 10;
const CACHE_TTL: Duration = Duration::from_secs(10 * 60);
const MONTHS: usize = 12;

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Purchase {
    pub at: u64,
    pub robux: u64,
    pub name: String,
    pub kind: String,
    pub universe_id: Option<u64>,
    pub game: Option<String>,
    pub seller: String,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub key: String,
    pub universe_id: Option<u64>,
    pub robux: u64,
    pub count: u64,
    pub icon: Option<String>,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Spending {
    pub total: u64,
    pub count: u64,
    pub this_month: u64,
    pub months: Vec<(String, u64)>,
    pub kinds: Vec<Group>,
    pub games: Vec<Group>,
    pub biggest: Vec<Purchase>,
    pub recent: Vec<Purchase>,
    pub since: Option<u64>,
    pub partial: bool,
}

fn kind_of(t: &str) -> &'static str {
    match t {
        "Asset" | "Bundle" => "Avatar items",
        "GamePass" => "Game passes",
        "DeveloperProduct" => "In-game purchases",
        "PrivateServer" | "PrivateServerProduct" => "Private servers",
        _ => "Other",
    }
}

pub fn parse_page(v: &Value) -> (Vec<Purchase>, Option<String>) {
    let list = v["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|t| t["currency"]["type"].as_str().is_none_or(|c| c == "Robux"))
        .filter_map(|t| {
            let robux = t["currency"]["amount"].as_i64()?.unsigned_abs();
            let d = &t["details"];
            Some(Purchase {
                at: super::personal::parse_ts(t["created"].as_str()?)?,
                robux,
                name: d["name"].as_str().unwrap_or("Unnamed item").to_string(),
                kind: kind_of(d["type"].as_str().unwrap_or_default()).to_string(),
                universe_id: d["place"]["universeId"].as_u64(),
                game: d["place"]["name"].as_str().map(str::to_string),
                seller: t["agent"]["name"].as_str().unwrap_or_default().to_string(),
            })
        })
        .collect();
    let next = v["nextPageCursor"]
        .as_str()
        .filter(|c| !c.is_empty())
        .map(str::to_string);
    (list, next)
}

fn year_month(ts: u64, tz: i64) -> (i64, u32) {
    let days = (ts as i64 + tz).div_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m)
}

fn group(list: &[&Purchase], key: impl Fn(&Purchase) -> (String, Option<u64>)) -> Vec<Group> {
    let mut map: HashMap<String, Group> = HashMap::new();
    for p in list {
        let (k, id) = key(p);
        let g = map.entry(k.clone()).or_insert(Group {
            key: k,
            universe_id: id,
            robux: 0,
            count: 0,
            icon: None,
        });
        g.robux += p.robux;
        g.count += 1;
    }
    let mut out: Vec<Group> = map.into_values().collect();
    out.sort_by(|a, b| b.robux.cmp(&a.robux).then(b.count.cmp(&a.count)));
    out
}

pub fn summarize(purchases: &[Purchase], now: u64, tz: i64, partial: bool) -> Spending {
    let spent: Vec<&Purchase> = purchases.iter().filter(|p| p.robux > 0).collect();
    let (cy, cm) = year_month(now, tz);
    let mut months: Vec<(String, u64)> = (0..MONTHS)
        .rev()
        .map(|back| {
            let total = cy * 12 + i64::from(cm) - 1 - back as i64;
            (
                format!("{}-{:02}", total.div_euclid(12), total.rem_euclid(12) + 1),
                0,
            )
        })
        .collect();
    let mut s = Spending {
        partial,
        ..Default::default()
    };
    for p in &spent {
        s.total += p.robux;
        s.count += 1;
        s.since = Some(s.since.map_or(p.at, |x| x.min(p.at)));
        let (y, m) = year_month(p.at, tz);
        let key = format!("{y}-{m:02}");
        if let Some(slot) = months.iter_mut().find(|(k, _)| *k == key) {
            slot.1 += p.robux;
        }
        if (y, m) == (cy, cm) {
            s.this_month += p.robux;
        }
    }
    s.months = std::mem::take(&mut months);
    s.kinds = group(&spent, |p| (p.kind.clone(), None));
    s.games = group(
        &spent
            .iter()
            .copied()
            .filter(|p| p.universe_id.is_some())
            .collect::<Vec<_>>(),
        |p| {
            (
                p.game.clone().unwrap_or_else(|| "A game".into()),
                p.universe_id,
            )
        },
    );
    s.games.truncate(10);
    let mut biggest: Vec<Purchase> = spent.iter().map(|p| (*p).clone()).collect();
    biggest.sort_by(|a, b| b.robux.cmp(&a.robux));
    biggest.truncate(8);
    s.biggest = biggest;
    let mut recent: Vec<Purchase> = spent.iter().map(|p| (*p).clone()).collect();
    recent.sort_by(|a, b| b.at.cmp(&a.at));
    recent.truncate(20);
    s.recent = recent;
    s
}

static CACHE: LazyLock<Mutex<HashMap<u64, (Instant, Vec<Purchase>, bool)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

async fn fetch(user_id: u64, cookie: &str) -> Result<(Vec<Purchase>, bool), String> {
    let bases = [
        format!(
            "https://apis.roblox.com/transaction-records/v1/users/{user_id}/transactions?limit=100&transactionType=Purchase&itemPricingType=All"
        ),
        format!(
            "https://economy.roblox.com/v2/users/{user_id}/transactions?limit=100&transactionType=Purchase"
        ),
    ];
    let mut last_err = String::new();
    for base in &bases {
        let mut all = vec![];
        let mut cursor: Option<String> = None;
        let mut pages = 0;
        let mut failed = false;
        loop {
            let url = match &cursor {
                Some(c) => format!("{base}&cursor={}", urlencoding::encode(c)),
                None => base.clone(),
            };
            match get_json(&url, Some(cookie)).await {
                Ok(v) => {
                    let (list, next) = parse_page(&v);
                    all.extend(list);
                    pages += 1;
                    cursor = next;
                    if cursor.is_none() || pages >= PAGES {
                        break;
                    }
                }
                Err(e) if pages == 0 => {
                    last_err = e;
                    failed = true;
                    break;
                }
                Err(_) => break,
            }
        }
        if !failed {
            return Ok((all, cursor.is_some()));
        }
    }
    Err(format!("Couldn't load purchases from Roblox: {last_err}"))
}

pub async fn load(user_id: u64, cookie: &str, now: u64, tz: i64) -> Result<Spending, String> {
    let cached = CACHE
        .lock()
        .unwrap()
        .get(&user_id)
        .filter(|(at, _, _)| at.elapsed() < CACHE_TTL)
        .map(|(_, list, partial)| (list.clone(), *partial));
    let (list, partial) = match cached {
        Some(c) => c,
        None => {
            let fresh = fetch(user_id, cookie).await?;
            CACHE
                .lock()
                .unwrap()
                .insert(user_id, (Instant::now(), fresh.0.clone(), fresh.1));
            fresh
        }
    };
    let mut s = summarize(&list, now, tz, partial);
    let ids: Vec<u64> = s.games.iter().filter_map(|g| g.universe_id).collect();
    if !ids.is_empty() {
        let list = ids.iter().map(u64::to_string).collect::<Vec<_>>().join(",");
        if let Ok(v) = get_json(
            &format!("https://thumbnails.roblox.com/v1/games/icons?universeIds={list}&size=150x150&format=Png&isCircular=false"),
            None,
        )
        .await
        {
            for d in v["data"].as_array().into_iter().flatten() {
                let (Some(id), Some(url)) = (d["targetId"].as_u64(), d["imageUrl"].as_str()) else { continue };
                if let Some(g) = s.games.iter_mut().find(|g| g.universe_id == Some(id)) {
                    g.icon = Some(url.to_string());
                }
            }
        }
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn page() -> Value {
        json!({
            "nextPageCursor": "abc",
            "data": [
                {
                    "created": "2026-11-05T12:00:00.000Z",
                    "agent": { "id": 1, "type": "Group", "name": "Grandma's Favourite Games" },
                    "details": { "id": 9, "name": "Diamond Hunter Pass", "type": "GamePass", "place": { "placeId": 79546208627805u64, "universeId": 7326934954u64, "name": "99 Nights in the Forest" } },
                    "currency": { "amount": -399, "type": "Robux" }
                },
                {
                    "created": "2026-10-20T09:30:00.000Z",
                    "agent": { "id": 2, "type": "User", "name": "Roblox" },
                    "details": { "id": 1365767, "name": "Valkyrie Helm", "type": "Asset" },
                    "currency": { "amount": -50000, "type": "Robux" }
                },
                {
                    "created": "2026-11-06T18:00:00.000Z",
                    "agent": { "id": 1, "type": "Group", "name": "Grandma's Favourite Games" },
                    "details": { "id": 10, "name": "Medic Class", "type": "DeveloperProduct", "place": { "placeId": 79546208627805u64, "universeId": 7326934954u64, "name": "99 Nights in the Forest" } },
                    "currency": { "amount": -100, "type": "Robux" }
                },
                { "created": "bad", "details": {}, "currency": { "amount": -5, "type": "Robux" } }
            ]
        })
    }

    #[test]
    fn parses_transaction_pages() {
        let (list, next) = parse_page(&page());
        assert_eq!(next.as_deref(), Some("abc"));
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].robux, 399);
        assert_eq!(list[0].kind, "Game passes");
        assert_eq!(list[0].universe_id, Some(7326934954));
        assert_eq!(list[1].kind, "Avatar items");
        assert_eq!(list[2].kind, "In-game purchases");
        assert_eq!(
            parse_page(&json!({ "data": [], "nextPageCursor": null })).1,
            None
        );
    }

    #[test]
    fn months_and_groups() {
        assert_eq!(year_month(0, 0), (1970, 1));
        assert_eq!(
            year_month(
                super::super::personal::parse_ts("2026-02-28T23:30:00").unwrap(),
                3600
            ),
            (2026, 3)
        );
        assert_eq!(
            year_month(
                super::super::personal::parse_ts("2024-12-31T12:00:00").unwrap(),
                0
            ),
            (2024, 12)
        );
        let (list, _) = parse_page(&page());
        let now = super::super::personal::parse_ts("2026-11-08T10:00:00").unwrap();
        let s = summarize(&list, now, 0, true);
        assert_eq!(s.total, 50_499);
        assert_eq!(s.count, 3);
        assert_eq!(s.this_month, 499);
        assert_eq!(s.months.len(), 12);
        assert_eq!(s.months[11], ("2026-11".to_string(), 499));
        assert_eq!(s.months[10], ("2026-10".to_string(), 50_000));
        assert_eq!(s.months[0].0, "2025-12");
        assert_eq!(s.kinds[0].key, "Avatar items");
        assert_eq!(s.games.len(), 1);
        assert_eq!((s.games[0].robux, s.games[0].count), (499, 2));
        assert_eq!(s.biggest[0].name, "Valkyrie Helm");
        assert_eq!(s.recent[0].name, "Medic Class");
        assert!(s.partial);
    }
}
