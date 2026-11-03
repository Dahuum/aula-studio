//! Evidence-bounded F75 packets. Encoding a write is not permission to transmit it.
//! All transport policy and explicit user authorization belong to the caller.
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod macro_wire;

pub const SLEEP_SECONDS: &[u16] = &[0, 60, 90, 120, 180, 240, 300, 600, 900, 1200];
pub const REPORT_ID: u8 = 0x06;
pub const REPORT_SIZE: usize = 520;
pub const HEADER_SIZE: usize = 8;
pub const CONFIG_SIZE: usize = 128;
pub const COLORS_SIZE: usize = 512;
// Target 0, reserved 0, one chunk, chunk index 0; these are not memory addresses.
const SINGLE_TARGET_CHUNK: [u8; 4] = [0, 0, 1, 0];

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("Expected {expected} bytes, received {actual}")]
    Length { expected: usize, actual: usize },
    #[error("Invalid report ID, command, target/chunk fields, or length header")]
    Header,
    #[error("Configuration signature 5A A5 is missing")]
    Signature,
    #[error("Effect palette signature 5A A5 at bytes506/507 is missing")]
    PaletteSignature,
    #[error("Unsupported value: {0}")]
    Unsupported(&'static str),
    #[error("Matrix index is out of bounds")]
    Index,
    #[error("Checksum mismatch")]
    Checksum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Region {
    Config,
    Colors,
}
impl Region {
    pub const fn size(self) -> usize {
        match self {
            Self::Config => CONFIG_SIZE,
            Self::Colors => COLORS_SIZE,
        }
    }
    pub const fn read_command(self) -> u8 {
        match self {
            Self::Config => 0x84,
            Self::Colors => 0x8a,
        }
    }
    pub const fn write_command(self) -> u8 {
        match self {
            Self::Config => 0x04,
            Self::Colors => 0x0a,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeatureReport([u8; REPORT_SIZE]);
impl FeatureReport {
    fn header(region: Region, command: u8) -> Self {
        let mut bytes = [0; REPORT_SIZE];
        bytes[0] = REPORT_ID;
        bytes[1] = command;
        bytes[2..6].copy_from_slice(&SINGLE_TARGET_CHUNK);
        bytes[6..8].copy_from_slice(&(region.size() as u16).to_le_bytes());
        Self(bytes)
    }
    pub fn read(region: Region) -> Self {
        Self::header(region, region.read_command())
    }
    /// Encodes a bounded configuration report; this function never transmits.
    pub fn write(region: Region, data: &[u8]) -> Result<Self, ProtocolError> {
        exact_length(data, region.size())?;
        if region == Region::Config {
            ConfigRegion::parse(data)?;
        }
        let mut report = Self::header(region, region.write_command());
        report.0[HEADER_SIZE..HEADER_SIZE + data.len()].copy_from_slice(data);
        Ok(report)
    }
    pub fn as_bytes(&self) -> &[u8; REPORT_SIZE] {
        &self.0
    }
    pub fn into_bytes(self) -> [u8; REPORT_SIZE] {
        self.0
    }
    /// Accept a complete 520B report, or the exact header+region length.
    /// Do not pad short responses or silently accept another command's data.
    pub fn decode_response(region: Region, data: &[u8]) -> Result<Vec<u8>, ProtocolError> {
        if data.len() != REPORT_SIZE && data.len() != HEADER_SIZE + region.size() {
            return Err(ProtocolError::Length {
                expected: HEADER_SIZE + region.size(),
                actual: data.len(),
            });
        }
        let expected = Self::read(region);
        if data[0] != REPORT_ID
            || ![region.read_command(), region.write_command()].contains(&data[1])
            || data[2..8] != expected.0[2..8]
        {
            return Err(ProtocolError::Header);
        }
        let value = data[HEADER_SIZE..HEADER_SIZE + region.size()].to_vec();
        if region == Region::Config {
            ConfigRegion::parse(&value)?;
        }
        Ok(value)
    }
}
fn exact_length(data: &[u8], expected: usize) -> Result<(), ProtocolError> {
    if data.len() == expected {
        Ok(())
    } else {
        Err(ProtocolError::Length {
            expected,
            actual: data.len(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigRegion([u8; CONFIG_SIZE]);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LightingSettings {
    pub effect: u8,
    pub brightness: u8,
    pub speed: u8,
    pub colorful: bool,
}
impl LightingSettings {
    pub fn validate(self) -> Result<(), ProtocolError> {
        if self.effect > 18 && self.effect != 21 {
            return Err(ProtocolError::Unsupported("effect"));
        }
        if self.brightness > 9 {
            return Err(ProtocolError::Unsupported("brightness must be 0..9"));
        }
        if self.speed > 4 {
            return Err(ProtocolError::Unsupported("speed must be 0..4"));
        }
        Ok(())
    }
}
impl ConfigRegion {
    pub fn parse(data: &[u8]) -> Result<Self, ProtocolError> {
        exact_length(data, CONFIG_SIZE)?;
        if data[126..] != [0x5a, 0xa5] {
            return Err(ProtocolError::Signature);
        }
        let mut bytes = [0; CONFIG_SIZE];
        bytes.copy_from_slice(data);
        Ok(Self(bytes))
    }
    pub fn as_bytes(&self) -> &[u8; CONFIG_SIZE] {
        &self.0
    }
    pub fn effect_id(&self) -> u8 {
        self.0[10]
    }
    pub fn lighting(&self) -> Result<LightingSettings, ProtocolError> {
        let effect = self.effect_id();
        if effect > 18 && effect != 21 {
            return Err(ProtocolError::Unsupported("unknown hardware effect ID"));
        }
        if effect == 0 || effect == 21 {
            // Off/custom have no brightness/speed controls in the supplied package.
            return Ok(LightingSettings {
                effect,
                brightness: 0,
                speed: 0,
                colorful: false,
            });
        }
        let offset = 56 + effect as usize * 2;
        let settings = LightingSettings {
            effect,
            brightness: self.0[offset],
            speed: self.0[offset + 1] >> 4,
            colorful: self.0[offset + 1] & 0x0f == 7,
        };
        settings.validate()?;
        Ok(settings)
    }
    /// Evidence-derived offsets; preserves every unrelated byte. Not hardware validated locally.
    pub fn with_lighting(&self, settings: LightingSettings) -> Result<Self, ProtocolError> {
        settings.validate()?;
        let mut next = self.clone();
        next.0[9] = u8::from(settings.effect == 21);
        next.0[10] = settings.effect;
        if settings.effect == 21 {
            // StartGp defaults to 0x20; this package has only group 0.
            next.0[11] = 0x20;
        } else if settings.effect != 0 {
            let offset = 56 + settings.effect as usize * 2;
            next.0[offset] = settings.brightness;
            let speed = if settings.effect == 1 {
                next.0[offset + 1] & 0xf0
            } else {
                settings.speed << 4
            };
            let color = if effect_supports_color(settings.effect) {
                if settings.colorful {
                    7
                } else {
                    0
                }
            } else {
                next.0[offset + 1] & 0x0f
            };
            next.0[offset + 1] = speed | color;
        }
        Ok(next)
    }
    /// Change only the reported preset's brightness byte. Preserve mode flags,
    /// selected effect, color selector, speed and all other configuration data.
    pub fn with_preset_brightness(&self, value: u8) -> Result<Self, ProtocolError> {
        if value > 9 {
            return Err(ProtocolError::Unsupported("brightness must be0..9"));
        }
        let effect = self.effect_id();
        if !(1..=18).contains(&effect) {
            return Err(ProtocolError::Unsupported(
                "reported mode has no preset brightness field",
            ));
        }
        self.lighting()?;
        let mut next = self.clone();
        next.0[56 + usize::from(effect) * 2] = value;
        Ok(next)
    }
    /// Standard Fw24 sleep field: official SleepUI/SleepHW table and fill0x49bbf0+0x18.
    pub fn sleep_seconds(&self) -> Result<u16, ProtocolError> {
        let seconds = u16::from(self.0[24]) * 30;
        if SLEEP_SECONDS.contains(&seconds) {
            Ok(seconds)
        } else {
            Err(ProtocolError::Unsupported("unknown sleep setting"))
        }
    }
    pub fn with_sleep_seconds(&self, seconds: u16) -> Result<Self, ProtocolError> {
        if !SLEEP_SECONDS.contains(&seconds) {
            return Err(ProtocolError::Unsupported(
                "sleep must match the official supported choices",
            ));
        }
        let mut next = self.clone();
        next.0[24] = (seconds / 30) as u8;
        Ok(next)
    }
    /// Synthetic fixture; never represents a factory profile or a hardware read.
    pub fn demo() -> Self {
        let mut bytes = [0; CONFIG_SIZE];
        bytes[10] = 1;
        bytes[126..].copy_from_slice(&[0x5a, 0xa5]);
        for effect in 0..=21 {
            bytes[56 + effect * 2] = 5;
            bytes[57 + effect * 2] = 0x27;
        }
        Self(bytes)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorTable([u8; COLORS_SIZE]);
impl ColorTable {
    pub fn parse(data: &[u8]) -> Result<Self, ProtocolError> {
        exact_length(data, COLORS_SIZE)?;
        let mut b = [0; COLORS_SIZE];
        b.copy_from_slice(data);
        Ok(Self(b))
    }
    /// Synthetic, signed empty palette; never a factory profile.
    pub fn black() -> Self {
        let mut bytes = [0; COLORS_SIZE];
        bytes[506..508].copy_from_slice(&[0x5a, 0xa5]);
        Self(bytes)
    }
    pub fn as_bytes(&self) -> &[u8; COLORS_SIZE] {
        &self.0
    }
    /// Fw24/CRC=1 package uses a signature, not an additive checksum.
    pub fn validate_for_write(&self) -> Result<(), ProtocolError> {
        if self.0[506..508] != [0x5a, 0xa5] {
            return Err(ProtocolError::PaletteSignature);
        }
        Ok(())
    }
    /// First RGB triple of a seven-color effect record, payload offset 21*HWID.
    /// OemDrv 0x49c7ce..0x49c7fa; RGBIndex=0,1,2 in the supplied package.
    pub fn effect_color(&self, effect: u8) -> Result<[u8; 3], ProtocolError> {
        let offset = effect_color_offset(effect)?;
        Ok([self.0[offset], self.0[offset + 1], self.0[offset + 2]])
    }
    pub fn with_effect_color(&self, effect: u8, color: [u8; 3]) -> Result<Self, ProtocolError> {
        self.validate_for_write()?;
        if !effect_supports_color(effect) {
            return Err(ProtocolError::Unsupported("effect has no adjustable color"));
        }
        let offset = effect_color_offset(effect)?;
        let mut next = self.clone();
        next.0[offset..offset + 3].copy_from_slice(&color);
        Ok(next)
    }
}
/// Color/random controls advertised by the supplied F75 package's LedOpt entries.
pub const fn effect_supports_color(effect: u8) -> bool {
    matches!(effect, 1 | 2 | 4..=14 | 18)
}
fn effect_color_offset(effect: u8) -> Result<usize, ProtocolError> {
    if !(1..=18).contains(&effect) {
        return Err(ProtocolError::Unsupported("effect has no palette record"));
    }
    Ok(usize::from(effect) * 21)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    Base,
    Fn,
}
impl Layer {
    fn byte(self) -> u8 {
        match self {
            Self::Base => 0,
            Self::Fn => 1,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyAction {
    pub page: u8,
    pub usage: u8,
}
/// Consumer records explicitly emitted by the supplied OemDrv key converter.
/// These are application record type2, not USB usage page number2.
pub const SUPPORTED_MEDIA_USAGES: [u8; 7] = [0xcd, 0xb7, 0xb6, 0xb5, 0xe9, 0xea, 0xe2];
pub fn media_action_name(usage: u8) -> Option<&'static str> {
    match usage {
        0xcd => Some("Play / pause"),
        0xb7 => Some("Stop playback"),
        0xb6 => Some("Previous track"),
        0xb5 => Some("Next track"),
        0xe9 => Some("Volume up"),
        0xea => Some("Volume down"),
        0xe2 => Some("Mute"),
        _ => None,
    }
}
#[cfg(test)]
mod media_record_tests {
    use super::*;
    #[test]
    fn official_converter_constants_match_media_records() {
        // OemDrv49a920 return DWORDs, in little-endian byte order.
        for (usage, native) in [
            (0xcd, 0xcd000002u32),
            (0xb7, 0xb7000002),
            (0xb6, 0xb6000002),
            (0xb5, 0xb5000002),
            (0xe9, 0xe9000002),
            (0xea, 0xea000002),
            (0xe2, 0xe2000002),
        ] {
            assert_eq!(
                KeyAction { page: 2, usage }.record().unwrap(),
                native.to_le_bytes()
            );
            assert!(SUPPORTED_MEDIA_USAGES.contains(&usage));
            assert!(media_action_name(usage).is_some());
        }
        assert!(media_action_name(0xb0).is_none());
    }
}
impl KeyAction {
    pub fn record(self) -> Result<[u8; 4], ProtocolError> {
        self.validate()?;
        Ok(if self.page == 0 && (0xe0..=0xe7).contains(&self.usage) {
            [0, 1u8 << (self.usage - 0xe0), 0, 0]
        } else {
            [self.page, 0, 0, self.usage]
        })
    }
    pub fn validate(self) -> Result<(), ProtocolError> {
        if self.page == 0 || self.page == 2 {
            Ok(())
        } else {
            Err(ProtocolError::Unsupported("key action page"))
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keymap([u8; REPORT_SIZE]);
impl Keymap {
    /// Official GetMatrix 0x499640: 128 slots * 4 bytes, one bounded chunk.
    pub fn read_request(layer: Layer) -> FeatureReport {
        let mut bytes = [0; REPORT_SIZE];
        bytes[..8].copy_from_slice(&[6, 0x83, layer.byte(), 0, 1, 0, 0, 2]);
        FeatureReport(bytes)
    }
    /// Validate layer/length/signature, then normalize command to write-form 03.
    /// Firmware may echo either 83 or 03; all other header fields must match.
    pub fn decode_response(layer: Layer, bytes: &[u8]) -> Result<Self, ProtocolError> {
        exact_length(bytes, REPORT_SIZE)?;
        let request = Self::read_request(layer);
        if bytes[0] != REPORT_ID
            || ![0x83, 0x03].contains(&bytes[1])
            || bytes[2..8] != request.0[2..8]
        {
            return Err(ProtocolError::Header);
        }
        let mut normalized = [0; REPORT_SIZE];
        normalized.copy_from_slice(bytes);
        normalized[1] = 0x03;
        Self::parse(&normalized)
    }
    pub fn parse(bytes: &[u8]) -> Result<Self, ProtocolError> {
        exact_length(bytes, REPORT_SIZE)?;
        if bytes[0] != 6
            || bytes[1] != 3
            || bytes[2] > 1
            || bytes[3..6] != [0, 1, 0]
            || bytes[6..8] != [0, 2]
        {
            return Err(ProtocolError::Header);
        }
        if bytes[518..] != [0x5a, 0xa5] {
            return Err(ProtocolError::Signature);
        }
        let mut b = [0; REPORT_SIZE];
        b.copy_from_slice(bytes);
        Ok(Self(b))
    }
    /// Local empty edit buffer. NEVER use this as a physical factory template.
    pub fn empty(layer: Layer) -> Self {
        let mut b = [0; REPORT_SIZE];
        b[..8].copy_from_slice(&[6, 3, layer.byte(), 0, 1, 0, 0, 2]);
        b[518..].copy_from_slice(&[0x5a, 0xa5]);
        Self(b)
    }
    pub fn layer(&self) -> Layer {
        if self.0[2] == 0 {
            Layer::Base
        } else {
            Layer::Fn
        }
    }
    pub fn as_bytes(&self) -> &[u8; REPORT_SIZE] {
        &self.0
    }
    pub fn get(&self, index: usize) -> Result<KeyAction, ProtocolError> {
        if index > 126 {
            return Err(ProtocolError::Index);
        };
        let o = 8 + 4 * index;
        let modifier = self.0[o + 1];
        let usage = if self.0[o] == 0
            && modifier.is_power_of_two()
            && self.0[o + 2] == 0
            && self.0[o + 3] == 0
        {
            0xe0 + modifier.trailing_zeros() as u8
        } else {
            self.0[o + 3]
        };
        Ok(KeyAction {
            page: self.0[o],
            usage,
        })
    }
    /// Restore only known original ANSI source slots from a previously read bank.
    /// Firmware controls and unknown matrix slots are never copied from a backup.
    pub fn restore_known_sources(&self, saved: &Self) -> Result<Self, ProtocolError> {
        if self.layer() != saved.layer() {
            return Err(ProtocolError::Unsupported("backup layer mismatch"));
        }
        let mut next = self.clone();
        for index in [
            12,
            18,
            24,
            30,
            36,
            42,
            48,
            54,
            60,
            66,
            72,
            78,
            FN_MATRIX_INDEX,
        ] {
            let o = 8 + 4 * index;
            if self.0[o..o + 4] != saved.0[o..o + 4] {
                return Err(ProtocolError::Unsupported("backup protected controls differ; revision or external changes require investigation"));
            }
        }
        for usage in 1..=255u16 {
            if (58..=69).contains(&usage) {
                continue;
            }
            if let Ok(index) = matrix_index_for_usage(usage) {
                let o = 8 + 4 * index;
                if self.layer() == Layer::Base && saved.0[o..o + 4] == [0, 0, 0, 0] {
                    return Err(ProtocolError::Unsupported(
                        "base backup would disable a physical key",
                    ));
                }
                next.0[o..o + 4].copy_from_slice(&saved.0[o..o + 4]);
            }
        }
        Ok(next)
    }
    pub fn matches_action(&self, index: usize, action: KeyAction) -> Result<bool, ProtocolError> {
        if index > 126 {
            return Err(ProtocolError::Index);
        }
        let o = 8 + 4 * index;
        Ok(self.0[o..o + 4] == action.record()?)
    }
    pub fn set(&mut self, index: usize, action: KeyAction) -> Result<(), ProtocolError> {
        if index > 126 {
            return Err(ProtocolError::Index);
        }
        action.validate()?;
        // F1-F12 and Fn are device control sources; keep them intact.
        if [
            12,
            18,
            24,
            30,
            36,
            42,
            48,
            54,
            60,
            66,
            72,
            78,
            FN_MATRIX_INDEX,
        ]
        .contains(&index)
        {
            return Err(ProtocolError::Unsupported("firmware-reserved source key"));
        }
        let o = 8 + 4 * index;
        let record = action.record()?;
        self.0[o..o + 4].copy_from_slice(&record);
        Ok(())
    }
    /// The supplied French package has a different bottom row. Confirm the
    /// ANSI Fn marker from a physical base bank before exposing matrix writes.
    pub fn validate_ansi_fn_position(&self) -> Result<(), ProtocolError> {
        let offset = 8 + 4 * FN_MATRIX_INDEX;
        if self.layer() != Layer::Base
            || self.0[offset..offset + 4] != [13, 0, 0, 0]
            || self.0[8 + 4 * 59..12 + 4 * 59] == [13, 0, 0, 0]
        {
            return Err(ProtocolError::Unsupported("ANSI Fn position does not match the locally observed layout; mapping and per-key writes are unavailable for this revision"));
        }
        Ok(())
    }
}

/// Official SetGame (06) planar RGB payload; matrix/LED mapping is revision-specific.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PerKeyColors([u8; 378]);
impl PerKeyColors {
    /// Official GetGame 0x499d50, group 0, three 126-byte planes.
    pub fn read_request() -> FeatureReport {
        let mut bytes = [0; REPORT_SIZE];
        bytes[..8].copy_from_slice(&[6, 0x86, 0, 0, 1, 0, 0x7a, 1]);
        FeatureReport(bytes)
    }
    pub fn decode_response(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() != REPORT_SIZE && bytes.len() != HEADER_SIZE + 378 {
            return Err(ProtocolError::Length {
                expected: HEADER_SIZE + 378,
                actual: bytes.len(),
            });
        }
        let request = Self::read_request();
        if bytes[0] != REPORT_ID
            || ![0x86, 0x06].contains(&bytes[1])
            || bytes[2..8] != request.0[2..8]
        {
            return Err(ProtocolError::Header);
        }
        Self::parse(&bytes[8..386])
    }
    pub fn as_bytes(&self) -> &[u8; 378] {
        &self.0
    }
    pub fn black() -> Self {
        Self([0; 378])
    }
    pub fn parse(bytes: &[u8]) -> Result<Self, ProtocolError> {
        exact_length(bytes, 378)?;
        let mut b = [0; 378];
        b.copy_from_slice(bytes);
        Ok(Self(b))
    }
    pub fn set(&mut self, index: usize, rgb: [u8; 3]) -> Result<(), ProtocolError> {
        if index >= 126 {
            return Err(ProtocolError::Index);
        }
        for (plane, value) in rgb.iter().enumerate() {
            self.0[126 * plane + index] = *value
        }
        Ok(())
    }
    pub fn get(&self, index: usize) -> Result<[u8; 3], ProtocolError> {
        if index >= 126 {
            return Err(ProtocolError::Index);
        }
        Ok([self.0[index], self.0[126 + index], self.0[252 + index]])
    }
    pub fn restore_known_colors(&self, saved: &Self) -> Result<Self, ProtocolError> {
        let mut next = self.clone();
        for index in (1..=255u16)
            .filter_map(|u| matrix_index_for_usage(u).ok())
            .chain(std::iter::once(FN_MATRIX_INDEX))
        {
            next.set(index, saved.get(index)?)?;
        }
        Ok(next)
    }
    pub fn offline_report(&self) -> [u8; 520] {
        let mut b = [0; 520];
        b[..8].copy_from_slice(&[6, 6, 0, 0, 1, 0, 0x7a, 1]);
        b[8..386].copy_from_slice(&self.0);
        b
    }
}

/// Fn has no ordinary USB keyboard usage; it is a firmware-owned source.
pub const FN_MATRIX_INDEX: usize = 53;

/// Original ANSI F75 source position. USB usages identify physical keys here,
/// independently of any remap currently stored on the keyboard.
/// Most positions agree with FR KB.ini. ANSI has Fn53/RightCtrl59, while
/// the supplied French package has RightAlt53/Fn59. Local base reads confirm it.
/// ANSI backslash is slot80; the FR package instead has slot75 plus ISO slot76.
/// Knob is excluded: it has no keyboard-page usage.
pub fn matrix_index_for_usage(usage: u16) -> Result<usize, ProtocolError> {
    let index = match usage {
        4..=29 => [
            9, 34, 22, 21, 20, 27, 33, 39, 50, 45, 51, 57, 46, 40, 56, 62, 8, 26, 15, 32, 44, 28,
            14, 16, 38, 10,
        ][usize::from(usage - 4)],
        30..=38 => 7 + 6 * usize::from(usage - 30),
        39 => 61,
        40 => 81, // Enter, same electrical slot in ANSI and ISO geometry
        41 => 0,
        42 => 79,
        43 => 2,
        44 => 35,
        45 => 67,
        46 => 73,
        47 => 68,
        48 => 74,
        49 => 80, // ANSI backslash
        51 => 63,
        52 => 69,
        53 => 1,
        54 => 52,
        55 => 58,
        56 => 64,
        57 => 3,
        58..=69 => 12 + 6 * usize::from(usage - 58),
        75 => 86,
        76 => 85,
        77 => 88,
        78 => 87,
        79 => 89,
        80 => 77,
        81 => 83,
        82 => 82,
        224 => 5,
        225 => 4,
        226 => 17,
        227 => 11,
        228 => 59,
        229 => 70,
        _ => {
            return Err(ProtocolError::Unsupported(
                "physical key not present in original ANSI F75 layout",
            ))
        }
    };
    Ok(index)
}

/// Known 20-byte wireless framing, OFFLINE ONLY. F75 dongle behavior is not verified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WirelessFrame([u8; 20]);
impl WirelessFrame {
    pub fn config_read() -> Self {
        let mut b = [0; 20];
        b[..4].copy_from_slice(&[0x13, 0x44, 0x0a, 0]);
        b[19] = checksum(&b[..19]);
        Self(b)
    }
    pub fn parse(bytes: &[u8]) -> Result<Self, ProtocolError> {
        exact_length(bytes, 20)?;
        if bytes[0] != 0x13 || ![0x44, 0x04, 0x09, 0x02, 0x0a].contains(&bytes[1]) {
            return Err(ProtocolError::Header);
        }
        if checksum(&bytes[..19]) != bytes[19] {
            return Err(ProtocolError::Checksum);
        }
        let mut b = [0; 20];
        b.copy_from_slice(bytes);
        Ok(Self(b))
    }
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}
pub fn checksum(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0u8, |sum, b| sum.wrapping_add(*b))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_fixture() {
        let p = FeatureReport::read(Region::Config);
        assert_eq!(&p.as_bytes()[..8], &[6, 0x84, 0, 0, 1, 0, 0x80, 0]);
        assert!(p.as_bytes()[8..].iter().all(|b| *b == 0));
    }
    #[test]
    fn color_header() {
        assert_eq!(
            &FeatureReport::read(Region::Colors).as_bytes()[..8],
            &[6, 0x8a, 0, 0, 1, 0, 0, 2]
        );
    }
    #[test]
    fn lengths_rejected() {
        for n in [0, 1, 7, 127, 129, 519, 521] {
            assert!(ConfigRegion::parse(&vec![0; n]).is_err());
            assert!(FeatureReport::decode_response(Region::Config, &vec![0; n]).is_err());
        }
    }
    #[test]
    fn response_strict() {
        let mut p = FeatureReport::write(Region::Config, ConfigRegion::demo().as_bytes())
            .unwrap()
            .into_bytes();
        assert!(FeatureReport::decode_response(Region::Config, &p).is_ok());
        p[2] = 1;
        assert_eq!(
            FeatureReport::decode_response(Region::Config, &p),
            Err(ProtocolError::Header)
        );
    }
    #[test]
    fn wrong_report_command_length_rejected() {
        for (offset, value) in [(0, 9), (1, 0x83), (6, 0x81), (7, 1)] {
            let mut p = FeatureReport::write(Region::Config, ConfigRegion::demo().as_bytes())
                .unwrap()
                .into_bytes();
            p[offset] = value;
            assert!(FeatureReport::decode_response(Region::Config, &p).is_err());
        }
    }
    #[test]
    fn preserve_unknown_fields() {
        let old = ConfigRegion::demo();
        let next = old
            .with_lighting(LightingSettings {
                effect: 7,
                brightness: 9,
                speed: 4,
                colorful: false,
            })
            .unwrap();
        for i in 0..128 {
            if ![9, 10, 70, 71].contains(&i) {
                assert_eq!(old.0[i], next.0[i]);
            }
        }
        assert_eq!(next.lighting().unwrap().speed, 4);
    }
    #[test]
    fn palette_roundtrip() {
        let mut b = [0; 512];
        b[7] = 0xaa;
        let t = ColorTable::parse(&b).unwrap();
        assert_eq!(t.as_bytes(), &b);
        assert!(ColorTable::parse(&b[..511]).is_err());
    }
    #[test]
    fn keymap_public_capture_fixture() {
        for b in [
            include_bytes!("../../../fixtures/keymap/f87-factory-base.bin"),
            include_bytes!("../../../fixtures/keymap/f87-factory-fn.bin"),
        ] {
            let map = Keymap::parse(b).unwrap();
            assert_eq!(map.as_bytes(), b);
        }
    }
    #[test]
    fn keymap_patch_only_one_slot() {
        let b = include_bytes!("../../../fixtures/keymap/f87-factory-base.bin");
        let mut map = Keymap::parse(b).unwrap();
        map.set(
            58,
            KeyAction {
                page: 0,
                usage: 0x68,
            },
        )
        .unwrap();
        assert_eq!(map.get(58).unwrap().usage, 0x68);
        for (i, value) in b.iter().enumerate() {
            if !(240..244).contains(&i) {
                assert_eq!(map.as_bytes()[i], *value);
            }
        }
        assert!(map.set(127, KeyAction { page: 0, usage: 0 }).is_err());
        assert!(map.set(12, KeyAction { page: 0, usage: 4 }).is_err());
    }
    #[test]
    fn layer_and_action_validation() {
        let mut k = Keymap::empty(Layer::Fn);
        assert_eq!(k.layer(), Layer::Fn);
        assert!(k.set(1, KeyAction { page: 1, usage: 4 }).is_err());
        let mut bytes = *k.as_bytes();
        bytes[2] = 2;
        assert!(Keymap::parse(&bytes).is_err());
    }
    #[test]
    fn wireless_checksum_fixture() {
        let f = WirelessFrame::config_read();
        assert_eq!(f.as_bytes()[19], 0x61);
        assert_eq!(WirelessFrame::parse(f.as_bytes()).unwrap(), f);
        let mut b = *f.as_bytes();
        b[5] = 1;
        assert_eq!(WirelessFrame::parse(&b), Err(ProtocolError::Checksum));
        assert_eq!(checksum(&[255, 2]), 1);
    }
    #[test]
    fn settings_bounds() {
        for (effect, brightness, speed) in [(19, 4, 2), (255, 4, 2), (1, 10, 2), (1, 9, 5)] {
            assert!(LightingSettings {
                effect,
                brightness,
                speed,
                colorful: false
            }
            .validate()
            .is_err());
        }
    }
}

#[cfg(test)]
mod planar_tests {
    use super::*;
    #[test]
    fn official_planar_layout() {
        let mut p = PerKeyColors::black();
        p.set(125, [7, 8, 9]).unwrap();
        let b = p.offline_report();
        assert_eq!(&b[..8], &[6, 6, 0, 0, 1, 0, 0x7a, 1]);
        assert_eq!(b[133], 7);
        assert_eq!(b[259], 8);
        assert_eq!(b[385], 9);
        assert!(p.set(126, [0; 3]).is_err());
        assert_eq!(p.get(125).unwrap(), [7, 8, 9]);
    }
}

#[cfg(test)]
mod modifier_tests {
    use super::*;
    #[test]
    fn modifier_encoding_matches_native_and_local_read() {
        let mut k = Keymap::empty(Layer::Base);
        for u in 0xe0..=0xe7 {
            k.set(9, KeyAction { page: 0, usage: u }).unwrap();
            assert_eq!(&k.as_bytes()[44..48], &[0, 1 << (u - 0xe0), 0, 0]);
            assert_eq!(k.get(9).unwrap().usage, u);
        }
        k.set(9, KeyAction { page: 0, usage: 4 }).unwrap();
        assert_eq!(&k.as_bytes()[44..48], &[0, 0, 0, 4]);
    }
}

#[cfg(test)]
mod sleep_tests {
    use super::*;
    #[test]
    fn official_sleep_mapping_preserves_unknown_fields() {
        let old = ConfigRegion::demo();
        for seconds in SLEEP_SECONDS {
            let next = old.with_sleep_seconds(*seconds).unwrap();
            assert_eq!(next.sleep_seconds().unwrap(), *seconds);
            for (i, b) in old.as_bytes().iter().enumerate() {
                if i != 24 {
                    assert_eq!(*b, next.as_bytes()[i])
                }
            }
        }
        assert!(old.with_sleep_seconds(30).is_err());
        assert!(old.with_sleep_seconds(65535).is_err());
    }
}

#[cfg(test)]
mod native_protocol_tests {
    use super::*;
    #[test]
    fn palette_patch_is_one_rgb_triple_and_keeps_signature() {
        let old = ColorTable::black();
        let next = old.with_effect_color(7, [7, 8, 9]).unwrap();
        assert_eq!(next.effect_color(7).unwrap(), [7, 8, 9]);
        for (i, v) in old.as_bytes().iter().enumerate() {
            if !(147..150).contains(&i) {
                assert_eq!(*v, next.as_bytes()[i]);
            }
        }
        for id in [0, 3, 15, 16, 17, 21, 255] {
            assert!(old.with_effect_color(id, [1; 3]).is_err());
        }
        assert!(ColorTable::parse(&[0; 512])
            .unwrap()
            .with_effect_color(1, [1; 3])
            .is_err());
    }
    #[test]
    fn layer_read_headers_and_malformed_responses() {
        for layer in [Layer::Base, Layer::Fn] {
            let request = Keymap::read_request(layer);
            assert_eq!(request.as_bytes()[1], 0x83);
            assert_eq!(&request.as_bytes()[4..8], &[1, 0, 0, 2]);
            let mut reply = *Keymap::empty(layer).as_bytes();
            reply[1] = 0x83;
            assert_eq!(
                Keymap::decode_response(layer, &reply).unwrap().layer(),
                layer
            );
            reply[2] ^= 1;
            assert!(Keymap::decode_response(layer, &reply).is_err());
            reply[2] ^= 1;
            reply[518] = 0;
            assert!(Keymap::decode_response(layer, &reply).is_err());
            assert!(Keymap::decode_response(layer, &reply[..519]).is_err());
        }
    }
    #[test]
    fn planar_read_uses_exact_bank_and_rejects_wrong_command() {
        let mut colors = PerKeyColors::black();
        colors.set(9, [1, 2, 3]).unwrap();
        let mut response = colors.offline_report();
        response[1] = 0x86;
        assert_eq!(PerKeyColors::decode_response(&response).unwrap(), colors);
        assert_eq!(
            PerKeyColors::decode_response(&response[..386]).unwrap(),
            colors
        );
        response[2] = 1;
        assert!(PerKeyColors::decode_response(&response).is_err());
        response[2] = 0;
        response[1] = 0x8a;
        assert!(PerKeyColors::decode_response(&response).is_err());
        assert_eq!(
            &PerKeyColors::read_request().as_bytes()[..8],
            &[6, 0x86, 0, 0, 1, 0, 0x7a, 1]
        );
    }
    #[test]
    fn ansi_matrix_is_unique_and_reserved_positions_are_not_editable() {
        let indices = (1..=255u16)
            .filter_map(|u| matrix_index_for_usage(u).ok())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(indices.len(), 79);
        assert!(!indices.contains(&FN_MATRIX_INDEX));
        assert_eq!(matrix_index_for_usage(49).unwrap(), 80);
        assert_eq!(matrix_index_for_usage(40).unwrap(), 81);
        assert_eq!(matrix_index_for_usage(228).unwrap(), 59);
        assert!(matrix_index_for_usage(230).is_err());
        let mut map = Keymap::empty(Layer::Fn);
        for usage in 58..=69 {
            assert!(map
                .set(
                    matrix_index_for_usage(usage).unwrap(),
                    KeyAction { page: 0, usage: 4 }
                )
                .is_err());
        }
        assert!(map
            .set(FN_MATRIX_INDEX, KeyAction { page: 0, usage: 4 })
            .is_err());
    }
    #[test]
    fn observed_ansi_control_records_gate_the_regional_layout() {
        let mut bytes = *Keymap::empty(Layer::Base).as_bytes();
        bytes[8 + 4 * 53..12 + 4 * 53].copy_from_slice(&[13, 0, 0, 0]);
        bytes[8 + 4 * 59..12 + 4 * 59].copy_from_slice(&[0, 16, 0, 0]);
        let map = Keymap::parse(&bytes).unwrap();
        map.validate_ansi_fn_position().unwrap();
        assert_eq!(
            map.get(59).unwrap(),
            KeyAction {
                page: 0,
                usage: 0xe4
            }
        );
        bytes[8 + 4 * 53..12 + 4 * 53].copy_from_slice(&[0, 0, 0, 0]);
        bytes[8 + 4 * 59..12 + 4 * 59].copy_from_slice(&[13, 0, 0, 0]);
        assert!(Keymap::parse(&bytes)
            .unwrap()
            .validate_ansi_fn_position()
            .is_err());
    }
    #[test]
    fn unsupported_parameters_are_preserved_and_custom_is_explicit() {
        let old = ConfigRegion::demo();
        let fixed = old
            .with_lighting(LightingSettings {
                effect: 1,
                brightness: 8,
                speed: 4,
                colorful: false,
            })
            .unwrap();
        assert_eq!(fixed.as_bytes()[59] >> 4, old.as_bytes()[59] >> 4);
        let rainbow = old
            .with_lighting(LightingSettings {
                effect: 3,
                brightness: 4,
                speed: 1,
                colorful: false,
            })
            .unwrap();
        assert_eq!(rainbow.as_bytes()[63] & 15, old.as_bytes()[63] & 15);
        let custom = old
            .with_lighting(LightingSettings {
                effect: 21,
                brightness: 9,
                speed: 4,
                colorful: true,
            })
            .unwrap();
        assert_eq!(&custom.as_bytes()[9..12], &[1, 21, 32]);
        let off = custom
            .with_lighting(LightingSettings {
                effect: 0,
                brightness: 9,
                speed: 4,
                colorful: true,
            })
            .unwrap();
        assert_eq!(&off.as_bytes()[9..11], &[0, 0]);
        assert_eq!(&off.as_bytes()[56..126], &old.as_bytes()[56..126]);
    }
}

#[cfg(test)]
mod palette_marker_regression {
    use super::*;
    #[test]
    fn observed_physical_tail_and_native_stack_offsets() {
        let mut bytes = [0; 512];
        bytes[500..].copy_from_slice(&[0, 0, 0, 0, 0, 0, 0x5a, 0xa5, 0, 0, 0, 0]);
        let p = ColorTable::parse(&bytes).unwrap();
        p.validate_for_write().unwrap();
        p.with_effect_color(1, [17, 34, 51]).unwrap();
        let mut wrong = [0; 512];
        wrong[510..].copy_from_slice(&[0x5a, 0xa5]);
        assert!(ColorTable::parse(&wrong)
            .unwrap()
            .validate_for_write()
            .is_err());
    }
}

#[cfg(test)]
mod brightness_only_tests {
    use super::*;
    #[test]
    fn brightness_patch_preserves_retained_flags_and_nonstandard_color_selector() {
        let mut bytes = *ConfigRegion::demo().as_bytes();
        bytes[9] = 1;
        bytes[10] = 7;
        bytes[71] = 0x33;
        let old = ConfigRegion::parse(&bytes).unwrap();
        let new = old.with_preset_brightness(4).unwrap();
        for (i, b) in old.as_bytes().iter().enumerate() {
            assert_eq!(new.as_bytes()[i], if i == 70 { 4 } else { *b });
        }
        assert!(old.with_preset_brightness(10).is_err());
        for effect in [0, 21, 255] {
            let mut b = bytes;
            b[10] = effect;
            assert!(ConfigRegion::parse(&b)
                .unwrap()
                .with_preset_brightness(4)
                .is_err());
        }
    }
}
