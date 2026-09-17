use super::*;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[test]
fn storage_child() {
    let Ok(mode) = std::env::var("SIFR_DX3_CHILD") else {
        return;
    };
    let scope = std::env::current_dir().unwrap();
    let required = [Path::new("payload")];
    if mode == "prune" {
        crate::cache_storage::prune(u64::MAX, 0, false).unwrap();
        return;
    }
    if mode == "pressure" {
        let old = prepare_cached_artifact("fixture", "dx3", &scope, "old", &required).unwrap();
        let PreparedArtifactCache::Miss(pending) = old else {
            panic!("fresh root")
        };
        let file = std::fs::File::create(pending.workspace_root().join("payload")).unwrap();
        file.set_len(21 * 1024 * 1024 * 1024).unwrap();
        let old = pending.commit(&required).unwrap();
        let old_path = old.workspace_root().to_path_buf();
        std::thread::sleep(Duration::from_millis(20));
        let new = prepare_cached_artifact("fixture", "dx3", &scope, "current", &required).unwrap();
        let PreparedArtifactCache::Miss(pending) = new else {
            panic!("fresh root")
        };
        std::fs::write(pending.workspace_root().join("payload"), b"current").unwrap();
        let current = pending.commit(&required).unwrap();
        assert!(
            child(
                &crate::cache_storage::root(),
                "prune",
                &old_path.join("unused")
            )
            .status()
            .unwrap()
            .success()
        );
        assert!(
            old_path.exists(),
            "concurrent prune removed active older reader"
        );
        drop(old);
        crate::cache_storage::prune(1, u64::MAX, false).unwrap();
        assert!(old_path.exists(), "size alone caused cleanup");
        crate::cache_storage::prune(u64::MAX, 0, true).unwrap();
        assert!(old_path.exists(), "dry run changed storage");
        crate::cache_storage::prune(u64::MAX, 0, false).unwrap();
        assert!(
            !old_path.exists(),
            "pressure failed to reclaim inactive entry"
        );
        assert!(
            current.workspace_root().exists(),
            "active current entry removed"
        );
        return;
    }
    let prepared = prepare_cached_artifact("fixture", "dx3", &scope, "one", &required).unwrap();
    match prepared {
        PreparedArtifactCache::Hit(entry) => {
            if mode == "read" {
                std::fs::write(
                    std::env::var("SIFR_DX3_READY").unwrap(),
                    entry.workspace_root().as_os_str().as_encoded_bytes(),
                )
                .unwrap();
                std::thread::sleep(Duration::from_secs(60));
            }
        }
        PreparedArtifactCache::Miss(pending) => {
            let stage = pending.workspace_root().to_path_buf();
            if mode == "before" {
                std::fs::write(
                    std::env::var("SIFR_DX3_READY").unwrap(),
                    stage.as_os_str().as_encoded_bytes(),
                )
                .unwrap();
                std::thread::sleep(Duration::from_secs(60));
            }
            std::fs::write(stage.join("payload"), b"complete").unwrap();
            if mode == "descendant" {
                let mut descendant = Command::new("sleep").arg("60").spawn().unwrap();
                let ready = PathBuf::from(std::env::var("SIFR_DX3_READY").unwrap());
                std::fs::write(ready.with_extension("pid"), descendant.id().to_string()).unwrap();
                std::fs::write(&ready, stage.as_os_str().as_encoded_bytes()).unwrap();
                descendant.wait().unwrap();
            }

            if mode == "during" {
                std::fs::write(
                    std::env::var("SIFR_DX3_READY").unwrap(),
                    stage.as_os_str().as_encoded_bytes(),
                )
                .unwrap();
                std::thread::sleep(Duration::from_secs(60));
            }
            let entry = pending.commit(&required).unwrap();
            assert!(
                entry
                    .workspace_root()
                    .starts_with(crate::cache_storage::root())
            );
            if mode == "after" {
                std::fs::write(
                    std::env::var("SIFR_DX3_READY").unwrap(),
                    entry.workspace_root().as_os_str().as_encoded_bytes(),
                )
                .unwrap();
                std::thread::sleep(Duration::from_secs(60));
            }
        }
    }
}

fn child(root: &Path, mode: &str, ready: &Path) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "build::workspace::tests::storage_child",
            "--nocapture",
        ])
        .env("SIFR_CACHE_DIR", root)
        .env("SIFR_DX3_CHILD", mode)
        .env("SIFR_DX3_READY", ready)
        .env("TMPDIR", "/dev/shm")
        .stdout(Stdio::null());
    command
}

fn wait_ready(path: &Path) {
    let start = Instant::now();
    while !path.exists() {
        assert!(
            start.elapsed() < Duration::from_secs(15),
            "child did not signal readiness"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn dx3_killed_writers_readers_and_cross_filesystem() {
    let root = std::env::temp_dir().join(format!("sifr-dx3-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::MetadataExt;
        assert_ne!(
            std::fs::metadata(&root).unwrap().dev(),
            std::fs::metadata("/dev/shm").unwrap().dev()
        );
    }
    for mode in ["before", "during", "after", "descendant"] {
        let cache = root.join(mode);
        let ready = root.join(format!("{mode}.ready"));
        let mut process = child(&cache, mode, &ready).spawn().unwrap();
        wait_ready(&ready);
        let entry = PathBuf::from(std::fs::read_to_string(&ready).unwrap());
        assert!(
            entry.starts_with(&cache),
            "staging escaped destination filesystem"
        );
        process.kill().unwrap();
        process.wait().unwrap();
        if mode == "descendant" {
            let key = entry
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .split(".stage-")
                .next()
                .unwrap();
            let lock = crate::cache_storage::entry_lock(entry.parent().unwrap(), key).unwrap();
            assert!(
                lock.try_lock().is_err(),
                "orphan native descendant lost the writer lease"
            );
            let pid = std::fs::read_to_string(ready.with_extension("pid")).unwrap();
            assert!(
                Command::new("kill")
                    .args(["-KILL", pid.trim()])
                    .status()
                    .unwrap()
                    .success()
            );
            let start = Instant::now();
            while lock.try_lock().is_err() {
                assert!(start.elapsed() < Duration::from_secs(5));
                std::thread::sleep(Duration::from_millis(10));
            }
        }

        assert!(child(&cache, "finish", &ready).status().unwrap().success());
        let mut reader = child(&cache, "read", &root.join("reader.ready"))
            .spawn()
            .unwrap();
        wait_ready(&root.join("reader.ready"));
        let path = PathBuf::from(std::fs::read_to_string(root.join("reader.ready")).unwrap());
        let key = path.file_name().unwrap().to_str().unwrap();
        let lock = crate::cache_storage::entry_lock(path.parent().unwrap(), key).unwrap();
        assert!(lock.try_lock().is_err(), "active reader lost protection");
        // Concurrent readers must proceed without deadlock.
        assert!(child(&cache, "finish", &ready).status().unwrap().success());
        reader.kill().unwrap();
        reader.wait().unwrap();
        assert!(lock.try_lock().is_ok(), "dead reader retained OS lock");
        drop(lock);
        std::fs::remove_file(root.join("reader.ready")).unwrap();
        std::fs::remove_file(path.join("payload")).unwrap();
        assert!(
            child(&cache, "finish", &ready).status().unwrap().success(),
            "missing manifest payload was not repaired"
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn dx3_rejects_traversal_symlinks_and_invalid_winner() {
    assert!(crate::cache_storage::relative(Path::new("../outside")).is_err());
    let root = std::env::temp_dir().join(format!("sifr-dx3-winner-{}", std::process::id()));
    crate::cache_storage::directory(&root).unwrap();
    let staging = root.join("stage");
    crate::cache_storage::directory(&staging).unwrap();
    let final_root = root.join("final");
    std::fs::create_dir(&final_root).unwrap();
    std::fs::write(final_root.join("untrusted"), b"not a manifest").unwrap();
    std::fs::write(staging.join("payload"), b"ready").unwrap();
    let lease = std::sync::Arc::new(crate::cache_storage::entry_lock(&root, "test").unwrap());
    lease.lock().unwrap();
    let pending = PendingCachedArtifact {
        lease,
        scope: std::env::current_dir().unwrap(),
        native_identity: "fixture".into(),
        final_root,
        staging_root: staging.clone(),
        report: ArtifactCacheReport {
            namespace: "test".into(),
            key: "key".into(),
            workspace_root: staging,
            cache_hit: false,
            miss_reason: None,
        },
    };
    assert!(pending.commit(&[Path::new("payload")]).is_err());
    std::os::unix::fs::symlink("/tmp", root.join("escape")).unwrap();
    assert!(crate::cache_storage::directory(&root.join("escape")).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn dx3_pressure_cleanup_is_owned_and_preserves_active_candidate() {
    let root = std::env::temp_dir().join(format!("sifr-dx3-pressure-{}", std::process::id()));
    assert!(
        child(&root, "pressure", &root.join("ready"))
            .status()
            .unwrap()
            .success()
    );
    std::fs::remove_dir_all(root).unwrap();
    assert!(
        crate::cache_storage::check_owned(Path::new("/")).is_err(),
        "foreign owner accepted"
    );
}
