//! Versioned local profiles. Saving a profile never writes a keyboard.
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
pub type Result<T> = std::result::Result<T, String>;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Lighting {
    pub effect: u8,
    pub brightness: u8,
    pub speed: u8,
    pub color: [u8; 3],
    pub direction: u8,
    #[serde(default)]
    pub colorful: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MacroEvent {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ms: Option<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Macro {
    pub id: String,
    pub name: String,
    pub events: Vec<MacroEvent>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub lighting: Lighting,
    #[serde(default)]
    pub per_key: BTreeMap<String, [u8; 3]>,
    #[serde(default)]
    pub keymap: BTreeMap<String, u16>,
    #[serde(default)]
    pub fn_layer: BTreeMap<String, u16>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub consumer_keymap: BTreeMap<String, u8>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub consumer_fn_layer: BTreeMap<String, u8>,
    #[serde(default)]
    pub macros: Vec<Macro>,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sleep_seconds: Option<u16>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Preferences {
    pub demo_mode: bool,
    pub reduce_motion: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_colors: Vec<[u8; 3]>,
}
pub fn new_id() -> String {
    format!(
        "p-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}
pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}
impl Profile {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            return Err("Unsupported profile schema".into());
        }
        if !valid_id(&self.id)
            || self.name.trim().is_empty()
            || self.name.len() > 120
            || self.updated_at.len() > 64
        {
            return Err("Invalid profile identity".into());
        }
        if self.lighting.brightness > 9 || self.lighting.speed > 4 || self.lighting.direction > 3 {
            return Err("Lighting parameter outside supported range".into());
        }
        for key in self
            .per_key
            .keys()
            .filter(|key| key.as_str() != "fn")
            .chain(self.keymap.keys())
            .chain(self.fn_layer.keys())
            .chain(self.consumer_keymap.keys())
            .chain(self.consumer_fn_layer.keys())
        {
            if key
                .parse::<u16>()
                .ok()
                .filter(|x| *x > 0 && *x <= 255)
                .is_none()
            {
                return Err("Key must be a USB HID usage from 1 to 255".into());
            }
        }
        if self
            .keymap
            .values()
            .chain(self.fn_layer.values())
            .any(|v| *v == 0 || *v > 255)
        {
            return Err("Only keyboard-page HID usages 1..255 are supported".into());
        }
        for (keyboard, media) in [
            (&self.keymap, &self.consumer_keymap),
            (&self.fn_layer, &self.consumer_fn_layer),
        ] {
            let mut seen = BTreeSet::new();
            for source in keyboard.keys().chain(media.keys()) {
                // Numeric aliases (e.g.04 and4) must not create duplicate edits.
                let usage = source.parse::<u16>().map_err(|_| "Invalid source key")?;
                if !seen.insert(usage) {
                    return Err("A source key can have only one assignment in each layer".into());
                }
            }
            if media
                .values()
                .any(|v| aula_protocol::media_action_name(*v).is_none())
            {
                return Err("Unsupported media action".into());
            }
        }
        if self
            .sleep_seconds
            .is_some_and(|s| ![0, 60, 90, 120, 180, 240, 300, 600, 900, 1200].contains(&s))
        {
            return Err("Unsupported sleep time".into());
        }
        if self.macros.len() > 64 {
            return Err("Too many local macros".into());
        }
        for m in &self.macros {
            if !valid_id(&m.id)
                || m.name.trim().is_empty()
                || m.name.len() > 120
                || m.events.len() > 2048
            {
                return Err("Invalid macro or too many events".into());
            }
            let mut held = BTreeSet::new();
            for e in &m.events {
                match e.kind.as_str() {
                    "down" | "up"
                        if e.key.is_some_and(|k| (4..=231).contains(&k)) && e.ms.is_none() =>
                    {
                        let key = e.key.unwrap();
                        if (e.kind == "down" && !held.insert(key))
                            || (e.kind == "up" && !held.remove(&key))
                        {
                            return Err("Macro presses and releases must be balanced".into());
                        }
                    }
                    "delay" if e.ms.is_some_and(|ms| ms <= 60000) && e.key.is_none() => (),
                    _ => return Err("Invalid macro event".into()),
                }
            }
            if !held.is_empty() {
                return Err("Release all pressed keys before saving a macro".into());
            }
        }
        Ok(())
    }
}
pub fn decode_profile(json: &str) -> Result<Profile> {
    if json.len() > 512 * 1024 {
        return Err("Profile exceeds 512 KiB limit".into());
    }
    let mut v: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    // v0 local draft format had the same fields, without an explicit version.
    if v.get("schema_version").is_none() || v.get("schema_version") == Some(&serde_json::json!(0)) {
        v["schema_version"] = 1.into()
    }
    let p: Profile = serde_json::from_value(v).map_err(|e| e.to_string())?;
    p.validate()?;
    Ok(p)
}
pub struct Store {
    root: PathBuf,
    config: PathBuf,
}
impl Store {
    pub fn new(root: PathBuf) -> Result<Self> {
        Self::with_config_dir(root.clone(), root)
    }
    pub fn for_app() -> Result<Self> {
        Self::with_config_dir(data_dir(), config_dir())
    }
    pub fn with_config_dir(root: PathBuf, config: PathBuf) -> Result<Self> {
        fs::create_dir_all(root.join("profiles")).map_err(|e| e.to_string())?;
        fs::create_dir_all(root.join("history")).map_err(|e| e.to_string())?;
        Ok(Self { root, config })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn list(&self) -> Result<Vec<Profile>> {
        let mut result = vec![];
        for entry in fs::read_dir(self.root.join("profiles")).map_err(|e| e.to_string())? {
            let p = entry.map_err(|e| e.to_string())?.path();
            if p.extension().is_some_and(|x| x == "json") {
                let s = fs::read_to_string(p).map_err(|e| e.to_string())?;
                result.push(decode_profile(&s)?);
            }
        }
        result.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(result)
    }
    pub fn save(&self, p: &Profile) -> Result<()> {
        p.validate()?;
        let serialized = serde_json::to_string_pretty(p).map_err(|e| e.to_string())?;
        if serialized.len() > 512 * 1024 {
            return Err(
                "Serialized profile exceeds512 KiB. Reduce macro events before saving.".into(),
            );
        }
        let dest = self.profile_path(&p.id)?;
        if dest.exists() {
            let old = fs::read(&dest).map_err(|e| e.to_string())?;
            atomic_write(
                &self
                    .root
                    .join("history")
                    .join(format!("{}-{}.json", p.id, new_id())),
                &old,
            )?;
        }
        atomic_write(&dest, serialized.as_bytes())
    }
    fn profile_path(&self, id: &str) -> Result<PathBuf> {
        if !valid_id(id) {
            return Err("Invalid profile ID".into());
        }
        Ok(self.root.join("profiles").join(format!("{id}.json")))
    }
    pub fn get(&self, id: &str) -> Result<Profile> {
        decode_profile(&fs::read_to_string(self.profile_path(id)?).map_err(|e| e.to_string())?)
    }
    pub fn delete(&self, id: &str) -> Result<()> {
        let p = self.get(id)?;
        atomic_write(
            &self
                .root
                .join("history")
                .join(format!("deleted-{id}-{}.json", new_id())),
            serde_json::to_string_pretty(&p).unwrap().as_bytes(),
        )?;
        fs::remove_file(self.profile_path(id)?).map_err(|e| e.to_string())
    }
    pub fn import(&self, json: &str) -> Result<Profile> {
        let mut p = decode_profile(json)?;
        p.id = new_id();
        self.save(&p)?;
        Ok(p)
    }
    pub fn preferences(&self) -> Result<Preferences> {
        let current = self.config.join("preferences.json");
        // Read legacy preferences without moving/deleting them. Migration
        // happens on the next explicit preference save, never during a read.
        let p = if current.exists() {
            current
        } else {
            self.root.join("preferences.json")
        };
        if !p.exists() {
            return Ok(Preferences::default());
        }
        if fs::metadata(&p).map_err(|e| e.to_string())?.len() > 65536 {
            return Err("Preferences exceed64 KiB limit".into());
        }
        let preferences: Preferences =
            serde_json::from_slice(&fs::read(p).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if preferences.custom_colors.len() > 10 {
            return Err("At most10 custom colors are supported".into());
        }
        Ok(preferences)
    }
    pub fn save_preferences(&self, p: &Preferences) -> Result<()> {
        if p.custom_colors.len() > 10 {
            return Err("At most10 custom colors are supported".into());
        }
        fs::create_dir_all(&self.config).map_err(|e| e.to_string())?;
        atomic_write(
            &self.config.join("preferences.json"),
            &serde_json::to_vec_pretty(p).unwrap(),
        )
    }
}
pub fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    let temp = path.with_extension(format!("{}.tmp", new_id()));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut f = options.open(&temp).map_err(|e| e.to_string())?;
    let result = (|| {
        f.write_all(data)?;
        f.sync_all()?;
        fs::rename(&temp, path)?;
        if let Some(parent) = path.parent() {
            fs::File::open(parent)?.sync_all()?
        }
        Ok::<(), std::io::Error>(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result.map_err(|e| e.to_string())
}
pub fn data_dir() -> PathBuf {
    if let Some(p) = std::env::var_os("AULA_DATA_DIR") {
        return p.into();
    }
    xdg_base(
        std::env::var_os("XDG_DATA_HOME"),
        home_dir().join(".local/share"),
    )
    .join("aula-studio")
}
pub fn config_dir() -> PathBuf {
    // AULA_DATA_DIR is an explicit all-storage override for isolated QA.
    if let Some(p) = std::env::var_os("AULA_DATA_DIR") {
        return p.into();
    }
    xdg_base(
        std::env::var_os("XDG_CONFIG_HOME"),
        home_dir().join(".config"),
    )
    .join("aula-studio")
}
fn home_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
}
fn xdg_base(setting: Option<std::ffi::OsString>, fallback: PathBuf) -> PathBuf {
    setting
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or(fallback)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn profile() -> Profile {
        decode_profile(r#"{"schema_version":1,"id":"test","name":"Test","lighting":{"effect":1,"brightness":3,"speed":3,"color":[1,2,3],"direction":0},"updated_at":"local"}"#).unwrap()
    }
    #[test]
    fn schema_and_bounds() {
        let mut p = profile();
        p.lighting.speed = 5;
        assert!(p.validate().is_err());
        p = profile();
        p.id = "../bad".into();
        assert!(p.validate().is_err());
        assert!(decode_profile("{}").is_err());
    }
    #[test]
    fn event_validation() {
        let mut p = profile();
        p.macros.push(Macro {
            id: "m".into(),
            name: "M".into(),
            events: vec![MacroEvent {
                kind: "delay".into(),
                key: Some(4),
                ms: Some(5),
            }],
        });
        assert!(p.validate().is_err());
    }
    #[test]
    fn macro_balance_is_enforced_by_storage_not_only_the_ui() {
        let mut p = profile();
        let press = MacroEvent {
            kind: "down".into(),
            key: Some(4),
            ms: None,
        };
        let release = MacroEvent {
            kind: "up".into(),
            key: Some(4),
            ms: None,
        };
        p.macros.push(Macro {
            id: "balanced".into(),
            name: "Balanced".into(),
            events: vec![press.clone(), release.clone()],
        });
        p.validate().unwrap();
        for events in [
            vec![press.clone()],
            vec![release.clone()],
            vec![press.clone(), press, release],
        ] {
            p.macros[0].events = events;
            assert!(p.validate().is_err());
        }
    }
    #[test]
    fn save_import_delete_history() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/service-tests")
            .join(new_id());
        let s = Store::new(root.clone()).unwrap();
        let p = profile();
        s.save(&p).unwrap();
        s.save(&p).unwrap();
        assert_eq!(s.list().unwrap().len(), 1);
        let imported = s.import(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_ne!(imported.id, p.id);
        s.delete(&p.id).unwrap();
        assert_eq!(s.list().unwrap().len(), 1);
        assert_eq!(fs::read_dir(root.join("history")).unwrap().count(), 2);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn migration() {
        let json = serde_json::to_value(profile()).unwrap();
        let mut v = json;
        v.as_object_mut().unwrap().remove("schema_version");
        assert_eq!(decode_profile(&v.to_string()).unwrap().schema_version, 1);
    }
    #[test]
    fn optional_media_assignments_roundtrip_without_changing_old_profiles() {
        let mut p = profile();
        assert!(p.consumer_keymap.is_empty());
        assert!(!serde_json::to_string(&p)
            .unwrap()
            .contains("consumer_keymap"));
        p.consumer_keymap.insert("4".into(), 0xcd);
        p.consumer_fn_layer.insert("4".into(), 0xe2);
        assert_eq!(
            decode_profile(&serde_json::to_string(&p).unwrap()).unwrap(),
            p
        );
        p.keymap.insert("04".into(), 5);
        assert!(p.validate().is_err());
        p.keymap.clear();
        p.consumer_keymap.insert("5".into(), 0xb0);
        assert!(p.validate().is_err());
    }
    #[test]
    fn preferences_use_config_location_and_preserve_legacy_on_migration() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/service-tests")
            .join(new_id());
        let data = root.join("data");
        let config = root.join("config");
        let store = Store::with_config_dir(data.clone(), config.clone()).unwrap();
        let legacy = data.join("preferences.json");
        let original = br#"{"demo_mode":true,"reduce_motion":true}"#;
        fs::write(&legacy, original).unwrap();
        assert!(store.preferences().unwrap().demo_mode);
        assert!(
            !config.exists(),
            "Reading must not migrate or create config files"
        );
        store
            .save_preferences(&Preferences {
                demo_mode: false,
                reduce_motion: true,
                ..Preferences::default()
            })
            .unwrap();
        assert!(!store.preferences().unwrap().demo_mode);
        assert!(store.preferences().unwrap().reduce_motion);
        assert_eq!(fs::read(&legacy).unwrap(), original);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(config.join("preferences.json"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn custom_palette_roundtrips_and_rejects_oversized_storage() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/service-tests")
            .join(new_id());
        let store = Store::new(root.clone()).unwrap();
        let mut preferences = Preferences {
            custom_colors: vec![[18, 52, 86]; 10],
            ..Preferences::default()
        };
        store.save_preferences(&preferences).unwrap();
        assert_eq!(
            store.preferences().unwrap().custom_colors,
            preferences.custom_colors
        );
        preferences.custom_colors.push([255, 0, 0]);
        assert!(store.save_preferences(&preferences).is_err());
        assert_eq!(store.preferences().unwrap().custom_colors.len(), 10);
        fs::write(
            root.join("preferences.json"),
            serde_json::to_vec(&preferences).unwrap(),
        )
        .unwrap();
        assert!(store.preferences().is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn invalid_legacy_preferences_are_not_silently_replaced() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/service-tests")
            .join(new_id());
        let data = root.join("data");
        let config = root.join("config");
        let store = Store::with_config_dir(data.clone(), config.clone()).unwrap();
        fs::write(data.join("preferences.json"), b"invalid JSON").unwrap();
        assert!(store.preferences().is_err());
        assert!(!config.exists());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn relative_xdg_roots_use_the_standard_fallback() {
        let fallback = PathBuf::from("/user/.config");
        assert_eq!(
            xdg_base(Some("relative".into()), fallback.clone()),
            fallback
        );
        assert_eq!(xdg_base(Some("".into()), fallback.clone()), fallback);
        assert_eq!(
            xdg_base(Some("/custom/config".into()), fallback),
            PathBuf::from("/custom/config")
        );
    }
    #[test]
    fn oversized_save_preserves_readable_original() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/service-tests")
            .join(new_id());
        let store = Store::new(root.clone()).unwrap();
        let p = profile();
        store.save(&p).unwrap();
        let mut huge = p.clone();
        huge.macros = (0..8)
            .map(|i| Macro {
                id: format!("m-{i}"),
                name: "Macro".into(),
                events: vec![
                    MacroEvent {
                        kind: "delay".into(),
                        key: None,
                        ms: Some(1)
                    };
                    2048
                ],
            })
            .collect();
        assert!(store.save(&huge).is_err());
        assert_eq!(store.get(&p.id).unwrap(), p);
        assert_eq!(fs::read_dir(root.join("history")).unwrap().count(), 0);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn fn_color_is_allowed_without_fn_remapping() {
        let mut p = profile();
        p.per_key.insert("fn".into(), [1, 2, 3]);
        p.validate().unwrap();
        p.keymap.insert("fn".into(), 4);
        assert!(p.validate().is_err());
    }
}
