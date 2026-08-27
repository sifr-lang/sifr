use super::cli_model_and_entrypoint::diagnostic_with_code;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use sifr_format::config::EffectiveFormatConfig;
use sifr_frontend::{CompilerFingerprint, SourceProvider};
use std::fs;
use std::io;
use std::path::Path;

const FORMATTER_CACHE_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Deserialize, PartialEq, Eq, Serialize)]
struct FormatterCacheIdentity {
    schema_version: u32,
    canonical_path: String,
    source_sha256: String,
    final_newline: bool,
    line_length: u16,
    preview: bool,
    compiler_fingerprint: String,
}

pub(super) fn try_formatter_cache_hit(
    path: &Path,
    options: sifr_format::FormatOptions,
    config: &EffectiveFormatConfig,
    provider: &mut impl SourceProvider,
) -> Result<bool, Vec<RenderedDiagnostic>> {
    if config.no_cache {
        return Ok(false);
    }
    let identity = formatter_cache_identity(path, options, provider)?;
    let marker = config.cache_dir.join(identity_digest(&identity)?);
    Ok(read_identity(&marker).as_ref() == Some(&identity))
}

pub(super) fn write_formatter_cache_entry(
    path: &Path,
    options: sifr_format::FormatOptions,
    config: &EffectiveFormatConfig,
    provider: &mut impl SourceProvider,
) -> Result<(), Vec<RenderedDiagnostic>> {
    if config.no_cache {
        return Ok(());
    }
    let identity = formatter_cache_identity(path, options, provider)?;
    let digest = identity_digest(&identity)?;
    fs::create_dir_all(&config.cache_dir).map_err(|error| {
        vec![diagnostic(format!(
            "could not create formatter cache {}: {error}",
            config.cache_dir.display()
        ))]
    })?;
    let marker = config.cache_dir.join(digest);
    if marker.is_file() && read_identity(&marker).as_ref() != Some(&identity) {
        return Err(vec![diagnostic(format!(
            "formatter cache key collision at {}; stored full key does not match",
            marker.display()
        ))]);
    }
    let raw = serde_json::to_vec(&identity).map_err(|error| {
        vec![diagnostic(format!(
            "could not serialize formatter cache key: {error}"
        ))]
    })?;
    if read_identity(&marker).as_ref() == Some(&identity) {
        return Ok(());
    }
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = config.cache_dir.join(format!(
        ".formatter-cache-{}-{nonce}.tmp",
        std::process::id()
    ));
    fs::write(&temporary, raw).map_err(|error| {
        vec![diagnostic(format!(
            "could not write formatter cache {}: {error}",
            temporary.display()
        ))]
    })?;
    match fs::hard_link(&temporary, &marker) {
        Ok(()) => {
            let _ = fs::remove_file(temporary);
            Ok(())
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            let _ = fs::remove_file(temporary);
            if read_identity(&marker).as_ref() == Some(&identity) {
                Ok(())
            } else {
                Err(vec![diagnostic(format!(
                    "formatter cache key collision at {}; concurrent full key does not match",
                    marker.display()
                ))])
            }
        }
        Err(error) => {
            let _ = fs::remove_file(temporary);
            Err(vec![diagnostic(format!(
                "could not publish formatter cache {}: {error}",
                marker.display()
            ))])
        }
    }
}

fn formatter_cache_identity(
    path: &Path,
    options: sifr_format::FormatOptions,
    provider: &mut impl SourceProvider,
) -> Result<FormatterCacheIdentity, Vec<RenderedDiagnostic>> {
    let source = provider
        .read_file(path)
        .map(|source| source.as_str().to_string())
        .map_err(|error| {
            vec![diagnostic(format!(
                "could not read file {}: {error}",
                path.display()
            ))]
        })?;
    let canonical_path = provider
        .canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf());
    Ok(FormatterCacheIdentity {
        schema_version: FORMATTER_CACHE_SCHEMA_VERSION,
        canonical_path: canonical_path.display().to_string(),
        source_sha256: sha256_hex(source.as_bytes()),
        final_newline: options.final_newline,
        line_length: options.line_length,
        preview: options.preview,
        compiler_fingerprint: CompilerFingerprint::current().as_str().to_string(),
    })
}

fn identity_digest(identity: &FormatterCacheIdentity) -> Result<String, Vec<RenderedDiagnostic>> {
    serde_json::to_vec(identity)
        .map(|raw| sha256_hex(&raw))
        .map_err(|error| {
            vec![diagnostic(format!(
                "could not serialize formatter cache key: {error}"
            ))]
        })
}

fn read_identity(path: &Path) -> Option<FormatterCacheIdentity> {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

fn sha256_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn diagnostic(message: impl Into<String>) -> RenderedDiagnostic {
    diagnostic_with_code(message, DiagnosticCode::FMT_FORMATTING_DRIFT)
}

#[cfg(test)]
mod tests {
    use super::{FormatterCacheIdentity, identity_digest};

    #[test]
    fn formatter_cache_digest_is_sha256_and_content_sensitive() {
        let identity = FormatterCacheIdentity {
            schema_version: 2,
            canonical_path: "/repo/main.sifr".to_string(),
            source_sha256: "a".repeat(64),
            final_newline: true,
            line_length: 88,
            preview: false,
            compiler_fingerprint: "compiler-a".to_string(),
        };
        let digest = identity_digest(&identity).expect("identity should serialize");
        let mut changed = identity;
        changed.preview = true;

        assert_eq!(digest.len(), 64);
        assert_ne!(
            digest,
            identity_digest(&changed).expect("identity should serialize")
        );
    }
}
