use serde_json::json;
fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str){
 Some("models")=>println!("{}",serde_json::to_string_pretty(aula_models::registry()?).unwrap()),
 Some("model-devices")=>println!("{}",serde_json::to_string_pretty(&aula_hid::discover_catalog_devices().map_err(|e|e.to_string())?).unwrap()),
 Some("identity")=>{let path=args.get(1).ok_or("Usage: aula identity /dev/hidrawN (bounded G5 identity read only)")?;let identity=aula_hid::read_model_identity(path).map_err(|e|e.to_string())?;let matches=aula_models::identity_candidates(identity)?;println!("{}",serde_json::to_string_pretty(&json!({"identity":identity,"candidates":matches.iter().map(|m|&m.id).collect::<Vec<_>>(),"configuration_changed":false})).unwrap())},
 Some("model-profile")=>{let path=args.get(1).ok_or("Usage: aula model-profile PROFILE.json (validate offline)")?;let p=aula_models::decode_draft(&std::fs::read_to_string(path).map_err(|e|e.to_string())?)?;println!("Valid local model profile for {}: {}. No HID packets sent.",p.model_id,p.name)},
 Some("devices")=>println!("{}",serde_json::to_string_pretty(&aula_hid::discover().map_err(|e|e.to_string())?).unwrap()),
 Some("diagnostics")=>println!("{}",serde_json::to_string_pretty(&json!({"version":env!("CARGO_PKG_VERSION"),"devices":aula_hid::discover().map_err(|e|e.to_string())?,"hardware_writes_enabled":false})).unwrap()),
 Some("read")=>{let path=args.get(1).ok_or("Usage: aula read /dev/hidrawN (verified wired interface only)")?;println!("{}",serde_json::to_string_pretty(&aula_hid::read_snapshot(path).map_err(|e|e.to_string())?).unwrap())},
 Some("read-layer")=>{let path=args.get(1).ok_or("Usage: aula read-layer /dev/hidrawN base|fn")?;let layer=match args.get(2).map(String::as_str){Some("base")=>aula_protocol::Layer::Base,Some("fn")=>aula_protocol::Layer::Fn,_=>return Err("Choose base or fn layer".into())};let(s,m)=aula_hid::read_keymap_snapshot(path,layer).map_err(|e|e.to_string())?;println!("{}",serde_json::to_string_pretty(&json!({"snapshot":s,"layer":layer,"keymap":m.as_bytes().to_vec()})).unwrap())},
 Some("read-per-key")=>{let path=args.get(1).ok_or("Usage: aula read-per-key /dev/hidrawN")?;let(s,c)=aula_hid::read_per_key_snapshot(path).map_err(|e|e.to_string())?;println!("{}",serde_json::to_string_pretty(&json!({"snapshot":s,"per_key":c.as_bytes().to_vec()})).unwrap())},
 Some("validate")=>{let path=args.get(1).ok_or("Usage: aula validate PROFILE.json")?;let p=aula_service::decode_profile(&std::fs::read_to_string(path).map_err(|e|e.to_string())?)?;println!("Valid profile: {} (schema {})",p.name,p.schema_version)},
 Some("profiles")=>println!("{}",serde_json::to_string_pretty(&aula_service::Store::new(aula_service::data_dir())?.list()?).unwrap()),
 _=>println!("AULA Studio diagnostics\n  aula models\n  aula model-devices\n  aula identity /dev/hidrawN\n  aula model-profile PROFILE.json\n  aula devices\n  aula diagnostics\n  aula read /dev/hidrawN\n  aula validate PROFILE.json\n  aula profiles\n\nNo hardware-write commands are included.")};
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("aula: {e}");
        std::process::exit(1)
    }
}
