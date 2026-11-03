#!/usr/bin/env python3
"""Check local publication artifacts and landing-page links. No network writes."""
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import tarfile
from urllib.parse import unquote, urlparse

ROOT = Path(__file__).resolve().parents[1]


class Page(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []
        self.ids = set()
        self.errors = []
    def handle_starttag(self, tag, attrs):
        values = dict(attrs)
        if values.get('id'):
            if values['id'] in self.ids:
                self.errors.append('Duplicate ID: '+values['id'])
            self.ids.add(values['id'])
        for key in ['href', 'src', 'data-zoom']:
            if values.get(key):
                self.links.append(values[key])
        if tag == 'img' and 'alt' not in values:
            self.errors.append('Image missing alt text')
        if any(key.lower().startswith('on') for key in values):
            self.errors.append('Inline event handler')


def main():
    site = ROOT / 'site/dist'
    page = Page()
    text = (site / 'index.html').read_text()
    page.feed(text)
    assert not page.errors, page.errors
    assert '@@' not in text, 'Unresolved template values'
    for link in page.links:
        url = urlparse(link)
        if url.scheme:
            assert url.scheme == 'https', link
            continue
        if not url.path and url.fragment:
            assert url.fragment in page.ids, link
        elif url.path:
            target = (site / unquote(url.path)).resolve()
            assert target.is_relative_to(site) and target.is_file(), link
    source = json.loads((ROOT / 'dist/publication/manifest.json').read_text())
    root = Path(source['source_archive']).parent / f"aula-studio-{source['version']}"
    broken = []
    for p in root.rglob('*.md'):
        for target in re.findall(r'\[[^\]]*\]\(([^)]+)\)', p.read_text()):
            url = urlparse(target.strip('<>'))
            if url.scheme or target.startswith('#'):
                continue
            if not (p.parent / unquote(url.path)).exists():
                broken.append(f'{p.relative_to(root)}: {target}')
    assert not broken, broken
    for line in (site / 'downloads/SHA256SUMS').read_text().splitlines():
        digest, filename = line.split('  ', 1)
        f = site / 'downloads' / filename
        assert hashlib.sha256(f.read_bytes()).hexdigest() == digest, filename
    for name in [source['source_archive'], source.get('binary_archive')]:
        if not name:
            continue
        with tarfile.open(name) as tar:
            for member in tar.getmembers():
                parts = Path(member.name).parts
                assert not member.issym() and not member.islnk()
                assert not Path(member.name).is_absolute() and '..' not in parts
                assert not any(p in {'analysis', 'original', '.git', 'node_modules', 'target'} for p in parts)
                assert not member.name.lower().endswith(('.exe', '.dll', '.pdb'))
                assert Path(member.name).name not in {'CONTINUATION.md', 'PROGRESS.md', 'FINAL_REPORT.md'}
                if member.name.endswith('.md'):
                    assert b'/home/void_id' not in tar.extractfile(member).read(), member.name
    result = {'status': 'passed', 'html_links': len(page.links), 'source_files': len(source['files']),
              'archives': 'paths, contents and checksums checked', 'remote_actions': False}
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
