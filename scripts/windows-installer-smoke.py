#!/usr/bin/env python3
"""Test only our freshly built NSIS package on an ephemeral Windows CI runner."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys


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
    assert digest(installed[0]) == digest(original), 'Installed binary differs from tested build'
    try:
        subprocess.run([sys.executable, str(root / 'scripts/native-platform-smoke.py'),
                        '--binary', str(installed[0]), '--cli', str(release / 'aula.exe'),
                        '--output', str(out / 'launch')], check=True, timeout=100)
    finally:
        uninstallers = list(install.glob('*ninstall*.exe'))
        assert len(uninstallers) == 1, 'No unique owned uninstaller'
        subprocess.run([str(uninstallers[0]), '/S', '_?=' + str(install)], check=True, timeout=120)
    assert not installed[0].exists(), 'Uninstall left the application binary'
    result = {'status': 'passed', 'installer_sha256': digest(candidates[0]),
              'installed_binary_sha256': digest(original), 'physical_validation': False,
              'checks': ['current-user install', 'installed binary identity',
                         'installed native GUI launch with isolated demo data', 'uninstall']}
    (out / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
