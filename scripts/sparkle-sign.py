#!/usr/bin/env python3
"""Run a pinned Sparkle signer using its original, Keychain-trusted key reader.

generate_keys owns the Keychain item it created. Other ad-hoc signed Sparkle
tools can fail authorization even while that reader still works. Use Sparkle's
supported export/stdin interfaces without changing the key or broadening its
Keychain ACL. The export is private, short-lived, and removed before signing.
Never put key material in arguments, environment variables, or diagnostic logs.
"""

import argparse
import base64
import binascii
from pathlib import Path
import stat
import subprocess
import sys
import tempfile


class SigningError(Exception):
    pass


def read_signing_key(sparkle, account, expected_public):
    reader = str(Path(sparkle) / "bin/generate_keys")
    result = subprocess.run(
        [reader, "--account", account, "-p"], capture_output=True, timeout=60
    )
    if result.returncode or result.stdout.strip() != expected_public.encode("ascii"):
        raise SigningError("The Keychain signing key does not match SUPublicEDKey.")

    # generate_keys writes atomically and cannot export straight to a pipe.
    # TemporaryDirectory is mode 0700; the child also uses a restrictive umask.
    with tempfile.TemporaryDirectory(prefix="skypdf-sparkle-") as directory:
        key_file = Path(directory) / "key"
        result = subprocess.run(
            [reader, "--account", account, "-x", str(key_file)],
            capture_output=True, timeout=60, umask=0o077,
        )
        if result.returncode:
            raise SigningError("Sparkle could not read the signing key from Keychain.")
        info = key_file.lstat()
        if not stat.S_ISREG(info.st_mode) or info.st_mode & 0o077:
            raise SigningError("Sparkle did not create a private key export safely.")
        if info.st_size > 256:
            raise SigningError("Sparkle returned an invalid signing key.")
        secret = key_file.read_bytes().strip()
        key_file.unlink()
    try:
        decoded = base64.b64decode(secret, validate=True)
    except (ValueError, binascii.Error):
        raise SigningError("Sparkle returned an invalid signing key.") from None
    if len(decoded) not in (32, 96):
        raise SigningError("Sparkle returned an invalid signing key.")
    return secret


def run_signer(sparkle, account, expected_public, tool, arguments):
    if tool not in ("generate_appcast", "sign_update"):
        raise SigningError("Unsupported Sparkle signing tool.")
    # Keep the key source under this wrapper's control.
    if any(arg.split("=", 1)[0] in ("--ed-key-file", "--account", "-s", "-f")
           for arg in arguments):
        raise SigningError("Do not override the Sparkle signing key source.")
    secret = read_signing_key(sparkle, account, expected_public)
    result = subprocess.run(
        [str(Path(sparkle) / "bin" / tool), "--ed-key-file", "-", *arguments],
        input=secret, capture_output=True,
    )
    # Sparkle's usual output contains only public signatures and archive details.
    # Redact the key defensively even if an upstream diagnostic changes.
    sys.stdout.buffer.write(result.stdout.replace(secret, b"[redacted]"))
    sys.stderr.buffer.write(result.stderr.replace(secret, b"[redacted]"))
    return result.returncode


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sparkle-root", required=True)
    parser.add_argument("--public-key", required=True)
    parser.add_argument("--account", default="MyPDF")
    parser.add_argument("tool", choices=("generate_appcast", "sign_update"))
    parser.add_argument("arguments", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    try:
        return run_signer(args.sparkle_root, args.account, args.public_key,
                          args.tool, args.arguments)
    except (SigningError, OSError, subprocess.TimeoutExpired) as error:
        # Subprocess output is deliberately excluded, including on export failure.
        message = str(error) if isinstance(error, SigningError) else "Sparkle signing could not complete."
        print(message, file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
