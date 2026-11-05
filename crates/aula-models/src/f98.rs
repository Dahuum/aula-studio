//! Offline-only frames recovered from the F98 Pro driver (SHA256 16fb7173…ab19d0).
//!
//! No HID transport, begin/end transaction, read request or reset is exposed.
//! Native 0042d290 and its instruction listing establish the scalar offsets;
//! 00456e40 prepends report ID zero to 64 bytes. See docs/F98_RESEARCH.md.

use crate::Result;

pub const REPORT_LEN: usize = 65;
pub const PER_KEY_PAYLOAD_LEN: usize = 576;
pub const PER_KEY_PAGES: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightingParameters {
    /// Native preset IDs: 0 is Off; 1..=19 are the RGB branch's named presets.
    /// The separate custom-mode value 20 is deliberately excluded here.
    pub effect: u8,
    pub color: [u8; 3],
    pub colorful: bool,
    pub brightness: u8,
    pub speed: u8,
    /// Preserved native byte; effect-specific direction meanings are incomplete.
    /// This is not a claim that every byte value is a supported direction.
    pub direction_raw: u8,
}
impl LightingParameters {
    pub fn encode(self) -> Result<[u8; REPORT_LEN]> {
        if self.effect > 19 || self.brightness > 5 || self.speed > 5 {
            return Err("F98 scalar lighting outside the native RGB preset/slider range".into());
        }
        let mut report = [0; REPORT_LEN];
        report[1] = self.effect;
        report[2..5].copy_from_slice(&self.color);
        report[9] = u8::from(self.colorful);
        report[10] = self.brightness;
        report[11] = self.speed;
        report[12] = self.direction_raw;
        report[15..17].copy_from_slice(&[0xaa, 0x55]);
        Ok(report)
    }
    /// Decode only the canonical scalar form made by native 0042d290.
    /// Unknown reserved bytes are rejected rather than silently discarded.
    pub fn decode(report: &[u8]) -> Result<Self> {
        if report.len() != REPORT_LEN || report[0] != 0 {
            return Err("F98 requires a 65-byte unnumbered feature report".into());
        }
        if report[9] > 1 || report[15..17] != [0xaa, 0x55] {
            return Err("F98 scalar color selector or literal integrity marker is invalid".into());
        }
        let value = Self {
            effect: report[1],
            color: [report[2], report[3], report[4]],
            colorful: report[9] == 1,
            brightness: report[10],
            speed: report[11],
            direction_raw: report[12],
        };
        if value.encode()?.as_slice() != report {
            return Err("Noncanonical F98 scalar report: reserved bytes would be lost".into());
        }
        Ok(value)
    }
}

/// Encode only the resource-known F98 LED positions. Native 0042d440 uses nine
/// 64-byte pages, four bytes per occupied position and AA55 at payload574/575.
/// Unlike scalar color, per-key data is index,B,G,R (0042d960 reconstructs ARGB).
/// Unrecovered positions remain zero; this is not a complete hardware snapshot.
pub fn per_key_payload(colors: &[(u16, [u8; 3])]) -> Result<[u8; PER_KEY_PAYLOAD_LEN]> {
    let model = crate::model("f98-pro")?;
    let mut seen = std::collections::BTreeSet::new();
    let mut payload = [0; PER_KEY_PAYLOAD_LEN];
    for &(index, [red, green, blue]) in colors {
        if !model.keys.iter().any(|key| key.light_index == Some(index)) {
            return Err("F98 LED index is not established by this package's layout".into());
        }
        if !seen.insert(index) {
            return Err("Duplicate F98 LED index".into());
        }
        let offset = usize::from(index) * 4;
        if index > u8::MAX.into() || offset + 4 > PER_KEY_PAYLOAD_LEN - 2 {
            return Err("F98 LED index overlaps the native transfer boundary".into());
        }
        payload[offset..offset + 4].copy_from_slice(&[index as u8, blue, green, red]);
    }
    payload[574..576].copy_from_slice(&[0xaa, 0x55]);
    Ok(payload)
}

/// Only data pages, without the state-changing native preamble/commit sequence.
pub fn per_key_frames(colors: &[(u16, [u8; 3])]) -> Result<[[u8; REPORT_LEN]; PER_KEY_PAGES]> {
    let payload = per_key_payload(colors)?;
    let mut frames = [[0; REPORT_LEN]; PER_KEY_PAGES];
    for (page, data) in frames.iter_mut().zip(payload.chunks_exact(64)) {
        page[1..].copy_from_slice(data);
    }
    Ok(frames)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> LightingParameters {
        LightingParameters {
            effect: 7,
            color: [0x11, 0x22, 0x33],
            colorful: true,
            brightness: 5,
            speed: 3,
            direction_raw: 2,
        }
    }

    #[test]
    fn scalar_matches_independent_native_offset_fixture() {
        let report = example().encode().unwrap();
        assert_eq!(
            &report[..17],
            &[0, 7, 0x11, 0x22, 0x33, 0, 0, 0, 0, 1, 5, 3, 2, 0, 0, 0xaa, 0x55]
        );
        assert!(report[17..].iter().all(|byte| *byte == 0));
        assert_eq!(LightingParameters::decode(&report).unwrap(), example());
    }

    #[test]
    fn rejects_foreign_reports_markers_reserved_bytes_and_ranges() {
        let report = example().encode().unwrap();
        for offset in [0, 5, 8, 13, 14, 15, 16, 64] {
            let mut bad = report;
            bad[offset] ^= 0x80;
            assert!(LightingParameters::decode(&bad).is_err(), "offset{offset}");
        }
        assert!(LightingParameters::decode(&report[..64]).is_err());
        for bad in [
            LightingParameters {
                effect: 20,
                ..example()
            },
            LightingParameters {
                brightness: 6,
                ..example()
            },
            LightingParameters {
                speed: 6,
                ..example()
            },
        ] {
            assert!(bad.encode().is_err());
        }
        let mut bad = report;
        bad[9] = 2;
        assert!(LightingParameters::decode(&bad).is_err());
    }

    #[test]
    fn per_key_is_bgr_and_keeps_all_nine_pages_and_literal_tail() {
        let payload =
            per_key_payload(&[(56, [0x11, 0x22, 0x33]), (119, [0xaa, 0xbb, 0xcc])]).unwrap();
        assert_eq!(&payload[224..228], &[56, 0x33, 0x22, 0x11]);
        assert_eq!(&payload[476..480], &[119, 0xcc, 0xbb, 0xaa]);
        assert_eq!(&payload[574..], &[0xaa, 0x55]);
        let pages = per_key_frames(&[(56, [0x11, 0x22, 0x33]), (119, [0xaa, 0xbb, 0xcc])]).unwrap();
        assert!(pages.iter().all(|page| page[0] == 0));
        assert_eq!(&pages[8][63..], &[0xaa, 0x55]);
        assert_eq!(
            pages
                .iter()
                .flat_map(|page| page[1..].iter().copied())
                .collect::<Vec<_>>(),
            payload
        );
    }

    #[test]
    fn per_key_rejects_unrecovered_duplicate_and_out_of_bounds_positions() {
        for index in [0, 127, 143, 144, u16::MAX] {
            assert!(per_key_payload(&[(index, [1, 2, 3])]).is_err());
        }
        assert!(per_key_payload(&[(56, [1, 2, 3]), (56, [3, 2, 1])]).is_err());
    }
}
