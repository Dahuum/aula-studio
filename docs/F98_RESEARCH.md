# F98 Pro native contract — unreleased 0.6

Status: **IMPLEMENTED / OFFLINE TESTED** for the independent scalar/per-key data encoders and local lighting workspace. **UNKNOWN / BLOCKED ON HARDWARE VALIDATION** for physical configuration. No F98 transport, live read, transaction, reset or firmware operation is implemented or attempted.

## Targets and selection

The [official package](https://cdn.shopify.com/s/files/1/0695/6987/1965/files/AULA_F98Pro_Software.exe?v=1756176508) was acquired from the official catalog snapshot and extracted without execution. Installer SHA256: `d75ffce415aa10f291c7d26afb55e17f6a0308795b27efdabee6a38e510d5935`.

| File | SHA256 |
| --- | --- |
| `DeviceDriver.exe` | `16fb7173c3b1ab3d080dcb0eb4d648b443aa03b22b324362abaa753122ab19d0` |
| `mui.dll` | `efcf4bf8c2a2cd945e0013768d2870edc1704f14dd8fc415a78e931f42873055` |
| `device.xml` | `f5881bba2fdf6813d230435c415ee9c50a9786b406a2a8a16ddea600867c9d98` |
| `KeyboardLayout.xml` | `b8dd5ef662562c15c8e9d1c4ad633ce8899cf836679fc14060607caef3c5ef01` |

`device.xml` declares wired `0c45:800a`, exact product `AULA-F98Pro` and application identity `RKGK890`. It does not identify a HID interface or attest a firmware revision. The registry therefore retains an unspecified interface and passive product matching only. This VID/PID is shared by other families; matching it never grants access.

The layout supplies95 positions, not a verified98-key physical reconstruction. All95 recovered matrix/light indices are equal and unique, spanning1..123 with gaps. The Fn resource has vendor codeAF, matrix/light index96; it remains protected with no ordinary HID usage. Local Fn intent was already exposed from the resource, but no onboard Fn encoding is asserted. Missing positions are not synthesized. The resource also contains unescaped characters that a strict XML parser rejects; existing bounded factual extraction is not evidence that arbitrary XML should be accepted or executed.

REA6.7.0 MCP and Ghidra12.1.4 analyzed the executable and its separate UI DLL. All sessions were exported and closed. Private bundles: `analysis/f98-native-contract/pass2/evidence.json` (65 retained observations), `analysis/f98-native-contract/mui/evidence.json` (12 observations). The earlier interrupted session also exported/closed. No vendor code, driver, firmware helper or DLL was run or copied into the project/release source.

## RGB branch and native controls

Main constructor00413c00 writes DWORD01010001 at object offset83B, making byte83E equal1. The subsequent device-resource initializer00415280 does not clear it. Lighting constructor00439BE0 selects the RGB list when that byte is nonzero. This establishes the configured software branch, not successful operation of every firmware revision.

| ID | Native label | Brightness | Speed | Single color |
| ---: | --- | --- | --- | --- |
| 0 | LED Off | No | No | No |
| 1 | Static | Yes | No | Yes |
| 2 | SingleOn | Yes | Yes | Yes |
| 3 | SingleOff | Yes | Yes | Yes |
| 4 | Glittering | Yes | Yes | Yes |
| 5 | Falling | Yes | Yes | Yes |
| 6 | Colourful | Yes | Yes | No |
| 7 | Breath | Yes | Yes | Yes |
| 8 | Spectrum | Yes | Yes | No |
| 9 | Outward | Yes | Yes | Yes |
| 10 | Scrolling | Yes | Yes | Yes |
| 11 | Rolling | Yes | Yes | Yes |
| 12 | Rotating | Yes | Yes | Yes |
| 13 | Explode | Yes | Yes | Yes |
| 14 | Launch | Yes | Yes | Yes |
| 15 | Ripples | Yes | Yes | Yes |
| 16 | Flowing | Yes | Yes | Yes |
| 17 | Pulsating | Yes | Yes | Yes |
| 18 | Tilt | Yes | Yes | Yes |
| 19 | Shuttle | Yes | Yes | Yes |

The list uses item data1..19 and0 for Off. Localization labels220/221 exist but are not added to this RGB preset list; their presence does not establish extra presets. Custom mode20 has a separate path and is excluded from the scalar codec. No animation behavior is inferred from names.

The executable sets each slider maximum at object offset14C to5. Independently analyzed `mui.dll` constructor1006BA60 initializes the minimum at148 to0; getters100971E0/100971A0 and SetProgress1006BD30 corroborate that0..5 is the UI range. Native defaults are brightness5/speed3 in the control constructor; project local drafts continue to use their own documented defaults. These values are not a measured device factory state.

0042CA40 selects control groups.0042C810 hides speed for Static;0042C6B0 hides controls for Off. Colourful/Spectrum hide the single-color selector. Direction radio groups exist for some modes, but complete value semantics are not implemented. The local GUI disables the recovered unsupported brightness/speed/color controls instead of presenting them as effective parameters. Random-color, direction, sleep and other hardware settings are not added to the local profile schema in this pass.

## Report framing

00458090 resolves Windows HID functions dynamically. `HidD_SetFeature` is stored at597204, which explains why a direct named-import lookup did not resolve.00456E40 zeroes a65-byte buffer, copies64 data bytes after the initial zero, and sends65 bytes. The feature report is therefore an unnumbered report ID00 plus64 payload bytes, distinct from F75 report06/520 and F2088 report07/8.

The read helper00458A20 uses overlapped `DeviceIoControl` with codeB0192. Its decompiler prototype omits the return value: assembly458A75 returns minus1 on failure and458AA0 loads the actual returned byte count. Conclusions about response/error flow must use instructions as well as pseudocode. This helper's existence does not establish a safe, complete F98 read contract.

## Scalar lighting data

0042D290 constructs the following zero-filled64-byte data payload after querying the application's SQLite lighting record through00402DB0. Full-report offsets add1 for report ID00.

| Payload offset | Meaning |
| ---: | --- |
| 0 | Preset ID0..19 in the normal RGB preset path |
| 1,2,3 | Red,green,blue, respectively |
| 4..7 | Zero in this constructor |
| 8 | `colortype`;0/1 selector, with1 checked as random/colorful in the UI |
| 9 | Brightness, UI0..5 |
| 10 | Speed, UI0..5 |
| 11 | Native direction byte; preserved raw, semantic values incomplete |
| 12,13 | Zero |
| 14,15 | LiteralAA55 marker |
| 16..63 | Zero |

There is no discovered checksum calculation for this structure. AA55 is a literal marker, not a claimed CRC. The offline decoder requires exact length/report ID, known preset/ranges/selector, marker and all canonical reserved bytes; it rejects unknown bytes instead of losing them during re-encoding. It is not a decoder for arbitrary device memory or another command's replies.

Independent fixture, full-report first17 bytes, followed by48 zeros:

```text
00 07 11 22 33 00 00 00 00 01 05 03 02 00 00 AA 55
```

This is constructed from independently traced native offsets, not a captured or physically accepted report. Direction02 is an explicit raw fixture value, not a physical direction assertion.

The official operation surrounds this data with0418,0413 (payload8=1),0402 and a trailing04F0 exchange. The scalar data send itself does not request the helper's response read. Full state transitions, acknowledgment and the trailing exchange's meaning remain incomplete. None of those control exchanges is exposed by the new codec or sent by the application.

## Per-key data differs from scalar color

0042D440 selects the RGB branch's9-page data length:576 bytes, with literalAA55 at574/575. Each occupied slot starts at4×index and contains `[index,B,G,R]`.0042D960 reconstructs the on-screen ARGB color as `FF,slot[3],slot[2],slot[1]`; the separate UI DLL uses GDI+ color for keyboard preview. This is not the scalar path's RGB order. The UI's `CColorButton` also stores a Windows COLORREF; it must not be mistaken for the keyboard preview's ARGB value.

Examples for resource-known slots:

```text
payload224..227 = 38 33 22 11  # index56, RGB11/22/33
payload476..479 = 77 CC BB AA  # index119, RGBAA/BB/CC
payload574..575 = AA 55
```

The pure encoder accepts only the95 resource-known light indices, rejects duplicates/unknown or overlapping indices, and leaves unrecovered positions zero. It returns9 data reports, each ID00 plus64 bytes. The last report's bytes63/64 containAA55. This is an offline constructor, **not a complete snapshot or safe restoration image**; clearing unmodeled state on a physical keyboard would be unsafe.

The vendor operation surrounds these pages with0418,0423 (payload8=9),0402 and04F0. Its multi-page sender retries failed sends and has no verified all-bank recovery. The pure implementation exposes no preamble/commit or transport.

The vendor per-key reader uses04F5 with a9-page selector through004572B0. Its first read is passed the page-count argument as a byte length, followed by65-byte reads; return/prototype and acknowledgment handling require further assembly tracing. This is a material unresolved safety issue. No live F98 read is enabled, even though an offline data constructor exists.

## Local compatibility and validation

The local workspace now exposes the native20-preset table,0..5 ranges and per-key color intent, with per-effect disabled states. Assignments/Fn/macros remain local intent. The application still exposes no physical F98 Read/Apply/Upload control. G5 capture selection rejects F98 explicitly.

Earlier schema1 F98 profiles serialized `lighting:null`. Native `decode_draft` and the browser parser migrate only that model/schema combination to a default local lighting draft while preserving ID,name,assignments,Fn intent and macros. Reading leaves the original file/storage bytes unchanged; saving is explicit. Other models and future schemas are not guessed. This is forward reading of the older format, not a promise that old0.5 can import new lighting data.

Tests cover independent scalar offsets, RGB versus BGR fixtures, all9 pages/tail placement, malformed IDs/lengths/markers/reserved data, range/custom-mode rejection, unknown/duplicate resource slots, model gates, non-destructive legacy loading and explicit persistence. Browser tests cover effect-specific disabled controls and legacy-profile workflows. None is a physical-device test.

## Evidence index

All executable addresses below apply only to the target hash above;100-prefixed addresses belong to the separately hashed UI DLL.

| Observation | Function / Evidence |
| --- | --- |
| Main RGB flag |00413C00; `ev_c4223df06788035853198b8ba2b93af5ce78e39e8fe71ab7a85ff6ff59668a5f`; assembly `ev_001af5ccad8f801eeb04657fc762aa7f706eb478d645daa30827cad1e0ca239d` |
| Resource initializer |00415280; `ev_a7b63ee7e62512fa86f5e30b6ee989baf0cbb056cd573773576c41bdfd61c2a0` |
| Preset/control construction |00439BE0; `ev_502498274d1cda79630ba8848a3126aea52af9ff7842c13f706d29bfc231b073` |
| Mode-specific controls |0042CA40; `ev_e641ee1e9ef9df346dc8457b5513f886215224e93419a2008ca882ef140370c2` |
| Slider minimum/defaults |1006BA60; `ev_e5b57c12c274aaffde972e6c485789db11cdea8f8a800bbda52de8462579c475` |
| Slider bounds/getter |1006BD30; `ev_4ac638376749f6c865b94f9a059f951501c97ea828611fa793d26506250ac6ed`;100971A0 `ev_beb256ca532af183e9456135c85f4aadc860bbfd5ccf4db7c509829de614754f` |
| Feature report sender |00456E40; `ev_a1fc4bf62f787a41c973a07d663f335821311cbe284c009442576c017d866b66` |
| Read return assembly |00458A20; `ev_6f21c364e51f6b1c3b690cadfd3806ccb9705da6ecb3a4d944c2e9c237332689` |
| Scalar constructor |0042D290; `ev_9f90ad570a750ed16e59451a19934d88c984cb580c859586fb2dc40233cafff6`; assembly `ev_cc9449d665152923cbf7e2dcce4025d799c8105a6f94d7565cf83de6d72cc89b` |
| SQL field mapping |00402DB0; `ev_5010a2798788a297fec38c370d734838ce4eefbab39ca90add64dd9c7cbacae3` |
| Per-key constructor |0042D440; `ev_0f2b4cf68c1f7858c0c3c09bdfe15258bf57c6b9fccffef95d0dc0872bc5c03d`; assembly `ev_ebb654722f6644a072d31737af220e61804410a093d82ad9acd5d02afc67df54` |
| Per-key response/colors |0042D960; `ev_285ba3a8b00f7439304032627e6aad1d54c8e7a6ca06656dc4de8a3de9a66f55` |
| Paged-reader limitation |004572B0; `ev_0964bff941df043465b47c69e325cd6a5c323f70306ba825eeb4705c48b52a66` |
| Keyboard GDI+ color representation |1003BD90; `ev_acdca6f036c45285c539db3cc79cea5f7bc63ff300c1e6279117af5f131f2e7e` |

These are **confirmed by official binary static analysis**, with uncertain interpretations explicitly separated. There are no public hardware traces or local physical verification for these new operations. Recovery is unknown; offline generation changes nothing, while physical implementation must wait for independent identity, bounded complete backup, response semantics and a reviewed recovery strategy. Bundled firmware/reset/LCD-clock writers are excluded; an early private query filename is not evidence that a function was read-only.
