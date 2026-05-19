#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageNameResolution {
    pub requested: String,
    pub cargo_package: String,
}
