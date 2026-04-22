#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
from __future__ import annotations

import argparse
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render a Launchpad-ready Debian source tree for the CLI."
    )
    parser.add_argument("--version", required=True, help="Upstream CLI semver, e.g. 0.1.0")
    parser.add_argument("--suite", required=True, help="Ubuntu suite/codename, e.g. noble")
    parser.add_argument(
        "--series-version",
        required=True,
        help="Ubuntu series version for the PPA suffix, e.g. 24.04",
    )
    parser.add_argument(
        "--output-dir",
        required=True,
        help="Directory where the rendered source tree should be written",
    )
    parser.add_argument(
        "--ppa-revision",
        type=int,
        default=1,
        help="PPA revision suffix, defaults to 1",
    )
    return parser.parse_args()


def copy_cli_tree(destination: Path) -> None:
    def _ignore(_: str, names: list[str]) -> set[str]:
        ignored = {".git", "target", ".github", "__pycache__"}
        return {name for name in names if name in ignored or name.endswith(".pyc")}

    shutil.copytree(ROOT, destination, ignore=_ignore)


def vendor_dependencies(source_root: Path) -> None:
    cargo_dir = source_root / ".cargo"
    cargo_dir.mkdir(parents=True, exist_ok=True)
    vendor_output = subprocess.check_output(
        [
            "cargo",
            "vendor",
            "--locked",
            "vendor",
            "--manifest-path",
            str(source_root / "Cargo.toml"),
        ],
        cwd=source_root,
        text=True,
    )
    (cargo_dir / "config.toml").write_text(vendor_output, encoding="utf-8")


def write_debian_packaging(source_root: Path, version: str, suite: str, series_version: str, ppa_revision: int) -> None:
    debian_dir = source_root / "debian"
    source_dir = debian_dir / "source"
    source_dir.mkdir(parents=True, exist_ok=True)

    deb_version = f"{version}-1~ppa{ppa_revision}~ubuntu{series_version}.1"
    rfc2822_date = datetime.now(timezone.utc).strftime("%a, %d %b %Y %H:%M:%S +0000")

    changelog = f"""lukuid-cli ({deb_version}) {suite}; urgency=medium

  * Automated Launchpad release for lukuid-cli {version}.

 -- LukuID <hello@lukuid.com>  {rfc2822_date}
"""

    control = """Source: lukuid-cli
Section: utils
Priority: optional
Maintainer: LukuID <hello@lukuid.com>
Build-Depends:
 debhelper-compat (= 13),
 cargo,
 rustc,
 pkg-config,
 libssl-dev
Standards-Version: 4.7.0
Rules-Requires-Root: no
Homepage: https://github.com/lukuid/cli
Vcs-Browser: https://github.com/lukuid/cli
Vcs-Git: https://github.com/lukuid/cli.git

Package: lukuid-cli
Architecture: any
Depends: ${misc:Depends}, ${shlibs:Depends}, xdg-utils
Description: Open, verify, and browse .luku forensic evidence packages
 LukuID CLI opens, verifies, and inspects .luku forensic evidence archives from
 the terminal. It shares verification logic with the Rust SDK and is suitable
 for local audits, CI checks, and support workflows.
"""

    rules = """#!/usr/bin/make -f

export DEB_BUILD_MAINT_OPTIONS = hardening=+all
export CARGO_HOME = $(CURDIR)/.cargo-home
export CARGO_TARGET_DIR = $(CURDIR)/target
export CARGO_NET_OFFLINE = true

%:
\tdh $@

override_dh_auto_configure:
\tmkdir -p $(CARGO_HOME)
\tcp -a .cargo/config.toml $(CARGO_HOME)/config.toml

override_dh_auto_build:
\tcargo build --release --frozen --offline --locked

override_dh_auto_test:
\t@echo "Skipping upstream tests during Launchpad package builds"

override_dh_auto_install:
\tinstall -D -m0755 target/release/lukuid-cli debian/lukuid-cli/usr/bin/lukuid-cli
"""

    copyright_text = """Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: lukuid-cli
Upstream-Contact: LukuID <hello@lukuid.com>
Source: https://github.com/lukuid/cli

Files: *
Copyright: 2026 LukuID
License: Apache-2.0
 Licensed under the Apache License, Version 2.0.
 .
 On Debian systems, the full text of the Apache-2.0 license can be found in
 /usr/share/common-licenses/Apache-2.0.
"""

    (debian_dir / "changelog").write_text(changelog, encoding="utf-8")
    (debian_dir / "control").write_text(control, encoding="utf-8")
    (debian_dir / "rules").write_text(rules, encoding="utf-8")
    (debian_dir / "copyright").write_text(copyright_text, encoding="utf-8")
    (source_dir / "format").write_text("3.0 (quilt)\n", encoding="utf-8")
    (debian_dir / "compat").unlink(missing_ok=True)
    (debian_dir / "rules").chmod(0o755)


def main() -> int:
    args = parse_args()
    output_dir = Path(args.output_dir).resolve()
    source_root = output_dir / f"lukuid-cli-{args.version}"

    if source_root.exists():
        shutil.rmtree(source_root)
    output_dir.mkdir(parents=True, exist_ok=True)

    copy_cli_tree(source_root)
    vendor_dependencies(source_root)
    write_debian_packaging(
        source_root,
        version=args.version,
        suite=args.suite,
        series_version=args.series_version,
        ppa_revision=args.ppa_revision,
    )
    print(source_root)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
