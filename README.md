<div align="center">

# AULA Studio

### Native Linux software for the original AULA F75.

Native configuration for the original ANSI AULA F75.<br>
Rust · Tauri · React · Wayland · MIT

**Early access · v0.4.3 · USB wired configuration**

[Website](https://dahuum.github.io/aula-studio/) · [Downloads](https://github.com/Dahuum/aula-studio/releases) · [Get started](#get-started) · [What works](#what-works) · [Documentation](#documentation) · [Contribute](CONTRIBUTING.md)

<img src="docs/screenshots/lighting-demo.png" width="1000" alt="Native Linux lighting editor with the F75 keyboard, effects, color wheel, brightness and RGB controls">

*Actual native application in explicit demo mode, using independently drawn artwork.*

</div>

AULA Studio brings F75 lighting, configuration reads, profiles and local editing tools to a Linux desktop. It uses the existing Linux HID interface: launch as your normal user, without Wine, a VM, a browser configurator or a privileged daemon.

This is an independent community project, **not affiliated with AULA**. It is under active development. Hardware coverage is deliberately narrow, and implemented controls are not all physically validated.

## What works

| Capability | Current status |
| --- | --- |
| Wired detection; configuration, palette, base/Fn and per-key bank reads | Verified on physical hardware |
| Theme switching | Visually confirmed by the keyboard owner |
| Scoped brightness change and restoration | Physically tested: **6 → 5 → 6**, both writes read back, all five banks restored exactly |
| Preset effects/colors, brightness/speed, per-key RGB, base/Fn keyboard and seven media assignments, sleep and scoped restoration | Real reviewed HID transactions implemented; other write scopes still need physical validation |
| Profiles, import/export, history, preferences and diagnostics | Implemented and software-tested |
| Local macro recording, editing, import/export and offline preview | Implemented; preview does not inject keys into other applications |
| Onboard macro uploads; wireless configuration | **Unavailable** |
| Debounce, polling, OS/revision settings and broader shortcut bindings | Under investigation |

**96 passing Rust tests and 37 passing frontend tests**, plus native GUI and startup integration checks, are separate from hardware validation. See the [compatibility matrix](docs/FEATURE_MATRIX.md), [test record](docs/TESTING.md) and [remaining work](docs/ROADMAP.md).

**Supported target:** original **ANSI F75**, USB wired mode, `258a:010c`, configuration interface 1 with report `0x06` / 520 bytes. Tested development environment: Arch Linux, Hyprland and Wayland. Other F75 variants, ISO layouts, revisions, Bluetooth and receiver configuration are not assumed compatible. Receiver detection alone does not establish wireless configuration support.

## Get started

### Install a native package

Download your package from **[Releases](https://github.com/Dahuum/aula-studio/releases)**, verify it against that release's `SHA256SUMS`, then run:

| Distribution | Install the downloaded package |
| --- | --- |
| Arch Linux and compatible derivatives | `sudo pacman -U ./aula-studio-0.4.3-1-x86_64.pkg.tar.zst` |
| Debian 12+, Ubuntu 24.04+ and compatible Mint releases | `sudo apt install ./aula-studio_0.4.3_amd64.deb` |
| Fedora with WebKitGTK 4.1 | `sudo dnf install ./aula-studio-0.4.3-1.x86_64.rpm` |
| Other x86_64 GNU/Linux | Extract the portable tarball and run `./aula-studio-0.4.3-linux-x86_64/bin/aula-studio`; system GTK/WebKitGTK libraries required |

Then launch **AULA Studio** from the desktop menu or run `aula-studio`. Run it as your regular user. Packages include the desktop entry, icon, CLI and narrow wired-F75 udev rule. Reconnect the keyboard after installation. **Node/npm, Rust, Wine and REA are not runtime requirements.**

These binaries are built against **Debian 12 / glibc 2.36**. They do not cover every Linux distribution: Alpine/musl, older glibc releases and ARM binaries are outside the current binary target. See the [installation guide](docs/INSTALLATION.md) for exact prerequisites, checksums, test coverage, USB access, updates and removal.

**Registry status:** no npm package, AUR entry, Flathub listing or hosted apt/dnf repository is published. Native files are available from GitHub Releases. A local [AUR-style binary recipe](packaging/aur/PKGBUILD) is provided; this is not a registered AUR package. Never use an unrelated similarly named package as a substitute.

### Build from source

Use Rust 1.93.1 (the release-tested compiler) or a compatible newer version, Node 24/npm, Python 3.11+ and native libraries. Distribution-specific prerequisites are in the [guide](docs/INSTALLATION.md).

```sh
git clone https://github.com/Dahuum/aula-studio.git
cd aula-studio
npm --prefix app ci
npm --prefix app run build
cargo build --locked --release -p aula-studio -p aula-cli
./target/release/aula-studio
```

The frontend is embedded; no development web server is needed. Version 0.4.2 fixed TTY launch when a unique trusted user-owned Wayland display is available. Neither release changes your shell or compositor settings.

## Try it without a keyboard

Choose **Enter demo mode** at the bottom of the sidebar. Explore the editors, save profiles and record local macros. Demo Apply changes only an in-memory simulation and is labeled accordingly.

<details>
<summary>See the local macro editor</summary>

![Native local macro editor in demo mode](docs/screenshots/macros-demo.png)

Local and onboard macros are distinct. Hardware uploading remains disabled until its protocol and recovery behavior are established.

</details>

## Apply a hardware change

1. Connect the supported keyboard in **wired mode** and exit demo mode.
2. Edit the relevant page. **Save profile** saves locally; it does not configure hardware.
3. Choose **Review … write**. Preparation reads the affected configuration and saves a durable backup.
4. Check the device, exact changes, scope and backup, then explicitly choose **Write … to keyboard**.
5. Success requires matching readback. Power-cycle persistence is still unverified.

Reviews expire after 120 seconds and are single use. Startup, hotplug and profile save/load never apply configuration. Failures are reported as unconfirmed, without automatic retry or rollback. The CLI has no configuration-write commands.

Known ANSI Fn and function-key input sources are protected. Scoped restoration preserves unknown bytes and protected entries; it is not a factory reset or arbitrary memory clone. Read the [hardware safety contract](docs/HARDWARE_SAFETY.md) before exercising an unvalidated scope. Do not run multiple configurators against the same device concurrently.

> **Upgrading an older development build?** Use version 0.4 or newer for mapping/per-key operations. Versions 0.1–0.3 had an incorrect French-derived ANSI bottom-row assignment; current versions protect Fn at slot 53 and map Right Ctrl to slot 59. No mapping/per-key write was performed during that earlier development.

## Diagnostics

```sh
./target/release/aula devices
./target/release/aula diagnostics
./target/release/aula validate path/to/profile.json
```

Enumerate before using a path: `/dev/hidrawN` changes after reconnection. Bounded reads are documented in the [installation and diagnostics guide](docs/INSTALLATION.md). Sanitize reports; do not upload the installer, serials, personal paths or full hardware backups.

## Development

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
npm --prefix app test
npm --prefix app run build
```

Hot reload: `cd app && npm run tauri dev -- --no-default-features`.

Hardware communication is separate from the UI. `aula-protocol` owns typed packets and validation, `aula-hid` provides transport and transactions, and `aula-service` owns profiles and persistence. Tauri manages reviews, backups and journals. Mock transports and explicit demo mode keep ordinary tests off the keyboard.

Read [CONTRIBUTING.md](CONTRIBUTING.md) before adding protocol operations. Standard automation must block physical writes. A successful mock test is not hardware verification.

## Documentation

- [Install, permissions, troubleshooting and uninstall](docs/INSTALLATION.md)
- [Compatibility](docs/FEATURE_MATRIX.md) · [Limitations](docs/LIMITATIONS.md) · [Roadmap](docs/ROADMAP.md)
- [Protocol](docs/PROTOCOL.md) · [Architecture](docs/ARCHITECTURE.md)
- [Reverse-engineering evidence](docs/REVERSE_ENGINEERING.md) · [Sources and attribution](docs/SOURCES.md)
- [Hardware safety](docs/HARDWARE_SAFETY.md) · [Testing](docs/TESTING.md) · [Changelog](CHANGELOG.md)
- [Security reporting](SECURITY.md)

## License and provenance

Original code is [MIT licensed](LICENSE). Attributed MIT F87-family fixture buffers retain their own [license](fixtures/keymap/LICENSE) and are never treated as F75 factory defaults. Dependencies retain their respective licenses.

No proprietary installer, EXE/DLL, decompiler output or original artwork is distributed. Optional user-supplied artwork is read locally; the app also works with independently drawn assets. The original UI recreation is **unfinished**, particularly keyboard artwork/fonts and several panel layouts. This project does not claim full Windows-software parity.
