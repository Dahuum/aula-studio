#!/usr/bin/env python3
"""Stage only native test summaries/logs, without traversing private profile data."""
import argparse
import json
from pathlib import Path
import shutil

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--include-failures', action='store_true', help='Retain explicit failure evidence; never report it as a passing suite')
options = parser.parse_args()
destination = root / 'dist/linux-smoke'
for suite in ['native-demo-smoke', 'native-model-smoke']:
    if not (root / 'analysis' / suite).is_dir() and options.include_failures:
        continue
    runs = [p for p in (root / 'analysis' / suite).iterdir() if p.name.isdecimal() and p.is_dir() and not p.is_symlink()]
    assert len(runs) == 1, 'Use a fresh CI checkout with exactly one run per suite'
    summary = runs[0] / 'result.json'
    if summary.is_file():
        result = json.loads(summary.read_text(encoding='utf-8'))
        assert result['status'] == 'passed' and result['physical_writes'] is False
    else:
        assert options.include_failures, 'Suite did not pass'
    out = destination / suite
    out.mkdir(parents=True, exist_ok=False)
    for name in ['result.json', 'driver.log', 'failure-state.json', 'failure-buttons.json']:
        source = runs[0] / name
        if not source.exists():
            continue
        assert source.is_file() and not source.is_symlink()
        shutil.copyfile(source, out / name)
    print(suite + ': staged summary and driver log; private profiles excluded')
