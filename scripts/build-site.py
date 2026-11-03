#!/usr/bin/env python3
"""Build the portable static landing page locally. Does not publish anything."""
import argparse
import html
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import zipfile
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('publication_export', ROOT / 'scripts/prepare-public-release.py')
export = importlib.util.module_from_spec(spec)
spec.loader.exec_module(export)


def build(include_binaries=False, repository_url=None, site_url=None):
    project = json.loads((ROOT / 'site/project.json').read_text())
    for name, value in [('repository_url', repository_url), ('site_url', site_url)]:
        if value:
            parsed = urlparse(value)
            if parsed.scheme != 'https' or not parsed.netloc or parsed.username or parsed.password or parsed.query or parsed.fragment:
                raise ValueError(f'{name} must be a normal HTTPS URL')
            if name == 'repository_url' and (parsed.netloc != 'github.com' or len(parsed.path.strip('/').split('/')) != 2):
                raise ValueError('Repository URL must identify a real GitHub owner/repository')
            project[name] = value.rstrip('/')
    report = export.prepare(include_binaries)
    version = report['version']
    out = ROOT / 'site/dist'
    if out.exists():
        shutil.rmtree(out)
    (out / 'assets').mkdir(parents=True)
    (out / 'downloads').mkdir()
    (out / 'docs').mkdir()
    for name in ['style.css', 'app.js']:
        shutil.copyfile(ROOT / 'site' / name, out / name)
    shutil.copyfile(ROOT / 'docs/assets/mark.svg', out / 'assets/mark.svg')
    for name in ['lighting-demo.png', 'macros-demo.png']:
        shutil.copyfile(ROOT / 'docs/screenshots' / name, out / 'assets' / name)
    for p in Path(report['source_archive']).parent.joinpath(f'aula-studio-{version}/docs').glob('*.md'):
        shutil.copyfile(p, out / 'docs' / p.name)
    shutil.copyfile(ROOT / 'LICENSE', out / 'LICENSE.txt')
    for f in [Path(report['source_archive']), *([Path(report['binary_archive'])] if report['binary_archive'] else []), Path(report['source_archive']).parent / 'SHA256SUMS']:
        shutil.copyfile(f, out / 'downloads' / f.name)
    download = Path(report['binary_archive'] or report['source_archive']).name
    values = {'VERSION': version, 'DOWNLOAD_URL': 'downloads/'+download,
              'DOWNLOAD_LABEL': 'Download for Linux' if include_binaries else 'Download source',
              'DOWNLOAD_NOTE': 'x86_64 GNU/Linux · glibc 2.36+ · GTK / WebKitGTK libraries required.' if include_binaries else 'MIT source archive · build instructions included.'}
    if project.get('repository_url') and not include_binaries:
        values.update(DOWNLOAD_URL=project['repository_url']+'/releases', DOWNLOAD_LABEL='View releases',
                      DOWNLOAD_NOTE='Check release availability and runtime requirements. Source download included.')
    page = (ROOT / 'site/index.html').read_text()
    for name, value in values.items():
        page = page.replace('@@'+name+'@@', html.escape(value, quote=True))
    if '@@' in page:
        raise ValueError('Unresolved landing-page template value')
    if project.get('site_url'):
        url = html.escape(project['site_url'], quote=True)
        page = page.replace('</head>', f'<link rel="canonical" href="{url}/">\n<meta property="og:image" content="{url}/assets/lighting-demo.png">\n</head>')
    (out / 'index.html').write_text(page)
    project.update(version=version, binary_download_included=bool(report['binary_archive']))
    (out / 'project.json').write_text(json.dumps(project, indent=2)+'\n')
    (out / '.nojekyll').write_text('')
    site_archive = ROOT / 'dist/publication' / f'aula-studio-{version}-site.zip'
    with zipfile.ZipFile(site_archive, 'w', compression=zipfile.ZIP_DEFLATED) as zipped:
        for path in sorted(out.rglob('*')):
            if path.is_file():
                zipped.write(path, path.relative_to(out))
    sums = site_archive.parent / 'SHA256SUMS'
    with sums.open('a') as stream:
        stream.write(hashlib.sha256(site_archive.read_bytes()).hexdigest()+'  '+site_archive.name+'\n')
    print(json.dumps({'directory': str(out), 'version': version, 'binary_download': include_binaries,
                      'repository_url': project.get('repository_url'), 'site_archive': str(site_archive),
                      'publicly_deployed': False}, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--include-binaries', action='store_true')
    parser.add_argument('--repository-url')
    parser.add_argument('--site-url')
    args = parser.parse_args()
    build(args.include_binaries, args.repository_url, args.site_url)
