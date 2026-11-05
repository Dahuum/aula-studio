# Validation

## F98 follow-up — current unreleased source

139 Rust tests (56 HID,30 model/codec,28 protocol,16 service,9 native backend),70 frontend tests and16 Python checks pass, with formatting/strict Clippy/TypeScript/Vite. New cases independently check scalar offsets and RGB order, nine-page per-key BGR/tail placement, malformed report/range/index rejection, F98 versus G5 gates, actual-file legacy migration without implicit rewriting, and effect-specific GUI controls. Two new publication tests check the general native builder's documentation allowlist/private-note exclusion and Git/symlink preservation. Older schema1 work remains intact. No F98 hardware request or new physical configuration write occurred. [Contract/evidence](F98_RESEARCH.md).

An initial Rust assertion incorrectly assumed the existing resource-declared local Fn intent was absent; it was corrected without changing that capability. A simultaneous frontend run timed out before any tests because workers could not start under high host load. After closing both owned Ghidra sessions, the same assertions pass with one frontend worker and two Cargo jobs. No unrelated process or host setting was changed.

Current host-built optimized source passed actual native F98 control/per-key persistence and legacy-profile load/explicit-save/history, all10 local workspaces, existing F75 demo workflows and8 Wayland/TTY startup cases. Result directories: `analysis/native-model-smoke/1793801280734548401`, `analysis/native-demo-smoke/1793801291181759570`, `analysis/native-startup-smoke/1793801299431818237`. GUI SHA256 `9c046aec11a0201f7139d71d317894fcccbd8991b3909113869941297ce9d160`; CLI `7365b69dd7bdd72a10659c6b5047eba3d0522f1f8beebd5192672f794cbc3a30`. These are Arch host/glibc2.39 binaries; the Debian-baseline packages below are an earlier checkpoint and are not silently replaced by this follow-up. Latest passive enumeration still sees only3554:fa09 receiver interfaces.

## 0.6.0 baseline checkpoint — 2026-11-04

- 132 Rust tests pass, including 56 HID,23 model/codec,28 protocol,16 service and9 native backend tests. The full suite also passes offline inside the Debian12 build container. Formatting and strict Clippy pass.
- 68 frontend tests pass; TypeScript/Vite production build passes. New coverage includes explicit macro timing/cancellation, editing/paging, exact copy-name byte/character boundaries, Unicode copies and independent event storage, typed model captures and the partial F2088 layout.
- 14 Python acquisition/publication checks pass, including synthetic Advanced Installer prefix decoding and malformed container rejection. No proprietary target was executed.
- The final Debian-baseline native app passed the F75 demo, all10 local model workspaces and8 Wayland/TTY startup cases. Long Unicode macro/profile copies were saved through the actual Rust backend. WebKitWebDriver ignored non-BMP send-keys in an initial attempt; the final test uses the controlled input paste handler and verifies the actual saved name, without replacing native IPC. An additional unsaved-profile confirmation is explicitly accepted in this test.
- F2088's eight-byte codec, original F75 macro wire inspection and new-model captures are offline-tested. No F2088 hardware access, new-model write, macro upload or wireless operation is enabled. The latest passive enumeration sees a3554:fa09 receiver, not the required wired F75 configuration interface.

Private logs: `analysis/f2088-static/{debian-build-and-tests,frontend-final,frontend-build-final,clippy-final,python-final,debian-native-f75,debian-native-models,debian-native-startup}.log`. No new physical configuration write occurred. Published0.5 validation below remains historical and independent.

Final native results: `analysis/native-demo-smoke/1793782031004601171/result.json`, `analysis/native-model-smoke/1793782039850283871/result.json` and `analysis/native-startup-smoke/1793782050022769251/result.json`; final logs use the `debian-native-final-*` prefix. GUI SHA256 `1f819d3bfa18b0fcd289a427a6d5a501f9c46bbba6ca1cb79188dc487a5177ef`; CLI SHA256 `4e02df552867d75682c057578446ef2b04970858f2b114daf5008d77f81d2bc8`. This is an unreleased development build, not physical validation of untested operations.

Local baseline packages in `dist/development-0.6.0-baseline/` pass an exact native-tested ELF identity audit in all four binary formats (portable,Arch,Debian,RPM). The149-file clean source export matches the sanitized allowlist; archive paths/types, private/proprietary exclusions, omitted Arch .BUILDINFO, Markdown targets and five checksums pass. Fresh Debian12, Ubuntu24.04 and Fedora43 containers each pass package install, CLI version/write guard, actual mapped GTK window as UID1000, and uninstall. Evidence logs `analysis/f2088-static/package-smoke-{debian12,ubuntu2404,fedora43}.log`. No host package installation or USB/display~ mount occurred. The disposable container-only WebKit namespace-sandbox override is never set in the shipped app. Existing public0.4.3/0.5 assets and original installer hashes remain unchanged.

## Published 0.5.0 — 2026-11-03

## Multi-model implementation

- 111 Rust tests pass:48 HID,14 models/codecs,28 protocol,12 service,9 native backend. Strict Clippy and rustfmt pass.
- 44 frontend tests pass, including model layouts, selected-model click regression, separate profile storage, colors/assignments, protected controls, cross-model import selection and numpad macro recording/wrapper import. TypeScript/Vite production build passes.
- 12 Python acquisition/publication safety tests pass; only the exact two public factual data JSON files are allowlisted. Raw model configs/backups and proprietary installers/binaries stay excluded.
- Native WebKit/Tauri Device library test passes with real Rust profile storage: all9 new layouts, research-only control suppression, save/clean state, local per-key/Fn data, focused macro recording, export/import/delete/history, malformed import/legacy-namespace rejection,0600 files and return to F75. No mocked IPC. AULA_DISABLE_HARDWARE_WRITES=1; no physical journal/backup created.
- The native model test found/fixed clicking the already selected model clearing its editor, and legacy global CSS overlaying the new flex workspace. Browser tests alone did not establish native rendering correctness. Workspace bounds are now asserted in native QA.
- Existing F75 native demo workflow and8 startup/Wayland/TTY cases pass. These are software/window checks, not additional hardware writes.
- REA/Ghidra F87/S75 sessions and bounded Hub-core static analysis exported their evidence and closed; Windows code/JavaScript bootstrap were never executed.
- One permitted physical bounded G5 identity read returned0300000000cd on the wired original F75. It does not validate other devices, settings, new-model writes or persistence. No configuration writes were performed in this expansion.

Private detailed logs: analysis/multi-model-research/{rust-tests,frontend-tests,python-tests,native-model,native-f75,native-startup}.log. Reproduce the native workflow with `python3 scripts/native-model-smoke.py --fallback --binary target/release/aula-studio`; requires WebKitWebDriver/Xvfb. Native screenshots are captured from the actual app, never invented. Older read/write evidence below remains separate and is not repeated by standard tests.

The0.5 release binary was also built offline against Debian12/glibc2.36 and passed native F75 GUI/demo and all8 startup/Wayland/TTY checks on Arch. Its Device library workflow passed with an isolated session bus. An earlier simultaneous GUI run lost the WebDriver session during its final screenshot; that interrupted run is retained and not counted as a pass. No production display/sandbox settings were changed.

GUI SHA256: `772a98b8a3c6bcacf788705be847c03d3af59623304f8a030ada7ec9cf63539a`. CLI SHA256: `2bfdc3711b44ed18f8b6918ed36df7edcaec14e5b7297cc531b1428450450233`. Portable/Debian/RPM/Arch payloads use these exact binaries. Static readelf validation rejects a package build requiring glibc newer than2.36. Arch package publication omits host-wide .BUILDINFO inventory and its mtree entry while verifying all retained file hashes.

## Earlier release validation


##0.4 baseline passing automated checks

| Check | Result | Evidence |
|---|---|---|
| cargo test --workspace --locked |92 passed:47 HID,28 protocol,12 service,5 native backend |analysis/release04-rust-tests.log |
| cargo clippy --workspace --all-targets --locked -- -D warnings |Pass |analysis/release04-clippy.log |
| cargo fmt --all -- --check |Pass |analysis/release04-format.log |
| npm --prefix app test |37 passed across5 files |analysis/release04-frontend-tests.log |
| npm --prefix app run build |TypeScript/Vite pass |analysis/release04-frontend-build.log |
| cargo build --locked --release -p aula-studio -p aula-cli |Pass |analysis/release04-build.log |
| Actual native WebKit demo with standalone artwork |Pass |analysis/release04-native-fallback.log |
| Actual native WebKit demo with separate XDG roots/private original artwork |Pass |analysis/release04-native-xdg-private.log |
| Native Wayland launch under Hyprland |Pass: mapped=true, xwayland=false, title AULA F75 |analysis/native04-wayland/result.json |
| Hardware brightness dry-run |Safely rejected: no supported wired interface; no writes |analysis/release04-hardware-preflight.log |

Native tests use the real release executable, React/WebKit frontend and Tauri Rust bridge, not a mocked bridge. AULA_DISABLE_HARDWARE_WRITES=1 is reflected in diagnostics, and scripts reject commit_* calls. Test data/preferences stay under analysis/. All tests compare native demo/profile results and check no hardware-operation journal/backup was created. Wayland QA closed only its own isolated process.

Native coverage: disconnected disabled controls, demo toggle/apply/read, private preferences0600, separate XDG paths, reduced-motion preference, sleep in demo, all navigation panels, all20 effects and OFF disabled controls, hex/RGB/numeric/wheel controls, brightness/speed, palette persistence, Fn per-key assign/clear, base/Fn media assignments, focused A macro recording/Stop/balanced events, profile save/duplicate/rename/load/export/import, native backend deletion, invalid macro rejection, clean saved state and scaled small-window bounds. Profile deletion's GUI confirmation is not automated in this native smoke; backend/storage deletion is tested. Native titlebar minimize/drag and a complete manual accessibility sweep are not claimed as tested.

Xvfb software acceleration initially hung while snapshotting after rapid UI changes. Disabling WebKit compositing in the headless test environment resolves this; it is not set in the application or the separate Wayland launch. Earlier failed attempts are retained privately and do not count as passing tests. Native color-entry testing exposed a stale state issue on fast Enter; the handler now reads the actual input value. A toast/actionbar overlap was also corrected. Current passing runs use these fixes.

## Protocol, transactions and storage

Tests cover request/report lengths and IDs, payload framing, malformed reports, signature locations, planar offsets, ANSI mapping uniqueness, observed Fn53/RightCtrl59 roles and mismatching-layout rejection, key/media/modifier packing, protected sources, raw macro bounds/directory/event fixtures, RGB/sleep validation and profile schema migrations.

Mock transport tests cover stale banks, tampered plans, no-ops, disconnects, exact readback mismatch, partial/ambiguous writes, no automatic retry/rollback, palette/config and per-key/custom ordering, scoped restore, protected/unknown-byte preservation and advisory-lock exclusion/release. Native backend tests cover expiry/single-use tokens and backup tampering. Storage tests cover atomic/history/import/delete, XDG legacy preservation/migration, relative-path fallback, private permissions and custom palette max10 on read/save.

Frontend unit tests use a mocked native bridge where physical review/commit dispatch must be exercised; these are not physical tests. They cover lighting/mapping/per-key/sleep reviews, cancel/error/no-op, restore cancellation, capability disabled states, macro bounds/focus/release handling, import validation, original navigation/effects/colors/palette and correct ANSI geometry.

## Physical evidence and outstanding checks

Earlier wired258a:010c interface1/report06/520B reads succeeded for config128B, palette512B, base/Fn520B each and planar378B. Private evidence is under analysis/hardware and the earlier native read/prepare/cancel run under analysis/native-review-smoke. Those are earlier builds' bounded reads, not a fresh0.4 physical-write test. The0.4 hardware scripts have updated UI selectors but cannot run past wired-device preflight in the current environment.

The prior French-derived bottom-row roles were wrong. Local saved bytes prove Fn53=[13,0,0,0] and RightCtrl59=[0,16,0,0].0.4 fixes/protects them and verifies the fresh base marker before matrix operations. Old0.1–0.3 mapping/per-key builds must not be used. Earlier73 ordinary-record matches and raw bank reads remain evidence; LED correspondence still needs physical validation.

Before the0.4.1 follow-up, no agent-run physical configuration write, firmware/reset, macro upload or receiver command was performed. The later authorized brightness-only result is recorded below. The user subsequently confirmed a visible hardware theme change (see below). Other effects/controls, input behavior, persistence, physical restore, other revisions/ISO and wireless configuration remain unverified. The already-authorized one-step brightness/readback/restoration test must wait for exactly one supported wired interface; permission does not need repeating. Full pixel identity with every original Windows dialog/font is also unverified. See ORIGINAL_UI.md and HARDWARE_VALIDATION.md.

Portable/Arch staging and final audit passed:40 portable entries, exact native-tested executable identities in both packages, all runtime libraries found, package metadata0.4.0-1, no proprietary skin/binary/analysis tree included and unchanged installer SHA256 (analysis/release04-artifact-smoke.json). Packages are built, not installed. No privileged service/rule or system configuration was installed or changed.


## User hardware confirmation after release

On2026-11-02, after the0.4 handoff, the user reported testing a theme change and seeing it work on the physical keyboard. Status: VERIFIED ON HARDWARE (user-reported theme switching). Exact effect/build/connection details and transaction logs were not supplied; no independent readback, all-effects sweep, persistence, per-key, remapping or restore validation is inferred. The agent performed no additional USB operations or configuration writes for this update.


##0.4.1 visual refinement and first agent-run physical write test

Optional original12px round and14×7 triangular slider handles and13px checkbox faces now load locally with independent CSS fallbacks. Tracks are3px; independently drawn keycaps gain bevels/USB connector. These are specific visual corrections, not complete original-UI parity. Remaining panel/font/keyboard-art gaps are in ORIGINAL_UI.md and ROADMAP.md.

The wired interface reappeared as258a:010c interface1/report06/520B. The dry-run passed with zero writes and all banks unchanged (analysis/release041-brightness-dry-run.log). Then the already-authorized test passed (analysis/release041-brightness-hardware.log; private durable evidence analysis/brightness-validation/1793645774584079067): effect2, payload60 brightness6→5→6, exactly two GUI write actions, both native journals readback_verified. Config128B,palette512B,base/Fn520B and per-key378B all exactly match the pre-test backup after restoration. No remap/per-key/sleep/receiver/firmware/reset write was performed. Visible brightness change and power-cycle persistence are not independently measured. User-reported visible theme switching remains separate evidence.

The initial dry-run exposed an automation bug: Tauri's invoke property is intentionally immutable, so the old script's attempted interception observed no result although prepare saved its read-only backup. That failed attempt sent no configuration writes. The harness now observes actual review DOM plus durable operation journals, explicitly limits its Python GUI Write actions to two brightness-only calls, refuses direct commit invocation and preserves all stale-state/connection/all-bank checks. No application IPC protection was weakened. Successful test binary hashes are retained privately beside the evidence. Later status-text updates reflect this result without broadening hardware capability claims.

Follow-up frontend37 and native backend5 tests, production frontend/release build, native standalone/private artwork workflows, guarded real-device review/cancel and package audit results are recorded in PROGRESS.md as they complete. No additional hardware-write test is implied by rebuilding status text.


0.4.1 follow-up results: full92 Rust tests,37 frontend tests, strict Clippy, production frontend and locked release build pass. The actual wired read/prepare/cancel test also passes on0.4.1 with writes blocked, confirming durable base/Fn/per-key/sleep reviews and byte-identical config/palette/base/Fn/per-key afterward (analysis/release041-native-read-review.log). Native private/XDG and standalone workflows pass; native Wayland launch maps=true/xwayland=false (analysis/native041-wayland/result.json). Final sidebar sprite corrections are RGB navi5 and Macro navi4, replacing the incorrect cloud7/RGB5 choices; unused account/cloud reads are removed. Win now matches the original legend while preserving MetaLeft/usage227. Final visual/native/package audit is recorded in PROGRESS.md as completed. The first frontend invocation raced npm ci's dependency refresh; only the subsequent complete37-pass runs count, and future runs must sequence dependency mutation before tests.


Final0.4.1 package audit passed:40 entries, both portable and Arch executables match the final native-tested hashes; runtime libraries resolve, Arch metadata0.4.1-1 is correct and the installer hash is unchanged. No proprietary skins/binaries or analysis tree is bundled. Final original-art and fallback native logs are analysis/release041-final-native-private.log and analysis/release041-final-native-fallback.log. The status-text/icon/legend rebuild made no additional configuration writes.


##0.4.2 GTK/TTY startup regression

The user's0.4.1 GTK panic was reproduced with DISPLAY/WAYLAND_DISPLAY/GDK_BACKEND absent and XDG_SESSION_TYPE=tty (analysis/gtk-startup/old-tty.log, exit101). A real user-owned Wayland socket and session bus remained available. Earlier forced-backend native tests missed this launch environment.

Startup now selects a unique user-owned wayland-* socket only when both display variables are absent and the backend allows Wayland. It validates a private absolute user-owned runtime directory, ignores regular files/locks/symlinks, rejects ambiguity and preserves explicit display/backend choices. The standard /run/user/euid fallback supports a missing runtime variable. Environment changes are process-local and occur before GTK/background work. GTK init is checked explicitly on the main thread so failure returns exit1 with display guidance, before app-data initialization, rather than Tao panicking.

96 Rust tests pass (47 HID,28 protocol,12 service,9 native backend including4 startup planning tests),37 frontend tests pass; strict Clippy, formatting and frontend/release builds pass. Actual native startup integration passes8 cases: default desktop, TTY without display variables, TTY without runtime variable (all mapped=true/xwayland=false), plus clean failures for headless, forced-X11/no-display, explicit stale Wayland, relative runtime and multiple sockets. Negative cases verify exit1, no panic and no app data created. Full native GUI/demo/profile workflows still pass using explicit X11/Xvfb with separate XDG paths/private artwork. No hardware writes, user window termination or system configuration changes occur in these tests.

Evidence: analysis/gtk-startup/{rust-tests,frontend-tests,frontend-build,release-build,clippy,native-startup,native-gui}.log; startup result analysis/native-startup-smoke/1793652937524028600/result.json. The first integration attempt hit the UNIX socket path-length limit in its ambiguity fixture; shorter project-local fixture paths fixed that test setup. Only the final complete8-case run counts. Reproduce with python3 scripts/native-startup-smoke.py from the active Hyprland session. Final release staging/audit is recorded in PROGRESS.md.


Final0.4.2 release audit passed (analysis/gtk-startup/artifact-smoke.json):40 archive entries, portable/Arch binaries equal native-tested hashes, all runtime libraries found, correct0.4.2-1 metadata, unchanged installer hash and no proprietary skin/binary/analysis tree bundled. The actual staged portable command was launched with TTY display variables missing and mapped=true/xwayland=false. No hardware writes/system configuration changes were performed.

## Public continuous integration

GitHub Actions uses Rust 1.93.1, matching the compiler used for the tested 0.4.2 native binaries. The first run with moving stable Rust 1.99 passed all 96 Rust tests and 37 frontend tests, then stopped on the newly introduced stylistic `chunks_exact_to_as_chunks` lint. The release pipeline pins the tested toolchain so unrelated lint additions cannot silently change release acceptance. Upgrading that compiler is a separate reviewed maintenance task. This does not change protocol behavior or hardware validation.

## 0.4.3 distribution packaging

Native x86_64 binaries are built in `packaging/linux/Dockerfile`, using Debian 12, Rust 1.93.1 and Node 24. The build container receives only a clean source checkout, no keyboard devices, host display sockets, home directory or secrets. The host Docker bridge was disabled; the local build used temporary host networking for dependency downloads without changing Docker or host network settings. No physical configuration writes occurred.

The resulting executable passed the full native WebKit GUI/demo/profile/macro suite on Arch, and all eight native Wayland/TTY startup cases. The 96 Rust and 37 frontend tests, strict Clippy and formatting pass. The Debian-baseline executable's highest referenced GLIBC symbol is 2.34; packages conservatively require the Debian 12 baseline, glibc 2.36+. Distribution compatibility still depends on GTK/WebKitGTK and package-manager dependencies.

`Package installation` is a manual GitHub Actions matrix: Debian 12, Ubuntu 24.04 and Fedora 43. It verifies release checksums, installs each native package with its package manager in a fresh disposable container, starts the actual GTK window as UID1000 under Xvfb, checks the read-only CLI, then uninstalls and verifies removal. No USB device is passed. Standard Docker seccomp prevents WebKit's extra namespace sandbox; the smoke test disables that extra sandbox only inside the disposable device-free test container. Users are not instructed to disable their desktop sandbox or security settings.

See the [0.4.3 release notes](https://github.com/Dahuum/aula-studio/releases/tag/v0.4.3) and [package-validation workflow runs](https://github.com/Dahuum/aula-studio/actions/workflows/package-validation.yml) for completed matrix results. A committed workflow alone is not a passing test. These tests check package resolution, native startup and uninstallation, not physical USB configuration or a full interactive desktop session on each distribution. Arch's real desktop/Wayland tests remain distinct from container coverage. Mint and other derivatives are compatible targets, not separately tested desktops.

No npm, AUR, apt/dnf repository or Flathub registration is implied by source files or an installation recipe. No ARM, Alpine/musl, AppImage or Flatpak binary is claimed.

## Unreleased 0.6 development verification

124 Rust unit/integration tests, 55 frontend tests and 12 Python checks pass. Formatting, Clippy (`-D warnings`), TypeScript/Vite and optimized native build pass. New tests cover exact-model identities, standard378 versus legacy384 RGB transfers,126-byte plane strides, missing config-tail rejection before subsequent IO, immutable capture storage and profile-schema isolation. Frontend tests retain late-result/model-switch locking and reject incomplete/cross-model DTO shapes.

Native WebKit/Tauri integration passed for all nine local layouts and actual Rust F87/F108/H108 synthetic capture/storage/lighting-projection workflows, raw JSON export with an actual file download, demo-mode physical-read rejection, private file modes and separate profile namespaces. Private evidence: `analysis/native-model-smoke/1793747684698905558/result.json`; native GUI SHA256 `dbc25721cfac43e6fe9b879614bdc4244b6417db8d57442cea9889bd46fb873e`. No IPC mocks, real-device queries or hardware configuration writes were used by this suite.

The full native F75 demo GUI regression passed (`analysis/native-demo-smoke/1793747714980527705/result.json`), including lighting, per-key/Fn/media/local macro/profile flows. All eight native startup cases passed (`analysis/native-startup-smoke/1793747722855473253/result.json`), including actual Wayland windows, both TTY recovery paths and clean error cases. No F87/F108/H108 physical reads or writes have been validated. Passive metadata enumeration found no eligible wired read candidate during this run; this does not establish the user's current connection mode at another time. S75 live queries remain disabled. Published0.5 package-install results above apply only to0.5, not this host-built development binary.

## Macro continuation — unreleased0.6

128 Rust tests,62 frontend tests,12 Python checks, Clippy/fmt/TypeScript/Vite and optimized build pass. New fixture tests cover after-event keyboard/modifier encoding, adjacent/trailing delay preservation, unsupported leading silence/usages/names,20-bit overflow and4KiB default-pool bounds. Local GUI tests cover event edits/reorder/insertion, independent duplication, pagination and preview cancellation on profile replacement. Native actual Rust wire inspection passes; the toolbar's first hit-area failure was fixed and the actual native click test rerun successfully.

Final GUI SHA256 `4cab78c6326ac2d87d8e97dd32b553c136db9b5263586ba20588520a5ba1b50c`. Full native F75 demo/edit/inspection regression: `analysis/native-demo-smoke/1793749345019481883/result.json`. Multi-model regression: `analysis/native-model-smoke/1793749369449764855/result.json`. All8 Wayland/TTY startup cases: `analysis/native-startup-smoke/1793749379023603147/result.json`. All tests isolate storage/block hardware writes. No actual macro HID query, upload or playback was performed. These new source changes are not in the older staged development archive or public0.5 release.

## Original layer tabs follow-up (unreleased0.6)

Optimized GUI f6f935721f9e660ea688a50dd55a09ae02b57271884de8410ddade55379a1bba passed native F75 integration, all ten local model workspaces, and eight startup/Wayland/TTY/error cases. Layer tests check85×45 dimensions,1px row spacing through measured bounds, right alignment with actual key rectangles,10px vertical gap and notification-time hit targets. Existing frontend regression covers pressed-state navigation and protected Fn. Native tests isolate app data and block physical writes. Earlier failed geometry and asynchronous save/list runs are retained; the final passes are native-demo1793836285231860300, native-model1793836299256374894 and native-startup1793836322751906999 under private analysis. Packaged F98 and Debian-baseline artifacts predate this UI refinement and retain their original test identities.

## Fourteen media actions (unreleased0.6)

142 Rust,73 frontend and16 Python checks pass, along with rustfmt/strict Clippy/TypeScript/Vite and optimized app/CLI builds. Exact native DWORD fixtures cover fourteen actions and both layers. Mock tests exercise all28 action/layer combinations, comparing complete bank contents and preserving an opaque unrelated record. Backend read projection rejects unsupported high-byte usages/flags/chords without corrupting raw data; profiles retain16-bit values and reject truncation aliases. Frontend reviews preserve high-byte values and never commit on local assignment.

Native GUI89e763989faa49ab707ec1ccd408b8497e078e7b2b4b9b98fe0055614f0459b5 passes F75 demo→actual Rust storage→export/import with browser-home/calculator alongside play/mute, all10 model workspaces and8 startup/Wayland/TTY/error cases. Private result IDs: demo1793836759482810022,model1793836774305839475,startup1793836782658792261. An initial existing focus assertion raced React effects; waiting for actual focus resolves the harness race, and the original failed run is retained. Tests block physical writes. No physical media event, new model access or power-cycle persistence is verified.

## Normal mouse assignments (unreleased0.6)

145 Rust,76 frontend and16 Python checks pass, plus rustfmt/strict Clippy/TypeScript/Vite/optimized builds. Exact five-record fixtures and both-layer roundtrip reject unknown buttons and preserve noncanonical repeat/tail forms. Combined media/mouse mock transactions cover38 action/layer combinations, full affected-bank equality and opaque unrelated records. Service/backend tests cover old-profile compatibility, cross-type/numeric-alias conflicts, exact read projection and review names; frontend tests verify local replacement and review-only native requests.

Native GUI95458ae76ee589f1ae48321dab04e8f246fea071dfde2f2944d9881a9592e768 passes actual Rust F75 demo/profile/export/import with normal mouse assignments in both layers, all ten model workspaces and eight startup/Wayland/TTY/error cases. Private results demo1793837460010341374,model1793837503506428169,startup1793837504639171106. The initial obsolete blanket type1 rejection failure is retained; it now tests invalid button6, and exact native fixtures establish buttons1–5. No physical mouse event, hardware write or persistence is inferred. See [mouse evidence and gaps](MOUSE_BINDINGS.md).
