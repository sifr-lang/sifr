use super::complete_graph::CompleteGraph;
use super::validation_facts::ValidationFacts;
use super::*;

pub(super) fn original_complete(store: &MetadataStore) -> Result<()> {
    // Original ID-based traversal remains the lazy consumer's production path.
    // Avoid any positional fact cache in this independent differential oracle.
    for (id, entry) in &store.directory {
        let bytes = store.read_payload(entry)?;
        let references = super::decode::references(store, entry.kind, &bytes)?;
        for (reference, kind) in references {
            if store
                .directory
                .get(&reference)
                .is_none_or(|entry| entry.kind != kind)
            {
                return Err(err("missing or wrong-kind complete record reference"));
            }
        }
        store.validate_graph(*id, &bytes)?;
    }
    Ok(())
}
fn identity() -> Compatibility {
    Compatibility {
        compiler: [1; 32],
        semantic_target: [2; 32],
        stdlib_inputs: [3; 32],
    }
}
fn reference(number: usize) -> Ref<Type> {
    Ref::anchor(&[&number.to_le_bytes()])
}
fn graph(seed: u64) -> Vec<u8> {
    let mut encoder = MetadataEncoder::new(identity(), Limits::default());
    let mut random = seed + 1;
    for number in 0..16 {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        let value = if number == 0 {
            Type::Int
        } else {
            // Even seeds form shared DAGs, odd seeds may include cycles/forward
            // edges. Duplicate and differently ordered paths retain DFS semantics.
            let span = if seed % 2 == 0 { number } else { 16 };
            let left = (random as usize) % span;
            let right = ((random >> 32) as usize) % span;
            Type::Tuple(vec![reference(left), reference(right), reference(left)])
        };
        encoder.insert(reference(number), &value).unwrap();
    }
    encoder.finish().unwrap()
}
#[test]
fn positional_complete_walk_preserves_original_verdicts_and_limits() {
    for seed in 0..64 {
        let bytes = graph(seed);
        for depth in [0, 1, 3, 256] {
            for work in [8_000, 30_000, 64 * 1024 * 1024] {
                let mut store =
                    MetadataStore::open_bytes(bytes.clone(), identity(), Limits::default())
                        .unwrap();
                store.limits.graph_depth = depth;
                store.limits.retained_bytes = work;
                let expected = original_complete(&store);
                for budget in [0, 700, 1 << 20] {
                    assert_eq!(
                        store.validate_complete_with_facts(&mut ValidationFacts::new(budget)),
                        expected,
                        "seed={seed}, depth={depth}, work={work}, budget={budget}"
                    );
                }
            }
        }
    }
}
#[test]
fn complete_walk_generation_wrap_clears_old_colors_and_checks_again() {
    let bytes = graph(2);
    let store = MetadataStore::open_bytes(bytes, identity(), Limits::default()).unwrap();
    let mut graph = CompleteGraph::new(store.directory.0.len());
    let mut facts = ValidationFacts::new(1 << 20);
    for index in 0..store.directory.0.len() {
        graph.validate(&store, index, &mut facts).unwrap();
    }
    graph.force_generation_wrap();
    for index in 0..store.directory.0.len() {
        graph.validate(&store, index, &mut facts).unwrap();
    }
    assert_eq!(original_complete(&store), Ok(()));
}
