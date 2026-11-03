# Media bindings

Status: **IMPLEMENTED / OFFLINE TESTED**, physical writes and emitted input events remain unverified. This adds no automatic hardware operation.

The supplied OemDrv key converter at0x49a920 (REA query0x49a9f0, evidence `ev_409622321c0d33148c095aefb3de6f5ec29e7e9b3de89c22abe2d5ccf184124f`) emits these DWORD constants. Their little-endian bytes form the existing four-byte keymap slot:

| Action | Consumer usage | Slot bytes |
|---|---|---|
| Play / pause | CD |02 00 00 CD |
| Stop playback | B7 |02 00 00 B7 |
| Previous track | B6 |02 00 00 B6 |
| Next track | B5 |02 00 00 B5 |
| Volume up | E9 |02 00 00 E9 |
| Volume down | EA |02 00 00 EA |
| Mute | E2 |02 00 00 E2 |

The first byte02 is the device's record type. It is **not** the USB usage-page number: these actions belong to the Consumer page0C in the [USB-IF HID Usage Tables1.5, section15](https://usb.org/sites/default/files/hut1_5.pdf). Static application evidence establishes serialization, not a successful hardware write or Linux input event.

In Key Remapping or Fn Layers select a physical source, choose Action type → Media control and an action, then Assign locally. The keyboard preview shows the assignment. Save profile stores it locally; the layer's existing Review/Write flow is required for hardware. Fn and F1–F12 source controls remain protected. An assignment replaces that source's keyboard assignment in the same layer. The other layer is independent.

Schema1 adds optional `consumer_keymap` and `consumer_fn_layer` objects. Existing numeric keyboard maps retain their meaning; absent media maps default to empty and empty maps are omitted by Rust serialization. For example:

```json
{
  "keymap": {},
  "fn_layer": {},
  "consumer_keymap": {"4": 205},
  "consumer_fn_layer": {"4": 226}
}
```

This excerpt binds physical A to play/pause in base and mute in Fn. The rest of a valid profile is still required. Conflicting keyboard/media source entries, numeric aliases of the same source and unsupported media values are rejected. Old app versions do not read these new optional fields; retain an export when downgrading.

The read-bank operation returns supported media actions separately and preserves raw records. Protected source controls, shortcuts not expressible in the simple editor, vendor actions and16-bit launch/browser usages stay in the raw snapshot and are not normalized into editable assignments. The existing typed transaction preserves every other slot and bank, checks stale state and full readback, and saves a complete affected-bank backup. No new command/report/address is introduced.

Tests cover exact bytes derived from the native constants, profile compatibility/conflicts, base/Fn independence, selected-record-only writes through mock HID, read-back decoding, frontend review boundaries and native demo/profile storage. Mock and demo tests do not establish physical behavior. Onboard macro binding,16-bit application launch actions and arbitrary shortcuts remain outside this editor.
