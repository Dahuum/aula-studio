//! Bounded keymap and per-key transactions prepared from complete bank reads.
//! These operations require a durable backup and an explicit reviewed GUI action.
use crate::{snapshot_from_transport, transaction::ConfigTransport, HidError, Result, Snapshot};
use aula_protocol::{
    matrix_index_for_usage, ConfigRegion, KeyAction, Keymap, Layer, LightingSettings, PerKeyColors,
    Region,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
enum Intent {
    Remap(Vec<(u16, KeyAction)>),
    PerKey(Vec<(u16, [u8; 3])>),
    RestoreKeymap(Keymap),
    RestoreColors(PerKeyColors),
}

#[derive(Debug, Clone)]
pub struct MappingPlan {
    pub before: Snapshot,
    pub config: Vec<u8>,
    pub keymap_before: Option<Keymap>,
    pub keymap: Option<Keymap>,
    pub per_key_before: Option<PerKeyColors>,
    pub per_key: Option<PerKeyColors>,
    intent: Intent,
}
impl MappingPlan {
    pub fn remap(
        before: Snapshot,
        keymap_before: Keymap,
        changes: &[(u16, KeyAction)],
    ) -> Result<Self> {
        before.validate()?;
        let keymap = patch_keymap(&keymap_before, changes)?;
        Ok(Self {
            config: before.config.clone(),
            before,
            keymap_before: Some(keymap_before),
            keymap: Some(keymap),
            per_key_before: None,
            per_key: None,
            intent: Intent::Remap(changes.to_vec()),
        })
    }
    pub fn per_key(
        before: Snapshot,
        per_key_before: PerKeyColors,
        changes: &[(u16, [u8; 3])],
    ) -> Result<Self> {
        before.validate()?;
        let per_key = patch_colors(&per_key_before, changes)?;
        let config = custom_config(&before)?;
        Ok(Self {
            before,
            config,
            keymap_before: None,
            keymap: None,
            per_key_before: Some(per_key_before),
            per_key: Some(per_key),
            intent: Intent::PerKey(changes.to_vec()),
        })
    }
    pub fn restore_keymap(before: Snapshot, current: Keymap, saved: Keymap) -> Result<Self> {
        before.validate()?;
        let keymap = current.restore_known_sources(&saved)?;
        Ok(Self {
            config: before.config.clone(),
            before,
            keymap_before: Some(current),
            keymap: Some(keymap),
            per_key_before: None,
            per_key: None,
            intent: Intent::RestoreKeymap(saved),
        })
    }
    pub fn restore_colors(
        before: Snapshot,
        current: PerKeyColors,
        saved: PerKeyColors,
    ) -> Result<Self> {
        before.validate()?;
        let colors = current.restore_known_colors(&saved)?;
        Ok(Self {
            config: before.config.clone(),
            before,
            keymap_before: None,
            keymap: None,
            per_key_before: Some(current),
            per_key: Some(colors),
            intent: Intent::RestoreColors(saved),
        })
    }
    pub fn validate(&self) -> Result<()> {
        self.before.validate()?;
        let expected = match &self.intent {
            Intent::RestoreKeymap(saved) => Self::restore_keymap(
                self.before.clone(),
                self.keymap_before.clone().ok_or_else(invalid_plan)?,
                saved.clone(),
            )?,
            Intent::RestoreColors(saved) => Self::restore_colors(
                self.before.clone(),
                self.per_key_before.clone().ok_or_else(invalid_plan)?,
                saved.clone(),
            )?,
            Intent::Remap(changes) => Self::remap(
                self.before.clone(),
                self.keymap_before.clone().ok_or_else(invalid_plan)?,
                changes,
            )?,
            Intent::PerKey(changes) => Self::per_key(
                self.before.clone(),
                self.per_key_before.clone().ok_or_else(invalid_plan)?,
                changes,
            )?,
        };
        if self.config != expected.config
            || self.keymap_before != expected.keymap_before
            || self.keymap != expected.keymap
            || self.per_key_before != expected.per_key_before
            || self.per_key != expected.per_key
        {
            return Err(invalid_plan());
        }
        Ok(())
    }
    pub fn has_changes(&self) -> bool {
        self.config != self.before.config
            || self.keymap != self.keymap_before
            || self.per_key != self.per_key_before
    }
}
fn invalid_plan() -> HidError {
    HidError::Unsupported("mapping plan was modified outside its reviewed changes".into())
}
fn check_sources<T>(changes: &[(u16, T)], allow_fn_color: bool) -> Result<()> {
    if changes.is_empty() || changes.len() > 80 {
        return Err(HidError::Unsupported(
            "a mapping change must contain 1..80 selected keys".into(),
        ));
    }
    let mut seen = BTreeSet::new();
    for (usage, _) in changes {
        if *usage != 0 || !allow_fn_color {
            matrix_index_for_usage(*usage)?;
        }
        if !seen.insert(*usage) {
            return Err(HidError::Unsupported("duplicate source key".into()));
        }
    }
    Ok(())
}
fn supported_output(action: KeyAction) -> Result<()> {
    action.validate()?;
    let supported = match action.page {
        0 => (4..=0xa4).contains(&action.usage) || (0xe0..=0xe7).contains(&action.usage),
        2 => aula_protocol::media_action_name(action.usage).is_some(),
        _ => false,
    };
    if !supported {
        return Err(HidError::Unsupported(
            "unsupported output usage; disabling keys is not allowed".into(),
        ));
    }
    Ok(())
}
fn patch_keymap(before: &Keymap, changes: &[(u16, KeyAction)]) -> Result<Keymap> {
    check_sources(changes, false)?;
    let mut result = before.clone();
    for (usage, action) in changes {
        supported_output(*action)?;
        // Keymap::set additionally protects F1..F12 and the device's Fn source.
        let index = matrix_index_for_usage(*usage)?;
        if !result.matches_action(index, *action)? {
            result.set(index, *action)?;
        }
    }
    Ok(result)
}
fn patch_colors(before: &PerKeyColors, changes: &[(u16, [u8; 3])]) -> Result<PerKeyColors> {
    check_sources(changes, true)?;
    let mut result = before.clone();
    for (usage, rgb) in changes {
        let index = if *usage == 0 {
            aula_protocol::FN_MATRIX_INDEX
        } else {
            matrix_index_for_usage(*usage)?
        };
        result.set(index, *rgb)?;
    }
    Ok(result)
}
fn custom_config(before: &Snapshot) -> Result<Vec<u8>> {
    Ok(ConfigRegion::parse(&before.config)?
        .with_lighting(LightingSettings {
            effect: 21,
            brightness: 0,
            speed: 0,
            colorful: false,
        })?
        .as_bytes()
        .to_vec())
}

pub trait ExtendedTransport: ConfigTransport {
    fn read_keymap(&mut self, layer: Layer) -> Result<Keymap>;
    fn write_keymap(&mut self, map: &Keymap) -> Result<()>;
    fn read_per_key(&mut self) -> Result<PerKeyColors>;
    fn write_per_key(&mut self, colors: &PerKeyColors) -> Result<()>;
}
#[derive(Debug, Clone)]
pub struct MappingResult {
    pub snapshot: Snapshot,
    pub keymap: Option<Keymap>,
    pub per_key: Option<PerKeyColors>,
}

pub fn execute_mapping(
    transport: &mut impl ExtendedTransport,
    plan: &MappingPlan,
) -> Result<MappingResult> {
    plan.validate()?;
    let current =
        snapshot_from_transport(transport, plan.before.device.clone(), &plan.before.source)?;
    if current.config != plan.before.config || current.colors != plan.before.colors {
        return Err(stale());
    }
    if let Some(before) = &plan.keymap_before {
        if transport.read_keymap(before.layer())? != *before {
            return Err(stale());
        }
    }
    if let Some(before) = &plan.per_key_before {
        if transport.read_per_key()? != *before {
            return Err(stale());
        }
    }
    let mut completed = 0usize;
    if let Some(map) = &plan.keymap {
        if Some(map) != plan.keymap_before.as_ref() {
            transport
                .write_keymap(map)
                .map_err(|e| partial("keymap write", completed, e))?;
            completed += 1;
        }
    }
    if let Some(colors) = &plan.per_key {
        if Some(colors) != plan.per_key_before.as_ref() {
            transport
                .write_per_key(colors)
                .map_err(|e| partial("per-key write", completed, e))?;
            completed += 1;
        }
    }
    // Official per-key path commits current configuration after the LED bank,
    // including when custom mode was already selected.
    if plan.config != plan.before.config || plan.per_key != plan.per_key_before {
        transport
            .write_region(Region::Config, &plan.config)
            .map_err(|e| partial("config write", completed, e))?;
        completed += 1;
    }

    let snapshot =
        snapshot_from_transport(transport, plan.before.device.clone(), &plan.before.source)
            .map_err(|e| partial("readback", completed, e))?;
    let keymap = match &plan.keymap {
        Some(map) => Some(
            transport
                .read_keymap(map.layer())
                .map_err(|e| partial("keymap readback", completed, e))?,
        ),
        None => None,
    };
    let per_key = if plan.per_key.is_some() {
        Some(
            transport
                .read_per_key()
                .map_err(|e| partial("per-key readback", completed, e))?,
        )
    } else {
        None
    };
    if snapshot.config != plan.config
        || snapshot.colors != plan.before.colors
        || keymap != plan.keymap
        || per_key != plan.per_key
    {
        return Err(HidError::Device(format!(
            "mapping readback mismatch after {completed} completed writes. Apply is unconfirmed; no retry or rollback was sent"
        )));
    }
    Ok(MappingResult {
        snapshot,
        keymap,
        per_key,
    })
}
fn stale() -> HidError {
    HidError::Unsupported(
        "keyboard settings or mapping changed since review; no writes were sent. Read a new backup and review again".into(),
    )
}
fn partial(operation: &str, completed: usize, error: HidError) -> HidError {
    HidError::Device(format!(
        "{operation} failed after {completed} completed writes: {error}. Apply is unconfirmed and state may be partially changed; no retry or rollback was sent"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{demo_device, MockTransport, Transport};

    struct MockExtended {
        regions: MockTransport,
        base: Keymap,
        fn_layer: Keymap,
        colors: PerKeyColors,
        writes: Vec<&'static str>,
        ignore_writes: bool,
        fail_write_at: Option<usize>,
    }
    impl Default for MockExtended {
        fn default() -> Self {
            Self {
                regions: MockTransport::default(),
                base: Keymap::empty(Layer::Base),
                fn_layer: Keymap::empty(Layer::Fn),
                colors: PerKeyColors::black(),
                writes: Vec::new(),
                ignore_writes: false,
                fail_write_at: None,
            }
        }
    }
    impl MockExtended {
        fn writing(&mut self, name: &'static str) -> Result<()> {
            self.writes.push(name);
            if self.fail_write_at == Some(self.writes.len()) {
                return Err(HidError::Disconnected);
            }
            Ok(())
        }
        fn before(&mut self) -> Snapshot {
            snapshot_from_transport(self, demo_device(), "demo").unwrap()
        }
    }
    impl Transport for MockExtended {
        fn read_region(&mut self, region: Region) -> Result<Vec<u8>> {
            self.regions.read_region(region)
        }
    }
    impl ConfigTransport for MockExtended {
        fn write_region(&mut self, region: Region, data: &[u8]) -> Result<()> {
            self.writing("config")?;
            if !self.ignore_writes {
                self.regions.apply_demo(region, data)?;
            }
            Ok(())
        }
    }
    impl ExtendedTransport for MockExtended {
        fn read_keymap(&mut self, layer: Layer) -> Result<Keymap> {
            Ok(match layer {
                Layer::Base => self.base.clone(),
                Layer::Fn => self.fn_layer.clone(),
            })
        }
        fn write_keymap(&mut self, map: &Keymap) -> Result<()> {
            self.writing("keymap")?;
            if !self.ignore_writes {
                match map.layer() {
                    Layer::Base => self.base = map.clone(),
                    Layer::Fn => self.fn_layer = map.clone(),
                }
            }
            Ok(())
        }
        fn read_per_key(&mut self) -> Result<PerKeyColors> {
            Ok(self.colors.clone())
        }
        fn write_per_key(&mut self, colors: &PerKeyColors) -> Result<()> {
            self.writing("per_key")?;
            if !self.ignore_writes {
                self.colors = colors.clone();
            }
            Ok(())
        }
    }
    fn action() -> KeyAction {
        KeyAction {
            page: 0,
            usage: 0x68,
        }
    }
    fn remap(device: &mut MockExtended, layer: Layer) -> MappingPlan {
        let before = device.before();
        let map = device.read_keymap(layer).unwrap();
        MappingPlan::remap(before, map, &[(0x37, action())]).unwrap()
    }
    fn colors(device: &mut MockExtended) -> MappingPlan {
        let before = device.before();
        let colors = device.read_per_key().unwrap();
        MappingPlan::per_key(before, colors, &[(0x37, [1, 2, 3])]).unwrap()
    }

    #[test]
    fn base_patch_preserves_every_other_slot_and_layer() {
        let mut device = MockExtended::default();
        let plan = remap(&mut device, Layer::Base);
        let before = plan.keymap_before.as_ref().unwrap().as_bytes();
        let after = plan.keymap.as_ref().unwrap().as_bytes();
        for i in 0..520 {
            if !(240..244).contains(&i) {
                assert_eq!(before[i], after[i]);
            }
        }
        let fn_before = device.fn_layer.clone();
        execute_mapping(&mut device, &plan).unwrap();
        assert_eq!(device.writes, ["keymap"]);
        assert_eq!(device.fn_layer, fn_before);
        assert_eq!(device.base.get(58).unwrap(), action());
    }
    #[test]
    fn fn_layer_is_independent() {
        let mut device = MockExtended::default();
        let plan = remap(&mut device, Layer::Fn);
        let base_before = device.base.clone();
        execute_mapping(&mut device, &plan).unwrap();
        assert_eq!(device.base, base_before);
        assert_eq!(device.fn_layer.get(58).unwrap(), action());
    }
    #[test]
    fn media_remap_writes_only_selected_record_in_its_layer() {
        for layer in [Layer::Base, Layer::Fn] {
            let mut device = MockExtended::default();
            let before = device.before();
            let bank = device.read_keymap(layer).unwrap();
            let media = KeyAction {
                page: 2,
                usage: 0xcd,
            };
            let plan = MappingPlan::remap(before, bank, &[(4, media)]).unwrap();
            let index = matrix_index_for_usage(4).unwrap();
            let offset = 8 + 4 * index;
            assert_eq!(
                &plan.keymap.as_ref().unwrap().as_bytes()[offset..offset + 4],
                &[2, 0, 0, 0xcd]
            );
            execute_mapping(&mut device, &plan).unwrap();
            assert_eq!(device.writes, ["keymap"]);
            let other = if layer == Layer::Base {
                &device.fn_layer
            } else {
                &device.base
            };
            assert_eq!(*other, Keymap::empty(other.layer()));
        }
    }
    #[test]
    fn refuses_unknown_reserved_duplicate_and_disabled_assignments() {
        let mut device = MockExtended::default();
        for changes in [
            vec![(0xffff, action())],
            vec![(0x3a, action())],
            vec![(0x37, KeyAction { page: 0, usage: 0 })],
            vec![(0x37, action()), (0x37, action())],
            vec![(0x37, KeyAction { page: 2, usage: 1 })],
            vec![],
        ] {
            assert!(MappingPlan::remap(device.before(), device.base.clone(), &changes).is_err());
        }
        assert!(device.writes.is_empty());
    }
    #[test]
    fn stale_keymap_aborts_without_write() {
        let mut device = MockExtended::default();
        let plan = remap(&mut device, Layer::Base);
        device.base.set(9, action()).unwrap();
        assert!(execute_mapping(&mut device, &plan)
            .unwrap_err()
            .to_string()
            .contains("changed since review"));
        assert!(device.writes.is_empty());
    }
    #[test]
    fn perkey_preserves_other_leds_and_selects_custom_after_bank_write() {
        let mut device = MockExtended::default();
        let plan = colors(&mut device);
        let table = plan.per_key.as_ref().unwrap();
        for index in 0..126 {
            assert_eq!(
                table.get(index).unwrap(),
                if index == 58 { [1, 2, 3] } else { [0; 3] }
            );
        }
        for (i, old) in plan.before.config.iter().enumerate() {
            if ![9, 10, 11].contains(&i) {
                assert_eq!(*old, plan.config[i]);
            }
        }
        assert_eq!(plan.config[9], 1);
        assert_eq!(plan.config[10], 21);
        assert_eq!(plan.config[11], 0x20);
        execute_mapping(&mut device, &plan).unwrap();
        assert_eq!(device.writes, ["per_key", "config"]);
    }
    #[test]
    fn stale_perkey_aborts_without_write() {
        let mut device = MockExtended::default();
        let plan = colors(&mut device);
        device.colors.set(9, [9, 8, 7]).unwrap();
        assert!(execute_mapping(&mut device, &plan).is_err());
        assert!(device.writes.is_empty());
    }
    #[test]
    fn partial_apply_keeps_backup_and_does_not_retry() {
        let mut device = MockExtended::default();
        let plan = colors(&mut device);
        device.fail_write_at = Some(2);
        let error = execute_mapping(&mut device, &plan).unwrap_err().to_string();
        assert!(error.contains("after 1 completed writes"));
        assert_eq!(device.writes, ["per_key", "config"]);
        assert_eq!(device.colors, *plan.per_key.as_ref().unwrap());
        assert_eq!(
            device.read_region(Region::Config).unwrap(),
            plan.before.config
        );
    }
    #[test]
    fn readback_mismatch_is_unconfirmed() {
        let mut device = MockExtended::default();
        let plan = remap(&mut device, Layer::Base);
        device.ignore_writes = true;
        let error = execute_mapping(&mut device, &plan).unwrap_err().to_string();
        assert!(error.contains("readback mismatch"));
        assert_eq!(device.writes, ["keymap"]);
    }
    #[test]
    fn tampered_config_rejected_before_io() {
        let mut device = MockExtended::default();
        let mut plan = remap(&mut device, Layer::Base);
        plan.config[20] ^= 1;
        assert!(execute_mapping(&mut device, &plan).is_err());
        assert!(device.writes.is_empty());
    }
    #[test]
    fn matching_map_is_a_noop() {
        let mut device = MockExtended::default();
        device.base.set(58, action()).unwrap();
        let plan = remap(&mut device, Layer::Base);
        assert!(!plan.has_changes());
        execute_mapping(&mut device, &plan).unwrap();
        assert!(device.writes.is_empty());
    }
    #[test]
    fn updating_already_selected_custom_mode_still_commits_config() {
        let mut device = MockExtended::default();
        let initial = colors(&mut device);
        execute_mapping(&mut device, &initial).unwrap();
        device.writes.clear();
        let before = device.before();
        let current = device.read_per_key().unwrap();
        let plan = MappingPlan::per_key(before, current, &[(55, [4, 5, 6])]).unwrap();
        assert_eq!(plan.config, plan.before.config);
        execute_mapping(&mut device, &plan).unwrap();
        assert_eq!(device.writes, ["per_key", "config"]);
    }
}

#[cfg(test)]
mod fn_color_tests {
    use super::*;
    use crate::{demo_device, MockTransport};
    #[test]
    fn fn_rgb_is_not_a_remapping_source() {
        let mut m = MockTransport::default();
        let before = snapshot_from_transport(&mut m, demo_device(), "demo").unwrap();
        let p =
            MappingPlan::per_key(before.clone(), PerKeyColors::black(), &[(0, [7, 8, 9])]).unwrap();
        p.validate().unwrap();
        assert_eq!(
            p.per_key
                .unwrap()
                .get(aula_protocol::FN_MATRIX_INDEX)
                .unwrap(),
            [7, 8, 9]
        );
        assert!(MappingPlan::remap(
            before,
            Keymap::empty(Layer::Base),
            &[(0, KeyAction { page: 0, usage: 4 })]
        )
        .is_err());
    }
}

#[cfg(test)]
mod restore_bank_tests {
    use super::*;
    use crate::{demo_device, MockTransport};
    #[test]
    fn restoring_known_banks_preserves_controls_and_unknown_slots() {
        let mut m = MockTransport::default();
        let before = snapshot_from_transport(&mut m, demo_device(), "demo").unwrap();
        let mut raw = *Keymap::empty(Layer::Base).as_bytes();
        for usage in 1..=255u16 {
            if let Ok(index) = matrix_index_for_usage(usage) {
                let o = 8 + 4 * index;
                raw[o..o + 4].copy_from_slice(&[0, 0, 0, usage as u8]);
            }
        }
        raw[8 + 4 * 59..12 + 4 * 59].copy_from_slice(&[0, 16, 0, 0]);
        let saved = Keymap::parse(&raw).unwrap();
        let mut current = saved.clone();
        current.set(9, KeyAction { page: 0, usage: 5 }).unwrap();
        let plan = MappingPlan::restore_keymap(before.clone(), current, saved.clone()).unwrap();
        plan.validate().unwrap();
        assert_eq!(plan.keymap.unwrap().get(9).unwrap().usage, 4);
        let mut bad = *saved.as_bytes();
        bad[8 + 4 * 12 + 3] = 4;
        assert!(saved
            .restore_known_sources(&Keymap::parse(&bad).unwrap())
            .is_err());
        let mut original = PerKeyColors::black();
        original
            .set(aula_protocol::FN_MATRIX_INDEX, [1, 2, 3])
            .unwrap();
        let mut now = original.clone();
        now.set(9, [9; 3]).unwrap();
        now.set(125, [7; 3]).unwrap();
        let plan = MappingPlan::restore_colors(before, now, original).unwrap();
        plan.validate().unwrap();
        let colors = plan.per_key.unwrap();
        assert_eq!(colors.get(9).unwrap(), [0; 3]);
        assert_eq!(colors.get(125).unwrap(), [7; 3]);
        assert_eq!(
            colors.get(aula_protocol::FN_MATRIX_INDEX).unwrap(),
            [1, 2, 3]
        );
    }
}

#[cfg(test)]
mod combination_binding_regression {
    use super::*;
    use crate::{demo_device, MockTransport};
    #[test]
    fn explicit_plain_key_replaces_modifier_combo() {
        let mut m = MockTransport::default();
        let before = snapshot_from_transport(&mut m, demo_device(), "demo").unwrap();
        let mut bytes = *Keymap::empty(Layer::Base).as_bytes();
        bytes[44..48].copy_from_slice(&[0, 1, 0, 4]);
        let map = Keymap::parse(&bytes).unwrap();
        assert!(!map
            .matches_action(9, KeyAction { page: 0, usage: 4 })
            .unwrap());
        let plan =
            MappingPlan::remap(before, map, &[(4, KeyAction { page: 0, usage: 4 })]).unwrap();
        assert!(plan.has_changes());
        assert_eq!(&plan.keymap.unwrap().as_bytes()[44..48], &[0, 0, 0, 4]);
    }
}
