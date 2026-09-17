use super::{Result, wire};
use sifr_identity::{CompilerIdentity, IdentityEncoder, TargetSemanticId};
use sifr_stdlib_manifest::{LoadedStdlibSource, load_stdlib_tooling_sources_from_sysroot};
use std::path::Path;

pub(super) struct Inputs {
    pub sysroot: sifr_sysroot::ResolvedSysroot,
    pub sources: Vec<LoadedStdlibSource>,
    pub compatibility: wire::Compatibility,
}
fn fail(error: impl std::fmt::Display) -> wire::MetadataError {
    wire::MetadataError(error.to_string())
}
fn digest(text: &str) -> Result<[u8; 32]> {
    if text.len() != 64 {
        return Err(fail("expected a 32-byte compiler identity"));
    }
    let mut value = [0; 32];
    for (i, byte) in value.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).map_err(fail)?;
    }
    Ok(value)
}
impl Inputs {
    pub(super) fn capture(
        identity: &CompilerIdentity,
        source_root: &Path,
        target: &str,
    ) -> Result<Self> {
        if !matches!(
            target,
            "x86_64-unknown-linux-gnu"
                | "aarch64-unknown-linux-gnu"
                | "x86_64-apple-darwin"
                | "aarch64-apple-darwin"
        ) {
            return Err(fail(format!(
                "unsupported metadata semantic target: {target}"
            )));
        }
        let source_root = source_root.canonicalize().map_err(fail)?;
        let sysroot = sifr_sysroot::resolve_sysroot(Some(source_root))
            .map_err(|error| fail(error.boundary_message()))?;
        if sysroot.mode() != sifr_sysroot::SysrootMode::SourceTreeDevelopment {
            return Err(fail(
                "metadata production requires an explicit source-development snapshot",
            ));
        }
        let sources = load_stdlib_tooling_sources_from_sysroot(&sysroot).map_err(fail)?;
        let semantic = TargetSemanticId::from_records([
            ("triple", target.as_bytes()),
            ("layout", b"pointer64-little-endian-v1".as_slice()),
        ]);
        let mut inputs = IdentityEncoder::new("stdlib-inputs-v1");
        inputs.field("semantic-target", semantic.as_str().as_bytes());
        inputs.field("producer-policy", b"checked-source-indexed-v1");
        // These captured source tokens describe portable semantic producer inputs.
        // Host/profile/rustc configuration remains in the required compiler envelope.
        for (name, token) in [
            (
                "sifr_stdlib_manifest",
                sifr_stdlib_manifest::portable_source_token(),
            ),
            (
                "sifr_stdlib_imports",
                sifr_stdlib_imports::portable_source_token(),
            ),
            ("sifr_ir", sifr_ir::portable_source_token()),
        ] {
            inputs.field(name, token.as_bytes());
        }
        for source in &sources {
            inputs.field("module", source.module.as_bytes());
            inputs.field(
                "kind",
                match source.kind {
                    sifr_stdlib_manifest::LoadedStdlibSourceKind::Public => b"public",
                    sifr_stdlib_manifest::LoadedStdlibSourceKind::PrivateDeclaration => {
                        b"private-declaration"
                    }
                },
            );
            inputs.field(
                "path",
                source
                    .path
                    .strip_prefix(&sysroot.paths.stdlib_root)
                    .map_err(fail)?
                    .as_os_str()
                    .as_encoded_bytes(),
            );
            inputs.field("source", source.source.as_bytes());
        }
        for name in ["sysroot.toml", "Cargo.lock"] {
            inputs.field(name, &std::fs::read(sysroot.root.join(name)).map_err(fail)?);
        }
        let compatibility = wire::Compatibility {
            compiler: digest(identity.as_str())?,
            semantic_target: digest(semantic.as_str())?,
            stdlib_inputs: digest(&inputs.finish())?,
        };
        Ok(Self {
            sysroot,
            sources,
            compatibility,
        })
    }
    pub(super) fn produce(&self) -> Result<Vec<u8>> {
        let compiled =
            crate::stdlib::compile_stdlib_sources_with_sysroot(&self.sources, self.sysroot.clone())
                .map_err(|errors| {
                    fail(
                        errors
                            .iter()
                            .map(|e| e.message.as_str())
                            .collect::<Vec<_>>()
                            .join("\n"),
                    )
                })?;
        super::project::project(
            &compiled,
            &self.sources,
            self.compatibility,
            &self.sysroot.paths.stdlib_root,
        )
    }
}
