#!/usr/bin/env python3
"""Verify and open our own DMG from a local copy on an ephemeral macOS runner."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target', required=True)
    args = parser.parse_args()
    assert platform.system() == 'Darwin' and os.environ.get('GITHUB_ACTIONS') == 'true'
    assert os.environ.get('AULA_DISABLE_HARDWARE_WRITES') == '1'
    assert args.target in {'aarch64-apple-darwin', 'x86_64-apple-darwin'}
    root = Path(__file__).resolve().parents[1]
    release = root / 'target' / args.target / 'release'
    candidates = list((release / 'bundle/dmg').glob('*.dmg'))
    assert len(candidates) == 1
    out = root / 'dist/macos-installer-smoke'
    out.mkdir(parents=True, exist_ok=False)
    mount = out / 'mount'
    mount.mkdir()
    subprocess.run(['hdiutil', 'verify', str(candidates[0])], check=True, timeout=120)
    subprocess.run(['hdiutil', 'attach', '-readonly', '-nobrowse', '-mountpoint', str(mount),
                    str(candidates[0])], check=True, timeout=120)
    app = out / 'installed/AULA Studio.app'
    app.parent.mkdir()
    try:
        assert (mount / 'AULA Studio.app').is_dir()
        subprocess.run(['ditto', str(mount / 'AULA Studio.app'), str(app)], check=True, timeout=60)
    finally:
        subprocess.run(['hdiutil', 'detach', str(mount)], check=True, timeout=60)
    binary = app / 'Contents/MacOS/aula-studio'
    original = release / 'bundle/macos/AULA Studio.app/Contents/MacOS/aula-studio'
    digest = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
    assert digest(binary) == digest(original), 'DMG application differs from the tested build'
    subprocess.run([sys.executable, str(root / 'scripts/native-platform-smoke.py'),
                    '--binary', str(binary), '--cli', str(release / 'aula'),
                    '--output', str(out / 'launch')], check=True, timeout=100)
    result = {'status': 'passed', 'target': args.target, 'dmg_sha256': digest(candidates[0]),
              'binary_sha256': digest(binary), 'physical_validation': False,
              'checks': ['DMG integrity', 'read-only mount', 'copied app binary identity',
                         'copied native GUI launch with isolated demo data', 'image unmounted'],
              'limits': ['CI does not reproduce downloaded-file quarantine or establish notarization']}
    (out / 'result.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
