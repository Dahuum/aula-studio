# Multi-model research record

This record describes the official catalog captured on **2026-11-03** and static findings from selected Windows artifacts. It does not establish hardware compatibility for additional models. No Windows application, installer, firmware helper or bundle bootstrap was executed. No device communication was performed during this investigation.

The descriptions below are independently written observations. Vendor executables, extracted JavaScript, resources, images and decompiler output remain in ignored `analysis/`; they are not included in source or release packages. REA evidence identifiers refer to the privately retained analysis records. Addresses apply only to the identified executable hash.

## Catalog acquisition

The source was the [official AULA download page](https://www.aulagaming.com/pages/download), captured at `2026-11-03T11:09:58.865130+00:00`. Its HTML SHA256 is `fa39a4e15a0a6a73ef15c2eb5f240c6a873e068ac70659d4996d02720433e31e`.

| Classification | Entries | Result |
| --- | ---: | --- |
| Direct Windows configuration packages | 12 | Downloaded privately: eight keyboard packages, three mouse packages, one Hub ZIP |
| Browser configurators | 11 | Excluded from the requested direct-download scope |
| Hosted file-service pages | 6 | Excluded; five distinct URLs because two F99 cards share one link |
| Standalone firmware packages | 2 | Excluded |

The 12 downloads total **223,810,740 bytes**. Acquisition used HTTPS, reviewed destination hosts, size/time bounds, atomic temporary files, SHA256 manifests and non-executable file permissions. Extraction preflight bounded member counts, paths and expansion. Ten Inno packages were extracted; Hub was recovered through ZIP → NSIS → inner archive → Electron ASAR. F2088 was initially unrecognized by the available Inno/7z extraction paths. Unreleased0.6 subsequently recovered its Advanced Installer containers by correcting their first512-byte complement, extracted them without execution and independently traced its eight-byte normal lighting protocol; see [F2088 follow-up](F2088_RESEARCH.md). Bundled firmware tools were indexed, never installed or run.

Exact links, package hashes and exclusion reasons are recorded in [DOWNLOAD_CATALOG.md](DOWNLOAD_CATALOG.md) and [download-catalog.json](../data/download-catalog.json). This is one captured catalog, not an inventory of every AULA product ever sold. The F75 Pro hosted filename resembles the supplied F75 installer name; this is catalog ambiguity, not Pro compatibility evidence.

Reproduction helpers are [research_catalog.py](../scripts/research_catalog.py) and [research_extract.py](../scripts/research_extract.py). Downloading is opt-in. Proprietary artifacts are not copied into public metadata.

## Evidence and method

Native analysis used **REA 6.7.0 MCP stdio with Ghidra 12.1.4**, after Ghidra's default static analysis completed. PE comparisons, resource/configuration inspection and assembly corroborated decompiler results. The original artifacts were immutable inputs; temporary Ghidra databases were closed after evidence export.

| Target | Extracted executable SHA256 | Scope |
| --- | --- | --- |
| F87 package, `OemDrv.exe` | `c46e6cad5ad85addbaef4975d27721719aa567bcb27582312b797a7108ec5a38` | Model selection, feature access, configuration construction and apply orchestration |
| S75 Pro, `DeviceDriver.exe` | `e92e83b8466fa35bf370f8bc1092f2a851ec303c41893e62d96073ba4d620e3e` | Feature transport, lighting, keymap/Fn and macro bank construction |
| F108 Pro, `DeviceDriver.exe` | `446b362b2820844a2c670342208bb1e09500a38b4d69435ea2be8b3bff6179b6` | Configuration inspection and bounded PE byte comparisons only |

Decompiler pseudocode is not original source and can omit live instructions after incorrectly modeled imports. In particular, S75's per-key packing loop was removed as “unreachable” in pseudocode; the instruction listing preserves it. Recovered C prototypes and return types also require care.

## Shared USB identifiers require model gates

Configuration metadata for F75, F87, F87 Pro and a wired F87 Pro revision shares `258A:010C`. F87's native selector additionally examines HID capabilities and a six-byte identity called `Psd`; VID/PID alone is insufficient.

| Resource identity | Six bytes |
| --- | --- |
| F75 resource | `03 00 00 00 00 CD` |
| F87 resource | `03 00 00 00 00 8F` |
| F87 Pro resource | `03 00 00 00 01 0B` |
| F87 Pro wired resource | `03 00 00 00 00 A9` |

These values are resource/native selection evidence, not a complete revision database. A model name or matching identity does not establish that every operation is safe for that revision.

S75 Pro and F108 Pro present another collision: both configurations name wired `0C45:800A`, interface `MI_00`. Their configured receivers differ: S75 `0C45:FEFE`, F108 Pro `05AC:024F`, both `MI_03`. Their executables differ, including `.text`; sampled S75 protocol-address byte ranges are not identical in the F108 image. No corresponding F108 function analysis or hardware comparison establishes protocol equivalence.

## F87 native findings

`FUN_00488650` selects the wired branch for the internal firmware selector `0x18` (a software selector, not a measured firmware version). It checks vendor usage page `FF00`, usage `0001`, and feature report size `0208` (520 bytes), then compares the six identity bytes against the configured model.

`FUN_00489A20` constructs a bounded identity read. For that wired selector, the full feature report begins:

```text
06 82 01 00 01 00 06 00 [512 zero data bytes]
```

It sends a 520-byte `HidD_SetFeature` request followed by `HidD_GetFeature`, then consumes six bytes at full-report offsets `8..13`. Sending a feature request is still device I/O; this finding does not authorize arbitrary probing or establish local hardware verification for F87. The alternative selector uses a different report ID; it is not automatically a wireless implementation.

`FUN_00488A20` shows the generic access header:

| Full-report offset | Observed meaning |
| ---: | --- |
| 0 | Report ID: `06` for the `0x18` branch |
| 1 | Command byte supplied by caller |
| 2 | Selector supplied by caller |
| 3 | Zero |
| 4 | Total chunk count |
| 5 | Zero-based chunk index |
| 6–7 | Current payload length, little endian, at most 512 |
| 8–519 | Data / padding |

The routine sends request headers for reads as well as writes; its read branch performs a following GET_FEATURE. It splits data into at most 512-byte chunks. No computed checksum is evident in this inspected wrapper; this is not a statement about every operation in the device family.

`FUN_0048ABF0` constructs a 128-byte configuration buffer from model-specific resource fields. `FUN_0048AFA0` orchestrates loading configuration before modification, setting matrix layers, macro data and lighting. Its matrix call uses four bytes per matrix position and considers up to four layers with model-dependent masks. Its macro path has separate buffers and model-dependent sizes. These observations support continued reconstruction; they do not establish a complete F87 keymap, macro or persistence contract.

| Finding | REA evidence ID |
| --- | --- |
| Selector / identity comparison, `00488650` | `ev_283428da9c9dd4bb847e9743e03e358eff86bbe6f3d6443ccecdeecb9940f6e0` |
| Identity request, `00489A20` | `ev_c1abc94b4922c6fbc395d979eecac000518a972cdc487705b7c3a1f61a037c36` |
| Generic feature access, `00488A20` | `ev_b8c3b9f2a1da8204289bbac81f63d8184600a045bfead48ab193317cebf7feac` |
| Configuration construction, `0048ABF0` | `ev_115e8d6643c69718c66613f1cd1886305479d7e5d91f154d4528cb2227612b16` |
| Apply orchestration, `0048AFA0` | `ev_2ffeb4eddef2bf45ee6f41b691ad37116369711f4289d9f6bc3f78d9f7ee21b6` |

## S75 Pro native findings

All offsets in this section are **payload offsets**. Add one for the full HID report offset.

### Transport

`FUN_00451350` calls the dynamically resolved `HidD_SetFeature` with exactly **65 bytes**. `FUN_0044EDB0` prepends a zero byte to 64 payload bytes: **report ID `00`, payload length 64**. This is a different framing family from F75/F87's report `06` and 520-byte transfers.

`FUN_004513F0` receives feature data through overlapped `DeviceIoControl(0x000B0192)` followed by `GetOverlappedResult(..., TRUE)`. That IOCTL is GET_FEATURE, not ReadFile. Its observed return count is 64 payload bytes. Native waits and retry behavior are not a suitable Linux timeout design without adaptation.

The generic send routine may request a response but does not itself validate an acknowledgment. Long sends split into whole 64-byte pages and ignore any remainder; Linux encoders reject incorrect lengths instead. No computed checksum was found in these inspected feature constructors. `AA 55` below is a constant marker.

### Scalar lighting

`FUN_0042AFD0` constructs this sequence:

```text
04 18                 observed preamble
04 13, payload[8]=1   select one body page
[64-byte body]
04 02                 observed end/commit phase
04 F0                 observed follow-up; purpose unresolved
```

“Commit” describes its position in the transaction; it does not prove flash persistence. The routine's body fields are:

| Payload offset | Field |
| ---: | --- |
| 0 | Lighting mode |
| 1–3 | Low, middle, high bytes of stored color DWORD |
| 8 | `colorful` parameter |
| 9 | Brightness |
| 10 | Speed |
| 11 | Direction |
| 14–15 | Constant `AA 55` |

Parameter names are corroborated by the SQLite `t_light_data` schema and `FUN_00408520`'s column-to-structure assignments. Mode zero leaves color/parameters zero. The shipped S75 layout specifies brightness/speed maxima of five. Physical RGB versus BGR interpretation, legality of all effect/parameter combinations and direction enumeration remain unverified.

### Per-key lighting and readback uncertainty

`FUN_00427F60` selects `04 20` with payload byte 8 equal to eight, sends **512 bytes / eight pages**, then `04 02`. Records are packed sequentially as four bytes: **index, raw channel 0, raw channel 1, raw channel 2**. The index is obtained from the layout map; it is not necessarily the record's position. Native code first scales UI color components by a per-key factor divided by 256.

Assembly locations `004280E1`, `00428120`, `0042813B`, `0042815C` show the index/channel stores; `00428164..0042816D` advances record pointers by four. This is the loop omitted by the decompiler.

`FUN_0042ADB0` instead requests `04 F5`, payload byte 8 equal to **nine**, through `FUN_0044F280`; read results can contain **576 bytes**. The helper initially receives an unexpected length argument of nine or three, retries with 65 after failure, and only enters collection on that failure/retry path. It then bounds collection to nine pages, checks 64-byte return counts and stops after a call exceeding 360 ms.

The **nine-page read / eight-page write mismatch is unresolved**, as is the unusual initial-length/control-flow behavior. There is no verified complete backup/restore transaction for this family. An offline read-request constructor is not a claim that physical readback has been validated.

### Keymap, Fn and macro evidence

`FUN_00418550` selects `04 11` for the base layer or `04 27` for Fn, with payload byte 8 equal to nine. It sends a 576-byte bank, uses four-byte records indexed by the layout's key index, places `AA 55` at bank offsets `574..575`, then sends `04 02` and `04 F0`. Ordinary key records use type `02`; consumer and macro bindings use distinct record types. Fn reservations and model defaults affect which records are emitted. Replacing an entire bank with incomplete defaults could disable keys; no complete S75 layer-upload workflow is exposed.

`FUN_0042D5D0` provides substantial onboard macro evidence: `04 19`, then `04 15` with a page count, a paged bank, and `04 02`. The bank begins with a 400-byte pointer area; macro data starts at offset 400, has an eight-byte header and four-byte event records. Event-count fields and data offsets are little endian. Empty pointer entries are `FF FF FF FF`; the final bank page ends in `AA 55`. Native sizing adds a page after the rounded data end.

Observed internal action types 2/3 emit event tags `B0`/`30`; types 4/5 emit `90`/`10` with masks `1`, `4` or `2`. Delay records use tag `50` and a little-endian value clamped to at least 10. The recorder-to-action semantic link, time units, firmware limits, slot stability, readback and persistence remain incomplete. These findings do **not** constitute working macro upload support. `FUN_00451480` independently exposes Windows-key-to-USB-usage conversion used by macro serialization.

| Finding | REA evidence ID |
| --- | --- |
| SetFeature length, `00451350` | `ev_84462bb300bc3afaf274643522ecc088f064a56ddd5239e567c3378b21bdae4f` |
| GET_FEATURE wrapper, `004513F0` | `ev_25b88477bec32a094dd5aba3587f93c6a45187f06983f07a05fe17c18c0a29df` |
| Report construction, `0044EDB0` | `ev_ae63f4d79ab57c3f507b771ea3156b1dd7633385ab71a39f2a566ff413b36459` |
| Scalar lighting assembly, `0042AFD0` | `ev_ceb41426be05537122bc104cf6e5425c37aae245c3f6ecbb25c66047c2972b44` |
| Parameter lookup, `00408520` | `ev_4ec0d5e3022af8d3ccf8468984271d5542e48fc72a8747d6d276be850ad45ed6` |
| Per-key assembly, `00427F60` | `ev_a7bf306050961e8a80dcfed86611dbe81ead3d32cd79d00f193ddcab217d5bde` |
| Per-key read request, `0042ADB0` | `ev_7a707e412b6559185c2519de8a4f55881df2f638d4a2e05984b19d3c1c1731b1` |
| Bounded read helper, `0044F280` | `ev_6a1ef5504dbd787daf4ce88ca3d30a97f3e5749ea58f94d6c95cd01fd2b9ed8c` |
| Keymap/Fn assembly, `00418550` | `ev_549681fa4dc94b77eb4c4e2c8d85514e9d380f3f8bad17906b0837785ec8c6bb` |
| Macro assembly, `0042D5D0` | `ev_302609df8c766e14a30b6d496699c03337856850a6a958f8254ddc2f09df1461` |
| Key conversion, `00451480` | `ev_e56a4259c336ed4f6de34f467a998bf829c0b3b9bb211175e28a0a3c5a4c8894` |

### Offline implementation status

[sonix.rs](../crates/aula-models/src/sonix.rs) contains original pure encoders/validators for observed 65-byte frames, scalar fields, eight-page per-key bodies and selector headers. Seven focused tests cover lengths, report ID rejection, field offsets, parameter bounds, page boundaries, record capacity and duplicate indices. They establish **IMPLEMENTED / OFFLINE TESTED**, not physical operation. The module contains no transport handle or hardware upload path. No S75 model identity query has been established.

## AULA Hub: partial analysis and bounded next steps

Hub's recovered `app.asar` SHA256 is `31f5e5c3269956f4d92cf9a87405f82555fbd141a6017c391859b3768aa6906b`. Whole-application REA analysis produced partial evidence **`ev_2c45eaf31840b42079936d6beb25389c788c60d3fac6c7997ebe94ec11569f64`**, then hit its 300,000 ms timeout while starting `dist/draco/draco_decoder.js` at source 163 of 244. Its retained JSON is approximately 169 MiB. The analysis also reports a 1,048,576-byte primitive-string budget limit. Earlier observations remain useful, but graph coverage is explicitly incomplete; omitted findings cannot establish absence.

Repeating the whole-ASAR request is unnecessary. A bounded static follow-up should preserve each selected file's hash and artifact-relative path, stage only the relevant inert source, and record unresolved imports instead of automatically pulling the whole dependency tree into analysis. Never evaluate the string-array bootstrap or bundled application to resolve names.

Recommended batches, in order:

1. **Protocol core alone:** `dist/assets/index-BPTK-vUa.js`, 182,661 bytes, SHA256 `3090652c88abbaf61e68f1b643929ff4995068d309dbd6307f8aa6452e00d19b`. The existing AST method inventory identifies lighting, feature receive, keymaps, macro indexing, macro allocation/upload and checksum methods across several separate classes. Resolve each class-to-device mapping before transferring any packet assumptions between them.
2. **One family module per request:** `HFD-DHK8ScUl.js` (358,148 bytes), `HFD-DSWOC1e8.js` (59,967), `Bytech-DuNSRtja.js` (60,290), then `SparkLinkV1-t4aSMPxg.js` (33,217), `SparkLinkV2-JhtdFxGV.js` (53,784), `RLW-BrsvSsKQ.js` (68,778) and `WYFX-SGL1Vmc5.js` (39,389), only where model-dispatch evidence warrants it. These are distinct implementation candidates, not interchangeable families.
3. **Device dispatch and transport bridge:** inspect selected declarations/import references from the larger `index-npIkQlTa.js` plus Electron preload/main files. Preserve dependencies and source positions when reducing scope. Establish report IDs/lengths, model matching and actual sender/receiver behavior before treating frontend method names as protocol proof.

Useful existing core AST coordinates (JavaScript source offsets, not PE addresses or guaranteed byte offsets): class `Fa` macro setter `28845..33179`; class `Wa` macro setter `69042..73427`; class `Oa` lighting setter `130306..132666`; class `_a` feature receiver `61499..61731`; class `Ya` checksum method `179999..180126`. These are navigation leads from the inert AST inventory. Their labels alone do not prove reachability, supported devices or hardware behavior. Large 3D/Draco, UI frameworks, image editors, translations and unrelated assets should stay outside protocol-focused batches.

### Completed bounded core follow-up

A subsequent REA request staged **only the unchanged 182,661-byte core file**, selected that exact source path, and requested a summary first. It parsed one file, visited 66,906 AST nodes and reported zero parse failures without the whole-Hub timeout. Parent evidence is **`ev_eac3b12543b84756e69746ed2592d83dc12f5c77996704898e92b5481ad85afb`**; summary evidence is **`ev_6beba0c86efc9080e09b880e4879ca860f8982f2ebf35fe64d559d2fd7a2ce7d`**. The retained semantic graph remains partial at the deliberately bounded 6,000-node / 2,000-unknown capacities. This is successful bounded parsing, not proof of complete semantic reconstruction.

Independent inspection of that inert source and its AST method ranges supplies these narrower, corroborating facts about class `Fa`:

| Source range | Explicit observation |
| --- | --- |
| `16587..17187` | Allocates a 519-byte outgoing payload and selects the feature report ID from the HID collection descriptor. A fixed numeric ID `06` is **not** established by this constructor. |
| `17258..17433` | An identity-like method constructs the seven-byte request prefix `82 01 00 01 00 0A 00`. This requests ten bytes rather than the six-byte native F87 identity request. |
| `39053..39403` | The named UUID parser reads six identity bytes at array offsets `7..12`, two version bytes at `13..14`, and a screen-size lookup index at `15`. These are this parser's array offsets, not full native HID-report offsets. |
| `18356..18472`, `19220..20072` | Reads a configuration with prefix `84 00 00 01 00 80`, then modifies lighting fields and constructs write prefix `04 00 00 01 00 80 00`. Effect parameter positions depend on a searched `FF FF` marker and mode-dependent arithmetic; this does not establish portable fixed offsets for another model. |
| `20072..20800` | Per-key colors use 126-element red, green and blue planes, 378 bytes total. Read prefix is `86 00 00 01 00 7A 01`; write prefix is `06 00 00 01 00 7A 01`. The source computes a key index from `(pos−8)/4`. |
| `18836..19220`, `40107..40281` | Key values are serialized as four bytes, most significant first, into a 512-byte data area. A setter constructs command `03`, a selector combining two low layer bits with one system bit, and length bytes `F8 01`; the last two outgoing bytes become `5A A5`. The relationship between advertised 504 bytes and the padded 512-byte area needs response/layout corroboration. |
| `28845..33179` | Macro construction uses a 512-byte bank, four-byte little-endian offset/length entries, UTF-8 names prefixed by a one-byte length, and four-byte event records. An event combines a high-nibble action category with a 20-bit duration field and one key/action byte. It constructs macro write command `05`. This differs from the S75 native macro bank. |

The 519-byte WebHID payload plus separately supplied report ID resembles the native G5 family's 520-byte framing. That is **format corroboration**, not a proven mapping from class `Fa` to any particular AULA product. The one-file scope omits physical-model dispatch. Computed names remain unresolved where the code depends on its obfuscated string-table bootstrap; neither the bundle nor that bootstrap was evaluated.

Focused REA semantic traces for literal values 130, 519 and 378 returned **partial, zero retained seed matches**. Their evidence IDs are respectively `ev_3907c4a234568dd675ff49af03ef833adf643a6a2a12560bf8e3b18765c1fc39`, `ev_c7803ce1ce93fec3ff14bc2a438079bb2323b8bfb60f33bfe13c8ba37ec10c49` and `ev_2aac11f333d18e4fdea0973d6fd5371e26dd9fb53f2c0c5e7320ab3770e5b022`. This does not contradict the literals present in the source; it limits what that retained graph proves. Packet meanings above are explicitly source/AST observations, not claims that REA proved dynamic call reachability. Private evidence and the source-hash manifest were saved, and the REA process was closed. No hardware implementation or compatibility gate was enabled by this follow-up.

## Remaining validation boundary

No additional model is **VERIFIED ON HARDWARE** by this research. Catalog/resource metadata, static encoders and mock tests must remain separate from working-device claims. Remaining work includes model/revision discrimination, descriptor confirmation, exact response validation, complete readable backups, unknown-field preservation, macro bounds and recovery, and separately authorized physical validation. Wireless configuration does not follow from wired findings.


## F98 Pro follow-up — unreleased0.6

[Independent executable/DLL analysis](F98_RESEARCH.md) establishes the F98 RGB branch,20 presets,0–5 slider bounds, scalar RGB versus per-key BGR and nine-page data format. Original code now supplies offline scalar/per-key encoders and local lighting/per-key drafts with per-effect controls; legacy schema1 lighting-null profiles load without losing assignments/macros or rewriting storage. This extends the95-position local editor, not hardware compatibility. Shared0c45:800a cannot select a safe transport. Native initial read length, acknowledgment, identity/full backup and recovery remain unresolved; no F98 hardware operation is enabled.
