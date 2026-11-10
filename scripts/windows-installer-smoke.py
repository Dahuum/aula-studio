#!/usr/bin/env python3
"""Test only our freshly built NSIS package on an ephemeral Windows CI runner."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys


def verify_binary_identity(installed, original):
    """Allow only Tauri's documented UNK -> NSS bundler tag, at the same offset.

    https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle.rs
    Every other byte, including code, resources and length, must match.
    """
    if installed == original:
        return 'exact bytes'
    before = b'__TAURI_BUNDLE_TYPE_VAR_UNK'
    after = b'__TAURI_BUNDLE_TYPE_VAR_NSS'
    assert original.count(before) == installed.count(after) == 1, 'Non-unique Tauri bundle marker'
    assert original.index(before) == installed.index(after), 'Moved Tauri bundle marker'
    assert installed.replace(after, before, 1) == original, 'Installed binary has unrelated changes'
    return 'exact bytes except the three-byte Tauri UNK-to-NSS bundle tag'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target', default='x86_64-pc-windows-msvc')
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    # Never install on an ordinary workstation or accept an arbitrary executable.
    assert os.name == 'nt' and os.environ.get('GITHUB_ACTIONS') == 'true'
    assert os.environ.get('AULA_DISABLE_HARDWARE_WRITES') == '1'
    release = root / 'target' / args.target / 'release'
    candidates = list((release / 'bundle/nsis').glob('*.exe'))
    assert len(candidates) == 1
    original = release / 'aula-studio.exe'
    out = root / 'dist/windows-installer-smoke'
    out.mkdir(parents=True, exist_ok=False)
    install = out / 'installed'
    # NSIS /D must be the final argument. All paths are this CI checkout's.
    subprocess.run([str(candidates[0]), '/S', '/D=' + str(install)], check=True, timeout=180)
    installed = list(install.rglob('aula-studio.exe'))
    assert len(installed) == 1, 'Installer did not provide our application'
    digest = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    try:
        identity = verify_binary_identity(installed[0].read_bytes(), original.read_bytes())
        installed_sha = digest(installed[0])
        subprocess.run([sys.executable, str(root / 'scripts/native-platform-smoke.py'),
                        '--binary', str(installed[0]), '--cli', str(release / 'aula.exe'),
                        '--output', str(out / 'launch')], check=True, timeout=100)
    finally:
        uninstallers = list(install.glob('*ninstall*.exe'))
        assert len(uninstallers) == 1, 'No unique owned uninstaller'
        subprocess.run([str(uninstallers[0]), '/S', '_?=' + str(install)], check=True, timeout=120)
    assert not installed[0].exists(), 'Uninstall left the application binary'
    result = {'status': 'passed', 'installer_sha256': digest(candidates[0]),
              'installed_binary_sha256': installed_sha, 'build_binary_sha256': digest(original),
              'binary_identity': identity, 'physical_validation': False,
              'checks': ['current-user install', 'installed binary identity',
                         'installed native GUI launch with isolated demo data', 'uninstall']}
    (out / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
