#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use aula_service::{atomic_write, new_id, Preferences, Profile, Store};
use serde_json::{json, Value};
use std::sync::Mutex;
use tauri::State;
mod local_skin;
mod mapping_operations;
mod operations;
#[cfg(target_os = "linux")]
mod startup;
type Result<T> = std::result::Result<T, String>;
#[tauri::command]
fn preview_f75_macro_wire(
    local_macro: aula_service::Macro,
) -> Result<aula_service::macro_preview::WirePreview> {
    aula_service::macro_preview::preview(&local_macro)
}
#[tauri::command]
fn local_ui_assets() -> Value {
    local_skin::assets()
}
#[tauri::command]
fn window_action(action: String, window: tauri::WebviewWindow) -> Result<()> {
    match action.as_str() {
        "minimize" => window.minimize(),
        "close" => window.close(),
        "drag" => window.start_dragging(),
        _ => return Err("Unknown window action".into()),
    }
    .map_err(|e| e.to_string())
}
struct AppState {
    store: Mutex<Store>,
    demo: Mutex<Option<Profile>>,
    pending: Mutex<operations::PendingLighting>,
    mappings: Mutex<mapping_operations::PendingMappings>,
}
#[tauri::command]
fn get_preferences(s: State<AppState>) -> Result<Preferences> {
    s.store.lock().map_err(|e| e.to_string())?.preferences()
}
#[tauri::command]
fn save_preferences(preferences: Preferences, s: State<AppState>) -> Result<()> {
    s.pending.lock().map_err(|e| e.to_string())?.clear();
    s.mappings.lock().map_err(|e| e.to_string())?.clear();
    s.store
        .lock()
        .map_err(|e| e.to_string())?
        .save_preferences(&preferences)
}
#[tauri::command]
fn set_demo_mode(enabled: bool, s: State<AppState>) -> Result<()> {
    s.pending.lock().map_err(|e| e.to_string())?.clear();
    s.mappings.lock().map_err(|e| e.to_string())?.clear();
    let store = s.store.lock().map_err(|e| e.to_string())?;
    let mut p = store.preferences()?;
    p.demo_mode = enabled;
    store.save_preferences(&p)
}
#[tauri::command]
fn get_status(s: State<AppState>) -> Result<Value> {
    let demo = get_preferences(s)?.demo_mode;
    let devices = aula_hid::discover().map_err(|e| e.to_string())?;
    let readable = devices.iter().any(|d| d.read_supported);
    let connected = devices
        .iter()
        .any(|d| d.vendor_id == 0x258a && d.product_id == 0x010c);
    Ok(
        json!({"mode":if demo{"demo"}else{"hardware"},"connected":if demo{true}else{connected},"devices":devices,"capabilities":{"read_config":demo||readable,"write_lighting":demo||readable,"write_per_key":demo||readable,"write_keymap":demo||readable,"write_fn":demo||readable,"onboard_macros":false},"notice":if demo{"Demo mode: Apply changes only the in-memory simulation."}else{"Wired lighting changes require a backup and explicit review. A scoped brightness change/readback/restoration passed on wired ANSI F75; other writes and persistence require validation. Per-key and layer changes have separate scoped reviews. Onboard macros remain local."}}),
    )
}
fn model_store(s: &State<AppState>) -> Result<aula_models::Store> {
    Ok(aula_models::Store::new(
        s.store.lock().map_err(|e| e.to_string())?.root(),
    ))
}
#[tauri::command]
fn list_models() -> Result<Vec<aula_models::Model>> {
    Ok(aula_models::registry()?.to_vec())
}
#[tauri::command]
fn model_devices() -> Result<Value> {
    Ok(json!(
        aula_hid::discover_catalog_devices().map_err(|e| e.to_string())?
    ))
}
#[tauri::command]
fn new_model_profile(model_id: String, name: String) -> Result<aula_models::Draft> {
    aula_models::Draft::new(&model_id, &name)
}
#[tauri::command]
fn list_model_profiles(s: State<AppState>) -> Result<Vec<aula_models::Draft>> {
    model_store(&s)?.list()
}
#[tauri::command]
fn save_model_profile(profile: aula_models::Draft, s: State<AppState>) -> Result<()> {
    model_store(&s)?.save(&profile)
}
#[tauri::command]
fn delete_model_profile(id: String, s: State<AppState>) -> Result<()> {
    model_store(&s)?.delete(&id)
}
#[tauri::command]
fn import_model_profile(json: String, s: State<AppState>) -> Result<aula_models::Draft> {
    model_store(&s)?.import(&json)
}
#[tauri::command]
fn model_profile_json(profile: aula_models::Draft) -> Result<String> {
    profile.validate()?;
    let text = serde_json::to_string_pretty(&profile).map_err(|e| e.to_string())?;
    aula_models::decode_draft(&text)?;
    Ok(text)
}
#[tauri::command]
fn export_model_profile(id: String, s: State<AppState>) -> Result<String> {
    serde_json::to_string_pretty(&model_store(&s)?.get(&id)?).map_err(|e| e.to_string())
}

fn capture_store(s: &State<AppState>) -> Result<aula_models::g5_capture::CaptureStore> {
    Ok(aula_models::g5_capture::CaptureStore::new(
        s.store.lock().map_err(|e| e.to_string())?.root(),
    ))
}
#[tauri::command]
fn list_model_captures(
    model_id: String,
    s: State<AppState>,
) -> Result<Vec<aula_models::g5_capture::Capture>> {
    capture_store(&s)?.list(&model_id)
}
#[tauri::command]
async fn read_model_capture(
    model_id: String,
    path: String,
    s: State<'_, AppState>,
) -> Result<aula_models::g5_capture::Capture> {
    let root = {
        let store = s.store.lock().map_err(|e| e.to_string())?;
        if store.preferences()?.demo_mode {
            return Err("Exit F75 demo mode before physical reads, or use the explicit synthetic capture action".into());
        }
        store.root().to_path_buf()
    };
    tauri::async_runtime::spawn_blocking(move || {
        let capture = aula_hid::read_model_capture(&path, &model_id).map_err(|e| e.to_string())?;
        aula_models::g5_capture::CaptureStore::new(&root).save(&capture)?;
        Ok(capture)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
fn simulate_model_capture(
    model_id: String,
    s: State<AppState>,
) -> Result<aula_models::g5_capture::Capture> {
    let capture = aula_hid::model_read::simulate_capture(&model_id).map_err(|e| e.to_string())?;
    capture_store(&s)?.save(&capture)?;
    Ok(capture)
}
#[tauri::command]
fn export_model_capture(id: String, s: State<AppState>) -> Result<String> {
    serde_json::to_string_pretty(&capture_store(&s)?.get(&id)?).map_err(|e| e.to_string())
}
#[tauri::command]
fn model_capture_draft(id: String, name: String, s: State<AppState>) -> Result<aula_models::Draft> {
    capture_store(&s)?.get(&id)?.lighting_draft(&name)
}
#[tauri::command]
fn list_profiles(s: State<AppState>) -> Result<Vec<Profile>> {
    s.store.lock().map_err(|e| e.to_string())?.list()
}
#[tauri::command]
fn save_profile(profile: Profile, s: State<AppState>) -> Result<()> {
    s.store.lock().map_err(|e| e.to_string())?.save(&profile)
}
#[tauri::command]
fn delete_profile(id: String, s: State<AppState>) -> Result<()> {
    s.store.lock().map_err(|e| e.to_string())?.delete(&id)
}
#[tauri::command]
fn import_profile(json: String, s: State<AppState>) -> Result<Profile> {
    s.store.lock().map_err(|e| e.to_string())?.import(&json)
}
#[tauri::command]
fn export_profile(id: String, s: State<AppState>) -> Result<String> {
    serde_json::to_string_pretty(&s.store.lock().map_err(|e| e.to_string())?.get(&id)?)
        .map_err(|e| e.to_string())
}
fn do_read(demo: bool, mock: Option<Profile>) -> Result<Value> {
    if demo {
        return Ok(
            json!({"source":"demo","lighting":mock.as_ref().map(|p|&p.lighting),"profile":mock,"notice":"Simulated configuration, no hardware read."}),
        );
    }
    let devices = aula_hid::discover().map_err(|e| e.to_string())?;
    let d=devices.iter().find(|d|d.read_supported).ok_or("No supported, accessible wired configuration interface. Inspect Diagnostics and OS device permissions.")?;
    let snapshot = aula_hid::read_snapshot(&d.path).map_err(|e| e.to_string())?;
    let lighting = aula_protocol::ConfigRegion::parse(&snapshot.config)
        .and_then(|c| c.lighting())
        .ok();
    let color = lighting.as_ref().and_then(|l| {
        if aula_protocol::effect_supports_color(l.effect) {
            aula_protocol::ColorTable::parse(&snapshot.colors)
                .and_then(|p| p.effect_color(l.effect))
                .ok()
        } else {
            None
        }
    });
    let sleep = aula_protocol::ConfigRegion::parse(&snapshot.config)
        .and_then(|c| c.sleep_seconds())
        .ok();
    let mut value = serde_json::to_value(snapshot).map_err(|e| e.to_string())?;
    value["sleep_seconds"] = json!(sleep);
    value["lighting"] = serde_json::to_value(lighting).map_err(|e| e.to_string())?;
    if let Some(color) = color {
        value["lighting"]["color"] = json!(color);
    }
    Ok(value)
}
#[tauri::command]
async fn read_configuration(s: State<'_, AppState>) -> Result<Value> {
    let demo = get_preferences(s.clone())?.demo_mode;
    let mock = s.demo.lock().map_err(|e| e.to_string())?.clone();
    tauri::async_runtime::spawn_blocking(move || do_read(demo, mock))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn prepare_lighting(profile: Profile, s: State<'_, AppState>) -> Result<Value> {
    if get_preferences(s.clone())?.demo_mode {
        return Err("Exit demo mode before preparing a hardware operation.".into());
    }
    s.pending.lock().map_err(|e| e.to_string())?.clear();
    s.mappings.lock().map_err(|e| e.to_string())?.clear();
    let (plan, mut detail) =
        tauri::async_runtime::spawn_blocking(move || operations::prepare(&profile))
            .await
            .map_err(|e| e.to_string())??;
    // No mutex is held across USB reads. Recheck the mode before publishing review.
    if get_preferences(s.clone())?.demo_mode {
        return Err("Mode changed during preparation. Prepare again.".into());
    }
    let root = s
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .root()
        .to_path_buf();
    let backup_id = operations::backup(&root, &plan.before)?;
    let token =
        s.pending
            .lock()
            .map_err(|e| e.to_string())?
            .insert(operations::PreparedLighting {
                plan,
                backup_id: backup_id.clone(),
                created: std::time::Instant::now(),
                kind: "lighting".into(),
            })?;
    detail["token"] = json!(token);
    detail["backup_id"] = json!(backup_id);
    Ok(detail)
}
#[tauri::command]
async fn prepare_settings(profile: Profile, s: State<'_, AppState>) -> Result<Value> {
    if get_preferences(s.clone())?.demo_mode {
        return Err("Exit demo mode before preparing a hardware operation.".into());
    }
    s.pending.lock().map_err(|e| e.to_string())?.clear();
    s.mappings.lock().map_err(|e| e.to_string())?.clear();
    let (plan, mut detail) =
        tauri::async_runtime::spawn_blocking(move || operations::prepare_settings(&profile))
            .await
            .map_err(|e| e.to_string())??;
    // No mutex is held across USB reads. Recheck the mode before publishing review.
    if get_preferences(s.clone())?.demo_mode {
        return Err("Mode changed during preparation. Prepare again.".into());
    }
    let root = s
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .root()
        .to_path_buf();
    let backup_id = operations::backup(&root, &plan.before)?;
    let token =
        s.pending
            .lock()
            .map_err(|e| e.to_string())?
            .insert(operations::PreparedLighting {
                plan,
                backup_id: backup_id.clone(),
                created: std::time::Instant::now(),
                kind: "settings".into(),
            })?;
    detail["token"] = json!(token);
    detail["backup_id"] = json!(backup_id);
    Ok(detail)
}
#[tauri::command]
async fn prepare_brightness(value: u8, s: State<'_, AppState>) -> Result<Value> {
    if get_preferences(s.clone())?.demo_mode {
        return Err("Exit demo mode before preparing a hardware operation.".into());
    }
    s.pending.lock().map_err(|e| e.to_string())?.clear();
    s.mappings.lock().map_err(|e| e.to_string())?.clear();
    let (plan, mut detail) =
        tauri::async_runtime::spawn_blocking(move || operations::prepare_brightness(value))
            .await
            .map_err(|e| e.to_string())??;
    // No mutex is held across USB reads. Recheck the mode before publishing review.
    if get_preferences(s.clone())?.demo_mode {
        return Err("Mode changed during preparation. Prepare again.".into());
    }
    let root = s
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .root()
        .to_path_buf();
    let backup_id = operations::backup(&root, &plan.before)?;
    let token =
        s.pending
            .lock()
            .map_err(|e| e.to_string())?
            .insert(operations::PreparedLighting {
                plan,
                backup_id: backup_id.clone(),
                created: std::time::Instant::now(),
                kind: "brightness".into(),
            })?;
    detail["token"] = json!(token);
    detail["backup_id"] = json!(backup_id);
    Ok(detail)
}
#[tauri::command]
async fn prepare_restore(id: String, s: State<'_, AppState>) -> Result<Value> {
    if get_preferences(s.clone())?.demo_mode {
        return Err("Use demo restore for a simulated snapshot.".into());
    }
    s.pending.lock().map_err(|e| e.to_string())?.clear();
    s.mappings.lock().map_err(|e| e.to_string())?.clear();
    let root = s
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .root()
        .to_path_buf();
    let work_root = root.clone();
    let (plan, mut detail) =
        tauri::async_runtime::spawn_blocking(move || operations::prepare_restore(&id, &work_root))
            .await
            .map_err(|e| e.to_string())??;
    if get_preferences(s.clone())?.demo_mode {
        return Err("Mode changed. Prepare again.".into());
    }
    let backup_id = operations::backup(&root, &plan.before)?;
    let token =
        s.pending
            .lock()
            .map_err(|e| e.to_string())?
            .insert(operations::PreparedLighting {
                plan,
                backup_id: backup_id.clone(),
                created: std::time::Instant::now(),
                kind: "restore".into(),
            })?;
    detail["token"] = json!(token);
    detail["backup_id"] = json!(backup_id);
    Ok(detail)
}
#[tauri::command]
async fn commit_lighting(token: String, s: State<'_, AppState>) -> Result<Value> {
    let prepared = s
        .pending
        .lock()
        .map_err(|e| e.to_string())?
        .consume(&token)?;
    if get_preferences(s.clone())?.demo_mode {
        return Err("Mode changed. Hardware review discarded.".into());
    }
    let root = s
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .root()
        .to_path_buf();
    tauri::async_runtime::spawn_blocking(move || operations::commit(root, token, prepared))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn prepare_mapping(profile: Profile, kind: String, s: State<'_, AppState>) -> Result<Value> {
    if get_preferences(s.clone())?.demo_mode {
        return Err("Use demo Apply for simulation.".into());
    }
    s.pending.lock().map_err(|e| e.to_string())?.clear();
    s.mappings.lock().map_err(|e| e.to_string())?.clear();
    let root = s
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .root()
        .to_path_buf();
    let (p, mut detail) = tauri::async_runtime::spawn_blocking(move || {
        mapping_operations::prepare(profile, kind, root)
    })
    .await
    .map_err(|e| e.to_string())??;
    if get_preferences(s.clone())?.demo_mode {
        return Err("Mode changed. Prepare again.".into());
    }
    let token = s.mappings.lock().map_err(|e| e.to_string())?.insert(p)?;
    detail["token"] = json!(token);
    Ok(detail)
}
#[tauri::command]
async fn prepare_mapping_restore(id: String, s: State<'_, AppState>) -> Result<Value> {
    if get_preferences(s.clone())?.demo_mode {
        return Err("Use demo Apply for simulation.".into());
    }
    s.pending.lock().map_err(|e| e.to_string())?.clear();
    s.mappings.lock().map_err(|e| e.to_string())?.clear();
    let root = s
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .root()
        .to_path_buf();
    let (p, mut detail) =
        tauri::async_runtime::spawn_blocking(move || mapping_operations::prepare_restore(id, root))
            .await
            .map_err(|e| e.to_string())??;
    if get_preferences(s.clone())?.demo_mode {
        return Err("Mode changed. Prepare again.".into());
    }
    let token = s.mappings.lock().map_err(|e| e.to_string())?.insert(p)?;
    detail["token"] = json!(token);
    Ok(detail)
}
#[tauri::command]
async fn commit_mapping(token: String, s: State<'_, AppState>) -> Result<Value> {
    let p = s
        .mappings
        .lock()
        .map_err(|e| e.to_string())?
        .consume(&token)?;
    if get_preferences(s.clone())?.demo_mode {
        return Err("Mode changed. Review discarded.".into());
    }
    let root = s
        .store
        .lock()
        .map_err(|e| e.to_string())?
        .root()
        .to_path_buf();
    tauri::async_runtime::spawn_blocking(move || mapping_operations::commit(root, token, p))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn read_mapping(kind: String, s: State<'_, AppState>) -> Result<Value> {
    if get_preferences(s)?.demo_mode {
        return Err(
            "Read bank requires a physical keyboard. Demo Apply uses its local simulation.".into(),
        );
    }
    tauri::async_runtime::spawn_blocking(move || mapping_operations::read(kind))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
fn apply_profile(profile: Profile, s: State<AppState>) -> Result<Value> {
    profile.validate()?;
    if !get_preferences(s.clone())?.demo_mode {
        return Err("Hardware operations require Prepare lighting followed by its explicit reviewed write action.".into());
    }
    *s.demo.lock().map_err(|e| e.to_string())? = Some(profile);
    Ok(json!({"message":"Applied to demo simulation only. No hardware was changed."}))
}
#[tauri::command]
async fn backup_configuration(s: State<'_, AppState>) -> Result<Value> {
    let value = read_configuration(s.clone()).await?;
    let store = s.store.lock().map_err(|e| e.to_string())?;
    let dir = store.root().join("backups");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let id = new_id();
    let kind = if value.get("source") == Some(&json!("demo")) {
        "demo"
    } else {
        "hardware"
    };
    let record =
        json!({"id":id,"created_at":id,"kind":kind,"device":value.get("device"),"snapshot":value});
    atomic_write(
        &dir.join(format!("{id}.json")),
        &serde_json::to_vec_pretty(&record).unwrap(),
    )?;
    Ok(
        json!({"id":id,"message":"Snapshot backed up locally. Supported preset lighting and sleep settings can be restored through a separate review."}),
    )
}
#[tauri::command]
fn list_backups(s: State<AppState>) -> Result<Vec<Value>> {
    let store = s.store.lock().map_err(|e| e.to_string())?;
    let dir = store.root().join("backups");
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = vec![];
    for e in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let path = e.map_err(|e| e.to_string())?.path();
        if path.extension().is_some_and(|e| e == "json") {
            let mut v: Value =
                serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            v.as_object_mut()
                .ok_or("Malformed backup")?
                .remove("snapshot");
            out.push(v)
        }
    }
    Ok(out)
}
#[tauri::command]
fn restore_backup(id: String, s: State<AppState>) -> Result<Value> {
    if !aula_service::valid_id(&id) {
        return Err("Invalid backup ID".into());
    }
    if !get_preferences(s.clone())?.demo_mode {
        return Err("Physical restoration disabled pending hardware validation.".into());
    }
    let store = s.store.lock().map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_slice(
        &std::fs::read(store.root().join("backups").join(format!("{id}.json")))
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if v["kind"] != "demo" {
        return Err("A hardware snapshot cannot be restored into a profile simulation.".into());
    }
    let profile: Option<Profile> =
        serde_json::from_value(v["snapshot"]["profile"].clone()).map_err(|e| e.to_string())?;
    *s.demo.lock().map_err(|e| e.to_string())? = profile;
    Ok(json!({"message":"Demo snapshot restored. No hardware changed."}))
}
#[tauri::command]
fn diagnostics(s: State<AppState>) -> Result<Value> {
    let status = get_status(s)?;
    let writes_enabled = std::env::var_os("AULA_DISABLE_HARDWARE_WRITES").is_none();
    Ok(
        json!({"application":"AULA Studio","version":env!("CARGO_PKG_VERSION"),"platform":std::env::consts::OS,"status":status,"safety":{"hardware_writes_enabled":writes_enabled,"hardware_write_policy":"single-use reviewed settings/mapping transactions only; no automatic writes","firmware_operations":false},"protocol":"ID 0x06; wired 520-byte feature report; bounded reads and backed-up lighting transactions","physical_validation":"See docs/TESTING.md for this release's actual read evidence."}),
    )
}
fn main() {
    #[cfg(target_os = "linux")]
    if let Err(error) = startup::initialize() {
        eprintln!("{error}");
        std::process::exit(1);
    }
    let store = match Store::for_app() {
        Ok(store) => store,
        Err(error) => {
            eprintln!("AULA Studio could not open its application data directory: {error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = tauri::Builder::default()
        .manage(AppState {
            store: Mutex::new(store),
            demo: Mutex::new(None),
            pending: Mutex::new(operations::PendingLighting::default()),
            mappings: Mutex::new(mapping_operations::PendingMappings::default()),
        })
        .invoke_handler(tauri::generate_handler![
            preview_f75_macro_wire,
            local_ui_assets,
            window_action,
            get_preferences,
            save_preferences,
            set_demo_mode,
            get_status,
            list_models,
            model_devices,
            new_model_profile,
            list_model_profiles,
            list_model_captures,
            read_model_capture,
            simulate_model_capture,
            export_model_capture,
            model_capture_draft,
            save_model_profile,
            delete_model_profile,
            import_model_profile,
            export_model_profile,
            model_profile_json,
            list_profiles,
            save_profile,
            delete_profile,
            import_profile,
            export_profile,
            read_configuration,
            apply_profile,
            prepare_lighting,
            prepare_settings,
            prepare_brightness,
            prepare_restore,
            commit_lighting,
            prepare_mapping,
            prepare_mapping_restore,
            commit_mapping,
            read_mapping,
            backup_configuration,
            list_backups,
            restore_backup,
            diagnostics
        ])
        .run(tauri::generate_context!())
    {
        eprintln!("AULA Studio could not launch: {error}");
        std::process::exit(1);
    }
}
