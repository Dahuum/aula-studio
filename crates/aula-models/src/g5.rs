//! Read-only model identity framing recovered from F87 OemDrv.exe0x489a20.
//! Encoding never transmits. No firmware, pairing or identity-write operation.
use aula_protocol::{ProtocolError, REPORT_SIZE};

pub fn identity_request() -> [u8; REPORT_SIZE] {
    let mut b = [0; REPORT_SIZE];
    b[..8].copy_from_slice(&[6, 0x82, 1, 0, 1, 0, 6, 0]);
    b
}
pub fn decode_identity(bytes: &[u8]) -> Result<[u8; 6], ProtocolError> {
    if bytes.len() != REPORT_SIZE && bytes.len() != 14 {
        return Err(ProtocolError::Length {
            expected: 14,
            actual: bytes.len(),
        });
    }
    if bytes[..8] != identity_request()[..8] {
        return Err(ProtocolError::Header);
    }
    Ok(bytes[8..14]
        .try_into()
        .expect("validated six-byte model identity"))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_bounded_identity_packet() {
        let b = identity_request();
        assert_eq!(&b[..8], &[6, 0x82, 1, 0, 1, 0, 6, 0]);
        assert!(b[8..].iter().all(|v| *v == 0));
    }
    #[test]
    fn identity_decoder_rejects_wrong_scope_and_length() {
        let mut b = identity_request();
        b[8..14].copy_from_slice(&[3, 0, 0, 0, 0, 0x8f]);
        assert_eq!(decode_identity(&b).unwrap(), [3, 0, 0, 0, 0, 0x8f]);
        assert!(decode_identity(&b[..13]).is_err());
        b[2] = 0;
        assert!(decode_identity(&b).is_err());
    }
}
