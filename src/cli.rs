// SPDX-License-Identifier: Apache-2.0
use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "lukuid-cli",
    version,
    about = "Open, verify, and browse .luku forensic evidence archives"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Print a high-level summary of a .luku archive.
    Info {
        /// Path to the .luku archive.
        path: PathBuf,
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Open a UI to interactively view records and attachments.
    Open {
        /// Path to the .luku archive.
        path: PathBuf,
    },
    /// Verify a .luku archive using the Rust SDK verification pipeline.
    Verify {
        /// Path to the .luku archive.
        path: PathBuf,
        /// Allow records without trusted roots. Useful for local fixtures.
        #[arg(long)]
        allow_untrusted_roots: bool,
        /// Skip certificate time-bound checks.
        #[arg(long)]
        skip_certificate_temporal_checks: bool,
        /// Trusted root fingerprints for external identity verification.
        #[arg(long)]
        trusted_external_fingerprint: Vec<String>,
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Browse archive contents by block and record.
    Browse {
        /// Path to the .luku archive.
        path: PathBuf,
        /// Select a block by zero-based block index.
        #[arg(long)]
        block: Option<usize>,
        /// Select a record by zero-based index within the chosen block.
        #[arg(long)]
        record: Option<usize>,
        /// Include full payloads and nested objects in record detail output.
        #[arg(long)]
        show_payload: bool,
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Create a new .luku archive (placeholder).
    Export {
        /// Path to the source directory or file.
        path: PathBuf,
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Talk to LukuID hardware devices (placeholder).
    Device {
        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}
