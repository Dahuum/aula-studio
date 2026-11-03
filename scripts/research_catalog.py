#!/usr/bin/env python3
"""Inventory official direct Windows driver downloads; never execute their code.

Outputs stay under project analysis/. Downloads are opt-in, HTTPS-only and bounded.
No hosted landing pages, browser drivers or firmware-only packages are downloaded.
"""
import argparse
import concurrent.futures
import hashlib
from html.parser import HTMLParser
import json
import os
from pathlib import Path
import re
import shutil
import threading
import time
import tempfile
from urllib.parse import unquote, urljoin, urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener

ROOT = Path(__file__).resolve().parents[1]
SOURCE = 'https://www.aulagaming.com/pages/download'
ALLOWED_HOSTS = {'cdn.shopify.com', 'www.aulastar.com', 'aulastar.com'}
EXTENSIONS = {'.exe', '.zip', '.7z', '.rar', '.msi'}


class CatalogParser(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.cards = []
        self.card = None
        self.in_name = False
    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == 'article' and 'data-driver-card' in attrs:
            self.card = {'name': '', 'device': attrs.get('data-driver-device'),
                         'type': attrs.get('data-driver-type'), 'url': None, 'product_url': None}
        if self.card is None:
            return
        if tag == 'h3':
            self.in_name = True
        if tag == 'a':
            href = attrs.get('href', '')
            if 'data-driver-download' in attrs:
                self.card['url'] = urljoin(SOURCE, href)
            elif '/products/' in href:
                self.card['product_url'] = urljoin(SOURCE, href)
    def handle_data(self, data):
        if self.card is not None and self.in_name:
            self.card['name'] += data
    def handle_endtag(self, tag):
        if tag == 'h3':
            self.in_name = False
        if tag == 'article' and self.card is not None:
            self.card['name'] = ' '.join(self.card['name'].split())
            self.cards.append(self.card)
            self.card = None
            self.in_name = False


def classify(card):
    url = urlsplit(card.get('url') or '')
    if card['type'] == 'firmware':
        return 'excluded_firmware_only'
    if card['type'] == 'web_driver':
        return 'excluded_browser_configurator'
    if card['type'] != 'driver':
        return 'excluded_unknown_role'
    if url.scheme != 'https':
        return 'excluded_non_https'
    if Path(unquote(url.path)).suffix.lower() not in EXTENSIONS:
        return 'excluded_hosted_landing_page'
    if url.hostname not in ALLOWED_HOSTS:
        return 'excluded_unreviewed_download_host'
    return 'eligible_direct_windows_package'


def sha256(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(chunk)
    return value.hexdigest()


class RestrictedRedirect(HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, newurl):
        parsed = urlsplit(newurl)
        if parsed.scheme != 'https' or parsed.hostname not in ALLOWED_HOSTS:
            raise ValueError('Redirect outside reviewed HTTPS download hosts')
        return super().redirect_request(request, fp, code, msg, headers, newurl)


def download(card, out, max_bytes):
    stem = re.sub(r'[^a-z0-9]+', '-', card['name'].lower()).strip('-')[:80]
    suffix = Path(unquote(urlsplit(card['url']).path)).suffix.lower()
    destination = out / 'downloads' / (stem + suffix)
    destination.parent.mkdir(parents=True, exist_ok=True)
    metadata = destination.with_suffix(destination.suffix + '.metadata.json')
    if destination.exists():
        if not destination.is_file() or destination.is_symlink():
            raise ValueError('Unexpected existing artifact path')
        if destination.stat().st_size > max_bytes or not metadata.is_file():
            raise ValueError('Existing artifact exceeds bounds or lacks provenance; preserve it and choose a new research output')
        cached = json.loads(metadata.read_text())
        actual = sha256(destination)
        if cached.get('source_url') != card['url'] or cached.get('sha256') != actual or cached.get('bytes') != destination.stat().st_size:
            raise ValueError('Existing artifact provenance mismatch; preserve it and choose a new research output')
        return {'status': 'existing_download', 'path': str(destination.relative_to(ROOT)),
                'size': destination.stat().st_size, 'sha256': actual}
    descriptor, temporary_name = tempfile.mkstemp(prefix=destination.name+'-',suffix='.part',dir=destination.parent)
    os.close(descriptor)
    temporary = Path(temporary_name)
    total = 0
    digest = hashlib.sha256()
    started = time.monotonic()
    opener = build_opener(RestrictedRedirect())
    try:
        with opener.open(Request(card['url'], headers={'User-Agent':'AULA-Studio-static-research/1.0'}), timeout=30) as response:
            declared = response.headers.get('Content-Length')
            if declared and int(declared) > max_bytes:
                raise ValueError('Declared package exceeds download size cap')
            if 'html' in response.headers.get('Content-Type', '').lower():
                raise ValueError('Download returned HTML rather than a Windows package')
            with temporary.open('wb') as stream:
                while True:
                    if time.monotonic() - started > 180:
                        raise TimeoutError('Bounded package download exceeded 180 seconds')
                    chunk = response.read(1024 * 1024)
                    if not chunk:
                        break
                    total += len(chunk)
                    if total > max_bytes:
                        raise ValueError('Package exceeds download size cap')
                    stream.write(chunk)
                    digest.update(chunk)
            final_url = response.url
            content_type = response.headers.get('Content-Type')
        with temporary.open('rb') as stream:
            magic = stream.read(8)
        if not magic.startswith((b'MZ', b'PK\x03\x04', b'7z\xbc\xaf\x27\x1c', b'Rar!', b'\xd0\xcf\x11\xe0')):
            raise ValueError('Unrecognized Windows executable/archive signature')
        temporary.chmod(0o644)
        os.replace(temporary, destination)
        metadata.write_text(json.dumps({'source_url':card['url'],'sha256':digest.hexdigest(),'bytes':total},indent=2)+'\n')
        return {'status':'downloaded', 'path':str(destination.relative_to(ROOT)), 'size':total,
                'sha256':digest.hexdigest(), 'final_url':final_url, 'content_type':content_type,
                'execution_allowed':False}
    finally:
        if temporary.exists():
            temporary.unlink()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--page', type=Path, default=ROOT/'analysis/catalog-2026-11-03/download-page.html')
    parser.add_argument('--output', type=Path, default=ROOT/'analysis/catalog-2026-11-03')
    parser.add_argument('--download', action='store_true')
    parser.add_argument('--defer', action='append', default=[])
    parser.add_argument('--max-bytes', type=int, default=512*1024*1024)
    options = parser.parse_args()
    out = options.output.resolve()
    if not out.is_relative_to(ROOT/'analysis'):
        raise SystemExit('Research output must remain under project analysis/')
    if not 1 <= options.max_bytes <= 1024*1024*1024:
        raise SystemExit('Download cap must be between1 byte and1GiB')
    out.mkdir(parents=True, exist_ok=True)
    page = options.page.read_bytes()
    parsed = CatalogParser(); parsed.feed(page.decode('utf-8'))
    if not parsed.cards:
        raise SystemExit('No driver cards parsed; refusing guessed downloads')
    for card in parsed.cards:
        card['classification'] = classify(card)
        card['acquisition'] = {'status':'not_requested'}
        if card['name'] in options.defer:
            card['acquisition'] = {'status':'deferred_to_parallel_research'}
    report = {'schema_version':1, 'source_url':SOURCE, 'source_page_sha256':hashlib.sha256(page).hexdigest(),
              'retrieved_at_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
              'windows_code_executed':False, 'max_download_bytes':options.max_bytes, 'cards':parsed.cards}
    manifest = out/'manifest.json'
    lock = threading.Lock()
    def save():
        temporary = manifest.with_suffix('.json.tmp')
        temporary.write_text(json.dumps(report,indent=2)+'\n')
        temporary.replace(manifest)
    save()
    eligible = [c for c in parsed.cards if c['classification']=='eligible_direct_windows_package' and c['name'] not in options.defer]
    eligible.sort(key=lambda c:(c['device']!='keyboard',c['device']!='software',c['name']))
    if options.download:
        if shutil.disk_usage(out).free < 3*1024**3:
            raise SystemExit('Insufficient free space for bounded research downloads')
        def acquire(card):
            try:
                status = download(card,out,options.max_bytes)
            except Exception as error:
                status = {'status':'failed','error':str(error)}
            with lock:
                card['acquisition'] = status; save()
            print(card['name']+': '+status['status'], flush=True)
        with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
            list(pool.map(acquire,eligible))
    from collections import Counter
    print(json.dumps({'cards':len(parsed.cards), 'classifications':dict(Counter(c['classification'] for c in parsed.cards)),
                      'acquisitions':dict(Counter(c['acquisition']['status'] for c in parsed.cards)),
                      'manifest':str(manifest.relative_to(ROOT))},indent=2))


if __name__ == '__main__':
    main()
