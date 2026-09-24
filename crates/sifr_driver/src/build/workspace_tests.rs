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
    #[cfg(unix)]
    if mode == "family-test" {
        family_case();
        return;
    }
    #[cfg(unix)]
    if mode == "stage-test" {
        stage_case();
        return;
    }
    #[cfg(unix)]
    if mode == "aux-test" {
        auxiliary_case();
        return;
    }
    #[cfg(unix)]
    if mode == "wedged-lease" {
        let parent = crate::cache_storage::root().join("native/families");
        let lease = crate::cache_storage::entry_lock(&parent, "wedged").unwrap();
        crate::cache_storage::lock_bounded(
            &lease,
            &parent.join(".locks/wedged"),
            false,
            crate::cache_storage::LEASE_WAIT,
        )
        .unwrap();
        std::fs::write(std::env::var("SIFR_DX3_READY").unwrap(), b"ready").unwrap();
        std::thread::sleep(Duration::from_secs(60));
        return;
    }
    #[cfg(unix)]
    if mode == "family-descendant" {
        let family = super::super::native_storage::NativeFamily::acquire(
            "test-toolchain",
            "test-sources",
            "test-env",
            "test-trust",
        )
        .unwrap();
        let mut descendant = Command::new("sleep").arg("60").spawn().unwrap();
        let ready = PathBuf::from(std::env::var("SIFR_DX3_READY").unwrap());
        std::fs::write(ready.with_extension("pid"), descendant.id().to_string()).unwrap();
        std::fs::write(&ready, family.root.as_os_str().as_encoded_bytes()).unwrap();
        descendant.wait().unwrap();
        return;
    }
    if mode == "prune" {
        crate::cache_storage::prune(u64::MAX, 0, false).unwrap();
        return;
    }
    if mode == "pressure" {
        // This helper is a dedicated subprocess, so its permissive umask cannot
        // leak into the parent harness or other tests.
        #[cfg(unix)]
        #[allow(unsafe_code)]
        unsafe {
            libc::umask(0o002);
        }

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
        let auxiliary = crate::cache_storage::root().join("native/artifacts/cargo_resolution");
        std::fs::create_dir_all(&auxiliary).unwrap();
        std::fs::write(auxiliary.join("auxiliary"), b"protected").unwrap();
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
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("/tmp", root.join("escape")).unwrap();
        assert!(crate::cache_storage::directory(&root.join("escape")).is_err());
    }
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

#[cfg(windows)]
#[test]
fn windows_portability_intact_concurrent_winner_is_adopted() {
    let temp = tempfile::tempdir().unwrap();
    let cache = temp.path().join("cache");
    let result = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "build::workspace::tests::windows_portability_winner_child",
            "--nocapture",
        ])
        .env("SIFR_CACHE_DIR", &cache)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[cfg(windows)]
#[test]
fn windows_portability_winner_child() {
    if std::env::var_os("SIFR_CACHE_DIR").is_none() {
        return;
    }
    let scope = std::env::current_dir().unwrap();
    let required = [Path::new("payload")];
    let prepared =
        prepare_cached_artifact("fixture", "private-winner", &scope, "one", &required).unwrap();
    let PreparedArtifactCache::Miss(pending) = prepared else {
        panic!("fresh candidate expected")
    };
    std::fs::write(pending.workspace_root().join("payload"), b"winner").unwrap();
    let winner = pending.final_root.clone();
    crate::cache_storage::directory(&winner).unwrap();
    for name in ["artifact_cache.json", "payload"] {
        let bytes = std::fs::read(pending.workspace_root().join(name)).unwrap();
        let mut file = crate::cache_storage::new_private_file(&winner.join(name)).unwrap();
        std::io::Write::write_all(&mut file, &bytes).unwrap();
        file.sync_all().unwrap();
    }
    let staging = pending.workspace_root().to_path_buf();
    let entry = pending.commit(&required).unwrap();
    assert_eq!(entry.workspace_root(), winner);
    assert_eq!(std::fs::read(winner.join("payload")).unwrap(), b"winner");
    assert!(!staging.exists());
}

#[test]
#[cfg(unix)]
fn n06_wedged_sibling_wait_is_bounded_and_diagnostic() {
    let root = std::env::temp_dir().join(format!("sifr-n06-wait-{}", std::process::id()));
    let ready = root.join("ready");
    std::fs::create_dir_all(&root).unwrap();
    let mut owner = child(&root, "wedged-lease", &ready).spawn().unwrap();
    wait_ready(&ready);
    let parent = root.join("native/families");
    let lease = crate::cache_storage::entry_lock(&parent, "wedged").unwrap();
    let error = crate::cache_storage::lock_bounded(
        &lease,
        &parent.join(".locks/wedged"),
        false,
        Duration::from_millis(80),
    )
    .unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    let diagnostic = error.to_string();
    assert!(diagnostic.contains("wedged"));
    assert!(diagnostic.contains("last_exclusive_pid"));
    let mut polls = 0;
    let cancelled = crate::cache_storage::lock_bounded_with_cancel(
        &lease,
        &parent.join(".locks/wedged"),
        false,
        Duration::from_secs(1),
        || {
            polls += 1;
            if polls > 2 {
                Err(std::io::Error::from(std::io::ErrorKind::Interrupted))
            } else {
                Ok(())
            }
        },
    )
    .unwrap_err();
    assert_eq!(cancelled.kind(), std::io::ErrorKind::Interrupted);
    owner.kill().unwrap();
    owner.wait().unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn n06_family_prune_preserves_inherited_live_child_then_reclaims_owner() {
    let root = std::env::temp_dir().join(format!("sifr-n06-family-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    assert!(
        child(&root, "family-test", &root.join("ready"))
            .status()
            .unwrap()
            .success()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
fn family_case() {
    let root = crate::cache_storage::root();
    let ready = root.join("ready");
    std::fs::create_dir_all(&root).unwrap();
    let mut owner = child(&root, "family-descendant", &ready).spawn().unwrap();
    wait_ready(&ready);
    let family = PathBuf::from(std::fs::read_to_string(&ready).unwrap());
    owner.kill().unwrap();
    owner.wait().unwrap();
    let key = family.file_name().unwrap().to_str().unwrap();
    let lock = crate::cache_storage::entry_lock(family.parent().unwrap(), key).unwrap();
    assert!(
        lock.try_lock().is_err(),
        "native child lost inherited family lease"
    );
    let wait = crate::cache_storage::lock_bounded(
        &lock,
        &family.parent().unwrap().join(".locks").join(key),
        false,
        Duration::from_millis(80),
    )
    .unwrap_err();
    assert_eq!(wait.kind(), std::io::ErrorKind::TimedOut);
    assert!(wait.to_string().contains("live child"));
    let inspected = crate::cache_storage::inspect().unwrap();
    assert!(
        inspected
            .entries
            .iter()
            .any(|entry| entry.path == family && entry.protected)
    );
    crate::cache_storage::prune(u64::MAX, 0, false).unwrap();
    assert!(family.exists(), "prune removed live family");
    let pid = std::fs::read_to_string(ready.with_extension("pid")).unwrap();
    assert!(
        Command::new("kill")
            .args(["-KILL", pid.trim()])
            .status()
            .unwrap()
            .success()
    );
    let started = Instant::now();
    while lock.try_lock().is_err() {
        assert!(started.elapsed() < Duration::from_secs(5));
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(lock);
    crate::cache_storage::prune(u64::MAX, 0, false).unwrap();
    assert!(!family.exists(), "inactive owned family was not reclaimed");
}

#[test]
#[cfg(unix)]
fn n06_abandoned_stage_prunes_only_matching_owner_scope() {
    let root = std::env::temp_dir().join(format!("sifr-n06-stage-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    assert!(
        child(&root, "stage-test", &root.join("ready"))
            .status()
            .unwrap()
            .success()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
fn stage_case() {
    let root = crate::cache_storage::root();
    let artifacts = root.join("native/artifacts/fixture");
    crate::cache_storage::directory(&artifacts).unwrap();
    let key = "a".repeat(64);
    let lock = crate::cache_storage::entry_lock(&artifacts, &key).unwrap();
    let abandoned = artifacts.join(format!("{key}.stage-abandoned"));
    crate::cache_storage::directory(&abandoned).unwrap();
    let scope = crate::cache_storage::owner_scope().unwrap();
    std::fs::write(
        abandoned.join("artifact_cache.json"),
        serde_json::to_vec(&serde_json::json!({"scope":scope})).unwrap(),
    )
    .unwrap();
    let foreign_key = "b".repeat(64);
    let _foreign_lock = crate::cache_storage::entry_lock(&artifacts, &foreign_key).unwrap();
    let foreign = artifacts.join(format!("{foreign_key}.stage-foreign"));
    crate::cache_storage::directory(&foreign).unwrap();
    std::fs::write(
        foreign.join("artifact_cache.json"),
        serde_json::to_vec(&serde_json::json!({"scope":"/another/session"})).unwrap(),
    )
    .unwrap();
    crate::cache_storage::prune(u64::MAX, 0, true).unwrap();
    assert!(abandoned.exists() && foreign.exists());
    crate::cache_storage::prune(u64::MAX, 0, false).unwrap();
    assert!(!abandoned.exists(), "abandoned owned stage remained");
    assert!(foreign.exists(), "foreign stage was removed");
    assert!(
        artifacts.join(".locks").join(key).exists(),
        "ownership lock inode was unlinked"
    );
    drop(lock);
}

#[test]
#[cfg(unix)]
fn n06_orphan_auxiliary_roots_require_owner_and_lease() {
    let root = std::env::temp_dir().join(format!("sifr-n06-aux-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    assert!(
        child(&root, "aux-test", &root.join("ready"))
            .status()
            .unwrap()
            .success()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
fn auxiliary_case() {
    let cache = crate::cache_storage::root();
    let root = cache.join("native/artifacts/cargo_resolution");
    crate::cache_storage::directory(&root).unwrap();
    let unknown = cache.join("native/unknown-owner");
    crate::cache_storage::directory(&unknown).unwrap();
    let inventory = crate::cache_storage::inspect().unwrap();
    assert!(inventory.protected_roots.contains(&unknown));
    assert!(inventory.owners.iter().any(|owner| {
        owner.path == cache.join("native/families") && owner.owner == "native Cargo families"
    }));
    let scope = crate::cache_storage::owner_scope().unwrap();
    let key = "c".repeat(64);
    let owned = root.join(&key);
    crate::cache_storage::directory(&owned).unwrap();
    std::fs::write(
        owned.join("resolution_owner.json"),
        serde_json::to_vec(&serde_json::json!({"schema":1,"key":key,"owner_scope":scope})).unwrap(),
    )
    .unwrap();
    std::fs::write(owned.join("Cargo.lock"), b"orphan").unwrap();
    let lease = crate::cache_storage::entry_lock(&root, &key).unwrap();
    lease.lock().unwrap();
    let foreign_key = "d".repeat(64);
    let foreign = root.join(&foreign_key);
    crate::cache_storage::directory(&foreign).unwrap();
    std::fs::write(
        foreign.join("resolution_owner.json"),
        serde_json::to_vec(
            &serde_json::json!({"schema":1,"key":foreign_key,"owner_scope":"/another/session"}),
        )
        .unwrap(),
    )
    .unwrap();
    let _foreign_lock = crate::cache_storage::entry_lock(&root, &foreign_key).unwrap();
    crate::cache_storage::prune(u64::MAX, 0, false).unwrap();
    assert!(owned.exists(), "live auxiliary root was removed");
    drop(lease);
    crate::cache_storage::prune(u64::MAX, 0, false).unwrap();
    assert!(!owned.exists(), "orphan owned auxiliary root remained");
    assert!(foreign.exists(), "foreign auxiliary root was removed");
    assert!(unknown.exists(), "unknown auxiliary owner was removed");
    assert!(
        root.join(".locks").join(key).exists(),
        "auxiliary lock inode was unlinked"
    );
}
