// SPDX-License-Identifier: Apache-2.0
use serde_json::Value;
use anyhow::Result;

pub fn print_json(value: &Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
