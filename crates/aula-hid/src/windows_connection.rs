//! Read-only PnP connection epoch. No registry edits or device-mode changes.
use windows_sys::Win32::Devices::{
    DeviceAndDriverInstallation::{
        CM_Get_DevNode_PropertyW, CM_Get_Device_Interface_PropertyW, CM_Locate_DevNodeW, CR_SUCCESS,
    },
    Properties::{
        DEVPKEY_Device_InstanceId, DEVPKEY_Device_LastArrivalDate, DEVPROP_TYPE_FILETIME,
        DEVPROP_TYPE_STRING,
    },
};

pub(super) fn identity(path: &str) -> Option<String> {
    if path.len() > 4096 || path.contains('\0') {
        return None;
    }
    let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
    let mut instance = [0u16; 512];
    let mut kind = 0;
    let mut bytes = std::mem::size_of_val(&instance) as u32;
    // SAFETY: nul-terminated input, initialized bounded output and valid sizes.
    let result = unsafe {
        CM_Get_Device_Interface_PropertyW(
            wide.as_ptr(),
            &DEVPKEY_Device_InstanceId,
            &mut kind,
            instance.as_mut_ptr().cast(),
            &mut bytes,
            0,
        )
    };
    if result != CR_SUCCESS
        || kind != DEVPROP_TYPE_STRING
        || !(2..=1024).contains(&bytes)
        || bytes & 1 != 0
    {
        return None;
    }
    let end = instance[..bytes as usize / 2]
        .iter()
        .position(|v| *v == 0)?;
    if end == 0 {
        return None;
    }
    let mut node = 0;
    // SAFETY: returned instance is explicitly checked for a nul terminator.
    if unsafe { CM_Locate_DevNodeW(&mut node, instance.as_ptr(), 0) } != CR_SUCCESS {
        return None;
    }
    let mut arrival = [0u8; 8];
    bytes = 8;
    kind = 0;
    // SAFETY: property output is an eight-byte FILETIME buffer, checked below.
    if unsafe {
        CM_Get_DevNode_PropertyW(
            node,
            &DEVPKEY_Device_LastArrivalDate,
            &mut kind,
            arrival.as_mut_ptr(),
            &mut bytes,
            0,
        )
    } != CR_SUCCESS
        || kind != DEVPROP_TYPE_FILETIME
        || bytes != 8
    {
        return None;
    }
    let epoch = u64::from_le_bytes(arrival);
    (epoch != 0).then(|| format!("pnp:{node}:{epoch}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_interfaces_never_become_connection_epochs() {
        assert!(identity("not-a-device").is_none());
        assert!(identity("invalid\0device").is_none());
        assert!(identity(&"x".repeat(4097)).is_none());
    }
}
