#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BackendTrustSummary {
    pub native_dependencies: Vec<String>,
    pub build_script_dependencies: Vec<String>,
    pub proc_macro_dependencies: Vec<String>,
}
