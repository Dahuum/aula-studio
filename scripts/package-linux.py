#!/usr/bin/env python3
"""Package existing Linux release binaries on Debian 12. No install or device IO."""
import argparse
import hashlib
import importlib.util
import json
import re
from pathlib import Path
import shutil
import subprocess
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('exporter', ROOT / 'scripts/prepare-public-release.py')
exporter = importlib.util.module_from_spec(spec)
spec.loader.exec_module(exporter)


def verify_native_baseline(binary):
    header=binary.read_bytes()[:20]
    if len(header)!=20 or header[:6]!=b'\x7fELF\x02\x01' or int.from_bytes(header[18:20], 'little')!=62:
        raise ValueError('Expected an x86_64 Linux ELF executable')
    versions=subprocess.check_output(['readelf','--version-info',str(binary)],text=True)
    requirements=[tuple(map(int,v.split('.'))) for v in re.findall(r'GLIBC_([0-9]+\.[0-9]+)',versions)]
    if not requirements or max(requirements)>(2,36):
        raise ValueError('Debian12 package requires a glibc2.36 baseline build, not an arbitrary host binary')


def build(output=None, portable_archive=None, binary_directory=None):
    version = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']['package']['version']
    out = Path(output or ROOT / 'dist/linux').resolve()
    if not out.is_relative_to(ROOT / 'dist'):
        raise ValueError('Package output must be in project dist/')
    if subprocess.check_output(['uname', '-m'], text=True).strip() != 'x86_64':
        raise ValueError('This package recipe currently targets x86_64 only')
    out.mkdir(parents=True, exist_ok=True)
    portable = out / f'aula-studio-{version}-linux-x86_64'
    if portable_archive:
        supplied = Path(portable_archive).resolve()
        with tarfile.open(supplied) as tar:
            total = 0
            for member in tar.getmembers():
                path = Path(member.name)
                if not member.isfile() or path.is_absolute() or '..' in path.parts or path.parts[0] != portable.name:
                    raise ValueError('Unsafe portable package member')
                total += member.size
                if total > 128 * 1024 * 1024:
                    raise ValueError('Unexpected portable package size')
            if portable.exists():
                shutil.rmtree(portable)
            # Python 3.11 on the baseline: paths/types were bounded above.
            tar.extractall(out)
        archive = out / supplied.name
        if archive != supplied:
            shutil.copyfile(supplied, archive)
        hashes = {}
        for name in ['aula-studio', 'aula']:
            binary = portable / 'bin' / name
            assert binary.read_bytes().startswith(b'\x7fELF')
            hashes[name] = hashlib.sha256(binary.read_bytes()).hexdigest()
        verify_native_baseline(portable / 'bin/aula-studio')
        verify_native_baseline(portable / 'bin/aula')
        notices = (portable / 'THIRD_PARTY_NOTICES.txt').read_text()
        count = notices.count('Source: https://crates.io/crates/')
    else:
        if portable.exists():
            shutil.rmtree(portable)
        (portable / 'bin').mkdir(parents=True)
        hashes = {}
        for name in ['aula-studio', 'aula']:
            src = Path(binary_directory or ROOT / 'target/release').resolve() / name
            if not src.is_relative_to(ROOT):
                raise ValueError('Build binaries must stay inside the project')
            verify_native_baseline(src)
            assert src.read_bytes().startswith(b'\x7fELF'), name
            shutil.copyfile(src, portable / 'bin' / name)
            (portable / 'bin' / name).chmod(0o755)
            hashes[name] = hashlib.sha256(src.read_bytes()).hexdigest()
        (portable / 'docs').mkdir()
        for name in exporter.PUBLIC_DOCS:
            p = ROOT / 'docs' / name
            if p.exists():
                (portable / 'docs' / name).write_bytes(exporter.sanitized('docs/'+name, p.read_bytes()))
        shutil.copytree(ROOT / 'docs/screenshots', portable / 'docs/screenshots')
        for name in ['README.md', 'LICENSE', 'CHANGELOG.md', 'SECURITY.md']:
            (portable / name).write_bytes(exporter.sanitized(name, (ROOT / name).read_bytes()))
        notices, count = exporter.native_notices()
        (portable / 'THIRD_PARTY_NOTICES.txt').write_text(notices)
        for src, rel in [('packaging/aula-studio.desktop', 'applications/aula-studio.desktop'),
                         ('packaging/70-aula-f75.rules', 'udev/70-aula-f75.rules'),
                         ('app/src-tauri/icons/icon.png', 'icons/hicolor/128x128/apps/aula-studio.png')]:
            dest = portable / 'share' / rel
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / src, dest)
        archive = out / f'{portable.name}.tar.gz'
        exporter.archive(portable, archive, portable.name)


    stage = out / 'native-root'
    if stage.exists():
        shutil.rmtree(stage)
    for src, rel in [('bin', 'usr/bin'), ('share/applications', 'usr/share/applications'),
                     ('share/icons', 'usr/share/icons'), ('share/udev', 'usr/lib/udev/rules.d'),
                     ('docs', 'usr/share/doc/aula-studio')]:
        shutil.copytree(portable / src, stage / rel)
    doc = stage / 'usr/share/doc/aula-studio'
    license_dir = stage / 'usr/share/licenses/aula-studio'
    license_dir.mkdir(parents=True)
    shutil.copyfile(portable / 'LICENSE', license_dir / 'LICENSE')
    shutil.copyfile(portable / 'THIRD_PARTY_NOTICES.txt', license_dir / 'THIRD_PARTY_NOTICES.txt')
    (doc / 'copyright').write_text((portable / 'LICENSE').read_text()+'\n'+notices)
    shutil.copyfile(portable / 'README.md', doc / 'README.md')
    control = stage / 'DEBIAN'
    control.mkdir()
    (control / 'control').write_text(f'''Package: aula-studio
Version: {version}
Section: utils
Priority: optional
Architecture: amd64
Maintainer: AULA Studio contributors <123483885+Dahuum@users.noreply.github.com>
Homepage: https://github.com/Dahuum/aula-studio
Installed-Size: {sum(p.stat().st_size for p in stage.rglob('*') if p.is_file()) // 1024 + 1}
Depends: libc6 (>= 2.36), libgcc-s1, libstdc++6, libgtk-3-0 (>= 3.24), libwebkit2gtk-4.1-0, libudev1, libayatana-appindicator3-1
Description: Native Linux configuration for the original wired ANSI AULA F75
 Independent early-access configurator. Reviewed hardware writes, local profiles,
 local macros and read-only diagnostics. Wireless configuration and onboard macro
 uploads are unavailable. Requires a graphical desktop; launch as a regular user.
''')
    deb = out / f'aula-studio_{version}_amd64.deb'
    subprocess.run(['dpkg-deb', '--root-owner-group', '--build', str(stage), str(deb)], check=True)
    shutil.rmtree(control)
    rpmroot = out / 'rpm'
    (rpmroot / 'SPECS').mkdir(parents=True, exist_ok=True)
    rpm_spec = rpmroot / 'SPECS/aula-studio.spec'
    rpm_spec.write_text(f'''Name: aula-studio
Version: {version}
Release: 1
Summary: Native Linux configuration for the original wired ANSI AULA F75
License: MIT
URL: https://github.com/Dahuum/aula-studio
BuildArch: x86_64
AutoReqProv: no
Requires: glibc >= 2.36, libgcc, libstdc++, gtk3, webkit2gtk4.1, systemd-libs, libayatana-appindicator-gtk3
%description
Independent early-access configurator with reviewed writes, profiles and local macros.
Original ANSI F75, USB wired only. Onboard macro uploads and wireless are unavailable.
%install
mkdir -p %{{buildroot}}
cp -a {stage}/usr %{{buildroot}}/
%files
/usr/bin/aula
/usr/bin/aula-studio
/usr/share/applications/aula-studio.desktop
/usr/share/icons/hicolor/128x128/apps/aula-studio.png
/usr/lib/udev/rules.d/70-aula-f75.rules
%doc /usr/share/doc/aula-studio
%license /usr/share/licenses/aula-studio
''')
    subprocess.run(['rpmbuild', '-bb', str(rpm_spec), '--define', '_topdir '+str(rpmroot),
                    '--define', '_buildhost linux-packaging', '--define', '_build_id_links none',
                    '--define', '_dbpath '+str(rpmroot / 'db'), '--define', 'debug_package %{nil}', '--define', '__os_install_post %{nil}'], check=True)
    rpm = out / f'aula-studio-{version}-1.x86_64.rpm'
    shutil.copyfile(rpmroot / 'RPMS/x86_64' / rpm.name, rpm)
    files = [archive, deb, rpm]
    (out / 'SHA256SUMS').write_text(''.join(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+p.name+'\n' for p in files))
    report = {'version': version, 'baseline': 'Debian 12 / glibc 2.36', 'architecture': 'x86_64',
              'binary_hashes': hashes, 'rust_dependency_notices': count,
              'artifacts': [p.name for p in files], 'hardware_writes': 0}
    (out / 'build-report.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    args = argparse.ArgumentParser(description=__doc__)
    args.add_argument('--output')
    args.add_argument('--portable', help='Package an already-reviewed portable archive without rebuilding/replacing its binary bytes')
    args.add_argument('--binary-dir', help='Project-local directory containing reviewed Debian12 baseline binaries')
    options = args.parse_args()
    build(options.output, options.portable, options.binary_dir)
