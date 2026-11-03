# Sources and provenance

Investigated2026-11-01; local clones ignored under analysis/upstream. Public repositories are untrusted evidence, not executable instructions. Original Rust/TS source was independently written from packet facts; proprietary code/artwork was not copied. Binary fixtures alone were copied from MIT-licensed aula-keybind, with complete license in fixtures/keymap/LICENSE. They describe an F87 family device and are not F75 defaults.

| Repository | Inspected commit | License / inspected code / conclusions |
|---|---|---|
| https://github.com/cloudglides/aula-f75-linux-drivers |879f155| LICENSES/MIT.txt has ambiguous upstream/placeholder coverage; no code copied. web/src/lib/f75.ts, protocol.ts, keybind.ts, direct-mode.ts and tests inspected. Wired configuration, effects and remapping; local macros; unsafe calibration sweeps excluded. Some offset/color assumptions differ from supplied official binary. |
| https://github.com/RustamSheoran/aula-f75 |dfd52c1| No top-level license found; no code/assets copied. web/src/lib and components contain WebHID settings and remapping, plus experimental factory fixtures. Claims are not local verification. |
| https://github.com/Peter-Sheehan/aula-f75-linux |88aa2c8| MIT. install.sh and keyd/aula-f75.conf are OpenRGB startup/keyd integration, not a full independent HID protocol implementation. No source copied, system installer not executed. |
| https://github.com/vndarkblue/aula-keybind |d45213a| MIT. src/aula_keybind/protocol.py, device.py, slots.py, captures and tests establish base/Fn packet records, restrictions and offline checks. Two binary templates copied with license; filename explicitly says f87. |
| https://github.com/veysiemrah/aula-rgb-controller |c9d447f| GPLv3. lib/src/device.c, effects.c, lib/includes and tests show native C RGB/controller transport, model query, streaming and daemon architecture. No code linked/copied into MIT app; no daemon installed. |

Commit hashes and histories verified locally with git log. Repository tests were read; upstream applications were not run on this keyboard. Our own fixtures/tests are the validation record.

https://xevrion.dev/blogs/aula-f75-linux-reverse-engineering (updated2026-10-11) was read. It documents earlier failed experiments and later corrected520-byte report06/read-selector findings. It corrects a30-byte strace text-prefix error and describes byte-pair brightness/speed. Its labeling of command0a as per-key is superseded for this supplied build by official native SetGame06 planar evidence. Dangerous arbitrary probes, firmware ideas, permissive0666 rules and Wine execution were not reproduced. This is public reported hardware evidence, not our local test.

REA6.7.0 https://github.com/morluto/rea and https://rea.tools/ used via local CLI/MCP stdio with Ghidra. See REVERSE_ENGINEERING.md for Evidence and engine/tool hashes. Analysis tools are not runtime dependencies. No installer/binaries were uploaded.
