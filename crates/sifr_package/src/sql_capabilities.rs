use crate::SifrManifest;
use sifr_sql_contract::PackageCapabilityResolver;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPackageCapabilities {
    package_identity: String,
    granted: BTreeSet<String>,
}

impl ResolvedPackageCapabilities {
    #[must_use]
    pub fn from_manifest(package_identity: impl Into<String>, manifest: &SifrManifest) -> Self {
        Self {
            package_identity: package_identity.into(),
            granted: manifest
                .trust
                .security_capabilities
                .iter()
                .cloned()
                .collect(),
        }
    }
}

impl PackageCapabilityResolver for ResolvedPackageCapabilities {
    fn allows(&self, package_identity: &str, capability: &str) -> bool {
        package_identity == self.package_identity && self.granted.contains(capability)
    }
}
