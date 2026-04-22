#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VERSION_FILE = ROOT / "VERSION"
CARGO_FILE = ROOT / "Cargo.toml"
SEMVER_RE = re.compile(r"^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$")
PACKAGE_VERSION_RE = re.compile(r'(?m)^version = "[^"]+"$')
SDK_DEP_RE = re.compile(
    r'lukuid-sdk = \{ git = "https://github\.com/lukuid/sdk\.git", version = "[^"]+", default-features = false \}'
)


def fail(message: str) -> int:
    print(message, file=sys.stderr)
    return 1


def load_version() -> str:
    version = VERSION_FILE.read_text(encoding="utf-8").strip()
    if not SEMVER_RE.match(version):
        raise ValueError(f"VERSION must be semver, got {version!r}")
    return version


def apply(version: str) -> int:
    if not SEMVER_RE.match(version):
        return fail(f"Refusing to write non-semver version: {version}")

    VERSION_FILE.write_text(f"{version}\n", encoding="utf-8")

    content = CARGO_FILE.read_text(encoding="utf-8")
    content, count = PACKAGE_VERSION_RE.subn(f'version = "{version}"', content, count=1)
    if count != 1:
        return fail("Could not update package version in Cargo.toml")

    CARGO_FILE.write_text(content, encoding="utf-8")
    return 0


def check(version: str) -> int:
    if not SEMVER_RE.match(version):
        return fail(f"VERSION must be semver, got {version!r}")

    content = CARGO_FILE.read_text(encoding="utf-8")
    mismatches: list[str] = []

    package_match = PACKAGE_VERSION_RE.search(content)
    if package_match is None:
        mismatches.append("Cargo.toml package version is missing")
    else:
        package_version = package_match.group(0).split('"')[1]
        if package_version != version:
            mismatches.append(f"Cargo.toml package version is {package_version!r}")

    if SDK_DEP_RE.search(content) is None:
        mismatches.append(
            "Cargo.toml lukuid-sdk dependency must use the public GitHub SDK source with default-features = false"
        )

    if mismatches:
        for mismatch in mismatches:
            print(mismatch, file=sys.stderr)
        return 1

    print(f"CLI version metadata matches {version}")
    return 0


def main(argv: list[str]) -> int:
    if len(argv) < 2 or argv[1] not in {"check", "apply"}:
        return fail("Usage: version_sync.py <check|apply> [version]")
    command = argv[1]
    version = argv[2] if len(argv) > 2 else load_version()
    if command == "apply":
        return apply(version)
    return check(version)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
