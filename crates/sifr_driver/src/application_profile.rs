pub use sifr_compiler_services::ApplicationProfile;
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
