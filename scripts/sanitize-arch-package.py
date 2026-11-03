#!/usr/bin/env python3
"""Omit host package inventory from an owned makepkg artifact; verify its payload.

Only .BUILDINFO and its mtree entry are removed. All other file bytes are kept.
The input/output stay in this project. No installation or hardware access.
"""
import argparse
import gzip
import hashlib
import io
from pathlib import Path
import re
import subprocess
import tarfile

ROOT=Path(__file__).resolve().parents[1]

def sanitize(source, destination):
    source,destination=Path(source).resolve(),Path(destination).resolve()
    if not source.is_relative_to(ROOT) or not destination.is_relative_to(ROOT/'dist') or source==destination:
        raise ValueError('Use a project-local input and separate dist/ output')
    raw=subprocess.check_output(['zstd','-d','-c',str(source)])
    if len(raw)>128*1024*1024:
        raise ValueError('Unexpected package expansion')
    payload=io.BytesIO()
    with tarfile.open(fileobj=io.BytesIO(raw))as original,tarfile.open(fileobj=payload,mode='w',format=tarfile.PAX_FORMAT)as clean:
        names=original.getnames()
        if len(names)!=len(set(names)) or '.PKGINFO' not in names or '.MTREE' not in names:
            raise ValueError('Invalid package manifest')
        info=original.extractfile('.PKGINFO').read()
        if b'pkgname = aula-studio\n' not in info or b'arch = x86_64\n' not in info:
            raise ValueError('Expected the owned AULA package')
        for member in original.getmembers():
            path=Path(member.name)
            if path.is_absolute() or '..' in path.parts or not(member.isfile()or member.isdir()) or member.uid or member.gid:
                raise ValueError('Unsafe package member')
            if member.name=='.BUILDINFO':continue
            data=original.extractfile(member).read()if member.isfile()else None
            if member.name=='.MTREE':
                lines=gzip.decompress(data).decode().splitlines()
                lines=[line for line in lines if not line.startswith('./.BUILDINFO ')]
                for line in lines:
                    match=re.search(r'^\./([^ ]+).*sha256digest=([a-f0-9]{64})',line)
                    if match:
                        assert hashlib.sha256(original.extractfile(match[1]).read()).hexdigest()==match[2],match[1]
                data=gzip.compress(('\n'.join(lines)+'\n').encode(),mtime=0)
                member.size=len(data)
            clean.addfile(member,io.BytesIO(data)if data is not None else None)
    destination.parent.mkdir(parents=True,exist_ok=True)
    with destination.open('wb')as stream:
        subprocess.run(['zstd','-q','-T0','-19','-c'],input=payload.getvalue(),stdout=stream,check=True)
    with tarfile.open(fileobj=io.BytesIO(subprocess.check_output(['zstd','-d','-c',str(destination)])))as final:
        assert '.BUILDINFO' not in final.getnames()
        assert b'.BUILDINFO' not in gzip.decompress(final.extractfile('.MTREE').read())
    print(hashlib.sha256(destination.read_bytes()).hexdigest()+'  '+destination.name)

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source');parser.add_argument('destination')
    args=parser.parse_args();sanitize(args.source,args.destination)
