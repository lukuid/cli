# LukuID CLI

[![CLI CI](https://github.com/lukuid/cli/actions/workflows/ci.yml/badge.svg)](https://github.com/lukuid/cli/actions/workflows/ci.yml)
[![CLI Release](https://github.com/lukuid/cli/actions/workflows/release.yml/badge.svg)](https://github.com/lukuid/cli/actions/workflows/release.yml)
[![CLI Launchpad](https://github.com/lukuid/cli/actions/workflows/launchpad.yml/badge.svg)](https://github.com/lukuid/cli/actions/workflows/launchpad.yml)
[![crates.io](https://img.shields.io/crates/v/lukuid-cli?style=flat-square&logo=rust)](https://crates.io/crates/lukuid-cli)
[![Homebrew](https://img.shields.io/badge/Homebrew-lukuid--cli-orange?style=flat-square)](https://github.com/lukuid/cli/releases)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](LICENSE)

Rust command-line interface for opening, verifying, and browsing `.luku` forensic evidence packages using the shared `lukuid-sdk`.

> License: This repository is licensed under Apache License 2.0 (`SPDX: Apache-2.0`). See [LICENSE](LICENSE).

## Use Cases

This CLI is intended for:

- Auditors who need to inspect a `.luku` archive offline without opening the desktop app.
- Developers who need a fast way to validate `.luku` exports while working on the SDK, firmware, or exporter flows.
- CI and automation jobs that need a machine-readable verification result and a non-zero exit code on critical failures.
- Support and forensic workflows that need to inspect blocks, records, attachment hashes, and verification issues from a terminal session.

## Build And Run

By default, the CLI builds against the public `lukuid-sdk` source repository on GitHub, so a standalone checkout works without a sibling SDK repository.

From this directory:

```bash
cargo build
```

If you want to test local SDK changes without editing `Cargo.toml`, create `cli/.cargo/config.toml` with a source patch that points at your local SDK checkout:

```toml
[patch."https://github.com/lukuid/sdk.git"]
lukuid-sdk = { path = "../../sdk/src/rust/lukuid-sdk" }
```

This preserves the old fast local-development workflow when `cli/` and `sdk/` are checked out side-by-side. An example file is included at `.cargo/config.toml.example`.

Remove `cli/.cargo/config.toml` to switch back to the public SDK repository dependency.

Release build:

```bash
cargo build --release
```

Direct execution during development:

```bash
cargo run -- <command> [options]
```

Show the top-level help:

```bash
cargo run -- --help
```

Show help for a specific command:

```bash
cargo run -- info --help
cargo run -- open --help
cargo run -- verify --help
cargo run -- browse --help
```

## Commands

The CLI currently exposes these user-facing archive commands:

- `info`: open a `.luku` archive and print a high-level summary.
- `open`: open an interactive terminal UI for browsing records and attachments.
- `verify`: run the SDK verification pipeline and report issues.
- `browse`: inspect the archive structure by block and record.

### Info

Purpose:

- Confirm the archive opens successfully.
- Review manifest metadata quickly.
- See record counts, block counts, device IDs, attachment hashes, and record-type distribution.

Command:

```bash
cargo run -- info path/to/evidence.luku
```

Human-readable output:

- Archive path
- Expected mimetype
- Manifest type, version, timestamp, and description
- Block count, total record count, and attachment count
- Device IDs found in the archive
- Record-type counts
- Attachment hashes, if present
- Per-block summary lines showing `block_id`, `added_by`, `device_id`, record count, and record types

Example output shape:

```text
Archive: path/to/evidence.luku
Mimetype: application/vnd.lukuid.package+zip
Manifest: type=LukuArchive version=1.0 created_at_utc=1770825000
Description: Exported 12 records
Blocks: 2 | Records: 12 | Attachments: 1
Devices: LUK-1005-EU
Record types: attachment=1, scan=11
Attachment hashes:
  - a1b2c3...
Blocks:
  - [0] added_by=LUK-1005-EU device_id=LUK-1005-EU records=10 types=scan=10
  - [1] added_by=Vet-Mobile-App device_id=LUK-1005-EU records=2 types=attachment=1, scan=1
```

JSON output:

```bash
cargo run -- info path/to/evidence.luku --json
```

- Emits a single JSON object.
- Includes manifest metadata, counts, device list, record-type aggregates, attachment hashes, block summaries, and a `text` field containing the human-readable rendering.

### Open

Purpose:

- Browse records interactively in the terminal.
- Open extracted attachments in the host system's default viewer.
- Inspect a `.luku` archive without manually drilling through JSON output.

Command:

```bash
cargo run -- open path/to/evidence.luku
```

### Verify

Purpose:

- Run the Rust SDK verification logic against a `.luku` archive.
- Detect critical evidence failures such as broken block linkage, record-chain breaks, signature failures, attachment corruption, or hash mismatches.
- Produce a CI-safe exit code.

Command:

```bash
cargo run -- verify path/to/evidence.luku
```

Verbose verifier trace:

```bash
LUKUID_SDK_DEBUG=1 cargo run -- verify path/to/evidence.luku
```

Overriding trust profile (for development certificates):

```bash
LUKUID_TRUST_PROFILE=dev cargo run -- verify path/to/evidence.luku
```

- Emits SDK verifier diagnostics to `stderr` with per-block and per-record context.
- Useful when a failing archive produces repeated issue codes and you need to see which record index, counter, timestamp, or attestation step caused each failure.

Human-readable output:

- Archive path
- Verification status:
  - `verified`
  - `verified_with_warnings`
  - `invalid`
- Counts of `critical`, `warning`, and `info` issues
- A list of verification issues, if any

Example output shape:

```text
Archive: path/to/evidence.luku
Verification status: verified_with_warnings (critical=0 warning=1 info=0)
Blocks: 2 | Records: 12 | Attachments: 1
Issues:
  - [warning] ATTESTATION_CHAIN_MISSING: Missing DAC attestation chain for device LUK-1005-EU.
```

JSON output:

```bash
cargo run -- verify path/to/evidence.luku --json
```

- Emits a JSON object containing:
  - `status`
  - `counts`
  - `issues`
  - `text`

Fixture-friendly verification options:

```bash
cargo run -- verify path/to/evidence.luku \
  --allow-untrusted-roots \
  --skip-certificate-temporal-checks
```

- `--allow-untrusted-roots` is useful for local test exports or incomplete chains.
- `--skip-certificate-temporal-checks` is useful when testing fixtures that do not have realistic issuance windows.

Exit codes:

- `0`: archive opened and no critical verification issues were found.
- `2`: archive opened, verification ran, and at least one critical issue was found.
- `1`: command usage error, open failure, or unexpected runtime error.

### Browse

Purpose:

- Inspect archive structure incrementally.
- Browse by block first, then drill into a specific record.
- Review individual record JSON in a terminal without opening the full desktop viewer.

List all blocks:

```bash
cargo run -- browse path/to/evidence.luku
```

Human-readable output:

- Archive path
- One summary line per block with:
  - block index
  - `timestamp_utc`
  - `added_by`
  - `device_id`
  - record count
  - attachment count

Inspect a specific block:

```bash
cargo run -- browse path/to/evidence.luku --block 0
```

Human-readable output:

- Block metadata
- One summary line per record in that block with:
  - record index
  - `type`
  - best-effort primary ID
  - `ctr`
  - `timestamp_utc`
  - shortened `signature`

Inspect a specific record:

```bash
cargo run -- browse path/to/evidence.luku --block 0 --record 2 --show-payload
```

- `--record` requires `--block`.
- Without `--show-payload`, the CLI trims heavy nested fields and shows summary-safe output.
- With `--show-payload`, the CLI prints the full record JSON.

JSON output:

```bash
cargo run -- browse path/to/evidence.luku --json
cargo run -- browse path/to/evidence.luku --block 0 --json
cargo run -- browse path/to/evidence.luku --block 0 --record 2 --json
```

- Archive-level browse returns block summaries.
- Block-level browse returns the selected block plus record summaries.
- Record-level browse returns the selected block summary and the selected record.

## Notes

- The CLI expects the `.luku` archive `mimetype` entry to be `application/vnd.lukuid.package+zip`.
- `verify` exits `2` when critical verification issues are detected, which makes it suitable for CI and scripted evidence checks.
- The CLI reads `.luku` archives through the shared Rust SDK. If `.luku` verification logic changes in the SDK, this CLI inherits that behavior.
- The documented `open`, `verify`, and `browse` output contract is covered by CLI unit tests to reduce drift between the README and the binary behavior.
