import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('nsis_smoke', Path(__file__).resolve().parents[1] / 'scripts/windows-installer-smoke.py')
smoke = importlib.util.module_from_spec(spec)
spec.loader.exec_module(smoke)


class InstallerIdentityTests(unittest.TestCase):
    def test_exact_or_single_bundle_tag_are_accepted(self):
        original = b'code\0__TAURI_BUNDLE_TYPE_VAR_UNK\0resources'
        self.assertEqual(smoke.verify_binary_identity(original, original), 'exact bytes')
        smoke.verify_binary_identity(original.replace(b'_UNK', b'_NSS'), original)

    def test_unrelated_changes_are_rejected(self):
        original = b'code\0__TAURI_BUNDLE_TYPE_VAR_UNK\0resources'
        nsis = original.replace(b'_UNK', b'_NSS')
        for changed in [nsis + b'X', nsis.replace(b'code', b'evil'), nsis[1:],
                        b'__TAURI_BUNDLE_TYPE_VAR_NSS' + nsis,
                        nsis.replace(b'_NSS', b'_MSI')]:
            with self.subTest(changed=changed), self.assertRaises(AssertionError):
                smoke.verify_binary_identity(changed, original)
