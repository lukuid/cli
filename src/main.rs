// SPDX-License-Identifier: Apache-2.0
mod cli;
mod commands;
mod output;
mod error;

use clap::Parser;
use cli::{Cli, Commands};
use error::handle_error;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Info { path, json } => {
            commands::inspect::run_info(&path, json)
        }
        Commands::Open { path } => {
            commands::open::run_open(&path)
        }
        Commands::Verify {
            path,
            allow_untrusted_roots,
            skip_certificate_temporal_checks,
            trusted_external_fingerprint,
            json,
        } => {
            commands::verify::run_verify(
                &path,
                allow_untrusted_roots,
                skip_certificate_temporal_checks,
                trusted_external_fingerprint,
                json,
            )
        }
        Commands::Browse {
            path,
            block,
            record,
            show_payload,
            json,
        } => {
            commands::inspect::run_browse(&path, block, record, show_payload, json)
        }
        Commands::Export { path, json } => {
            commands::export::run_export(&path, json)
        }
        Commands::Device { json } => {
            commands::device::run_device(json)
        }
    };

    match result {
        Ok(code) => std::process::exit(code),
        Err(e) => std::process::exit(handle_error(e)),
    }
}
