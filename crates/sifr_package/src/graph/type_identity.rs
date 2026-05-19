use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::graph::derive::SifrPackageId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageTypeIdentity {
    pub package_id: SifrPackageId,
    pub cargo_package_id: CargoPackageId,
    pub module_path: String,
    pub type_name: String,
    pub dependency_path: Vec<CargoPackageId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeIdentityMismatch {
    pub expected: PackageTypeIdentity,
    pub actual: PackageTypeIdentity,
}

impl TypeIdentityMismatch {
    #[must_use]
    pub fn diagnostic(&self) -> PackageDiagnostic {
        PackageDiagnostic::type_identity_mismatch(
            &self.actual.cargo_package_id,
            self.expected.display_key(),
            self.actual.display_key(),
        )
    }
}

impl PackageTypeIdentity {
    #[must_use]
    pub fn display_key(&self) -> String {
        let type_key = format!(
            "{} [{}]::{}.{}",
            self.package_id.0, self.cargo_package_id.0, self.module_path, self.type_name
        );
        if self.dependency_path.is_empty() {
            type_key
        } else {
            let path = self
                .dependency_path
                .iter()
                .map(|package_id| package_id.0.as_str())
                .collect::<Vec<_>>()
                .join(" -> ");
            format!("{type_key} via {path}")
        }
    }
}
