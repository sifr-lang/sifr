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
            assert!(replay(GuidedTarget::Lowering, seed));
            source_for(GuidedTarget::Lowering, seed)
        })
        .collect();
    assert_ne!(sources[0], sources[1]);
    assert_ne!(sources[1], sources[2]);
    assert!(sources.iter().any(|source| source.contains("if ")));
    assert!(sources.iter().any(|source| source.contains("for ")));
    assert!(sources.iter().any(|source| source.contains("while ")));
    assert!(sources.iter().any(|source| source.contains("add_0(")));
}

#[test]
fn ownership_seed_replay() {
    let moved = include_bytes!("../../../verification/fuzz/corpus/ownership/move");
    let borrowed = include_bytes!("../../../verification/fuzz/corpus/ownership/borrow");
    let branch = include_bytes!("../../../verification/fuzz/corpus/ownership/branch");
    let list_call = include_bytes!("../../../verification/fuzz/corpus/ownership/list_call");

    assert!(!replay(GuidedTarget::Ownership, moved));
    assert!(replay(GuidedTarget::Ownership, borrowed));
    assert!(!replay(GuidedTarget::Ownership, branch));
    assert!(replay(GuidedTarget::Ownership, list_call));

    let moved_source = source_for(GuidedTarget::Ownership, moved);
    let branch_source = source_for(GuidedTarget::Ownership, branch);
    let list_source = source_for(GuidedTarget::Ownership, list_call);
    assert!(moved_source.contains("moved_0 = owned_str\n    print(owned_str)"));
    assert!(branch_source.contains("if len(owned_str) > threshold:"));
    assert!(branch_source.contains("join_moved_0: str = owned_str"));
    assert!(branch_source.contains("    print(len(owned_str))"));
    assert!(list_source.contains("print(take_list_0(owned_list))"));
}
