use base64::{Engine, engine::general_purpose::STANDARD as B64};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Stored {
    user_id: u64,
    name: String,
    display_name: String,

    cookie: String,

    #[serde(default)]
    profile_id: Option<String>,
    #[serde(default)]
    auto_rejoin: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub user_id: u64,
    pub name: String,
    pub display_name: String,
    pub avatar: Option<String>,
    pub profile_id: Option<String>,
    pub auto_rejoin: bool,
}

fn file() -> std::path::PathBuf {
    super::paths::base().join("accounts.json")
}

fn load() -> Vec<Stored> {
    fs::read_to_string(file())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save(list: &[Stored]) -> Result<(), String> {
    fs::create_dir_all(super::paths::base()).map_err(|e| e.to_string())?;
    fs::write(file(), serde_json::to_string_pretty(list).unwrap()).map_err(|e| e.to_string())
}

pub async fn list() -> Vec<Account> {
    let stored = load();
    let ids: Vec<u64> = stored.iter().map(|s| s.user_id).collect();
    let avatars = if ids.is_empty() {
        Default::default()
    } else {
        super::roblox::headshots(&ids).await
    };
    stored
        .into_iter()
        .map(|s| Account {
            avatar: avatars.get(&s.user_id).cloned(),
            user_id: s.user_id,
            name: s.name,
            display_name: s.display_name,
            profile_id: s.profile_id,
            auto_rejoin: s.auto_rejoin,
        })
        .collect()
}

pub fn ids() -> Vec<u64> {
    load().iter().map(|s| s.user_id).collect()
}

pub async fn add(cookie: &str) -> Result<Account, String> {
    let user = super::roblox::authenticated_user(cookie).await?;
    let enc = B64.encode(super::dpapi::protect(cookie.as_bytes())?);
    let mut list = load();
    let old = list.iter().find(|s| s.user_id == user.id);
    let profile_id = old.and_then(|s| s.profile_id.clone());
    let auto_rejoin = old.is_some_and(|s| s.auto_rejoin);
    list.retain(|s| s.user_id != user.id);
    list.push(Stored {
        user_id: user.id,
        name: user.name.clone(),
        display_name: user.display_name.clone(),
        cookie: enc,
        profile_id: profile_id.clone(),
        auto_rejoin,
    });
    save(&list)?;
    Ok(Account {
        user_id: user.id,
        name: user.name,
        display_name: user.display_name,
        avatar: None,
        profile_id,
        auto_rejoin,
    })
}

pub fn remove(user_id: u64) -> Result<(), String> {
    let mut list = load();
    list.retain(|s| s.user_id != user_id);
    save(&list)
}

pub fn set_profile(user_id: u64, profile_id: Option<String>) -> Result<(), String> {
    let mut list = load();
    let s = list
        .iter_mut()
        .find(|s| s.user_id == user_id)
        .ok_or("That account isn't saved anymore.")?;
    s.profile_id = profile_id;
    save(&list)
}

pub fn set_auto_rejoin(user_id: u64, on: bool) -> Result<(), String> {
    let mut list = load();
    let s = list
        .iter_mut()
        .find(|s| s.user_id == user_id)
        .ok_or("That account isn't saved anymore.")?;
    s.auto_rejoin = on;
    save(&list)
}

pub fn auto_rejoin_of(user_id: u64) -> bool {
    load().iter().any(|s| s.user_id == user_id && s.auto_rejoin)
}

pub fn display_names() -> std::collections::HashMap<u64, String> {
    load()
        .into_iter()
        .map(|s| (s.user_id, s.display_name))
        .collect()
}

pub fn profile_of(user_id: u64) -> Option<String> {
    load()
        .into_iter()
        .find(|s| s.user_id == user_id)
        .and_then(|s| s.profile_id)
}

pub fn cookie(user_id: u64) -> Result<String, String> {
    let s = load()
        .into_iter()
        .find(|s| s.user_id == user_id)
        .ok_or("That account isn't saved anymore.")?;
    let raw = B64
        .decode(&s.cookie)
        .map_err(|_| "Saved account is corrupted. Log in again.")?;
    String::from_utf8(super::dpapi::unprotect(&raw)?)
        .map_err(|_| "Saved account is corrupted. Log in again.".into())
}

pub fn any_cookie() -> Option<String> {
    load().first().and_then(|s| cookie(s.user_id).ok())
}
