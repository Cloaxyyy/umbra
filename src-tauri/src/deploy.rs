use futures_util::StreamExt;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};
use tauri::{AppHandle, Emitter};

const CDN: &str = "https://setup.rbxcdn.com/channel/common";

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClientVersion {
    pub version: String,
    pub client_version_upload: String,
}

pub async fn latest(channel: &str) -> Result<ClientVersion, String> {
    let path = if channel.eq_ignore_ascii_case("production") {
        "/v2/client-version/WindowsPlayer".to_string()
    } else {
        format!("/v2/client-version/WindowsPlayer/channel/{channel}")
    };
    let mut last_err = String::new();
    for host in [
        "https://clientsettingscdn.roblox.com",
        "https://clientsettings.roblox.com",
    ] {
        match fetch_json::<ClientVersion>(&format!("{host}{path}")).await {
            Ok(v) => return Ok(v),
            Err(e) => last_err = e,
        }
    }
    Err(format!("Could not get Roblox version info: {last_err}"))
}

async fn fetch_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, String> {
    reqwest::get(url)
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

#[derive(Clone, Debug)]
struct Package {
    name: String,
    signature: String,
    packed_size: u64,
}

fn parse_manifest(text: &str) -> Result<Vec<Package>, String> {
    let mut lines = text.lines().map(str::trim);
    if lines.next() != Some("v0") {
        return Err("Unexpected package manifest format".into());
    }
    let mut out = Vec::new();
    while let (Some(name), Some(sig), Some(packed), Some(_size)) =
        (lines.next(), lines.next(), lines.next(), lines.next())
    {
        if name.is_empty() || name == "RobloxPlayerLauncher.exe" {
            break;
        }
        out.push(Package {
            name: name.into(),
            signature: sig.into(),
            packed_size: packed.parse().unwrap_or(0),
        });
    }
    Ok(out)
}

fn dir_for(package: &str) -> Option<&'static str> {
    Some(match package {
        "RobloxApp.zip" | "Libraries.zip" | "redist.zip" | "WebView2.zip" => "",
        "shaders.zip" => "shaders",
        "ssl.zip" => "ssl",
        "WebView2RuntimeInstaller.zip" => "WebView2RuntimeInstaller",
        "content-avatar.zip" => "content/avatar",
        "content-configs.zip" => "content/configs",
        "content-fonts.zip" => "content/fonts",
        "content-sky.zip" => "content/sky",
        "content-sounds.zip" => "content/sounds",
        "content-textures2.zip" => "content/textures",
        "content-models.zip" => "content/models",
        "content-textures3.zip" => "PlatformContent/pc/textures",
        "content-terrain.zip" => "PlatformContent/pc/terrain",
        "content-platform-fonts.zip" => "PlatformContent/pc/fonts",
        "content-platform-dictionaries.zip" => "PlatformContent/pc/shared_compression_dictionaries",
        "extracontent-luapackages.zip" => "ExtraContent/LuaPackages",
        "extracontent-translations.zip" => "ExtraContent/translations",
        "extracontent-models.zip" => "ExtraContent/models",
        "extracontent-textures.zip" => "ExtraContent/textures",
        "extracontent-places.zip" => "ExtraContent/places",
        _ => return None,
    })
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub stage: String,
    pub detail: String,
    pub done: u64,
    pub total: u64,
}

fn emit(app: &AppHandle, stage: &str, detail: &str, done: u64, total: u64) {
    let _ = app.emit(
        "install-progress",
        Progress {
            stage: stage.into(),
            detail: detail.into(),
            done,
            total,
        },
    );
}

fn md5_file(path: &Path) -> io::Result<String> {
    let mut f = fs::File::open(path)?;
    let mut h = Md5::new();
    io::copy(&mut f, &mut h)?;
    Ok(format!("{:x}", h.finalize()))
}

async fn download(
    app: &AppHandle,
    guid: &str,
    p: &Package,
    mut done: u64,
    total: u64,
) -> Result<u64, String> {
    let dest = super::paths::downloads().join(&p.signature);
    if dest.exists() && md5_file(&dest).map_err(|e| e.to_string())? == p.signature {
        return Ok(done + p.packed_size);
    }
    let mut last_err = String::new();
    for _attempt in 0..3 {
        let resp = match reqwest::get(format!("{CDN}/{guid}-{}", p.name))
            .await
            .and_then(|r| r.error_for_status())
        {
            Ok(r) => r,
            Err(e) => {
                last_err = e.to_string();
                continue;
            }
        };
        let tmp = dest.with_extension("part");
        let mut file = fs::File::create(&tmp).map_err(|e| e.to_string())?;
        let mut stream = resp.bytes_stream();
        let start = done;
        let mut failed = false;
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    io::Write::write_all(&mut file, &bytes).map_err(|e| e.to_string())?;
                    done += bytes.len() as u64;
                    emit(app, "download", &p.name, done, total);
                }
                Err(e) => {
                    last_err = e.to_string();
                    failed = true;
                    break;
                }
            }
        }
        drop(file);
        if !failed && md5_file(&tmp).map_err(|e| e.to_string())? == p.signature {
            fs::rename(&tmp, &dest).map_err(|e| e.to_string())?;
            return Ok(start + p.packed_size);
        }
        done = start;
        let _ = fs::remove_file(&tmp);
        if !failed {
            last_err = "checksum mismatch".into();
        }
    }
    Err(format!("Download of {} failed: {last_err}", p.name))
}

fn extract(zip_path: &Path, into: &Path) -> Result<(), String> {
    let file = fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;

        let name = entry.name().replace('\\', "/");
        if name.split('/').any(|seg| seg == "..") {
            continue;
        }
        let out = into.join(&name);
        if name.ends_with('/') {
            fs::create_dir_all(&out).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut f = fs::File::create(&out).map_err(|e| format!("{}: {e}", out.display()))?;
        io::copy(&mut entry, &mut f).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn install(app: &AppHandle, guid: &str) -> Result<(), String> {
    let dir = super::paths::version_dir(guid);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(super::paths::downloads()).map_err(|e| e.to_string())?;

    emit(app, "manifest", "Reading package list", 0, 1);
    let text = reqwest::get(format!("{CDN}/{guid}-rbxPkgManifest.txt"))
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    let packages = parse_manifest(&text)?;
    let total: u64 = packages.iter().map(|p| p.packed_size).sum();

    let mut done = 0;
    for p in &packages {
        done = download(app, guid, p, done, total).await?;
        if p.name == "WebView2RuntimeInstaller.zip" {
            continue;
        }
        let Some(sub) = dir_for(&p.name) else {
            continue;
        };
        emit(app, "extract", &p.name, done, total);
        let zip_path = super::paths::downloads().join(&p.signature);
        let target = dir.join(sub);
        tokio::task::spawn_blocking(move || extract(&zip_path, &target))
            .await
            .map_err(|e| e.to_string())??;
    }

    fs::write(
        dir.join("AppSettings.xml"),
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<Settings>\r\n\t<ContentFolder>content</ContentFolder>\r\n\t<BaseUrl>http://www.roblox.com</BaseUrl>\r\n</Settings>\r\n",
    ).map_err(|e| e.to_string())?;
    emit(app, "done", "Installed", total, total);
    Ok(())
}

pub fn prune(keep: &[String]) {
    if let Ok(rd) = fs::read_dir(super::paths::versions()) {
        for e in rd.flatten() {
            if !keep.iter().any(|k| e.file_name() == k.as_str()) {
                let _ = fs::remove_dir_all(e.path());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_manifest_and_stops_at_launcher() {
        let m = parse_manifest("v0\r\nRobloxApp.zip\r\nabc\r\n100\r\n200\r\nRobloxPlayerLauncher.exe\r\nzzz\r\n1\r\n2\r\n").unwrap();
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].packed_size, 100);
        assert_eq!(
            dir_for("content-textures3.zip"),
            Some("PlatformContent/pc/textures")
        );
        assert_eq!(dir_for("nope.zip"), None);
    }
}
