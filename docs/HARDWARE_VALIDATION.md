# Bounded hardware validation plan

Authorization: on2026-11-02 the user explicitly replied “Yes—authorize reversible configuration tests” to the backed-up one-step brightness change, readback and restoration proposal. This supersedes the original prohibition for this bounded test; do not ask for the same permission again. The wired F75 reappeared during the0.4.1 follow-up, and the authorized brightness test completed successfully. A tool flag alone is not authorization for a future unrelated session or operation.

## First proposed test

1. Require exactly one wired258a:010c configuration interface, matching report06/520 bytes and a fresh connection identity. Receiver configuration is excluded.
2. Read and durably back up config/palette/base/Fn/per-key banks. Save the restoration instructions before any write.
3. If the reported preset supports brightness1..18, choose a one-step brightness change (decrease by1 when above0, otherwise0→1). Abort for off/custom/unknown layouts; do not change effect to manufacture a test.
4. Prepare through the production brightness-only operation and verify its plan changes exactly one known brightness byte. Preserve mode flag9, effect10, group11, color selector, speed, sleep, palette and input mappings.
5. With the recorded explicit scoped authorization, perform the actual native GUI Write action once, verify exact readback and compare all other banks. No retries or undocumented commands.
6. Restore the original brightness through a separately reviewed operation using the saved snapshot; verify exact baseline bytes. This restoration is part of the proposed test, not an open-ended automatic rollback policy.
7. If disconnect, timeout or any unrelated state change prevents matching the expected state, stop writing and retain backups/journals. Do not blindly replay a snapshot over a changed keyboard.

Maximum intended configuration writes: one change plus one restoration, both documented04 config reports. No firmware/reset/erase/mode-switch operation, remapping, macros or receiver commands. USB reads and offline packet/unit tests remain authorized. Readback verification must not be presented as visible LED verification or power-cycle persistence.

The helper `ConfigRegion::with_preset_brightness` and `LightingPlan::prepare_brightness` preserve every other byte, including the observed retained flag9 state. The native `prepare_brightness` command creates its normal durable backup and review token; physical commit remains guarded and separate.


## Completed test,2026-11-02

Guarded dry-run: passed, zero writes, all five banks unchanged. Authorized native GUI run: passed, brightness6→5→6 on effect2/config payload60, exactly two brightness actions and readback_verified journals. Final config/palette/base/Fn/per-key bytes equal the saved baseline. Evidence: analysis/release041-brightness-hardware.log and private analysis/brightness-validation/1793645774584079067, including tested executable hashes. No visible-brightness measurement or power-cycle test is claimed. This completes the originally proposed bounded test; do not automatically repeat writes just because the device reconnects. Other scopes retain independent validation status.
