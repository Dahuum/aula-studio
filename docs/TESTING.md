# Validation through0.4.2,2026-11-02

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
