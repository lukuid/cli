// SPDX-License-Identifier: Apache-2.0
use serde_json::Value;
use std::collections::BTreeMap;

pub fn shorten(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        value.to_string()
    } else {
        let prefix = value
            .chars()
            .take(max_chars.saturating_sub(3))
            .collect::<String>();
        format!("{prefix}...")
    }
}

pub fn join_or_dash(values: Vec<String>) -> String {
    if values.is_empty() {
        "-".to_string()
    } else {
        values.join(", ")
    }
}

pub fn primary_record_id(record: &Value) -> String {
    const CANDIDATES: [&str; 7] = [
        "scan_id",
        "event_id",
        "custody_id",
        "parent_record_id",
        "attachment_id",
        "location_id",
        "id",
    ];

    for key in CANDIDATES {
        if let Some(value) = record.get(key).and_then(Value::as_str) {
            return value.to_string();
        }
        if let Some(value) = record
            .get("payload")
            .and_then(|payload| payload.get(key))
            .and_then(Value::as_str)
        {
            return value.to_string();
        }
    }

    "-".to_string()
}

pub fn count_record_types(records: &[Value]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for record in records {
        let kind = record
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        *counts.entry(kind).or_insert(0) += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn primary_record_id_prefers_explicit_identifier_fields() {
        let record = json!({
            "type": "scan",
            "scan_id": "SCAN-123",
            "payload": {
                "id": "fallback-id"
            }
        });

        assert_eq!(primary_record_id(&record), "SCAN-123");
    }

    #[test]
    fn primary_record_id_supports_custody_records() {
        let record = json!({
            "type": "custody",
            "custody_id": "CUST-123",
            "payload": {
                "event": "handoff",
                "status": "received"
            }
        });

        assert_eq!(primary_record_id(&record), "CUST-123");
    }
}
