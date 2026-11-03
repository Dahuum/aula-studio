# Changelog

## 0.4.2 — 2026-11-02

- Recover native Wayland startup from a TTY without display variables when one user-owned display is available.
- Preserve explicit display choices; reject ambiguous or untrusted runtime paths.
- Replace GTK initialization panics with useful errors before app data opens.
- Add eight native startup cases and four Rust planning tests.

## 0.4.1 — 2026-11-02

- Refine sliders, checkboxes, keycap shading, USB connector and navigation icons.
- Physically verify brightness 6 → 5 → 6: both writes read back, all five banks restored exactly.
- Observe actual hardware reviews/journals without replacing Tauri's immutable IPC function.
- Keep hardware validation and unfinished UI parity visible.

## 0.4.0 — 2026-11-02

- Recreate the main Windows-app layout with independent graphics and optional private original artwork.
- Correct ANSI Fn/Right Ctrl roles and gate matrix operations on the observed Fn marker.
- Preserve reviewed configuration operations, local macros, profiles, diagnostics and demo mode.

Older 0.1–0.3 builds have the corrected bottom-row error and must not be used for mapping/per-key operations. No mapping/per-key write was performed during that earlier development.
