# Install and run

AULA Studio targets the original ANSI F75 in USB wired mode on Linux. Arch with Hyprland/Wayland is the tested desktop. Other distributions need compatible GTK 3, WebKitGTK 4.1, libudev and appindicator libraries.

## Build

Use current stable Rust, Node/npm, Python 3.11+ and native build tools. Arch dependencies include `base-devel`, `pkgconf`, `gtk3`, `webkit2gtk-4.1`, `systemd-libs` and `libayatana-appindicator`.

```sh
npm --prefix app ci
npm --prefix app run build
cargo build --locked --release -p aula-studio -p aula-cli
./target/release/aula-studio
```

The release bundles its frontend. Wine, a browser configurator, Ghidra, REA and Windows software are not runtime dependencies. These commands do not install system integration.

## Portable build

Extract the archive for your architecture and run `bin/aula-studio`. Runtime GTK/WebKitGTK libraries must still be installed. `bin/aula` is the read-only CLI. Launch as your normal user.

## Arch package

From a complete source checkout:

```sh
./scripts/build-release.sh
cp dist/aula-studio-0.4.2-linux-x86_64.tar.gz packaging/
cd packaging
makepkg -f
```

Installation is a separate explicit action:

```sh
sudo pacman -U aula-studio-0.4.2-1-x86_64.pkg.tar.zst
```

The package includes the desktop entry, icon and a wired-device-specific `uaccess` rule. No privileged daemon is supplied.

## USB permissions

First enumerate with `aula devices`. If the supported wired interface is present but access is denied, the narrow rule is:

```sh
sudo install -m644 packaging/70-aula-f75.rules /etc/udev/rules.d/70-aula-f75.rules
sudo udevadm control --reload-rules
```

Reconnect afterward. Access is granted to the active local seat user, not made world-writable. Do not launch as root or replace HID drivers. Remote/TTY sessions may not receive the same seat permissions as the active desktop.

## Display startup

Version 0.4.2 fixes the GTK panic from a TTY without display variables. With no `DISPLAY` or `WAYLAND_DISPLAY`, the app can discover one user-owned `wayland-*` socket in a private runtime directory. It changes only its own environment, never shell or compositor settings.

Explicit settings are respected. If multiple sessions exist, select the intended one. For example, **only when this is your graphical session**:

```sh
GDK_BACKEND=wayland WAYLAND_DISPLAY=wayland-1 \
  XDG_RUNTIME_DIR=/run/user/$(id -u) ./target/release/aula-studio
```

The window opens on the desktop, not inside the TTY. Missing or ambiguous displays give a concise error. Do not weaken display-server permissions or change graphics drivers to fix a missing environment variable.

## Read-only diagnostics

```sh
./target/release/aula devices
./target/release/aula diagnostics
./target/release/aula read /dev/hidrawN
./target/release/aula read-layer /dev/hidrawN base
./target/release/aula read-layer /dev/hidrawN fn
./target/release/aula read-per-key /dev/hidrawN
./target/release/aula validate path/to/profile.json
```

Replace `hidrawN` with the freshly enumerated supported interface. Reads use known bounded operations. There is no CLI configuration-write command. Receiver detection does not establish wireless support.

## Data

| Data | Location |
| --- | --- |
| Profiles, history, backups and journals | `$XDG_DATA_HOME/aula-studio` or `~/.local/share/aula-studio` |
| Preferences | `$XDG_CONFIG_HOME/aula-studio/preferences.json` or `~/.config/aula-studio/preferences.json` |

JSON files use private permissions. Legacy preferences migrate on explicit save without deletion. `AULA_DATA_DIR` redirects both roots for isolated tests. `AULA_DISABLE_HARDWARE_WRITES=1` blocks physical commits while retaining read/prepare.

Optional local artwork: set `AULA_ORIGINAL_SKIN_DIR=/absolute/path/to/extracted/app` to a directory containing `skins/`. Nothing is uploaded or bundled. `AULA_NO_ORIGINAL_SKIN=1` selects independent graphics.

## Uninstall

Use `sudo pacman -R aula-studio` for the Arch package, or delete the extracted portable directory. Profiles and backups are separate and retained unless intentionally removed.

If you manually installed the rule, remove that specific file, reload rules and reconnect:

```sh
sudo rm /etc/udev/rules.d/70-aula-f75.rules
sudo udevadm control --reload-rules
```
