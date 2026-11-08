use super::catalog::thumbs;
use super::roblox::{get_json, headshots, post};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ItemValue {
    pub name: String,
    pub acronym: String,
    pub rap: u64,
    pub value: Option<u64>,
    pub shown: u64,
    pub demand: i8,
    pub trend: i8,
    pub projected: bool,
    pub hyped: bool,
    pub rare: bool,
}

pub fn parse_values(v: &Value) -> HashMap<u64, ItemValue> {
    let mut out = HashMap::new();
    for (id, a) in v["items"].as_object().into_iter().flatten() {
        let (Ok(id), Some(a)) = (id.parse::<u64>(), a.as_array()) else {
            continue;
        };
        let n = |i: usize| a.get(i).and_then(Value::as_i64).unwrap_or(-1);
        let rap = n(2).max(0) as u64;
        let value = u64::try_from(n(3)).ok();
        out.insert(
            id,
            ItemValue {
                name: a
                    .first()
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                acronym: a
                    .get(1)
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                rap,
                value,
                shown: value.unwrap_or(rap),
                demand: n(5) as i8,
                trend: n(6) as i8,
                projected: n(7) == 1,
                hyped: n(8) == 1,
                rare: n(9) == 1,
            },
        );
    }
    out
}

static VALUES: Mutex<Option<(Instant, Arc<HashMap<u64, ItemValue>>)>> = Mutex::new(None);

pub async fn values() -> Result<Arc<HashMap<u64, ItemValue>>, String> {
    let cached = VALUES.lock().unwrap().clone();
    if let Some((at, v)) = &cached {
        if at.elapsed() < Duration::from_secs(300) {
            return Ok(v.clone());
        }
    }
    match get_json("https://api.rolimons.com/items/v2/itemdetails", None).await {
        Ok(v) => {
            let fresh = Arc::new(parse_values(&v));
            *VALUES.lock().unwrap() = Some((Instant::now(), fresh.clone()));
            Ok(fresh)
        }
        Err(e) => cached
            .map(|(_, v)| v)
            .ok_or(format!("Couldn't load item values from Rolimons: {e}")),
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Limited {
    pub user_asset_id: u64,
    pub asset_id: u64,
    pub name: String,
    pub serial: Option<u64>,
    pub rap: u64,
    pub info: Option<ItemValue>,
    pub shown: u64,
    pub thumb: Option<String>,
}

fn limited(a: &Value, vals: &HashMap<u64, ItemValue>) -> Option<Limited> {
    let asset_id = a["assetId"].as_u64().or(a["itemTarget"]["targetId"]
        .as_str()
        .and_then(|s| s.parse().ok()))?;
    let info = vals.get(&asset_id).cloned();
    let rap = a["recentAveragePrice"]
        .as_u64()
        .or(info.as_ref().map(|i| i.rap))
        .unwrap_or(0);
    Some(Limited {
        user_asset_id: a["id"].as_u64().or(a["userAssetId"].as_u64()).unwrap_or(0),
        asset_id,
        name: a["name"]
            .as_str()
            .or(a["itemName"].as_str())
            .map(str::to_string)
            .or(info.as_ref().map(|i| i.name.clone()))
            .unwrap_or_default(),
        serial: a["serialNumber"].as_u64(),
        rap,
        shown: info.as_ref().map(|i| i.shown).unwrap_or(rap),
        info,
        thumb: None,
    })
}

async fn with_thumbs(mut items: Vec<Limited>) -> Vec<Limited> {
    let t = thumbs(
        &items
            .iter()
            .map(|i| ("Asset".to_string(), i.asset_id))
            .collect::<Vec<_>>(),
    )
    .await;
    for i in &mut items {
        i.thumb = t.get(&("Asset".to_string(), i.asset_id)).cloned();
    }
    items
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub items: Vec<Limited>,
    pub total_rap: u64,
    pub total_value: u64,
}

pub async fn collection(user_id: u64, cookie: Option<&str>) -> Result<Collection, String> {
    let vals = values().await.unwrap_or_default();
    let mut items = vec![];
    let mut cursor = String::new();
    for _ in 0..10 {
        let url = format!(
            "https://inventory.roblox.com/v1/users/{user_id}/assets/collectibles?limit=100&sortOrder=Desc&cursor={}",
            urlencoding::encode(&cursor)
        );
        let v = get_json(&url, cookie).await.map_err(|e| {
            if e.contains("403") {
                "This account's inventory is private.".to_string()
            } else {
                e
            }
        })?;
        items.extend(
            v["data"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|a| limited(a, &vals)),
        );
        match v["nextPageCursor"].as_str() {
            Some(c) if !c.is_empty() => cursor = c.to_string(),
            _ => break,
        }
    }
    items.sort_by(|a, b| b.shown.cmp(&a.shown));
    let items = with_thumbs(items).await;
    Ok(Collection {
        total_rap: items.iter().map(|i| i.rap).sum(),
        total_value: items.iter().map(|i| i.shown).sum(),
        items,
    })
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Partner {
    pub id: u64,
    pub name: String,
    pub display_name: String,
    pub avatar: Option<String>,
}

fn partner(u: &Value) -> Partner {
    Partner {
        id: u["id"].as_u64().unwrap_or(0),
        name: u["name"].as_str().unwrap_or_default().to_string(),
        display_name: u["displayName"]
            .as_str()
            .or(u["name"].as_str())
            .unwrap_or_default()
            .to_string(),
        avatar: None,
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeSummary {
    pub id: u64,
    pub partner: Partner,
    pub created: String,
    pub expiration: Option<String>,
    pub status: String,
    pub is_active: bool,
}

pub async fn list(cookie: &str, kind: &str) -> Result<Vec<TradeSummary>, String> {
    if !["inbound", "outbound", "completed", "inactive"].contains(&kind) {
        return Err("Unknown trade list.".into());
    }
    let v = get_json(
        &format!("https://trades.roblox.com/v1/trades/{kind}?limit=50&sortOrder=Desc"),
        Some(cookie),
    )
    .await?;
    let mut out: Vec<TradeSummary> = v["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|t| {
            Some(TradeSummary {
                id: t["id"].as_u64()?,
                partner: partner(&t["user"]),
                created: t["created"].as_str().unwrap_or_default().to_string(),
                expiration: t["expiration"].as_str().map(str::to_string),
                status: t["status"].as_str().unwrap_or_default().to_string(),
                is_active: t["isActive"].as_bool().unwrap_or(false),
            })
        })
        .collect();
    let ids: Vec<u64> = out.iter().map(|t| t.partner.id).collect();
    let avatars = if ids.is_empty() {
        HashMap::new()
    } else {
        headshots(&ids).await
    };
    for t in &mut out {
        t.partner.avatar = avatars.get(&t.partner.id).cloned();
    }
    Ok(out)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Side {
    pub user: Partner,
    pub items: Vec<Limited>,
    pub robux: u64,
    pub rap: u64,
    pub value: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeDetail {
    pub id: u64,
    pub status: String,
    pub is_active: bool,
    pub give: Side,
    pub get: Side,
    pub net_value: i64,
    pub net_rap: i64,
    pub warnings: Vec<String>,
}

fn side(o: &Value, vals: &HashMap<u64, ItemValue>) -> Side {
    let items: Vec<Limited> = o["userAssets"]
        .as_array()
        .or(o["items"].as_array())
        .into_iter()
        .flatten()
        .filter_map(|a| limited(a, vals))
        .collect();
    let robux = o["robux"].as_u64().unwrap_or(0);
    Side {
        user: partner(&o["user"]),
        rap: items.iter().map(|i| i.rap).sum::<u64>() + robux,
        value: items.iter().map(|i| i.shown).sum::<u64>() + robux,
        items,
        robux,
    }
}

pub fn warnings(give: &Side, get: &Side) -> Vec<String> {
    let mut w = vec![];
    for i in &get.items {
        let Some(info) = &i.info else { continue };
        if info.projected {
            w.push(format!(
                "{} is projected: its price was pushed up and isn't trusted.",
                i.name
            ));
        }
        if (0..=1).contains(&info.demand) {
            w.push(format!(
                "{} has low demand and may be hard to trade away.",
                i.name
            ));
        }
    }
    for i in &give.items {
        if i.info.as_ref().is_some_and(|x| x.rare) {
            w.push(format!("You're giving away {}, which is rare.", i.name));
        }
    }
    if get.value < give.value {
        w.push(format!(
            "You lose {} in value on this trade.",
            give.value - get.value
        ));
    }
    w
}

pub async fn detail(user_id: u64, cookie: &str, trade_id: u64) -> Result<TradeDetail, String> {
    let vals = values().await.unwrap_or_default();
    let v = get_json(
        &format!("https://trades.roblox.com/v1/trades/{trade_id}"),
        Some(cookie),
    )
    .await?;
    let offers: Vec<&Value> = v["offers"].as_array().into_iter().flatten().collect();
    let (mine, theirs) = match offers
        .iter()
        .position(|o| o["user"]["id"].as_u64() == Some(user_id))
    {
        Some(0) | None => (offers.first(), offers.get(1)),
        Some(_) => (offers.get(1), offers.first()),
    };
    let mut give = side(mine.copied().unwrap_or(&Value::Null), &vals);
    let mut get = side(theirs.copied().unwrap_or(&Value::Null), &vals);
    give.items = with_thumbs(give.items).await;
    get.items = with_thumbs(get.items).await;
    if let Some(a) = headshots(&[get.user.id]).await.remove(&get.user.id) {
        get.user.avatar = Some(a);
    }
    Ok(TradeDetail {
        id: trade_id,
        status: v["status"].as_str().unwrap_or_default().to_string(),
        is_active: v["isActive"].as_bool().unwrap_or(false),
        net_value: get.value as i64 - give.value as i64,
        net_rap: get.rap as i64 - give.rap as i64,
        warnings: warnings(&give, &get),
        give,
        get,
    })
}

pub async fn decline(cookie: &str, trade_id: u64) -> Result<(), String> {
    let r = post(
        &format!("https://trades.roblox.com/v1/trades/{trade_id}/decline"),
        Some(cookie),
        json!({}),
    )
    .await?;
    if r.status().is_success() {
        Ok(())
    } else {
        Err(format!("Roblox didn't decline the trade ({}).", r.status()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str, shown: u64, info: Option<ItemValue>) -> Limited {
        Limited {
            user_asset_id: 0,
            asset_id: 0,
            name: name.into(),
            serial: None,
            rap: shown,
            shown,
            info,
            thumb: None,
        }
    }
    fn val(demand: i8, projected: bool, rare: bool) -> ItemValue {
        ItemValue {
            name: String::new(),
            acronym: String::new(),
            rap: 0,
            value: None,
            shown: 0,
            demand,
            trend: -1,
            projected,
            hyped: false,
            rare,
        }
    }
    fn side_of(items: Vec<Limited>) -> Side {
        let value = items.iter().map(|i| i.shown).sum();
        Side {
            user: partner(&Value::Null),
            rap: value,
            value,
            items,
            robux: 0,
        }
    }

    #[test]
    fn parses_rolimons_rows() {
        let v = json!({ "items": {
            "1029025": ["The Classic ROBLOX Fedora", "CF", 431023, 450000, 450000, 4, 2, -1, -1, -1, 1],
            "1028606": ["Red Baseball Cap", "", 1438, -1, 1438, -1, -1, -1, -1, -1, 1],
            "5": ["Pumped", "", 55131, -1, 55131, 1, 0, 1, -1, 1, 1]
        } });
        let m = parse_values(&v);
        assert_eq!(
            (m[&1029025].value, m[&1029025].shown, m[&1029025].demand),
            (Some(450000), 450000, 4)
        );
        assert_eq!((m[&1028606].value, m[&1028606].shown), (None, 1438));
        assert!(m[&5].projected && m[&5].rare && !m[&5].hyped);
    }

    #[test]
    fn warns_about_bad_trades() {
        let give = side_of(vec![item("Rare hat", 900, Some(val(3, false, true)))]);
        let get = side_of(vec![item("Pumped", 500, Some(val(1, true, false)))]);
        let w = warnings(&give, &get);
        assert_eq!(w.len(), 4, "{w:?}");
        assert!(
            w.iter().any(|x| x.contains("projected")) && w.iter().any(|x| x.contains("lose 400"))
        );
        let fair = warnings(
            &side_of(vec![item("A", 100, None)]),
            &side_of(vec![item("B", 150, Some(val(3, false, false)))]),
        );
        assert!(fair.is_empty());
    }
}

#[cfg(test)]
mod real {
    #[test]
    #[ignore]
    fn real_collection() {
        let c = tauri::async_runtime::block_on(super::collection(1, None)).unwrap();
        println!(
            "items {} rap {} value {}",
            c.items.len(),
            c.total_rap,
            c.total_value
        );
        for i in c.items.iter().take(5) {
            println!(
                "  {} rap {} shown {} demand {:?} thumb {}",
                i.name,
                i.rap,
                i.shown,
                i.info.as_ref().map(|x| x.demand),
                i.thumb.is_some()
            );
        }
        assert!(!c.items.is_empty());
    }
}
