import importlib.util
import os
from pathlib import Path
import tarfile
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('exporter', ROOT / 'scripts/prepare-public-release.py')
exporter = importlib.util.module_from_spec(spec)
spec.loader.exec_module(exporter)
spec = importlib.util.spec_from_file_location('site_builder', ROOT / 'scripts/build-site.py')
site_builder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(site_builder)


class PublicationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        (ROOT / "target").mkdir(exist_ok=True)
        (ROOT / "dist").mkdir(exist_ok=True)
    def test_private_or_proprietary_paths_are_excluded(self):
        for name in ['analysis/extracted/app/OemDrv.exe', 'original/installer-manifest.md', 'AULA F75.exe',
                     'docs/CONTINUATION.md', 'docs/PROGRESS.md', 'docs/FINAL_REPORT.md',
                     'app/node_modules/example/index.js', 'app/dist/index.html', 'site/dist/index.html',
                     '../secret.rs', '/tmp/secret.rs', 'fixtures/raw-device.bin', 'publication/account.json']:
            self.assertFalse(exporter.allowed(name), name)
    def test_buildable_source_and_attribution_are_retained(self):
        for name in ['Cargo.lock', 'README.md', 'LICENSE', 'crates/aula-protocol/src/lib.rs',
                     'app/src-tauri/src/startup.rs', 'app/package-lock.json', 'fixtures/keymap/LICENSE',
                     'fixtures/keymap/f87-factory-base.bin', 'site/index.html', 'third_party/licenses/index.json']:
            self.assertTrue(exporter.allowed(name), name)
    def test_likely_credentials_stop_export(self):
        with self.assertRaises(ValueError):
            exporter.sanitized('app/src/example.ts', ('ghp_' + 'x' * 36).encode())
    def test_home_paths_are_redacted_in_public_docs(self):
        result = exporter.sanitized('docs/example.md', b'Build /home/alice/Desktop/aula and /home/alice/.cargo')
        self.assertNotIn(b'/home/alice', result)
        self.assertIn(b'<project>', result)
    def test_archives_are_deterministic_and_have_no_host_ownership_or_git(self):
        with tempfile.TemporaryDirectory(dir=ROOT / 'target') as temporary:
            root = Path(temporary); source = root / 'src'; source.mkdir()
            (source / 'README.md').write_text('Public source')
            (source / '.git').mkdir(); (source / '.git/config').write_text('private metadata')
            (source / 'link').symlink_to(source / 'README.md')
            first = root / 'first.tar.gz'; second = root / 'second.tar.gz'
            exporter.archive(source, first, 'aula-studio')
            os.utime(source / 'README.md', (17, 17))
            exporter.archive(source, second, 'aula-studio')
            self.assertEqual(first.read_bytes(), second.read_bytes())
            with tarfile.open(first) as tar:
                self.assertEqual(tar.getnames(), ['aula-studio/README.md'])
                self.assertEqual(tar.getmembers()[0].uid, 0)
                self.assertEqual(tar.getmembers()[0].mtime, 0)
    def test_invalid_publication_urls_are_rejected_before_export(self):
        for url in ['http://github.com/user/repo', 'https://user:password@github.com/user/repo',
                    'https://example.com/user/repo', 'https://github.com/user/repo?token=value']:
            with self.assertRaises(ValueError):
                site_builder.build(repository_url=url)
    def test_existing_publication_git_repository_is_never_deleted(self):
        with tempfile.TemporaryDirectory(dir=ROOT / 'dist') as temporary:
            out = Path(temporary)
            git = out / '.git'
            git.mkdir()
            marker = git / 'config'
            marker.write_text('preserve existing repository')
            with self.assertRaisesRegex(ValueError, 'Git repository'):
                exporter.prepare(output=out)
            self.assertEqual(marker.read_text(), 'preserve existing repository')


if __name__ == '__main__':
    unittest.main()
