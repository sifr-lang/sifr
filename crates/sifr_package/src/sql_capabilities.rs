use crate::{SifrPackageGraph, SifrPackageId};
use sifr_sql_contract::{
    FragmentDraft, PackageCapabilityResolver, QueryContractError, SqlFragment, UnsafeSyntaxGrant,
    UnsafeSyntaxLint,
};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPackageCapabilities {
    package_identity: String,
    granted: BTreeSet<String>,
}

impl ResolvedPackageCapabilities {
    pub fn from_root_package(
        graph: &SifrPackageGraph,
        root: &SifrPackageId,
    ) -> Result<Self, PackageCapabilityResolutionError> {
        let package = graph
            .packages
            .get(root)
            .ok_or(PackageCapabilityResolutionError::UnknownRootPackage)?;
        if graph
            .cargo_edges
            .values()
            .any(|dependencies| dependencies.contains(root))
        {
            return Err(PackageCapabilityResolutionError::DependencyIsNotRoot);
        }
        Ok(Self {
            package_identity: root.0.clone(),
            granted: package
                .manifest
                .trust
                .security_capabilities
                .iter()
                .cloned()
                .collect(),
        })
    }

    pub fn unsafe_syntax_grant(
        &self,
        package_identity: &SifrPackageId,
        lint: UnsafeSyntaxLint,
        reason: impl Into<String>,
    ) -> Result<UnsafeSyntaxGrant, QueryContractError> {
        UnsafeSyntaxGrant::from_package_resolver(self, &package_identity.0, lint, reason)
    }

    pub fn compile_unsafe_fragment(
        &self,
        package_identity: &SifrPackageId,
        lint: UnsafeSyntaxLint,
        reason: impl Into<String>,
        draft: FragmentDraft,
    ) -> Result<SqlFragment, QueryContractError> {
        let grant = self.unsafe_syntax_grant(package_identity, lint, reason)?;
        SqlFragment::unsafe_checked(draft, &grant)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageCapabilityResolutionError {
    UnknownRootPackage,
    DependencyIsNotRoot,
}

impl PackageCapabilityResolver for ResolvedPackageCapabilities {
    fn allows(&self, package_identity: &str, capability: &str) -> bool {
        package_identity == self.package_identity && self.granted.contains(capability)
    }
}
