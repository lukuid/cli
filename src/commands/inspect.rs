// SPDX-License-Identifier: Apache-2.0
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Value};
use lukuid_sdk::{LukuFile, LUKU_MIMETYPE};
use crate::output::human::{shorten, join_or_dash, primary_record_id, count_record_types};

pub fn run_info(path: &Path, json_mode: bool) -> Result<i32> {
    let luku = LukuFile::open(path).map_err(|e| anyhow!("{e}"))?;
    let output = archive_summary(&luku, path);
    crate::output::print_output(&output, json_mode)?;
    Ok(0)
}

pub fn run_browse(
    path: &Path,
    block_index: Option<usize>,
    record_index: Option<usize>,
    show_payload: bool,
    json_mode: bool,
) -> Result<i32> {
    if record_index.is_some() && block_index.is_none() {
        bail!("--record requires --block");
    }

    let luku = LukuFile::open(path).map_err(|e| anyhow!("{e}"))?;
    let output = browse_output(&luku, path, block_index, record_index, show_payload)?;
    crate::output::print_output(&output, json_mode)?;
    Ok(0)
}

fn archive_summary(luku: &LukuFile, path: &Path) -> Value {
    let block_summaries: Vec<Value> = luku.blocks.iter().map(block_summary_json).collect();
    let devices: BTreeSet<String> = luku
        .blocks
        .iter()
        .map(|block| block.device.device_id.clone())
        .collect();
    let record_types = aggregate_record_types(luku);
    let attachments = attachment_paths(luku);
    let total_records: usize = luku.blocks.iter().map(|block| block.batch.len()).sum();

    let mut text = String::new();
    text.push_str(&format!("Archive: {}\n", path.display()));
    text.push_str(&format!("Mimetype: {LUKU_MIMETYPE}\n"));
    text.push_str(&format!(
        "Manifest: type={} version={} created_at_utc={}\n",
        luku.manifest.r#type, luku.manifest.version, luku.manifest.created_at_utc
    ));
    text.push_str(&format!("Description: {}\n", luku.manifest.description));
    text.push_str(&format!(
        "Blocks: {} | Records: {} | Attachments: {}\n",
        luku.blocks.len(),
        total_records,
        luku.attachments.len()
    ));
    text.push_str(&format!(
        "Devices: {}\n",
        join_or_dash(devices.iter().cloned().collect())
    ));
    text.push_str(&format!(
        "Record types: {}\n",
        join_or_dash(
            record_types
                .iter()
                .map(|(kind, count)| format!("{kind}={count}"))
                .collect(),
        )
    ));

    if !attachments.is_empty() {
        text.push_str("Attachment hashes:\n");
        for hash in &attachments {
            text.push_str(&format!("  - {hash}\n"));
        }
    }

    if !block_summaries.is_empty() {
        text.push_str("Blocks:\n");
        for block in &block_summaries {
            text.push_str(&format!(
                "  - [{}] device={} records={} types={}\n",
                block["block_id"],
                block["device_id"].as_str().unwrap_or("-"),
                block["record_count"],
                join_or_dash(
                    block["record_types"]
                        .as_object()
                        .map(|types| {
                            types
                                .iter()
                                .map(|(kind, count)| format!("{kind}={}", count))
                                .collect()
                        })
                        .unwrap_or_default(),
                ),
            ));
        }
    }

    json!({
        "path": path.display().to_string(),
        "mimetype": LUKU_MIMETYPE,
        "manifest": &luku.manifest,
        "manifest_sig_present": !luku.manifest_sig.trim().is_empty(),
        "block_count": luku.blocks.len(),
        "record_count": total_records,
        "attachment_count": luku.attachments.len(),
        "devices": devices.into_iter().collect::<Vec<_>>(),
        "record_types": record_types,
        "attachment_hashes": attachments,
        "blocks": block_summaries,
        "text": text,
    })
}

fn browse_output(
    luku: &LukuFile,
    path: &Path,
    block_index: Option<usize>,
    record_index: Option<usize>,
    show_payload: bool,
) -> Result<Value> {
    let mut text = String::new();
    text.push_str(&format!("Archive: {}\n", path.display()));

    match (block_index, record_index) {
        (None, None) => {
            let blocks: Vec<Value> = luku.blocks.iter().map(block_summary_json).collect();
            text.push_str("Blocks:\n");
            for block in &blocks {
                text.push_str(&format!(
                    "  - [{}] timestamp_utc={} device_id={} records={} attachments={}\n",
                    block["block_id"],
                    block["timestamp_utc"],
                    block["device_id"].as_str().unwrap_or("-"),
                    block["record_count"],
                    block["attachment_count"],
                ));
            }
            Ok(json!({
                "path": path.display().to_string(),
                "scope": "archive",
                "blocks": blocks,
                "text": text,
            }))
        }
        (Some(block_idx), None) => {
            let block = luku
                .blocks
                .get(block_idx)
                .with_context(|| format!("block index {} out of range", block_idx))?;
            let records: Vec<Value> = block
                .batch
                .iter()
                .enumerate()
                .map(|(index, record)| record_summary_json(index, record))
                .collect();

            text.push_str(&format!(
                "Block [{}]: timestamp_utc={} device_id={} records={}\n",
                block.block_id,
                block.timestamp_utc,
                block.device.device_id,
                block.batch.len()
            ));
            text.push_str("Records:\n");
            for record in &records {
                text.push_str(&format!(
                    "  - [{}] type={} id={} ctr={} timestamp_utc={} signature={}\n",
                    record["index"],
                    record["type"].as_str().unwrap_or("-"),
                    record["id"].as_str().unwrap_or("-"),
                    record["ctr"],
                    record["timestamp_utc"],
                    shorten(record["signature"].as_str().unwrap_or("-"), 18),
                ));
            }

            Ok(json!({
                "path": path.display().to_string(),
                "scope": "block",
                "block": block_summary_json(block),
                "records": records,
                "text": text,
            }))
        }
        (Some(block_idx), Some(record_idx)) => {
            let block = luku
                .blocks
                .get(block_idx)
                .with_context(|| format!("block index {} out of range", block_idx))?;
            let record = block.batch.get(record_idx).with_context(|| {
                format!(
                    "record index {} out of range for block {}",
                    record_idx, block_idx
                )
            })?;

            let display_record = record_detail_json(record, show_payload);
            text.push_str(&format!(
                "Block [{}] Record [{}]: type={} id={}\n",
                block.block_id,
                record_idx,
                record
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown"),
                primary_record_id(record),
            ));
            text.push_str(&serde_json::to_string_pretty(&display_record)?);
            text.push('\n');

            Ok(json!({
                "path": path.display().to_string(),
                "scope": "record",
                "block": block_summary_json(block),
                "record_index": record_idx,
                "record": display_record,
                "text": text,
            }))
        }
        (None, Some(_)) => bail!("--record requires --block"),
    }
}

fn block_summary_json(block: &lukuid_sdk::LukuBlock) -> Value {
    let attachment_count = block
        .batch
        .iter()
        .filter(|record| record.get("type").and_then(Value::as_str) == Some("attachment"))
        .count();
    let record_types = count_record_types(&block.batch);

    json!({
        "block_id": block.block_id,
        "timestamp_utc": block.timestamp_utc,
        "device_id": block.device.device_id,
        "public_key": shorten(&block.device.public_key, 24),
        "record_count": block.batch.len(),
        "attachment_count": attachment_count,
        "record_types": record_types,
        "previous_block_hash": block.previous_block_hash,
        "batch_hash": block.batch_hash,
        "block_hash": block.block_hash,
    })
}

fn record_summary_json(index: usize, record: &Value) -> Value {
    json!({
        "index": index,
        "type": record.get("type").and_then(Value::as_str).unwrap_or("unknown"),
        "id": primary_record_id(record),
        "ctr": payload_u64(record, "ctr"),
        "timestamp_utc": payload_u64(record, "timestamp_utc").or_else(|| record.get("timestamp_utc").and_then(Value::as_u64)),
        "signature": shorten(record.get("signature").and_then(Value::as_str).unwrap_or(""), 18),
        "previous_signature": shorten(record.get("previous_signature").or_else(|| record.get("parent_signature")).and_then(Value::as_str).unwrap_or(""), 18),
    })
}

fn record_detail_json(record: &Value, show_payload: bool) -> Value {
    if show_payload {
        return record.clone();
    }

    let mut trimmed = record.clone();
    if let Some(object) = trimmed.as_object_mut() {
        if let Some(payload) = object.remove("payload") {
            let payload_keys = payload
                .as_object()
                .map(|map| map.keys().cloned().collect::<Vec<_>>())
                .unwrap_or_default();
            object.insert("payload_keys".to_string(), json!(payload_keys));
        }
        if let Some(identity) = object.get("identity").cloned() {
            if identity.is_object() {
                let keys = identity
                    .as_object()
                    .map(|map| map.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                object.insert("identity_keys".to_string(), json!(keys));
                object.remove("identity");
            }
        }
    }
    trimmed
}

fn payload_u64(record: &Value, field: &str) -> Option<u64> {
    record
        .get("payload")
        .and_then(|payload| payload.get(field))
        .and_then(Value::as_u64)
}

fn aggregate_record_types(luku: &LukuFile) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for block in &luku.blocks {
        for (kind, count) in count_record_types(&block.batch) {
            *counts.entry(kind).or_insert(0) += count;
        }
    }
    counts
}

fn attachment_paths(luku: &LukuFile) -> Vec<String> {
    let mut hashes = luku.attachments.keys().cloned().collect::<Vec<_>>();
    hashes.sort();
    hashes
}

#[cfg(test)]
pub (crate) mod tests {
    use super::*;
    use lukuid_sdk::luku::LukuDeviceIdentity;
    use lukuid_sdk::{LukuBlock, LukuManifest};
    use std::collections::HashMap;

    #[test]
    fn record_detail_json_hides_payload_without_show_payload() {
        let record = json!({
            "type": "scan",
            "payload": {
                "ctr": 7,
                "timestamp_utc": 123
            },
            "identity": {
                "identity_version": 1,
                "signature": "abc"
            }
        });

        let display = record_detail_json(&record, false);
        let object = display.as_object().expect("object");

        assert!(!object.contains_key("payload"));
        assert!(!object.contains_key("identity"));
        assert_eq!(
            object.get("payload_keys"),
            Some(&json!(vec!["ctr", "timestamp_utc"]))
        );
        assert_eq!(
            object.get("identity_keys"),
            Some(&json!(vec!["identity_version", "signature"]))
        );
    }

    #[test]
    fn archive_summary_text_matches_documented_sections() {
        let luku = sample_luku_file();
        let output = archive_summary(&luku, Path::new("fixtures/demo.luku"));
        let text = output["text"].as_str().expect("text output");

        assert!(text.contains("Archive: fixtures/demo.luku"));
        assert!(text.contains(&format!("Mimetype: {}", LUKU_MIMETYPE)));
        assert!(text.contains("Blocks: 1 | Records: 2 | Attachments: 1"));
        assert!(text.contains("Devices: LUK-CLI-1"));
        assert!(text.contains("Record types: attachment=1, scan=1"));
        assert!(text.contains("Attachment hashes:"));
        assert!(text.contains("Blocks:\n  - [0] device=LUK-CLI-1 records=2"));
    }

    #[test]
    fn browse_outputs_use_documented_labels() {
        let luku = sample_luku_file();

        let archive_output =
            browse_output(&luku, Path::new("fixtures/demo.luku"), None, None, false)
                .expect("archive browse");
        let archive_text = archive_output["text"].as_str().expect("archive text");
        assert!(archive_text.contains("timestamp_utc=1770825000"));
        assert!(archive_text.contains("device_id=LUK-CLI-1"));

        let block_output =
            browse_output(&luku, Path::new("fixtures/demo.luku"), Some(0), None, false)
                .expect("block browse");
        let block_text = block_output["text"].as_str().expect("block text");
        assert!(block_text.contains("Block [0]: timestamp_utc=1770825000 device_id=LUK-CLI-1 records=2"));
        assert!(block_text.contains("timestamp_utc=1770823456"));

        let record_output = browse_output(
            &luku,
            Path::new("fixtures/demo.luku"),
            Some(0),
            Some(0),
            false,
        )
        .expect("record browse");
        let record = &record_output["record"];
        assert!(record.get("payload").is_none());
        assert_eq!(record["payload_keys"], json!(vec!["ctr", "timestamp_utc"]));
    }

    pub (crate) fn sample_luku_file() -> LukuFile {
        LukuFile::from_parts(
            LukuManifest {
                r#type: "LukuArchive".to_string(),
                version: "1.0".to_string(),
                created_at_utc: 1770825000,
                description: "CLI fixture".to_string(),
                blocks_hash: "abc123".to_string(),
                extra: HashMap::new(),
            },
            "manifest-signature".to_string(),
            vec![LukuBlock {
                block_id: 0,
                timestamp_utc: 1770825000,
                previous_block_hash: None,
                device: LukuDeviceIdentity {
                    device_id: "LUK-CLI-1".to_string(),
                    public_key: "ZGVtb19wdWJsaWNfa2V5".to_string(),
                },
                attestation_dac_der: None,
                attestation_manufacturer_der: None,
                attestation_intermediate_der: None,
                attestation_root_fingerprint: None,
                heartbeat_slac_der: None,
                heartbeat_der: None,
                heartbeat_intermediate_der: None,
                heartbeat_root_fingerprint: None,
                batch: vec![
                    json!({
                        "type": "scan",
                        "scan_id": "SCAN-1",
                        "signature": "scan-signature",
                        "previous_signature": "genesis",
                        "payload": {
                            "ctr": 7,
                            "timestamp_utc": 1770823456
                        },
                        "identity": {
                            "identity_version": 1,
                            "signature": "identity-signature"
                        }
                    }),
                    json!({
                        "type": "attachment",
                        "parent_record_id": "SCAN-1",
                        "signature": "attachment-signature",
                        "parent_signature": "scan-signature",
                        "checksum": "deadbeef",
                        "payload": {
                            "ctr": 8,
                            "timestamp_utc": 1770823460
                        }
                    }),
                ],
                batch_hash: "batch-hash".to_string(),
                block_canonical_string: "block-canonical-string".to_string(),
                block_hash: "block-hash".to_string(),
            }],
            HashMap::from([("deadbeef".to_string(), b"demo".to_vec())]),
            None,
        )
    }
}
