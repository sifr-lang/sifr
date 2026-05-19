#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImportBoundary {
    OwnPackage,
    DirectDependency,
    ReExport,
}
