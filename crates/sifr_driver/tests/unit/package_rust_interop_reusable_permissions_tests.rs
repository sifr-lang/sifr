//! Actual child writes and parent mutations remain reusable with a permissive host mask.
use super::*;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

#[test]
fn recreated_inputs_and_cargo_sentinels_remain_private_and_reusable() {
    const ENV: &str = "SIFR_RETAINED_FIXTURE_PERMISSIONS_CONTROL";
    let name = format!(
        "{}::recreated_inputs_and_cargo_sentinels_remain_private_and_reusable",
        module_path!().strip_prefix("sifr_driver::").unwrap()
    );
    let Some(path) = std::env::var_os(ENV) else {
        let temp = private_root();
        let output = Command::new(std::env::current_exe().unwrap())
            .args([&name, "--exact", "--nocapture", "--test-threads=1"])
            .env(ENV, temp.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "private fixture creation control failed: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    };
    // Only this dedicated single-test subprocess changes its process-wide mask.
    #[allow(unsafe_code)]
    unsafe {
        libc::umask(0o002);
    }
    let path = PathBuf::from(path);
    let current = BTreeMap::from([
        ("Cargo.toml".into(), b"[package]\nname='private-fixture-control'\nversion='0.0.0'\nedition='2024'\n[workspace]\n".to_vec()),
        ("Cargo.lock".into(), b"version = 4\n[[package]]\nname = 'private-fixture-control'\nversion = '0.0.0'\n".to_vec()),
        ("src/lib.rs".into(), b"pub fn control() {}\n".to_vec()),
        ("build.rs".into(), br#"fn main() { std::fs::write("CHILD_EXECUTED", "executed").unwrap(); }"#.to_vec()),
        (".sqlx/query.json".into(), b"current metadata".to_vec()),
    ]);
    let base = path.join("cache");
    for round in 0..2 {
        let owner = acquire(&base, &path, current.clone()).unwrap();
        assert_eq!(read_inputs(&owner.root).unwrap(), current);
        let sentinel = owner.root.join("CHILD_EXECUTED");
        assert!(
            !sentinel.exists(),
            "each fresh control must execute its child"
        );
        let output = owner
            .cargo()
            .args(["build", "--locked", "--offline", "--frozen", "--target-dir"])
            .arg(path.join(format!("fresh-target-{round}")))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(fs::read_to_string(&sentinel).unwrap(), "executed");
        crate::cache_storage::check_owned(&sentinel).unwrap();
        assert_eq!(
            fs::metadata(&sentinel).unwrap().permissions().mode() & 0o077,
            0
        );
        let query = owner.root.join(".sqlx/query.json");
        fs::remove_file(&query).unwrap();
        crate::build::native_storage::write_changed(&query, b"stale metadata").unwrap();
        crate::cache_storage::check_owned(&query).unwrap();
        assert_eq!(fs::read(&query).unwrap(), b"stale metadata");
        // SAFETY: this remains the dedicated subprocess. Reading/restoring the
        // mask proves the owned Cargo child did not change the parent policy.
        #[allow(unsafe_code)]
        let mask = unsafe { libc::umask(0o002) };
        assert_eq!(mask, 0o002);
    }
    let final_owner = acquire(&base, &path, current.clone()).unwrap();
    assert_eq!(read_inputs(&final_owner.root).unwrap(), current);
}
