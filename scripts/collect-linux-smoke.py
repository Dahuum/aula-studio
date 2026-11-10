#!/usr/bin/env python3
"""Stage only native test summaries/logs, without traversing private profile data."""
import json
from pathlib import Path
import shutil

root = Path(__file__).resolve().parents[1]
destination = root / 'dist/linux-smoke'
for suite in ['native-demo-smoke', 'native-model-smoke']:
    runs = [p for p in (root / 'analysis' / suite).iterdir() if p.name.isdecimal() and p.is_dir() and not p.is_symlink()]
    assert len(runs) == 1, 'Use a fresh CI checkout with exactly one run per suite'
    summary = runs[0] / 'result.json'
    result = json.loads(summary.read_text(encoding='utf-8'))
    assert result['status'] == 'passed' and result['physical_writes'] is False
    out = destination / suite
    out.mkdir(parents=True, exist_ok=False)
    for name in ['result.json', 'driver.log']:
        source = runs[0] / name
        assert source.is_file() and not source.is_symlink()
        shutil.copyfile(source, out / name)
    print(suite + ': staged summary and driver log; private profiles excluded')
