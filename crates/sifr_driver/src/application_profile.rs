//! Generated-application policy, independent of the compiler build profile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ApplicationProfile {
    #[default]
    Development,
    Test,
    Release,
}
impl ApplicationProfile {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Test => "test",
            Self::Release => "release",
        }
    }
    pub const fn cargo_name(self) -> &'static str {
        match self {
            Self::Development => "dev",
            Self::Test => "test",
            Self::Release => "release",
        }
    }
    pub const fn report_target(self) -> &'static str {
        match self {
            Self::Development => "development native",
            Self::Test => "test native",
            Self::Release => "release native",
        }
    }
    pub const fn build_stage(self) -> &'static str {
        match self {
            Self::Development => "Building development binary",
            Self::Test => "Building test binary",
            Self::Release => "Building release binary",
        }
    }
    pub const fn policy_identity(self) -> &'static str {
        "sifr-application-profiles-v1"
    }
    pub fn configure(self, command: &mut std::process::Command) {
        command.args(["--profile", self.cargo_name()]);
    }
}
/// All profiles are emitted together so Cargo retains compatible dependencies
/// when switching profiles in the same generated root.
pub(crate) const MANIFEST: &str = r#"
[profile.dev]
opt-level = 0
debug = 1
debug-assertions = true
overflow-checks = true
panic = "unwind"
incremental = true
lto = "off"
strip = "none"

[profile.test]
opt-level = 0
debug = 1
debug-assertions = true
overflow-checks = true
# Cargo test harnesses always unwind; panic is inherited from dev.
incremental = true
lto = "off"
strip = "none"

[profile.release]
opt-level = 3
debug = "line-tables-only"
debug-assertions = false
overflow-checks = true
panic = "unwind"
incremental = false
lto = "off"
strip = "none"
"#;
