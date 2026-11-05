//! Prepared base/Fn/per-key actions backed by complete bank snapshots.
use aula_hid::mapping_transaction::MappingPlan;
use aula_protocol::{matrix_index_for_usage, KeyAction, Layer};
use aula_service::{atomic_write, Profile};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, String>;
pub struct PreparedMapping {
    pub plan: MappingPlan,
    pub backup_id: String,
    pub kind: String,
    created: Instant,
}
#[derive(Default)]
pub struct PendingMappings(HashMap<String, PreparedMapping>);
impl PendingMappings {
    pub fn clear(&mut self) {
        self.0.clear()
    }
    pub fn insert(&mut self, p: PreparedMapping) -> Result<String> {
        let mut bytes = [0; 32];
        std::fs::File::open("/dev/urandom")
            .and_then(|mut f| f.read_exact(&mut bytes))
            .map_err(|e| e.to_string())?;
        let token = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
        self.clear();
        self.0.insert(token.clone(), p);
        Ok(token)
    }
    pub fn consume(&mut self, token: &str) -> Result<PreparedMapping> {
        let p = self
            .0
            .remove(token)
            .ok_or("Review is no longer valid. Prepare again.")?;
        if p.created.elapsed() > Duration::from_secs(120) {
            return Err("Review expired. Prepare again.".into());
        }
        Ok(p)
    }
}
fn selected_path() -> Result<String> {
    let devices = aula_hid::discover().map_err(|e| e.to_string())?;
    let mut paths = devices
        .into_iter()
        .filter(|d| d.read_supported)
        .map(|d| d.path)
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    if paths.len() != 1 {
        return Err("Connect exactly one supported wired F75.".into());
    }
    Ok(paths.remove(0))
}
pub fn prepare(profile: Profile, kind: String, root: PathBuf) -> Result<(PreparedMapping, Value)> {
    profile.validate()?;
    let path = selected_path()?;
    let mut changes = vec![];
    let plan = match kind.as_str() {
        "base" | "fn" => {
            let layer = if kind == "base" {
                Layer::Base
            } else {
                Layer::Fn
            };
            let (snapshot, map) =
                aula_hid::read_keymap_snapshot(&path, layer).map_err(|e| e.to_string())?;
            let edits = profile_edits(&profile, layer)?;
            let plan =
                MappingPlan::remap(snapshot, map.clone(), &edits).map_err(|e| e.to_string())?;
            for (usage, _action) in edits {
                let index = matrix_index_for_usage(usage).map_err(|e| e.to_string())?;
                let o = 8 + 4 * index;
                let next = plan.keymap.as_ref().ok_or("Missing planned keymap")?;
                if map.as_bytes()[o..o + 4] != next.as_bytes()[o..o + 4] {
                    changes.push(json!({"field":format!("{} source {}",kind,key_name(usage)),"before":binding_name(&map,index),"after":binding_name(next,index)}));
                }
            }
            plan
        }
        "per_key" => {
            let (snapshot, bank) =
                aula_hid::read_per_key_snapshot(&path).map_err(|e| e.to_string())?;
            let edits = profile
                .per_key
                .iter()
                .map(|(k, v)| {
                    Ok((
                        if k == "fn" {
                            0
                        } else {
                            k.parse::<u16>().map_err(|e| e.to_string())?
                        },
                        *v,
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            let plan =
                MappingPlan::per_key(snapshot, bank.clone(), &edits).map_err(|e| e.to_string())?;
            for (usage, color) in edits {
                let old = bank
                    .get(if usage == 0 {
                        aula_protocol::FN_MATRIX_INDEX
                    } else {
                        matrix_index_for_usage(usage).map_err(|e| e.to_string())?
                    })
                    .map_err(|e| e.to_string())?;
                if old != color {
                    changes.push(json!({"field":if usage==0{"Fn color".to_string()}else{format!("Key HID {usage} color")},"before":old,"after":color}))
                }
            }
            if plan.config != plan.before.config {
                changes.push(
                    json!({"field":"Lighting mode","before":plan.before.config[10],"after":21}),
                )
            }
            plan
        }
        _ => return Err("Unknown mapping operation".into()),
    };
    publish(root, kind, plan, changes, None)
}
fn publish(
    root: PathBuf,
    kind: String,
    plan: MappingPlan,
    changes: Vec<Value>,
    summary: Option<&str>,
) -> Result<(PreparedMapping, Value)> {
    if !plan.has_changes() {
        return Err(
            "Selected draft assignments already match the keyboard; no writes needed.".into(),
        );
    }
    let id = crate::operations::backup(&root, &plan.before)?;
    let record = json!({"id":id,"created_at":id,"kind":"hardware","mapping_kind":kind,"device":plan.before.device,"snapshot":plan.before,"keymap":plan.keymap_before.as_ref().map(|m|m.as_bytes().to_vec()),"per_key":plan.per_key_before.as_ref().map(|c|c.as_bytes().to_vec())});
    atomic_write(
        &root.join("backups").join(format!("{id}.json")),
        &serde_json::to_vec_pretty(&record).unwrap(),
    )?;
    let details = json!({"device":format!("{} · {} · USB 258a:010c",plan.before.device.product_name,plan.before.device.path),"summary":if let Some(summary)=summary{summary}else if kind=="per_key"{"Write the listed per-key colors and activate custom lighting. Other LEDs are preserved."}else{"Write only the listed assignments in the selected layer. Other keys and the other layer are preserved."},"backup_id":id,"changes":changes,"warnings":["Protocol and read mapping are verified; writes have not yet been locally tested on this hardware.","Backup includes the complete affected bank. Readback will be checked; power-cycle persistence is unknown.","Fn and function-key input remapping stays protected; RGB is independent. No firmware or reset commands are sent."]});
    Ok((
        PreparedMapping {
            plan,
            backup_id: id,
            kind,
            created: Instant::now(),
        },
        details,
    ))
}
pub fn prepare_restore(id: String, root: PathBuf) -> Result<(PreparedMapping, Value)> {
    if !aula_service::valid_id(&id) {
        return Err("Invalid backup ID".into());
    }
    let path = root.join("backups").join(format!("{id}.json"));
    if std::fs::metadata(&path).map_err(|e| e.to_string())?.len() > 65536 {
        return Err("Backup size exceeds limit".into());
    }
    let value: Value = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if value["kind"] != "hardware" {
        return Err("Select a physical mapping backup.".into());
    }
    let saved: aula_hid::Snapshot =
        serde_json::from_value(value["snapshot"].clone()).map_err(|e| e.to_string())?;
    saved.validate().map_err(|e| e.to_string())?;
    if saved.source != "physical_read" {
        return Err("Backup is not a physical read.".into());
    }
    let kind = value["mapping_kind"]
        .as_str()
        .ok_or("Snapshot does not contain a mapping bank")?
        .strip_prefix("restore_")
        .unwrap_or(value["mapping_kind"].as_str().unwrap())
        .to_string();
    let path = selected_path()?;
    let plan = match kind.as_str() {
        "base" | "fn" => {
            let layer = if kind == "base" {
                Layer::Base
            } else {
                Layer::Fn
            };
            let (current, map) =
                aula_hid::read_keymap_snapshot(&path, layer).map_err(|e| e.to_string())?;
            check_same_family(&current, &saved)?;
            let bytes: Vec<u8> =
                serde_json::from_value(value["keymap"].clone()).map_err(|e| e.to_string())?;
            let target = aula_protocol::Keymap::parse(&bytes).map_err(|e| e.to_string())?;
            MappingPlan::restore_keymap(current, map, target).map_err(|e| e.to_string())?
        }
        "per_key" => {
            let (current, colors) =
                aula_hid::read_per_key_snapshot(&path).map_err(|e| e.to_string())?;
            check_same_family(&current, &saved)?;
            let bytes: Vec<u8> =
                serde_json::from_value(value["per_key"].clone()).map_err(|e| e.to_string())?;
            let target = aula_protocol::PerKeyColors::parse(&bytes).map_err(|e| e.to_string())?;
            MappingPlan::restore_colors(current, colors, target).map_err(|e| e.to_string())?
        }
        _ => return Err("Unknown backup bank".into()),
    };
    let mut changes = vec![];
    if let (Some(a), Some(b)) = (&plan.keymap_before, &plan.keymap) {
        for usage in 1..=255u16 {
            if let Ok(i) = matrix_index_for_usage(usage) {
                let o = 8 + 4 * i;
                if a.as_bytes()[o..o + 4] != b.as_bytes()[o..o + 4] {
                    changes.push(json!({"field":format!("Restore source HID {usage}"),"before":binding_name(a,i),"after":binding_name(b,i)}));
                }
            }
        }
    }
    if let (Some(a), Some(b)) = (&plan.per_key_before, &plan.per_key) {
        for i in (1..=255u16)
            .filter_map(|u| matrix_index_for_usage(u).ok())
            .chain(std::iter::once(aula_protocol::FN_MATRIX_INDEX))
        {
            let old = a.get(i).map_err(|e| e.to_string())?;
            let new = b.get(i).map_err(|e| e.to_string())?;
            if old != new {
                changes.push(json!({"field":format!("Restore LED {i}"),"before":old,"after":new}));
            }
        }
    }
    publish(root,format!("restore_{kind}"),plan,changes,Some("Restore the known source/LED entries from this bank backup. Protected controls, unknown positions, current lighting mode and other banks are preserved. A new before-restore bank backup is created."))
}
fn check_same_family(current: &aula_hid::Snapshot, saved: &aula_hid::Snapshot) -> Result<()> {
    let a = &current.device;
    let b = &saved.device;
    if a.vendor_id != b.vendor_id
        || a.product_id != b.product_id
        || a.interface_number != b.interface_number
        || a.product_name != b.product_name
        || b.feature_report_bytes != Some(520)
    {
        return Err("Snapshot identity/report family does not match this device.".into());
    }
    Ok(())
}
fn verify_backup(root: &Path, p: &PreparedMapping) -> Result<()> {
    crate::operations::verify_snapshot_backup(root, &p.backup_id, &p.plan.before)?;
    let v: Value = serde_json::from_slice(
        &std::fs::read(root.join("backups").join(format!("{}.json", p.backup_id)))
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let expected_map = json!(p.plan.keymap_before.as_ref().map(|m| m.as_bytes().to_vec()));
    let expected_colors = json!(p
        .plan
        .per_key_before
        .as_ref()
        .map(|c| c.as_bytes().to_vec()));
    if v["keymap"] != expected_map || v["per_key"] != expected_colors {
        return Err("Saved mapping backup differs from the review. Prepare again.".into());
    }
    Ok(())
}
pub fn commit(root: PathBuf, token: String, p: PreparedMapping) -> Result<Value> {
    verify_backup(&root, &p)?;
    if token.len() != 64 || !token.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid review token".into());
    }
    let dir = root.join("operations");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{token}.json"));
    let mut journal = json!({"operation":p.kind,"state":"started","backup_id":p.backup_id,"device":p.plan.before.device});
    atomic_write(&path, &serde_json::to_vec_pretty(&journal).unwrap())?;
    match aula_hid::commit_mapping(p.plan) {
        Ok(result) => {
            journal["state"] = json!("readback_verified");
            journal["snapshot"] = json!(result.snapshot);
            journal["keymap"] = json!(result.keymap.as_ref().map(|m| m.as_bytes().to_vec()));
            journal["per_key"] = json!(result.per_key.as_ref().map(|c| c.as_bytes().to_vec()));
            atomic_write(&path,&serde_json::to_vec_pretty(&journal).unwrap()).map_err(|e|format!("Readback matched, but saving the result failed: {e}. Do not automatically retry."))?;
            Ok(
                json!({"message":"Keyboard configuration written and exact readback verified. Power-cycle persistence is not tested.","readback_verified":true,"persistence":"not_tested","backup_id":p.backup_id}),
            )
        }
        Err(e) => {
            journal["state"] = json!("unconfirmed");
            journal["error"] = json!(e.to_string());
            let _ = atomic_write(&path, &serde_json::to_vec_pretty(&journal).unwrap());
            Err(format!(
                "{e}. Keep backup {}; read the keyboard before retrying.",
                p.backup_id
            ))
        }
    }
}
// Shared read result intentionally retains raw bank and decoded known ANSI keys.
pub fn read(kind: String) -> Result<Value> {
    let path = selected_path()?;
    match kind.as_str() {
        "base" | "fn" => {
            let layer = if kind == "base" {
                Layer::Base
            } else {
                Layer::Fn
            };
            let (s, m) = aula_hid::read_keymap_snapshot(&path, layer).map_err(|e| e.to_string())?;
            let (assignments, consumer_assignments, mouse_assignments) = editable_assignments(&m)?;
            Ok(
                json!({"kind":kind,"snapshot":s,"keymap":m.as_bytes().to_vec(),"assignments":assignments,"consumer_assignments":consumer_assignments,"mouse_assignments":mouse_assignments}),
            )
        }
        "per_key" => {
            let (s, c) = aula_hid::read_per_key_snapshot(&path).map_err(|e| e.to_string())?;
            let mut colors = serde_json::Map::new();
            for usage in 1..=255u16 {
                if let Ok(index) = matrix_index_for_usage(usage) {
                    colors.insert(
                        usage.to_string(),
                        json!(c.get(index).map_err(|e| e.to_string())?),
                    );
                }
            }
            colors.insert(
                "fn".into(),
                json!(c
                    .get(aula_protocol::FN_MATRIX_INDEX)
                    .map_err(|e| e.to_string())?),
            );
            Ok(json!({"kind":kind,"snapshot":s,"per_key":c.as_bytes().to_vec(),"colors":colors}))
        }
        _ => Err("Unknown bank".into()),
    }
}

fn key_name(usage: u16) -> String {
    match usage {
        4..=29 => char::from_u32(u32::from(usage) + 61).unwrap().to_string(),
        30..=38 => (usage - 29).to_string(),
        39 => "0".into(),
        40 => "Enter".into(),
        41 => "Escape".into(),
        42 => "Backspace".into(),
        43 => "Tab".into(),
        44 => "Space".into(),
        45 => "-".into(),
        46 => "=".into(),
        47 => "[".into(),
        48 => "]".into(),
        49 => "Backslash".into(),
        51 => ";".into(),
        52 => "Quote".into(),
        53 => "Backquote".into(),
        54 => ",".into(),
        55 => ".".into(),
        56 => "/".into(),
        57 => "Caps Lock".into(),
        58..=69 => format!("F{}", usage - 57),
        104..=115 => format!("F{}", usage - 91),
        75 => "Page Up".into(),
        76 => "Delete".into(),
        77 => "End".into(),
        78 => "Page Down".into(),
        79 => "Right".into(),
        80 => "Left".into(),
        81 => "Down".into(),
        82 => "Up".into(),
        224..=231 => [
            "Left Ctrl",
            "Left Shift",
            "Left Alt",
            "Left Super",
            "Right Ctrl",
            "Right Shift",
            "Right Alt",
            "Right Super",
        ][usize::from(usage - 224)]
        .into(),
        _ => format!("HID {usage}"),
    }
}
fn binding_name(map: &aula_protocol::Keymap, index: usize) -> String {
    let o = 8 + 4 * index;
    let r = &map.as_bytes()[o..o + 4];
    if r == [0, 0, 0, 0] {
        return "Unassigned layer slot".into();
    }
    if r[0] == 0 {
        let mut parts = vec![];
        for bit in 0..8u8 {
            if r[1] & (1 << bit) != 0 {
                parts.push(key_name(224 + u16::from(bit)))
            }
        }
        let usage = u16::from_be_bytes([r[2], r[3]]);
        if usage != 0 {
            parts.push(key_name(usage))
        }
        return parts.join(" + ");
    }
    if r[0] == 1 && r[2..] == [1, 0] {
        if let Some(name) = aula_protocol::mouse_action_name(u16::from(r[1])) {
            return name.into();
        }
    }
    if r[0] == 3 {
        "Onboard macro binding (in backup)".into()
    } else if r[0] == 2 {
        if r[1] == 0 {
            if let Some(name) = aula_protocol::media_action_name(u16::from_be_bytes([r[2], r[3]])) {
                return name.into();
            }
        }
        format!(
            "Consumer action {} (raw backup)",
            u16::from_be_bytes([r[2], r[3]])
        )
    } else {
        format!("Vendor-defined action {:02x?} (in backup)", r)
    }
}

fn profile_edits(profile: &Profile, layer: Layer) -> Result<Vec<(u16, KeyAction)>> {
    profile.validate()?;
    let (keyboard, media, mouse) = match layer {
        Layer::Base => (
            &profile.keymap,
            &profile.consumer_keymap,
            &profile.mouse_keymap,
        ),
        Layer::Fn => (
            &profile.fn_layer,
            &profile.consumer_fn_layer,
            &profile.mouse_fn_layer,
        ),
    };
    let mut edits = Vec::new();
    for (source, usage) in keyboard {
        edits.push((
            source.parse::<u16>().map_err(|e| e.to_string())?,
            KeyAction {
                page: 0,
                usage: *usage,
            },
        ));
    }
    for (source, usage) in media {
        edits.push((
            source.parse::<u16>().map_err(|e| e.to_string())?,
            KeyAction {
                page: 2,
                usage: *usage,
            },
        ));
    }
    for (source, usage) in mouse {
        edits.push((
            source.parse::<u16>().map_err(|e| e.to_string())?,
            KeyAction {
                page: 1,
                usage: *usage,
            },
        ));
    }
    Ok(edits)
}

type Assignments = std::collections::BTreeMap<String, u16>;
fn editable_assignments(
    map: &aula_protocol::Keymap,
) -> Result<(Assignments, Assignments, Assignments)> {
    let mut keyboard = Assignments::new();
    let mut media = Assignments::new();
    let mut mouse = Assignments::new();
    for usage in 1..=255u16 {
        if (58..=69).contains(&usage) {
            continue;
        }
        if let Ok(index) = matrix_index_for_usage(usage) {
            let action = map.get(index).map_err(|e| e.to_string())?;
            if ![0, 1, 2].contains(&action.page)
                || action.usage == 0
                || action.validate().is_err()
                || !map
                    .matches_action(index, action)
                    .map_err(|e| e.to_string())?
            {
                continue;
            }
            if action.page == 0 {
                keyboard.insert(usage.to_string(), action.usage);
            } else if action.page == 1 {
                mouse.insert(usage.to_string(), action.usage);
            } else if aula_protocol::media_action_name(action.usage).is_some() {
                media.insert(usage.to_string(), action.usage);
            }
        }
    }
    Ok((keyboard, media, mouse))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn media_profile_edits_keep_type_and_layer_separate() {
        let mut p = aula_service::decode_profile(r#"{"schema_version":1,"id":"test","name":"Test","lighting":{"effect":1,"brightness":3,"speed":3,"color":[1,2,3],"direction":0},"updated_at":"local"}"#).unwrap();
        p.keymap.insert("5".into(), 6);
        p.consumer_keymap.insert("4".into(), 0xcd);
        p.consumer_fn_layer.insert("4".into(), 0xe9);
        assert_eq!(
            profile_edits(&p, Layer::Base).unwrap(),
            vec![
                (5, KeyAction { page: 0, usage: 6 }),
                (
                    4,
                    KeyAction {
                        page: 2,
                        usage: 0xcd
                    }
                )
            ]
        );
        assert_eq!(
            profile_edits(&p, Layer::Fn).unwrap(),
            vec![(
                4,
                KeyAction {
                    page: 2,
                    usage: 0xe9
                }
            )]
        );
    }
    #[test]
    fn bank_reader_preserves_media_and_omits_uneditable_records() {
        let mut map = aula_protocol::Keymap::empty(Layer::Fn);
        map.set(
            matrix_index_for_usage(4).unwrap(),
            KeyAction {
                page: 2,
                usage: 0xe9,
            },
        )
        .unwrap();
        map.set(
            matrix_index_for_usage(5).unwrap(),
            KeyAction {
                page: 0,
                usage: 0xe0,
            },
        )
        .unwrap();
        let mut bytes = map.as_bytes().to_vec();
        for (usage, record) in [
            (58, [0, 0, 0, 58]),
            (6, [0, 1, 0, 7]),
            (7, [2, 0, 1, 0x83]),
            (8, [13, 0, 0, 0]),
            (9, [2, 1, 1, 0x83]),
            (10, [2, 0, 3, 0xcd]),
            (11, [0, 0, 1, 4]),
        ] {
            let o = 8 + 4 * matrix_index_for_usage(usage).unwrap();
            bytes[o..o + 4].copy_from_slice(&record);
        }
        let map = aula_protocol::Keymap::parse(&bytes).unwrap();
        let (keyboard, media, mouse) = editable_assignments(&map).unwrap();
        assert!(mouse.is_empty());
        assert_eq!(keyboard, Assignments::from([("5".into(), 0xe0)]));
        assert_eq!(
            media,
            Assignments::from([("4".into(), 0xe9), ("7".into(), 0x183)])
        );
        assert_eq!(
            binding_name(&map, matrix_index_for_usage(4).unwrap()),
            "Volume up"
        );
        assert_eq!(
            binding_name(&map, matrix_index_for_usage(7).unwrap()),
            "Media player"
        );
        assert!(binding_name(&map, matrix_index_for_usage(9).unwrap()).contains("raw backup"));
    }
    #[test]
    fn all_native_media_actions_survive_profile_edits_and_bank_projection() {
        let mut p = aula_service::decode_profile(r#"{"schema_version":1,"id":"test","name":"Test","lighting":{"effect":1,"brightness":3,"speed":3,"color":[1,2,3],"direction":0},"updated_at":"local"}"#).unwrap();
        for (i, usage) in aula_protocol::SUPPORTED_MEDIA_USAGES
            .into_iter()
            .enumerate()
        {
            p.consumer_keymap.insert((4 + i).to_string(), usage);
            p.consumer_fn_layer.insert((4 + i).to_string(), usage);
        }
        for layer in [Layer::Base, Layer::Fn] {
            let mut map = aula_protocol::Keymap::empty(layer);
            for (source, action) in profile_edits(&p, layer).unwrap() {
                map.set(matrix_index_for_usage(source).unwrap(), action)
                    .unwrap();
                assert_eq!(
                    binding_name(&map, matrix_index_for_usage(source).unwrap()),
                    aula_protocol::media_action_name(action.usage).unwrap()
                );
            }
            let (keyboard, media, mouse) = editable_assignments(&map).unwrap();
            assert!(mouse.is_empty());
            assert!(keyboard.is_empty());
            assert_eq!(media, p.consumer_keymap);
        }
    }
    #[test]
    fn five_mouse_assignments_roundtrip_without_decoding_repeat_forms() {
        let mut p = aula_service::decode_profile(r#"{"schema_version":1,"id":"test","name":"Test","lighting":{"effect":1,"brightness":3,"speed":3,"color":[1,2,3],"direction":0},"updated_at":"local"}"#).unwrap();
        for button in 1..=5 {
            p.mouse_keymap.insert((button + 3).to_string(), button);
            p.mouse_fn_layer.insert((button + 3).to_string(), button);
        }
        for layer in [Layer::Base, Layer::Fn] {
            let mut map = aula_protocol::Keymap::empty(layer);
            for (source, action) in profile_edits(&p, layer).unwrap() {
                let index = matrix_index_for_usage(source).unwrap();
                map.set(index, action).unwrap();
                assert_eq!(
                    binding_name(&map, index),
                    aula_protocol::mouse_action_name(action.usage).unwrap()
                );
            }
            let mut raw = map.as_bytes().to_vec();
            for (source, record) in [(9, [1, 1, 2, 50]), (10, [1, 1, 1, 10]), (11, [1, 6, 1, 0])] {
                let offset = 8 + 4 * matrix_index_for_usage(source).unwrap();
                raw[offset..offset + 4].copy_from_slice(&record);
            }
            let map = aula_protocol::Keymap::parse(&raw).unwrap();
            let (keyboard, media, mouse) = editable_assignments(&map).unwrap();
            assert!(keyboard.is_empty() && media.is_empty());
            assert_eq!(mouse, p.mouse_keymap);
            assert_eq!(map.as_bytes().as_slice(), raw);
        }
    }
}
