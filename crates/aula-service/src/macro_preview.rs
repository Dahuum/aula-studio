//! Original F75 G5 macro record conversion for offline inspection only.
//! No HID dependency, command/header encoder, binding, or uploader.
use crate::{validate_macros, Macro, Result};
use aula_protocol::macro_wire::{encode_pool, EventKind, RawEvent, RawRecord};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct WirePreview {
    pub schema_version: u32,
    pub protocol: &'static str,
    pub evidence: &'static str,
    pub hardware_validated: bool,
    pub upload_available: bool,
    pub macro_id: String,
    pub name: String,
    pub nominal_duration_ms: u64,
    pub key_events: usize,
    pub record_bytes: Vec<u8>,
    pub single_record_pool_bytes: Vec<u8>,
    pub warnings: Vec<&'static str>,
}

/// Keyboard subset independently recovered from the native VK/HID table.
/// Do not guess a representation for unsupported application/vendor usages.
fn known_keyboard_usage(key: u16) -> bool {
    matches!(key, 4..=101 | 104 | 133 | 135..=139 | 144..=145 | 224..=231)
}
pub fn preview(m: &Macro) -> Result<WirePreview> {
    validate_macros(std::slice::from_ref(m))?;
    let mut events: Vec<RawEvent> = Vec::new();
    let mut duration = 0_u64;
    for event in &m.events {
        if event.kind == "delay" {
            let delay = event.ms.expect("validated delay");
            duration += u64::from(delay);
            if let Some(previous) = events.last_mut() {
                previous.ticks = previous
                    .ticks
                    .checked_add(delay)
                    .ok_or("Macro delay overflow")?;
                if previous.ticks > 0xfffff {
                    return Err("Adjacent delays exceed the observed 20-bit field; no clamping is performed".into());
                }
            } else if delay != 0 {
                return Err(
                    "A leading nonzero delay has no established original F75 G5 representation"
                        .into(),
                );
            }
        } else {
            let key = event.key.expect("validated keyboard event");
            if !known_keyboard_usage(key) {
                return Err(
                    "Keyboard usage is absent from the original application's macro mapping".into(),
                );
            }
            events.push(RawEvent {
                kind: if key >= 224 {
                    EventKind::Modifier
                } else {
                    EventKind::Keyboard
                },
                flagged: event.kind == "up",
                ticks: 0,
                value: key as u8,
            });
        }
    }
    if events.is_empty() {
        return Err("At least one balanced keyboard action is required for a wire preview".into());
    }
    let record = RawRecord {
        name: m.name.clone(),
        events,
    };
    let record_bytes = record.encode().map_err(|e| e.to_string())?;
    let pool = encode_pool(std::slice::from_ref(&record)).map_err(|e| e.to_string())?;
    Ok(WirePreview {
        schema_version: 1,
        protocol: "original_f75_g5",
        evidence: "official_binary_static_analysis_only",
        hardware_validated: false,
        upload_available: false,
        macro_id: m.id.clone(), name: m.name.clone(), nominal_duration_ms: duration,
        key_events: record.events.len(), record_bytes, single_record_pool_bytes: pool,
        warnings: vec![
            "Offline application-format reconstruction; no HID command is generated or sent.",
            "Delays follow events in the official application; physical firmware playback/timing is unverified.",
            "Name and size bounds are the observed application format/default, not measured hardware capacity.",
            "Bindings, repeat behavior, persistence and interrupted-upload recovery are not implemented.",
            "This original F75 G5 format is not a Sonix or wireless macro format.",
        ],
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalMacroFile {
    schema_version: u32,
    #[serde(rename = "macro")]
    local_macro: Macro,
}
pub fn preview_json(text: &str) -> Result<WirePreview> {
    if text.len() > 512 * 1024 {
        return Err("Macro JSON exceeds 512 KiB".into());
    }
    let input: LocalMacroFile = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if input.schema_version != 1 {
        return Err("Unsupported local macro schema".into());
    }
    preview(&input.local_macro)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MacroEvent;
    fn key(kind: &str, usage: u16) -> MacroEvent {
        MacroEvent {
            kind: kind.into(),
            key: Some(usage),
            ms: None,
        }
    }
    fn delay(ms: u32) -> MacroEvent {
        MacroEvent {
            kind: "delay".into(),
            key: None,
            ms: Some(ms),
        }
    }
    fn sample(events: Vec<MacroEvent>) -> Macro {
        Macro {
            id: "test".into(),
            name: "A".into(),
            events,
        }
    }
    #[test]
    fn native_keyboard_modifier_after_delay_fixture() {
        let m = sample(vec![
            key("down", 225),
            delay(25),
            key("down", 4),
            delay(40),
            key("up", 4),
            delay(5),
            key("up", 225),
            delay(10),
        ]);
        let p = preview(&m).unwrap();
        assert_eq!(
            p.record_bytes,
            [2, 65, 0, 0x10, 0, 25, 225, 0, 0, 40, 4, 0x80, 0, 5, 4, 0x90, 0, 10, 225]
        );
        assert_eq!(&p.single_record_pool_bytes[..4], &[4, 0, 19, 0]);
        assert_eq!(&p.single_record_pool_bytes[4..], p.record_bytes);
        assert_eq!(p.nominal_duration_ms, 80);
        assert!(!p.upload_available && !p.hardware_validated);
        assert_eq!(RawRecord::decode(&p.record_bytes).unwrap().events.len(), 4);
    }
    #[test]
    fn unrepresentable_timing_names_and_usages_are_rejected_not_normalized() {
        assert!(preview(&sample(vec![delay(1), key("down", 4), key("up", 4)])).is_err());
        let mut m = sample(vec![key("down", 4)]);
        m.events.extend(std::iter::repeat_n(delay(60000), 18));
        m.events.push(key("up", 4));
        assert!(preview(&m).is_err());
        let mut m = sample(vec![key("down", 4), key("up", 4)]);
        m.name = "x".repeat(31);
        assert!(preview(&m).is_err());
        assert!(preview(&sample(vec![key("down", 102), key("up", 102)])).is_err());
        assert!(preview(&sample(vec![key("down", 4)])).is_err());
        assert!(preview(&sample(vec![delay(0)])).is_err());
    }
    #[test]
    fn adjacent_and_trailing_delays_preserve_exact_application_times() {
        let p = preview(&sample(vec![
            delay(0),
            key("down", 4),
            delay(60000),
            delay(60000),
            key("up", 4),
            delay(9),
        ]))
        .unwrap();
        let r = RawRecord::decode(&p.record_bytes).unwrap();
        assert_eq!(r.events[0].ticks, 120000);
        assert_eq!(r.events[1].ticks, 9);
        assert_eq!(p.nominal_duration_ms, 120009);
    }
    #[test]
    fn default_pool_bound_and_file_schema_are_enforced() {
        let m = sample(
            (0..512)
                .flat_map(|_| [key("down", 4), key("up", 4)])
                .collect(),
        );
        assert!(preview(&m).is_err());
        let text =
            serde_json::json!({"schema_version":1,"macro":sample(vec![key("down",4),key("up",4)])})
                .to_string();
        assert!(preview_json(&text).is_ok());
        assert!(
            preview_json(&text.replace("\"schema_version\":1", "\"schema_version\":2")).is_err()
        );
        assert!(preview_json(&text.replace(
            "\"schema_version\":1",
            "\"extra\":true,\"schema_version\":1"
        ))
        .is_err());
    }
}
