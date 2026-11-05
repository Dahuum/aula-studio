//! F2088 normal panel-lighting packet recovered from ShinetekTools.exe.
//! Offline only: no device handles, transport, reset or arbitrary command API.
use crate::Result;
pub const REPORT_BYTES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelLighting {
    pub effect: u8,
    pub brightness: u8,
    pub speed: u8,
    /// Application selection byte, not an RGB value or verified mode name.
    pub selection: u8,
}
impl PanelLighting {
    pub fn encode(self) -> Result<[u8; REPORT_BYTES]> {
        if self.effect >= 20 || self.brightness > 13 || self.speed > 2 || self.selection > 3 {
            return Err("F2088 parameters exceed the observed application parser bounds".into());
        }
        // Normal sender0041fe90 replaces the direction byte with this mapped
        // selection and zeroes the last byte. Do not infer direction support.
        let selector = self.selection.saturating_sub(1);
        Ok([
            7,
            0xff,
            0xff,
            self.effect,
            self.brightness,
            self.speed,
            selector,
            0,
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodedPanel {
    pub effect: u8,
    pub brightness: u8,
    pub speed: u8,
    /// Wire selector; application selections0 and1 both encode as0.
    pub wire_selector: u8,
}
pub fn decode(bytes: &[u8]) -> Result<DecodedPanel> {
    if bytes.len() != REPORT_BYTES || bytes[..3] != [7, 0xff, 0xff] || bytes[7] != 0 {
        return Err("Not a normal eight-byte F2088 panel-lighting frame".into());
    }
    if bytes[3] >= 20 || bytes[4] > 13 || bytes[5] > 2 || bytes[6] > 2 {
        return Err("F2088 frame exceeds established normal-sender bounds".into());
    }
    Ok(DecodedPanel {
        effect: bytes[3],
        brightness: bytes[4],
        speed: bytes[5],
        wire_selector: bytes[6],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_normal_sender_fixture_has_id7_and_eight_bytes() {
        let b = PanelLighting {
            effect: 4,
            brightness: 13,
            speed: 2,
            selection: 0,
        }
        .encode()
        .unwrap();
        assert_eq!(b, [7, 0xff, 0xff, 4, 13, 2, 0, 0]);
        assert_eq!(
            decode(&b).unwrap(),
            DecodedPanel {
                effect: 4,
                brightness: 13,
                speed: 2,
                wire_selector: 0
            }
        );
        assert!(decode(&[6, 0xff, 0xff, 4, 13, 2, 0, 0]).is_err());
        assert!(decode(&[7; 65]).is_err());
        assert!(decode(&b[..7]).is_err());
    }
    #[test]
    fn native_selection_mapping_is_lossy_and_never_inferred_back() {
        for (input, wire) in [(0, 0), (1, 0), (2, 1), (3, 2)] {
            let b = PanelLighting {
                effect: 1,
                brightness: 13,
                speed: 0,
                selection: input,
            }
            .encode()
            .unwrap();
            assert_eq!(b, [7, 0xff, 0xff, 1, 13, 0, wire, 0]);
            assert_eq!(decode(&b).unwrap().wire_selector, wire);
        }
    }
    #[test]
    fn unknown_commands_effects_and_out_of_bound_values_are_rejected() {
        let p = PanelLighting {
            effect: 0,
            brightness: 0,
            speed: 0,
            selection: 0,
        };
        for invalid in [
            PanelLighting { effect: 20, ..p },
            PanelLighting { effect: 255, ..p },
            PanelLighting {
                brightness: 14,
                ..p
            },
            PanelLighting { speed: 3, ..p },
            PanelLighting { selection: 4, ..p },
        ] {
            assert!(invalid.encode().is_err());
        }
        for index in [0, 1, 2, 7] {
            let mut b = p.encode().unwrap();
            b[index] = 0xf1;
            assert!(decode(&b).is_err());
        }
    }
}
