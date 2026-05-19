#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReExportPolicy {
    pub wildcard_reexports_allowed: bool,
}
