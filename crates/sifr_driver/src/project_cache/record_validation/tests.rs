use super::*;
use crate::project_cache::tests as fixtures;
use sifr_frontend::persistence::Family;
use std::sync::Arc;

fn encoded(record: &CompletedCheck) -> (String, Vec<u8>) {
    let bytes = serde_json::to_vec(record).unwrap();
    (stamp("project-record-v1", &bytes).unwrap(), bytes)
}
fn owner() -> RecordValidation {
    RecordValidation::new(fixtures::context().identity().unwrap())
}

#[test]
fn reuse_binds_current_bytes_and_expected_stored_identity() {
    let (_root, file, _cache) = fixtures::fixture();
    let (id, bytes) = encoded(&fixtures::super_record(&file));
    let validation = owner();
    validation.validate(&id, &bytes, "mismatch").unwrap();
    validation.validate(&id, &bytes, "mismatch").unwrap();
    assert_eq!(validation.stats(), (1, 1, 1));
    let mut changed = bytes.clone();
    changed[0] = b'[';
    assert!(validation.validate(&id, &changed, "mismatch").is_err());
    assert!(
        validation
            .validate(&"0".repeat(64), &bytes, "mismatch")
            .is_err()
    );
    validation.validate(&id, &bytes, "mismatch").unwrap();
    assert_eq!(validation.stats(), (2, 3, 1));
}

#[test]
fn invalid_schema_context_diagnostics_and_json_never_publish_facts() {
    let (_root, file, _cache) = fixtures::fixture();
    let original = fixtures::super_record(&file);
    let mut bad_schema = original.clone();
    bad_schema.schema += 1;
    let mut bad_diagnostics = original.clone();
    bad_diagnostics.result.diagnostics = Family::Pending;
    let validation = owner();
    for record in [bad_schema, bad_diagnostics] {
        let (id, bytes) = encoded(&record);
        for _ in 0..2 {
            assert!(validation.validate(&id, &bytes, "mismatch").is_err());
        }
    }
    let bytes = b"{".to_vec();
    let id = stamp("project-record-v1", &bytes).unwrap();
    for _ in 0..2 {
        assert!(validation.validate(&id, &bytes, "mismatch").is_err());
    }
    assert_eq!(validation.stats(), (0, 6, 0));
    let incompatible = RecordValidation::new("0".repeat(64));
    let (id, bytes) = encoded(&original);
    for _ in 0..2 {
        assert!(incompatible.validate(&id, &bytes, "mismatch").is_err());
    }
    assert_eq!(incompatible.stats(), (0, 2, 0));
    validation.validate(&id, &bytes, "mismatch").unwrap();
    assert_eq!(validation.stats(), (0, 7, 1));
}

#[test]
fn bounded_recency_and_zero_capacity_preserve_validation() {
    let (_root, file, _cache) = fixtures::fixture();
    let original = fixtures::super_record(&file);
    let records: Vec<_> = (0..3)
        .map(|index| {
            let mut record = original.clone();
            record
                .displays
                .insert(format!("unused-{index}"), "unused".into());
            encoded(&record)
        })
        .collect();
    let validation = RecordValidation::with_capacity(fixtures::context().identity().unwrap(), 2);
    for index in [0, 1, 0, 2, 0] {
        validation
            .validate(&records[index].0, &records[index].1, "mismatch")
            .unwrap();
    }
    assert_eq!(validation.stats(), (2, 3, 2));
    validation
        .validate(&records[1].0, &records[1].1, "mismatch")
        .unwrap();
    assert_eq!(validation.stats(), (2, 4, 2));
    let disabled = RecordValidation::with_capacity(fixtures::context().identity().unwrap(), 0);
    for _ in 0..2 {
        disabled
            .validate(&records[0].0, &records[0].1, "mismatch")
            .unwrap();
    }
    assert_eq!(disabled.stats(), (0, 2, 0));
}

#[test]
fn byte_budget_evicts_proofs_without_losing_digest_facts() {
    let (_root, file, _cache) = fixtures::fixture();
    let original = fixtures::super_record(&file);
    let records: Vec<_> = (0..3)
        .map(|index| {
            let mut record = original.clone();
            record
                .displays
                .insert(format!("unused-{index}"), "unused".into());
            encoded(&record)
        })
        .collect();
    let budget = records[0].1.len();
    assert!(records.iter().all(|(_, bytes)| bytes.len() == budget));
    let validation =
        RecordValidation::with_limits(fixtures::context().identity().unwrap(), 2, budget);
    for (id, bytes) in &records[..2] {
        validation.validate(id, bytes, "mismatch").unwrap();
    }
    {
        let entries = validation.entries.lock().unwrap();
        assert_eq!(entries.retained_bytes, budget);
        assert!(entries.facts[0].bytes.is_none());
        assert_eq!(
            entries.facts[1].bytes.as_deref(),
            Some(records[1].1.as_slice())
        );
    }
    // The evicted byte proof still permits a digest hit, while corruption of
    // either representation is rejected and cannot displace a successful fact.
    for (id, bytes) in &records[..2] {
        validation.validate(id, bytes, "mismatch").unwrap();
        let mut changed = bytes.clone();
        changed[0] = b'[';
        assert!(validation.validate(id, &changed, "mismatch").is_err());
    }
    assert_eq!(validation.stats(), (2, 4, 2));
    validation
        .validate(&records[2].0, &records[2].1, "mismatch")
        .unwrap();
    let entries = validation.entries.lock().unwrap();
    assert_eq!(entries.retained_bytes, budget);
    assert_eq!(entries.facts.len(), 2);
    assert_eq!(
        entries.facts.back().unwrap().bytes.as_deref(),
        Some(records[2].1.as_slice())
    );
    assert!(entries.facts.front().unwrap().bytes.is_none());
}

#[test]
fn oversized_and_zero_budget_records_keep_digest_validation() {
    let (_root, file, _cache) = fixtures::fixture();
    let (id, bytes) = encoded(&fixtures::super_record(&file));
    for budget in [0, bytes.len() - 1] {
        let validation =
            RecordValidation::with_limits(fixtures::context().identity().unwrap(), 2, budget);
        for _ in 0..2 {
            validation.validate(&id, &bytes, "mismatch").unwrap();
        }
        assert!(
            validation
                .validate(&"0".repeat(64), &bytes, "mismatch")
                .is_err()
        );
        assert_eq!(validation.stats(), (1, 2, 1));
        let entries = validation.entries.lock().unwrap();
        assert_eq!(entries.retained_bytes, 0);
        assert!(entries.facts[0].bytes.is_none());
    }
    let disabled =
        RecordValidation::with_limits(fixtures::context().identity().unwrap(), 0, bytes.len());
    disabled.validate(&id, &bytes, "mismatch").unwrap();
    assert_eq!(disabled.entries.lock().unwrap().retained_bytes, 0);
}

#[test]
fn count_eviction_releases_byte_budget() {
    let (_root, file, _cache) = fixtures::fixture();
    let mut record = fixtures::super_record(&file);
    let (first_id, first) = encoded(&record);
    record.displays.insert("unused".into(), "unused".into());
    let (second_id, second) = encoded(&record);
    let validation = RecordValidation::with_limits(
        fixtures::context().identity().unwrap(),
        1,
        first.len() + second.len(),
    );
    for (id, bytes) in [
        (&first_id, &first),
        (&second_id, &second),
        (&first_id, &first),
    ] {
        validation.validate(id, bytes, "mismatch").unwrap();
        let entries = validation.entries.lock().unwrap();
        assert_eq!(entries.retained_bytes, bytes.len());
        assert_eq!(entries.facts.len(), 1);
        assert_eq!(entries.facts[0].bytes.as_deref(), Some(bytes.as_slice()));
    }
    assert_eq!(validation.stats(), (0, 3, 1));
}

#[test]
fn generation_keeps_fact_owner_alive_until_last_reader_drops() {
    let (_root, file, cache) = fixtures::fixture();
    assert_eq!(fixtures::run(&cache, &file).1.status, "published");
    let store = fixtures::store(&cache, &file);
    let weak = Arc::downgrade(&store.validation);
    let generation = store.latest().unwrap();
    assert_eq!(store.validation.stats(), (0, 1, 1));
    generation.records().next().unwrap().unwrap();
    assert_eq!(store.validation.stats(), (1, 1, 1));
    drop(store);
    assert!(weak.upgrade().is_some());
    drop(generation);
    assert!(weak.upgrade().is_none());
}

#[test]
fn warm_facts_reject_same_length_same_mtime_corruption_and_recover() {
    use std::fs::{self, File, FileTimes};
    let (_root, file, cache) = fixtures::fixture();
    assert_eq!(fixtures::run(&cache, &file).1.status, "published");
    let store = fixtures::store(&cache, &file);
    let generation = store.latest().unwrap();
    let path = generation
        .record_path(&generation.manifest.records[0])
        .unwrap();
    let original = fs::read(&path).unwrap();
    let modified = fs::metadata(&path).unwrap().modified().unwrap();
    generation.records().next().unwrap().unwrap();
    assert_eq!(store.validation.stats(), (1, 1, 1));
    let mut changed = original.clone();
    // Corrupt the payload, preserving the envelope, file size and mtime.
    changed[16] = b'[';
    fs::write(&path, &changed).unwrap();
    File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(FileTimes::new().set_modified(modified))
        .unwrap();
    assert_eq!(fs::metadata(&path).unwrap().len(), original.len() as u64);
    assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), modified);
    assert!(store.latest().is_none());
    assert!(generation.records().next().unwrap().is_err());
    fs::write(&path, original).unwrap();
    File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(FileTimes::new().set_modified(modified))
        .unwrap();
    assert!(store.latest().is_some());
    generation.records().next().unwrap().unwrap();
}

#[cfg(unix)]
#[test]
fn warm_facts_still_check_permissions_and_symlink_substitution() {
    use std::{
        fs,
        os::unix::fs::{PermissionsExt, symlink},
    };
    let (root, file, cache) = fixtures::fixture();
    assert_eq!(fixtures::run(&cache, &file).1.status, "published");
    let store = fixtures::store(&cache, &file);
    let generation = store.latest().unwrap();
    let path = generation
        .record_path(&generation.manifest.records[0])
        .unwrap();
    generation.records().next().unwrap().unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o666)).unwrap();
    assert!(store.latest().is_none());
    assert!(generation.records().next().unwrap().is_err());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(store.latest().is_some());
    let preserved = root.path().join("preserved-record");
    fs::rename(&path, &preserved).unwrap();
    symlink(&preserved, &path).unwrap();
    assert!(store.latest().is_none());
    assert!(generation.records().next().unwrap().is_err());
    fs::remove_file(&path).unwrap();
    fs::rename(&preserved, &path).unwrap();
    assert!(store.latest().is_some());
    generation.records().next().unwrap().unwrap();
}
