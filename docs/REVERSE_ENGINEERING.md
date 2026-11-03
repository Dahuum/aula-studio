# Reverse engineering evidence ledger

## Scope, identity and method

This investigation never executed the installer, extracted EXEs/DLLs, or Wine. It used local static extraction, PE resource parsing, string inspection, disassembly and REA. Nothing from the supplied proprietary package was uploaded. Extracted code/artwork and raw decompiler output remain in ignored `analysis/`; they are not release inputs.

| Artifact | SHA-256 |
|---|---|
| Original installer, unchanged at repository root | `1bbd68a9f153384a2b5542a75a4c15f5707613617fdf3fab11db543dbb9d4dee` |
| `analysis/extracted/app/OemDrv.exe` | `1dfe4803efdbe14f66680f970a6d6f26108342fd49754234fa448dd0ebae44a7` |
| `analysis/extracted/app/InitSetup.dll` | `59db7bfd9a92cc9a0998260807f17c536ec71159b727df10d08b998bc94de301` |
| `analysis/extracted/app/uninstall.dll` | `a55c54432d63052a76d24541a0aaebc0665c08fe22fe519de66ac0d6df39326b` |

`7z` did not support this installer. Its header strings identify Inno Setup 5.3.3 Unicode. A project-local extraction of Arch's `innoextract` package and its matching Boost libraries extracted the package without installation or target execution. The main application is native x86 PE32, with standard `.text` and resource sections and HID imports; no packer was established. The setup/uninstall DLLs are separate PE32 installer helpers. Their resource/import inventories were inspected; the HID implementation is in OemDrv.

Decoded private resources: `analysis/KB-decoded.txt`, `analysis/Cfg-decoded.txt`, `analysis/text-decoded.txt`. Package capabilities and regional-layout caveats are summarized in [OFFICIAL_FEATURES.md](OFFICIAL_FEATURES.md).

## REA setup and evidence

The existing `/usr/bin/rea` reports version 6.7.0. The reverse-engineer-anything skill is version 42. No REA tools were exposed in the active Codex session. Read-only `rea doctor --client codex --json` reports missing Codex registration and no configured native analysis engine. Its complete result and the read-only setup proposal are `analysis/rea-doctor.json` and `analysis/rea-setup-plan.json`. No global Codex, shell or desktop configuration was changed. The supported CLI fallback is used, so a client restart is not necessary for current analysis.

Primary tool documentation checked: <https://rea.tools/>, <https://github.com/morluto/rea>, and the installed skill's native/artifact and recovery guides. Ghidra12.1.4 was downloaded, verified, and extracted locally in `analysis/tools/ghidra_12.1.4_PUBLIC` for the REA-supported native provider; its official release SHA-256 is `ddac49f903da9d5bac833e5cc79395098b9c33cfd3279be5f31bd00387d2d4db`.

Retained REA evidence (complete JSON stays local):

| Operation | Evidence ID | File |
|---|---|---|
| Installer PE resources | `ev_59f38e2261a0300859eec29ed33aeffcaa1eaa5efa4ea3583b72a08364bc544c` | `analysis/rea-installer-resources.json` |
| OemDrv PE resources | `ev_5815ab3e19a35a3a1137ad66b42c2fdb4b543e15422b0aff1f36e30af8e3cde9` | `analysis/rea-oemdrv-resources.json` |
| OemDrv GNU objdump headers | `ev_3de00815394f65db7b4ac563b94c58fd320c42fc7e7028c217df89393f2cbd4c` | `analysis/rea-oemdrv-headers.json` |

Separate ordinary GNU objdump evidence: `analysis/oemdrv-pe.txt`, `analysis/oemdrv-disasm.txt`. These are not falsely labeled REA decompilation. All addresses below are virtual addresses for this exact OemDrv build, image base `0x400000`.

## HID and device-family boundaries

OemDrv imports `CreateFileA/W`, `ReadFile`, `WriteFile`, SetupAPI enumeration, `HidD_GetAttributes`, `HidD_GetPreparsedData`, `HidP_GetCaps`, `HidP_GetSpecificButtonCaps`, `HidP_GetSpecificValueCaps`, `HidD_SetFeature` and `HidD_GetFeature`. There is no direct imported `DeviceIoControl` in the inspected import table; that does not rule out HID library internals using it.

`HidD_SetFeature` IAT `0x5e2200`, thunk `0x5ca028`; `HidD_GetFeature` IAT `0x5e2204`, thunk `0x5ca02e`.

The package specifies `Fw=24` and wired `258a:010c`, receiver `3554:fa09`. `Fw` is an application protocol-family selector, not the device's measured firmware. Generic code also contains CDev916KB, CDev3632, CDevComboFilm and CDevG5MS. In particular, mouse report-ID09 checksums and generic firmware/page operations must not be treated as F75 operations.

## Findings from official code

| Finding | Exact location | Confidence / boundary |
|---|---|---|
| G5 keyboard generic access engine | `0x4993c0`; SetFeature `0x49952f`, GetFeature `0x49956f` | Confirmed official disassembly; no physical test |
| 520-byte report, 512-byte chunks | `0x49947b` through `0x499529` | Confirmed official disassembly |
| Report06 selected when family equals24 | `0x4994b9` through `0x4994cd` | Confirmed branch; family binding supported by KB.ini |
| Chunk framing | `0x4994d1` through `0x499512` | `[id,cmd,target,0,total_chunks,chunk_index,len_lo,len_hi,payload...]` |
| Matrix write | `0x499600` through `0x499634` | cmd03, target=layer, caller supplies matrix buffer/length |
| Matrix read | `0x499640` | cmd83, layer-specific 520-byte exchange |
| Macro write | `0x499850` through `0x49989d` | cmd05, target0; not an upload specification by itself |
| Macro read | `0x4998a0` through `0x499a99` | cmd85, target0, chunked; not issued locally |
| LED/config write | `0x499aa0` through `0x499aed` | cmd04, target0 |
| LED/config read | `0x499af0` | cmd84, target0 |
| Config backup before modification | `0x49ca43` through `0x49cb00` | Reads128 bytes, requires final bytes5A A5 and retries; clear support for read/modify/write |
| Config rewrite | `0x49cb6b` through `0x49cb7d` | Calls config setter with128-byte preserved block |
| Matrix signature | `0x49b4ee` through `0x49b508` | When enabled, final DWORD is A55A0000: trailing bytes5A A5 |
| Config signature | `0x49bfba` through `0x49bfd3` | When enabled, bytes126=5A,127=A5 |
| Alternative checksum engine | `0x49dd30`; checksum loop `0x49de85`–`0x49dea5` | Sum of bytes2..519 modulo256 into byte1, ID09; separate G5 mouse context, not F75 report06 |

The four bytes often called a memory address by public implementations are **structured target/chunk fields** in the official engine. `00 00 01 00` means target0, reserved0, one chunk, chunk0. This is not permission to access arbitrary memory.

The engine retries SetFeature up to three times with70ms delays. Linux implementation may adopt stricter validation and must not blindly retry non-idempotent/unknown writes. Official engine success does not establish nonvolatile persistence. A power-cycle test is still needed.

## Configuration variants and lighting

Standard configuration fill `0x49bbf0` preserves the read record, writes the active hardware effect at payload offset10 (`0x49bcaa`), and a color selector at11 (`0x49bcbc`). It writes19 two-byte effect parameter records beginning at payload58 (`0x49bce0`–`0x49bd1d`); the first byte is brightness and the second is `(speed << 4) | (random ? RandomVal : 0)`. The parser binds SpeedHW to device-config offset0x2df0 and LightHW to0x2e10. RandomVal defaults7. Hardware effect1 corresponds to payload58/59, or full-report66/67: equivalently full-report64 + 2×hardware_id. Using64 as a payload offset is an eight-byte error. Offsets are relative to the **128-byte payload**, not the eight-byte report header. A separate low-delay fill routine `0x49b910` uses different offsets; `0x49cb00` selects between them through the SubFw field. The parser reads SubFw default0 at0x412470–0x412492, and this package omits SubFw; therefore its configured branch is the standard fill. CRC=1 is bound to the signature field by0x412313–0x412332. Other packages/revisions still require independent selection.

The standard fill writes sleep-related state at payload24 and debounce-related state at3; the low-delay logger explicitly names debounce at2, tap at8, sleep at9. These differences are a reason to gate settings by verified revision. The package declares a debounce control, but its runtime capabilities and ranges need parser tracing before writes.

Color tables, per-key RGB and effect parameter records are separate regions in the application. Package RGB ordering is0,1,2. An output color table is rebuilt from six colors per effect in the apply path, then padded to512 and marked5A A5 when the signature feature is enabled (`0x49c954`–`0x49c99d`). Do not confuse this table with the individual-key matrix.

### Individual-key RGB is a separate planar region

The G5 keyboard vtable begins at0x6247f4: slot0x74 points to0x499cf0, which sends command06 (SetGame); slot0x78 points to0x499d50, which sends command86 (GetGame). The apply routine0x49c546–0x49c6d0 constructs378 bytes in three126-byte color planes. With GameRGBIndex0,1,2, red is `payload[led_index]`, green `payload[126+led_index]`, blue `payload[252+led_index]`. Target is the color-group index; the supplied package has one group, index0. A key's LED index comes from its key-structure byte0x0d. The parser copies the matrix index into this LED index for eight-field KB.ini entries (`0x4136dc`–`0x4136e4`); all supplied FR key entries use that form. An optional ninth field can differ, so other packages must be checked. The exact group0 write header is `06 06 00 00 01 00 7a 01`, followed by378 planar bytes and zero-filled remainder to520. This path does not append5A A5. This is independent of command0A/8A's512-byte **effect-palette** region, which contains seven RGB colors per hardware effect. Treating the0A table as per-key RGB would be incorrect.

The parent engineering process saved a local physical read in `analysis/hardware/snapshot.json`: config128 bytes and effect palette512 bytes, signature5A A5. Its palette has repeated seven-color groups, consistent with this interpretation. This is locally verified reading, not a write test or complete per-key decode.

## Onboard macros: substantial partial recovery

Transport commands05/85 are now established for this build. Macro conversion function `0x49b000` and allocation/binding function `0x49b3d0` have been isolated, with apply sequence at `0x49c020`.

Observed serialization steps (official static evidence, not hardware verified):

- A macro record starts with one byte containing the name's byte length, followed by UTF-16LE name bytes. The helper bounds the name at30 UTF-16 code units (`0x49b06a`–`0x49b083`).
- Events occupy four bytes (`0x49b125`–`0x49b16c`). A20-bit timing value is clamped to0xFFFFF. Its high nibble is combined with an event-type nibble; the next two bytes are timing bits15..8 and7..0, and the final byte is an HID usage or event-specific value.
- A flag0x80 is added for types other than3/4/5 when the source event's action word is zero. A later recorder pass below establishes action0 as release and action1 as press; this is official static evidence, not hardware playback validation.
- Modifier codes are E0..E7, mouse-button values1,2,4,8,16 use type2; X/Y/wheel use types3/4/5. Ordinary keyboard codes come from a Windows-VK-to-HID table at0x61a288.
- Allocation builds four-byte directory entries containing16-bit relative offsets and16-bit sizes, then adjusts offsets by4×macro_count and concatenates records (`0x49b78d`–`0x49b830`).
- The apply routine rebuilds potentially affected layer matrices before writing the macro pool (`0x49c330`–`0x49c4d5`). Macro bindings and allocation are coupled; uploading a plausible event blob alone is unsafe.
- The parser reads MacroBufferSize with default0x1000 at0x411918–0x41193f. The supplied KB.ini omits this override, so4096 bytes is the official application default for this package, not a physical-capacity measurement. The generic apply routine has a0x2800 local macro-buffer ceiling (`0x49c1d7`), while device-specific resource/runtime limits determine actual permitted size. This ceiling is **not** an established F75 hardware limit.

Matrix macro conversion0x49ab32–0x49ab8f constructs bytes `[03, mode, repeat_count, slot]`: source repeat mode0 yields wire mode1 with caller repeat_count; source1 yields2, source2 yields4; the allocator inserts zero-based slot into the top byte at0x49b703–0x49b719. Exact UI semantic association of these mode values needs tracing.

Still unresolved: exact F75 macro count/capacity, repeat-mode UI association, cross-editor delay placement, behavior on interrupted multi-page writes, acknowledgement/persistence, receiver transport equivalence and revision differences. Onboard uploading must remain disabled pending those items and controlled human hardware validation. A local JSON macro editor does not depend on this proprietary format.

## Public-source comparison and confidence

Public projects are analyzed separately in the project protocol/provenance documentation. `cloudglides/aula-f75-linux-drivers/docs/OEM-PROTOCOL-NOTES.md` contains useful disassembly leads from an older binary with different addresses. Its ambiguous association of command04 with macros is superseded for this local build by the named macro wrappers above (05/85). Its checksum observations refer to another device-family engine. No public source was executed against hardware.

Evidence categories throughout the project must remain separate: official static observations; open-source code; reported public hardware traces; local descriptor enumeration; local configuration reads; and physical writes. This document primarily supplies official static observations and explicitly identifies the separate local read snapshot supplied by the engineering integration process; no physical write is claimed. Resource and code coverage boundaries remain explicit.

## Reproducing focused REA analysis

The host's Java21 path contains only a runtime; REA correctly rejected it as missing a full JDK. The existing full JDK25 works. No Java/system packages were changed. A local Python MCP client launched `rea --mcp`, discovered canonical schemas through `tools/list`, then called `open_binary` with provider `ghidra` and focused `procedure_pseudo_code`. This uses actual REA MCP capabilities without requiring Codex registration or restart. The client is retained in ignored `analysis/rea_decompile.py`; session output and stderr remain beside it.

A single-function CLI equivalent from the repository root is:

```sh
GHIDRA_INSTALL_DIR="$PWD/analysis/tools/ghidra_12.1.4_PUBLIC" \
JAVA_HOME=/usr/lib/jvm/java-25-graalvm-ce \
rea decompile "$PWD/analysis/extracted/app/OemDrv.exe" 0x49b000 --provider ghidra --json > analysis/macro-decompile.json
```

Paths to toolchains are observations of this host; discover equivalents on another system. REA/Ghidra are analysis dependencies only, never application/runtime dependencies.

## Safe next research steps

1. Complete focused REA/Ghidra decompilation at the function addresses above and retain Evidence IDs.
2. Trace capability-parser assignments for Fw24 and CRC/debounce flags to bind the exact configuration variant.
3. Trace macro recording into the12-byte application event structure, then build independent offline fixtures for the established format.
4. Obtain opt-in USB captures in a disposable Windows VM if static analysis cannot settle wire acknowledgements and persistence. Use a snapshot VM with no host directories, secrets or network, passthrough only the selected keyboard, and USBPcap. Never flash firmware/reset/change device modes. Preserve before/after reports for one explicit user action at a time.
5. Only after fixture review, expose a documented write behind an explicit GUI action and backup. Development tests must not transmit configuration writes.

## Completed REA/Ghidra decompilation pass

The locally launched MCP client completed nine focused Ghidra12.1.4 pseudocode queries and exported `analysis/rea-native-evidence.json` (9 retained records). The native session was closed successfully (`analysis/rea-native-close.json`). These are static decompiler observations, not execution. Full pseudocode stays ignored with the proprietary analysis output.

| Parameters | Evidence ID |
|---|---|
| `{"procedure": "0x49b000"}` | `ev_17138851d79f15b954bc3d28032f7e182409d371ba3f8eacec42e9b4228a52a3` |
| `{"procedure": "0x49a9f0"}` | `ev_409622321c0d33148c095aefb3de6f5ec29e7e9b3de89c22abe2d5ccf184124f` |
| `{"procedure": "0x49b3d0"}` | `ev_62aba85145f6436dab50e669feec3f41ccf2d5f9a808beff3b6fbce4cdcf0ebf` |
| `{"procedure": "0x499cf0"}` | `ev_8ce2a6bdf4134a802aa3920d344d99022d3400f0c95a84e9978aa758324f3f43` |
| `{"procedure": "0x499aa0"}` | `ev_91168d1492942d83b30bbe89d3b08e8f3bb4838b1d85f688863201e11ea412fd` |
| `{"procedure": "0x4993c0"}` | `ev_9371716708278e0741f3901e911c9035a179aa890f90307343d278d9b794c5c1` |
| `{"procedure": "0x402880"}` | `ev_a4fc5ae6ff4b0b5d407c246362c778f7431aa30b9305e792a8e876df85baeffb` |
| `{"procedure": "0x49bbf0"}` | `ev_a89bb04e1a80f58325d7a3a21e17c08108f69e9adf47b23065ed96445ac8eefd` |
| `{"procedure": "0x499850"}` | `ev_f39bd9eb7525ec3cdcc40ffed460c869d557477aeeaf1dace0e60df7ec86f21e` |

Decompiler output corroborates the framing, read/write wrappers, config fill, planar setter and macro/binding routines described above. Calling conventions/types are recovered approximations; no decompiler type guess establishes a hardware capability or persistence guarantee.

## Resumed hardware implementation and native validation

A second actual REA/Ghidra MCP pass is retained at `analysis/resumed-native/rea-native-evidence.json`; five focused queries cover GetMatrix0x499640, GetGame0x499d50, palette/apply0x49c7b0, apply0x49c020 and macro conversion0x49b000. The session was exported and closed. It corroborates83/86 reads, palette-before-config ordering, planar-before-custom activation and raw macro packing. No Windows code was executed.

Additional established facts:

- Preset first RGB is palette payload21×hardware-ID. Six other RGB triples in that effect record remain preserved. StartGp/default32 and config11 are the custom group selector, not the preset color selector; normal writes preserve11. Config9 selects custom mode iff effect21. Speed is preserved for effect1; color nibbles preserved for3/15/16/17, according to official capability flags.
- The palette signature is at payload506/507, **not510/511**. Native stack offsets at49c98d/49c995 relative to the transmitted buffer establish0x1fa/0x1fb. All local512B reads show marker506 and four zero bytes after it. A native read/prepare/cancel test initially rejected the mistaken510 check before any write; regression tests now use the independent observed12-byte tail.
- Standard sleep writes payload24; official SleepUI/HW maps seconds60,90,120,180,240,300,600,900,1200 to2,3,4,6,8,10,20,30,40. Zero disables sleep. Other settings such as debounce remain unexposed.
- Local base/Fn83 and per-key86 requests succeeded, saved under analysis/hardware. ANSI79-usage matrix mapping agrees for73 ordinary keys; modifier records use a mask in byte1 (e.g left Ctrl `[00,01,00,00]`), while the saved ANSI Fn53 is vendor action13 and RightCtrl59 is modifier mask10 (the earlier French-role interpretation was corrected in0.4). Native keyinfo conversion0x49a920 (retained query0x49a9f0) corroborates modifier mask packing and media constants. The setter now converts E0..E7 to that mask and preserves unrelated/vendor records. No remap write was performed.
- Raw macro codec implements observed UTF16 name length, four-byte event fields and contiguous offset/size directory. It exposes raw timing values and the release flag established in the recorder pass below; there is no millisecond conversion or upload. Application4096-byte default is not a measured hardware limit.

| Resumed query | Evidence ID |
|---|---|
| `0x49b000` | `ev_17138851d79f15b954bc3d28032f7e182409d371ba3f8eacec42e9b4228a52a3` |
| `0x49c7b0` | `ev_3084990d306afd754d6deb0e425969b670933979099ddf4e53032b5345eb7509` |
| `0x499d50` | `ev_8393a815008f95d9e83a744686cced4aa8e2288ed253e5f25dbf753109db27f3` |
| `0x499640` | `ev_992175c86995668750cfc28f390294fd3372273cff770922babcf2e6875db012` |
| `0x49c020` | `ev_a8e40746b976271d3b93ec16718758d1092d2503e9b8abc79790430f2aef1911` |

A guarded native recovery test also observed config9=1 while config10 held preset18. The cause was not established, so this is not attributed to hotkeys or any particular firmware behavior. Restoring a snapshot preserves its actual boolean9 value rather than normalizing it from10; a no-op restore regression test covers the observed combination. No physical write was performed.

## Recorder semantics pass —2026-11-02

REA/Ghidra retained35 records in `analysis/macro-recording/evidence.json` and closed the native session successfully. This pass followed the recorder and save handlers rather than inferring event semantics from icon filenames. No target execution occurred.

| Function / evidence | Established observation |
|---|---|
| Recorder43ba80; `ev_80a4f6e3f8fdf6efd8ded012073cef15306798abbbd54e1708b64b7abf0d3127` | Gets elapsed time from GetTickCount, caps interval at0xfffff, accumulates a timeline; input flag80 selects release pairing instead of creating a new press. |
| List recorder43f8c0; `ev_9306679072367831d4e44e36ba664295d7c20385504e4315aca07ad55191abde` | Stores action `(~(input_flags>>7)&1)` in the node's top byte. Updates the preceding sibling's low24-bit delay with the elapsed GetTickCount difference. |
| Save conversion43fc00; `ev_746f2aa5c53133cba4fb72a070101654a0e604cc859d22e4a9ccdf012475fd85` | Converts each node to a12-byte application event: VK16, action16, delay32 and auxiliary32. Action is copied from the top byte and delay from the low24 bits. |
| Wire conversion49b000; `ev_17138851d79f15b954bc3d28032f7e182409d371ba3f8eacec42e9b4228a52a3` | Emits flag80 for action0 on keyboard/modifier/mouse-button events; action1 emits no80. |

Together with402880's existing press/release pairing, this establishes **wire flag80 means release in the supplied application's conversion**, for event kinds0/1/2. The raw codec now exposes `key_transition()` and independent byte fixtures for those branches. MoveX/MoveY/Wheel have no press/release transition.

GetTickCount returns milliseconds, so the recorder's original measured values are in milliseconds ([Microsoft API documentation](https://learn.microsoft.com/windows/win32/api/sysinfoapi/nf-sysinfoapi-gettickcount)). The list recorder places an interval on the preceding node, whereas402880 accumulates a record's delay before processing it. The previous-sibling interpretation follows [TVM_GETNEXTITEM documentation](https://learn.microsoft.com/en-us/windows/win32/controls/tvm-getnextitem). This cross-editor delay-placement difference remains unresolved: it prevents confidently translating local delay events into hardware playback. The raw timing field remains exposed without automatic conversion or clamping of user macros. Hardware timing, repeat-mode semantics, capacity, binding/upload sequencing and interrupted-write recovery still require further evidence. No macro read/upload command is enabled.

The application's local macro validation was also hardened independently: Rust storage and the frontend both reject unbalanced presses/releases; imported frontend events reject unknown kinds and mixed key/delay fields. This does not assert hardware macro support.

## Media editor implementation

The retained49a920 conversion evidence (query49a9f0, `ev_409622321c0d33148c095aefb3de6f5ec29e7e9b3de89c22abe2d5ccf184124f`) was revisited to implement seven explicitly emitted media records. Independent native-DWORD fixtures now test the exact little-endian slot bytes. The complete GUI/profile/read-decode/typed transaction chain is connected; no copied proprietary source or assets are included. See MEDIA_BINDINGS.md for action constants and scope. This did not require speculative new packets or commands; physical validation remains pending.

## Debounce parser/control pass —0.3 research

Twenty-five actual REA/Ghidra records are retained in analysis/settings-static/evidence.json and the session was exported/closed. Confirmed static facts for this package:

- Device parser411830 reads ShowDebounce into the model descriptor (evidence `ev_b7548dd0e324ec567104725e636e84ef06a6b000d79799c4ccbac251d30d92df`). This package supplies04000101. Generic alternate DebounceUI/HW parsing at40fd80 is a separate option path (`ev_45769a3e5c3c68fba99340cf299fb254f66b5fe26f186133d9e7b08a4ed3db2a`); those tables are omitted here.
- Default construction47a580 assigns the descriptor bits15..8 to the application debounce value (`ev_565a46d9787033d0df00c88203bc07cb6e357dd57546cb9e65c9b9ea822f3693`): default raw1 in this package.
- Panel instructions460cb4–460cd9 extract descriptor bits31..24 as upper bound and23..16 as lower bound; zero upper bound falls back to0..4. Range helper44a370 populates every integer inclusively (`ev_b6ba5cf6ebdfe4d9c0f5731ab14b9ec1cee47db0cff8cad4b9f86139bf67d164`). Thus the supplied UI range is raw0..4. The complete panel assembly is retained separately (`ev_eeede549bae0ba82fd672a625e8df8609c8c0287f934f5b11150d466dc8d597b`) because its decompiler output omitted this branch.
- Handler4635f0 copies the selected value into application field3340 (`ev_803aaf6e4819732f73ad2a541711f5799bf6a3b6d83cd2e325cd6c11d7186f59`). Display instructions463527–463530 add1 to that value before formatting with a localization unit. Display assembly evidence is `ev_aa091f98d1428653c108f93804c1fc0dc5052949ab020264ba4b2edda4d0d0e3`. Exact unit binding is not yet established from the indirect localization lookup; do not infer firmware timing from this alone.
- The previously established standard fill49bbf0 writes the raw application value to configuration payload3. Alternate low-delay fill49b910 uses payload2 plus a conditional payload3 flag (`ev_af6a21a4ae0c2083b795996afe414c01ea13ae0ec9c4ddb9c096a625e02673a1`). These must never be conflated. One original local read has payload3=2; that verifies readable bytes, not measured debounce behavior.

No debounce setter or GUI control was exposed. The supplied software profile interpretation is substantially recovered, but per-revision capability identification, localization unit binding, physical timing and validation remain incomplete. Further analysis should follow the dynamically populated global localization unit670cf0 and controlled Windows traces if needed. This bounded result supersedes the earlier broad unknown-range note without inventing hardware support.


## Original UI reconstruction, 0.4

REA/Ghidra static UI analysis exported9 records to analysis/ui-static/evidence.json and closed the session. Main constructor459390 (query4593bd; Evidence ev_19a9a056f2b465632347998f206c5b0bd9035a309b13a26c8df1247301f5a008) loads main_nr.png, sizes from the image and places minimize1115,15/close1153,15. Skin loader456d70 (Evidence ev_d3989e6e183261befbe0f0b7c0cfef95c1ffb9d3f04832e7a7b5dc531279506a) references skins/mak.skn, an opaque520B resource. Keyboard panel431580 (query431650; Evidence ev_59a26c98a9aaefa083d9ff6538bd37b4418e708cb59f61340b4fcde427c8d220) supplies further static panel evidence. A failed lighting-panel query remains explicitly unresolved. See ORIGINAL_UI.md for exact measurements and reconstruction limitations. No original Windows GUI was executed.

The visual cross-check exposed a prior matrix-role error: the French package assigns RightAlt53/Fn59, but saved local ANSI base-current.json fullreport220..224 (slot53) is0D000000 (Fn), and244..248 (slot59) is00100000 (RightCtrl).0.4 corrects physical ANSI geometry/usage mapping and Fn RGB index to53, protects that source and requires a fresh matching base marker before per-key/keymap reads or mutations. Earlier read-bank byte evidence remains valid; the former French-derived labels and LED correspondence were not physical validation. No configuration writes occurred. Versions0.1–0.3 must not be used for mapping/per-key operations.

## Official download catalog expansion —2026-11-03

The user-supplied official catalog yielded12 direct Windows packages; source URLs/hashes and excluded browser/hosted/firmware entries are recorded in [DOWNLOAD_CATALOG.md](DOWNLOAD_CATALOG.md) and `data/download-catalog.json`. Ten Inno installers and the Hub NSIS/Electron archive were extracted inertly. F2088 extraction remains unresolved. Resources yielded independent layout/capability facts in `data/models.json`, not redistributed vendor assets.

REA6.7/Ghidra12.1.4 analyzed F87 and S75 native programs via local stdio MCP; sessions exported and closed. Hub ASAR static analysis returned partial evidence after a300s timeout on the unrelated Draco decoder; bounded analysis of its182661-byte G5 core succeeded separately. No vendor JavaScript, Windows application, installer, updater or helper was evaluated/executed. Exact native addresses, SHA256s,22 selected REA Evidence references, transport/macro findings and confidence boundaries are in [MULTI_MODEL_RESEARCH.md](MULTI_MODEL_RESEARCH.md).

The F87 GetPSD six-byte identity request was corroborated by its native device-selection comparison and independent Hub G5-like attribute framing. After offline fixtures, a permitted bounded local read returned original F75 identity0300000000cd. Configuration writes were not performed. Added model editors operate exclusively on local drafts, not physical configuration.
