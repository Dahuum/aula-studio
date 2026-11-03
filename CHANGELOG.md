# Changelog

## 0.5.0

- Add an evidence-bounded model library and native local workspaces for nine new resource-derived keyboard variants. Model-bound profile storage/history/import/export, local colors/assignments/Fn intent and layout-aware macro recording are independent of the F75 hardware path.
- Inventory31 official download entries and acquire12 eligible direct Windows packages with source URLs/hashes, excluding hosted/browser/firmware entries. Retain private binary analysis and publish original factual findings only.
- Use REA/Ghidra to recover F87 identity/transport and S75 lighting/keymap/macro framing; corroborate G5 structures through bounded static Hub analysis. Add tested offline G5 identity/S75 codecs.
- Require the original F75 six-byte identity before configuration access, preventing other keyboards sharing258a:010c from inheriting its controller. Exact local F75 identity read succeeded; no new configuration writes performed.
- Keep all additional model hardware access, onboard macro uploading and wireless configuration disabled. Offline tests and native GUI workflows do not establish device compatibility.

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

## 0.4.3

- Build native Linux packages against Debian 12 / glibc 2.36; add Debian and Fedora formats alongside Arch and portable builds.
- Document distribution-specific installation, updates, removal and unavailable registry listings.
- Replace landing-page branding and layout with a typographic design, direct package choices and actual application screenshots.
- Document evidence levels and physical-validation requirements for researching additional keyboard models.
