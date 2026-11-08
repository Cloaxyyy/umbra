use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

const CURSOR_ARROW: &[u8] = include_bytes!("../theme/cursor-arrow.png");
const CURSOR_HOVER: &[u8] = include_bytes!("../theme/cursor-hover.png");
const DEATH_SOUND: &[u8] = include_bytes!("../theme/death.ogg");

const TINT_DIRS: &[&str] = &[
    "ExtraContent/textures/ui/ImageSet/InGameMenu",
    "ExtraContent/textures/ui/ImageSet/LuaApp",
    "ExtraContent/textures/ui/InGameMenu",
    "content/textures/ui/TopBar",
    "content/textures/ui/Settings",
];
const TINT_STRENGTH: f32 = 0.75;
const GRAY_SPREAD: u8 = 24;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Theme {
    pub cursor: bool,
    pub tint: bool,
    pub death_sound: bool,
}

pub fn parse_hex(hex: &str) -> Option<[u8; 3]> {
    let h = hex.strip_prefix('#')?;
    if h.len() != 6 {
        return None;
    }
    let n = u32::from_str_radix(h, 16).ok()?;
    Some([(n >> 16) as u8, (n >> 8) as u8, n as u8])
}

fn tint_pixel(px: &mut [u8], accent: [u8; 3]) {
    let (lo, hi) = (px[0].min(px[1]).min(px[2]), px[0].max(px[1]).max(px[2]));
    if px[3] == 0 || hi - lo > GRAY_SPREAD {
        return;
    }
    let l = (px[0] as f32 + px[1] as f32 + px[2] as f32) / (3.0 * 255.0);
    for i in 0..3 {
        let target = 255.0 * (1.0 - TINT_STRENGTH) + accent[i] as f32 * TINT_STRENGTH;
        px[i] = (l * target).round().clamp(0.0, 255.0) as u8;
    }
}

fn tint_png(bytes: &[u8], accent: [u8; 3]) -> Option<Vec<u8>> {
    let mut dec = png::Decoder::new(io::Cursor::new(bytes));
    dec.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = dec.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    let data = &buf[..info.buffer_size()];
    let mut rgba: Vec<u8> = match info.color_type {
        png::ColorType::Rgba => data.to_vec(),
        png::ColorType::Rgb => data
            .chunks_exact(3)
            .flat_map(|c| [c[0], c[1], c[2], 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => data
            .chunks_exact(2)
            .flat_map(|c| [c[0], c[0], c[0], c[1]])
            .collect(),
        png::ColorType::Grayscale => data.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        _ => return None,
    };
    for px in rgba.chunks_exact_mut(4) {
        tint_pixel(px, accent);
    }
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, info.width, info.height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().ok()?;
        w.write_image_data(&rgba).ok()?;
    }
    Some(out)
}

fn tint_tree(dir: &Path, accent: [u8; 3]) -> io::Result<()> {
    let Ok(rd) = fs::read_dir(dir) else {
        return Ok(());
    };
    for e in rd.flatten() {
        let p = e.path();
        if e.file_type()?.is_dir() {
            tint_tree(&p, accent)?;
        } else if p
            .extension()
            .and_then(|x| x.to_str())
            .is_some_and(|x| x.eq_ignore_ascii_case("png"))
        {
            if let Some(out) = tint_png(&fs::read(&p)?, accent) {
                super::builder::place(&p, &out)?;
            }
        }
    }
    Ok(())
}

pub fn apply(dir: &Path, theme: &Theme, accent: &str) -> io::Result<()> {
    let place = super::builder::place;
    if theme.cursor {
        for base in ["content/textures/Cursors/KeyboardMouse", "content/textures"] {
            place(&dir.join(base).join("ArrowFarCursor.png"), CURSOR_ARROW)?;
            place(&dir.join(base).join("ArrowCursor.png"), CURSOR_HOVER)?;
        }
    }
    if theme.death_sound {
        place(&dir.join("content/sounds/ouch.ogg"), DEATH_SOUND)?;
    }
    if theme.tint {
        let accent = parse_hex(accent).unwrap_or([149, 128, 255]);
        for rel in TINT_DIRS {
            tint_tree(&dir.join(rel), accent)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tints_gray_art_and_keeps_colors() {
        let accent = [149, 128, 255];
        let mut white = [255, 255, 255, 255];
        tint_pixel(&mut white, accent);
        assert!(white[2] > white[1] && white[0] < 255);
        let mut green = [40, 200, 90, 255];
        tint_pixel(&mut green, accent);
        assert_eq!(green, [40, 200, 90, 255]);
        let mut black = [0, 0, 0, 255];
        tint_pixel(&mut black, accent);
        assert_eq!(black, [0, 0, 0, 255]);
        assert_eq!(parse_hex("#9580ff"), Some(accent));
        assert_eq!(parse_hex("9580ff"), None);
    }

    #[test]
    fn tint_png_round_trips_gray_alpha() {
        let mut src = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut src, 2, 1);
            enc.set_color(png::ColorType::GrayscaleAlpha);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header()
                .unwrap()
                .write_image_data(&[255, 255, 0, 0])
                .unwrap();
        }
        let out = tint_png(&src, [149, 128, 255]).unwrap();
        let mut r = png::Decoder::new(io::Cursor::new(&out[..]))
            .read_info()
            .unwrap();
        let mut buf = vec![0; r.output_buffer_size().unwrap()];
        let info = r.next_frame(&mut buf).unwrap();
        assert_eq!(
            (info.width, info.height, info.color_type),
            (2, 1, png::ColorType::Rgba)
        );
        assert_eq!(buf[3], 255);
        assert!(buf[2] > buf[1]);
        assert_eq!(buf[7], 0);
    }

    fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        for e in fs::read_dir(dir).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().and_then(|x| x.to_str()) == Some("png") {
                out.push(p);
            }
        }
    }

    #[test]
    #[ignore]
    fn tints_real_client_art() {
        let guid = std::env::var("UMBRA_TEST_GUID").expect("set UMBRA_TEST_GUID");
        let base = super::super::paths::version_dir(&guid);
        let mut files = Vec::new();
        for rel in TINT_DIRS {
            walk(&base.join(rel), &mut files);
        }
        let t = std::time::Instant::now();
        let failed: Vec<_> = files
            .iter()
            .filter(|f| tint_png(&fs::read(f).unwrap(), [149, 128, 255]).is_none())
            .collect();
        println!(
            "{} files, {} failed, {:?}",
            files.len(),
            failed.len(),
            t.elapsed()
        );
        if let Ok(out) = std::env::var("UMBRA_TINT_PREVIEW") {
            let sheet = base.join("ExtraContent/textures/ui/ImageSet/InGameMenu/img_set_1x_1.png");
            fs::write(
                out,
                tint_png(&fs::read(sheet).unwrap(), [149, 128, 255]).unwrap(),
            )
            .unwrap();
        }
        assert!(failed.is_empty(), "{failed:?}");
    }

    #[test]
    fn bundled_assets_are_real_files() {
        assert!(CURSOR_ARROW.starts_with(b"\x89PNG"));
        assert!(CURSOR_HOVER.starts_with(b"\x89PNG"));
        assert!(DEATH_SOUND.starts_with(b"OggS"));
    }
}
