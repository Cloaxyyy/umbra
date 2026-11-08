use super::profiles::Profile;
use serde_json::{Value, json};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

const STAMP: &str = ".umbra-build.json";

fn stamp_for(p: &Profile, guid: &str) -> Value {
    let mods: Vec<Value> = p
        .mods
        .iter()
        .map(|id| {
            let files = super::mods::list()
                .into_iter()
                .find(|m| &m.id == id)
                .map(|m| m.files)
                .unwrap_or(0);
            json!([id, files])
        })
        .collect();
    let accent = p.theme.tint.then(|| super::settings::load().accent);
    json!({ "guid": guid, "mods": mods, "font": p.custom_font, "theme": p.theme, "accent": accent })
}

fn link_tree(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for e in fs::read_dir(src)? {
        let e = e?;
        let (s, d) = (e.path(), dst.join(e.file_name()));
        if e.file_type()?.is_dir() {
            link_tree(&s, &d)?;
        } else if fs::hard_link(&s, &d).is_err() {
            fs::copy(&s, &d)?;
        }
    }
    Ok(())
}

pub(crate) fn place(dst: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(p) = dst.parent() {
        fs::create_dir_all(p)?;
    }
    if dst.exists() {
        fs::remove_file(dst)?;
    }
    fs::write(dst, bytes)
}

fn overlay(src: &Path, dst: &Path) -> io::Result<()> {
    for e in fs::read_dir(src)? {
        let e = e?;
        let (s, d) = (e.path(), dst.join(e.file_name()));
        if e.file_type()?.is_dir() {
            overlay(&s, &d)?;
        } else {
            place(&d, &fs::read(&s)?)?;
        }
    }
    Ok(())
}

fn apply_font(dir: &Path, font_file: &Path) -> io::Result<()> {
    place(
        &dir.join("content/fonts/CustomFont.ttf"),
        &fs::read(font_file)?,
    )?;
    let families = dir.join("content/fonts/families");
    let Ok(rd) = fs::read_dir(&families) else {
        return Ok(());
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let Ok(mut v) = serde_json::from_slice::<Value>(&fs::read(&p)?) else {
            continue;
        };
        if let Some(faces) = v["faces"].as_array_mut() {
            for f in faces {
                f["assetId"] = json!("rbxasset://fonts/CustomFont.ttf");
            }
        }
        place(&p, serde_json::to_string(&v).unwrap().as_bytes())?;
    }
    Ok(())
}

fn locked(e: io::Error, name: &str) -> String {
    if e.kind() == io::ErrorKind::PermissionDenied || e.raw_os_error() == Some(32) {
        format!("Close the Roblox clients using \"{name}\" first, then try again.")
    } else {
        format!("Couldn't prepare \"{name}\": {e}")
    }
}

pub fn build(p: &Profile, guid: &str) -> Result<PathBuf, String> {
    let dir = super::paths::instances().join(&p.id);
    let want = stamp_for(p, guid);
    let have = fs::read(dir.join(STAMP))
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok());

    if have.as_ref() != Some(&want) {
        if dir.exists() {
            fs::remove_dir_all(&dir).map_err(|e| locked(e, &p.name))?;
        }
        link_tree(&super::paths::version_dir(guid), &dir).map_err(|e| locked(e, &p.name))?;

        for id in p.mods.iter().rev() {
            let src = super::mods::dir(id);
            if src.is_dir() {
                overlay(&src, &dir).map_err(|e| locked(e, &p.name))?;
            }
        }
        if let Some(font) = &p.custom_font {
            let f = super::paths::fonts().join(font);
            if f.is_file() {
                apply_font(&dir, &f).map_err(|e| locked(e, &p.name))?;
            }
        }
        if p.theme != Default::default() {
            let accent = want["accent"].as_str().unwrap_or_default();
            super::theme::apply(&dir, &p.theme, accent).map_err(|e| locked(e, &p.name))?;
        }
        fs::write(dir.join(STAMP), serde_json::to_vec(&want).unwrap())
            .map_err(|e| e.to_string())?;
    }

    place(
        &dir.join("ClientSettings/ClientAppSettings.json"),
        serde_json::to_string_pretty(&p.flags).unwrap().as_bytes(),
    )
    .map_err(|e| locked(e, &p.name))?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_breaks_hard_links_instead_of_writing_through() {
        let t = std::env::temp_dir().join(format!("umbra-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&t);
        fs::create_dir_all(t.join("base")).unwrap();
        fs::write(t.join("base/a.txt"), b"original").unwrap();
        link_tree(&t.join("base"), &t.join("inst")).unwrap();
        place(&t.join("inst/a.txt"), b"modded").unwrap();
        assert_eq!(fs::read(t.join("base/a.txt")).unwrap(), b"original");
        assert_eq!(fs::read(t.join("inst/a.txt")).unwrap(), b"modded");
        fs::remove_dir_all(&t).unwrap();
    }

    #[test]
    #[ignore]
    fn real_build() {
        let guid = std::env::var("UMBRA_TEST_GUID").expect("set UMBRA_TEST_GUID");
        let mod_id = std::env::var("UMBRA_TEST_MOD").ok();
        let mut flags = serde_json::Map::new();
        flags.insert("FFlagDebugSkyGray".into(), json!(true));
        let p = Profile {
            id: "umbra-selftest".into(),
            name: "Self test".into(),
            flags,
            mods: mod_id.clone().into_iter().collect(),
            ..Default::default()
        };

        let base = super::super::paths::version_dir(&guid);
        let rel = "content/textures/Cursors/KeyboardMouse/ArrowCursor.png";
        let base_before = fs::read(base.join(rel)).ok();

        let t = std::time::Instant::now();
        let dir = build(&p, &guid).unwrap();
        let first = t.elapsed();
        assert!(dir.join("RobloxPlayerBeta.exe").is_file());
        let flags_json: Value = serde_json::from_slice(
            &fs::read(dir.join("ClientSettings/ClientAppSettings.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(flags_json["FFlagDebugSkyGray"], json!(true));
        if mod_id.is_some() {
            assert_eq!(
                fs::read(dir.join(rel)).unwrap(),
                b"\x89PNG fake cursor",
                "mod file should be in the instance"
            );
            assert_eq!(
                fs::read(base.join(rel)).ok(),
                base_before,
                "shared build must be untouched"
            );
        }
        let t = std::time::Instant::now();
        build(&p, &guid).unwrap();
        println!("first build {first:?}, unchanged rebuild {:?}", t.elapsed());
        fs::remove_dir_all(dir).unwrap();
    }
}
