//! Prepared, single-use GUI lighting actions. This module never auto-applies.
use aula_hid::{transaction::LightingPlan, Snapshot};
use aula_protocol::{ConfigRegion, LightingSettings};
use aula_service::{atomic_write, new_id, Profile};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
pub type Result<T> = std::result::Result<T, String>;
pub struct PreparedLighting {
    pub plan: LightingPlan,
    pub backup_id: String,
    pub created: Instant,
    pub kind: String,
}
#[derive(Default)]
pub struct PendingLighting {
    plans: HashMap<String, PreparedLighting>,
}
impl PendingLighting {
    pub fn clear(&mut self) {
        self.plans.clear()
    }
    pub fn insert(&mut self, prepared: PreparedLighting) -> Result<String> {
        let mut bytes = [0; 32];
        getrandom::fill(&mut bytes).map_err(|e| e.to_string())?;
        let token = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
        self.clear();
        self.plans.insert(token.clone(), prepared);
        Ok(token)
    }
    pub fn consume(&mut self, token: &str) -> Result<PreparedLighting> {
        let p = self
            .plans
            .remove(token)
            .ok_or("This review is no longer valid. Prepare and review the operation again.")?;
        if p.created.elapsed() > Duration::from_secs(120) {
            return Err("Review expired. Read and review fresh settings.".into());
        }
        Ok(p)
    }
}
pub fn backup(root: &Path, snapshot: &Snapshot) -> Result<String> {
    snapshot.validate().map_err(|e| e.to_string())?;
    let dir = root.join("backups");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let id = new_id();
    let record = json!({"id":id,"created_at":id,"kind":"hardware","device":snapshot.device,"snapshot":snapshot});
    atomic_write(
        &dir.join(format!("{id}.json")),
        &serde_json::to_vec_pretty(&record).map_err(|e| e.to_string())?,
    )?;
    Ok(id)
}
pub fn verify_backup(root: &Path, prepared: &PreparedLighting) -> Result<()> {
    verify_snapshot_backup(root, &prepared.backup_id, &prepared.plan.before)
}
pub fn verify_snapshot_backup(root: &Path, backup_id: &str, before: &Snapshot) -> Result<()> {
    if !aula_service::valid_id(backup_id) {
        return Err("Invalid backup identifier".into());
    }
    let p = root.join("backups").join(format!("{}.json", backup_id));
    if std::fs::metadata(&p).map_err(|e| e.to_string())?.len() > 65536 {
        return Err("Invalid backup size".into());
    }
    let v: Value = serde_json::from_slice(&std::fs::read(p).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let saved: Snapshot =
        serde_json::from_value(v["snapshot"].clone()).map_err(|e| e.to_string())?;
    saved.validate().map_err(|e| e.to_string())?;
    if saved.config != before.config
        || saved.colors != before.colors
        || saved.device != before.device
    {
        return Err(
            "Saved backup differs from the reviewed device snapshot. Prepare again.".into(),
        );
    }
    Ok(())
}
pub fn prepare(profile: &Profile) -> Result<(LightingPlan, Value)> {
    profile.validate()?;
    if profile.lighting.effect == 21 {
        return Err("Custom per-key RGB uses a separate operation; use a preset in Lighting until its mapping workflow is available.".into());
    }
    let devices = aula_hid::discover().map_err(|e| e.to_string())?;
    let mut paths = devices
        .iter()
        .filter(|d| d.read_supported)
        .map(|d| d.path.clone())
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    if paths.len() != 1 {
        return Err("Connect exactly one supported wired F75 to prepare a lighting change.".into());
    }
    let before = aula_hid::read_snapshot(&paths[0]).map_err(|e| e.to_string())?;
    plan_from_snapshot(profile, before)
}
pub fn prepare_brightness(value: u8) -> Result<(LightingPlan, Value)> {
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
    let before = aula_hid::read_snapshot(&paths[0]).map_err(|e| e.to_string())?;
    let current = ConfigRegion::parse(&before.config).map_err(|e| e.to_string())?;
    let old = current.lighting().map_err(|e| e.to_string())?.brightness;
    let plan = LightingPlan::prepare_brightness(before, value).map_err(|e| e.to_string())?;
    if !plan.has_changes() {
        return Err("Brightness already matches; no write needed.".into());
    }
    let details = json!({"device":format!("{} · {} · USB 258a:010c",plan.before.device.product_name,plan.before.device.path),"summary":"Change only the reported preset brightness. Effect, mode flags, colors, speed, sleep and input mappings are preserved.","changes":[{"field":"Preset brightness","before":old,"after":value}],"warnings":["This operation changes exactly one configuration byte. A brightness change/readback/restoration test passed on wired ANSI F75; other revisions and persistence remain unverified.","The saved before-state must be retained for the separately planned restoration."]});
    Ok((plan, details))
}
pub fn prepare_settings(profile: &Profile) -> Result<(LightingPlan, Value)> {
    profile.validate()?;
    let seconds = profile
        .sleep_seconds
        .ok_or("Choose a sleep setting before preparing the write")?;
    let devices = aula_hid::discover().map_err(|e| e.to_string())?;
    let mut paths = devices
        .iter()
        .filter(|d| d.read_supported)
        .map(|d| d.path.clone())
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    if paths.len() != 1 {
        return Err("Connect exactly one supported wired F75.".into());
    }
    let before = aula_hid::read_snapshot(&paths[0]).map_err(|e| e.to_string())?;
    let old = ConfigRegion::parse(&before.config)
        .and_then(|c| c.sleep_seconds())
        .map_err(|e| e.to_string())?;
    let plan = LightingPlan::prepare_sleep(before, seconds).map_err(|e| e.to_string())?;
    if !plan.has_changes() {
        return Err("The sleep setting already matches the keyboard.".into());
    }
    let detail = json!({"device":format!("{} · {} · USB 258a:010c",plan.before.device.product_name,plan.before.device.path),"summary":"Change only the standard F75 sleep timer. Zero means sleep disabled.","changes":[{"field":"Sleep seconds","before":old,"after":seconds}],"warnings":["Protocol verified from official standard config fill and SleepUI/SleepHW resource table. Writes have not yet been locally tested.","Readback will be verified. Persistence after power loss is not tested."]});
    Ok((plan, detail))
}
pub fn prepare_restore(id: &str, root: &Path) -> Result<(LightingPlan, Value)> {
    if !aula_service::valid_id(id) {
        return Err("Invalid snapshot ID".into());
    }
    let path = root.join("backups").join(format!("{id}.json"));
    if std::fs::metadata(&path).map_err(|e| e.to_string())?.len() > 65536 {
        return Err("Snapshot exceeds size limit".into());
    }
    let v: Value = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if v["kind"] != "hardware" {
        return Err("Choose a physical snapshot for hardware restore.".into());
    }
    let saved: Snapshot =
        serde_json::from_value(v["snapshot"].clone()).map_err(|e| e.to_string())?;
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
    let before = aula_hid::read_snapshot(&paths[0]).map_err(|e| e.to_string())?;
    let plan = LightingPlan::prepare_restore(before, saved).map_err(|e| e.to_string())?;
    if !plan.has_changes() {
        return Err("Restorable settings already match this snapshot.".into());
    }
    let mut changes = vec![];
    for (i, (&a, &b)) in plan.before.config.iter().zip(&plan.config).enumerate() {
        if a != b {
            changes.push(json!({"field":format!("Known setting byte {i}"),"before":a,"after":b}))
        }
    }
    if plan.colors != plan.before.colors {
        changes.push(json!({"field":"Preset colors","before":"Current effect palette","after":format!("Saved colors from {id}")}))
    }
    let detail = json!({"device":format!("{} · {} · USB 258a:010c",plan.before.device.product_name,plan.before.device.path),"summary":"Restore supported preset lighting and sleep settings only. Key mappings, per-key banks, debounce and unknown bytes are preserved. A new before-restore backup will be saved.","changes":changes,"warnings":["Restore writes have not yet been locally hardware-tested. Readback is checked; power-cycle persistence is unknown.","This is a scoped settings restoration, not a factory reset or full memory restoration."]});
    Ok((plan, detail))
}
pub fn plan_from_snapshot(profile: &Profile, before: Snapshot) -> Result<(LightingPlan, Value)> {
    let old = ConfigRegion::parse(&before.config)
        .and_then(|c| c.lighting())
        .map_err(|e| {
            format!("Current configuration does not match the supported standard F75 layout: {e}")
        })?;
    let requested = LightingSettings {
        effect: profile.lighting.effect,
        brightness: profile.lighting.brightness,
        speed: profile.lighting.speed,
        colorful: profile.lighting.colorful,
    };
    let supports_color = aula_protocol::effect_supports_color(requested.effect);
    let color = if requested.colorful || !supports_color {
        None
    } else {
        Some(profile.lighting.color)
    };
    let plan = LightingPlan::prepare(before, requested, color).map_err(|e| e.to_string())?;
    let settings = ConfigRegion::parse(&plan.config)
        .and_then(|c| c.lighting())
        .map_err(|e| e.to_string())?;
    let mut changes = vec![];
    for (field, before, after) in [
        ("Effect", json!(old.effect), json!(settings.effect)),
        (
            "Brightness",
            json!(old.brightness),
            json!(settings.brightness),
        ),
        ("Speed", json!(old.speed), json!(settings.speed)),
        ("Multicolor", json!(old.colorful), json!(settings.colorful)),
    ] {
        if before != after {
            changes.push(json!({"field":field,"before":before,"after":after}))
        }
    }
    if plan.colors != plan.before.colors {
        changes.push(json!({"field":"Effect color","before":"Saved palette in backup","after":profile.lighting.color}))
    }
    if !plan.has_changes() {
        return Err(
            "These lighting settings already match the keyboard; no writes are needed.".into(),
        );
    }
    let detail = json!({"device":format!("{} · {} · USB 258a:010c",plan.before.device.product_name,plan.before.device.path),"summary":"Apply only the reviewed preset lighting settings. Keymaps, per-key colors and macros are not included.","changes":changes,"warnings":["Protocol verified from official code. Theme switching has user confirmation; complete preset/color/control behavior still requires physical validation.","Readback confirms current settings only. Persistence after power loss is not established.","Do not disconnect the keyboard or run another configurator during Apply."]});
    Ok((plan, detail))
}
pub fn commit(root: PathBuf, token: String, prepared: PreparedLighting) -> Result<Value> {
    verify_backup(&root, &prepared)?;
    if token.len() != 64 || !token.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid review token".into());
    }
    let dir = root.join("operations");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{token}.json"));
    let mut record = json!({"operation":prepared.kind,"state":"started","backup_id":prepared.backup_id,"device":prepared.plan.before.device,"hardware_persistence":"not_tested"});
    atomic_write(&path, &serde_json::to_vec_pretty(&record).unwrap())?;
    let result = aula_hid::commit_lighting(prepared.plan);
    match result {
        Ok(snapshot) => {
            record["state"] = json!("readback_verified");
            record["snapshot"] = json!(snapshot);
            atomic_write(&path,&serde_json::to_vec_pretty(&record).unwrap()).map_err(|e|format!("Hardware readback matched, but saving the operation result failed: {e}. Do not retry automatically."))?;
            Ok(
                json!({"message":"Configuration written and read back successfully. Power-cycle persistence has not been tested.","readback_verified":true,"persistence":"not_tested","backup_id":prepared.backup_id}),
            )
        }
        Err(e) => {
            record["state"] = json!("unconfirmed");
            record["error"] = json!(e.to_string());
            let _ = atomic_write(&path, &serde_json::to_vec_pretty(&record).unwrap());
            Err(format!(
                "{e}. Backup {} is retained; read the keyboard before trying again.",
                prepared.backup_id
            ))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use aula_hid::{demo_device, snapshot_from_transport, MockTransport};
    fn prepared() -> PreparedLighting {
        let mut m = MockTransport::default();
        let before = snapshot_from_transport(&mut m, demo_device(), "demo").unwrap();
        let plan = LightingPlan::prepare(
            before,
            LightingSettings {
                effect: 7,
                brightness: 5,
                speed: 2,
                colorful: true,
            },
            None,
        )
        .unwrap();
        PreparedLighting {
            plan,
            backup_id: new_id(),
            created: Instant::now(),
            kind: "lighting".into(),
        }
    }
    #[test]
    fn single_use_expiry_and_invalidation() {
        let mut p = PendingLighting::default();
        let id = p.insert(prepared()).unwrap();
        assert!(p.consume(&id).is_ok());
        assert!(p.consume(&id).is_err());
        let mut old = prepared();
        old.created = Instant::now() - Duration::from_secs(121);
        let id = p.insert(old).unwrap();
        assert!(p.consume(&id).is_err());
        let id = p.insert(prepared()).unwrap();
        p.clear();
        assert!(p.consume(&id).is_err());
    }
    #[test]
    fn backup_required_and_tamper_rejected() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/operation-tests")
            .join(new_id());
        let mut p = prepared();
        assert!(verify_backup(&root, &p).is_err());
        p.backup_id = backup(&root, &p.plan.before).unwrap();
        verify_backup(&root, &p).unwrap();
        let path = root.join("backups").join(format!("{}.json", p.backup_id));
        let mut v: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        v["snapshot"]["config"][0] = json!(99);
        atomic_write(&path, &serde_json::to_vec(&v).unwrap()).unwrap();
        assert!(verify_backup(&root, &p).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
