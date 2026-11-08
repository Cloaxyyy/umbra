use serde::{Deserialize, Serialize};
use std::{
    fs,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Favorite {
    pub universe_id: u64,
    pub root_place_id: u64,
    pub name: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub added: u64,
}

static LOCK: Mutex<()> = Mutex::new(());

fn file() -> std::path::PathBuf {
    super::paths::base().join("favorites.json")
}

fn read() -> Vec<Favorite> {
    fs::read_to_string(file())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn list() -> Vec<Favorite> {
    let _g = LOCK.lock().unwrap();
    read()
}

pub fn toggle(mut f: Favorite) -> Result<bool, String> {
    if f.name.trim().is_empty() || f.name.chars().count() > 200 {
        return Err("That game has no usable name.".into());
    }
    let _g = LOCK.lock().unwrap();
    let mut all = read();
    let had = all.iter().any(|x| x.universe_id == f.universe_id);
    if had {
        all.retain(|x| x.universe_id != f.universe_id);
    } else {
        f.added = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        all.insert(0, f);
    }
    fs::create_dir_all(super::paths::base()).map_err(|e| e.to_string())?;
    fs::write(file(), serde_json::to_string_pretty(&all).unwrap()).map_err(|e| e.to_string())?;
    Ok(!had)
}
