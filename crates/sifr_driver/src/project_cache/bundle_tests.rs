//! Publication bounds and reference ownership, independent of filesystem layout.
use super::{AtomicBool, storage, tests};
use std::fs;

#[test]
fn large_records_write_only_the_new_payload_and_bound_retained_history() {
    let (_root, file, cache) = tests::fixture();
    let store = tests::store(&cache, &file);
    let mut record = tests::super_record(&file);
    // Five near-limit records exceed the 64 MiB history bound. Each publication
    // must write one new payload rather than copying the retained history.
    record
        .displays
        .insert("padding".into(), "p".repeat(15 * 1024 * 1024));
    let mut written = 0;
    let mut payload_bytes = 0;
    for index in 0..5 {
        record.displays.insert("nonce".into(), index.to_string());
        let bytes = serde_json::to_vec(&record).unwrap();
        let id = store.publish(&bytes, &AtomicBool::new(false)).unwrap();
        let path = store.root.join("generations").join(id);
        let length = fs::metadata(path).unwrap().len();
        assert!(length < bytes.len() as u64 + storage::MANIFEST_LIMIT);
        written += length;
        payload_bytes += bytes.len() as u64;
    }
    assert!(written < payload_bytes + 5 * storage::MANIFEST_LIMIT);
    let latest = store.latest().unwrap();
    assert_eq!(latest.manifest.records.len(), 4);
    assert!(latest.records().all(|record| record.is_ok()));
    assert!(
        latest
            .manifest
            .payloads
            .values()
            .map(|payload| payload.length)
            .sum::<u64>()
            <= 64 * 1024 * 1024
    );
    let before = fs::read(store.root.join("latest")).unwrap();
    let oversized = vec![b' '; storage::RECORD_LIMIT as usize + 1];
    assert!(store.publish(&oversized, &AtomicBool::new(false)).is_err());
    assert_eq!(fs::read(store.root.join("latest")).unwrap(), before);
    assert_eq!(store.prune(true, false).unwrap().deleted_generations, 1);
    assert!(latest.records().all(|record| record.is_ok()));
}

#[test]
fn bundle_corruption_never_restores_or_reclaims_an_ambiguous_history() {
    let (_root, file, cache) = tests::fixture();
    let store = tests::store(&cache, &file);
    tests::publish_variants(&store, &file, 2);
    let reader = store.latest().unwrap();
    let path = reader.path.clone();
    let original = fs::read(&path).unwrap();
    let payload_length = reader.manifest.payloads[reader.manifest.records.last().unwrap()].length;
    let mut mutations = vec![original[..8].to_vec(), original[..16].to_vec()];
    mutations.push(original[..16 + payload_length as usize].to_vec()); // no manifest
    let mut invalid_length = original.clone();
    invalid_length[8..16].copy_from_slice(&(storage::RECORD_LIMIT + 1).to_le_bytes());
    mutations.push(invalid_length);
    let mut invalid_magic = original.clone();
    invalid_magic[0] ^= 1;
    mutations.push(invalid_magic);
    let mut invalid_manifest = original.clone();
    *invalid_manifest.last_mut().unwrap() = b'!';
    mutations.push(invalid_manifest);
    let mut trailing = original.clone();
    trailing.extend_from_slice(b"garbage");
    mutations.push(trailing);
    for changed in mutations {
        fs::write(&path, changed).unwrap();
        assert!(store.latest().is_none());
        assert_eq!(store.prune(true, false).unwrap().deleted_generations, 0);
        fs::write(&path, &original).unwrap();
        assert!(store.latest().is_some());
    }
    // Cached semantic validation must not admit changed inherited bytes.
    let inherited = reader.record_path(&reader.manifest.records[0]).unwrap();
    let saved = fs::read(&inherited).unwrap();
    let mut changed = saved.clone();
    changed[16] ^= 1;
    fs::write(&inherited, changed).unwrap();
    assert!(store.latest().is_none());
    assert!(reader.records().any(|record| record.is_err()));
    fs::write(&inherited, &saved).unwrap();
    assert!(reader.records().all(|record| record.is_ok()));
    assert!(store.latest().is_some());
    // The current manifest is the authority for an inherited record. Its
    // predecessor's historical index is no longer part of that record, just as
    // corrupting an old directory manifest did not corrupt a retained hardlink.
    let mut retired_index = saved;
    *retired_index.last_mut().unwrap() = b'!';
    fs::write(inherited, retired_index).unwrap();
    assert!(store.latest().is_some());
    assert!(reader.records().all(|record| record.is_ok()));
}

#[test]
fn legacy_namespace_is_neither_consumed_nor_reclaimed() {
    let (_root, file, cache) = tests::fixture();
    let store = tests::store(&cache, &file);
    let legacy = cache
        .join("projects")
        .join(&store.workspace)
        .join(&store.context);
    fs::create_dir_all(&legacy).unwrap();
    fs::write(legacy.join("latest"), b"legacy sentinel").unwrap();
    assert!(store.latest().is_none());
    assert_eq!(tests::run(&cache, &file).1.status, "published");
    super::housekeeping::prune_workspace(&cache, file.parent().unwrap(), true, false).unwrap();
    assert_eq!(fs::read(legacy.join("latest")).unwrap(), b"legacy sentinel");
}

#[test]
fn detached_history_keeps_inherited_payload_owners_until_reader_release() {
    let (_root, file, cache) = tests::fixture();
    let store = tests::store(&cache, &file);
    tests::publish_variants(&store, &file, 2);
    let reader = store.latest().unwrap();
    let retained: Vec<_> = reader
        .manifest
        .records
        .iter()
        .map(|id| {
            (
                id.clone(),
                reader.record_path(id).unwrap(),
                reader.read_record(id).unwrap(),
            )
        })
        .collect();
    assert_ne!(retained[0].1, reader.path);
    tests::publish_variants(&store, &file, storage::RECORD_COUNT + 2);
    let latest = store.latest().unwrap();
    assert!(
        retained
            .iter()
            .all(|(id, _, _)| !latest.manifest.records.contains(id))
    );
    store.prune(true, false).unwrap();
    for (id, path, bytes) in &retained {
        assert!(path.exists());
        assert_eq!(reader.read_record(id).unwrap(), *bytes);
    }
    // An active predecessor whose index cannot be decoded supplies no safe
    // reachability proof. Preserve its payload owners and all ambiguous history.
    let original = fs::read(&reader.path).unwrap();
    let mut damaged_index = original.clone();
    *damaged_index.last_mut().unwrap() = b'!';
    fs::write(&reader.path, damaged_index).unwrap();
    assert_eq!(store.prune(true, false).unwrap().deleted_generations, 0);
    assert!(retained.iter().all(|(_, path, _)| path.exists()));
    fs::write(&reader.path, original).unwrap();
    drop(reader);
    store.prune(true, false).unwrap();
    assert!(retained.iter().all(|(_, path, _)| !path.exists()));
    assert!(latest.records().all(|record| record.is_ok()));
}

#[test]
fn reader_admission_blocks_new_readers_but_preserves_existing_readers() {
    let (_root, file, cache) = tests::fixture();
    let store = tests::store(&cache, &file);
    tests::publish_variants(&store, &file, 2);
    let reader = store.latest().unwrap();
    let admission = sifr_cache_storage::ProcessLease::exclusive(
        super::housekeeping::existing_lease(&store.root, storage::READER_ADMISSION).unwrap(),
    )
    .unwrap();
    assert!(store.latest().is_none());
    assert!(reader.records().all(|record| record.is_ok()));
    drop(admission);
    assert!(store.latest().is_some());
}

#[cfg(unix)]
#[test]
fn reclamation_under_low_descriptor_limit_preserves_reader_and_latest() {
    // Reuse the real eviction/reader assertions inside a process with fewer
    // descriptors than the retained history has payload owners. No process-wide
    // limits are changed in the test harness itself.
    let result = std::process::Command::new("/bin/sh")
        .args([
            "-c",
            "ulimit -n 64 && exec \"$@\"",
            "history-descriptor-control",
        ])
        .arg(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "project_cache::history_tests::eviction_preserves_active_reader_and_latest",
            "--test-threads=1",
            "--nocapture",
        ])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        String::from_utf8_lossy(&result.stdout)
            .contains("test result: ok. 1 passed; 0 failed; 0 ignored;")
    );
}
