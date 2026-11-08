use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const ROOTS: [&str; 3] = ["content", "PlatformContent", "ExtraContent"];
const BLOCKED: [&str; 12] = [
    "exe", "dll", "bat", "cmd", "com", "ps1", "vbs", "js", "msi", "scr", "lnk", "sys",
];
const MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
    pub id: String,
    pub name: String,
    pub files: u64,
    pub bytes: u64,
    pub added: u64,
}

fn index() -> PathBuf {
    super::paths::mods().join("mods.json")
}
pub fn list() -> Vec<Mod> {
    fs::read_to_string(index())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}
fn save(list: &[Mod]) -> Result<(), String> {
    fs::create_dir_all(super::paths::mods()).map_err(|e| e.to_string())?;
    fs::write(index(), serde_json::to_string_pretty(list).unwrap()).map_err(|e| e.to_string())
}
pub fn dir(id: &str) -> PathBuf {
    super::paths::mods().join(id)
}

pub fn target_path(raw: &str) -> Option<String> {
    let norm = raw.replace('\\', "/");
    let segs: Vec<&str> = norm.split('/').filter(|s| !s.is_empty()).collect();
    if segs
        .iter()
        .any(|s| *s == ".." || *s == "." || s.contains(':'))
    {
        return None;
    }
    let i = segs
        .iter()
        .position(|s| ROOTS.iter().any(|r| r.eq_ignore_ascii_case(s)))?;
    let rest = &segs[i..];
    if rest.len() < 2 {
        return None;
    }
    let ext = rest
        .last()?
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    if BLOCKED.contains(&ext.as_str()) {
        return None;
    }
    let root = ROOTS.iter().find(|r| r.eq_ignore_ascii_case(rest[0]))?;
    Some(
        std::iter::once(*root)
            .chain(rest[1..].iter().copied())
            .collect::<Vec<_>>()
            .join("/"),
    )
}

fn new_id(name: &str) -> String {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let s: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .take(24)
        .collect();
    format!("{}-{:x}", s.trim_matches('-'), n % 0xffffff)
}

pub fn import(src: &Path) -> Result<Mod, String> {
    let name = src
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Mod")
        .to_string();
    let id = new_id(&name);
    let out = dir(&id);
    let result = if src.is_dir() {
        import_dir(src, &out)
    } else {
        import_zip(src, &out)
    };
    let (files, bytes) = match result {
        Ok(v) => v,
        Err(e) => {
            let _ = fs::remove_dir_all(&out);
            return Err(e);
        }
    };
    if files == 0 {
        let _ = fs::remove_dir_all(&out);
        return Err("No Roblox asset files found. A mod needs content/, PlatformContent/ or ExtraContent/ folders.".into());
    }
    let m = Mod {
        id,
        name,
        files,
        bytes,
        added: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    let mut all = list();
    all.insert(0, m.clone());
    save(&all)?;
    Ok(m)
}

fn write_file(out: &Path, rel: &str, mut r: impl io::Read) -> Result<u64, String> {
    let dest = out.join(rel);
    if let Some(p) = dest.parent() {
        fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let mut f = fs::File::create(&dest).map_err(|e| e.to_string())?;
    io::copy(&mut r, &mut f).map_err(|e| e.to_string())
}

fn import_zip(src: &Path, out: &Path) -> Result<(u64, u64), String> {
    let file = fs::File::open(src).map_err(|e| format!("Couldn't open {}: {e}", src.display()))?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|_| "That isn't a ZIP file Umbra can read.".to_string())?;
    let (mut files, mut bytes) = (0, 0);
    for i in 0..zip.len() {
        let entry = zip.by_index(i).map_err(|e| e.to_string())?;
        if entry.is_dir() {
            continue;
        }
        let Some(rel) = target_path(entry.name()) else {
            continue;
        };
        bytes += entry.size();
        if bytes > MAX_BYTES {
            return Err("Mods over 2 GB aren't supported.".into());
        }
        write_file(out, &rel, entry)?;
        files += 1;
    }
    Ok((files, bytes))
}

fn import_dir(src: &Path, out: &Path) -> Result<(u64, u64), String> {
    let (mut files, mut bytes) = (0, 0);
    let mut stack = vec![src.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).map_err(|e| e.to_string())?.flatten() {
            let p = e.path();
            let ft = e.file_type().map_err(|e| e.to_string())?;
            if ft.is_symlink() {
                continue;
            }
            if ft.is_dir() {
                stack.push(p);
                continue;
            }
            let rel_src = p.strip_prefix(src).unwrap().to_string_lossy().to_string();
            let Some(rel) = target_path(&rel_src) else {
                continue;
            };
            bytes += e.metadata().map(|m| m.len()).unwrap_or(0);
            if bytes > MAX_BYTES {
                return Err("Mods over 2 GB aren't supported.".into());
            }
            write_file(out, &rel, fs::File::open(&p).map_err(|e| e.to_string())?)?;
            files += 1;
        }
    }
    Ok((files, bytes))
}

pub fn delete(id: &str) -> Result<(), String> {
    let mut all = list();
    if !all.iter().any(|m| m.id == id) {
        return Err("That mod is already gone.".into());
    }
    all.retain(|m| m.id != id);
    save(&all)?;
    super::profiles::remove_mod_everywhere(id)?;
    let _ = fs::remove_dir_all(dir(id));
    Ok(())
}

pub fn import_font(src: &Path) -> Result<String, String> {
    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if ext != "ttf" && ext != "otf" {
        return Err("Pick a .ttf or .otf font file.".into());
    }
    let bytes = fs::read(src).map_err(|e| e.to_string())?;
    let magic = bytes.get(0..4).unwrap_or_default();
    if !(magic == [0, 1, 0, 0] || magic == b"OTTO" || magic == b"true") {
        return Err("That file isn't a valid font.".into());
    }
    let name: String = src
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("font.ttf")
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    fs::create_dir_all(super::paths::fonts()).map_err(|e| e.to_string())?;
    fs::write(super::paths::fonts().join(&name), bytes).map_err(|e| e.to_string())?;
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::target_path;

    #[test]
    fn maps_and_filters_paths() {
        assert_eq!(
            target_path("content/sounds/oof.ogg").as_deref(),
            Some("content/sounds/oof.ogg")
        );
        assert_eq!(
            target_path("MyMod\\Content\\textures\\a.png").as_deref(),
            Some("content/textures/a.png")
        );
        assert_eq!(
            target_path("x/platformcontent/pc/terrain/grass.dds").as_deref(),
            Some("PlatformContent/pc/terrain/grass.dds")
        );
        assert_eq!(target_path("RobloxPlayerBeta.exe"), None);
        assert_eq!(target_path("content/evil.dll"), None);
        assert_eq!(target_path("content/../../RobloxPlayerBeta.exe"), None);
        assert_eq!(target_path("C:/content/a.png"), None);
        assert_eq!(target_path("readme.txt"), None);
        assert_eq!(target_path("content"), None);
    }
}
