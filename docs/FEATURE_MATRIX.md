# Feature compatibility — 0.5.0

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
| Macro wire event/record/directory codec | IMPLEMENTED / OFFLINE TESTED |Raw codec plus unreleased0.6 bounded local→wire inspection; application after-event timing established; physical playback unverified |
| Onboard macro uploading | INVESTIGATED / NOT IMPLEMENTED |Physical timing/repeat/binding/persistence/recovery gaps remain |
| Mouse-button assignments | IMPLEMENTED / OFFLINE TESTED |Five native normal-button actions in both layers; local/profile/readback and reviewed matrix transaction implemented; physical output unverified |
| Multimedia UI assignments | IMPLEMENTED / OFFLINE TESTED |Fourteen native media/display/application-launch actions in base/Fn, including16-bit usages; offline codecs/transactions/profile/readback tests pass; physical writes/events unverified |
| Diagnostics / CLI / native GUI / demo | IMPLEMENTED / OFFLINE TESTED |Native read/prepare QA separate from mocked tests |
| Original Windows UI recreation | PARTIALLY IMPLEMENTED |Measured canvas/colors/effect names; optional private artwork; independently drawn ANSI keyboard and standalone assets; full pixel identity unverified |
| Hotplug/reconnect | PARTIALLY IMPLEMENTED |Polling and connection-epoch checks; removal observed, full reconnect write validation pending |
| Debounce / OS layout / polling / firmware version | INVESTIGATED / NOT IMPLEMENTED or UNKNOWN |No guessed controls |
| Wireless configuration | INVESTIGATED / NOT IMPLEMENTED |Receiver enumeration only |
| Direction | UNSUPPORTED for supplied package |All direction capability flags zero |
| Firmware/reset operations | UNSUPPORTED by design |No such operation exposed |

Theme switching has user-reported physical confirmation. Brightness-only6→5→6 has agent-run physical readback and exact all-bank restoration. Other write operations, exhaustive effect/control behavior, power-cycle persistence and restore results remain **BLOCKED ON HARDWARE VALIDATION**. Agent-run native tests remain separate from this user observation.

## Additional models

| Feature | Status | Scope |
| --- | --- | --- |
| Official catalog acquisition/classification | IMPLEMENTED / OFFLINE TESTED |31 entries,12 direct Windows packages, hashes/source URLs retained; no execution |
| Model registry/local workspaces | IMPLEMENTED / OFFLINE TESTED |14 entries;9 new resource-derived keyboard variant editors; see [workspace table](MODEL_WORKSPACES.md) |
| New-model profile/history/import/export | IMPLEMENTED / OFFLINE TESTED |Separate model-bound schema/namespace; never imported into F75 hardware flows |
| New-model RGB/remap/Fn intent and macros | IMPLEMENTED / OFFLINE TESTED |Local drafts only; capability/resource bounds, larger layouts/numpad recording |
| Exact F75 model identity read | VERIFIED ON HARDWARE |Read0300000000cd; guard rejects foreign shared-ID models |
| S75 report/scalar/per-key codecs | IMPLEMENTED / OFFLINE TESTED |Pure offline65B/64B framing; no physical transport or channel-order assumption |
| F87/S75/Hub native/static protocol research | INVESTIGATED / NOT IMPLEMENTED for hardware configuration |[Evidence](MULTI_MODEL_RESEARCH.md), unresolved operations remain unavailable |
| New model physical configuration | BLOCKED ON HARDWARE VALIDATION |Identity/backup/revision/semantic gaps must also be resolved; no enabled writes |
| F2088 and mouse controllers | INVESTIGATED / NOT IMPLEMENTED |Research metadata only, no recovered controller/layout |

## 0.6 read-only additions

F87/F87 Pro010b/F87 Pro Wired and independently analyzed F108/H108: **IMPLEMENTED / OFFLINE TESTED** bounded identity-gated captures, private raw export and lighting-only local draft projection. Physical read acceptance and all new-model writes are **BLOCKED ON HARDWARE VALIDATION**. Config/palette/base/RGB sizes differ by model and are tested separately. S75/F108Pro/F98Pro passive product-name matching improves discovery only. See [F87](F87_READ_CONTRACT.md), [F108/H108](G5_READ_CONTRACTS.md) and [S75 limitations](S75_READ_CONTRACT.md). Older0.5 downloads retain their original local-only scope.

F2088 in0.6: **PARTIALLY IMPLEMENTED**103-position local workspace with unknown Fn/LED/matrix roles omitted; normal eight-byte panel-lighting codec **IMPLEMENTED / OFFLINE TESTED**. All F2088 hardware access remains disabled. The native Windows host-action path is not presented as onboard macro support. [Exact evidence/limits](F2088_RESEARCH.md).

F98 Pro in0.6: **IMPLEMENTED / OFFLINE TESTED** native20-preset lighting definitions,0–5 slider ranges, independent scalar/per-key data codecs, local per-key intent, per-effect disabled controls and compatible loading of older local profiles. **INVESTIGATED / NOT IMPLEMENTED** physical access: incomplete identity/backup/response/recovery, no live Read/Apply/Upload. The95-position resource is not a verified physical98-key reconstruction. [Exact evidence/limits](F98_RESEARCH.md).

## Native desktop port — 0.6 preview

Linux remains **VERIFIED ON HARDWARE** only for the specific operations recorded above. Native Windows x86_64 and macOS Apple Silicon/Intel backends, OS paths, randomness, connection-identity gates, locking and installers are **IMPLEMENTED / OFFLINE TESTED**, with native packaging/window/lifecycle checks tracked in the platform test record. Their physical configuration operations remain **BLOCKED ON HARDWARE VALIDATION**. No new model, wireless or macro-upload capability is implied. [Platform evidence](CROSS_PLATFORM.md).
