# Mouse-button assignments

Status: **IMPLEMENTED / OFFLINE TESTED**. These bind keyboard source keys to normal mouse-button actions; they do not configure an AULA mouse. Physical button output and persistence remain unverified.

## Evidence and exact records

The supplied original F75 OemDrv.exe SHA256 `1dfe4803efdbe14f66680f970a6d6f26108342fd49754234fa448dd0ebae44a7` constructs five mouse choices in `0041f730`, with internal actions11,13,12,14,15. Dispatcher `00425dd0` uses category1. Converter `0049a920` emits these exact DWORD constants; their little-endian bytes are the existing four-byte matrix records.

| Choice | Internal action (hex) | Device button code | Native DWORD | Record bytes |
| --- | --- | --- | --- | --- |
| Left click |11 |1 |00010101 |01 01 01 00 |
| Right click |13 |2 |00010201 |01 02 01 00 |
| Middle click |12 |3 |00010301 |01 03 01 00 |
| Mouse button4 |14 |4 |00010401 |01 04 01 00 |
| Mouse button5 |15 |5 |00010501 |01 05 01 00 |

First byte01 is a device record type, not a USB usage-page number. For this supported subset the next byte holds the button code, and the last two bytes must be exactly01 00. No meaning for other tail combinations is required or exposed. Native converter constants for additional click/repeat forms exist, but this editor neither constructs nor normalizes them. Unknown records stay in the raw backup.

The converter has a context-dependent branch swapping codes4/5 when a context field at offset14 equals1. The field's meaning is unresolved; Linux exposes the explicit numeric device codes and does not guess that setting or call either choice Back/Forward. Button handling and handedness can depend on the desktop/application. No host mouse event or shell command is injected.

Primary static evidence from actual REA6.7/Ghidra12.1.4: control construction `ev_6e96c8c337234ccafce5f1ef377491eb00f5cd7c38bea5b0d7f73369fcebc14d`; converter pseudocode `ev_2a6d12577c5a9989a44c6a7f5c3d3838f4a93caf62624b57ebdda9c084d6828e`; converter assembly `ev_4bbef0d5de99508173057e731cb26f319546c7cad59b4e52b212eb188526ddfc`. Native constants occur at0049a956/0049a964/0049a95d/0049a96b/0049a972 respectively. Original analysis is exported/indexed privately, session closed; target code was never executed. Confidence is high for serialization and normal-choice enumeration, with no physical validation.

## Editor and storage

Choose Key Remapping or Fn Layers, select a physical key, choose Mouse button and an action, then Assign locally. The preview shows Mouse1–Mouse5. Saving the profile is local. A physical mutation still requires the separate existing backed-up layer review and explicit Write. Fn/F1–F12 sources stay protected. Mouse assignments replace a keyboard/media assignment only for the same source and layer; clearing a selected draft or whole layer clears all represented action types in that scope.

Schema1 adds optional `mouse_keymap` and `mouse_fn_layer` objects. Empty maps are omitted; older profiles need no rewrite and retain their previous serialization. For example, `"mouse_keymap":{"6":1}` binds physical C to normal button1 in the local base draft. Keyboard, media and mouse sources cannot overlap within one layer, including numeric aliases such as6/06. Only integer button codes1–5 are accepted. Older app versions cannot import the new fields; retain exports before downgrading.

Reading a bank exposes only canonical normal-button records in `mouse_assignments`. Unknown buttons or noncanonical tail bytes remain raw and do not become editable actions. Reviewed descriptions name known mouse records. The existing03/83 matrix path, exact device/revision gates, scoped backups, locking, stale checks and exact readback are reused. No new HID command, report, address, host-input API or automatic write is added. An uncertain write requires a fresh read and retained backup, with no blind retry/reset/rollback.

## Verification and gaps

Exact native fixtures cover five records in both layers. Combined media/mouse mock tests exercise38 action/layer combinations and compare the full affected bank, preserving an unrelated opaque vendor record and the other layer. Tests reject invalid buttons, numeric aliases and cross-type conflicts; read projection leaves repeat/flag variants untouched. Frontend tests verify mouse assignments replace the correct local keyboard source, retain other assignments/layers and reach the native prepare endpoint only on explicit review. Native demo/profile/export/import checks exercise actual Rust storage without physical writes.

Physical mouse events, application behavior, power-cycle persistence and hardware restoration of mouse bindings remain unverified. Repeat/double-click, wheel, pointer movement, mouse DPI/polling and OS-specific swapped-button settings are not implemented. This work does not establish support for other AULA devices or wireless configuration.
