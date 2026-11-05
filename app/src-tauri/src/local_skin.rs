//! Optional, bounded reads of user-supplied original artwork. No proprietary
//! images enter Git or release artifacts. The CSS/SVG interface is standalone.
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::io::Read;
use std::path::{Path, PathBuf};

fn root() -> Option<PathBuf> {
    if std::env::var_os("AULA_NO_ORIGINAL_SKIN").is_some() {
        return None;
    }
    if let Some(p) = std::env::var_os("AULA_ORIGINAL_SKIN_DIR") {
        return Some(p.into());
    }
    // Locate only the supplied project's extracted assets, never scan the host.
    let exe = std::env::current_exe().ok()?;
    exe.ancestors()
        .take(5)
        .map(|p| p.join("analysis/extracted/app"))
        .find(|p| p.join("skins/main_nr.png").is_file())
}
fn png(path: &Path, dimensions: (u32, u32)) -> Option<String> {
    let size = std::fs::metadata(path).ok()?.len();
    if !(24..=1_048_576).contains(&size) {
        return None;
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .take(1_048_577)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > 1_048_576
        || bytes.get(..8)? != b"\x89PNG\r\n\x1a\n"
        || bytes.get(12..16)? != b"IHDR"
    {
        return None;
    }
    let width = u32::from_be_bytes(bytes.get(16..20)?.try_into().ok()?);
    let height = u32::from_be_bytes(bytes.get(20..24)?.try_into().ok()?);
    if (width, height) != dimensions {
        return None;
    }
    Some(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}
pub fn assets() -> Value {
    let mut images = serde_json::Map::new();
    if let Some(root) = root() {
        for (key, name, dimensions) in [
            ("background", "skins/main_nr.png", (1200, 720)),
            ("colorwheel", "skins/colorwheel.png", (116, 117)),
            ("slider", "skins/slider.png", (12, 12)),
            ("rgbSlider", "skins/slider2.png", (14, 7)),
            ("checkOff", "skins/check_nr.png", (13, 13)),
            ("checkOn", "skins/check_dn.png", (13, 13)),
        ] {
            if let Some(data) = png(&root.join(name), dimensions) {
                images.insert(key.into(), json!(data));
            }
        }
        for name in ["new", "del", "copy", "rename", "import", "export"] {
            if let Some(data) = png(
                &root.join(format!("skins/icon18x18/{name}_nr.png")),
                (18, 18),
            ) {
                images.insert(format!("macro_{name}_nr"), json!(data));
            }
        }
        for id in [1, 2, 3, 4, 5] {
            for state in ["nr", "dn"] {
                if let Some(data) = png(
                    &root.join(format!("skins/navi50x50/navi{id}_{state}.png")),
                    (50, 50),
                ) {
                    images.insert(format!("nav{id}_{state}"), json!(data));
                }
            }
        }
    }
    json!({"assets":images,"local_only":true,"redistributed":false})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_wrong_shapes_and_non_png_files() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/skin-tests");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("not-an-image");
        std::fs::write(&path, b"not png").unwrap();
        assert!(png(&path, (1200, 720)).is_none());
        let mut header = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        header.extend(9999u32.to_be_bytes());
        header.extend(9999u32.to_be_bytes());
        std::fs::write(&path, header).unwrap();
        assert!(png(&path, (1200, 720)).is_none());
        std::fs::remove_file(path).unwrap();
    }
}
