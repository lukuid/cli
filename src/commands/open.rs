// SPDX-License-Identifier: Apache-2.0
use std::path::Path;
use anyhow::{Context, Result};
use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};
use lukuid_sdk::LukuFile;
use crate::output::human::{primary_record_id, shorten};
use tempfile::tempdir;

#[derive(Clone, Debug, PartialEq)]
pub enum MenuItem {
    Exit,
    Record { block_idx: usize, record_idx: usize, display: String },
    Attachment { hash: String, display: String },
}

impl std::fmt::Display for MenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuItem::Exit => write!(f, "[Exit]"),
            MenuItem::Record { display, .. } => write!(f, "{}", display),
            MenuItem::Attachment { display, .. } => write!(f, "{}", display),
        }
    }
}

pub fn build_menu_items(luku: &LukuFile) -> Vec<MenuItem> {
    let mut items = vec![MenuItem::Exit];
    
    for (b_idx, block) in luku.blocks.iter().enumerate() {
        for (r_idx, record) in block.batch.iter().enumerate() {
            let r_type = record.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
            let id = primary_record_id(record);
            items.push(MenuItem::Record {
                block_idx: b_idx,
                record_idx: r_idx,
                display: format!("Record: {} ({})", r_type, id),
            });
        }
    }

    let mut hashes: Vec<_> = luku.attachments.keys().collect();
    hashes.sort();
    for hash in hashes {
        items.push(MenuItem::Attachment {
            hash: hash.clone(),
            display: format!("Attachment: {}", shorten(hash, 16)),
        });
    }

    items
}

pub fn format_record_view(luku: &LukuFile, block_idx: usize, record_idx: usize) -> Result<String> {
    let block = luku.blocks.get(block_idx).context("Block not found")?;
    let record = block.batch.get(record_idx).context("Record not found")?;
    let pretty = serde_json::to_string_pretty(record)?;
    Ok(format!("Block: {}\nRecord Index: {}\n\n{}", block.block_id, record_idx, pretty))
}

pub fn extract_attachment(luku: &LukuFile, hash: &str, out_dir: &Path) -> Result<std::path::PathBuf> {
    let data = luku.attachments.get(hash).context("Attachment not found")?;
    let ext = if data.starts_with(b"\x89PNG\r\n\x1a\n") {
        "png"
    } else if data.starts_with(b"%PDF-") {
        "pdf"
    } else if data.starts_with(b"\xFF\xD8\xFF") {
        "jpg"
    } else {
        "bin"
    };
    let file_path = out_dir.join(format!("{}.{}", hash, ext));
    std::fs::write(&file_path, data)?;
    Ok(file_path)
}

pub fn run_open(path: &Path) -> Result<i32> {
    let luku = LukuFile::open(path).map_err(|e| anyhow::anyhow!("{e}"))?;
    let items = build_menu_items(&luku);
    let temp_dir = tempdir()?;

    let term = Term::stdout();

    loop {
        term.clear_screen()?;
        println!("Archive: {}\n", path.display());

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an item to view")
            .default(0)
            .items(&items)
            .interact_on_opt(&term)?;

        match selection {
            Some(idx) => {
                let item = &items[idx];
                match item {
                    MenuItem::Exit => break,
                    MenuItem::Record { block_idx, record_idx, .. } => {
                        term.clear_screen()?;
                        let view = format_record_view(&luku, *block_idx, *record_idx)?;
                        println!("{}\n", view);
                        println!("Press Enter to go back...");
                        term.read_line()?;
                    }
                    MenuItem::Attachment { hash, .. } => {
                        let file_path = extract_attachment(&luku, hash, temp_dir.path())?;
                        println!("Opening attachment: {}...", file_path.display());
                        if let Err(e) = open::that(&file_path) {
                            println!("Failed to open attachment: {}", e);
                        }
                        println!("Press Enter to go back...");
                        term.read_line()?;
                    }
                }
            }
            None => break,
        }
    }

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::inspect::tests::sample_luku_file;
    use tempfile::tempdir;

    #[test]
    fn test_build_menu_items() {
        let luku = sample_luku_file();
        let items = build_menu_items(&luku);
        assert_eq!(items.len(), 4);
        assert!(matches!(items[0], MenuItem::Exit));
        assert!(matches!(items[1], MenuItem::Record { block_idx: 0, record_idx: 0, .. }));
        assert!(matches!(items[2], MenuItem::Record { block_idx: 0, record_idx: 1, .. }));
        assert!(matches!(items[3], MenuItem::Attachment { ref hash, .. } if hash == "deadbeef"));
    }

    #[test]
    fn test_format_record_view() {
        let luku = sample_luku_file();
        let view = format_record_view(&luku, 0, 0).unwrap();
        assert!(view.contains("SCAN-1"));
    }

    #[test]
    fn test_extract_attachment() {
        let luku = sample_luku_file();
        let dir = tempdir().unwrap();
        let path = extract_attachment(&luku, "deadbeef", dir.path()).unwrap();
        assert!(path.exists());
        let data = std::fs::read(&path).unwrap();
        assert_eq!(data, b"demo");
    }
}
