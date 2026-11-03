//! Offline encoders for frames observed in the S75 Pro native Windows driver.
//!
//! No transport, device selection, upload, persistence or hardware validation is
//! provided here. F108 Pro shares USB IDs/configuration conventions, but its
//! binary has a different hash and these encoders do not establish compatibility.
//!
//! Target SHA256: e92e83b8466fa35bf370f8bc1092f2a851ec303c41893e62d96073ba4d620e3e.
//! REA/Ghidra 12.1.4 evidence is retained privately in analysis/s75-native/.
//! Function addresses below refer to that exact executable only.

use std::fmt;

pub const REPORT_ID: u8 = 0;
pub const PAYLOAD_LEN: usize = 64;
pub const REPORT_LEN: usize = 65;
pub const PER_KEY_WRITE_PAGES: usize = 8;
pub const PER_KEY_READ_PAGES: usize = 9;
pub const KEYMAP_PAGES: usize = 9;
pub const PER_KEY_RECORD_CAPACITY: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameError {
    Length { expected: usize, actual: usize },
    ReportId(u8),
    Parameter(&'static str),
    TooManyRecords(usize),
    DuplicateIndex(u8),
}
impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for FrameError {}

/// A full HID feature report, including the otherwise unnumbered report ID zero.
/// FUN_00451350 sends exactly 0x41 bytes; FUN_0044edb0 prepends zero to 64 data bytes.
/// REA ev_84462bb300bc3afaf274643522ecc088f064a56ddd5239e567c3378b21bdae4f,
/// ev_ae63f4d79ab57c3f507b771ea3156b1dd7633385ab71a39f2a566ff413b36459.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeatureFrame([u8; REPORT_LEN]);
impl FeatureFrame {
    pub fn from_payload(payload: &[u8]) -> Result<Self, FrameError> {
        if payload.len() != PAYLOAD_LEN {
            return Err(FrameError::Length {
                expected: PAYLOAD_LEN,
                actual: payload.len(),
            });
        }
        let mut report = [0; REPORT_LEN];
        report[1..].copy_from_slice(payload);
        Ok(Self(report))
    }
    pub fn decode(report: &[u8]) -> Result<Self, FrameError> {
        if report.len() != REPORT_LEN {
            return Err(FrameError::Length {
                expected: REPORT_LEN,
                actual: report.len(),
            });
        }
        if report[0] != REPORT_ID {
            return Err(FrameError::ReportId(report[0]));
        }
        let mut bytes = [0; REPORT_LEN];
        bytes.copy_from_slice(report);
        Ok(Self(bytes))
    }
    pub fn as_bytes(&self) -> &[u8; REPORT_LEN] {
        &self.0
    }
    pub fn payload(&self) -> &[u8] {
        &self.0[1..]
    }
}

fn header(command: u8, pages: u8) -> FeatureFrame {
    let mut report = [0; REPORT_LEN];
    report[1] = 4;
    report[2] = command;
    report[9] = pages;
    FeatureFrame(report)
}

/// Observed preamble; the exact device state transition is not independently verified.
pub fn begin_configuration_frame() -> FeatureFrame {
    header(0x18, 0)
}
/// Observed end/commit frame. This is not evidence that flash persistence occurred.
pub fn end_configuration_frame() -> FeatureFrame {
    header(0x02, 0)
}
/// Observed after scalar/keymap writes; the meaning of 04 F0 remains unknown.
pub fn unknown_f0_frame() -> FeatureFrame {
    header(0xf0, 0)
}
pub fn lighting_selection_frame() -> FeatureFrame {
    header(0x13, 1)
}

/// SQL schema and FUN_00408520 map fields to the body in FUN_0042afd0.
/// `stored_color_bytes` are low, middle, high bytes of the stored DWORD. Their
/// physical RGB/BGR channel semantics are deliberately not assumed.
/// REA ev_ceb41426be05537122bc104cf6e5425c37aae245c3f6ecbb25c66047c2972b44,
/// ev_4ec0d5e3022af8d3ccf8468984271d5542e48fc72a8747d6d276be850ad45ed6.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightingFields {
    pub mode: u8,
    pub stored_color_bytes: [u8; 3],
    pub colorful: u8,
    pub brightness: u8,
    pub speed: u8,
    pub direction: u8,
}
pub fn lighting_body(fields: LightingFields) -> Result<FeatureFrame, FrameError> {
    // Bounds come from the shipped S75 layout, not a claim about every Sonix device.
    if fields.brightness > 5 {
        return Err(FrameError::Parameter("S75 brightness exceeds 5"));
    }
    if fields.speed > 5 {
        return Err(FrameError::Parameter("S75 speed exceeds 5"));
    }
    let mut payload = [0; PAYLOAD_LEN];
    payload[0] = fields.mode;
    if fields.mode != 0 {
        payload[1..4].copy_from_slice(&fields.stored_color_bytes);
        payload[8] = fields.colorful;
        payload[9] = fields.brightness;
        payload[10] = fields.speed;
        payload[11] = fields.direction;
    }
    // A constant marker, not a computed checksum.
    payload[14..16].copy_from_slice(&[0xaa, 0x55]);
    FeatureFrame::from_payload(&payload)
}

/// FUN_00427f60 writes eight pages; FUN_0042adb0 requests nine. The unexplained
/// extra read page prevents treating the write body as a complete backup format.
/// REA ev_a7bf306050961e8a80dcfed86611dbe81ead3d32cd79d00f193ddcab217d5bde,
/// ev_7a707e412b6559185c2519de8a4f55881df2f638d4a2e05984b19d3c1c1731b1.
pub fn per_key_write_selection_frame() -> FeatureFrame {
    header(0x20, 8)
}
pub fn per_key_read_request_frame() -> FeatureFrame {
    header(0xf5, 9)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PerKeyRecord {
    pub index: u8,
    /// Already scaled wire bytes; no RGB/BGR or brightness transformation implied.
    pub channels: [u8; 3],
}
/// Records are packed sequentially, each carrying its own key/light index.
/// The native driver scales UI color components by brightness / 256 first.
/// Callers supply the resulting raw bytes here; this function does not rescale.
pub fn per_key_write_body(records: &[PerKeyRecord]) -> Result<[FeatureFrame; 8], FrameError> {
    if records.len() > PER_KEY_RECORD_CAPACITY {
        return Err(FrameError::TooManyRecords(records.len()));
    }
    let mut seen = [false; 256];
    let mut body = [0; PER_KEY_WRITE_PAGES * PAYLOAD_LEN];
    for (record, chunk) in records.iter().zip(body.chunks_exact_mut(4)) {
        if seen[record.index as usize] {
            return Err(FrameError::DuplicateIndex(record.index));
        }
        seen[record.index as usize] = true;
        chunk[0] = record.index;
        chunk[1..].copy_from_slice(&record.channels);
    }
    Ok(std::array::from_fn(|page| {
        let mut report = [0; REPORT_LEN];
        report[1..].copy_from_slice(&body[page * PAYLOAD_LEN..(page + 1) * PAYLOAD_LEN]);
        FeatureFrame(report)
    }))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeymapLayer {
    Base,
    Fn,
}
/// FUN_00418550 chooses 04 11 / 04 27 and sends nine data pages.
/// REA ev_549681fa4dc94b77eb4c4e2c8d85514e9d380f3f8bad17906b0837785ec8c6bb.
/// This header alone is not a complete or validated layer upload transaction.
pub fn keymap_selection_frame(layer: KeymapLayer) -> FeatureFrame {
    header(
        match layer {
            KeymapLayer::Base => 0x11,
            KeymapLayer::Fn => 0x27,
        },
        9,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frames_have_exact_feature_size_and_id() {
        let frame = begin_configuration_frame();
        assert_eq!(frame.as_bytes().len(), 65);
        assert_eq!(&frame.as_bytes()[..4], &[0, 4, 0x18, 0]);
        assert_eq!(FeatureFrame::decode(frame.as_bytes()), Ok(frame));
        assert!(matches!(
            FeatureFrame::decode(&[0; 64]),
            Err(FrameError::Length { .. })
        ));
        let mut invalid = [0; 65];
        invalid[0] = 6;
        assert_eq!(FeatureFrame::decode(&invalid), Err(FrameError::ReportId(6)));
        assert!(FeatureFrame::from_payload(&[0; 63]).is_err());
        assert!(FeatureFrame::from_payload(&[0; 65]).is_err());
    }
    fn fields() -> LightingFields {
        LightingFields {
            mode: 11,
            stored_color_bytes: [0x12, 0x34, 0x56],
            colorful: 1,
            brightness: 5,
            speed: 3,
            direction: 2,
        }
    }
    #[test]
    fn lighting_offsets_match_observed_body() {
        let report = lighting_body(fields()).unwrap();
        let body = report.payload();
        assert_eq!(&body[..4], &[11, 0x12, 0x34, 0x56]);
        assert_eq!(&body[8..12], &[1, 5, 3, 2]);
        assert_eq!(&body[14..16], &[0xaa, 0x55]);
        assert!(body[16..].iter().all(|b| *b == 0));
    }
    #[test]
    fn off_mode_zeros_parameters_but_retains_marker() {
        let mut settings = fields();
        settings.mode = 0;
        let body = lighting_body(settings).unwrap();
        assert!(body.payload()[..14].iter().all(|b| *b == 0));
        assert_eq!(&body.payload()[14..16], &[0xaa, 0x55]);
    }
    #[test]
    fn reject_values_beyond_shipped_layout_bounds() {
        let mut settings = fields();
        settings.brightness = 6;
        assert!(lighting_body(settings).is_err());
        settings = fields();
        settings.speed = 6;
        assert!(lighting_body(settings).is_err());
    }
    #[test]
    fn observed_selection_commit_and_unknown_frames() {
        let cases = [
            (lighting_selection_frame(), 0x13, 1),
            (per_key_write_selection_frame(), 0x20, 8),
            (per_key_read_request_frame(), 0xf5, 9),
            (keymap_selection_frame(KeymapLayer::Base), 0x11, 9),
            (keymap_selection_frame(KeymapLayer::Fn), 0x27, 9),
            (end_configuration_frame(), 0x02, 0),
            (unknown_f0_frame(), 0xf0, 0),
        ];
        for (frame, command, pages) in cases {
            assert_eq!(&frame.payload()[..2], &[4, command]);
            assert_eq!(frame.payload()[8], pages);
            assert!(frame
                .payload()
                .iter()
                .enumerate()
                .all(|(i, b)| i < 2 || i == 8 || *b == 0));
        }
        assert_eq!(PER_KEY_READ_PAGES * PAYLOAD_LEN, 576);
        assert_eq!(PER_KEY_WRITE_PAGES * PAYLOAD_LEN, 512);
    }
    #[test]
    fn per_key_records_are_sequential_and_page_aligned() {
        let records: Vec<_> = (0..128)
            .map(|index| PerKeyRecord {
                index,
                channels: [index, 0xa5, 0x5a],
            })
            .collect();
        let pages = per_key_write_body(&records).unwrap();
        assert_eq!(pages.len(), 8);
        assert_eq!(
            &pages[0].payload()[..8],
            &[0, 0, 0xa5, 0x5a, 1, 1, 0xa5, 0x5a]
        );
        assert_eq!(&pages[1].payload()[..4], &[16, 16, 0xa5, 0x5a]);
        assert_eq!(&pages[7].payload()[60..], &[127, 127, 0xa5, 0x5a]);
    }
    #[test]
    fn reject_overfull_or_ambiguous_per_key_bodies() {
        let record = PerKeyRecord {
            index: 3,
            channels: [1, 2, 3],
        };
        assert_eq!(
            per_key_write_body(&[record, record]),
            Err(FrameError::DuplicateIndex(3))
        );
        assert_eq!(
            per_key_write_body(&[record; 129]),
            Err(FrameError::TooManyRecords(129))
        );
        assert!(per_key_write_body(&[])
            .unwrap()
            .iter()
            .all(|p| p.payload().iter().all(|b| *b == 0)));
    }
}
