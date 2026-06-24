# LukuID CLI

[![CLI CI](https://github.com/lukuid/cli/actions/workflows/ci.yml/badge.svg)](https://github.com/lukuid/cli/actions/workflows/ci.yml)
[![CLI Release](https://github.com/lukuid/cli/actions/workflows/release.yml/badge.svg)](https://github.com/lukuid/cli/actions/workflows/release.yml)
[![crates.io](https://img.shields.io/crates/v/lukuid-cli?style=flat-square&logo=rust)](https://crates.io/crates/lukuid-cli)
[![Homebrew](https://img.shields.io/badge/Homebrew-lukuid--cli-orange?style=flat-square&logo=homebrew)](https://github.com/lukuid/homebrew-tap)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](LICENSE)

A lightweight, blazing-fast Rust command-line tool to open, verify, and interactively browse `.luku` forensic evidence packages. Backed by the shared LukuID Cryptographic SDK, this tool brings secure offline verification straight to your terminal.

---

## Key Capabilities

*   🛡️ **Offline Verification**: Perform rigorous cryptographic birth and chain checks on `.luku` files without a network connection.
*   🖥️ **Interactive Terminal UI (TUI)**: Drill down through block structures, view record histories, and extract attachments directly.
*   🤖 **CI/CD Ready**: Machine-readable JSON output and standard exit codes make it ideal for automated pipeline validation.
*   ⚡ **Zero-Config Install**: Standalone binaries available for macOS, Linux, and Windows.

---

## Installation

### macOS (Homebrew)
Add the official tap and install the CLI:
```bash
brew tap lukuid/tap
brew install lukuid-cli
```

### Via Cargo (Rust Users)
If you have Rust and Cargo installed, compile the latest release directly from crates.io:
```bash
cargo install lukuid-cli
```

### Pre-compiled Binaries
You can also download standalone binaries for your platform directly from the [GitHub Releases](https://github.com/lukuid/cli/releases) page.

---

## 3-Minute Quick Start

### 1. Verify an Archive (Cryptographic Integrity)
Run the LukuID verification pipeline on any `.luku` package:
```bash
lukuid-cli verify path/to/evidence.luku
```
_Exits with `0` on success, `2` on critical cryptographic or structural issues, and `1` on read errors._

### 2. Quick Summary (`info`)
Output metadata, devices, record counts, and block chains:
```bash
lukuid-cli info path/to/evidence.luku
```
```text
Archive: path/to/evidence.luku
Manifest: type=LukuArchive version=1.0.0 created_at_utc=1770825000
Blocks: 2 | Records: 12 | Attachments: 1
Devices: LUK-1005-EU
Record types: attachment=1, scan=11
Blocks:
  - [0] added_by=LUK-1005-EU device_id=LUK-1005-EU records=10 types=scan=10
  - [1] added_by=Vet-Mobile-App device_id=LUK-1005-EU records=2 types=attachment=1, scan=1
```

### 3. Open the Interactive Terminal Browser (`open`)
Launch the full terminal user interface to browse records and extract files on-the-fly:
```bash
lukuid-cli open path/to/evidence.luku
```

---

## Command Reference

The LukuID CLI provides four core commands:

| Command | Purpose | Common Options |
| :--- | :--- | :--- |
| **`verify`** | Run full cryptographic audit pipeline | `--json`, `--allow-untrusted-roots`, `--require-continuity` |
| **`info`** | Print high-level overview of the archive | `--json` |
| **`open`** | Open interactive terminal browser (TUI) | _None_ |
| **`browse`** | Inspect JSON layout of specific blocks/records | `--block <idx>`, `--record <idx>`, `--show-payload` |

### Advanced Verification Options
To skip certificate temporal constraints (for testing old fixtures) or to trust custom external root certificates:
```bash
# Verify using development certificates
LUKUID_TRUST_PROFILE=dev lukuid-cli verify evidence.luku

# Add a trusted root certificate fingerprint manually
lukuid-cli verify evidence.luku --trusted-external-fingerprint <SHA256_FINGERPRINT>
```

---

## Development Guide

### Build from Source
First, clone this repository, then run the compilation steps:
```bash
cargo build --release
```
Your compiled binary will be located at `target/release/lukuid-cli`.

### Sibling-SDK Development Setup
By default, the CLI compiles against the public `lukuid-sdk` repository on GitHub. If you are developing local SDK changes side-by-side with the CLI, create `.cargo/config.toml` inside this directory to patch the source:

```toml
[patch."https://github.com/lukuid/sdk.git"]
lukuid-sdk = { path = "../sdk/src/rust/lukuid-sdk" }
```
An example template is available at `.cargo/config.toml.example`. Remove this file to fall back to the public repository dependency.

---

## Versioning and release

The LukuID CLI release version is governed by a single source of truth: [`VERSION`](VERSION).

When you want to cut a new CLI release:

1. Update the canonical semver and sync the Cargo.toml manifest:
   `python3 scripts/version_sync.py apply 1.0.8`
2. Ensure the `lukuid-sdk` dependency in `Cargo.toml` points to the remote GitHub repository and the intended release commit/version (not a local path).
3. Update the Rust lockfile to match the new version:
   `cargo check`
4. Verify that the managed package version matches:
   `python3 scripts/version_sync.py check`
5. Review the resulting manifest changes and commit them normally.
6. Create the release tag as `v1.0.8` after the release commit is on `main`.

The release workflows do not publish on every merge. They only publish when all of the following are true:

- the ref is a semver tag in the form `vX.Y.Z`
- the tag version matches `VERSION`
- the tagged commit is reachable from `main`

This means a normal merge without a version bump is safe: CI can still test and build the CLI, but no package will be published.

If you merge release-relevant changes without bumping the version, the repository simply remains ahead of the last published CLI release until you later update `VERSION`, run the sync script, commit the versioned manifest, and tag that commit. Reusing an already-published version number is not safe, because registry publishes are immutable and the publish jobs will fail once they try to push an existing version.
