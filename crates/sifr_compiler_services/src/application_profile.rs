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
