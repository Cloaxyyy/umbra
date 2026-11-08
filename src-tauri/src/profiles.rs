use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
    fs,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Session {
    pub at: u64,
    pub label: String,
    pub secs: u64,
    pub crashed: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            at: 0,
            label: String::new(),
            secs: 0,
            crashed: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub color: String,

    pub pinned_version: Option<String>,
    pub flags: Map<String, Value>,

    pub mods: Vec<String>,

    pub custom_font: Option<String>,
    pub launches: u64,
    pub playtime_secs: u64,
    pub last_played: Option<u64>,
    pub history: Vec<Session>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            id: "main".into(),
            name: "Main".into(),
            color: "#9580ff".into(),
            pinned_version: None,
            flags: Map::new(),
            mods: vec![],
            custom_font: None,
            launches: 0,
            playtime_secs: 0,
            last_played: None,
            history: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Store {
    pub default_id: String,
    pub profiles: Vec<Profile>,
    #[serde(default)]
    pub games: std::collections::HashMap<String, String>,
}

static LOCK: Mutex<()> = Mutex::new(());

fn file() -> std::path::PathBuf {
    super::paths::base().join("profiles.json")
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn read() -> Store {
    if let Some(s) = fs::read_to_string(file())
        .ok()
        .and_then(|s| serde_json::from_str::<Store>(&s).ok())
    {
        if !s.profiles.is_empty() {
            return s;
        }
    }

    let legacy: Map<String, Value> = fs::read_to_string(super::paths::base().join("flags.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    Store {
        default_id: "main".into(),
        profiles: vec![Profile {
            flags: legacy,
            ..Default::default()
        }],
        games: Default::default(),
    }
}

fn write(s: &Store) -> Result<(), String> {
    fs::create_dir_all(super::paths::base()).map_err(|e| e.to_string())?;
    fs::write(file(), serde_json::to_string_pretty(s).unwrap()).map_err(|e| e.to_string())
}

fn with<T>(f: impl FnOnce(&mut Store) -> Result<T, String>) -> Result<T, String> {
    let _g = LOCK.lock().unwrap();
    let mut s = read();
    let out = f(&mut s)?;
    write(&s)?;
    Ok(out)
}

pub fn load() -> Store {
    let _g = LOCK.lock().unwrap();
    read()
}

pub fn resolve(id: Option<&str>) -> Profile {
    let s = load();
    let want = id.unwrap_or(&s.default_id);
    s.profiles
        .iter()
        .find(|p| p.id == want)
        .or_else(|| s.profiles.iter().find(|p| p.id == s.default_id))
        .or(s.profiles.first())
        .cloned()
        .unwrap_or_default()
}

fn validate(p: &Profile) -> Result<(), String> {
    let name = p.name.trim();
    if name.is_empty() || name.chars().count() > 32 {
        return Err("Give the instance a name of 1 to 32 characters.".into());
    }
    let c = p.color.as_bytes();
    if !(c.len() == 7 && c[0] == b'#' && p.color[1..].chars().all(|ch| ch.is_ascii_hexdigit())) {
        return Err("Color must look like #9580ff.".into());
    }
    if let Some(v) = &p.pinned_version {
        if !super::paths::valid_guid(v) {
            return Err("A pinned build looks like version-0123456789abcdef.".into());
        }
    }
    if p.flags
        .values()
        .any(|v| !(v.is_string() || v.is_number() || v.is_boolean()))
    {
        return Err("Flag values must be text, numbers or true/false.".into());
    }
    Ok(())
}

fn slug(name: &str) -> String {
    let base: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let base = base.trim_matches('-');
    let base = if base.is_empty() { "instance" } else { base };
    format!("{}-{:x}", &base[..base.len().min(20)], now() % 0xfffff)
}

pub fn create(name: &str) -> Result<Profile, String> {
    with(|s| {
        let mut p = Profile {
            id: slug(name),
            name: name.trim().to_string(),
            color: "#62d4ff".into(),
            ..Default::default()
        };
        if p.name.is_empty() {
            p.name = "New instance".into();
        }
        validate(&p)?;
        s.profiles.push(p.clone());
        Ok(p)
    })
}

pub fn update(p: Profile) -> Result<Profile, String> {
    validate(&p)?;
    with(|s| {
        let cur = s
            .profiles
            .iter_mut()
            .find(|x| x.id == p.id)
            .ok_or("That instance no longer exists.")?;
        cur.name = p.name.trim().to_string();
        cur.color = p.color.to_lowercase();
        cur.pinned_version = p.pinned_version;
        cur.flags = p.flags;
        cur.mods = p.mods;
        cur.custom_font = p.custom_font;
        Ok(cur.clone())
    })
}

pub fn delete(id: &str) -> Result<(), String> {
    with(|s| {
        if s.profiles.len() <= 1 {
            return Err("You need at least one instance.".into());
        }
        s.profiles.retain(|p| p.id != id);
        s.games.retain(|_, v| v != id);
        if s.default_id == id {
            s.default_id = s.profiles[0].id.clone();
        }
        Ok(())
    })?;
    let _ = fs::remove_dir_all(super::paths::instances().join(id));
    Ok(())
}

pub fn set_game(place_id: u64, profile_id: Option<String>) -> Result<(), String> {
    with(|s| {
        match profile_id {
            Some(id) => {
                if !s.profiles.iter().any(|p| p.id == id) {
                    return Err("That instance no longer exists.".into());
                }
                s.games.insert(place_id.to_string(), id);
            }
            None => {
                s.games.remove(&place_id.to_string());
            }
        }
        Ok(())
    })
}

pub fn game_profile(place_id: u64) -> Option<String> {
    let s = load();
    s.games
        .get(&place_id.to_string())
        .filter(|id| s.profiles.iter().any(|p| &p.id == *id))
        .cloned()
}

pub fn set_default(id: &str) -> Result<(), String> {
    with(|s| {
        if !s.profiles.iter().any(|p| p.id == id) {
            return Err("That instance no longer exists.".into());
        }
        s.default_id = id.to_string();
        Ok(())
    })
}

pub fn remove_mod_everywhere(mod_id: &str) -> Result<(), String> {
    with(|s| {
        for p in &mut s.profiles {
            p.mods.retain(|m| m != mod_id);
        }
        Ok(())
    })
}

pub fn record_launch(id: &str) {
    let _ = with(|s| {
        if let Some(p) = s.profiles.iter_mut().find(|p| p.id == id) {
            p.launches += 1;
            p.last_played = Some(now());
        }
        Ok(())
    });
}

pub fn record_exit(id: &str, label: &str, secs: u64, crashed: bool) {
    let _ = with(|s| {
        if let Some(p) = s.profiles.iter_mut().find(|p| p.id == id) {
            p.playtime_secs += secs;
            p.history.insert(
                0,
                Session {
                    at: now().saturating_sub(secs),
                    label: label.to_string(),
                    secs,
                    crashed,
                },
            );
            p.history.truncate(30);
        }
        Ok(())
    });
}

pub fn pinned_versions() -> Vec<String> {
    load()
        .profiles
        .into_iter()
        .filter_map(|p| p.pinned_version)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_profiles() {
        assert!(validate(&Profile::default()).is_ok());
        assert!(
            validate(&Profile {
                name: " ".into(),
                ..Default::default()
            })
            .is_err()
        );
        assert!(
            validate(&Profile {
                color: "red".into(),
                ..Default::default()
            })
            .is_err()
        );
        assert!(
            validate(&Profile {
                pinned_version: Some("version-0123456789abcdef".into()),
                ..Default::default()
            })
            .is_ok()
        );
        assert!(
            validate(&Profile {
                pinned_version: Some("version-../../x".into()),
                ..Default::default()
            })
            .is_err()
        );
        let mut flags = Map::new();
        flags.insert("X".into(), serde_json::json!({ "nested": 1 }));
        assert!(
            validate(&Profile {
                flags,
                ..Default::default()
            })
            .is_err()
        );
    }

    #[test]
    fn slugs_are_safe_folder_names() {
        let s = slug("Alt Farm / ../evil");
        assert!(
            s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'),
            "{s}"
        );
        assert!(slug("!!!").starts_with("instance-"));
    }
}
