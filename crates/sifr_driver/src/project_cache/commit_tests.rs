//! The immutable publication is the commit; mutable hints grant no authority.
use super::*;
use std::{fs, thread, time::Duration};

#[test]
fn committed_history_survives_death_before_advisory_hint() {
    let (root, file, cache) = tests::fixture();
    let marker = root.path().join("committed");
    let mut child = tests::worker(&file, &cache, "after-commit", &marker);
    let deadline = Instant::now() + Duration::from_secs(15);
    while !marker.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(marker.exists(), "writer reached durable commit");
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(!file.parent().unwrap().join(".sifrbuildinfo").exists());
    let store = tests::store(&cache, &file);
    let committed = store.latest().unwrap();
    assert!(committed.records().all(|record| record.is_ok()));
    assert_eq!(store.prune(true, false).unwrap().deleted_generations, 0);
    let restored = root.path().join("restored.json");
    assert!(
        tests::worker(&file, &cache, "none", &restored)
            .wait()
            .unwrap()
            .success()
    );
    let result: serde_json::Value = serde_json::from_slice(&fs::read(restored).unwrap()).unwrap();
    assert_eq!(result["report"]["status"], "restored");
    assert_eq!(result["report"]["restored_checks"], 1);
}

#[test]
fn invalid_highest_commit_never_rolls_back_or_authorizes_reclamation() {
    let (_root, file, cache) = tests::fixture();
    let store = tests::store(&cache, &file);
    tests::publish_variants(&store, &file, 2);
    let latest = store.latest().unwrap();
    let original = fs::read(&latest.path).unwrap();
    fs::write(&latest.path, b"corrupt committed bytes").unwrap();
    assert!(store.latest().is_none());
    assert_eq!(store.prune(true, false).unwrap().deleted_generations, 0);
    fs::write(&latest.path, original).unwrap();
    assert!(store.latest().is_some());
    // A larger sequence in a filename cannot bypass the manifest's own sequence
    // and digest binding, even when its bytes are otherwise a valid generation.
    let false_head = latest.path.parent().unwrap().join(format!(
        "0000000000000010-{}",
        latest.manifest.identity().unwrap()
    ));
    fs::copy(&latest.path, &false_head).unwrap();
    drop(
        crate::cache_storage::process_entry_lock(
            false_head.parent().unwrap(),
            false_head.file_name().unwrap().to_str().unwrap(),
        )
        .unwrap(),
    );
    assert!(store.latest().is_none());
    assert_eq!(store.prune(true, false).unwrap().deleted_generations, 0);
    fs::remove_file(false_head).unwrap();
    assert!(store.latest().is_some());
}
