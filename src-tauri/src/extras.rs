use super::roblox::{Target, get_json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fs, sync::Mutex};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub awarded: u64,
    pub win_rate: f64,
    pub icon: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pass {
    pub id: u64,
    pub name: String,
    pub price: Option<u64>,
    pub icon: Option<String>,
}

async fn icons(kind: &str, param: &str, ids: &[u64]) -> HashMap<u64, String> {
    if ids.is_empty() {
        return HashMap::new();
    }
    let list = ids.iter().map(u64::to_string).collect::<Vec<_>>().join(",");
    let url = format!(
        "https://thumbnails.roblox.com/v1/{kind}?{param}={list}&size=150x150&format=Png&isCircular=false"
    );
    let Ok(v) = get_json(&url, None).await else {
        return HashMap::new();
    };
    v["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|d| Some((d["targetId"].as_u64()?, d["imageUrl"].as_str()?.to_string())))
        .collect()
}

pub async fn badges(universe_id: u64) -> Result<Vec<Badge>, String> {
    let v = get_json(
        &format!(
            "https://badges.roblox.com/v1/universes/{universe_id}/badges?limit=100&sortOrder=Asc"
        ),
        None,
    )
    .await?;
    let mut list: Vec<Badge> = v["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|b| b["enabled"].as_bool().unwrap_or(true))
        .filter_map(|b| {
            Some(Badge {
                id: b["id"].as_u64()?,
                name: b["displayName"]
                    .as_str()
                    .or(b["name"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                description: b["displayDescription"]
                    .as_str()
                    .or(b["description"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                awarded: b["statistics"]["awardedCount"].as_u64().unwrap_or(0),
                win_rate: b["statistics"]["winRatePercentage"].as_f64().unwrap_or(0.0),
                icon: None,
            })
        })
        .collect();
    let ids: Vec<u64> = list.iter().map(|b| b.id).take(100).collect();
    let i = icons("badges/icons", "badgeIds", &ids).await;
    for b in &mut list {
        b.icon = i.get(&b.id).cloned();
    }
    Ok(list)
}

pub async fn passes(universe_id: u64) -> Result<Vec<Pass>, String> {
    let v = get_json(
        &format!("https://apis.roblox.com/game-passes/v1/universes/{universe_id}/game-passes?passView=Full&pageSize=50"),
        None,
    )
    .await?;
    let mut list: Vec<Pass> = v["gamePasses"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| {
            Some(Pass {
                id: p["id"].as_u64()?,
                name: p["displayName"]
                    .as_str()
                    .or(p["name"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                price: if p["isForSale"].as_bool().unwrap_or(false) {
                    p["price"].as_u64()
                } else {
                    None
                },
                icon: None,
            })
        })
        .collect();
    let ids: Vec<u64> = list.iter().map(|p| p.id).collect();
    let i = icons("game-passes", "gamePassIds", &ids).await;
    for p in &mut list {
        p.icon = i.get(&p.id).cloned();
    }
    Ok(list)
}

const ALLOWLIST_URL: &str =
    "https://raw.githubusercontent.com/Cloaxyyy/umbra/main/flags/allowlist.json";

fn valid_flags(v: &Value) -> Vec<String> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|f| {
            !f.is_empty()
                && f.len() <= 120
                && f.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
        .take(1000)
        .map(String::from)
        .collect()
}

pub async fn flag_allowlist() -> Result<Vec<String>, String> {
    let list = valid_flags(&get_json(ALLOWLIST_URL, None).await?);
    if list.is_empty() {
        return Err("The flag allow-list was empty.".into());
    }
    Ok(list)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub account_ids: Vec<u64>,
    pub target: Target,
    #[serde(default)]
    pub profile_id: Option<String>,
}

static LOCK: Mutex<()> = Mutex::new(());

fn file() -> std::path::PathBuf {
    super::paths::base().join("presets.json")
}

fn read() -> Vec<Preset> {
    fs::read_to_string(file())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write(list: &[Preset]) -> Result<(), String> {
    fs::create_dir_all(super::paths::base()).map_err(|e| e.to_string())?;
    fs::write(file(), serde_json::to_string_pretty(list).unwrap()).map_err(|e| e.to_string())
}

pub fn presets() -> Vec<Preset> {
    let _g = LOCK.lock().unwrap();
    read()
}

pub fn save_preset(mut p: Preset) -> Result<Preset, String> {
    let name = p.name.trim();
    if name.is_empty() || name.chars().count() > 40 {
        return Err("Give the preset a name of 1 to 40 characters.".into());
    }
    if p.account_ids.is_empty() {
        return Err("Pick at least one account for the preset.".into());
    }
    p.name = name.to_string();
    let _g = LOCK.lock().unwrap();
    let mut all = read();
    if p.id.is_empty() {
        p.id = format!(
            "preset-{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                % 0xffffff
        );
        all.push(p.clone());
    } else {
        let slot = all
            .iter_mut()
            .find(|x| x.id == p.id)
            .ok_or("That preset no longer exists.")?;
        *slot = p.clone();
    }
    write(&all)?;
    Ok(p)
}

pub fn delete_preset(id: &str) -> Result<(), String> {
    let _g = LOCK.lock().unwrap();
    let mut all = read();
    all.retain(|x| x.id != id);
    write(&all)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_keeps_only_flag_names() {
        let v = serde_json::json!([
            "FIntDebugForceMSAASamples",
            "",
            "bad name",
            5,
            "DFFlagDisableDPIScale"
        ]);
        assert_eq!(
            valid_flags(&v),
            vec!["FIntDebugForceMSAASamples", "DFFlagDisableDPIScale"]
        );
        assert!(valid_flags(&serde_json::json!({"a": 1})).is_empty());
        let shipped: Value =
            serde_json::from_str(include_str!("../../flags/allowlist.json")).unwrap();
        assert!(valid_flags(&shipped).len() > 10);
    }

    #[test]
    fn preset_round_trips_with_target() {
        let p = Preset {
            id: "preset-1".into(),
            name: "Alt farm".into(),
            account_ids: vec![1, 2],
            target: Target::Place {
                place: "920587237".into(),
                job_id: None,
            },
            profile_id: Some("potato".into()),
        };
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("\"kind\":\"place\""));
        assert!(json.contains("\"accountIds\":[1,2]"));
        let back: Preset = serde_json::from_str(&json).unwrap();
        assert_eq!(back, p);
    }
}
