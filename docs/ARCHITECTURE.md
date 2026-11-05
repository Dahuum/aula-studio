# Architecture — 0.4

React editor → Tauri prepare/review/commit commands → typed plan + durable backup → locked HID transport → physical F75. Demo Apply stays in memory. Profile Save stays in local storage.

`aula-protocol` is IO-free: packet framing, signatures, standard config/sleep fields, effect palette, planar RGB, ANSI matrix mapping, keymap records/modifiers and offline raw macro codec. `aula-hid` has real reads, private typed writes and mockable transaction traits. Lighting/setting/restore plans and mapping plans reconstruct their intended mutations on validation, reject stale baselines, preserve unrelated bytes and require exact readback. No arbitrary command transport exists.

Tauri keeps random review tokens in memory,120s/single-use, invalidated by mode/preferences/new preparation. A commit verifies its durable backup, consumes the token before blocking work and journals started/readback_verified/unconfirmed. A lost or failed commit never silently retries. No successful hardware state is inferred from local saving or simulation. Physical entry points honor AULA_DISABLE_HARDWARE_WRITES for native QA.

`aula-service` validates schema1 profiles with backward-compatible optional multicolor/sleep/consumer-layer fields. It uses protocol media names/allowlist without depending on HID access. Atomic0600 saves use fsync and retain history. Import gets a new ID. Runtime data uses XDG_DATA_HOME/aula-studio, override AULA_DATA_DIR. Backups/journals live there. Preferences use XDG_CONFIG_HOME/aula-studio; legacy data-directory preferences are read without modification and migrate on the next explicit preference save. AULA_DATA_DIR redirects both config and data for QA. Current app instances coordinate HID through process mutex and user runtime/cache advisory lock; other tools may ignore it. Device connection epoch is sysfs-derived and rechecked before opening.

Long reads/commits run off the GUI thread; status enumeration polls every3s. Tauri bundles frontend assets and uses GTK/WebKitGTK; no network/shell plugins. CSP permits IPC only. Diagnostics omit serials/unrelated devices. Macro recording is focused-editor only, and preview never injects global input. No privileged service, firmware/reset path or mandatory browser.


The frontend renders a measured1200×720 canvas scaled to the available window. Main/context navigation, keyboard interaction and original-position lighting controls call the same Rust operations as before. local_ui_assets reads only allowlisted bounded PNGs of expected dimensions from an explicitly configured private asset root or the project extraction beside the executable; no arbitrary frontend path and no network request are accepted. Assets are optional and never staged into release archives. AULA_NO_ORIGINAL_SKIN disables them; AULA_ORIGINAL_SKIN_DIR selects the extraction root containing skins/. Window actions are limited to drag/minimize/close on the invoking window. Color palettes persist as at most10 RGB triples in private XDG preferences.

Matrix reads and mutations first verify the fresh base bank has the observed ANSI Fn marker at53 and no Fn marker at59. This prevents silently applying the supplied French role table to ANSI hardware. Lighting-only operations remain separate from these layout-specific gates.


Linux startup preflight (startup.rs) runs before store initialization/background work. It keeps explicit display settings, can recover a unique current-user Wayland session for TTY launch without DISPLAY/WAYLAND_DISPLAY, and fails safely on ambiguous/untrusted runtime directories. It modifies only the app process environment, initializes GTK once on the main thread and reports actionable errors before Tao constructs its event loop. No shell/compositor configuration is written. Unit planning tests avoid global environment mutation; native child-process tests cover actual successful windows and clean failures.

## Model library and identity boundary —0.5

`aula-models` embeds a validated factual registry from `data/models.json`. It has no HID dependency or handles. It supplies typed model-specific local drafts, an incompatible profile schema/namespace, bounded validation and atomic history-backed storage. The native Device library calls these local services; it never calls legacy F75 prepare/commit commands. The macro editor accepts the selected layout rather than assuming F75 keys. Browser preview has an equivalent explicitly offline namespace.

The IO-free G5 identity codec encodes only the bounded six-byte read. The HID layer uses it after descriptor/connection checks and under the existing operation lock; configuration callbacks run only when identity matches original F75. The timeout includes identity verification. USB metadata discovery keeps multiple model candidates and never chooses from a shared VID/PID. S75 offline framing/scalar/per-key codecs have no transport and cannot establish F108 Pro compatibility.

## Model capture adapters — unreleased 0.6

Device library → Tauri capture commands → `aula-hid::model_read` → finite `aula-models::g5_capture::ReadRequest` → locked physical transport. Only independently established F87/F108/H108 contracts are allowlisted. Each operation validates the exact live identity before bank queries, and each response before continuing. Model-specific RGB lengths and config markers prevent cross-model decoding. No setters exist in this adapter. CLI uses the same backend.

`CaptureStore` stores immutable private raw captures separately from both profile schemas and F75 backups. Projection creates an unsaved local lighting-only draft; unknown mappings/Fn/macros remain unprojected. Synthetic fixture generation traverses the actual Rust decoder/store with source `simulated`; GUI demo mode independently rejects physical captures before device access. The frontend holds its operation lock until the pending native promise ends, even when ignoring results or switching models. Exact USB product-name matching narrows passive discovery candidates only; descriptor and model-identity checks remain independent.

## F75 action records — unreleased0.6

KeyAction retains a device record type and16-bit value. Type0 validates one-byte keyboard targets and modifier encoding; type2 validates the fourteen native Consumer selections and preserves both usage bytes; type1 validates five normal mouse-button codes and emits only canonical01/button/01/00 records. The type field is not a USB usage page. Profile keyboard/consumer/mouse maps are separate optional namespaces with per-layer source-conflict checks. Read projection requires exact record equivalence before exposing an editable action, preserving unknown usages, flags, chords and repeat forms in raw backups. All supported actions share the existing typed matrix transaction, protection/backup/review/stale/readback boundaries. No host event injection or new hardware command is involved.
