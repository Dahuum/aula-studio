#!/usr/bin/env python3
"""Create a sanitized publication snapshot and archives. Never push or upload.

The existing working repository, its history and historical releases stay intact.
Only allowlisted source/docs and independently drawn screenshots are exported.
"""
import argparse
import gzip
import hashlib
import io
import json
from pathlib import Path
import re
import shutil
import subprocess
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
PUBLIC_DOCS = {'ARCHITECTURE.md', 'FEATURE_MATRIX.md', 'HARDWARE_SAFETY.md', 'HARDWARE_VALIDATION.md',
               'INSTALLATION.md', 'LIMITATIONS.md', 'MEDIA_BINDINGS.md', 'OFFICIAL_FEATURES.md',
               'MOUSE_BINDINGS.md', 'ORIGINAL_UI.md', 'PROTOCOL.md', 'REVERSE_ENGINEERING.md', 'ROADMAP.md', 'SOURCES.md',
               'TESTING.md', 'PUBLISHING.md', 'DEPENDENCIES.md', 'MODEL_SUPPORT.md', 'DOWNLOAD_CATALOG.md', 'MULTI_MODEL_RESEARCH.md', 'MODEL_WORKSPACES.md', 'F2088_RESEARCH.md', 'F98_RESEARCH.md', 'MACRO_FORMAT.md', 'F87_READ_CONTRACT.md', 'G5_READ_CONTRACTS.md', 'S75_READ_CONTRACT.md', 'F75_MATRIX_FRAMING.md'}
ROOT_FILES = {'README.md', 'LICENSE', 'CONTRIBUTING.md', 'SECURITY.md', 'CHANGELOG.md',
              'Cargo.toml', 'Cargo.lock', '.gitignore', '.dockerignore'}
SECRET = re.compile(rb'(?:gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|AKIA[A-Z0-9]{16}|-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----)')


def allowed(name):
    p = Path(name)
    if '..' in p.parts or p.is_absolute():
        return False
    if name in ROOT_FILES:
        return True
    if name.startswith('docs/'):
        return (len(p.parts) == 2 and p.name in PUBLIC_DOCS) or name in {
            'docs/assets/mark.svg', 'docs/screenshots/lighting-demo.png', 'docs/screenshots/macros-demo.png',
            'docs/validation/brightness-2026-11-02.json', 'docs/validation/startup-2026-11-02.json'}
    if name in {'data/models.json', 'data/download-catalog.json'}:
        return True
    if name.startswith('fixtures/'):
        return name in {'fixtures/keymap/f87-factory-base.bin', 'fixtures/keymap/f87-factory-fn.bin', 'fixtures/keymap/LICENSE'}
    if name.startswith('site/fonts/'):
        return p.name in {'instrument-sans.woff2', 'ibm-plex-mono.woff2', 'OFL-instrument-sans.txt', 'OFL-ibm-plex-mono.txt', 'sources.json'} and len(p.parts) == 3
    if name.startswith('third_party/licenses/'):
        return p.suffix in {'.txt', '.json'} and len(p.parts) == 3
    if name in {'packaging/linux/Dockerfile', 'packaging/linux/package-smoke.sh', 'packaging/aur/PKGBUILD', 'packaging/aur/.SRCINFO'}:
        return True
    if name.startswith('packaging/'):
        return p.name in {'PKGBUILD', '70-aula-f75.rules', 'aula-studio.desktop'} and len(p.parts) == 2
    if name.startswith(('analysis/', 'original/', 'dist/', 'target/', 'publication/')):
        return False
    if any(part in {'node_modules', 'dist', 'target', 'gen', '.git', '.env', '__pycache__'} for part in p.parts):
        return False
    if name.startswith(('app/', 'crates/', 'tests/', 'scripts/', 'site/', '.github/')):
        return p.suffix in {'.rs', '.toml', '.json', '.tsx', '.ts', '.css', '.html', '.js', '.mjs', '.svg', '.png', '.py', '.sh', '.md', '.yml', '.yaml'}
    return False


def stage_public_docs(destination):
    """Replace only owned generated docs with the same publication allowlist."""
    destination = Path(destination).resolve()
    if not destination.is_relative_to(ROOT / 'dist') or (destination / '.git').exists():
        raise ValueError('Native staging must be an owned project dist/ directory without Git metadata')
    docs = destination / 'docs'
    if docs.is_symlink() or (docs / '.git').exists():
        raise ValueError('Refusing symlinked or Git-managed staging documentation')
    if docs.exists():
        shutil.rmtree(docs)
    docs.mkdir(parents=True)
    for name in PUBLIC_DOCS:
        source = ROOT / 'docs' / name
        if source.is_symlink():
            raise ValueError('Refusing symlinked publication documentation')
        if source.is_file():
            (docs / name).write_bytes(sanitized('docs/' + name, source.read_bytes()))
    (docs / 'screenshots').mkdir()
    for name in ['lighting-demo.png', 'macros-demo.png']:
        source = ROOT / 'docs/screenshots' / name
        if source.is_symlink():
            raise ValueError('Refusing a symlinked publication screenshot')
        shutil.copyfile(source, docs / 'screenshots' / name)


def sanitized(name, data):
    if SECRET.search(data):
        raise ValueError(f'Possible credential in {name}; publication stopped')
    if name.endswith('.md'):
        text = data.decode('utf-8')
        text = re.sub(r'/home/[^/\s`"<>]+/Desktop/aula', '<project>', text)
        text = re.sub(r'/home/[^/\s`"<>]+', '~', text)
        return text.encode()
    return data


def archive(directory, destination, root_name):
    """Normalize timestamps/ownership and avoid symlinks or local Git metadata."""
    with destination.open('wb') as stream:
        with gzip.GzipFile(filename='', fileobj=stream, mode='wb', mtime=0) as zipped:
            with tarfile.open(fileobj=zipped, mode='w', format=tarfile.PAX_FORMAT) as tar:
                for path in sorted(directory.rglob('*')):
                    if not path.is_file() or path.is_symlink() or '.git' in path.relative_to(directory).parts:
                        continue
                    data = path.read_bytes()
                    info = tarfile.TarInfo(str(Path(root_name) / path.relative_to(directory)))
                    info.size = len(data)
                    info.mtime = 0
                    info.uid = info.gid = 0
                    info.uname = info.gname = ''
                    info.mode = 0o755 if path.stat().st_mode & 0o111 else 0o644
                    tar.addfile(info, io.BytesIO(data))


def native_notices():
    """Bundle cached upstream license/notice texts; never include cache paths."""
    raw = subprocess.check_output(['cargo', 'metadata', '--locked', '--offline', '--format-version=1',
                                   '--filter-platform', 'x86_64-unknown-linux-gnu'], cwd=ROOT, text=True)
    meta = json.loads(raw)
    packages = {p['id']: p for p in meta['packages']}
    nodes = {n['id']: n for n in meta['resolve']['nodes']}
    roots = [p['id'] for p in meta['packages'] if p['name'] in {'aula-studio', 'aula-cli'} and p.get('source') is None]
    reached, queue = set(), roots[:]
    while queue:
        current = queue.pop()
        if current in reached:
            continue
        reached.add(current)
        queue.extend(d['pkg'] for d in nodes.get(current, {}).get('deps', []))
    retained = json.loads((ROOT / 'third_party/licenses/index.json').read_text())
    chunks = ['AULA Studio third-party notices\n\nOriginal project code: MIT (see LICENSE).\n'
              'This inventory conservatively includes resolved Linux build dependencies. '
              'Upstream packages are unmodified; source is available at the versioned crates.io URLs below.\n'
              'System GTK/WebKitGTK libraries are not bundled and retain their own licenses.\n']
    missing = []
    count = 0
    for p in sorted((packages[i] for i in reached if packages[i].get('source')), key=lambda p: (p['name'], p['version'])):
        count += 1
        directory = Path(p['manifest_path']).parent
        chunks.append(f"\n{'='*72}\n{p['name']} {p['version']}\nLicense: {p.get('license') or 'see upstream files'}\nSource: https://crates.io/crates/{p['name']}/{p['version']}\n")
        files = {f for f in directory.iterdir() if f.is_file() and f.name.lower().startswith(('license', 'licence', 'copying', 'notice', 'copyright'))}
        if p.get('license_file'):
            candidate = directory / p['license_file']
            if candidate.is_file() and candidate.resolve().is_relative_to(directory.resolve()):
                files.add(candidate)
        if not files:
            for name in retained.get(p['name']+'@'+p['version'], {}).get('files', []):
                candidate = ROOT / 'third_party/licenses' / name
                if candidate.is_file() and candidate.resolve().is_relative_to(ROOT / 'third_party/licenses'):
                    files.add(candidate)
        if not files:
            missing.append(f"{p['name']} {p['version']}")
        for f in sorted(files):
            if f.stat().st_size > 200000:
                raise ValueError(f'Unexpectedly large license file for {p["name"]}')
            chunks.append('\n--- '+f.name+' ---\n'+f.read_text(errors='replace')+'\n')
    for name in ['react', 'react-dom', 'scheduler', '@tauri-apps/api']:
        directory = ROOT / 'app/node_modules' / name
        if not directory.exists():
            raise ValueError('Install frontend dependencies before exporting binaries')
        p = json.loads((directory / 'package.json').read_text())
        chunks.append(f"\n{'='*72}\n{name} {p['version']}\n")
        for f in sorted(directory.iterdir()):
            if f.is_file() and f.name.lower().startswith(('license', 'notice', 'copyright')):
                chunks.append('\n--- '+f.name+' ---\n'+f.read_text(errors='replace')+'\n')
    sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], text=True).strip())
    for filename in ['LICENSE-MIT', 'LICENSE-APACHE', 'COPYRIGHT']:
        f = sysroot / 'share/doc/rust' / filename
        if f.is_file():
            chunks.append(f'\nRust toolchain {filename}\n'+f.read_text(errors='replace')+'\n')
    if missing:
        raise ValueError('Missing cached dependency license texts: '+', '.join(missing))
    return '\n'.join(chunks), count


def prepare(include_binaries=False, output=None):
    version = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']['package']['version']
    out = Path(output or ROOT / 'dist/publication').resolve()
    if not out.is_relative_to(ROOT / 'dist'):
        raise ValueError('Publication output must stay inside project dist/')
    source = out / f'aula-studio-{version}'
    if (out / '.git').exists() or (source / '.git').exists():
        raise ValueError('Publication output contains a Git repository; refusing to replace it. Keep a separate publication checkout.')
    if source.exists():
        shutil.rmtree(source)
    source.mkdir(parents=True)
    raw = subprocess.check_output(['git', 'ls-files', '--cached', '--others', '--exclude-standard', '-z'], cwd=ROOT)
    included = []
    for name in sorted(set(n.decode() for n in raw.split(b'\0') if n)):
        if not allowed(name):
            continue
        original = ROOT / name
        if not original.is_file() or original.is_symlink():
            continue
        data = sanitized(name, original.read_bytes())
        destination = source / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(data)
        destination.chmod(0o755 if original.stat().st_mode & 0o111 else 0o644)
        included.append(name)
    for required in ['LICENSE', 'README.md', 'Cargo.lock', 'app/package-lock.json', 'app/src-tauri/src/main.rs']:
        if not (source / required).is_file():
            raise ValueError(f'Missing public source file: {required}')
    binary_archive = None
    dependency_count = None
    binary_hashes = {}
    if include_binaries:
        native = ROOT / f'dist/aula-studio-{version}-linux-x86_64'
        portable = out / f'aula-studio-{version}-linux-x86_64'
        if (portable / '.git').exists():
            raise ValueError('Portable output contains a Git repository; refusing to replace it')
        if portable.exists():
            shutil.rmtree(portable)
        (portable / 'bin').mkdir(parents=True)
        for name in ['aula', 'aula-studio']:
            data = (native / 'bin' / name).read_bytes()
            if not data.startswith(b'\x7fELF'):
                raise ValueError(f'Expected a trusted native Linux build for {name}')
            f = portable / 'bin' / name
            f.write_bytes(data); f.chmod(0o755)
            binary_hashes[name] = hashlib.sha256(data).hexdigest()
        for original, relative in [
            (source / 'packaging/aula-studio.desktop', 'share/applications/aula-studio.desktop'),
            (source / 'packaging/70-aula-f75.rules', 'share/udev/70-aula-f75.rules'),
            (source / 'app/src-tauri/icons/icon.png', 'share/icons/hicolor/128x128/apps/aula-studio.png'),
        ]:
            destination = portable / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(original, destination)
        shutil.copytree(source / 'docs', portable / 'docs')
        for name in ['README.md', 'LICENSE', 'CHANGELOG.md', 'SECURITY.md']:
            shutil.copyfile(source / name, portable / name)
        (portable / 'README.md').write_text(
            f'# AULA Studio {version} — Linux portable build\n\n'
            'x86_64 GNU/Linux early-access build; Debian 12 / glibc 2.36 baseline. Compatible GTK 3, WebKitGTK 4.1, libudev and appindicator libraries are required. See docs/TESTING.md for distribution validation.\n\n'
            'Run `./bin/aula-studio` as your normal desktop user. `./bin/aula devices` and `./bin/aula diagnostics` are read-only. '
            'Wine, a VM, a browser configurator, REA and the Windows software are not runtime requirements.\n\n'
            'Choose Enter demo mode to explore without a keyboard. Hardware changes require a backup, concrete review and separate Write. '
            'Saving a profile never configures hardware. Onboard macro uploads and wireless configuration are unavailable.\n\n'
            'See [installation and permissions](docs/INSTALLATION.md), [compatibility](docs/FEATURE_MATRIX.md), '
            '[safety](docs/HARDWARE_SAFETY.md), [validation](docs/TESTING.md) and [limitations](docs/LIMITATIONS.md).\n\n'
            f'The matching source archive is `aula-studio-{version}-source.tar.gz`. It contains the full project README, build scripts and contribution guide.\n\n'
            'Original code: [MIT](LICENSE). Dependencies retain their own licenses; see THIRD_PARTY_NOTICES.txt. '
            'No proprietary artwork is bundled. This project is not affiliated with AULA.\n'
        )
        notices, dependency_count = native_notices()
        (portable / 'THIRD_PARTY_NOTICES.txt').write_text(notices)
        binary_archive = out / f'{portable.name}.tar.gz'
        archive(portable, binary_archive, portable.name)
        p = source / 'packaging/PKGBUILD'
        p.write_text(re.sub(r'^sha256sums=.*', "sha256sums=('"+hashlib.sha256(binary_archive.read_bytes()).hexdigest()+"')", p.read_text(), flags=re.M))
    source_archive = out / f'aula-studio-{version}-source.tar.gz'
    archive(source, source_archive, source.name)
    files = [source_archive] + ([binary_archive] if binary_archive else [])
    (out / 'SHA256SUMS').write_text(''.join(hashlib.sha256(f.read_bytes()).hexdigest()+'  '+f.name+'\n' for f in files))
    report = {'version': version, 'files': included, 'source_archive': str(source_archive),
              'binary_archive': str(binary_archive) if binary_archive else None,
              'binary_hashes': binary_hashes, 'noticed_rust_dependencies': dependency_count,
              'remote_actions': False, 'private_history_exported': False}
    (out / 'manifest.json').write_text(json.dumps(report, indent=2)+'\n')
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--include-binaries', action='store_true', help='Use existing trusted local binaries and cached dependency notices')
    args = parser.parse_args()
    report = prepare(args.include_binaries)
    print(json.dumps({k: v for k, v in report.items() if k != 'files'}, indent=2))
