#!/usr/bin/env python3
"""Package only our native runner builds. Never executes a vendor installer."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import platform
import shutil
import subprocess
import tarfile
import tomllib
import zipfile

ROOT=Path(__file__).resolve().parents[1]
VERSION=tomllib.loads((ROOT/'Cargo.toml').read_text())['workspace']['package']['version']

def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()

def prepare(target):
    spec=importlib.util.spec_from_file_location('exporter',ROOT/'scripts/prepare-public-release.py')
    exporter=importlib.util.module_from_spec(spec);spec.loader.exec_module(exporter)
    out=ROOT/'dist/native-resources';out.mkdir(parents=True,exist_ok=True)
    notices,count=exporter.native_notices(target)
    (out/'THIRD_PARTY_NOTICES.txt').write_text(notices)
    shutil.copyfile(ROOT/'LICENSE',out/'LICENSE.txt')
    exe='aula.exe' if 'windows' in target else 'aula'
    cli=ROOT/'target'/target/'release'/exe
    assert cli.is_file(), 'Build our native CLI before preparing resources'
    config={'bundle':{'resources':{str(out/'THIRD_PARTY_NOTICES.txt'):'THIRD_PARTY_NOTICES.txt',
            str(out/'LICENSE.txt'):'LICENSE.txt',str(cli):exe}}}
    (out/'tauri-native.json').write_text(json.dumps(config,indent=2)+'\n')
    print(json.dumps({'resources':str(out),'dependency_notices':count}))

def package(target):
    windows='windows' in target
    assert (windows and platform.system()=='Windows') or ('apple' in target and platform.system()=='Darwin')
    arch='aarch64' if target.startswith('aarch64') else 'x86_64'
    system='windows' if windows else 'macos'
    prefix=f'aula-studio-{VERSION}-{system}-{arch}'
    release=ROOT/'target'/target/'release';out=ROOT/'dist/native-desktop';out.mkdir(parents=True,exist_ok=True)
    candidates=list((release/'bundle'/('nsis' if windows else 'dmg')).glob('*.exe' if windows else '*.dmg'))
    assert len(candidates)==1, 'Expected exactly one freshly built native installer'
    installer=out/(prefix+('-setup.exe' if windows else '.dmg'))
    assert not installer.exists(), 'Do not replace a reviewed native artifact'
    shutil.copyfile(candidates[0],installer)
    payload=out/prefix;payload.mkdir()
    cli=release/('aula.exe' if windows else 'aula')
    shutil.copyfile(cli,payload/cli.name);(payload/cli.name).chmod(0o755)
    for name in ['LICENSE.txt','THIRD_PARTY_NOTICES.txt']:
        shutil.copyfile(ROOT/'dist/native-resources'/name,payload/name)
    (payload/'README.txt').write_text(f'AULA Studio {VERSION} native {system}/{arch} preview.\n'
        'Unsigned; macOS is not notarized. Windows needs Microsoft WebView2.\n'
        'Native build/window tests are separate from keyboard hardware validation.\n'
        'USB operations retain descriptor, device-identity and explicit backed-up review gates.\n'
        'No firmware/reset, wireless configuration or onboard macro uploader.\n'
        'https://github.com/Dahuum/aula-studio\n')
    if windows:
        binary=release/'aula-studio.exe';shutil.copyfile(binary,payload/binary.name)
        portable=out/(prefix+'.zip')
        with zipfile.ZipFile(portable,'w',zipfile.ZIP_DEFLATED) as archive:
            for p in payload.iterdir():archive.write(p,str(Path(prefix)/p.name))
    else:
        binary=release/'bundle/macos/AULA Studio.app/Contents/MacOS/aula-studio'
        portable=out/(prefix+'-cli.tar.gz')
        with tarfile.open(portable,'w:gz') as archive:
            for p in payload.iterdir():archive.add(p,arcname=str(Path(prefix)/p.name),recursive=False)
    assert binary.is_file()
    report={'version':VERSION,'target':target,'platform':platform.system(),'architecture':arch,
            'signed':False,'notarized':False,'physical_validation':False,'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'binary_sha256':digest(binary),'cli_sha256':digest(cli),'files':{p.name:digest(p) for p in [installer,portable]}}
    (out/(prefix+'-build.json')).write_text(json.dumps(report,indent=2)+'\n')
    (out/'SHA256SUMS').write_text(''.join(digest(p)+'  '+p.name+'\n' for p in sorted(out.iterdir()) if p.is_file() and p.name!='SHA256SUMS'))
    print(json.dumps(report,indent=2))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('action',choices=['prepare','package']);parser.add_argument('--target',required=True)
    args=parser.parse_args();(prepare if args.action=='prepare' else package)(args.target)
