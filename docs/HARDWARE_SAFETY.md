# Hardware safety — 0.2

The old0.1 blanket-disabled write policy was replaced with bounded, reviewed transactions. No operation runs automatically on startup, hotplug or profile save/load. Standard tests block writes; the separately authorized bounded brightness harness is the explicit exception. The CLI has no writes. Native QA sets AULA_DISABLE_HARDWARE_WRITES=1, which blocks both physical commit entry points.

Preparation validates a profile, reads the relevant hardware banks, constructs a typed plan that preserves unrelated bytes, and durably saves a0600 snapshot. The GUI presents device, scope, changed fields, backup and test status. A separate explicit Write action consumes a random single-use token (120s expiry). Commit verifies the backup, device connection epoch, VID/PID/interface/descriptor and current complete bank before writing. Mode changes invalidate reviews.

Only documented config04, palette0a, keymap03 and planar06 operations are reachable through private real transports. All payloads come from validated snapshots and reviewed supported changes. Key disabling and reserved Fn/F-source edits are rejected. No arbitrary command/address, firmware mode, calibration sweep, reset, erase or bootloader path exists.

After writing, exact config/palette/affected-bank readback is required before reporting success. An error, disconnect or mismatch is unconfirmed; no retry or rollback is sent. A before-state backup and operation journal remain. Readback is not power-cycle persistence. Restore is separately reviewed and limited to known preset/sleep fields.

Current instances use process and OS advisory locks; deadlines gate each HID operation. Kernel waits may outlive UI timeouts; locks remain held until completion. Other configurators are not controlled, so avoid concurrent use.

The device rule grants active-seat uaccess for wired258a:010c, never0666. No rule, daemon or system setting was installed automatically. Never execute supplied Windows code on the host. Proprietary files remain ignored and outside packages.

## Scoped validation authorization2026-11-02

The user explicitly authorized the backed-up one-step brightness/readback/restoration test, superseding the original no-automatic-writes restriction for that bounded test. See HARDWARE_VALIDATION.md. The test uses production GUI review/write actions and a brightness-only typed plan, with at most two04 writes; all other banks are compared. Receiver-only state is rejected. No firmware, reset, unknown command or blind rollback is authorized. Physical-test consent does not itself establish successful hardware verification.

## ANSI bottom-row correction

Do not use0.1–0.3 mapping/per-key binaries. The supplied French package uses RightAlt53/Fn59, while the saved local ANSI base read identifies Fn53 with record0D000000 and RightCtrl59 with record00100000. Corrected source protects Fn53 and maps physical RightCtrl usage228 to59. Physical mapping/per-key reads and commits now verify the base Fn marker and reject mismatched regional/revision layouts before mutation. Original bank reads remain evidence of bytes, not a validation of the earlier incorrect role labels. No mapping/per-key write was performed during development. The later authorized brightness6→5→6 test passed with exact all-bank restoration; see TESTING.md.

## Additional models

Model library discovery is metadata-only and retains ambiguous USB candidates. Native F75 configuration operations verify its exact six-byte identity under the same descriptor/connection lock. The diagnostic identity command is limited to a statically established bounded G5 read; it is not arbitrary probing. All additional model editors save a separate incompatible local profile format. Offline S75 encoders have no transport. No new model inherits HID writes from a USB match, and no new-model udev permissions are installed.
