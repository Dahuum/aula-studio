#!/usr/bin/env python3
"""Refuse Pages publication when an advertised GitHub release asset is missing."""
from concurrent.futures import ThreadPoolExecutor
from html.parser import HTMLParser
import json
from pathlib import Path
import urllib.request


class Downloads(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = set()

    def handle_starttag(self, tag, attrs):
        if tag == 'a':
            value = dict(attrs).get('href', '')
            if value.startswith(self.prefix):
                self.links.add(value)


def check(url):
    request = urllib.request.Request(url, method='HEAD', headers={'User-Agent': 'AULA-Studio-release-check'})
    with urllib.request.urlopen(request, timeout=30) as response:
        if response.status != 200 or not response.url.startswith('https://'):
            raise ValueError('Download is unavailable: ' + url)
    return {'url': url, 'status': 200}


def main():
    root = Path(__file__).resolve().parents[1]
    project = json.loads((root / 'site/project.json').read_text(encoding='utf-8'))
    repository = project['repository_url'].rstrip('/')
    if not repository.startswith('https://github.com/'):
        raise ValueError('Expected the configured public GitHub repository')
    parser = Downloads()
    parser.prefix = repository + '/releases/download/'
    parser.feed((root / 'site/dist/index.html').read_text(encoding='utf-8'))
    if len(parser.links) < 8:
        raise ValueError('Expected Windows, both Mac targets, four Linux formats and checksums')
    with ThreadPoolExecutor(max_workers=4) as pool:
        results = list(pool.map(check, sorted(parser.links)))
    print(json.dumps({'status': 'passed', 'release_downloads': results}, indent=2))


if __name__ == '__main__':
    main()
