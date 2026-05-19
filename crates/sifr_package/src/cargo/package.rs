#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CargoPackageRole {
    SifrSource,
    BackendRust,
    RustBackedSifr,
}
