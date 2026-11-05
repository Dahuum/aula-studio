#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
version=$(python3 -c 'import tomllib;print(tomllib.load(open("Cargo.toml","rb"))["workspace"]["package"]["version"])')
architecture=$(uname -m)
npm --prefix app ci
npm --prefix app run build
cargo build --locked --release -p aula-studio -p aula-cli
stage="dist/aula-studio-${version}-linux-${architecture}"
mkdir -p "$stage/bin" "$stage/share/applications" "$stage/share/icons/hicolor/128x128/apps" "$stage/share/udev" "$stage/docs"
for executable in aula-studio aula; do
  install -m755 "target/release/$executable" "$stage/bin/.$executable.new"
  mv -f "$stage/bin/.$executable.new" "$stage/bin/$executable"
done
install -m644 packaging/aula-studio.desktop "$stage/share/applications/"
install -m644 app/src-tauri/icons/icon.png "$stage/share/icons/hicolor/128x128/apps/aula-studio.png"
install -m644 packaging/70-aula-f75.rules "$stage/share/udev/"
python3 - "$stage" <<'PY'
import importlib.util, sys
from pathlib import Path
root=Path.cwd()
stage=Path(sys.argv[1]).resolve()
if not stage.is_relative_to(root/'dist') or (stage/'.git').exists():
    raise ValueError('Native staging must be an owned project dist/ directory')
spec=importlib.util.spec_from_file_location('exporter', root/'scripts/prepare-public-release.py')
exporter=importlib.util.module_from_spec(spec);spec.loader.exec_module(exporter)
# Keep progress/continuation and stale staging notes out of this build path too.
exporter.stage_public_docs(stage)
for name in ['README.md','LICENSE','CHANGELOG.md','SECURITY.md']:
    (stage/name).write_bytes(exporter.sanitized(name,(root/name).read_bytes()))
notices,_=exporter.native_notices()
(stage/'THIRD_PARTY_NOTICES.txt').write_text(notices)
PY
archive="dist/aula-studio-${version}-linux-${architecture}.tar.gz"
tar -C dist -czf "$archive" "aula-studio-${version}-linux-${architecture}"
python3 - "$archive" "$version" <<'PY'
import hashlib,re,sys
from pathlib import Path
h=hashlib.sha256(Path(sys.argv[1]).read_bytes()).hexdigest()
p=Path('packaging/PKGBUILD');s=re.sub(r'^pkgver=.*','pkgver='+sys.argv[2],p.read_text(),flags=re.M);s=re.sub(r'^sha256sums=.*',"sha256sums=('"+h+"')",s,flags=re.M);p.write_text(s)
PY
sha256sum dist/*.tar.gz > dist/SHA256SUMS
