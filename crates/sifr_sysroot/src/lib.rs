//! Sysroot identity, fixed-layout paths, and resolver precedence.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

mod digest;
mod error;
mod layout;
mod manifest;
mod resolve;

pub use digest::{
    CanonicalDigestPolicy, CanonicalTreeDigest, CanonicalTreeDigestEntry,
    canonical_sysroot_tree_digest, sha256_file, sha256_hex,
};
pub use error::{SysrootError, SysrootErrorKind};
pub use layout::{ResolvedSysroot, SysrootPaths};
pub use manifest::{
    COMPILER_SIFR_VERSION, SUPPORTED_SYSROOT_SCHEMA_VERSION, SYSROOT_MANIFEST_FIELDS,
    SysrootManifest, parse_sysroot_manifest,
};
pub use resolve::{
    SIFR_SYSROOT_ENV, SysrootResolutionInput, discover_source_tree_root,
    is_source_tree_development_mode, resolve_sysroot, resolve_sysroot_with,
    set_process_sysroot_override,
};

#[cfg(test)]
mod tests;
