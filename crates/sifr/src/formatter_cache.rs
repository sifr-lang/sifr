//! Formatter success markers are advisory and tied to exact compiler and input identities.

use super::check_and_package_commands::formatter_cli_diagnostic;
use sifr_diagnostics::RenderedDiagnostic;
use sifr_format::config::EffectiveFormatConfig;
use sifr_format::{DocstringCodeLineLength, FormatOptions};
use sifr_frontend::SourceProvider;
use sifr_identity::IdentityEncoder;
use std::fs;
use std::io;
use std::path::Path;

const MARKER_OWNER: &str = "sifr-formatter-cache";
const MARKER_SCHEMA: u32 = 1;

pub(super) fn try_formatter_cache_hit(
    path: &Path,
    options: FormatOptions,
    config: &EffectiveFormatConfig,
    provider: &mut impl SourceProvider,
) -> Result<bool, Vec<RenderedDiagnostic>> {
    if config.no_cache {
        return Ok(false);
    }
    let source = read_formatter_source(path, provider)?;
    let identity =
        formatter_cache_identity(path, &source, options, crate::compiler_identity().as_str())?;
    let marker = config.cache_dir.join(&identity);
    let metadata = match fs::symlink_metadata(&marker) {
        Ok(metadata) if metadata.file_type().is_file() => metadata,
        Ok(_) => return Ok(false),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(_) => return Ok(false),
    };
    if metadata.len() != marker_contents(&identity).len() as u64 {
        return Ok(false);
    }
    Ok(fs::read(&marker).is_ok_and(|bytes| bytes == marker_contents(&identity)))
}

pub(super) fn write_formatter_cache_entry(
    path: &Path,
    options: FormatOptions,
    config: &EffectiveFormatConfig,
    provider: &mut impl SourceProvider,
) -> Result<(), Vec<RenderedDiagnostic>> {
    if config.no_cache {
        return Ok(());
    }
    let source = read_formatter_source(path, provider)?;
    let identity =
        formatter_cache_identity(path, &source, options, crate::compiler_identity().as_str())?;
    fs::create_dir_all(&config.cache_dir)
        .map_err(|error| cache_error("create", &config.cache_dir, error))?;
    let mut staging = tempfile::NamedTempFile::new_in(&config.cache_dir)
        .map_err(|error| cache_error("stage", &config.cache_dir, error))?;
    use io::Write as _;
    staging
        .write_all(&marker_contents(&identity))
        .and_then(|()| staging.as_file().sync_all())
        .map_err(|error| cache_error("write", &config.cache_dir, error))?;
    let destination = config.cache_dir.join(&identity);
    staging
        .persist(&destination)
        .map_err(|error| cache_error("publish", &destination, error.error))?;
    Ok(())
}

pub(super) fn read_formatter_source(
    path: &Path,
    provider: &mut impl SourceProvider,
) -> Result<String, Vec<RenderedDiagnostic>> {
    provider
        .read_file(path)
        .map(|source| source.as_str().to_string())
        .map_err(|error| {
            vec![formatter_cli_diagnostic(format!(
                "could not read file {}: {error}",
                path.display()
            ))]
        })
}

fn formatter_cache_identity(
    path: &Path,
    source: &str,
    options: FormatOptions,
    implementation: &str,
) -> Result<String, Vec<RenderedDiagnostic>> {
    let normalized = fs::canonicalize(path).map_err(|error| {
        vec![formatter_cli_diagnostic(format!(
            "could not resolve formatter path {}: {error}",
            path.display()
        ))]
    })?;
    let mut identity = IdentityEncoder::new("sifr-formatter-success-v1");
    identity.field("implementation", implementation.as_bytes());
    identity.field("path", normalized.as_os_str().as_encoded_bytes());
    identity.field("source", source.as_bytes());
    identity.field("final_newline", &[u8::from(options.final_newline)]);
    identity.field("line_length", &options.line_length.to_le_bytes());
    identity.field("preview", &[u8::from(options.preview)]);
    identity.field(
        "docstring_code_format",
        &[u8::from(options.docstring_code_format)],
    );
    match options.docstring_code_line_length {
        DocstringCodeLineLength::Dynamic => {
            identity.field("docstring_code_line_length", b"dynamic")
        }
        DocstringCodeLineLength::Fixed(width) => {
            identity.field("docstring_code_line_length", b"fixed");
            identity.field("docstring_code_line_length_value", &width.to_le_bytes());
        }
    }
    Ok(identity.finish())
}

fn marker_contents(identity: &str) -> Vec<u8> {
    format!("{MARKER_OWNER}\n{MARKER_SCHEMA}\n{identity}\n").into_bytes()
}

fn cache_error(action: &str, path: &Path, error: io::Error) -> Vec<RenderedDiagnostic> {
    vec![formatter_cli_diagnostic(format!(
        "could not {action} formatter cache {}: {error}",
        path.display()
    ))]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_binds_revision_options_source_and_normalized_path() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("main.sifr");
        let other = root.path().join("other.sifr");
        fs::write(&file, "x = 1\n").unwrap();
        fs::write(&other, "x = 1\n").unwrap();
        let options = FormatOptions::default();
        let base = formatter_cache_identity(&file, "x = 1\n", options, "revision-a").unwrap();
        assert_eq!(
            base,
            formatter_cache_identity(
                &root.path().join("./main.sifr"),
                "x = 1\n",
                options,
                "revision-a"
            )
            .unwrap()
        );
        assert_ne!(
            base,
            formatter_cache_identity(&file, "x = 1\n", options, "revision-b").unwrap()
        );
        assert_ne!(
            base,
            formatter_cache_identity(&file, "x = 2\n", options, "revision-a").unwrap()
        );
        assert_ne!(
            base,
            formatter_cache_identity(&other, "x = 1\n", options, "revision-a").unwrap()
        );
        for changed in [
            FormatOptions {
                final_newline: false,
                ..options
            },
            FormatOptions {
                line_length: 79,
                ..options
            },
            FormatOptions {
                preview: true,
                ..options
            },
            FormatOptions {
                docstring_code_format: true,
                ..options
            },
            FormatOptions {
                docstring_code_line_length: DocstringCodeLineLength::Fixed(72),
                ..options
            },
        ] {
            assert_ne!(
                base,
                formatter_cache_identity(&file, "x = 1\n", changed, "revision-a").unwrap()
            );
        }
    }

    #[test]
    fn invalid_markers_do_not_authorize_a_hit() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("main.sifr");
        fs::write(&file, "x = 1\n").unwrap();
        let config = EffectiveFormatConfig {
            cache_dir: root.path().join("cache"),
            ..EffectiveFormatConfig::default()
        };
        let mut provider = sifr_frontend::DiskSourceProvider::new();
        let options = FormatOptions::default();
        write_formatter_cache_entry(&file, options, &config, &mut provider).unwrap();
        assert!(try_formatter_cache_hit(&file, options, &config, &mut provider).unwrap());
        let identity = formatter_cache_identity(
            &file,
            "x = 1\n",
            options,
            crate::compiler_identity().as_str(),
        )
        .unwrap();
        let marker = config.cache_dir.join(&identity);
        for invalid in [
            b"ok".as_slice(),
            b"sifr-formatter-cache\n0\n",
            b"other-owner\n1\n",
        ] {
            fs::write(&marker, invalid).unwrap();
            assert!(!try_formatter_cache_hit(&file, options, &config, &mut provider).unwrap());
        }
        let valid = marker_contents(&identity);
        for index in [0, MARKER_OWNER.len() + 1, valid.len() - 2] {
            let mut invalid = valid.clone();
            invalid[index] = if invalid[index] == b'0' { b'1' } else { b'0' };
            fs::write(&marker, &invalid).unwrap();
            assert!(!try_formatter_cache_hit(&file, options, &config, &mut provider).unwrap());
        }
        fs::remove_file(&marker).unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&file, &marker).unwrap();
            assert!(!try_formatter_cache_hit(&file, options, &config, &mut provider).unwrap());
        }
    }

    #[test]
    fn unchanged_source_with_old_formatter_revision_is_a_miss() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("main.sifr");
        fs::write(&file, "x = 1\n").unwrap();
        let config = EffectiveFormatConfig {
            cache_dir: root.path().join("cache"),
            ..EffectiveFormatConfig::default()
        };
        fs::create_dir(&config.cache_dir).unwrap();
        let options = FormatOptions::default();
        let old_identity =
            formatter_cache_identity(&file, "x = 1\n", options, "previous-revision").unwrap();
        let old_marker = config.cache_dir.join(&old_identity);
        fs::write(&old_marker, marker_contents(&old_identity)).unwrap();
        let mut provider = sifr_frontend::DiskSourceProvider::new();
        assert!(!try_formatter_cache_hit(&file, options, &config, &mut provider).unwrap());
    }

    #[test]
    fn incomplete_staging_file_does_not_authorize_a_hit() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("main.sifr");
        fs::write(&file, "x = 1\n").unwrap();
        let config = EffectiveFormatConfig {
            cache_dir: root.path().join("cache"),
            ..EffectiveFormatConfig::default()
        };
        fs::create_dir(&config.cache_dir).unwrap();
        let options = FormatOptions::default();
        let identity = formatter_cache_identity(
            &file,
            "x = 1\n",
            options,
            crate::compiler_identity().as_str(),
        )
        .unwrap();
        fs::write(
            config.cache_dir.join(".partial-stage"),
            b"sifr-formatter-cache\n1\n",
        )
        .unwrap();
        let mut provider = sifr_frontend::DiskSourceProvider::new();
        assert!(!try_formatter_cache_hit(&file, options, &config, &mut provider).unwrap());
        assert!(!config.cache_dir.join(identity).exists());
    }

    #[test]
    fn publication_failure_leaves_no_success_marker() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("main.sifr");
        let cache = root.path().join("cache");
        fs::write(&file, "x = 1\n").unwrap();
        fs::write(&cache, "occupied").unwrap();
        let config = EffectiveFormatConfig {
            cache_dir: cache.clone(),
            ..EffectiveFormatConfig::default()
        };
        let mut provider = sifr_frontend::DiskSourceProvider::new();
        assert!(
            write_formatter_cache_entry(&file, FormatOptions::default(), &config, &mut provider)
                .is_err()
        );
        assert_eq!(fs::read(&cache).unwrap(), b"occupied");
    }
}
