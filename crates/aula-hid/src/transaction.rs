//! Reviewable, bounded lighting transactions. Preparing a plan performs no I/O.
//! The caller must save `before` durably and obtain an explicit apply action
//! before passing a physical transport to `execute_plan`.
use crate::{snapshot_from_transport, HidError, Result, Snapshot, Transport};
use aula_protocol::{ColorTable, ConfigRegion, LightingSettings, Region};

/// A lighting-only change derived from a previously read device snapshot.
/// Private intent fields allow validation to reject edits to the public buffers.
#[derive(Debug, Clone)]
pub struct LightingPlan {
    pub before: Snapshot,
    pub config: Vec<u8>,
    pub colors: Vec<u8>,
    settings: LightingSettings,
    color: Option<[u8; 3]>,
    sleep_seconds: Option<u16>,
    restore: Option<Snapshot>,
    preset_brightness: Option<u8>,
}
impl LightingPlan {
    pub fn prepare(
        before: Snapshot,
        settings: LightingSettings,
        color: Option<[u8; 3]>,
    ) -> Result<Self> {
        before.validate()?;
        let (config, colors) = prepare_regions(&before, settings, color)?;
        Ok(Self {
            before,
            config,
            colors,
            settings,
            color,
            sleep_seconds: None,
            restore: None,
            preset_brightness: None,
        })
    }

    pub fn prepare_sleep(before: Snapshot, seconds: u16) -> Result<Self> {
        before.validate()?;
        let config = ConfigRegion::parse(&before.config)?;
        let settings = config.lighting()?;
        let next = config.with_sleep_seconds(seconds)?;
        Ok(Self {
            colors: before.colors.clone(),
            before,
            config: next.as_bytes().to_vec(),
            settings,
            color: None,
            sleep_seconds: Some(seconds),
            restore: None,
            preset_brightness: None,
        })
    }
    /// Restore only settings this application can change: preset records,
    /// adjustable first palette colors and sleep. Unknown bytes are preserved.
    pub fn prepare_restore(before: Snapshot, saved: Snapshot) -> Result<Self> {
        let (config, colors) = restore_regions(&before, &saved)?;
        let settings = ConfigRegion::parse(&saved.config)?.lighting()?;
        Ok(Self {
            before,
            config,
            colors,
            settings,
            color: None,
            sleep_seconds: None,
            restore: Some(saved),
            preset_brightness: None,
        })
    }
    pub fn prepare_brightness(before: Snapshot, value: u8) -> Result<Self> {
        before.validate()?;
        let original = ConfigRegion::parse(&before.config)?;
        let settings = original.lighting()?;
        let config = original.with_preset_brightness(value)?.as_bytes().to_vec();
        Ok(Self {
            colors: before.colors.clone(),
            before,
            config,
            settings,
            color: None,
            sleep_seconds: None,
            restore: None,
            preset_brightness: Some(value),
        })
    }
    /// Check lengths, signatures, intent bounds, and that no unrelated bytes
    /// were changed after the reviewed plan was prepared.
    pub fn validate(&self) -> Result<()> {
        self.before.validate()?;
        let (config, colors) = if let Some(saved) = &self.restore {
            restore_regions(&self.before, saved)?
        } else if let Some(seconds) = self.sleep_seconds {
            (
                ConfigRegion::parse(&self.before.config)?
                    .with_sleep_seconds(seconds)?
                    .as_bytes()
                    .to_vec(),
                self.before.colors.clone(),
            )
        } else if let Some(value) = self.preset_brightness {
            (
                ConfigRegion::parse(&self.before.config)?
                    .with_preset_brightness(value)?
                    .as_bytes()
                    .to_vec(),
                self.before.colors.clone(),
            )
        } else {
            prepare_regions(&self.before, self.settings, self.color)?
        };
        if self.config != config || self.colors != colors {
            return Err(HidError::Unsupported(
                "lighting plan was modified outside its reviewed settings".into(),
            ));
        }
        Ok(())
    }

    pub fn has_changes(&self) -> bool {
        self.config != self.before.config || self.colors != self.before.colors
    }
}

fn prepare_regions(
    before: &Snapshot,
    settings: LightingSettings,
    color: Option<[u8; 3]>,
) -> Result<(Vec<u8>, Vec<u8>)> {
    settings.validate()?;
    // Custom/per-key activation requires another protocol and is intentionally
    // outside a preset-lighting transaction.
    if settings.effect > 18 {
        return Err(HidError::Unsupported(
            "hardware plan supports preset effects 0..18 only".into(),
        ));
    }
    if color.is_some() && settings.colorful {
        return Err(HidError::Unsupported(
            "a fixed color cannot be combined with colorful/random mode".into(),
        ));
    }
    if color.is_some()
        && ![1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 18].contains(&settings.effect)
    {
        return Err(HidError::Unsupported(
            "selected effect does not support a fixed color in the official package".into(),
        ));
    }
    let config = ConfigRegion::parse(&before.config)?.with_lighting(settings)?;
    let palette = ColorTable::parse(&before.colors)?;
    let colors = match color {
        Some(rgb) => palette.with_effect_color(settings.effect, rgb)?,
        None => palette,
    };
    Ok((config.as_bytes().to_vec(), colors.as_bytes().to_vec()))
}

fn restore_regions(before: &Snapshot, saved: &Snapshot) -> Result<(Vec<u8>, Vec<u8>)> {
    before.validate()?;
    saved.validate()?;
    if saved.source != "physical_read"
        || saved.device.vendor_id != before.device.vendor_id
        || saved.device.product_id != before.device.product_id
        || saved.device.interface_number != before.device.interface_number
        || saved.device.product_name != before.device.product_name
        || saved.device.feature_report_bytes != Some(520)
    {
        return Err(HidError::Unsupported(
            "backup is not a matching physical F75 snapshot".into(),
        ));
    }
    let old = ConfigRegion::parse(&saved.config)?;
    let selected = old.lighting()?;
    if selected.effect == 21 {
        return Err(HidError::Unsupported("Snapshot used custom per-key mode; restore its per-key bank separately instead of a preset restore".into()));
    }
    let current = ConfigRegion::parse(&before.config)?;
    current.lighting()?;
    current.sleep_seconds()?;
    let mut config = current
        .with_sleep_seconds(old.sleep_seconds()?)?
        .as_bytes()
        .to_vec();
    if saved.config[9] > 1 {
        return Err(HidError::Unsupported(
            "unknown saved custom-mode flag".into(),
        ));
    }
    // Local reads can retain this flag with a normal preset; the cause is unverified.
    // Restore the actual saved field instead of normalizing a legal read state.
    config[9] = saved.config[9];
    config[10] = selected.effect;
    let mut colors = ColorTable::parse(&before.colors)?;
    let saved_colors = ColorTable::parse(&saved.colors)?;
    saved_colors.validate_for_write()?;
    for effect in 1..=18u8 {
        let o = 56 + 2 * usize::from(effect);
        if saved.config[o] > 9 || saved.config[o + 1] >> 4 > 4 {
            return Err(HidError::Unsupported(
                "backup preset parameters are outside supported bounds".into(),
            ));
        }
        config[o] = saved.config[o];
        let speed = if effect == 1 {
            config[o + 1] & 0xf0
        } else {
            saved.config[o + 1] & 0xf0
        };
        let flag = if aula_protocol::effect_supports_color(effect) {
            saved.config[o + 1] & 0x0f
        } else {
            config[o + 1] & 0x0f
        };
        config[o + 1] = speed | flag;
        if aula_protocol::effect_supports_color(effect) {
            colors = colors.with_effect_color(effect, saved_colors.effect_color(effect)?)?;
        }
    }
    Ok((config, colors.as_bytes().to_vec()))
}

/// A transport that can apply only the two enumerated configuration regions.
/// There is no arbitrary command or address in this interface.
pub trait ConfigTransport: Transport {
    fn write_region(&mut self, region: Region, data: &[u8]) -> Result<()>;
}

/// Detect stale state before the first write, write only changed regions, and
/// confirm the resulting RAM configuration through exact readback.
///
/// The returned snapshot does not establish persistence after power loss.
/// A failed write may have reached the device even if the host saw an error;
/// no retry or automatic rollback is attempted. The saved `before` snapshot
/// remains the recovery reference for a separately reviewed restoration.
pub fn execute_plan(transport: &mut impl ConfigTransport, plan: &LightingPlan) -> Result<Snapshot> {
    plan.validate()?;
    let current =
        snapshot_from_transport(transport, plan.before.device.clone(), &plan.before.source)?;
    if current.config != plan.before.config || current.colors != plan.before.colors {
        return Err(HidError::Unsupported(
            "keyboard settings changed since review; no writes were sent. Read a new snapshot and review again".into(),
        ));
    }

    let mut completed = 0usize;
    // Write colors before selecting their effect. Each write remains bounded
    // to a full existing region, preserving all bytes outside the plan.
    for (region, data, original) in [
        (Region::Colors, &plan.colors, &plan.before.colors),
        (Region::Config, &plan.config, &plan.before.config),
    ] {
        // Official ApplySetting commits04 after palette0a even if parameter bytes
        // did not change. Follow that sequence rather than assuming palette-only
        // persistence/activation semantics.
        if data == original && !(region == Region::Config && plan.colors != plan.before.colors) {
            continue;
        }
        transport.write_region(region, data).map_err(|error| {
            HidError::Device(format!(
                "{region:?} write failed after {completed} completed region writes: {error}. Device state may be partially changed; no retry or rollback was sent. Keep the backup and read the device again"
            ))
        })?;
        completed += 1;
    }
    let after = snapshot_from_transport(
        transport,
        plan.before.device.clone(),
        &plan.before.source,
    )
    .map_err(|error| {
        HidError::Device(format!(
            "readback failed after {completed} completed region writes: {error}. Apply is unconfirmed; no retry or rollback was sent"
        ))
    })?;
    if after.config != plan.config || after.colors != plan.colors {
        return Err(HidError::Device(format!(
            "readback mismatch after {completed} completed region writes. Apply is unconfirmed; device state may be partially changed. No retry or rollback was sent"
        )));
    }
    Ok(after)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{demo_device, MockTransport};

    #[derive(Default)]
    struct RecordingTransport {
        mock: MockTransport,
        writes: Vec<Region>,
        reads: usize,
        fail_write: bool,
        fail_write_at: Option<usize>,
        ignore_write: bool,
        disconnect_after_write: bool,
    }
    impl Transport for RecordingTransport {
        fn read_region(&mut self, region: Region) -> Result<Vec<u8>> {
            self.reads += 1;
            self.mock.read_region(region)
        }
    }
    impl ConfigTransport for RecordingTransport {
        fn write_region(&mut self, region: Region, data: &[u8]) -> Result<()> {
            self.writes.push(region);
            if self.fail_write || self.fail_write_at == Some(self.writes.len()) {
                // A USB write can succeed at the device and then fail at the
                // host boundary. Model that partial/ambiguous failure.
                self.mock.apply_demo(region, data)?;
                return Err(HidError::Disconnected);
            }
            if !self.ignore_write {
                self.mock.apply_demo(region, data)?;
            }
            if self.disconnect_after_write {
                self.mock.set_connected(false);
            }
            Ok(())
        }
    }
    fn snapshot(device: &mut RecordingTransport) -> Snapshot {
        snapshot_from_transport(device, demo_device(), "demo").unwrap()
    }
    fn settings() -> LightingSettings {
        LightingSettings {
            effect: 7,
            brightness: 8,
            speed: 3,
            colorful: true,
        }
    }

    #[test]
    fn prepare_preserves_unrelated_bytes() {
        let mut device = RecordingTransport::default();
        let before = snapshot(&mut device);
        let plan = LightingPlan::prepare(before.clone(), settings(), None).unwrap();
        plan.validate().unwrap();
        assert_eq!(plan.colors, before.colors);
        for (i, byte) in before.config.iter().enumerate() {
            if ![9, 10, 11, 70, 71].contains(&i) {
                assert_eq!(plan.config[i], *byte, "unexpected edit at {i}");
            }
        }
        assert!(device.writes.is_empty());
    }

    #[test]
    fn successful_apply_writes_changed_regions_only() {
        let mut device = RecordingTransport::default();
        let plan = LightingPlan::prepare(snapshot(&mut device), settings(), None).unwrap();
        let result = execute_plan(&mut device, &plan).unwrap();
        assert_eq!(result.config, plan.config);
        assert_eq!(result.colors, plan.colors);
        assert_eq!(device.writes, [Region::Config]);
    }

    #[test]
    fn unchanged_plan_sends_no_writes() {
        let mut device = RecordingTransport::default();
        let initial = snapshot(&mut device);
        let configured = LightingPlan::prepare(initial, settings(), None).unwrap();
        device
            .mock
            .apply_demo(Region::Config, &configured.config)
            .unwrap();
        let before = snapshot(&mut device);
        let plan = LightingPlan::prepare(before, settings(), None).unwrap();
        assert!(!plan.has_changes());
        execute_plan(&mut device, &plan).unwrap();
        assert!(device.writes.is_empty());
    }

    #[test]
    fn stale_config_aborts_before_writes() {
        let mut device = RecordingTransport::default();
        let plan = LightingPlan::prepare(snapshot(&mut device), settings(), None).unwrap();
        let mut changed = plan.before.config.clone();
        changed[20] ^= 1;
        device.mock.apply_demo(Region::Config, &changed).unwrap();
        let error = execute_plan(&mut device, &plan).unwrap_err().to_string();
        assert!(error.contains("changed since review"));
        assert!(device.writes.is_empty());
    }

    #[test]
    fn stale_palette_aborts_before_writes() {
        let mut device = RecordingTransport::default();
        let plan = LightingPlan::prepare(snapshot(&mut device), settings(), None).unwrap();
        let mut changed = plan.before.colors.clone();
        changed[20] ^= 1;
        device.mock.apply_demo(Region::Colors, &changed).unwrap();
        assert!(execute_plan(&mut device, &plan).is_err());
        assert!(device.writes.is_empty());
    }

    #[test]
    fn tampered_plan_aborts_before_reads() {
        let mut device = RecordingTransport::default();
        let mut plan = LightingPlan::prepare(snapshot(&mut device), settings(), None).unwrap();
        plan.config[20] ^= 1;
        let reads_before = device.reads;
        assert!(execute_plan(&mut device, &plan).is_err());
        assert_eq!(device.reads, reads_before);
        assert!(device.writes.is_empty());
    }

    #[test]
    fn disconnect_before_apply_sends_no_writes() {
        let mut device = RecordingTransport::default();
        let plan = LightingPlan::prepare(snapshot(&mut device), settings(), None).unwrap();
        device.mock.set_connected(false);
        assert!(matches!(
            execute_plan(&mut device, &plan),
            Err(HidError::Disconnected)
        ));
        assert!(device.writes.is_empty());
    }

    #[test]
    fn ambiguous_write_failure_does_not_retry_or_rollback() {
        let mut device = RecordingTransport::default();
        let plan = LightingPlan::prepare(snapshot(&mut device), settings(), None).unwrap();
        device.fail_write = true;
        let error = execute_plan(&mut device, &plan).unwrap_err().to_string();
        assert!(error.contains("state may be partially changed"));
        assert_eq!(device.writes, [Region::Config]);
        assert_eq!(
            device.mock.read_region(Region::Config).unwrap(),
            plan.config
        );
    }

    #[test]
    fn readback_mismatch_is_not_reported_as_success() {
        let mut device = RecordingTransport::default();
        let plan = LightingPlan::prepare(snapshot(&mut device), settings(), None).unwrap();
        device.ignore_write = true;
        let error = execute_plan(&mut device, &plan).unwrap_err().to_string();
        assert!(error.contains("readback mismatch"));
        assert_eq!(device.writes, [Region::Config]);
    }

    #[test]
    fn disconnect_after_write_reports_unconfirmed_apply() {
        let mut device = RecordingTransport::default();
        let plan = LightingPlan::prepare(snapshot(&mut device), settings(), None).unwrap();
        device.disconnect_after_write = true;
        let error = execute_plan(&mut device, &plan).unwrap_err().to_string();
        assert!(error.contains("Apply is unconfirmed"));
        assert_eq!(device.writes, [Region::Config]);
    }

    #[test]
    fn unsupported_custom_effect_rejected_before_io() {
        let mut device = RecordingTransport::default();
        let before = snapshot(&mut device);
        let settings = LightingSettings {
            effect: 21,
            ..settings()
        };
        assert!(LightingPlan::prepare(before, settings, None).is_err());
        assert!(device.writes.is_empty());
    }
    #[test]
    fn tampered_palette_aborts_without_device_io() {
        let mut device = RecordingTransport::default();
        let mut plan = LightingPlan::prepare(snapshot(&mut device), settings(), None).unwrap();
        plan.colors[50] ^= 1;
        let reads_before = device.reads;
        assert!(execute_plan(&mut device, &plan).is_err());
        assert_eq!(device.reads, reads_before);
        assert!(device.writes.is_empty());
    }

    #[test]
    fn preflight_timeout_does_not_write_and_next_explicit_attempt_can_recover() {
        let mut device = RecordingTransport::default();
        let plan = LightingPlan::prepare(snapshot(&mut device), settings(), None).unwrap();
        device.mock.fail_next(HidError::Timeout);
        assert!(matches!(
            execute_plan(&mut device, &plan),
            Err(HidError::Timeout)
        ));
        assert!(device.writes.is_empty());
        execute_plan(&mut device, &plan).unwrap();
        assert_eq!(device.writes, [Region::Config]);
    }
    fn color_plan(device: &mut RecordingTransport) -> LightingPlan {
        let settings = LightingSettings {
            colorful: false,
            ..settings()
        };
        LightingPlan::prepare(snapshot(device), settings, Some([0x31, 0x72, 0xb4])).unwrap()
    }

    #[test]
    fn fixed_color_preserves_other_palette_bytes_and_uses_official_write_order() {
        let mut device = RecordingTransport::default();
        let plan = color_plan(&mut device);
        for (i, old) in plan.before.colors.iter().enumerate() {
            if !(147..150).contains(&i) {
                assert_eq!(plan.colors[i], *old, "unexpected palette edit at {i}");
            }
        }
        assert_eq!(&plan.colors[147..150], &[0x31, 0x72, 0xb4]);
        execute_plan(&mut device, &plan).unwrap();
        assert_eq!(device.writes, [Region::Colors, Region::Config]);
    }

    #[test]
    fn palette_only_change_commits_unchanged_config() {
        let mut device = RecordingTransport::default();
        let initial = color_plan(&mut device);
        device
            .mock
            .apply_demo(Region::Config, &initial.config)
            .unwrap();
        let plan = color_plan(&mut device);
        execute_plan(&mut device, &plan).unwrap();
        assert_eq!(device.writes, [Region::Colors, Region::Config]);
    }

    #[test]
    fn second_region_failure_reports_partial_state_without_rollback() {
        let mut device = RecordingTransport::default();
        let plan = color_plan(&mut device);
        device.fail_write_at = Some(2);
        let error = execute_plan(&mut device, &plan).unwrap_err().to_string();
        assert!(error.contains("after 1 completed region writes"));
        assert!(error.contains("state may be partially changed"));
        assert_eq!(device.writes, [Region::Colors, Region::Config]);
        assert_eq!(
            device.mock.read_region(Region::Colors).unwrap(),
            plan.colors
        );
    }

    #[test]
    fn missing_palette_signature_blocks_color_plan() {
        let mut device = RecordingTransport::default();
        let mut before = snapshot(&mut device);
        before.colors[507] = 0;
        let settings = LightingSettings {
            colorful: false,
            ..settings()
        };
        assert!(LightingPlan::prepare(before, settings, Some([1, 2, 3])).is_err());
        assert!(device.writes.is_empty());
    }

    #[test]
    fn off_preserves_every_effect_parameter() {
        let mut device = RecordingTransport::default();
        let before = snapshot(&mut device);
        let off = LightingSettings {
            effect: 0,
            brightness: 9,
            speed: 4,
            colorful: true,
        };
        let plan = LightingPlan::prepare(before.clone(), off, None).unwrap();
        for (i, old) in before.config.iter().enumerate() {
            if ![9, 10].contains(&i) {
                assert_eq!(plan.config[i], *old, "off edited unrelated byte {i}");
            }
        }
    }

    #[test]
    fn noncolor_effect_and_conflicting_color_options_rejected() {
        let mut device = RecordingTransport::default();
        let before = snapshot(&mut device);
        for effect in [0, 3, 15, 16, 17, 21] {
            let settings = LightingSettings {
                effect,
                colorful: false,
                ..settings()
            };
            assert!(LightingPlan::prepare(before.clone(), settings, Some([1, 2, 3])).is_err());
        }
        assert!(LightingPlan::prepare(before, settings(), Some([1, 2, 3])).is_err());
    }
}

#[cfg(test)]
mod sleep_transaction_tests {
    use super::*;
    use crate::{demo_device, MockTransport};
    #[test]
    fn sleep_intent_boundaries_and_tamper() {
        let mut m = MockTransport::default();
        let before = snapshot_from_transport(&mut m, demo_device(), "demo").unwrap();
        let mut p = LightingPlan::prepare_sleep(before, 300).unwrap();
        p.validate().unwrap();
        assert_eq!(p.config[24], 10);
        p.config[3] ^= 1;
        assert!(p.validate().is_err());
    }
}

#[cfg(test)]
mod restore_tests {
    use super::*;
    use crate::{demo_device, MockTransport};
    #[test]
    fn restore_is_bounded_and_requires_physical_snapshot() {
        let mut mock = MockTransport::default();
        let mut saved = snapshot_from_transport(&mut mock, demo_device(), "physical_read").unwrap();
        saved.device.connection_id = "older-connection".into();
        let mut current = saved.clone();
        current.config[10] = 7;
        current.config[70] = 9;
        current.config[24] = 20;
        current.config[3] = 14;
        let p = LightingPlan::prepare_restore(current.clone(), saved.clone()).unwrap();
        assert_eq!(p.config[10], 1);
        assert_eq!(p.config[70], saved.config[70]);
        assert_eq!(p.config[24], saved.config[24]);
        assert_eq!(p.config[3], 14);
        p.validate().unwrap();
        saved.source = "demo".into();
        assert!(LightingPlan::prepare_restore(current, saved).is_err());
    }
}

#[cfg(test)]
mod restore_flag_regression {
    use super::*;
    use crate::{demo_device, MockTransport};
    #[test]
    fn unchanged_physical_snapshot_with_retained_flag_is_noop() {
        let mut m = MockTransport::default();
        let mut saved = snapshot_from_transport(&mut m, demo_device(), "physical_read").unwrap();
        saved.config[9] = 1;
        saved.config[10] = 18;
        let plan = LightingPlan::prepare_restore(saved.clone(), saved).unwrap();
        assert!(!plan.has_changes());
        plan.validate().unwrap();
    }
}

#[cfg(test)]
mod brightness_plan_tests {
    use super::*;
    use crate::{demo_device, MockTransport};
    #[test]
    fn brightness_intent_is_exactly_one_byte_and_reversible_offline() {
        let mut m = MockTransport::default();
        let mut before = snapshot_from_transport(&mut m, demo_device(), "demo").unwrap();
        before.config[9] = 1;
        let old = before.config[58];
        let plan = LightingPlan::prepare_brightness(before.clone(), old - 1).unwrap();
        plan.validate().unwrap();
        assert_eq!(plan.colors, before.colors);
        assert_eq!(
            plan.config
                .iter()
                .zip(&before.config)
                .filter(|(a, b)| a != b)
                .count(),
            1
        );
        let mut expected_after = before.clone();
        expected_after.config = plan.config;
        let restored = LightingPlan::prepare_brightness(expected_after, old).unwrap();
        assert_eq!(restored.config, before.config);
        assert_eq!(restored.colors, before.colors);
        let mut tampered = restored;
        tampered.config[9] ^= 1;
        assert!(tampered.validate().is_err());
    }
}
