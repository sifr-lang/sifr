//! One Cargo storage context for probes and native consumers of the same inputs.
//!
//! This is artifact storage, never an authorization or successful-result cache.
//! Every caller still validates request trust, prepares its own resolution, runs
//! Cargo and validates its output. Private project roots and the family lease
//! protect generated inputs and capture even when root executable names coincide.
use super::cargo_resolution::CargoResolutionPolicy;
use super::native_storage::NativeFamily;
use sifr_identity::IdentityEncoder;
use sifr_stdlib_manifest::CargoVendorMode;
use std::path::Path;

impl CargoResolutionPolicy {
    pub(crate) fn native_family(
        &self,
        runtime_crate: &Path,
        vendor_mode: CargoVendorMode,
        cargo_prefix_args: &[String],
        environment: &str,
    ) -> std::io::Result<NativeFamily> {
        let tools = self
            .native_toolchain
            .as_ref()
            .map_err(|error| std::io::Error::other(error.clone()))?;
        NativeFamily::acquire(
            tools.identity(),
            &self.native_family_sources(runtime_crate, vendor_mode, cargo_prefix_args),
            environment,
            // Declaration-specific trust belongs to request validation and the
            // finalized artifact identity. It cannot authorize Cargo reuse.
            "validated-cargo-request-v1",
        )
    }

    fn native_family_sources(
        &self,
        runtime_crate: &Path,
        vendor_mode: CargoVendorMode,
        cargo_prefix_args: &[String],
    ) -> String {
        let mut id = IdentityEncoder::new("native-cargo-sources-v2");
        id.field(
            "runtime-crate",
            runtime_crate.as_os_str().as_encoded_bytes(),
        );
        id.field("vendor-mode", format!("{vendor_mode:?}").as_bytes());
        id.field(
            "resolution-vendor-mode",
            format!("{:?}", self.cargo_vendor_mode).as_bytes(),
        );
        id.field("lock-mode", self.lock_mode.as_str().as_bytes());
        id.field(
            "profile",
            self.application_profile.policy_identity().as_bytes(),
        );
        id.field("profile-name", self.application_profile.name().as_bytes());
        for argument in cargo_prefix_args {
            id.field("cargo-prefix-argument", argument.as_bytes());
        }
        // Keep source replacement boundaries: different vendor roots must
        // not thrash each other's Cargo artifacts.
        for path in &self.trusted_vendor_dirs {
            id.field("trusted-vendor", path.as_os_str().as_encoded_bytes());
        }
        // Lock authorities and normal-mode seeds belong to each generated
        // project's resolution, not dependency artifact storage. Every caller
        // prepares and validates its own lock before Cargo runs. Cargo checks
        // the selected units even when another request prepared this target.
        id.finish()
    }
}

#[cfg(test)]
#[path = "cargo_native_family_tests.rs"]
mod tests;
