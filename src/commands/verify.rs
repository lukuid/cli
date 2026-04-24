// SPDX-License-Identifier: Apache-2.0
use std::collections::BTreeMap;
use std::path::Path;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use lukuid_sdk::{Criticality, LukuFile, LukuVerifyOptions, VerificationIssue};

pub fn run_verify(
    path: &Path,
    allow_untrusted_roots: bool,
    skip_certificate_temporal_checks: bool,
    require_continuity: bool,
    trusted_external_fingerprints: Vec<String>,
    json_mode: bool,
    run_test: bool,
) -> Result<i32> {
    let mut self_test_results = None;
    if run_test {
        if !json_mode {
            let _ = super::test::run_test(false);
            println!();
        }
        self_test_results = Some(lukuid_sdk::LukuidSdk::self_test());
    }

    let luku = LukuFile::open(path).map_err(|e| anyhow!("{e}"))?;
    let options = LukuVerifyOptions {
        allow_untrusted_roots,
        skip_certificate_temporal_checks,
        require_continuity,
        trusted_external_fingerprints,
        ..Default::default()
    };
    let issues = luku.verify(options.clone());
    let mut output = verification_output(&luku, path, &issues, &options.trust_profile);
    
    if let Some(st_results) = self_test_results {
        if let Value::Object(ref mut map) = output {
            map.insert("self_test".to_string(), json!(st_results));
        }
    }
    
    crate::output::print_output(&output, json_mode)?;
    
    if issues.iter().any(|i| i.criticality == Criticality::Critical) {
        Ok(2)
    } else {
        Ok(0)
    }
}

fn verification_output(luku: &LukuFile, path: &Path, issues: &[VerificationIssue], trust_profile: &str) -> Value {
    let mut counts: BTreeMap<&'static str, usize> =
        BTreeMap::from([("critical", 0), ("warning", 0), ("info", 0)]);
    for issue in issues {
        match issue.criticality {
            Criticality::Critical => *counts.get_mut("critical").expect("critical key") += 1,
            Criticality::Warning => *counts.get_mut("warning").expect("warning key") += 1,
            Criticality::Info => *counts.get_mut("info").expect("info key") += 1,
        }
    }

    let mut status = if counts["critical"] > 0 {
        "invalid".to_string()
    } else if counts["warning"] > 0 {
        "verified_with_warnings".to_string()
    } else {
        "verified".to_string()
    };

    if trust_profile != "prod" {
        status.push_str(" (development profile)");
    }

    let mut text = String::new();
    text.push_str(&format!("Archive: {}\n", path.display()));
    text.push_str(&format!(
        "Verification status: {status} (critical={} warning={} info={})\n",
        counts["critical"], counts["warning"], counts["info"]
    ));
    text.push_str(&format!(
        "Blocks: {} | Records: {} | Attachments: {}\n",
        luku.blocks.len(),
        luku.blocks
            .iter()
            .map(|block| block.batch.len())
            .sum::<usize>(),
        luku.attachments.len()
    ));
    if issues.is_empty() {
        text.push_str("No verification issues detected.\n");
    } else {
        text.push_str("Issues:\n");
        for issue in issues {
            text.push_str(&format!(
                "  - [{}] {}: {}\n",
                criticality_label(&issue.criticality),
                issue.code,
                issue.message
            ));
        }
    }

    json!({
        "path": path.display().to_string(),
        "status": status,
        "counts": counts,
        "issues": issues,
        "text": text,
    })
}

fn criticality_label(level: &Criticality) -> &'static str {
    match level {
        Criticality::Critical => "critical",
        Criticality::Warning => "warning",
        Criticality::Info => "info",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::inspect::tests::sample_luku_file;

    #[test]
    fn verification_output_marks_invalid_when_critical_issues_exist() {
        let luku = sample_luku_file();
        let issues = vec![
            VerificationIssue {
                code: "ATTACHMENT_CORRUPT".to_string(),
                message: "Attachment hash mismatch".to_string(),
                criticality: Criticality::Critical,
            },
            VerificationIssue {
                code: "ATTESTATION_CHAIN_MISSING".to_string(),
                message: "Missing DAC chain".to_string(),
                criticality: Criticality::Warning,
            },
        ];

        let output = verification_output(&luku, Path::new("fixtures/demo.luku"), &issues, "prod");
        let text = output["text"].as_str().expect("text output");

        assert_eq!(output["status"], json!("invalid"));
        assert_eq!(output["counts"]["critical"], json!(1));
        assert_eq!(output["counts"]["warning"], json!(1));
        assert!(text.contains("Verification status: invalid (critical=1 warning=1 info=0)"));
        assert!(text.contains("[critical] ATTACHMENT_CORRUPT: Attachment hash mismatch"));
        assert!(text.contains("[warning] ATTESTATION_CHAIN_MISSING: Missing DAC chain"));
    }
}
