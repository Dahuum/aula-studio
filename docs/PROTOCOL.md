# F75 protocol — evidence revision2

Sources: supplied20260514 native Windows build, retained REA/Ghidra evidence, public source/captures and local wired reads. These findings are revision-specific. See REVERSE_ENGINEERING.md for addresses/Evidence IDs; never infer a physical write test from an encoder or mock result.

## Identity and framing

Wired258a:010c, interface1; descriptor must declare feature ID06 with519 data bytes plus report-ID byte =520. Receiver3554:fa09 is enumeration-only. Fw24 in KB.ini selects an application protocol family, not a measured device firmware version. Supplied package uses standard SubFw0 branch.

| Byte | Field |
|---|---|
|0|Report ID06|
|1|Command; bit7 selects read|
|2|Target:0 config/palette/per-key,0/base or1/Fn keymap|
|3|Reserved0|
|4|Total chunks,1 for supported operations|
|5|Chunk index,0 for supported operations|
|6..7|Payload length, unsigned little endian|
|8..|Payload then padding to520|

The bytes often called an address (`00 00 01 00`) are target/chunk fields in official engine0x4993c0. The app exposes no arbitrary address or command API. Reads send SetFeature with the documented read selector, wait20ms, then GetFeature ID06. Writes use typed buffers and wait70ms before subsequent IO. No invented additive checksum is used for wired report06.

| Bank | Write/read | Header example | Payload | Signature |
|---|---|---|---|---|
|Config|04/84|06 84 00 00 01 00 80 00|128B|payload126/127=5a a5|
|Effect palette|0a/8a|06 8a 00 00 01 00 00 02|512B|payload506/507=5a a5, followed by4 bytes|
|Base keymap|03/83|06 83 00 00 01 00 00 02|512B|fullreport518/519=5a a5|
|Fn keymap|03/83|06 83 01 00 01 00 00 02|512B|same|
|Per-key group0|06/86|06 86 00 00 01 00 7a 01|378B|no appended signature|
|Macros|05/85|variable/chunked|not exposed in transport|allocation/semantics incomplete|

Config/palette/base/Fn/per-key **reads are locally verified on hardware**. Read replies accept only expected ID/command/target/chunks/length and signature, then normalize keymap read83 to editable03 form. Config/per-key allow exact header+payload or520-byte replies; keymaps require520.

## Standard configuration

Selected hardware effect at payload10. Official standard fill writes custom flag9 true for effect21; local reads may retain9=1 alongside a normal preset, so that relationship is not a universal read-state invariant. Payload11 is custom group selector StartGp+group (package default32/group0), unrelated to fixed/random preset colors. Normal preset changes preserve11. Off only changes selector/custom flag, not nonexistent off parameters.

Effect IDs0(off),1..18,21(custom). Parameters for effect1..18: brightness at56+2×id, packed speed/random at57+2×id. Brightness0..9; speed0..4 in high nibble. Random/multicolor low nibble7, fixed0. Resource LightHW/SpeedHW mappings are identity. Speed is not adjustable for effect1 and is preserved. Color flags are unavailable for3/15/16/17 and are preserved. Direction flags are all zero. Alternative SubFw configuration branches are not supported.

Sleep is payload24 with official30-second units. Supported seconds `[0,60,90,120,180,240,300,600,900,1200]`; zero disables. Other settings/unknown bytes are preserved; no debounce, polling or OS-mode write is inferred.

## Effect colors

The512-byte0a palette is not a per-key table. Seven RGB triples per hardware effect; first RGB at21×id. Only the first RGB of a color-capable effect is changed; all other colors, padding and signature are preserved. Signature506/507 is corroborated by native stack offsets49c98d/49c995 and all local palette reads. The initial510/511 assumption was rejected during native prepare QA and corrected before any physical write.

## Per-key RGB and ANSI indexing

378B =126 red +126 green +126 blue bytes. LED index is the matrix index for supplied8-field key definitions; optional ninth-field overrides exist on other layouts. `matrix_index_for_usage` maps79 original ANSI HID usages; Fn is reserved matrix53 and has no ordinary HID usage. ANSI backslash=80, Enter=81. Public ANSI mapping, official FR common positions and local base-map reads corroborate the table; visible LED correspondence after a write remains unverified. Geometry is ANSI80 keys plus knob; ISO is not silently substituted.

Per-key transactions read/preserve the complete bank, change only requested known indices, write06 before activating custom mode through04, then read back both. They do not send the palette as per-key data.

## Key mappings

Four-byte records from fullreport8,128 slots with final signature occupying the end. Indices0..126 only; index127 overlaps trailer and is rejected. Keyboard records `[00,modifier_mask,usage_hi,usage_lo]` for the supported subset; ordinary keyboard usage is final byte. Modifier targetsE0..E7 encode a single bit in byte1 with remaining bytes zero. Local Ctrl/Shift/Alt/Super records and native keyinfo conversion corroborate this; Fn53 is vendor action13 and remains protected; physical RightCtrl59 uses modifier mask10. Media records use type byte02 (the USB Consumer usage page is0C); broader16-bit consumer/vendor/macro actions exist and are preserved raw, but are not all represented by the current profile editor.

A plan patches only explicit source keys in one read layer. Fn/F1–F12 source edits and disabling usages are rejected; unchanged protected assignments are retained. Reading into a draft omits unknown vendor actions and retains raw bank backups. No F87 factory fixture is written to an F75.

## Transaction and recovery contract

Preparation is read-only: validate intent, read complete baseline/affected bank and save0600 backup. A human reviews device, scope, changes and status. Commit consumes a random token, validates the durable backup and device connection epoch, locks across app instances, rereads the baseline and rejects stale state without writes. Only changed banks and their documented configuration commits are written. Palette precedes config; per-key bank precedes its config commit/custom activation. The unchanged config record is still committed after a changed palette/per-key bank, matching the official sequence rather than assuming standalone bank activation. Exact readback of config/palette/affected bank is required for success.

Timeout5s; lock remains held while outstanding kernel IO completes. Deadline checks prevent later requests after expiry. No automatic retries or rollback. Failures/partial writes are explicitly unconfirmed, with retained backup/journal. Power-cycle persistence is unknown. GUI actions are implemented/offline tested; physical writes remain unvalidated locally. AULA_DISABLE_HARDWARE_WRITES blocks physical commit entry points during QA.

Scoped restoration restores supported preset records, adjustable first colors and sleep from a matching physical snapshot. Unknown bytes/other banks remain unchanged. Custom-mode snapshots and full keymap/per-key restoration require separate bank-aware handling; this is not a factory reset.

## Macros and wireless

Official macro conversion0x49b000: byte UTF16 name length (max30 code units), then4-byte events. First event byte contains type high bits,20-bit timing's high nibble and action-zero flag0x80; then timing middle/low bytes and event value. Pool directory is `[offset_le16,size_le16]` per record followed by contiguous records. Raw codec exposes timing values and key-transition interpretation (flag80 means release in the official application); its4096B bound is the supplied app default, not measured device capacity. No local-ms conversion, read85, binding write or upload is exposed. Delay placement across editors, repeat semantics, capacity, interrupted-write recovery and persistence remain unknown.

Wireless has an offline20-byte report13 additive-checksum lead, not a verified configuration implementation. No receiver configuration or arbitrary probing is performed. Do not mix unrelated report09 device-family checksum engines into this protocol.

## Bank recovery and Fn RGB

Per-key profile key`fn` maps internally to the documented Fn LED index53; it is not HID usage0 and cannot become an input remapping source. All80 physical key LEDs can be edited, including Fn. The raw per-key bank still contains126 entries, with unknown/unused entries preserved.

Bank restoration copies only known ANSI source/LED entries from an app-created physical backup. Keymap layer and protected Fn/F-control records must match; changed protected controls cause rejection. Empty base-key records are rejected to avoid disabling input. Other rows and current config/lighting mode remain unchanged. Complete affected-bank readback confirms the bounded restore. Stored raw backup records preserve original modifiers, vendor actions and combinations; loading a bank into the ordinary editor omits combinations the current profile cannot represent rather than stripping their modifiers.

## Media assignment subset

Record type2 encodes a Consumer-page action; it is a device tag, not USB usage page2. Seven native converter constants are exposed:CD/B7/B6/B5/E9/EA/E2, encoded `[02,00,00,usage]`. See MEDIA_BINDINGS.md for native evidence, exact fixtures and the schema1 optional consumer maps. Both layers use the existing03 transaction; source controls stay protected. Read-bank decoding exposes supported media actions separately and leaves other records raw. Native demo/profile tests and mocked transport pass; no physical media write/event is verified.


###0.4 ANSI role correction

Saved physical base fullreport220..224 (slot53) is0D000000, identifying Fn;244..248 (slot59) is00100000, identifying RightCtrl. These differ from the supplied French RightAlt53/Fn59 labels. Fresh base-bank marker validation is required before matrix reads or writes; mismatches are rejected. Old0.1–0.3 mapping/per-key artifacts must not be used. Fn LED53 follows matrix evidence, but visible per-key correspondence still awaits a physical test.
