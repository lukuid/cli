// SPDX-License-Identifier: Apache-2.0
use anyhow::Error;

pub fn handle_error(error: Error) -> i32 {
    eprintln!("error: {error:#}");
    1
}
