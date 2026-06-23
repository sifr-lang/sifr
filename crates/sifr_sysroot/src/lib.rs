//! Sysroot identity, fixed-layout paths, and resolver precedence.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

mod digest;
mod error;
mod layout;
mod manifest;
mod resolve;

pub use digest::{
    canonical_sysroot_tree_digest, CanonicalDigestPolicy, CanonicalTreeDigest,
    CanonicalTreeDigestEntry,
};
pub use error::{SysrootError, SysrootErrorKind};
pub use layout::{ResolvedSysroot, SysrootPaths};
pub use manifest::{
    parse_sysroot_manifest, SysrootManifest, COMPILER_SIFR_VERSION,
    SUPPORTED_SYSROOT_SCHEMA_VERSION, SYSROOT_MANIFEST_FIELDS,
};
pub use resolve::{
    discover_source_tree_root, is_source_tree_development_mode, resolve_sysroot,
    resolve_sysroot_with, set_process_sysroot_override, SysrootResolutionInput, SIFR_SYSROOT_ENV,
};

#[cfg(test)]
mod tests;
