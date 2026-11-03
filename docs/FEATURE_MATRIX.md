# Feature compatibility — 0.4.2

A physical read does not validate a write. All write paths below require explicit GUI review/commit, backups and exact readback. Normal startup and standard tests never apply configuration. The completed, separately authorized two-action brightness test is documented in TESTING.md.

| Feature | Status | Concrete scope |
|---|---|---|
| Wired detection / receiver identity | VERIFIED ON HARDWARE |258a:010c /3554:fa09, interface/descriptor gate |
| Config and effect palette reads | VERIFIED ON HARDWARE |128B/512B, signatures validated |
| Base/Fn keymap reads | VERIFIED ON HARDWARE |520B each, official83 request |
| Per-key bank read | VERIFIED ON HARDWARE |378B planar, official86 request |
| ANSI layout and matrix mapping | IMPLEMENTED / OFFLINE TESTED; partially VERIFIED ON HARDWARE |79 usages plus Fn RGB,73 normal base records match; corrected ANSI Fn53/RightCtrl59, fresh base-marker gate; visible correspondence pending |
| Theme/effect switching | VERIFIED ON HARDWARE |User reported a successful visible hardware theme change on2026-11-02; exact effect/build and logs unspecified |
| Brightness-only change/readback/restoration | VERIFIED ON HARDWARE |Agent test on wired ANSI F75: effect2, payload60 changed6→5→6; both commits read back, all five banks restored exactly |
| All preset effects/off/color/brightness/speed | IMPLEMENTED / OFFLINE TESTED |Real04/0a write path; exhaustive controls/effect sweep still needs physical validation |
| Per-key RGB | IMPLEMENTED / OFFLINE TESTED |Real06 bank + custom-mode04; write/LED correspondence validation pending |
| Base/Fn assignments | IMPLEMENTED / OFFLINE TESTED |Real03 bank; protected Fn/F sources; write validation pending |
| Standard sleep timer | IMPLEMENTED / OFFLINE TESTED |Known24 byte, official choices incl disabled; write validation pending |
| Backup / scoped lighting+sleep restoration | IMPLEMENTED / OFFLINE TESTED |Durable before-state; real reviewed restore of supported fields only |
| Bank backup restoration | IMPLEMENTED / OFFLINE TESTED |Known ANSI source/LED entries restored through review; protected/unknown rows and current mode preserved |
| Local profiles CRUD/history/import/export/preferences | IMPLEMENTED / OFFLINE TESTED |Version1 with backward-compatible optional fields |
| Local macro record/edit/import/export/preview | IMPLEMENTED / OFFLINE TESTED |No global key injection |
| Macro wire event/record/directory codec | IMPLEMENTED / OFFLINE TESTED |Developer raw codec; release flag established from recorder; delay placement still prevents local macro conversion |
| Onboard macro uploading | INVESTIGATED / NOT IMPLEMENTED |Delay placement/repeat/binding/persistence/recovery gaps remain |
| Multimedia UI assignments | IMPLEMENTED / OFFLINE TESTED |Seven official media actions in base/Fn; native demo/profile workflows pass; physical writes/events unverified |
| Diagnostics / CLI / native GUI / demo | IMPLEMENTED / OFFLINE TESTED |Native read/prepare QA separate from mocked tests |
| Original Windows UI recreation | PARTIALLY IMPLEMENTED |Measured canvas/colors/effect names; optional private artwork; independently drawn ANSI keyboard and standalone assets; full pixel identity unverified |
| Hotplug/reconnect | PARTIALLY IMPLEMENTED |Polling and connection-epoch checks; removal observed, full reconnect write validation pending |
| Debounce / OS layout / polling / firmware version | INVESTIGATED / NOT IMPLEMENTED or UNKNOWN |No guessed controls |
| Wireless configuration | INVESTIGATED / NOT IMPLEMENTED |Receiver enumeration only |
| Direction | UNSUPPORTED for supplied package |All direction capability flags zero |
| Firmware/reset operations | UNSUPPORTED by design |No such operation exposed |

Theme switching has user-reported physical confirmation. Brightness-only6→5→6 has agent-run physical readback and exact all-bank restoration. Other write operations, exhaustive effect/control behavior, power-cycle persistence and restore results remain **BLOCKED ON HARDWARE VALIDATION**. Agent-run native tests remain separate from this user observation.
