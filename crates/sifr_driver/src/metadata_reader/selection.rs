use super::{Provider, Result, wire};
use sifr_identity::{CompilerIdentity, TargetSemanticId};
use sifr_sysroot::{ResolvedSysroot, SysrootMode};
use std::{
    io::Read,
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Descriptor {
    schema_version: u32,
    compiler_identity: String,
    semantic_target: String,
    semantic_target_id: String,
    stdlib_inputs_id: String,
    metadata_id: String,
    compiler_binary_sha256: String,
}
fn digest(text: &str) -> Result<[u8; 32]> {
    if text.len() != 64
        || !text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(wire::MetadataError(
            "invalid metadata descriptor identity".into(),
        ));
    }
    let mut result = [0; 32];
    for (i, byte) in result.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[2 * i..2 * i + 2], 16)
            .map_err(|e| wire::MetadataError(e.to_string()))?;
    }
    Ok(result)
}
pub(crate) fn target() -> &'static str {
    if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "aarch64-apple-darwin"
        } else {
            "x86_64-apple-darwin"
        }
    } else if cfg!(target_arch = "aarch64") {
        "aarch64-unknown-linux-gnu"
    } else {
        "x86_64-unknown-linux-gnu"
    }
}
pub(crate) fn select(
    identity: &CompilerIdentity,
    root: &ResolvedSysroot,
    override_path: Option<&Path>,
) -> Result<Arc<Provider>> {
    let prepared = if root.mode() == SysrootMode::SourceTreeDevelopment {
        let prepared = match override_path {
            Some(path) => crate::metadata_producer::validate_development_metadata(
                identity,
                &root.root,
                target(),
                path,
            )?,
            None => crate::metadata_producer::ensure_development_metadata(
                identity,
                &root.root,
                target(),
                &crate::cache_storage::root(),
                &AtomicBool::new(false),
            )?,
        };
        crate::metadata_producer::open_consumer(
            &prepared.path,
            prepared.compatibility,
            Some(&prepared.metadata_id),
        )?
    } else {
        let descriptor_path = root.root.join("lib/sifr/stdlib.metadata.json");
        let descriptor = std::fs::File::open(&descriptor_path).map_err(|e| {
            wire::MetadataError(format!(
                "required metadata descriptor {}: {e}; reinstall the selected toolchain",
                descriptor_path.display()
            ))
        })?;
        let mut bytes = Vec::new();
        descriptor
            .take(8193)
            .read_to_end(&mut bytes)
            .map_err(|e| wire::MetadataError(e.to_string()))?;
        if bytes.len() > 8192 {
            return Err(wire::MetadataError(
                "metadata descriptor exceeds size limit".into(),
            ));
        }
        let descriptor: Descriptor = serde_json::from_slice(&bytes).map_err(|e| {
            wire::MetadataError(format!("invalid installed metadata descriptor: {e}"))
        })?;
        let target_id = TargetSemanticId::from_records([
            ("triple", target().as_bytes()),
            ("layout", b"pointer64-little-endian-v1".as_slice()),
        ]);
        if descriptor.schema_version != 1
            || descriptor.compiler_identity != identity.as_str()
            || descriptor.semantic_target != target()
            || descriptor.semantic_target_id != target_id.as_str()
        {
            return Err(wire::MetadataError("installed metadata is incompatible with this compiler or target; reinstall the selected toolchain".into()));
        }
        digest(&descriptor.compiler_binary_sha256)?;
        digest(&descriptor.metadata_id)?;
        let expected = wire::Compatibility {
            compiler: digest(&descriptor.compiler_identity)?,
            semantic_target: digest(&descriptor.semantic_target_id)?,
            stdlib_inputs: digest(&descriptor.stdlib_inputs_id)?,
        };
        let path = root.root.join("lib/sifr/stdlib.sifrmeta");
        crate::metadata_producer::open_consumer(
            override_path.unwrap_or(&path),
            expected,
            Some(&descriptor.metadata_id),
        )?
    };
    Provider::new(prepared)
}
