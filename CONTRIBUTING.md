# Contributing to AULA Studio

Useful contributions include original-F75 compatibility reports, reproducible bugs, accessibility/UI improvements, protocol research with provenance, and tests that catch real transport failures. The exact-original UI recreation remains unfinished; see [ROADMAP.md](docs/ROADMAP.md).

## Set up and check

Follow [INSTALLATION.md](docs/INSTALLATION.md), then:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
npm --prefix app test
npm --prefix app run build
```

Use `AULA_DISABLE_HARDWARE_WRITES=1` for native QA. Demo and mocks cover ordinary development without a keyboard. `python3 scripts/native-demo-smoke.py --fallback` checks the native app; `--xdg` checks separate storage roots. Startup integration tests require Hyprland and deliberately omit display variables.

CI and ordinary tests must never configure hardware. The separately authorized brightness test already completed; do not run it on reconnect or as a general regression test.

## Report a bug or compatibility result

Include the app version, distribution/desktop, connection, exact model/layout/revision if known, reproduction steps, expected behavior and result. Distinguish demo behavior, visible hardware behavior, readback and power-cycle persistence.

Sanitize diagnostics. Never attach the Windows installer, proprietary files, raw backups, serials, personal paths or secrets. A concise report is better than an unreviewed data dump.

## Protocol changes

- Read [HARDWARE_SAFETY.md](docs/HARDWARE_SAFETY.md) and [PROTOCOL.md](docs/PROTOCOL.md).
- Provide exact evidence sources, packet fields, confidence and compatible revisions. Label hypotheses.
- Keep packets in Rust, separate from UI/business logic. Add independent fixtures and meaningful bounds/malformed/transaction tests.
- Preserve unknown bytes, protected sources, connection checks, backups and readback.
- No arbitrary commands, address sweeps, firmware/reset operations or speculative macro uploads.
- Mock tests do not establish hardware compatibility. Keep unverified features clearly labeled or unavailable.

## Pull requests

Keep changes focused. Explain behavior, evidence, checks and limitations. Preserve attribution/licenses; do not copy GPL/unlicensed or proprietary material into the MIT project.

AI-assisted contributions are welcome when contributors understand, review and can support them. Disclose substantial assistance and independently verify claims. Never submit invented protocol details or test results. This project itself was developed with substantial Codex assistance.

Be respectful. Do not pressure testers into unsafe experiments. Original contributions use the project's MIT license unless explicitly documented otherwise.
