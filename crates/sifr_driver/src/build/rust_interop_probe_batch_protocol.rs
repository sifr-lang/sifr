//! Cargo JSON is the per-crate observation protocol, not just its exit status.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(super) fn outcomes(
    stdout: &[u8],
    stderr: &[u8],
    success: bool,
    members: &[PathBuf],
) -> Result<Vec<Result<(), String>>, String> {
    let expected = members.iter().cloned().collect::<BTreeSet<_>>();
    let mut artifacts = BTreeSet::new();
    let mut errors: BTreeMap<PathBuf, String> = BTreeMap::new();
    let mut shared_errors = String::new();
    let mut finished = None;
    let cargo_stderr = String::from_utf8_lossy(stderr);
    // Cargo's stream also carries arbitrary procedural-macro stdout. Only
    // recognized Cargo records supply evidence; text can never authorize a probe.
    // https://doc.rust-lang.org/cargo/reference/external-tools.html#json-messages
    for line in stdout.split(|byte| *byte == b'\n') {
        let Ok(value) = serde_json::from_slice::<serde_json::Value>(line) else {
            continue;
        };
        if !matches!(
            value["reason"].as_str(),
            Some(
                "build-finished"
                    | "compiler-artifact"
                    | "compiler-message"
                    | "build-script-executed"
            )
        ) {
            continue;
        }
        if finished.is_some() {
            return Err("Rust probe Cargo output continues after completion".into());
        }
        match value["reason"].as_str() {
            Some("build-finished") => {
                let result = value["success"]
                    .as_bool()
                    .ok_or("Rust probe completion has no status")?;
                finished = Some(result);
            }
            Some("compiler-artifact") => {
                if let Some(source) = value["target"]["src_path"].as_str() {
                    let source = Path::new(source);
                    if expected.contains(source) {
                        artifacts.insert(source.to_path_buf());
                    }
                }
            }
            Some("compiler-message") if value["message"]["level"] == "error" => {
                let rendered = value["message"]["rendered"]
                    .as_str()
                    .ok_or("Rust probe error has no rendered diagnostic")?;
                let source = value["target"]["src_path"].as_str().map(Path::new);
                if let Some(source) = source.filter(|p| expected.contains(*p)) {
                    errors
                        .entry(source.to_path_buf())
                        .or_default()
                        .push_str(rendered);
                } else {
                    shared_errors.push_str(rendered);
                }
            }
            Some("compiler-message" | "build-script-executed") => {}
            _ => return Err("Rust probe Cargo output has an unknown record".into()),
        }
    }
    if finished != Some(success) {
        // Cargo resolution can fail before emitting build-finished. Preserve its
        // diagnostic as a failure for every unobserved member, never a pass.
        if !success && finished.is_none() && artifacts.is_empty() && errors.is_empty() {
            let message = cargo_stderr.to_string();
            if !message.trim().is_empty() {
                return Ok(members.iter().map(|_| Err(message.clone())).collect());
            }
        }
        return Err("Rust probe Cargo completion is missing or inconsistent".into());
    }
    if success && (!errors.is_empty() || !shared_errors.is_empty()) {
        return Err("Rust probe Cargo success contradicts error diagnostics".into());
    }
    let results = members
        .iter()
        .map(|member| {
            if let Some(error) = errors.get(member) {
                Ok(Err(error.clone()))
            } else if artifacts.contains(member) {
                Ok(Ok(()))
            } else if !shared_errors.is_empty() {
                Ok(Err(shared_errors.clone()))
            } else if !success && !cargo_stderr.trim().is_empty() {
                // Cargo reports runtime build-script failures only on stderr.
                // Preserve that failure for each unobserved dependent probe.
                Ok(Err(cargo_stderr.to_string()))
            } else {
                Err(format!(
                    "Rust probe Cargo did not observe {}",
                    member.display()
                ))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    if !success && results.iter().all(Result::is_ok) {
        return Err("Rust probe Cargo failed without an attributed failure".into());
    }
    Ok(results)
}
