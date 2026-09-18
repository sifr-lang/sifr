use super::cargo_resolution::CargoResolutionPolicy;
use super::materialize::materialize_binary_project_at_path;
use super::materialize::tests::{base_project, test_dependency_plan};
use crate::ApplicationProfile;

#[test]
fn dx10_b05_actual_profiles_keep_asserts_overflow_and_unwind_boundaries() {
    let root = std::env::temp_dir().join(format!("sifr-dx10-profiles-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("owned root");
    let mut plan = test_dependency_plan("dx10");
    plan.cargo_vendor_mode = sifr_stdlib_manifest::CargoVendorMode::PackageOwned;
    for profile in [
        ApplicationProfile::Development,
        ApplicationProfile::Test,
        ApplicationProfile::Release,
    ] {
        let mut policy = CargoResolutionPolicy::normal();
        policy.application_profile = profile;
        let mut project = base_project();
        // These are actual executables, not Rust test harnesses. Black-box input
        // prevents compile-time overflow elimination, and Drop proves unwinding.
        project.main_rs = format!(
            r#"
use std::sync::atomic::{{AtomicUsize, Ordering}};
static DROPS: AtomicUsize = AtomicUsize::new(0);
struct Resource;
impl Drop for Resource {{ fn drop(&mut self) {{ DROPS.fetch_add(1, Ordering::SeqCst); }} }}
fn main() {{
    assert_eq!(cfg!(debug_assertions), {});
    let error = std::panic::catch_unwind(|| {{
        let _resource = Resource;
        let value = std::hint::black_box(i64::MAX);
        std::hint::black_box(value + 1)
    }});
    assert!(error.is_err());
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
    assert!(std::panic::catch_unwind(|| assert!(std::hint::black_box(false))).is_err());
}}
"#,
            profile != ApplicationProfile::Release
        );
        let built = materialize_binary_project_at_path(
            &root.join(profile.name()),
            "profile_boundary",
            project,
            &plan,
            &policy,
        )
        .expect("compile actual profile");
        let output = std::process::Command::new(&built.binary_path)
            .output()
            .expect("execute profile");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    std::fs::remove_dir_all(root).expect("remove owned root");
}
