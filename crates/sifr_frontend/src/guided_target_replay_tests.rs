use crate::guided_targets::{GuidedTarget, replay, source_for};

#[test]
fn lowering_seed_replay() {
    let seeds: [&[u8]; 3] = [
        include_bytes!("../../../verification/fuzz/corpus/lowering/scalars"),
        include_bytes!("../../../verification/fuzz/corpus/lowering/branches"),
        include_bytes!("../../../verification/fuzz/corpus/lowering/collections"),
    ];
    let sources: Vec<_> = seeds
        .iter()
        .map(|seed| {
            replay(GuidedTarget::Lowering, seed);
            source_for(GuidedTarget::Lowering, seed)
        })
        .collect();
    assert!(sources.iter().any(|source| source.contains("if ")));
    assert!(sources.iter().any(|source| source.contains("list[int]")));
    assert!(sources.iter().any(|source| source.contains("= helper(")));
}

#[test]
fn ownership_seed_replay() {
    let seeds: [&[u8]; 3] = [
        include_bytes!("../../../verification/fuzz/corpus/ownership/move"),
        include_bytes!("../../../verification/fuzz/corpus/ownership/borrow"),
        include_bytes!("../../../verification/fuzz/corpus/ownership/branch"),
    ];
    let sources: Vec<_> = seeds
        .iter()
        .map(|seed| {
            replay(GuidedTarget::Ownership, seed);
            source_for(GuidedTarget::Ownership, seed)
        })
        .collect();
    assert!(sources.iter().any(|source| source.contains("list[int]")));
    assert!(sources.iter().any(|source| source.contains("print(len(")));
    assert!(sources.iter().any(|source| source.contains("if ")));
}
