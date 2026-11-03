//! Static-evidence macro record/pool format for offline research only.
//! Timing values remain raw. The official recorder establishes the action
//! flag as release; delay placement across its editors is still unresolved.
//! Nothing here converts local millisecond macros or uploads them.
use crate::ProtocolError;
const MAX_RECORD: usize = 4096;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Keyboard,
    Modifier,
    MouseButton,
    MoveX,
    MoveY,
    Wheel,
}
impl EventKind {
    fn code(self) -> u8 {
        match self {
            Self::Keyboard => 0,
            Self::Modifier => 1,
            Self::MouseButton => 2,
            Self::MoveX => 3,
            Self::MoveY => 4,
            Self::Wheel => 5,
        }
    }
    fn decode(n: u8) -> Result<Self, ProtocolError> {
        match n {
            0 => Ok(Self::Keyboard),
            1 => Ok(Self::Modifier),
            2 => Ok(Self::MouseButton),
            3 => Ok(Self::MoveX),
            4 => Ok(Self::MoveY),
            5 => Ok(Self::Wheel),
            _ => Err(ProtocolError::Unsupported("unknown macro event kind")),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawEvent {
    pub kind: EventKind,
    pub flagged: bool,
    pub ticks: u32,
    pub value: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyTransition {
    Press,
    Release,
}
impl RawEvent {
    /// Interpretation established from the supplied application's recorder,
    /// event structure and wire converter, not from hardware playback.
    pub fn key_transition(self) -> Option<KeyTransition> {
        match self.kind {
            EventKind::Keyboard | EventKind::Modifier | EventKind::MouseButton => {
                Some(if self.flagged {
                    KeyTransition::Release
                } else {
                    KeyTransition::Press
                })
            }
            _ => None,
        }
    }
    pub fn encode(self) -> Result<[u8; 4], ProtocolError> {
        if self.ticks > 0xfffff {
            return Err(ProtocolError::Unsupported("macro ticks exceed20 bits"));
        }
        let valid = match self.kind {
            EventKind::Keyboard => self.value != 0,
            EventKind::Modifier => (0xe0..=0xe7).contains(&self.value),
            EventKind::MouseButton => [1, 2, 4, 8, 16].contains(&self.value),
            _ => self.value != 0 && !self.flagged,
        };
        if !valid {
            return Err(ProtocolError::Unsupported(
                "invalid raw macro event value/flag",
            ));
        }
        Ok([
            (self.kind.code() << 4)
                | ((self.ticks >> 16) as u8)
                | if self.flagged { 0x80 } else { 0 },
            (self.ticks >> 8) as u8,
            self.ticks as u8,
            self.value,
        ])
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() != 4 {
            return Err(ProtocolError::Length {
                expected: 4,
                actual: bytes.len(),
            });
        }
        let e = Self {
            kind: EventKind::decode((bytes[0] >> 4) & 7)?,
            flagged: bytes[0] & 0x80 != 0,
            ticks: (u32::from(bytes[0] & 15) << 16)
                | (u32::from(bytes[1]) << 8)
                | u32::from(bytes[2]),
            value: bytes[3],
        };
        e.encode()?;
        Ok(e)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawRecord {
    pub name: String,
    pub events: Vec<RawEvent>,
}
impl RawRecord {
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let name = self.name.encode_utf16().collect::<Vec<_>>();
        if name.len() > 30 || name.contains(&0) {
            return Err(ProtocolError::Unsupported(
                "macro name exceeds30 UTF16 units or contains NUL",
            ));
        }
        let size = 1
            + name.len() * 2
            + self
                .events
                .len()
                .checked_mul(4)
                .ok_or(ProtocolError::Unsupported("macro size overflow"))?;
        if size > MAX_RECORD {
            return Err(ProtocolError::Unsupported(
                "macro record exceeds official application buffer",
            ));
        }
        let mut b = Vec::with_capacity(size);
        b.push((name.len() * 2) as u8);
        for c in name {
            b.extend_from_slice(&c.to_le_bytes())
        }
        for e in &self.events {
            b.extend_from_slice(&e.encode()?)
        }
        Ok(b)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.is_empty() || bytes.len() > MAX_RECORD {
            return Err(ProtocolError::Unsupported("macro record size"));
        }
        let n = usize::from(bytes[0]);
        if n > 60 || n & 1 != 0 || n + 1 > bytes.len() || (bytes.len() - n - 1) & 3 != 0 {
            return Err(ProtocolError::Unsupported("macro name/event boundary"));
        }
        let units = bytes[1..1 + n]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect::<Vec<_>>();
        let name = String::from_utf16(&units)
            .map_err(|_| ProtocolError::Unsupported("invalid UTF16 macro name"))?;
        let events = bytes[1 + n..]
            .chunks_exact(4)
            .map(RawEvent::decode)
            .collect::<Result<Vec<_>, _>>()?;
        let record = Self { name, events };
        record.encode()?;
        Ok(record)
    }
}
/// The4096B ceiling is the supplied application's default, not a physical
/// capacity measurement. Slot count is external to the directory.
pub fn encode_pool(records: &[RawRecord]) -> Result<Vec<u8>, ProtocolError> {
    if records.len() > 255 {
        return Err(ProtocolError::Unsupported(
            "macro slot count exceeds wire slot byte",
        ));
    }
    let encoded = records
        .iter()
        .map(RawRecord::encode)
        .collect::<Result<Vec<_>, _>>()?;
    let mut offset = records.len() * 4;
    let total = encoded
        .iter()
        .try_fold(offset, |n, r| n.checked_add(r.len()))
        .ok_or(ProtocolError::Unsupported("macro pool size overflow"))?;
    if total > 4096 {
        return Err(ProtocolError::Unsupported(
            "macro pool exceeds package default buffer",
        ));
    }
    let mut b = Vec::with_capacity(total);
    for record in &encoded {
        b.extend_from_slice(&(offset as u16).to_le_bytes());
        b.extend_from_slice(&(record.len() as u16).to_le_bytes());
        offset += record.len();
    }
    for r in encoded {
        b.extend_from_slice(&r)
    }
    Ok(b)
}
pub fn decode_pool(bytes: &[u8], count: u8) -> Result<Vec<RawRecord>, ProtocolError> {
    let directory = usize::from(count) * 4;
    if bytes.len() > 4096 || directory > bytes.len() {
        return Err(ProtocolError::Unsupported("macro pool bounds"));
    }
    let mut expected = directory;
    let mut records = Vec::with_capacity(usize::from(count));
    for item in bytes[..directory].chunks_exact(4) {
        let offset = usize::from(u16::from_le_bytes([item[0], item[1]]));
        let size = usize::from(u16::from_le_bytes([item[2], item[3]]));
        let end = offset
            .checked_add(size)
            .ok_or(ProtocolError::Unsupported("macro directory overflow"))?;
        if offset != expected || size == 0 || end > bytes.len() {
            return Err(ProtocolError::Unsupported(
                "macro directory is overlapping, sparse or out of bounds",
            ));
        }
        records.push(RawRecord::decode(&bytes[offset..end])?);
        expected = end;
    }
    if expected != bytes.len() {
        return Err(ProtocolError::Unsupported("macro pool trailing bytes"));
    }
    Ok(records)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn record() -> RawRecord {
        RawRecord {
            name: "A".into(),
            events: vec![
                RawEvent {
                    kind: EventKind::Keyboard,
                    flagged: true,
                    ticks: 0x12345,
                    value: 4,
                },
                RawEvent {
                    kind: EventKind::Modifier,
                    flagged: false,
                    ticks: 0xab,
                    value: 0xe0,
                },
            ],
        }
    }
    #[test]
    fn native_branch_fixture() {
        let bytes = [2, 0x41, 0, 0x81, 0x23, 0x45, 4, 0x10, 0, 0xab, 0xe0];
        assert_eq!(record().encode().unwrap(), bytes);
        assert_eq!(RawRecord::decode(&bytes).unwrap(), record());
    }
    #[test]
    fn recorder_release_flag_matches_converter_branch() {
        // Recorder action1=press becomes unflagged; action0=release sets80.
        for (kind_bits, value) in [(0x00, 4), (0x10, 0xe0), (0x20, 1)] {
            let press = RawEvent::decode(&[kind_bits, 0, 50, value]).unwrap();
            let release = RawEvent::decode(&[kind_bits | 0x80, 0, 25, value]).unwrap();
            assert_eq!(press.key_transition(), Some(KeyTransition::Press));
            assert_eq!(release.key_transition(), Some(KeyTransition::Release));
        }
        assert_eq!(
            RawEvent::decode(&[0x30, 0, 0, 1]).unwrap().key_transition(),
            None
        );
    }
    #[test]
    fn raw_limits_and_invalid_flag() {
        let mut e = record().events[0];
        e.ticks = 0x100000;
        assert!(e.encode().is_err());
        e.ticks = 1;
        e.kind = EventKind::MoveX;
        assert!(e.encode().is_err());
        let mut r = record();
        r.name = "x".repeat(31);
        assert!(r.encode().is_err());
        assert!(RawRecord::decode(&[3, 1, 2, 3]).is_err());
        assert!(RawEvent::decode(&[0x60, 0, 0, 1]).is_err());
    }
    #[test]
    fn pool_directory_fixture_and_corruption() {
        let r = record();
        let b = encode_pool(&[r.clone(), r.clone()]).unwrap();
        assert_eq!(&b[..8], &[8, 0, 11, 0, 19, 0, 11, 0]);
        assert_eq!(decode_pool(&b, 2).unwrap(), vec![r.clone(), r]);
        let mut bad = b.clone();
        bad[4] = 8;
        assert!(decode_pool(&bad, 2).is_err());
        bad = b;
        bad.push(0);
        assert!(decode_pool(&bad, 2).is_err());
        assert!(decode_pool(&[255; 4097], 1).is_err());
    }
}
