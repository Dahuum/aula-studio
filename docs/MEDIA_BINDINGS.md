# Media bindings

Status: **IMPLEMENTED / OFFLINE TESTED**. Physical writes, emitted input events and power-cycle persistence remain unverified. Saving or editing a profile sends no hardware operation.

## Native evidence

The supplied OemDrv.exe SHA256 `1dfe4803efdbe14f66680f970a6d6f26108342fd49754234fa448dd0ebae44a7` constructs fourteen media choices in `0041f730`. Input dispatch `00425dd0` tags the selected internal action as category4. Converter `0049a920` emits the following DWORD values. Their little-endian bytes form the existing four-byte keymap slot; the usage within that slot is big-endian.

| Action | Internal action | Consumer usage (hex) | Native DWORD | Slot bytes |
| --- | --- | --- | --- | --- |
| Media player |21 |0183 |83010002 |02 00 01 83 |
| Play / pause |22 |00CD |CD000002 |02 00 00 CD |
| Mute |28 |00E2 |E2000002 |02 00 00 E2 |
| Stop playback |23 |00B7 |B7000002 |02 00 00 B7 |
| Previous track |24 |00B6 |B6000002 |02 00 00 B6 |
| Next track |25 |00B5 |B5000002 |02 00 00 B5 |
| Volume up |26 |00E9 |E9000002 |02 00 00 E9 |
| Volume down |27 |00EA |EA000002 |02 00 00 EA |
| Browser home |34 |0223 |23020002 |02 00 02 23 |
| Calculator |31 |0192 |92010002 |02 00 01 92 |
| Email reader |30 |018A |8A010002 |02 00 01 8A |
| File browser |32 |0194 |94010002 |02 00 01 94 |
| Display brightness up |29 |006F |6F000002 |02 00 00 6F |
| Display brightness down |2A |0070 |70000002 |02 00 00 70 |

The first byte02 is the device's record type, not the USB usage-page number. These usages belong to Consumer page0C. Usage0183 is named Consumer Control Configuration by USB-IF; the UI's Media player label follows the common desktop interpretation. Usage0194 requests a local-machine browser, and006F/0070 concern display brightness, not keyboard RGB. Meaning and event handling follow [USB-IF HID Usage Tables1.5, section15](https://usb.org/sites/default/files/hut1_5.pdf); desktop application launch and display handling depend on installed applications/compositor shortcuts. No shell commands or host input injection are used.

Primary static Evidence: constructor `ev_6e96c8c337234ccafce5f1ef377491eb00f5cd7c38bea5b0d7f73369fcebc14d`; converter pseudocode `ev_2a6d12577c5a9989a44c6a7f5c3d3838f4a93caf62624b57ebdda9c084d6828e`; converter assembly `ev_4bbef0d5de99508173057e731cb26f319546c7cad59b4e52b212eb188526ddfc`. Actual REA6.7/Ghidra12.1.4 static analysis corroborates earlier converter Evidence `ev_409622321c0d33148c095aefb3de6f5ec29e7e9b3de89c22abe2d5ccf184124f`. Private records are exported/indexed and the session is closed; no original code/assets are redistributed. Confidence is high for application serialization and the advertised list, with **no physical media validation**.

## Editor and profile behavior

In Key Remapping or Fn Layers select a physical source, choose Action type → Media control, select an action, then Assign locally. The keyboard preview shows the assignment. Save profile stores it locally; the layer's separate Review/Write flow is required for hardware. Fn and F1–F12 source controls remain protected. An assignment replaces that source's keyboard assignment in the same layer; the other layer is independent.

Schema1 optional `consumer_keymap` and `consumer_fn_layer` objects store numeric usages. Values now retain all16 bits; existing seven-action profiles serialize identically and require no rewrite/migration. Absent maps default to empty; empty maps are omitted by Rust serialization. Keyboard maps remain limited to their previous1–255 usages. For example:

```json
{
  "keymap": {},
  "fn_layer": {},
  "consumer_keymap": {"4": 205, "5": 547},
  "consumer_fn_layer": {"4": 226, "5": 402}
}
```

This excerpt binds A to play/pause and B to browser home in base, with mute/calculator in Fn; the rest of a valid profile is still required. Unsupported consumer values, values that would truncate to a supported action, keyboard/media conflicts and numeric source aliases are rejected. Older versions cannot import profiles containing these newly supported actions; keep an export before downgrading. Loading an existing profile does not silently overwrite it.

Read-bank projection now retains known16-bit media records, without losing the high byte. Records with unknown usages, nonzero flags, keyboard chords or unsupported pages remain in the raw snapshot and are omitted from editable maps. The existing transaction preserves every other byte/slot/bank, rejects stale state, requires a complete affected-bank backup and verifies exact readback. It adds no new command, report ID, memory address or automatic write. Recovery after an uncertain write still requires a fresh read and the retained backup; no blind retry/reset/rollback is introduced.

## Verification and remaining gaps

Rust exact native-DWORD fixtures cover all fourteen actions and both-layer encode/decode. Mock transactions cover all28 action/layer combinations, full-bank preservation and an unrelated opaque vendor record. Backend tests cover profile roundtrip, read projection, review names and rejection/preservation of malformed or unsupported records. Frontend checks cover16-bit imports, layer independence and review-only transmission boundaries. Native GUI tests exercise the real Rust demo/profile/export/import storage with both old and high-byte actions; demo results do not establish physical compatibility.

Onboard macro binding, arbitrary shortcut editing, additional converter-only consumer actions remain outside this editor. Five normal mouse-button assignments are documented separately in [mouse bindings](MOUSE_BINDINGS.md). No physical configuration write or event injection was performed for this follow-up. Full original action-picker layout remains unfinished.
