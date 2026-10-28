use super::{generation_key, latest_generation, next_sequence};
use std::{fs, time::Instant};

fn name(sequence: u64, digit: char) -> String {
    format!("{sequence:016x}-{}", digit.to_string().repeat(64))
}

#[test]
fn commit_names_are_canonical_and_sequences_never_wrap() {
    assert_eq!(next_sequence(None).unwrap(), 0);
    assert_eq!(next_sequence(Some(&name(15, 'a'))).unwrap(), 16);
    assert!(next_sequence(Some(&name(u64::MAX, 'a'))).is_err());
    for bad in [
        name(1, 'x'),
        format!("../{}", name(1, 'a')),
        format!("{}.stage-1-2", name(1, 'a')),
        format!("F000000000000000-{}", "a".repeat(64)),
        "a".repeat(64),
        String::new(),
    ] {
        assert!(!generation_key(&bad), "{bad}");
        assert!(next_sequence(Some(&bad)).is_err(), "{bad}");
    }
}

#[test]
fn head_discovery_rejects_ambiguity_and_ignores_noncommitted_names() {
    let root = tempfile::tempdir().unwrap();
    assert_eq!(latest_generation(root.path()).unwrap(), None);
    for (sequence, digit) in [(9, 'a'), (1, 'b'), (16, 'c')] {
        fs::write(root.path().join(name(sequence, digit)), []).unwrap();
    }
    fs::write(root.path().join(format!("{}.stage-1-2", name(99, 'a'))), []).unwrap();
    fs::write(root.path().join("unrecognized"), []).unwrap();
    assert_eq!(latest_generation(root.path()).unwrap(), Some(name(16, 'c')));
    let duplicate = root.path().join(name(16, 'd'));
    fs::write(&duplicate, []).unwrap();
    assert!(latest_generation(root.path()).is_err());
    fs::remove_file(duplicate).unwrap();
    assert_eq!(latest_generation(root.path()).unwrap(), Some(name(16, 'c')));
}

#[test]
fn head_discovery_handles_4097_unpruned_names() {
    let root = tempfile::tempdir().unwrap();
    for sequence in (0..4097).rev() {
        fs::write(root.path().join(name(sequence, 'a')), []).unwrap();
    }
    let start = Instant::now();
    assert_eq!(
        latest_generation(root.path()).unwrap(),
        Some(name(4096, 'a'))
    );
    eprintln!(
        "unpruned-4097-name-head-discovery-us={}",
        start.elapsed().as_micros()
    );
}
