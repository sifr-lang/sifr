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
