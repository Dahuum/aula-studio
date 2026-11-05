#!/usr/bin/env python3
"""Bounded static extraction/inventory of the reviewed AULA catalog artifacts.

Never executes Windows code. Proprietary outputs remain under project analysis/.
Already-extracted trees are inventoried without being overwritten.
"""
import argparse
import hashlib
from html.parser import HTMLParser
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import struct
import shutil
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]
MAX_MEMBERS = 10000
MAX_EXPANDED = 2 * 1024**3


def safe_name(name):
    path = PurePosixPath(name.replace('\\', '/'))
    if path.is_absolute() or '..' in path.parts or ':' in name or '\x00' in name:
        raise ValueError('Unsafe artifact path')
    return path


def digest(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024*1024), b''):
            value.update(chunk)
    return value.hexdigest()


def run(command, log, env=None, timeout=120):
    result = subprocess.run(command, env=env, text=True, capture_output=True, timeout=timeout)
    log.parent.mkdir(parents=True, exist_ok=True)
    log.write_text(result.stdout+result.stderr)
    return result


def inert_tree(folder):
    files = 0
    size = 0
    for path in folder.rglob('*'):
        if path.is_symlink():
            raise ValueError('Extracted link rejected')
        if path.is_file():
            files += 1
            size += path.stat().st_size
            if files > MAX_MEMBERS or size > MAX_EXPANDED:
                raise ValueError('Extracted output exceeded bounds')
            path.chmod(0o644)


def inno_extract(source, output, logs):
    local_tool = ROOT/'analysis/tools/usr/bin/innoextract'
    tool = Path(shutil.which('innoextract') or local_tool)
    if not tool.is_file():
        raise RuntimeError('A native innoextract utility is required; install it or provide the project-local analysis/tools version')
    env = dict(os.environ)
    if tool == local_tool:
        env['LD_LIBRARY_PATH'] = str(ROOT/'analysis/tools/usr/lib')
    listing = run([str(tool),'--list','--color=0',str(source)],logs/'inno-list.txt',env)
    if listing.returncode:
        return {'format':'unsupported_by_innoextract','error':listing.stderr.strip()[:500]}
    names = re.findall(r'^ - "([^"]+)"',listing.stdout,re.M)
    if not 0 < len(names) <= MAX_MEMBERS:
        raise ValueError('Invalid/oversized Inno file inventory')
    for name in names:
        safe_name(name)
    expanded = sum(float(n)*{'B':1,'KiB':1024,'MiB':1024**2,'GiB':1024**3}[u]
                   for n,u in re.findall(r'\(([0-9.]+) (B|KiB|MiB|GiB)\)',listing.stdout))
    if expanded > MAX_EXPANDED:
        raise ValueError('Inno expansion exceeds budget')
    if not output.exists() or not any(output.iterdir()):
        output.mkdir(parents=True,exist_ok=True)
        result = run([str(tool),'--extract','--color=0','--timestamps=none','--output-dir',str(output),str(source)],logs/'inno-extract.txt',env)
        if result.returncode:
            raise RuntimeError('Inno extraction failed; inspect local log')
    inert_tree(output)
    return {'format':'Inno Setup','file_count':len(names),'listed_uncompressed_bytes':int(expanded),
            'extracted_root':str(output.relative_to(ROOT))}


def seven_inventory(source, log):
    listing = run(['7z','l','-slt',str(source)],log)
    if listing.returncode:
        raise RuntimeError('7z did not recognize nested archive')
    records = []
    for block in listing.stdout.split('\n\n'):
        values = dict(line.split(' = ',1) for line in block.splitlines() if ' = ' in line)
        name = values.get('Path')
        if not name or name == str(source):
            continue
        if 'Physical Size' in values:
            continue
        safe_name(name)
        if 'Symbolic Link' in values or 'Hard Link' in values:
            raise ValueError('Nested archive link rejected')
        records.append(values)
    if len(records)>MAX_MEMBERS:
        raise ValueError('Nested archive member cap')
    if sum(int(r.get('Size') or 0) for r in records)>MAX_EXPANDED:
        raise ValueError('Nested archive expansion cap')
    return records


def advanced_container(source_bytes, name):
    """Decode one explicitly named F2088 Advanced Installer container.

    Its trailer records the length/offset; flag2 complements only the first512
    bytes. This is static byte decoding, never bootstrapper execution.
    """
    if len(source_bytes) > 64*1024**2 or not source_bytes.startswith(b'MZ') or not source_bytes.endswith(b'ADVINSTSFX'):
        raise ValueError('Expected bounded Advanced Installer artifact')
    if name not in {'ShinetekTools.msi','MainFeature.cab','English.cab','ChineseSimplified.cab'}:
        raise ValueError('Container name is not allowlisted')
    token=name.encode('utf-16-le')
    index=source_bytes.rfind(token,max(0,len(source_bytes)-65536))
    if index < 20:
        raise ValueError('Named container table record absent')
    flags,encoding,size,offset,length=struct.unpack_from('<5I',source_bytes,index-20)
    if encoding != 2 or length != len(name) or size < 512 or size > 32*1024**2 or offset < 64 or offset+size > index-20:
        raise ValueError('Invalid container bounds/encoding')
    raw=source_bytes[offset:offset+size]
    decoded=bytes(value ^ 255 for value in raw[:512])+raw[512:]
    signature=bytes.fromhex('d0cf11e0a1b11ae1') if name.endswith('.msi') else b'MSCF'
    if not decoded.startswith(signature):
        raise ValueError('Decoded container signature mismatch')
    return decoded, {'container':name,'offset':offset,'bytes':size,'table_offset':index-20,
                     'encoding':'first512_bytes_complement_ff','sha256':hashlib.sha256(decoded).hexdigest()}


def f2088_extract(source, base):
    source,base=source.resolve(),base.resolve()
    if not source.is_relative_to(ROOT/"analysis") or not base.is_relative_to(ROOT/"analysis"):
        raise ValueError("F2088 research paths must stay under project analysis")
    if digest(source) != '8731252ed943e177d659558aef0f0a091a1f630f474abfb35da9459b5b0ed3ee':
        raise ValueError('F2088 installer differs from the investigated artifact')
    payload=source.read_bytes()
    containers=base/'decoded-containers/f2088';containers.mkdir(parents=True,exist_ok=True)
    recovered=[]
    for name in ['ShinetekTools.msi','MainFeature.cab','English.cab','ChineseSimplified.cab']:
        decoded,record=advanced_container(payload,name)
        destination=containers/name
        if destination.exists():
            if digest(destination)!=record['sha256']:raise ValueError('Existing decoded container differs')
        else:
            with destination.open('xb') as stream:stream.write(decoded)
            destination.chmod(0o644)
        recovered.append(record)
    output=base/'extracted/f2088'
    for name,folder in [('MainFeature.cab','app'),('English.cab','english'),('ChineseSimplified.cab','chinese')]:
        archive=containers/name;records=seven_inventory(archive,base/('inventory/f2088/'+folder+'-list.txt'))
        if not records or len(records)>300 or sum(int(r.get('Size') or 0) for r in records)>64*1024**2:
            raise ValueError('F2088 archive expansion cap')
        destination=output/folder
        if not destination.exists() or not any(destination.iterdir()):
            destination.mkdir(parents=True,exist_ok=True)
            result=run(['7z','x','-y','-o'+str(destination),str(archive)],base/('inventory/f2088/'+folder+'-extract.txt'))
            if result.returncode:raise RuntimeError('F2088 CAB extraction failed')
    inert_tree(output)
    return {'format':'Advanced Installer / prefix512 complement / MSI and MSZip CAB',
            'extracted_root':str(output.relative_to(ROOT)), 'containers':recovered,
            'windows_code_executed':False,'bundled_drivers':'inert, never installed or executed'}


def hub_extract(source, base):
    outer = base/'extracted/aula-hub-driver'
    if not outer.exists() or not any(outer.iterdir()):
        with zipfile.ZipFile(source) as zipped:
            members = zipped.infolist()
            if not 0<len(members)<=MAX_MEMBERS or sum(m.file_size for m in members)>MAX_EXPANDED:
                raise ValueError('Hub ZIP expansion cap')
            for member in members:
                safe_name(member.filename)
                if stat.S_ISLNK(member.external_attr>>16):
                    raise ValueError('Hub ZIP symbolic link rejected')
            outer.mkdir(parents=True,exist_ok=True)
            zipped.extractall(outer)
    inert_tree(outer)
    candidates = list(outer.rglob('*.exe'))
    if len(candidates)!=1:
        raise ValueError('Hub package installer selection is ambiguous')
    setup = candidates[0]
    inventory = seven_inventory(setup,base/'hub-7z-list.txt')
    if not any(r.get('Path')=='$PLUGINSDIR/app-64.7z' for r in inventory):
        raise ValueError('Expected explicit Hub NSIS member absent')
    nsis = base/'extracted/aula-hub-nsis'
    nsis.mkdir(parents=True,exist_ok=True)
    nested = nsis/'app-64.7z'
    if not nested.exists():
        result = run(['7z','e','-y','-o'+str(nsis),str(setup),'$PLUGINSDIR/app-64.7z'],base/'hub-nsis-extract.txt')
        if result.returncode:
            raise RuntimeError('Hub NSIS extraction failed')
    nested.chmod(0o644)
    seven_inventory(nested,base/'hub-app64-list.txt')
    output = base/'extracted/aula-hub'
    asar = output/'resources/app.asar'
    if not asar.exists():
        output.mkdir(parents=True,exist_ok=True)
        result = run(['7z','x','-y','-o'+str(output),str(nested),'resources/app.asar','resources/app.asar.unpacked/*','resources/app-update.yml'],base/'hub-resources-extract.txt')
        if result.returncode:
            raise RuntimeError('Hub Electron resources extraction failed')
    inert_tree(output)
    return {'format':'ZIP / NSIS / 7z / Electron ASAR',
            'extracted_root':str(output.relative_to(ROOT)),
            'artifact_chain':[{'path':str(p.relative_to(ROOT)),'sha256':digest(p),'bytes':p.stat().st_size}
                              for p in [source,setup,nested,asar]],
            'firmware_and_other_runtime_helpers':'indexed only; not executed'}


class DeviceModes(HTMLParser):
    """Read-only tolerance for vendor XML's invalid raw ampersands in HID IDs."""
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.modes = []
    def handle_starttag(self, tag, attrs):
        values = dict(attrs)
        if 'vid' in values or 'pid' in values:
            self.modes.append({'element':tag,**values})


def static_facts(folder):
    app=folder/'app'
    facts={'name':folder.name,'configuration_sources':[],'top_level_executables':[],
           'device_modes':[],'legacy_usb_lines':[],'layout_files':[],
           'bundled_firmware_index_only':[],'configuration_parser':'permissive tokenization, not XML validation'}
    if not app.exists():
        return facts
    for file in app.iterdir():
        if file.is_file() and file.suffix.lower()=='.exe':
            facts['top_level_executables'].append({'path':str(file.relative_to(ROOT)),'bytes':file.stat().st_size,'sha256':digest(file)})
    for file in app.rglob('*'):
        if not file.is_file():
            continue
        relative=file.relative_to(app)
        if 'firmware' in [p.lower() for p in relative.parts] or file.name.lower()=='update.exe':
            facts['bundled_firmware_index_only'].append(str(file.relative_to(ROOT)))
            continue
        if file.name.lower()=='config.xml':
            raw=file.read_bytes()
            if len(raw)>2*1024**2:
                raise ValueError('Unexpected config size')
            parser=DeviceModes();parser.feed(raw.decode('utf-8-sig',errors='replace'))
            facts['configuration_sources'].append(str(file.relative_to(ROOT)))
            facts['device_modes'].extend(parser.modes)
        if file.name.lower() in ['cfg.ini','kb.ini']:
            raw=file.read_bytes()
            if len(raw)>2*1024**2:
                raise ValueError('Unexpected INI size')
            text=raw.decode('utf-16' if raw.startswith((b'\xff\xfe',b'\xfe\xff')) else 'utf-8',errors='replace')
            facts['configuration_sources'].append(str(file.relative_to(ROOT)))
            facts['legacy_usb_lines'].extend({'source':str(file.relative_to(ROOT)),'line':line} for line in text.splitlines()
                if re.search(r'^(?:VID|PID|Psd|Company|Name|Device|Hid|Rpt|Version|Fw|Firmware|Usage|Usb)[\w ]*\s*=',line,re.I))
        if 'layouts' in relative.parts and file.suffix.lower()=='.xml':
            facts['layout_files'].append(str(file.relative_to(ROOT)))
    return facts


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest',type=Path,default=ROOT/'analysis/catalog-2026-11-03/manifest.json')
    options=parser.parse_args()
    base=options.manifest.resolve().parent
    if not base.is_relative_to(ROOT/'analysis'):
        raise SystemExit('Research input/output must stay under project analysis/')
    report=json.loads(options.manifest.read_text())
    extracted=[]
    for card in report['cards']:
        status=card['acquisition']
        if status['status'] not in ['downloaded','existing_download']:
            continue
        source=(ROOT/status['path']).resolve()
        if not source.is_relative_to(base) or digest(source)!=status['sha256']:
            raise ValueError('Catalog artifact identity mismatch')
        if card['name']=='F2088':
            item=f2088_extract(source,base)
        elif card['name']=='AULA Hub Driver':
            item=hub_extract(source,base)
        else:
            item=inno_extract(source,base/'extracted'/source.stem,base/'inventory'/source.stem)
        item['name']=card['name']
        extracted.append(item)
        print(card['name']+': '+item['format'],flush=True)
    (base/'extraction-summary.json').write_text(json.dumps(extracted,indent=2)+'\n')
    facts=[static_facts(folder) for folder in sorted((base/'extracted').iterdir())
           if folder.is_dir() and not folder.name.startswith('aula-hub')]
    (base/'static-facts.json').write_text(json.dumps(facts,indent=2)+'\n')


if __name__=='__main__':
    main()
