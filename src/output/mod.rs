// SPDX-License-Identifier: Apache-2.0
pub mod human;
pub mod json;

use serde_json::Value;
use anyhow::Result;

pub fn print_output(value: &Value, json_mode: bool) -> Result<()> {
    if json_mode {
        json::print_json(value)
    } else if let Some(text) = value.get("text").and_then(Value::as_str) {
        println!("{text}");
        Ok(())
    } else {
        json::print_json(value)
    }
}
