# Remaining work

1. Finish the exact-original UI request. Main canvas/layout is only partial parity: independently drawn ANSI keyboard/keycap geometry, font metrics, original remapping action picker, macro editor, global/device settings and Config dialogs still need screen-by-screen reconstruction and comparison.0.4.1 refines level/RGB slider handles, checkboxes and keycap bevel/cable; it is not full UI parity.
2. Human physical validation of the implemented reviewed writes: theme switching has user confirmation and scoped brightness6→5→6/readback/all-bank restoration passed. Remaining checks include other effects/color controls/ranges and controlled reconnect/power-cycle persistence. Repeat for per-key, base/Fn mapping, sleep and scoped restore. Standard tests block writes; the completed user-authorized bounded brightness validation is documented separately and must not be automatically repeated.
3. Extend known-entry bank restoration to other layouts/revisions, investigate consumer actions beyond the fourteen native media selections, arbitrary shortcuts and regional/revision capability mapping. The fourteen media/application/display and five normal mouse-button encodings are implemented; their physical behavior remains unverified.
4. Onboard macros: validate physical hardware playback timing (application after-event placement is now resolved), exact repeat binding semantics, count/capacity, allocation and interrupted-write recovery using static callers/recording and controlled opt-in traces. Raw codec and bounded local→wire inspection are implemented; uploading remains disabled.
5. Establish receiver framing/state and revision checks; never infer wireless settings from wired success.
6. Establish debounce/OS/polling/firmware query semantics before exposing controls.
7. Expand native automation, accessibility/file-dialog UX, log presentation/rotation, faithful effects and packaging CI.

## Model expansion priorities after0.5

1. Reconcile exact revisions/identity and obtain consenting owners' descriptors/bounded read traces for F87/F108/H108/S75 families.
2. Verify S75 stored color channels, selector/commit response semantics and04F5 nine-page backup versus eight-page color writes; finish safe identity/read/restore specification before enabling transport.
3. Resolve S99 conflicting indices and F98 physical geometry/complete backup/response contracts. F98 scalar/per-key codecs and local lighting are now independently recovered; physical access remains disabled.
4. Decode native macro action/timing/binding/slot limits and independently cross-check Hub SDK semantics; keep onboard upload disabled until complete and hardware-tested.
5. F2088 extraction is complete: finish its driver/host/onboard separation, missing layout roles and safe device read/backup contract. Continue per-package native analysis, mouse adapters and exact-device validation. Do not treat acquired files or local editors as shipped hardware support.
