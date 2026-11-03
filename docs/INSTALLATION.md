# Install AULA Studio

Binary packages target **x86_64 GNU/Linux**, glibc **2.36 or newer**, GTK 3 and WebKitGTK **4.1**. They are built inside Debian 12, rather than wrapping a rolling Arch build for other distributions. The application requires a graphical Linux desktop and runs as your regular user. Wayland and X11 are supported.

Original ANSI AULA F75 configuration is wired USB only. Installing on more distributions does not expand keyboard or wireless support. See [hardware support](FEATURE_MATRIX.md).

## Downloads and verification

Get files from [the project's GitHub Releases](https://github.com/Dahuum/aula-studio/releases), not a similarly named npm/system package. Download the package for your distribution and `SHA256SUMS` into the same directory. Version 0.4.3 files are:

| File | Purpose |
| --- | --- |
| `aula-studio-0.4.3-1-x86_64.pkg.tar.zst` | Arch package |
| `aula-studio_0.4.3_amd64.deb` | Debian-family package |
| `aula-studio-0.4.3-1.x86_64.rpm` | Fedora-family package |
| `aula-studio-0.4.3-linux-x86_64.tar.gz` | Portable directory using system runtime libraries |
| `aula-studio-0.4.3-source.tar.gz` | Source and build instructions |

```sh
sha256sum --check --ignore-missing SHA256SUMS
```

Confirm every downloaded file you intend to use reports **OK**, with no mismatch. These checksums detect corruption; they are not code-signing signatures. No package installs itself or writes keyboard settings merely by being downloaded.

## Arch Linux

After downloading and verifying the Arch package:

```sh
sudo pacman -U ./aula-studio-0.4.3-1-x86_64.pkg.tar.zst
aula-studio
```

Pacman resolves runtime dependencies from your configured repositories. The package contains the desktop launcher, read-only `aula` CLI and narrow uaccess rule. Reconnect after installation; if permissions have not refreshed, run `sudo udevadm control --reload-rules` before reconnecting. Do not run the application as root.

There is **no published AUR entry**. `packaging/aur/PKGBUILD` and `.SRCINFO` are a local recipe for `aula-studio-bin`, downloading the exact release tarball with a pinned checksum. Review before running `makepkg -si` inside that directory. Do not present `yay -S aula-studio` as an existing installation method.

Remove the native Arch package with `sudo pacman -R aula-studio` (or `aula-studio-bin` if you built that recipe).

## Debian, Ubuntu and Linux Mint

Debian **12+**, Ubuntu **24.04+**, and derivatives based on them can provide the required glibc/WebKitGTK runtime. Older distributions are not covered by this binary. Install the verified local package using apt so it resolves dependencies:

```sh
sudo apt install ./aula-studio_0.4.3_amd64.deb
aula-studio
```

The package depends on GTK 3, WebKitGTK 4.1, libudev, libgcc/libstdc++ and Ayatana appindicator. Do not use `dpkg --force-*` or install foreign library packages to bypass missing prerequisites. Compatible derivatives are not automatically desktop/hardware-validated merely because dependency resolution succeeds.

Remove with `sudo apt remove aula-studio`. There is no hosted apt repository or automatic updater; install a newer reviewed release with the same local apt command.

## Fedora

Use a Fedora release with WebKitGTK 4.1 and glibc 2.36+. Install the verified local RPM:

```sh
sudo dnf install ./aula-studio-0.4.3-1.x86_64.rpm
aula-studio
```

Dependencies include `gtk3`, `webkit2gtk4.1`, `systemd-libs`, `libgcc`, `libstdc++` and `libayatana-appindicator-gtk3`. Remove with `sudo dnf remove aula-studio`. No hosted dnf repository is supplied. Other RPM distributions use different dependency names and are not claimed supported by this Fedora recipe.

## Other Linux distributions

The tarball uses system libraries, not a bundled WebKit. Extract it and run:

```sh
tar -xf aula-studio-0.4.3-linux-x86_64.tar.gz
./aula-studio-0.4.3-linux-x86_64/bin/aula-studio
```

Install the GTK 3/WebKitGTK 4.1/libudev/appindicator runtime appropriate to your distribution. Check dependencies with `ldd .../bin/aula-studio`; every required library must resolve. The CLI is `.../bin/aula`. There is no installation into system directories or automatic desktop/udev integration with this format. Optional manual integration is described below.

No AppImage, Flatpak, Snap, npm package, aarch64 binary or musl build is published for this release. A single archive does not promise compatibility with every Linux system. Flatpak HID access requires a separately designed permission model; no broad device sandbox bypass is supplied as a shortcut.

## Build from source

Use Rust 1.93.1 (tested release compiler), Node 24/npm, Python 3.11+ and native build tools. Newer Rust can introduce additional Clippy lints; the CI compiler is pinned.

Arch prerequisites:

```sh
sudo pacman -S --needed base-devel pkgconf gtk3 webkit2gtk-4.1 systemd-libs libayatana-appindicator
```

Debian/Ubuntu prerequisites:

```sh
sudo apt install build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev libudev-dev libayatana-appindicator3-dev
```

Fedora prerequisites:

```sh
sudo dnf install gcc gcc-c++ make pkgconf-pkg-config gtk3-devel webkit2gtk4.1-devel systemd-devel libayatana-appindicator-gtk3-devel
```

Install Rust/Node using their official instructions or your package manager, then:

```sh
git clone https://github.com/Dahuum/aula-studio.git
cd aula-studio
npm --prefix app ci
npm --prefix app run build
cargo build --locked --release -p aula-studio -p aula-cli
./target/release/aula-studio
```

`npm install -g aula-studio` is **not** an installation method. The private frontend package is not published to npm; npm is a build dependency only. Source builds do not automatically install desktop integration or udev rules. REA, Ghidra and Windows software are not application runtime dependencies.

## Reproduce distribution packages

`packaging/linux/Dockerfile` builds a Debian 12 toolchain. Use a **clean public source checkout** as the container's `/work`, not a directory containing the proprietary installer or private analysis. No USB devices, display sockets, home directories or secrets should be mounted. Build as your regular UID; place Cargo/npm caches inside the project. Container network setup must not change the host's Docker/network configuration.

Inside that toolchain:

```sh
npm --prefix app ci
npm --prefix app run build
cargo build --locked --release -p aula-studio -p aula-cli
python3 scripts/package-linux.py
```

Packages and notices are in `dist/linux/`. The script stages files and produces `.deb`, `.rpm` and a tarball; it never installs them or talks to a keyboard. Review package metadata, ELF dependencies, licenses and launch tests before publishing. Arch packaging uses the same portable payload through `packaging/PKGBUILD`.

## USB permissions and desktop integration

Native packages include the rule under `/usr/lib/udev/rules.d`. After installation, refresh rules if needed and reconnect. Portable/source users can optionally install this specific rule:

```sh
sudo install -m644 packaging/70-aula-f75.rules /etc/udev/rules.d/70-aula-f75.rules
sudo udevadm control --reload-rules
```

This grants the active local seat user access to wired `258a:010c`. It is not world-writable. No drivers, kernel settings or privileged service are replaced. Remote/TTY sessions can have different seat permissions. Use the desktop user session.

For portable desktop integration, install the supplied `share/applications/aula-studio.desktop` and icon into your XDG user directories and adjust `Exec` to the actual absolute binary path. Avoid copying a launcher whose `Exec=aula-studio` is not on PATH. Native packages handle this for you.

## Startup and diagnostics

Launch from the desktop menu or a terminal in your graphical session. Version 0.4.2+ can discover one trusted user-owned Wayland socket when both display variables are absent; ambiguous/missing displays return a readable error. It changes only the app's environment, not your shell or compositor.

```sh
aula devices
aula diagnostics
aula read /dev/hidrawN
aula read-layer /dev/hidrawN base
aula read-layer /dev/hidrawN fn
aula read-per-key /dev/hidrawN
aula validate path/to/profile.json
```

Enumerate first; `hidrawN` changes on reconnect. Reads are known bounded operations. The CLI cannot write configuration. Receiver detection does not establish wireless support.

## User data and uninstall

| Data | Location |
| --- | --- |
| Profiles, history, backups, journals | `$XDG_DATA_HOME/aula-studio` or `~/.local/share/aula-studio` |
| Preferences | `$XDG_CONFIG_HOME/aula-studio/preferences.json` or `~/.config/aula-studio/preferences.json` |

Package removal keeps these files. Portable uninstall means deleting the extracted directory and any manually installed launcher/icon. Remove only an optional rule you personally installed under `/etc/udev/rules.d/70-aula-f75.rules`, reload rules and reconnect. Do not delete unrelated rules.

`AULA_DATA_DIR` redirects app storage for tests. `AULA_DISABLE_HARDWARE_WRITES=1` blocks physical commits while retaining read/prepare. Optional original artwork can be read locally through `AULA_ORIGINAL_SKIN_DIR`; it is never redistributed or uploaded.

## Evidence and current limits

Distribution install/launch evidence is recorded in [TESTING.md](TESTING.md), separately from F75 hardware validation. A container GUI launch verifies packaging/runtime behavior, not your compositor or physical keyboard. Compatible but untested derivatives remain labeled as such.

References: [Tauri Debian packaging](https://v2.tauri.app/distribute/debian/), [Linux baseline requirements](https://v2.tauri.app/distribute/appimage/), [Fedora WebKitGTK package](https://packages.fedoraproject.org/pkgs/webkitgtk/webkit2gtk4.1/), [Fedora appindicator package](https://packages.fedoraproject.org/pkgs/libayatana-appindicator/libayatana-appindicator-gtk3/).
