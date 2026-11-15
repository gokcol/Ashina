#!/usr/bin/env python3
"""Exercise signing isolation and failure handling with synthetic keys only."""
import base64
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
PUBLIC = base64.b64encode(b"public test key".ljust(32, b".")).decode()
SECRET = base64.b64encode(b"synthetic test seed".ljust(32, b".")).decode()
STUB = r'''#!/usr/bin/env python3
import os, pathlib, sys
args = sys.argv[1:]
root = pathlib.Path(os.environ['FIXTURE_ROOT'])
mode = os.environ['TEST_MODE']
secret = os.environ['TEST_SECRET']
if pathlib.Path(sys.argv[0]).name == 'generate_keys':
    if '-p' in args:
        print('wrong-key' if mode == 'mismatch' else os.environ['TEST_PUBLIC'])
    else:
        (root/'export-called').touch()
        path = pathlib.Path(args[args.index('-x') + 1])
        assert path.parent.stat().st_mode & 0o777 == 0o700
        path.write_text('invalid' if mode == 'invalid' else secret)
        assert path.stat().st_mode & 0o777 == 0o600
        if mode == 'unsafe': path.chmod(0o644)
        if mode == 'export-failed':
            print(secret, file=sys.stderr)
            sys.exit(7)
else:
    (root/'signer-called').touch()
    assert args[:2] == ['--ed-key-file', '-']
    assert secret not in ' '.join(sys.argv)
    assert not list((root/'tmp').iterdir()), 'export must be gone before signing'
    assert sys.stdin.read() == secret
    print('signature-created')
    if mode == 'signer-failed':
        print(secret, file=sys.stderr)
        sys.exit(7)
'''


class SigningTests(unittest.TestCase):
    def run_case(self, mode, arguments=()):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'bin').mkdir()
            (root / 'tmp').mkdir()
            for tool in ('generate_keys', 'generate_appcast', 'sign_update'):
                path = root / 'bin' / tool
                path.write_text(STUB)
                path.chmod(0o700)
            env = dict(os.environ, FIXTURE_ROOT=str(root), TMPDIR=str(root / 'tmp'),
                       TEST_MODE=mode, TEST_SECRET=SECRET, TEST_PUBLIC=PUBLIC)
            result = subprocess.run([
                sys.executable, '-B', str(ROOT / 'scripts/sparkle-sign.py'),
                '--sparkle-root', str(root), '--public-key', PUBLIC,
                'generate_appcast', *arguments,
            ], env=env, capture_output=True, text=True)
            self.assertNotIn(SECRET, result.stdout + result.stderr)
            self.assertEqual(list((root / 'tmp').iterdir()), [])
            return result, (root / 'export-called').exists(), (root / 'signer-called').exists()

    def test_key_is_private_and_removed_before_signing(self):
        result, exported, signed = self.run_case('ok', ['--embed-release-notes', 'archives'])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(exported and signed)
        self.assertIn('signature-created', result.stdout)

    def test_wrong_public_key_never_exports(self):
        result, exported, signed = self.run_case('mismatch')
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(exported or signed)

    def test_export_failure_is_cleaned_and_never_logged(self):
        result, exported, signed = self.run_case('export-failed')
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue(exported)
        self.assertFalse(signed)

    def test_invalid_or_exposed_key_never_reaches_signer(self):
        for mode in ('invalid', 'unsafe'):
            with self.subTest(mode=mode):
                result, exported, signed = self.run_case(mode)
                self.assertNotEqual(result.returncode, 0)
                self.assertTrue(exported)
                self.assertFalse(signed)

    def test_signer_failure_is_preserved_and_key_redacted(self):
        result, _, signed = self.run_case('signer-failed')
        self.assertEqual(result.returncode, 7)
        self.assertTrue(signed)
        self.assertIn('[redacted]', result.stderr)

    def test_key_source_cannot_be_overridden(self):
        for option in ('--ed-key-file=other', '--account', '-s', '-f'):
            with self.subTest(option=option):
                result, exported, signed = self.run_case('ok', [option])
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(exported or signed)


if __name__ == '__main__':
    unittest.main()
