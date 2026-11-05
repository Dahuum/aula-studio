//! Evidence-bounded model metadata and local workspaces. No HID handles or writes.
use aula_service::{new_id, valid_id, Macro};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::OnceLock,
};

pub mod f2088;
pub mod f98;
pub mod g5;
pub mod g5_capture;
pub mod sonix;

fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or("Missing profile parent")?;
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(parent).map_err(|e| e.to_string())?;
    aula_service::atomic_write(path, data)
}

pub type Result<T> = std::result::Result<T, String>;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct UsbIdentity {
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface_number: Option<i32>,
    pub connection: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_name: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Key {
    pub id: String,
    pub label: String,
    pub usage: Option<u16>,
    pub matrix_index: Option<u16>,
    pub light_index: Option<u16>,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub protected: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Effect {
    pub id: u16,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub controls: Option<EffectControls>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EffectControls {
    pub brightness: bool,
    pub speed: bool,
    pub color: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Lighting {
    pub brightness_max: u8,
    pub speed_max: u8,
    pub default_effect: u16,
    pub effects: Vec<Effect>,
    pub per_key: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub source_url: String,
    pub installer_sha256: String,
    pub configuration_path: String,
    pub configuration_sha256: String,
    pub level: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub device_kind: String,
    pub family: String,
    pub status: String,
    pub usb: Vec<UsbIdentity>,
    pub model_identity: Option<[u8; 6]>,
    pub keys: Vec<Key>,
    pub lighting: Option<Lighting>,
    pub fn_layer: bool,
    pub notes: Vec<String>,
    pub evidence: Vec<Evidence>,
}
impl Model {
    pub fn validate(&self) -> Result<()> {
        if !valid_id(&self.id)
            || self.name.is_empty()
            || self.name.len() > 120
            || self.keys.len() > 256
        {
            return Err("Invalid model identity or layout bounds".into());
        }
        if self.usb.iter().any(|u| {
            u.product_name
                .as_ref()
                .is_some_and(|s| s.is_empty() || s.len() > 256)
        }) {
            return Err("Invalid configured USB product string".into());
        }
        let mut ids = BTreeSet::new();
        for k in &self.keys {
            if !valid_id(&k.id)
                || !ids.insert(k.id.clone())
                || k.label.len() > 40
                || ![k.x, k.y, k.width, k.height]
                    .iter()
                    .all(|x| x.is_finite() && *x >= 0.0 && *x < 4096.0)
                || k.width <= 0.0
                || k.height <= 0.0
                || k.usage.is_some_and(|u| u == 0 || u > 255)
            {
                return Err(format!("Invalid/duplicate key in {}", self.id));
            }
        }
        if let Some(l) = &self.lighting {
            if l.brightness_max > 100
                || l.speed_max > 100
                || l.effects.is_empty()
                || l.effects.len() > 128
                || !l.effects.iter().any(|e| e.id == l.default_effect)
            {
                return Err("Invalid lighting metadata".into());
            }
            let mut ids = BTreeSet::new();
            if l.effects
                .iter()
                .any(|e| !ids.insert(e.id) || e.name.is_empty() || e.name.len() > 80)
            {
                return Err("Invalid/duplicate effect metadata".into());
            }
        }
        Ok(())
    }
    pub fn editable(&self) -> bool {
        self.id != "f75-ansi" && self.device_kind == "keyboard" && !self.keys.is_empty()
    }
}

pub fn registry() -> Result<&'static [Model]> {
    static MODELS: OnceLock<Result<Vec<Model>>> = OnceLock::new();
    MODELS
        .get_or_init(|| {
            let models: Vec<Model> =
                serde_json::from_str(include_str!("../../../data/models.json"))
                    .map_err(|e| e.to_string())?;
            let mut ids = BTreeSet::new();
            for model in &models {
                model.validate()?;
                if !ids.insert(model.id.clone()) {
                    return Err("Duplicate model id".into());
                }
            }
            Ok(models)
        })
        .as_ref()
        .map(|v| v.as_slice())
        .map_err(Clone::clone)
}
pub fn model(id: &str) -> Result<&'static Model> {
    registry()?
        .iter()
        .find(|m| m.id == id)
        .ok_or_else(|| "Unknown model id".into())
}
/// Candidates are metadata only: shared VID/PIDs are deliberately not resolved.
pub fn usb_candidates(vid: u16, pid: u16, interface: i32) -> Result<Vec<&'static Model>> {
    Ok(registry()?
        .iter()
        .filter(|m| {
            m.usb.iter().any(|u| {
                u.vendor_id == vid
                    && u.product_id == pid
                    && u.interface_number.is_none_or(|i| i == interface)
            })
        })
        .collect())
}
/// Prioritize an exact configured product string when available. This is only
/// metadata matching, never revision attestation or hardware access permission.
/// Unknown strings preserve all USB candidates instead of guessing a model.
pub fn usb_candidates_with_product(
    vid: u16,
    pid: u16,
    interface: i32,
    product: &str,
) -> Result<(Vec<&'static Model>, bool)> {
    let candidates = usb_candidates(vid, pid, interface)?;
    let named: Vec<_> = candidates
        .iter()
        .copied()
        .filter(|m| {
            m.usb.iter().any(|u| {
                u.vendor_id == vid
                    && u.product_id == pid
                    && u.interface_number.is_none_or(|i| i == interface)
                    && u.product_name.as_deref() == Some(product)
                    && !product.is_empty()
            })
        })
        .collect();
    if named.is_empty() {
        Ok((candidates, false))
    } else {
        Ok((named, true))
    }
}
pub fn identity_candidates(identity: [u8; 6]) -> Result<Vec<&'static Model>> {
    Ok(registry()?
        .iter()
        .filter(|m| m.model_identity == Some(identity))
        .collect())
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(
    tag = "kind",
    content = "usage",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Assignment {
    Keyboard(u16),
    Consumer(u16),
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LightingDraft {
    pub effect: u16,
    pub brightness: u8,
    pub speed: u8,
    pub color: [u8; 3],
}
impl LightingDraft {
    fn for_model(model: &Model) -> Option<Self> {
        model.lighting.as_ref().map(|lighting| Self {
            effect: lighting.default_effect,
            brightness: lighting.brightness_max,
            speed: lighting.speed_max / 2,
            color: [255, 96, 32],
        })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub model_id: String,
    pub lighting: Option<LightingDraft>,
    pub per_key: BTreeMap<String, [u8; 3]>,
    pub assignments: BTreeMap<String, Assignment>,
    pub fn_assignments: BTreeMap<String, Assignment>,
    pub macros: Vec<Macro>,
}
impl Draft {
    pub fn new(model_id: &str, name: &str) -> Result<Self> {
        let m = model(model_id)?;
        if !m.editable() {
            return Err("No resource layout has been recovered for this model".into());
        }
        let p = Self {
            schema_version: 1,
            id: new_id(),
            name: name.into(),
            model_id: model_id.into(),
            lighting: LightingDraft::for_model(m),
            per_key: BTreeMap::new(),
            assignments: BTreeMap::new(),
            fn_assignments: BTreeMap::new(),
            macros: vec![],
        };
        p.validate()?;
        Ok(p)
    }
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1
            || !valid_id(&self.id)
            || self.name.trim().is_empty()
            || self.name.len() > 120
        {
            return Err("Unsupported model-profile schema or identity".into());
        }
        let m = model(&self.model_id)?;
        if !m.editable() {
            return Err("Model has no recovered keyboard layout".into());
        }
        match (&self.lighting, &m.lighting) {
            (Some(p), Some(c))
                if c.effects.iter().any(|e| e.id == p.effect)
                    && p.brightness <= c.brightness_max
                    && p.speed <= c.speed_max => {}
            (None, None) => (),
            _ => return Err("Lighting is outside this model's documented capabilities".into()),
        }
        if !self.fn_assignments.is_empty() && !m.fn_layer {
            return Err("Fn-layer encoding/capability is not established for this model".into());
        }
        for key in self.per_key.keys() {
            if !m.lighting.as_ref().is_some_and(|l| l.per_key)
                || !m.keys.iter().any(|k| &k.id == key)
            {
                return Err("Per-key color references an unsupported key/model".into());
            }
        }
        for (key, action) in self.assignments.iter().chain(self.fn_assignments.iter()) {
            if !m
                .keys
                .iter()
                .any(|k| &k.id == key && k.usage.is_some() && !k.protected)
            {
                return Err("Remapping source is unknown or a protected control".into());
            }
            match action {
                Assignment::Keyboard(u) if (1..=255).contains(u) => (),
                Assignment::Consumer(u)
                    if [0xcd, 0xb7, 0xb6, 0xb5, 0xe9, 0xea, 0xe2].contains(u) => {}
                _ => return Err("Unsupported assignment usage".into()),
            }
        }
        let mut macro_ids = BTreeSet::new();
        if self.macros.iter().any(|m| !macro_ids.insert(&m.id)) {
            return Err("Duplicate local macro id".into());
        }
        aula_service::validate_macros(&self.macros)?;
        Ok(())
    }
}
pub fn decode_draft(text: &str) -> Result<Draft> {
    if text.len() > 512 * 1024 {
        return Err("Model profile exceeds512 KiB".into());
    }
    let mut p: Draft = serde_json::from_str(text).map_err(|e| e.to_string())?;
    // Earlier schema1 F98 workspaces had no lighting definitions. Keep their
    // identity/assignments/macros and add only the newly available local draft.
    // Loading never rewrites the saved file or configures a device.
    if p.schema_version == 1 && p.model_id == "f98-pro" && p.lighting.is_none() {
        p.lighting = LightingDraft::for_model(model("f98-pro")?);
    }
    p.validate()?;
    Ok(p)
}
/// Local workspaces have a separate namespace and incompatible shape from F75
/// hardware profiles; they cannot be sent to the legacy prepare/write commands.
pub struct Store(PathBuf);
impl Store {
    pub fn new(app_data: &Path) -> Self {
        Self(app_data.join("model-profiles"))
    }
    fn path(&self, id: &str) -> Result<PathBuf> {
        if !valid_id(id) {
            return Err("Invalid profile id".into());
        }
        Ok(self.0.join(format!("{id}.json")))
    }
    pub fn list(&self) -> Result<Vec<Draft>> {
        if !self.0.exists() {
            return Ok(vec![]);
        }
        let mut files = std::fs::read_dir(&self.0)
            .map_err(|e| e.to_string())?
            .collect::<std::io::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?;
        files.sort_by_key(|e| e.file_name());
        if files.len() > 4096 {
            return Err("Too many model profiles".into());
        }
        files
            .into_iter()
            .filter(|e| e.path().extension().is_some_and(|v| v == "json"))
            .map(|e| {
                self.get(
                    e.path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .ok_or("Invalid profile filename")?,
                )
            })
            .collect()
    }
    pub fn get(&self, id: &str) -> Result<Draft> {
        let path = self.path(id)?;
        if std::fs::metadata(&path).map_err(|e| e.to_string())?.len() > 512 * 1024 {
            return Err("Model profile exceeds512 KiB".into());
        }
        decode_draft(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)
    }
    pub fn save(&self, p: &Draft) -> Result<()> {
        p.validate()?;
        let data = serde_json::to_vec_pretty(p).map_err(|e| e.to_string())?;
        if data.len() > 512 * 1024 {
            return Err("Model profile exceeds512 KiB".into());
        }
        let path = self.path(&p.id)?;
        if path.exists() {
            let old = self.get(&p.id)?;
            if old.model_id != p.model_id {
                return Err("Cannot overwrite a profile belonging to another model".into());
            }
            atomic_write(
                &self
                    .0
                    .join("history")
                    .join(format!("{}-{}.json", p.id, new_id())),
                &std::fs::read(&path).map_err(|e| e.to_string())?,
            )?;
        }
        atomic_write(&path, &data)
    }
    pub fn delete(&self, id: &str) -> Result<()> {
        let p = self.get(id)?;
        atomic_write(
            &self
                .0
                .join("history")
                .join(format!("deleted-{}-{}.json", id, new_id())),
            &serde_json::to_vec_pretty(&p).map_err(|e| e.to_string())?,
        )?;
        std::fs::remove_file(self.path(id)?).map_err(|e| e.to_string())
    }
    pub fn import(&self, text: &str) -> Result<Draft> {
        let mut p = decode_draft(text)?;
        p.id = new_id();
        self.save(&p)?;
        Ok(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn f98_lighting_definitions_are_model_specific_and_do_not_grant_g5_access() {
        let model = model("f98-pro").unwrap();
        let lighting = model.lighting.as_ref().unwrap();
        assert_eq!((lighting.brightness_max, lighting.speed_max), (5, 5));
        assert_eq!(
            lighting.effects.iter().map(|e| e.id).collect::<Vec<_>>(),
            (0..20).collect::<Vec<_>>()
        );
        assert!(lighting.per_key);
        assert!(!lighting.effects[0].controls.as_ref().unwrap().brightness);
        assert!(!lighting.effects[1].controls.as_ref().unwrap().speed);
        assert!(!lighting.effects[6].controls.as_ref().unwrap().color);
        assert!(!lighting.effects[8].controls.as_ref().unwrap().color);
        assert!(model.model_identity.is_none());
        assert!(g5_capture::expected_identity("f98-pro").is_err());
        let mut draft = Draft::new("f98-pro", "New lighting").unwrap();
        for (effect, brightness, speed) in [(20, 5, 2), (1, 6, 2), (1, 5, 6)] {
            let value = draft.lighting.as_mut().unwrap();
            value.effect = effect;
            value.brightness = brightness;
            value.speed = speed;
            assert!(draft.validate().is_err());
        }
    }
    #[test]
    fn legacy_f98_lighting_migration_preserves_work_without_rewriting_the_file() {
        let root = std::env::temp_dir().join(format!("aula-f98-migration-{}", new_id()));
        let store = Store::new(&root);
        let mut old = Draft::new("f98-pro", "Existing F98 work").unwrap();
        old.lighting = None;
        let source = model("f98-pro")
            .unwrap()
            .keys
            .iter()
            .find(|k| k.usage == Some(4))
            .unwrap();
        old.assignments
            .insert(source.id.clone(), Assignment::Keyboard(5));
        old.macros.push(Macro {
            id: "macro1".into(),
            name: "Existing macro".into(),
            events: vec![
                aula_service::MacroEvent {
                    kind: "down".into(),
                    key: Some(4),
                    ms: None,
                },
                aula_service::MacroEvent {
                    kind: "up".into(),
                    key: Some(4),
                    ms: None,
                },
            ],
        });
        let original = serde_json::to_vec(&old).unwrap();
        std::fs::create_dir_all(&store.0).unwrap();
        let file = store.path(&old.id).unwrap();
        std::fs::write(&file, &original).unwrap();
        let migrated = store.get(&old.id).unwrap();
        assert_eq!(migrated.id, old.id);
        assert_eq!(migrated.name, old.name);
        assert_eq!(migrated.assignments, old.assignments);
        assert_eq!(migrated.macros, old.macros);
        assert_eq!(
            migrated.lighting,
            LightingDraft::for_model(model("f98-pro").unwrap())
        );
        assert_eq!(std::fs::read(&file).unwrap(), original);
        store.save(&migrated).unwrap();
        assert_ne!(std::fs::read(&file).unwrap(), original);
        assert_eq!(store.get(&old.id).unwrap(), migrated);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn lighting_migration_never_guesses_another_model_or_schema() {
        let mut draft = Draft::new("f87", "No guessed migration").unwrap();
        draft.lighting = None;
        assert!(decode_draft(&serde_json::to_string(&draft).unwrap()).is_err());
        draft.model_id = "f98-pro".into();
        draft.schema_version = 2;
        assert!(decode_draft(&serde_json::to_string(&draft).unwrap()).is_err());
    }
    #[test]
    fn catalogue_is_valid_and_shared_usb_ids_remain_ambiguous() {
        let models = registry().unwrap();
        assert!(models.len() >= 10);
        let g5 = usb_candidates(0x258a, 0x010c, 1).unwrap();
        assert!(g5.iter().any(|m| m.id == "f75-ansi"));
        assert!(g5.iter().any(|m| m.id == "f87"));
        assert!(usb_candidates(0x0c45, 0x800a, 0).unwrap().len() >= 2);
        assert!(identity_candidates([3, 0, 0, 0, 0, 0x8f])
            .unwrap()
            .iter()
            .all(|m| m.id == "f87"));
    }
    #[test]
    fn f2088_partial_resource_layout_never_inherits_rgb_fn_or_g5_access() {
        let m = model("f2088").unwrap();
        assert_eq!(m.keys.len(), 103);
        assert!(m
            .keys
            .iter()
            .all(|k| k.matrix_index.is_none() && k.light_index.is_none()));
        assert!(!m.fn_layer && m.lighting.is_none() && m.model_identity.is_none());
        assert!(!m.keys.iter().any(|k| k.usage == Some(231)));
        assert!(g5_capture::expected_identity("f2088").is_err());
        let candidates = usb_candidates(0x1a2c, 0x9ef4, 1).unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].id, "f2088");
        let mut draft = Draft::new("f2088", "Partial layout").unwrap();
        draft
            .assignments
            .insert("key-4".into(), Assignment::Keyboard(5));
        draft.validate().unwrap();
        draft.per_key.insert("key-4".into(), [1, 2, 3]);
        assert!(draft.validate().is_err());
    }
    #[test]
    fn configured_product_matching_is_exact_and_never_implies_protocol_support() {
        let (s75, matched) = usb_candidates_with_product(0x0c45, 0x800a, 0, "AULA-S75Pro").unwrap();
        assert!(matched);
        assert_eq!(s75.len(), 1);
        assert_eq!(s75[0].id, "s75-pro");
        let (f108, matched) =
            usb_candidates_with_product(0x0c45, 0x800a, 0, "AULA F108Pro").unwrap();
        assert!(matched);
        assert_eq!(f108[0].id, "f108-pro");
        for name in ["aula-s75pro", "AULA-S75Pro ", "unknown", ""] {
            let (all, matched) = usb_candidates_with_product(0x0c45, 0x800a, 0, name).unwrap();
            assert!(!matched);
            assert!(all.len() >= 3);
        }
        let (all, matched) =
            usb_candidates_with_product(0x258a, 0x010c, 1, "USB Gaming Keyboard").unwrap();
        assert!(!matched);
        assert!(all.len() > 1);
    }
    #[test]
    fn profiles_bind_to_model_and_reject_cross_model_and_protected_keys() {
        let mut p = Draft::new("s75-pro", "Desktop").unwrap();
        p.validate().unwrap();
        p.assignments
            .insert("unknown".into(), Assignment::Keyboard(4));
        assert!(p.validate().is_err());
        p.assignments.clear();
        let protected = model("s75-pro")
            .unwrap()
            .keys
            .iter()
            .find(|k| k.protected)
            .unwrap();
        p.assignments
            .insert(protected.id.clone(), Assignment::Keyboard(4));
        assert!(p.validate().is_err());
        p.assignments.clear();
        p.lighting.as_mut().unwrap().brightness = 6;
        assert!(p.validate().is_err());
    }
    #[test]
    fn profile_json_roundtrips_and_is_not_a_legacy_hardware_profile() {
        let p = Draft::new("f87", "Local").unwrap();
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(decode_draft(&json).unwrap(), p);
        assert!(aula_service::decode_profile(&json).is_err());
        assert!(Draft::new("sc620", "No layout").is_err());
        assert!(Draft::new("f75-ansi", "Use the existing controller").is_err());
    }
    #[test]
    fn local_store_preserves_history_and_refuses_cross_model_overwrite() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/model-store-tests")
            .join(new_id());
        let store = Store::new(&root);
        let mut p = Draft::new("f87", "Test").unwrap();
        store.save(&p).unwrap();
        p.name = "Renamed".into();
        store.save(&p).unwrap();
        assert_eq!(store.list().unwrap().len(), 1);
        let mut wrong = Draft::new("s75-pro", "Different model").unwrap();
        wrong.id = p.id.clone();
        assert!(store.save(&wrong).is_err());
        let imported = store.import(&serde_json::to_string(&p).unwrap()).unwrap();
        assert_ne!(imported.id, p.id);
        store.delete(&p.id).unwrap();
        assert_eq!(store.list().unwrap().len(), 1);
        assert!(store.get("../outside").is_err());
        assert!(
            root.join("model-profiles/history")
                .read_dir()
                .unwrap()
                .count()
                >= 2
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn macro_validation_and_size_limits_are_not_bypassed_by_model_workspaces() {
        let mut p = Draft::new("s99-pro", "Macro").unwrap();
        p.macros = vec![Macro {
            id: "macro1".into(),
            name: "Unbalanced".into(),
            events: vec![aula_service::MacroEvent {
                kind: "down".into(),
                key: Some(4),
                ms: None,
            }],
        }];
        assert!(p.validate().is_err());
        assert!(decode_draft(&" ".repeat(512 * 1024 + 1)).is_err());
    }
}
