# F2088 — extraction, partial layout and independent protocol

Status: **PARTIALLY IMPLEMENTED**. Local profiles, known-position key assignments and focused local macros work in Device library. The normal panel-lighting encoder/decoder is **IMPLEMENTED / OFFLINE TESTED**. All F2088 hardware access is disabled; no physical F2088 was available. No Windows program or bundled filter driver was executed/installed.

## Artifact and reproducible extraction

Official catalog artifact: `AULA_F2088_Driver_setup_v1.0.5.7_1_1.exe`,10074083 bytes, SHA256 `8731252ed943e177d659558aef0f0a091a1f630f474abfb35da9459b5b0ed3ee`. Its original download remains unchanged under private analysis. The original supplied F75 installer is a separate preserved artifact.

The F2088 bootstrapper ends with `ADVINSTSFX`. Its trailer lists embedded container filenames, lengths and absolute offsets. Encoded container records have encoding field2. **Only the first512 bytes of each container are complemented withFF**; the remaining bytes are unchanged. Complementing the entire file recovers its signature but corrupts the payload and fails archive validation. The corrected transform passes CAB listings and CRC-valid extraction. The MSI requires explicit `7z -tCompound`, avoiding automatic recursion into an icon stream.

`scripts/research_extract.py` now reproduces this inert extraction. It requires the exact investigated installer hash, allowlists container names, checks trailer/signatures/record bounds, preflights CAB paths/member sizes, bounds expansion and clears executable permissions. No bootstrapper `/extract`, Wine, installer custom action, DPInst or driver loading is used. Unit fixtures reject unknown filenames, traversal, invalid lengths/offsets/encoding and altered trailers.

| Container | Offset | Bytes |
| --- | ---: | ---: |
| ShinetekTools.msi |2166784 |1188864 |
| MainFeature.cab |3355648 |5413701 |
| ChineseSimplified.cab |8769349 |651800 |
| English.cab |9421149 |651802 |

MainFeature contains72 files/12928483 expanded bytes; each language CAB contains2 files/1321257 bytes. Private manifests contain derived hashes and extraction logs. Do not redistribute these proprietary containers or files.

```sh
# Reuse the acquired catalog; this performs static extraction/inventory only.
python3 scripts/research_extract.py --manifest analysis/catalog-2026-11-03/manifest.json
```

## Resource facts and partial layout

`ShinetekTools.exe` SHA256 `810f3ed7f10b877250f9254911eaaff178530494d3a715785e4d4a165e67ba8f` is a different native application from both original F75 G5 and newer Sonix software. Main/English configurations declare VID1a2c/PID9ef4, interface1, subdevice7; the native parser reads these as hexadecimal. Do not assume that subdevice is universally a report ID or identity/revision check.

`positions.xml` has104 records with nonzero dimensions. RightWin has a0,0 origin and is an unpositioned placeholder. The other103 records have unique HID usages and real rectangles; those alone are reproduced as original factual metadata, translated to origin6,60. No Fn position, knob role, LED index or hardware matrix index is invented. The UI explicitly labels this partial coverage as103 resource positions. This is not a claim that the physical keyboard has103 keys.

The partial workspace saves actual local profiles through the Rust store, supports local keyboard/consumer assignments and layout-aware macro editing. Lighting, per-key RGB, Fn and hardware capture controls remain absent. This separates usable recovered positions from unsupported capabilities instead of exposing unconnected controls.

`dlg_main.xml` exposes20 panel-effect names, slider groups with brightness maxima5/13 and speed maximum2, direction/loader choices and hidden side-LED/profile controls. Local `ledeffect.xml` has20 four-byte parameter records. The native parser establishes `[brightness,speed,deriction,selection]`, global parser bounds brightness0..13, speed0..2 and direction0..1. These are application bounds, not physically measured firmware limits or per-effect guarantees.

## Normal panel-lighting protocol — offline only

Native normal sender0041fe90 constructs exactly eight bytes including reportID07. At00404d44 the HID wrapper passes length8 to HidD_SetFeature. The normal sender emits:

```text
07 ff ff EFFECT BRIGHTNESS SPEED SELECTOR 00
```

Explicit effect indices are0..19. The application selection byte maps0→0,1→0,2→1,3→2 in SELECTOR. This mapping is lossy; the decoder reports the wire selector and does not invent the original selection. The normal sender overwrites the resource direction byte with this mapped selector and zeroes the last byte; no direction operation is implemented. A separate fast sender004206d0 copies all four raw parameter bytes without this normalization, so the two paths must not be treated as identical for nondefault fields.

The IO-free `aula-models::f2088` codec implements only the established normal path. It rejects unsupported effects, out-of-bound parameters, incorrect report IDs/commands/lengths and nonzero reserved tails. Known fixture for resource effect4, brightness13, speed2, selection0:

```text
07 ff ff 04 0d 02 00 00
```

The resource side-LED tab is hidden; its separate sender uses a different command prefix and is not exposed. An F1 exchange exists, but response identity/version semantics and exact-revision attestation remain unverified. Restore/reset paths are intentionally excluded. There is no live F2088 transport, raw packet sender or automatic query.

## Windows host actions and limits

The binary includes a Windows filter-driver interface (`DeviceIoControl`) and at least one host action path at00420560 which uses `keybd_event`, `MapVirtualKeyW` and application-side delays. This establishes host execution for that path, **not onboard macro uploading** or proof that every macro uses only the host. Another mapping path manipulates the driver with IOCTLs around a local editor/action update. Bundled `kbfiltr.sys`/DPInst files remain inert; no Linux driver replacement, global input injection or persistent service was installed. Full onboard-versus-host mapping behavior requires further driver/caller analysis.

## REA evidence and confidence

REA6.7.0/Ghidra12.1.4 independently analyzed the extracted native executable. Full private evidence is under `analysis/f2088-native/`; resource/container proofs are under `analysis/f2088-static/`. Native sessions were exported and closed.

| Observation | Native location / REA Evidence |
| --- | --- |
| Config VID/PID/interface/subdevice parser |0042a170; `ev_a0f6f88b28209856d3eac0ddf44112109cdce3a8f17656197a82ffefa39d8c57` |
| Normal lighting construction, assembly |0041fe90; `ev_04db0ecfd62a84c235f318e746e64d48c696298c0a250f01a48e36992ee697b4` |
| Eight-byte SetFeature boundary, assembly |00404cb0/00404d44; `ev_e2ba93987e1ed3efd6b4f3b018ce39d37157744eed21e011e396d833a63651a7` |
| Fast sender differs |004206d0; `ev_5a87d69304989e937a4dab45d19e880c1813b9d00345fac48a0de8e1d53c76c5` |
| Lighting XML reader/bounds |00429290; `ev_f47209675211de1e32e255cf9a2c5aee1aaf07dfee67ecff6e64504a68d35872` |
| Lighting XML writer/byte order |0041ab20; `ev_78f9ee8a0f311485045945e8b559d14e71b03c96869105b4147ae482866ff050` |
| Windows host key action |00420560; `ev_f60b6032a9bd83b6cc44ac502f39e493157b093017ba318d3eee775c04133373` |
| Driver mapping/editor path |0041e4f0; `ev_051092d6c870c544dcde3a4d77d4d2bfe2a91745d7c347414ba09bfa45a5579a` |

Packet construction/byte order are **confirmed by official binary analysis**. Container recovery is **confirmed by static format validation and CRC-valid extraction**. Layout positions are **official resource facts** with explicit omissions. All code paths are software-tested only. Hardware descriptors, revision discrimination, settings readback/backup/restoration, persistence, side LEDs, Fn/knob behavior and macro/binding portability remain unverified. No hardware recovery procedure is established, so hardware writes remain unavailable.
