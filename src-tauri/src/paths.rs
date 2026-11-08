use std::path::PathBuf;

pub fn base() -> PathBuf {
    dirs::data_local_dir()
        .expect("no local app data dir")
        .join("Umbra")
}
pub fn versions() -> PathBuf {
    base().join("Versions")
}
pub fn downloads() -> PathBuf {
    base().join("Downloads")
}
pub fn version_dir(guid: &str) -> PathBuf {
    versions().join(guid)
}

pub fn is_installed(guid: &str) -> bool {
    version_dir(guid).join("AppSettings.xml").exists()
        && version_dir(guid).join("RobloxPlayerBeta.exe").exists()
}
pub fn instances() -> PathBuf {
    base().join("Instances")
}
pub fn mods() -> PathBuf {
    base().join("Mods")
}
pub fn fonts() -> PathBuf {
    base().join("Fonts")
}

pub fn valid_guid(s: &str) -> bool {
    s.len() == 24 && s.starts_with("version-") && s[8..].chars().all(|c| c.is_ascii_hexdigit())
}
