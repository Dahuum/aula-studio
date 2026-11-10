# Native desktop platforms

The cross-platform port is under validation. Windows and macOS packages must be built and launched on their native runners before download links are published. Native runner testing does not establish physical AULA compatibility on those operating systems.

## Architecture and safeguards

Protocol encoders, profiles, macros, model workspaces and reviewed configuration services are shared Rust/TypeScript code. hidapi uses Linux hidraw, Windows native HID and non-exclusive macOS IOHID backends. The macos-shared-device feature prevents exclusive keyboard seizure. No Windows vendor executable, driver or firmware tool is used at runtime.

Linux reads the bounded report descriptor from sysfs. Windows/macOS read the operating system descriptor through the vendor collection handle; discovery sends no HID feature commands. Configuration requires original wired258a:010c interface1, vendor usageff00/1, report06 length520 and an OS connection identity. Linux uses its sysfs attach identity; macOS uses the IORegistry entry ID supplied by hidapi; Windows reads the PnP LastArrivalDate with Configuration Manager. Missing metadata fails closed. Connection identity is rechecked after opening the handle. The exact six-byte F75 identity remains mandatory before configuration. Read-only model adapters keep their independently established identities and bank bounds.

All existing scoped backup/review/token/stale-state/readback controls remain. Operating-system metadata is not new keyboard protocol evidence. Macro uploading, firmware/reset paths, wireless and new-model configuration writes remain unavailable. No new physical write was performed during this port.

## Platform integration

- Linux retains the existing Wayland/GTK startup recovery and XDG directories.
- Windows uses per-user local application data and roaming configuration, native OS randomness and an ordinary file lock. The NSIS installer targets the current user and uses Microsoft WebView2. No service/driver replacement is installed.
- macOS uses Application Support/cache locations, native OS randomness and shared HID access. Packages target macOS13+ on Apple Silicon and Intel. Native jobs run on macOS15; older supported OS versions still require owner validation.
- AULA_DATA_DIR and the hardware-write guard remain available for isolated testing. File data is flushed before atomic replacement; directory fsync is used on Unix where supported.

Windows previews are unsigned; macOS previews are not Developer-ID signed or notarized. No signing identity or certificate is fabricated. The download page must state this accurately without instructing users to disable operating-system security globally.

## Native verification pipeline

.github/workflows/native-desktop.yml builds on Windows2022/x86_64 and macOS15/AppleSilicon plus Intel. It runs frontend and Rust tests, formatting/Clippy, builds the native CLI and installer, then uses scripts/native-platform-smoke.py to verify the actual GUI window and read-only CLI in isolated demo storage with hardware writes disabled. Artifacts include checksums/build metadata and native smoke evidence. Our own compiled applications/installers may be executed for QA; supplied AULA Windows binaries remain excluded and are never executed.

scripts/package-desktop.py includes license and actual target dependency notices in the installers and portable diagnostics packages. No installer or extracted vendor asset is included in the public source snapshot. Native build/window tests, offline protocol tests and physical verification are distinct evidence levels. Current verified hardware evidence remains the earlier Linux wired theme/brightness observations; Windows/macOS physical configuration remains unverified.

## Reproduce native installers

Run on the matching operating system with the prerequisites above. Replace TARGET with `x86_64-pc-windows-msvc`, `aarch64-apple-darwin` or `x86_64-apple-darwin`.

```sh
npm --prefix app ci
npm --prefix app run build
cargo fetch --locked
cargo build --release --locked --target TARGET -p aula-cli
python scripts/package-desktop.py prepare --target TARGET
```

From `app/`, run `npm run tauri build -- --target TARGET --bundles nsis --config ../dist/native-resources/tauri-native.json` on Windows, or substitute `--bundles app,dmg` on macOS. From the repository root, `python scripts/package-desktop.py package --target TARGET` creates canonical filenames, target-specific notices and SHA256SUMS in `dist/native-desktop/`. The workflow additionally runs native-window and package lifecycle checks. The CI-only installer scripts deliberately refuse to install on a normal workstation.

The Linux workflow builds inside Debian12 and tests actual packaged binaries on clean Debian, Ubuntu and Fedora installations in isolated containers. It never mounts USB devices, host displays, home directories or secrets.
