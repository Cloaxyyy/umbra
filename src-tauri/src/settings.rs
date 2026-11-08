use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub channel: String,
    pub multi_instance: bool,

    pub installed_version: Option<String>,

    pub server_location: bool,

    pub discord_enabled: bool,
    pub discord_app_id: String,

    pub discord_join_button: bool,

    pub close_to_tray: bool,

    pub auto_check_updates: bool,
    pub accent: String,
    pub overlay_enabled: bool,
    pub overlay_hotkey: String,
    pub friend_alerts: bool,
    pub overlay_friends_here: bool,
    pub overlay_recent: bool,
    pub overlay_server_ip: bool,
    pub extension_bridge: bool,
    pub label_windows: bool,
    pub trim_background: bool,
    pub update_alerts: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            channel: "production".into(),
            multi_instance: true,
            installed_version: None,
            server_location: false,
            discord_enabled: false,
            discord_app_id: String::new(),
            discord_join_button: false,
            close_to_tray: false,
            auto_check_updates: true,
            accent: "#e8e8e8".into(),
            overlay_enabled: true,
            overlay_hotkey: "Ctrl+Shift+U".into(),
            friend_alerts: true,
            overlay_friends_here: false,
            overlay_recent: false,
            overlay_server_ip: false,
            extension_bridge: true,
            label_windows: false,
            trim_background: false,
            update_alerts: false,
        }
    }
}

fn file() -> std::path::PathBuf {
    super::paths::base().join("settings.json")
}

pub fn load() -> Settings {
    fs::read_to_string(file())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(s: &Settings) -> Result<(), String> {
    fs::create_dir_all(super::paths::base()).map_err(|e| e.to_string())?;
    fs::write(file(), serde_json::to_string_pretty(s).unwrap()).map_err(|e| e.to_string())
}
