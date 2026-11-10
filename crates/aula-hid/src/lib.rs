//! Conservative native HID access. Discovery never sends a device report.
//! Physical writes are restricted to reviewed, typed configuration transactions.
use aula_protocol::{ColorTable, ConfigRegion, FeatureReport, ProtocolError, Region, REPORT_SIZE};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{mpsc, Mutex},
    time::{Duration, Instant},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HidError {
    #[error("HID access failed: {0}")]
    Device(String),
    #[error("This interface is not approved for configuration reads: {0}")]
    Unsupported(String),
    #[error("A device operation is already in progress")]
    Busy,
    #[error("Device operation timed out; any pending kernel request must finish before retry")]
    Timeout,
    #[error("Device disconnected")]
    Disconnected,
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
}
pub type Result<T> = std::result::Result<T, HidError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceSummary {
    pub path: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface_number: i32,
    pub product_name: String,
    pub manufacturer: String,
    pub usage_page: u16,
    pub usage: u16,
    pub connection: String,
    pub read_supported: bool,
    pub write_supported: bool,
    pub feature_report_bytes: Option<usize>,
    pub reason: String,
    #[serde(default)]
    pub connection_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub schema_version: u32,
    pub device: DeviceSummary,
    pub config: Vec<u8>,
    pub colors: Vec<u8>,
    pub source: String,
}
impl Snapshot {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            return Err(HidError::Unsupported("snapshot schema version".into()));
        }
        ConfigRegion::parse(&self.config)?;
        ColorTable::parse(&self.colors)?;
        Ok(())
    }
}
fn known(vid: u16, pid: u16) -> bool {
    matches!(
        (vid, pid),
        (0x258a, 0x010c) | (0x3554, 0xfa09) | (0x258a, 0x010d)
    )
}

/// Read-only metadata/descriptor discovery. No HID report is sent.
/// Native Windows/macOS read OS descriptors through a non-seizing vendor handle.
pub fn discover() -> Result<Vec<DeviceSummary>> {
    let api = hidapi::HidApi::new().map_err(|e| HidError::Device(e.to_string()))?;
    let mut result = Vec::new();
    for info in api
        .device_list()
        .filter(|d| known(d.vendor_id(), d.product_id()))
    {
        let path = info.path().to_string_lossy().into_owned();
        let feature_report_bytes = descriptor_feature_length(&path).ok().flatten();
        let wired = info.vendor_id() == 0x258a && info.product_id() == 0x010c;
        let supported = wired
            && info.interface_number() == 1
            && info.usage_page() == 0xff00
            && info.usage() == 1
            && feature_report_bytes == Some(REPORT_SIZE)
            && connection_identity(&path).is_some();
        let reason = if supported {
            "G5 family descriptor matches report06, shared by several AULA models. Exact original-F75 identity is checked before bounded configuration reads or reviewed writes; a USB ID alone is not model identification."
        } else if !wired {
            "Receiver detected. Wireless configuration framing has not been verified for this hardware."
        } else {
            "Configuration requires the wired vendor interface1, report06/520B descriptor and an OS connection epoch."
        };
        result.push(DeviceSummary {
            path: path.clone(),
            vendor_id: info.vendor_id(),
            product_id: info.product_id(),
            interface_number: info.interface_number(),
            product_name: info
                .product_string()
                .unwrap_or("AULA-compatible HID interface")
                .into(),
            manufacturer: info.manufacturer_string().unwrap_or("").into(),
            usage_page: info.usage_page(),
            usage: info.usage(),
            connection: if wired { "usb" } else { "wireless" }.into(),
            read_supported: supported,
            write_supported: supported,
            feature_report_bytes,
            reason: reason.into(),
            connection_id: connection_identity(&path).unwrap_or_default(),
        });
    }
    result.sort_by(|a, b| a.path.cmp(&b.path).then(a.usage_page.cmp(&b.usage_page)));
    result.dedup_by(|a, b| a.path == b.path && a.usage_page == b.usage_page && a.usage == b.usage);
    Ok(result)
}
#[cfg(target_os = "linux")]
fn descriptor_feature_length(path: &str) -> std::result::Result<Option<usize>, String> {
    let name = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| s.starts_with("hidraw") && s[6..].chars().all(|c| c.is_ascii_digit()))
        .ok_or("Invalid hidraw path")?;
    let bytes = std::fs::read(
        Path::new("/sys/class/hidraw")
            .join(name)
            .join("device/report_descriptor"),
    )
    .map_err(|e| e.to_string())?;
    Ok(feature_report_lengths(&bytes)?.get(&6).copied())
}

/// Parse HID short items and global push/pop; sum every Feature main item's bits.
/// The returned length includes the report-ID byte. Long items are bounded/skipped.
pub fn feature_report_lengths(bytes: &[u8]) -> std::result::Result<BTreeMap<u8, usize>, String> {
    if bytes.len() > 4096 {
        return Err("Descriptor exceeds 4096-byte safety bound".into());
    }
    #[derive(Clone, Copy, Default)]
    struct Globals {
        size: u32,
        count: u32,
        id: u8,
    }
    let mut g = Globals::default();
    let mut stack = Vec::new();
    let mut bits: BTreeMap<u8, usize> = BTreeMap::new();
    let mut i = 0;
    while i < bytes.len() {
        let p = bytes[i];
        i += 1;
        if p == 0xfe {
            if i + 2 > bytes.len() {
                return Err("Truncated long descriptor item".into());
            }
            let n = bytes[i] as usize;
            i += 2;
            if i + n > bytes.len() {
                return Err("Truncated long descriptor payload".into());
            }
            i += n;
            continue;
        }
        let n = match p & 3 {
            3 => 4,
            n => n as usize,
        };
        if i + n > bytes.len() {
            return Err("Truncated descriptor item".into());
        }
        let mut val = 0u32;
        for j in 0..n {
            val |= (bytes[i + j] as u32) << (8 * j)
        }
        i += n;
        let typ = (p >> 2) & 3;
        let tag = p >> 4;
        if typ == 1 {
            match tag {
                7 => g.size = val,
                8 => {
                    if val == 0 || val > 255 {
                        return Err("Invalid report ID".into());
                    }
                    g.id = val as u8
                }
                9 => g.count = val,
                10 => {
                    if stack.len() >= 16 {
                        return Err("Descriptor global stack too deep".into());
                    }
                    stack.push(g)
                }
                11 => g = stack.pop().ok_or("Descriptor global stack underflow")?,
                _ => {}
            }
        } else if typ == 0 && tag == 11 {
            let n = (g.size as usize)
                .checked_mul(g.count as usize)
                .ok_or("Feature size overflow")?;
            let total = bits.entry(g.id).or_default();
            *total = total.checked_add(n).ok_or("Feature size overflow")?;
            if *total > 65535 * 8 {
                return Err("Feature report exceeds HID size bound".into());
            }
        }
    }
    if !stack.is_empty() {
        return Err("Unclosed descriptor global stack".into());
    }
    Ok(bits
        .into_iter()
        .map(|(id, n)| (id, n.div_ceil(8) + usize::from(id != 0)))
        .collect())
}

/// Mockable bounded transaction. No arbitrary packets or memory addresses.
pub trait Transport {
    fn read_region(&mut self, region: Region) -> Result<Vec<u8>>;
}
pub fn read_region(transport: &mut impl Transport, region: Region) -> Result<Vec<u8>> {
    transport.read_region(region)
}
pub fn snapshot_from_transport(
    transport: &mut impl Transport,
    device: DeviceSummary,
    source: &str,
) -> Result<Snapshot> {
    let config = transport.read_region(Region::Config)?;
    let colors = transport.read_region(Region::Colors)?;
    let snapshot = Snapshot {
        schema_version: 1,
        device,
        config,
        colors,
        source: source.into(),
    };
    snapshot.validate()?;
    Ok(snapshot)
}

/// Synthetic in-memory device; it never opens physical hardware.
pub struct MockTransport {
    config: Vec<u8>,
    colors: Vec<u8>,
    connected: bool,
    fail_next: Option<HidError>,
}
impl Default for MockTransport {
    fn default() -> Self {
        Self {
            config: ConfigRegion::demo().as_bytes().to_vec(),
            colors: ColorTable::black().as_bytes().to_vec(),
            connected: true,
            fail_next: None,
        }
    }
}
impl MockTransport {
    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
    }
    pub fn fail_next(&mut self, error: HidError) {
        self.fail_next = Some(error);
    }
    pub fn apply_demo(&mut self, region: Region, data: &[u8]) -> Result<()> {
        self.check()?;
        let packet = FeatureReport::write(region, data)?;
        let decoded = FeatureReport::decode_response(region, packet.as_bytes())?;
        match region {
            Region::Config => self.config = decoded,
            Region::Colors => self.colors = decoded,
        }
        Ok(())
    }
    fn check(&mut self) -> Result<()> {
        if !self.connected {
            return Err(HidError::Disconnected);
        }
        if let Some(e) = self.fail_next.take() {
            return Err(e);
        }
        Ok(())
    }
}
impl Transport for MockTransport {
    fn read_region(&mut self, region: Region) -> Result<Vec<u8>> {
        self.check()?;
        let data = match region {
            Region::Config => &self.config,
            Region::Colors => &self.colors,
        };
        Ok(data.clone())
    }
}
pub fn demo_device() -> DeviceSummary {
    DeviceSummary {
        path: "mock://f75".into(),
        vendor_id: 0x258a,
        product_id: 0x010c,
        interface_number: 1,
        product_name: "AULA F75 · simulated".into(),
        manufacturer: "Demo".into(),
        usage_page: 0xff00,
        usage: 1,
        connection: "demo".into(),
        read_supported: true,
        write_supported: false,
        feature_report_bytes: Some(520),
        reason: "In-memory simulation. Never sent to hardware.".into(),
        connection_id: "mock-session".into(),
    }
}

pub mod mapping_transaction;
pub mod model_read;
pub mod transaction;

struct HidTransport {
    device: hidapi::HidDevice,
    deadline: Instant,
}
impl HidTransport {
    fn read_model_identity(&mut self) -> Result<[u8; 6]> {
        let reply = self.read_report(&aula_models::g5::identity_request())?;
        Ok(aula_models::g5::decode_identity(&reply)?)
    }
    fn check_deadline(&self) -> Result<()> {
        if Instant::now() >= self.deadline {
            Err(HidError::Timeout)
        } else {
            Ok(())
        }
    }
}
impl Transport for HidTransport {
    fn read_region(&mut self, region: Region) -> Result<Vec<u8>> {
        self.check_deadline()?;
        let request = FeatureReport::read(region);
        self.device
            .send_feature_report(request.as_bytes())
            .map_err(|e| HidError::Device(e.to_string()))?;
        std::thread::sleep(Duration::from_millis(20));
        self.check_deadline()?;
        let mut response = [0; REPORT_SIZE];
        response[0] = 6;
        let n = self
            .device
            .get_feature_report(&mut response)
            .map_err(|e| HidError::Device(e.to_string()))?;
        Ok(FeatureReport::decode_response(region, &response[..n])?)
    }
}
impl transaction::ConfigTransport for HidTransport {
    fn write_region(&mut self, region: Region, data: &[u8]) -> Result<()> {
        self.check_deadline()?;
        let packet = FeatureReport::write(region, data)?;
        self.device
            .send_feature_report(packet.as_bytes())
            .map_err(|e| {
                HidError::Device(format!(
                    "Configuration write failed; keyboard state may have changed: {e}"
                ))
            })?;
        std::thread::sleep(Duration::from_millis(70));
        Ok(())
    }
}
impl HidTransport {
    fn read_report(&self, request: &[u8; REPORT_SIZE]) -> Result<Vec<u8>> {
        self.check_deadline()?;
        self.device
            .send_feature_report(request)
            .map_err(|e| HidError::Device(e.to_string()))?;
        std::thread::sleep(Duration::from_millis(20));
        self.check_deadline()?;
        let mut response = [0; REPORT_SIZE];
        response[0] = 6;
        let n = self
            .device
            .get_feature_report(&mut response)
            .map_err(|e| HidError::Device(e.to_string()))?;
        Ok(response[..n].to_vec())
    }
    fn write_report(&self, report: &[u8; REPORT_SIZE]) -> Result<()> {
        self.check_deadline()?;
        self.device
            .send_feature_report(report)
            .map_err(|e| HidError::Device(format!("Write outcome may be uncertain: {e}")))?;
        std::thread::sleep(Duration::from_millis(70));
        Ok(())
    }
}
impl mapping_transaction::ExtendedTransport for HidTransport {
    fn read_keymap(&mut self, layer: aula_protocol::Layer) -> Result<aula_protocol::Keymap> {
        let bytes = self.read_report(aula_protocol::Keymap::read_request(layer).as_bytes())?;
        Ok(aula_protocol::Keymap::decode_response(layer, &bytes)?)
    }
    fn write_keymap(&mut self, map: &aula_protocol::Keymap) -> Result<()> {
        self.write_report(map.as_bytes())
    }
    fn read_per_key(&mut self) -> Result<aula_protocol::PerKeyColors> {
        let bytes = self.read_report(aula_protocol::PerKeyColors::read_request().as_bytes())?;
        Ok(aula_protocol::PerKeyColors::decode_response(&bytes)?)
    }
    fn write_per_key(&mut self, colors: &aula_protocol::PerKeyColors) -> Result<()> {
        self.write_report(&colors.offline_report())
    }
}
fn selected_device(path: &str) -> Result<DeviceSummary> {
    discover()?
        .into_iter()
        .find(|d| d.path == path && d.read_supported)
        .ok_or_else(|| {
            HidError::Unsupported("device missing or descriptor/interface mismatch".into())
        })
}
pub fn read_keymap_snapshot(
    path: &str,
    layer: aula_protocol::Layer,
) -> Result<(Snapshot, aula_protocol::Keymap)> {
    use mapping_transaction::ExtendedTransport;
    let d = selected_device(path)?;
    let info = d.clone();
    run_device(d, move |t| {
        let snapshot = snapshot_from_transport(t, info, "physical_read")?;
        let map = t.read_keymap(layer)?;
        let base = if layer == aula_protocol::Layer::Base {
            map.clone()
        } else {
            t.read_keymap(aula_protocol::Layer::Base)?
        };
        base.validate_ansi_fn_position()?;
        Ok((snapshot, map))
    })
}
pub fn read_per_key_snapshot(path: &str) -> Result<(Snapshot, aula_protocol::PerKeyColors)> {
    use mapping_transaction::ExtendedTransport;
    let d = selected_device(path)?;
    let info = d.clone();
    run_device(d, move |t| {
        let snapshot = snapshot_from_transport(t, info, "physical_read")?;
        t.read_keymap(aula_protocol::Layer::Base)?
            .validate_ansi_fn_position()?;
        let colors = t.read_per_key()?;
        Ok((snapshot, colors))
    })
}
/// Only an explicit reviewed GUI action may enter this physical write path.
pub fn commit_mapping(
    plan: mapping_transaction::MappingPlan,
) -> Result<mapping_transaction::MappingResult> {
    check_write_policy()?;
    plan.validate()?;
    let d = plan.before.device.clone();
    if d.vendor_id != 0x258a
        || d.product_id != 0x010c
        || d.interface_number != 1
        || !d.read_supported
    {
        return Err(HidError::Unsupported(
            "Not a supported wired mapping plan".into(),
        ));
    }
    run_device(d, move |t| {
        use mapping_transaction::ExtendedTransport;
        t.read_keymap(aula_protocol::Layer::Base)?
            .validate_ansi_fn_position()?;
        mapping_transaction::execute_mapping(t, &plan)
    })
}
#[cfg(target_os = "linux")]
fn connection_identity(path: &str) -> Option<String> {
    let name = Path::new(path).file_name()?;
    std::fs::canonicalize(Path::new("/sys/class/hidraw").join(name).join("device"))
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}
#[cfg(target_os = "macos")]
fn connection_identity(path: &str) -> Option<String> {
    // hidapi obtains this attach-specific ID from IORegistryEntryGetRegistryEntryID.
    let id = path.strip_prefix("DevSrvsID:")?.parse::<u64>().ok()?;
    (id != 0).then(|| format!("ioreg:{id}"))
}
#[cfg(target_os = "windows")]
fn connection_identity(path: &str) -> Option<String> {
    windows_connection::identity(path)
}
#[cfg(target_os = "windows")]
mod windows_connection;

#[cfg(not(target_os = "linux"))]
fn descriptor_feature_length(path: &str) -> std::result::Result<Option<usize>, String> {
    let api = hidapi::HidApi::new().map_err(|e| e.to_string())?;
    let info = api
        .device_list()
        .find(|d| {
            d.path().to_bytes() == path.as_bytes()
                && d.vendor_id() == 0x258a
                && d.product_id() == 0x010c
                && d.interface_number() == 1
                && d.usage_page() == 0xff00
                && d.usage() == 1
        })
        .ok_or("Not the wired vendor configuration collection")?;
    // Only reads the OS descriptor. macos-shared-device prevents keyboard seizure.
    let device = info.open_device(&api).map_err(|e| e.to_string())?;
    let mut bytes = [0u8; 4097];
    let count = device
        .get_report_descriptor(&mut bytes)
        .map_err(|e| e.to_string())?;
    if count == 0 || count > 4096 {
        return Err("Truncated or oversized HID descriptor".into());
    }
    Ok(feature_report_lengths(&bytes[..count])?.get(&6).copied())
}
static OPERATION: Mutex<()> = Mutex::new(());
fn process_lock() -> Result<std::fs::File> {
    #[cfg(not(target_os = "linux"))]
    {
        lock_in_dir(&aula_service::cache_dir())
    }
    #[cfg(target_os = "linux")]
    {
        let base = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| {
                std::env::var_os("XDG_CACHE_HOME")
                    .map(std::path::PathBuf::from)
                    .filter(|p| p.is_absolute())
            })
            .or_else(|| {
                std::env::var_os("HOME").map(|p| std::path::PathBuf::from(p).join(".cache"))
            })
            .ok_or_else(|| {
                HidError::Device("No user runtime/cache directory for HID lock".into())
            })?;
        lock_in_dir(&base.join("aula-studio"))
    }
}
fn lock_in_dir(dir: &Path) -> Result<std::fs::File> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder
        .create(dir)
        .map_err(|e| HidError::Device(e.to_string()))?;
    let mut options = std::fs::OpenOptions::new();
    options.create(true).truncate(false).read(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let f = options
        .open(dir.join("hid.lock"))
        .map_err(|e| HidError::Device(e.to_string()))?;
    fs2::FileExt::try_lock_exclusive(&f).map_err(|_| HidError::Busy)?;
    Ok(f)
}

#[cfg(test)]
mod file_lock_tests {
    use super::*;
    #[test]
    fn independent_handles_cannot_overlap_and_close_releases_lock() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/hid-lock-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        let first = lock_in_dir(&dir).unwrap();
        assert!(matches!(lock_in_dir(&dir), Err(HidError::Busy)));
        drop(first);
        let next = lock_in_dir(&dir).unwrap();
        drop(next);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
fn run_device<T: Send + 'static>(
    selected: DeviceSummary,
    operation: impl FnOnce(&mut HidTransport) -> Result<T> + Send + 'static,
) -> Result<T> {
    run_device_checked(selected, true, operation)
}
fn run_device_checked<T: Send + 'static>(
    selected: DeviceSummary,
    require_original_f75: bool,
    operation: impl FnOnce(&mut HidTransport) -> Result<T> + Send + 'static,
) -> Result<T> {
    let (tx, rx) = mpsc::channel();
    let deadline = Instant::now() + Duration::from_secs(5);
    std::thread::spawn(move || {
        let outcome = (|| {
            let _guard = OPERATION.try_lock().map_err(|_| HidError::Busy)?;
            let _file = process_lock()?;
            if selected.connection_id.is_empty()
                || connection_identity(&selected.path).as_deref() != Some(&selected.connection_id)
            {
                return Err(HidError::Disconnected);
            }
            let api = hidapi::HidApi::new().map_err(|e| HidError::Device(e.to_string()))?;
            let info = api
                .device_list()
                .find(|d| {
                    d.path().to_bytes() == selected.path.as_bytes()
                        && d.vendor_id() == 0x258a
                        && d.product_id() == 0x010c
                        && d.interface_number() == 1
                        && d.usage_page() == 0xff00
                        && d.usage() == 1
                })
                .ok_or(HidError::Disconnected)?;
            if descriptor_feature_length(&selected.path).ok().flatten() != Some(REPORT_SIZE) {
                return Err(HidError::Unsupported("descriptor changed".into()));
            }
            let device = info
                .open_device(&api)
                .map_err(|e| HidError::Device(e.to_string()))?;
            if connection_identity(&selected.path).as_deref() != Some(&selected.connection_id) {
                return Err(HidError::Disconnected);
            }
            let mut transport = HidTransport { device, deadline };
            if require_original_f75 {
                require_f75_identity(transport.read_model_identity()?)?;
            }
            operation(&mut transport)
        })();
        let _ = tx.send(outcome);
    });
    rx.recv_timeout(Duration::from_secs(5))
        .map_err(|e| match e {
            mpsc::RecvTimeoutError::Timeout => HidError::Timeout,
            mpsc::RecvTimeoutError::Disconnected => HidError::Disconnected,
        })?
}
pub fn read_snapshot(path: &str) -> Result<Snapshot> {
    let selected = discover()?
        .into_iter()
        .find(|d| d.path == path && d.read_supported)
        .ok_or_else(|| {
            HidError::Unsupported("device missing or descriptor/interface mismatch".into())
        })?;
    let device = selected.clone();
    run_device(selected, move |transport| {
        snapshot_from_transport(transport, device, "physical_read")
    })
}
/// Only call after a human has reviewed a prepared plan and its durable backup.
/// No CLI or automated test invokes this physical entry point.
pub fn commit_lighting(plan: transaction::LightingPlan) -> Result<Snapshot> {
    check_write_policy()?;
    plan.validate()?;
    let selected = plan.before.device.clone();
    if !selected.read_supported
        || selected.vendor_id != 0x258a
        || selected.product_id != 0x010c
        || selected.interface_number != 1
    {
        return Err(HidError::Unsupported(
            "Not a supported wired lighting plan".into(),
        ));
    }
    run_device(selected, move |transport| {
        transaction::execute_plan(transport, &plan)
    })
}

fn check_write_policy() -> Result<()> {
    if std::env::var_os("AULA_DISABLE_HARDWARE_WRITES").is_some() {
        Err(HidError::Unsupported(
            "Physical configuration writes are disabled by AULA_DISABLE_HARDWARE_WRITES".into(),
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn descriptor_count_includes_id() {
        let d = [0x85, 6, 0x75, 8, 0x96, 7, 2, 0xb1, 2];
        assert_eq!(feature_report_lengths(&d).unwrap().get(&6), Some(&520));
    }
    #[test]
    fn descriptor_accumulates_and_pushes() {
        let d = [
            0x85, 6, 0x75, 8, 0x95, 10, 0xb1, 2, 0xa4, 0x85, 9, 0x95, 2, 0xb1, 2, 0xb4, 0xb1, 2,
        ];
        let n = feature_report_lengths(&d).unwrap();
        assert_eq!(n.get(&6), Some(&21));
        assert_eq!(n.get(&9), Some(&3));
    }
    #[test]
    fn bad_descriptors() {
        for d in [
            vec![0x96, 1],
            vec![0xb4],
            vec![0x85, 0],
            vec![0xfe, 4, 1, 0],
            vec![0xa4],
        ] {
            assert!(feature_report_lengths(&d).is_err())
        }
    }
    #[test]
    fn mock_disconnect_reconnect() {
        let mut m = MockTransport::default();
        m.set_connected(false);
        assert!(matches!(
            m.read_region(Region::Config),
            Err(HidError::Disconnected)
        ));
        m.set_connected(true);
        assert!(m.read_region(Region::Config).is_ok());
    }
    #[test]
    fn timeout_recovery() {
        let mut m = MockTransport::default();
        m.fail_next(HidError::Timeout);
        assert!(matches!(
            m.read_region(Region::Config),
            Err(HidError::Timeout)
        ));
        assert!(m.read_region(Region::Config).is_ok());
    }
    #[test]
    fn mock_snapshot_and_apply() {
        let mut m = MockTransport::default();
        let before = snapshot_from_transport(&mut m, demo_device(), "demo").unwrap();
        let config = ConfigRegion::parse(&before.config)
            .unwrap()
            .with_lighting(aula_protocol::LightingSettings {
                effect: 7,
                brightness: 9,
                speed: 4,
                colorful: true,
            })
            .unwrap();
        m.apply_demo(Region::Config, config.as_bytes()).unwrap();
        let after = snapshot_from_transport(&mut m, demo_device(), "demo").unwrap();
        assert_ne!(before.config, after.config);
        assert_eq!(after.source, "demo");
    }
    #[test]
    fn invalid_snapshot_schema() {
        let mut m = MockTransport::default();
        let mut s = snapshot_from_transport(&mut m, demo_device(), "demo").unwrap();
        s.schema_version = 2;
        assert!(s.validate().is_err());
    }
    #[test]
    fn corrupt_configuration_rejected() {
        let mut m = MockTransport::default();
        assert!(m.apply_demo(Region::Config, &[0; 128]).is_err());
    }
}

/// A documented read-only identity query. This does not unlock, change pairing,
/// write identity, switch firmware modes, or send a configuration report.
pub fn read_model_identity(path: &str) -> Result<[u8; 6]> {
    let selected = selected_device(path)?;
    run_device_checked(selected, false, |t| t.read_model_identity())
}
fn require_f75_identity(identity: [u8; 6]) -> Result<()> {
    if identity != [3, 0, 0, 0, 0, 0xcd] {
        return Err(HidError::Unsupported("Shared USB ID is not the verified original F75 model identity; configuration is blocked. Use the device library for model-specific tools.".into()));
    }
    Ok(())
}
#[derive(Debug, Clone, Serialize)]
pub struct CatalogDevice {
    pub path: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub interface_number: i32,
    pub product_name: String,
    pub candidates: Vec<String>,
    pub hardware_writes_enabled: bool,
    pub read_candidate: bool,
    pub product_name_matched: bool,
    pub note: String,
}
/// Model candidates from read-only USB metadata. Never opens a handle or guesses
/// a model when multiple products share the same vendor/product/interface.
pub fn discover_catalog_devices() -> Result<Vec<CatalogDevice>> {
    let api = hidapi::HidApi::new().map_err(|e| HidError::Device(e.to_string()))?;
    let mut result = Vec::new();
    for info in api.device_list() {
        let (candidates, product_name_matched) = aula_models::usb_candidates_with_product(
            info.vendor_id(),
            info.product_id(),
            info.interface_number(),
            info.product_string().unwrap_or(""),
        )
        .map_err(HidError::Unsupported)?;
        if candidates.is_empty() {
            continue;
        }
        let read_candidate = info.vendor_id() == 0x258a
            && info.product_id() == 0x010c
            && info.interface_number() == 1
            && info.usage_page() == 0xff00
            && info.usage() == 1
            && descriptor_feature_length(&info.path().to_string_lossy())
                .ok()
                .flatten()
                == Some(REPORT_SIZE);
        result.push(CatalogDevice {
            path:info.path().to_string_lossy().into_owned(), vendor_id:info.vendor_id(), product_id:info.product_id(),
            interface_number:info.interface_number(), product_name:info.product_string().unwrap_or("").into(),
            candidates:candidates.iter().map(|m|m.id.clone()).collect(), hardware_writes_enabled:false,
            product_name_matched, read_candidate,
            note:"USB metadata candidate. Explicit model captures verify exact identity before bounded reads; additional-model writes remain disabled.".into(),
        });
    }
    result.sort_by(|a, b| a.path.cmp(&b.path));
    result.dedup_by(|a, b| {
        if a.path == b.path {
            b.read_candidate |= a.read_candidate;
            true
        } else {
            false
        }
    });
    Ok(result)
}
#[cfg(test)]
mod model_identity_tests {
    use super::*;
    #[test]
    fn shared_vid_pid_does_not_grant_f75_configuration_access() {
        assert!(require_f75_identity([3, 0, 0, 0, 0, 0xcd]).is_ok());
        for id in [
            [3, 0, 0, 0, 0, 0x8f],
            [3, 0, 0, 0, 1, 0x0b],
            [3, 0, 0, 0, 1, 0xba],
            [0; 6],
        ] {
            assert!(require_f75_identity(id).is_err());
        }
    }
}

/// Explicit bounded reads for independently established G5 models. Identity is
/// checked inside the same device lease before any model-specific bank read.
pub fn read_model_capture(path: &str, model_id: &str) -> Result<aula_models::g5_capture::Capture> {
    aula_models::g5_capture::expected_identity(model_id).map_err(HidError::Unsupported)?;
    let selected = discover()?
        .into_iter()
        .find(|d| d.path == path && d.read_supported && d.usage_page == 0xff00 && d.usage == 1)
        .ok_or_else(|| {
            HidError::Unsupported(
                "Model reader requires wired interface1, vendor collectionff00/1 and feature06/520B"
                    .into(),
            )
        })?;
    let expected = model_id.to_string();
    run_device_checked(selected, false, move |t| {
        model_read::capture_from_transport(
            t,
            &expected,
            aula_models::g5_capture::CaptureSource::PhysicalRead,
        )
    })
}
impl model_read::ModelReadTransport for HidTransport {
    fn read(&mut self, request: aula_models::g5_capture::ReadRequest) -> Result<Vec<u8>> {
        self.check_deadline()?;
        std::thread::sleep(Duration::from_millis(20));
        self.read_report(&request.bytes())
    }
}
