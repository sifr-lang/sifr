//! Bounded immutable history, cheap lookup and observation authority.
use super::*;
use sifr_frontend::{
    DiskSourceProvider,
    persistence::{Observation, observations_match},
};
use std::fs;

fn source(index: usize) -> String {
    format!("def main() -> None:\n    value: int = {index}\n")
}
fn publish(store: &storage::Store, file: &Path, index: usize) {
    fs::write(file, source(index)).unwrap();
    let mut disk = DiskSourceProvider::new();
    let mut capture = CapturingSourceProvider::new(&mut disk);
    capture.read_file(file).unwrap();
    let record = CompletedCheck::capture(file, tests::context(), &capture, &[]).unwrap();
    store
        .publish(
            &serde_json::to_vec(&record).unwrap(),
            &AtomicBool::new(false),
        )
        .unwrap();
}
#[test]
fn history_over_4096_keeps_publishing_and_recent_aba() {
    let (_root, file, cache) = tests::fixture();
    let store = tests::store(&cache, &file);
    let mut rows = Vec::new();
    for index in 0..4097 {
        publish(&store, &file, index);
        // Exercise existing explicit prune without implementing housekeeping.
        if index % 32 == 0 {
            store.prune(true, false).unwrap();
        }
        if [1, 128, 1024, 4097].contains(&(index + 1)) {
            let start = Instant::now();
            let (diagnostics, report) = tests::run(&cache, &file);
            let elapsed = start.elapsed().as_micros();
            assert!(diagnostics.is_empty());
            assert_eq!(report.status, "restored");
            let generation = store.latest().unwrap();
            assert_eq!(
                generation.manifest.records.len(),
                (index + 1).min(storage::RECORD_COUNT)
            );
            let bytes: u64 = generation
                .manifest
                .records
                .iter()
                .map(|id| fs::metadata(generation.path.join(id)).unwrap().len())
                .sum();
            rows.push(serde_json::json!({
                "history_inputs": index + 1, "retained_records": generation.manifest.records.len(),
                "retained_bytes": bytes, "lookup_us": elapsed,
                "candidate_checks": report.candidate_checks, "interface_proofs": report.interface_proofs,
                "observation_count": report.observation_count,
            }));
        }
    }
    for index in [4095, 4096, 4095] {
        fs::write(&file, source(index)).unwrap();
        let (actual, report) = tests::run(&cache, &file);
        assert_eq!(report.status, "restored");
        assert_eq!(
            actual,
            tests::compute(&file, &mut DiskSourceProvider::new())
        );
    }
    fs::write(&file, source(0)).unwrap();
    assert_eq!(tests::run(&cache, &file).1.status, "published");
    if let Some(path) = std::env::var_os("DXF_HISTORY_REPORT") {
        fs::write(path, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
    }
}
#[test]
fn eviction_preserves_active_reader_and_latest() {
    let (_root, file, cache) = tests::fixture();
    let store = tests::store(&cache, &file);
    publish(&store, &file, 0);
    let reader = store.latest().unwrap();
    let old_path = reader.path.clone();
    for index in 1..=storage::RECORD_COUNT {
        publish(&store, &file, index);
    }
    let latest = store.latest().unwrap();
    let latest_path = latest.path.clone();
    assert!(
        !latest
            .manifest
            .records
            .contains(&reader.manifest.records[0])
    );
    store.prune(true, false).unwrap();
    assert!(old_path.exists() && latest_path.exists());
    assert_eq!(
        reader
            .records()
            .next()
            .unwrap()
            .unwrap()
            .result
            .inputs
            .source
            .text,
        source(0)
    );
    drop(reader);
    store.prune(true, false).unwrap();
    assert!(!old_path.exists() && latest_path.exists());
    assert_eq!(tests::run(&cache, &file).1.status, "restored");
}
#[test]
fn candidate_filter_bounds_proofs_without_false_hits() {
    let (_root, file, cache) = dx14_tests::fixture();
    let inputs = dx14_tests::inputs();
    assert_eq!(
        dx14_tests::run(&cache, &file, inputs.clone()).1.status,
        "published"
    );
    let store =
        storage::Store::open(&cache, file.parent().unwrap(), &inputs.identity().unwrap()).unwrap();
    for index in 0..100 {
        // Canonical unrelated entrypoints outside the resolver's project directory.
        let other = file
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(format!("other{index}.sifr"));
        fs::write(&other, source(index)).unwrap();
        let mut disk = DiskSourceProvider::new();
        let mut capture = CapturingSourceProvider::new(&mut disk);
        capture.read_file(&other).unwrap();
        let irrelevant = CompletedCheck::capture(&other, inputs.clone(), &capture, &[]).unwrap();
        store
            .publish(
                &serde_json::to_vec(&irrelevant).unwrap(),
                &AtomicBool::new(false),
            )
            .unwrap();
    }
    let helper = file.parent().unwrap().join("helper.sifr");
    fs::write(&helper, "def value() -> int:\n    return 2\n").unwrap();
    let (actual, report) = dx14_tests::run(&cache, &file, inputs.clone());
    assert!(actual.is_empty());
    assert_eq!(report.status, "interface-restored");
    assert_eq!(report.interface_proofs, 1);
    assert_eq!(report.candidate_checks, 101);
    assert!(report.observation_count < 100);
    fs::write(&helper, "def value() -> int:\n    return \"bad\"\n").unwrap();
    let (actual, report) = dx14_tests::run(&cache, &file, inputs.clone());
    let fresh = tempfile::tempdir().unwrap();
    assert_eq!(
        actual,
        dx14_tests::run(&fresh.path().join("cache"), &file, inputs).0
    );
    assert!(!actual.is_empty());
    assert_eq!(report.computed_checks, 1);
    assert_eq!(report.interface_proofs, 1);
}
#[test]
fn observation_compaction_preserves_ordered_absence() {
    let (_root, file, _cache) = tests::fixture();
    let absent = file.with_file_name("missing.sifr");
    let mut disk = DiskSourceProvider::new();
    let mut capture = CapturingSourceProvider::new(&mut disk);
    for _ in 0..4097 {
        assert!(!capture.is_file(&absent));
        capture.read_file(&file).unwrap();
        assert!(capture.is_file(&file));
    }
    assert_eq!(capture.observations().len(), 3);
    assert!(matches!(
        &capture.observations()[0],
        Observation::FileProbe { exists: false, .. }
    ));
    assert!(matches!(
        &capture.observations()[1],
        Observation::File { .. }
    ));
    assert!(matches!(
        &capture.observations()[2],
        Observation::FileProbe { exists: true, .. }
    ));
    assert!(observations_match(
        capture.observations(),
        &mut DiskSourceProvider::new()
    ));
    fs::write(&absent, "new").unwrap();
    assert!(capture.is_file(&absent));
    assert_eq!(capture.observations().len(), 4);
    assert!(!capture.unchanged()); // contradictory absence/presence both retained
    for index in 0..16384 {
        capture.is_file(&file.with_file_name(format!("absent{index}")));
    }
    assert_eq!(capture.observations().len(), 16384);
    assert!(!capture.observations_complete());
    assert!(CompletedCheck::capture(&file, tests::context(), &capture, &[]).is_err());
}
