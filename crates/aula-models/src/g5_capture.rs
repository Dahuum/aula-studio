//! Bounded model-specific G5 reads, each backed by its own native installer.
//! No setters or generic command constructor. Other G5 models are not inferred.
use crate::{g5, model, Draft, Result};
use aula_protocol::{ColorTable, REPORT_SIZE};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const KEYMAP_SLOTS: usize = 128;
// ApplySetting0048afa0 uses126-byte color-plane strides (ushort offset0x3f),
// but transfers MatrixLen*3=384 bytes in this older F87 binary. The final six
// bytes have no established LED meaning and are preserved without decoding.
pub const RGB_PLANE_LEN: usize = 126;
pub const LEGACY_RGB_TRANSFER_LEN: usize = 384;
pub const STANDARD_RGB_TRANSFER_LEN: usize = 378;
pub const LEGACY_MODELS: &[&str] = &["f87", "f87-pro-010b", "f87-pro-wired"];
pub const MODELS: &[&str] = &["f87", "f87-pro-010b", "f87-pro-wired", "f108", "h108"];

pub fn per_key_request(model_id: &str) -> Result<ReadRequest> {
    expected_identity(model_id)?;
    Ok(if LEGACY_MODELS.contains(&model_id) {
        ReadRequest::PerKeyLegacy384
    } else {
        ReadRequest::PerKeyStandard378
    })
}
pub fn per_key_len(model_id: &str) -> Result<usize> {
    let request = per_key_request(model_id)?.bytes();
    Ok(u16::from_le_bytes([request[6], request[7]]) as usize)
}

pub fn expected_identity(model_id: &str) -> Result<[u8; 6]> {
    if !MODELS.contains(&model_id) {
        return Err("This model has no independently established G5 read contract".into());
    }
    model(model_id)?
        .model_identity
        .ok_or("Missing model identity".into())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadRequest {
    Identity,
    Config,
    Palette,
    Base,
    PerKeyLegacy384,
    PerKeyStandard378,
}
impl ReadRequest {
    pub fn bytes(self) -> [u8; REPORT_SIZE] {
        if self == Self::Identity {
            return g5::identity_request();
        }
        let mut b = [0; REPORT_SIZE];
        let (command, size) = match self {
            Self::Config => (0x84, 128),
            Self::Palette => (0x8a, 512),
            Self::Base => (0x83, 512),
            Self::PerKeyLegacy384 => (0x86, LEGACY_RGB_TRANSFER_LEN),
            Self::PerKeyStandard378 => (0x86, STANDARD_RGB_TRANSFER_LEN),
            Self::Identity => unreachable!(),
        };
        b[..8].copy_from_slice(&[6, command, 0, 0, 1, 0, size as u8, (size >> 8) as u8]);
        b
    }
    pub fn payload(self, response: &[u8]) -> Result<Vec<u8>> {
        if self == Self::Identity {
            return g5::decode_identity(response)
                .map(|v| v.to_vec())
                .map_err(|e| e.to_string());
        }
        let request = self.bytes();
        let size = u16::from_le_bytes([request[6], request[7]]) as usize;
        if response.len() != REPORT_SIZE && response.len() != size + 8 {
            return Err(
                "Incomplete Model response; no padding or partial snapshot is accepted".into(),
            );
        }
        if response[0] != 6
            || ![request[1], request[1] & 0x7f].contains(&response[1])
            || response[2..8] != request[2..8]
        {
            return Err("Model response scope/length header mismatch".into());
        }
        let payload = response[8..8 + size].to_vec();
        self.validate_bank(&payload)?;
        Ok(payload)
    }
    /// Validate revision-specific bank rules before querying the next bank.
    pub fn payload_for_model(self, model_id: &str, response: &[u8]) -> Result<Vec<u8>> {
        expected_identity(model_id)?;
        if matches!(self, Self::PerKeyLegacy384 | Self::PerKeyStandard378)
            && self != per_key_request(model_id)?
        {
            return Err("RGB transfer request differs from the selected model contract".into());
        }
        let payload = self.payload(response)?;
        if self == Self::Config
            && !LEGACY_MODELS.contains(&model_id)
            && payload[126..128] != [0x5a, 0xa5]
        {
            return Err("Standard G5 configuration signature absent".into());
        }
        Ok(payload)
    }
    fn validate_bank(self, payload: &[u8]) -> Result<()> {
        match self {
            Self::Config if payload[56..58] != [0xff, 0xff] => {
                Err("G5 configuration sentinel absent; unsupported/empty response".into())
            }
            Self::Palette if payload[506..508] != [0x5a, 0xa5] => {
                Err("G5 palette marker absent".into())
            }
            Self::Base if payload[510..512] != [0x5a, 0xa5] => {
                Err("G5 base matrix marker absent".into())
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CaptureSource {
    PhysicalRead,
    Simulated,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Capture {
    pub schema_version: u32,
    pub id: String,
    pub model_id: String,
    pub identity: [u8; 6],
    pub source: CaptureSource,
    pub config: Vec<u8>,
    pub palette: Vec<u8>,
    pub base: Vec<u8>,
    pub per_key: Vec<u8>,
}
impl Capture {
    pub fn decode(
        model_id: &str,
        identity: [u8; 6],
        source: CaptureSource,
        reports: [Vec<u8>; 4],
    ) -> Result<Self> {
        if identity != expected_identity(model_id)? {
            return Err("Physical identity differs from the selected model".into());
        }
        let [config, palette, base, per_key] = reports;
        let result = Self {
            schema_version: 1,
            id: aula_service::new_id(),
            model_id: model_id.into(),
            identity,
            source,
            config: ReadRequest::Config.payload(&config)?,
            palette: ReadRequest::Palette.payload(&palette)?,
            base: ReadRequest::Base.payload(&base)?,
            per_key: per_key_request(model_id)?.payload(&per_key)?,
        };
        result.validate()?;
        Ok(result)
    }
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1
            || !aula_service::valid_id(&self.id)
            || self.identity != expected_identity(&self.model_id)?
        {
            return Err("Invalid model-bound capture identity/schema".into());
        }
        if self.config.len() != 128
            || self.palette.len() != 512
            || self.base.len() != 512
            || self.per_key.len() != per_key_len(&self.model_id)?
        {
            return Err("Capture bank sizes differ from this model contract".into());
        }
        // FillCfg0048abf0 sets the config sentinel; ApplySetting0048afa0
        // sets the palette marker. Licensed public F87 fixtures corroborate
        // the matrix marker. These also reject untouched zero request echoes.
        ReadRequest::Config.validate_bank(&self.config)?;
        if !LEGACY_MODELS.contains(&self.model_id.as_str()) && self.config[126..128] != [0x5a, 0xa5]
        {
            return Err("Standard G5 configuration signature absent".into());
        }
        ReadRequest::Palette.validate_bank(&self.palette)?;
        ReadRequest::Base.validate_bank(&self.base)?;
        Ok(())
    }
    /// Create local intent without assuming unknown mappings or Fn-layer roles.
    /// Reserved records are retained in the capture and not projected as defaults.
    pub fn lighting_draft(&self, name: &str) -> Result<Draft> {
        self.validate()?;
        let mut draft = Draft::new(&self.model_id, name)?;
        let effect = self.config[10] as u16;
        let supported = model(&self.model_id)?
            .lighting
            .as_ref()
            .ok_or("Missing lighting capability")?;
        if !supported.effects.iter().any(|e| e.id == effect) {
            return Err(
                "Captured effect is not in this revision's resource; raw capture retained".into(),
            );
        }
        let offset = 56 + effect as usize * 2;
        let (brightness, speed) = if effect == 0 || effect == 21 {
            (0, 0)
        } else {
            (self.config[offset], self.config[offset + 1] >> 4)
        };
        if (1..=18).contains(&effect) && self.config[offset + 1] & 0x0f != 0 {
            return Err("The captured palette selector cannot be represented by a fixed-color draft; raw capture is retained".into());
        }
        let color = if (1..=18).contains(&effect) {
            ColorTable::parse(&self.palette)
                .and_then(|p| p.effect_color(effect as u8))
                .map_err(|e| e.to_string())?
        } else {
            [0; 3]
        };
        let lighting = draft.lighting.as_mut().ok_or("Missing lighting draft")?;
        lighting.effect = effect;
        lighting.brightness = brightness;
        lighting.speed = speed;
        lighting.color = color;
        draft.per_key = self.key_colors()?;
        draft.validate()?;
        Ok(draft)
    }
    pub fn key_colors(&self) -> Result<BTreeMap<String, [u8; 3]>> {
        self.validate()?;
        let mut result = BTreeMap::new();
        for key in &model(&self.model_id)?.keys {
            if let Some(index) = key.light_index {
                let i = index as usize;
                if i >= RGB_PLANE_LEN {
                    return Err("Resource LED index exceeds the established RGB plane".into());
                }
                result.insert(
                    key.id.clone(),
                    [
                        self.per_key[i],
                        self.per_key[RGB_PLANE_LEN + i],
                        self.per_key[2 * RGB_PLANE_LEN + i],
                    ],
                );
            }
        }
        Ok(result)
    }
}

/// Immutable model-specific captures. No import/restore or legacy backup path.
pub struct CaptureStore(std::path::PathBuf);
impl CaptureStore {
    pub fn new(root: &std::path::Path) -> Self {
        Self(root.join("model-captures"))
    }
    fn path(&self, id: &str) -> Result<std::path::PathBuf> {
        if !aula_service::valid_id(id) {
            return Err("Invalid capture id".into());
        }
        Ok(self.0.join(format!("{id}.json")))
    }
    pub fn save(&self, capture: &Capture) -> Result<()> {
        capture.validate()?;
        let path = self.path(&capture.id)?;
        if path.exists() {
            return Err("Saved captures are immutable; use a fresh read".into());
        }
        let data = serde_json::to_vec_pretty(capture).map_err(|e| e.to_string())?;
        if data.len() > 32768 {
            return Err("Capture exceeds bounded storage size".into());
        }
        super::atomic_write(&path, &data)
    }
    pub fn get(&self, id: &str) -> Result<Capture> {
        let path = self.path(id)?;
        if std::fs::metadata(&path).map_err(|e| e.to_string())?.len() > 32768 {
            return Err("Capture exceeds bounded storage size".into());
        }
        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        let c: Capture = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
        c.validate()?;
        if c.id != id {
            return Err("Capture file identity mismatch".into());
        }
        Ok(c)
    }
    pub fn list(&self, model_id: &str) -> Result<Vec<Capture>> {
        expected_identity(model_id)?;
        if !self.0.exists() {
            return Ok(vec![]);
        }
        let files = std::fs::read_dir(&self.0)
            .map_err(|e| e.to_string())?
            .collect::<std::io::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?;
        if files.len() > 512 {
            return Err("Too many saved captures; archive captures before reading more".into());
        }
        let mut result = vec![];
        for file in files {
            let p = file.path();
            if p.extension().is_some_and(|s| s == "json") {
                let c = self.get(
                    p.file_stem()
                        .and_then(|s| s.to_str())
                        .ok_or("Invalid capture filename")?,
                )?;
                if c.model_id == model_id {
                    result.push(c);
                }
            }
        }
        result.sort_by(|a, b| b.id.cmp(&a.id));
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_read_scope_has_exact_length_id_and_no_write_command() {
        for (request, header) in [
            (ReadRequest::Config, [6, 0x84, 0, 0, 1, 0, 0x80, 0]),
            (ReadRequest::Palette, [6, 0x8a, 0, 0, 1, 0, 0, 2]),
            (ReadRequest::Base, [6, 0x83, 0, 0, 1, 0, 0, 2]),
            (ReadRequest::PerKeyLegacy384, [6, 0x86, 0, 0, 1, 0, 0x80, 1]),
            (
                ReadRequest::PerKeyStandard378,
                [6, 0x86, 0, 0, 1, 0, 0x7a, 1],
            ),
        ] {
            let b = request.bytes();
            assert_eq!(&b[..8], &header);
            assert!(b[8..].iter().all(|v| *v == 0));
            assert!(b[1] & 0x80 != 0);
        }
    }
    fn valid_capture() -> Capture {
        let mut reports = [
            ReadRequest::Config.bytes().to_vec(),
            ReadRequest::Palette.bytes().to_vec(),
            ReadRequest::Base.bytes().to_vec(),
            ReadRequest::PerKeyLegacy384.bytes().to_vec(),
        ];
        reports[0][8 + 56..8 + 58].copy_from_slice(&[0xff, 0xff]);
        reports[0][8 + 10] = 2;
        reports[1][8 + 506..8 + 508].copy_from_slice(&[0x5a, 0xa5]);
        reports[2][8 + 510..8 + 512].copy_from_slice(&[0x5a, 0xa5]);
        Capture::decode(
            "f87",
            expected_identity("f87").unwrap(),
            CaptureSource::Simulated,
            reports,
        )
        .unwrap()
    }
    #[test]
    fn immutable_capture_storage_is_separate_and_checks_file_identity() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/f87-capture-tests")
            .join(aula_service::new_id());
        let store = CaptureStore::new(&root);
        let c = valid_capture();
        store.save(&c).unwrap();
        assert!(store.save(&c).is_err());
        assert_eq!(store.get(&c.id).unwrap(), c);
        assert_eq!(store.list("f87").unwrap().len(), 1);
        assert!(store.list("f87-pro-wired").unwrap().is_empty());
        assert!(super::super::decode_draft(&serde_json::to_string(&c).unwrap()).is_err());
        assert!(aula_service::decode_profile(&serde_json::to_string(&c).unwrap()).is_err());
        assert!(store.get("../outside").is_err());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(store.path(&c.id).unwrap())
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        let mut tampered = c.clone();
        tampered.id = "another-id".into();
        std::fs::write(
            store.path(&c.id).unwrap(),
            serde_json::to_vec(&tampered).unwrap(),
        )
        .unwrap();
        assert!(store.get(&c.id).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn unsupported_scalar_or_palette_selection_never_gets_silently_normalized() {
        let mut c = valid_capture();
        c.config[60] = 9;
        c.validate().unwrap();
        assert!(c.lighting_draft("Captured").is_err());
        c.config[60] = 2;
        c.config[61] = 7;
        c.validate().unwrap();
        assert!(c.lighting_draft("Captured").is_err());
    }
    #[test]
    fn rejects_other_model_lengths_unknown_scopes_and_zero_echoes() {
        let request = ReadRequest::PerKeyLegacy384;
        let mut b = request.bytes();
        assert_eq!(request.payload(&b).unwrap().len(), 384);
        b[6] = 0x7a;
        assert!(request.payload(&b).is_err());
        let mut b = request.bytes();
        b[2] = 1;
        assert!(request.payload(&b).is_err());
        assert!(request.payload(&request.bytes()[..391]).is_err());
        let reports = [
            ReadRequest::Config.bytes().to_vec(),
            ReadRequest::Palette.bytes().to_vec(),
            ReadRequest::Base.bytes().to_vec(),
            ReadRequest::PerKeyLegacy384.bytes().to_vec(),
        ];
        assert!(Capture::decode(
            "f87",
            expected_identity("f87").unwrap(),
            CaptureSource::Simulated,
            reports
        )
        .is_err());
        assert!(expected_identity("f75-ansi").is_err());
        assert!(expected_identity("f108-pro").is_err());
    }
}
