//! Read-only model adapters. No configuration setter or arbitrary packet API.
use crate::{HidError, Result};
use aula_models::g5_capture::{Capture, CaptureSource, ReadRequest};

pub trait ModelReadTransport {
    fn read(&mut self, request: ReadRequest) -> Result<Vec<u8>>;
}

/// Reject unsupported/foreign models before any bank operation. A failed or
/// malformed read yields no accepted capture, and never triggers a retry/write.
pub fn capture_from_transport(
    transport: &mut impl ModelReadTransport,
    model_id: &str,
    source: CaptureSource,
) -> Result<Capture> {
    let expected =
        aula_models::g5_capture::expected_identity(model_id).map_err(HidError::Unsupported)?;
    let bytes = transport.read(ReadRequest::Identity)?;
    let actual = aula_models::g5::decode_identity(&bytes)?;
    if actual != expected {
        return Err(HidError::Unsupported(
            "Selected model identity differs from this keyboard; no configuration banks were queried"
                .into(),
        ));
    }
    let mut reports = Vec::with_capacity(4);
    for bank in [
        ReadRequest::Config,
        ReadRequest::Palette,
        ReadRequest::Base,
        aula_models::g5_capture::per_key_request(model_id).map_err(HidError::Unsupported)?,
    ] {
        let report = transport.read(bank)?;
        bank.payload_for_model(model_id, &report)
            .map_err(HidError::Unsupported)?;
        reports.push(report);
    }
    Capture::decode(
        model_id,
        actual,
        source,
        reports.try_into().expect("exactly four bounded banks"),
    )
    .map_err(HidError::Unsupported)
}

/// Synthetic fixture transport, never factory defaults or physical-device data.
/// The request set is the same typed read-only set used by the actual adapter.
pub struct SimulatedCaptureTransport {
    identity: [u8; 6],
    model_id: String,
}
impl SimulatedCaptureTransport {
    pub fn new(model_id: &str) -> Result<Self> {
        Ok(Self {
            model_id: model_id.into(),
            identity: aula_models::g5_capture::expected_identity(model_id)
                .map_err(HidError::Unsupported)?,
        })
    }
}
impl ModelReadTransport for SimulatedCaptureTransport {
    fn read(&mut self, request: ReadRequest) -> Result<Vec<u8>> {
        let mut bytes = request.bytes();
        match request {
            ReadRequest::Identity => bytes[8..14].copy_from_slice(&self.identity),
            ReadRequest::Config => {
                bytes[8 + 10] = 2;
                bytes[8 + 56..8 + 58].copy_from_slice(&[0xff, 0xff]);
                bytes[8 + 60] = 2;
                bytes[8 + 61] = 0x20;
                if !aula_models::g5_capture::LEGACY_MODELS.contains(&self.model_id.as_str()) {
                    bytes[8 + 126..8 + 128].copy_from_slice(&[0x5a, 0xa5]);
                }
            }
            ReadRequest::Palette => {
                bytes[8 + 42..8 + 45].copy_from_slice(&[32, 64, 128]);
                bytes[8 + 506..8 + 508].copy_from_slice(&[0x5a, 0xa5]);
            }
            ReadRequest::Base => bytes[8 + 510..8 + 512].copy_from_slice(&[0x5a, 0xa5]),
            ReadRequest::PerKeyLegacy384 | ReadRequest::PerKeyStandard378 => {
                for i in 0..126 {
                    bytes[8 + i] = (i * 2) as u8;
                    bytes[8 + 126 + i] = (255 - i) as u8;
                    bytes[8 + 252 + i] = (i * 3) as u8;
                }
                // Synthetic unknown tail proves captures preserve these bytes;
                // these are never claimed factory/default/LED values.
                if request == ReadRequest::PerKeyLegacy384 {
                    bytes[8 + 378..8 + 384].copy_from_slice(&[0xde, 0xad, 0xbe, 0xef, 0x12, 0x34]);
                }
            }
        }
        Ok(bytes.to_vec())
    }
}
pub fn simulate_capture(model_id: &str) -> Result<Capture> {
    capture_from_transport(
        &mut SimulatedCaptureTransport::new(model_id)?,
        model_id,
        CaptureSource::Simulated,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Spy {
        inner: SimulatedCaptureTransport,
        seen: Vec<ReadRequest>,
        fail: Option<ReadRequest>,
        malformed: Option<ReadRequest>,
        empty: Option<ReadRequest>,
    }
    impl Spy {
        fn new() -> Self {
            Self {
                inner: SimulatedCaptureTransport::new("f87").unwrap(),
                seen: vec![],
                fail: None,
                malformed: None,
                empty: None,
            }
        }
    }
    impl ModelReadTransport for Spy {
        fn read(&mut self, request: ReadRequest) -> Result<Vec<u8>> {
            self.seen.push(request);
            if self.fail == Some(request) {
                return Err(HidError::Disconnected);
            }
            let mut r = self.inner.read(request)?;
            if self.malformed == Some(request) {
                r[5] = 1;
            }
            if self.empty == Some(request) {
                r[8..].fill(0);
            }
            Ok(r)
        }
    }
    #[test]
    fn foreign_identity_stops_after_the_bounded_identity_read() {
        let mut spy = Spy::new();
        spy.inner.identity = [3, 0, 0, 0, 0, 0xcd];
        assert!(capture_from_transport(&mut spy, "f87", CaptureSource::Simulated).is_err());
        assert_eq!(spy.seen, vec![ReadRequest::Identity]);
        spy.seen.clear();
        assert!(capture_from_transport(&mut spy, "s75-pro", CaptureSource::Simulated).is_err());
        assert!(spy.seen.is_empty());
    }
    #[test]
    fn exact_sequence_has_only_read_selectors_and_model_specific_lengths() {
        let mut spy = Spy::new();
        let c = capture_from_transport(&mut spy, "f87", CaptureSource::Simulated).unwrap();
        assert_eq!(
            spy.seen,
            vec![
                ReadRequest::Identity,
                ReadRequest::Config,
                ReadRequest::Palette,
                ReadRequest::Base,
                ReadRequest::PerKeyLegacy384
            ]
        );
        for r in &spy.seen {
            assert_eq!(r.bytes()[1] & 0x80, 0x80);
        }
        assert_eq!(c.source, CaptureSource::Simulated);
        assert_eq!(c.per_key.len(), 384);
        assert_eq!(&ReadRequest::PerKeyLegacy384.bytes()[6..8], &[0x80, 1]);
    }
    #[test]
    fn failure_or_wrong_scope_aborts_without_further_io_or_partial_capture() {
        for malformed in [false, true] {
            let mut spy = Spy::new();
            if malformed {
                spy.malformed = Some(ReadRequest::Palette)
            } else {
                spy.fail = Some(ReadRequest::Palette)
            };
            assert!(capture_from_transport(&mut spy, "f87", CaptureSource::Simulated).is_err());
            assert_eq!(spy.seen.last(), Some(&ReadRequest::Palette));
            assert_eq!(spy.seen.len(), 3);
        }
    }
    #[test]
    fn untouched_echo_is_rejected_before_any_remaining_bank_is_queried() {
        let mut spy = Spy::new();
        spy.empty = Some(ReadRequest::Config);
        assert!(capture_from_transport(&mut spy, "f87", CaptureSource::Simulated).is_err());
        assert_eq!(spy.seen, vec![ReadRequest::Identity, ReadRequest::Config]);
    }
    #[test]
    fn simulated_projection_uses_native126_plane_strides_and_preserves384_raw_bytes() {
        let c = simulate_capture("f87").unwrap();
        let p = c.lighting_draft("Read lighting intent").unwrap();
        let a = aula_models::model("f87")
            .unwrap()
            .keys
            .iter()
            .find(|k| k.usage == Some(4))
            .unwrap();
        assert_eq!(p.per_key[&a.id], [18, 246, 27]);
        assert_eq!(p.lighting.unwrap().color, [32, 64, 128]);
        assert!(p.assignments.is_empty());
        assert!(p.fn_assignments.is_empty());
        assert!(p.macros.is_empty());
        assert_eq!(c.base.len(), 512);
        assert_eq!(&c.per_key[378..], &[0xde, 0xad, 0xbe, 0xef, 0x12, 0x34]);
    }
    #[test]
    fn every_variant_has_its_own_identity_and_cannot_cross_project() {
        for id in aula_models::g5_capture::MODELS {
            let c = simulate_capture(id).unwrap();
            assert_eq!(c.model_id, *id);
            assert_eq!(
                c.identity,
                aula_models::g5_capture::expected_identity(id).unwrap()
            );
        }
        let mut c = simulate_capture("f87").unwrap();
        c.model_id = "f87-pro-wired".into();
        assert!(c.validate().is_err());
    }
    #[test]
    fn standard_models_use378_bytes_and_require_their_config_tail_before_further_io() {
        for model_id in ["f108", "h108"] {
            let mut spy = Spy::new();
            spy.inner = SimulatedCaptureTransport::new(model_id).unwrap();
            let c = capture_from_transport(&mut spy, model_id, CaptureSource::Simulated).unwrap();
            assert_eq!(c.per_key.len(), 378);
            assert_eq!(&c.config[126..], &[0x5a, 0xa5]);
            assert_eq!(spy.seen.last(), Some(&ReadRequest::PerKeyStandard378));
            assert_eq!(&ReadRequest::PerKeyStandard378.bytes()[6..8], &[0x7a, 1]);
            let draft = c.lighting_draft("Standard lighting").unwrap();
            let key = aula_models::model(model_id)
                .unwrap()
                .keys
                .iter()
                .find(|k| k.usage == Some(4))
                .unwrap();
            let index = key.light_index.unwrap() as usize;
            assert_eq!(
                draft.per_key[&key.id],
                [(index * 2) as u8, (255 - index) as u8, (index * 3) as u8]
            );
            assert!(
                draft.assignments.is_empty()
                    && draft.fn_assignments.is_empty()
                    && draft.macros.is_empty()
            );
            // Keep the FF sentinel while removing only the standard config tail.
            struct MissingTail(Spy);
            impl ModelReadTransport for MissingTail {
                fn read(&mut self, r: ReadRequest) -> Result<Vec<u8>> {
                    let mut b = self.0.read(r)?;
                    if r == ReadRequest::Config {
                        b[8 + 126..8 + 128].fill(0);
                    }
                    Ok(b)
                }
            }
            let mut transport = MissingTail(Spy::new());
            transport.0.inner = SimulatedCaptureTransport::new(model_id).unwrap();
            assert!(
                capture_from_transport(&mut transport, model_id, CaptureSource::Simulated).is_err()
            );
            assert_eq!(
                transport.0.seen,
                vec![ReadRequest::Identity, ReadRequest::Config]
            );
            assert!(ReadRequest::PerKeyLegacy384
                .payload_for_model(model_id, &ReadRequest::PerKeyLegacy384.bytes())
                .is_err());
            assert!(ReadRequest::PerKeyStandard378
                .payload_for_model(model_id, &ReadRequest::PerKeyLegacy384.bytes())
                .is_err());
        }
    }
    #[test]
    fn standard_models_do_not_accept_each_others_identity() {
        let mut spy = Spy::new();
        spy.inner = SimulatedCaptureTransport::new("f108").unwrap();
        assert!(capture_from_transport(&mut spy, "h108", CaptureSource::Simulated).is_err());
        assert_eq!(spy.seen, vec![ReadRequest::Identity]);
    }
}
