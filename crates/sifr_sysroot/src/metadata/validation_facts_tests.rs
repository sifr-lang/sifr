use super::validation_facts::ValidationFacts;
use super::*;

fn identity() -> Compatibility {
    Compatibility {
        compiler: [1; 32],
        semantic_target: [2; 32],
        stdlib_inputs: [3; 32],
    }
}
fn reference<T>(number: u8) -> Ref<T> {
    Ref {
        id: [number; 32],
        marker: PhantomData,
    }
}
fn chain(count: u8) -> Vec<u8> {
    let mut encoder = MetadataEncoder::new(identity(), Limits::default());
    encoder.insert(reference::<Type>(1), &Type::Int).unwrap();
    for number in 2..=count {
        encoder
            .insert(
                reference::<Type>(number),
                &Type::List(reference(number - 1)),
            )
            .unwrap();
    }
    encoder.finish().unwrap()
}
fn open(bytes: &[u8]) -> MetadataStore {
    MetadataStore::open_bytes(bytes.to_vec(), identity(), Limits::default()).unwrap()
}
fn complete(bytes: &[u8], budget: u64) -> (Result<()>, u64, usize, usize) {
    let store = open(bytes);
    let mut facts = ValidationFacts::new(budget);
    let result = store.validate_complete_with_facts(&mut facts);
    assert_eq!(store.decoded_count::<Type>(), 0);
    (
        result,
        store.payload_read_count::<Type>(),
        facts.hits,
        facts.evictions,
    )
}

#[test]
fn complete_reuses_facts_but_never_lazy_consumer_records() {
    let bytes = chain(30);
    let (uncached, reads, hits, _) = complete(&bytes, 0);
    let (cached, fewer_reads, cache_hits, evictions) = complete(&bytes, 1 << 20);
    assert_eq!(uncached, Ok(()));
    assert_eq!(cached, uncached);
    assert_eq!(hits, 0);
    assert!(cache_hits > 30);
    assert_eq!(evictions, 0);
    assert_eq!(fewer_reads, 30);
    assert!(reads > fewer_reads * 10);
    let store = open(&bytes);
    store.validate_complete().unwrap();
    let qualified_reads = store.payload_read_count::<Type>();
    assert_eq!(store.decoded_count::<Type>(), 0);
    assert_eq!(
        *store.get(reference::<Type>(30)).unwrap(),
        Type::List(reference(29))
    );
    assert_eq!(store.decoded_count::<Type>(), 1);
    assert!(store.payload_read_count::<Type>() > qualified_reads);
    let before_second_validation = store.payload_read_count::<Type>();
    store.validate_complete().unwrap();
    assert_eq!(
        store.payload_read_count::<Type>() - before_second_validation,
        30
    );
}

#[test]
fn eviction_and_oversized_facts_recompute_without_changing_results() {
    let bytes = chain(30);
    let (uncached, uncached_reads, _, _) = complete(&bytes, 0);
    let (evicted, reads, hits, evictions) = complete(&bytes, 700);
    assert_eq!(evicted, uncached);
    assert!(hits > 0 && evictions > 0 && reads > 30);
    let (oversized, oversized_reads, hits, evictions) = complete(&bytes, 1);
    assert_eq!(oversized, uncached);
    assert_eq!(oversized_reads, uncached_reads);
    assert_eq!((hits, evictions), (0, 0));
}

#[test]
fn each_root_keeps_depth_and_work_limits_after_earlier_success() {
    for (depth, work, error) in [
        (1, 64 * 1024 * 1024, "record graph exceeds depth limit"),
        (256, 15000, "demanded graph exceeds decode work budget"),
    ] {
        for budget in [0, 700, 1 << 20] {
            let mut store = open(&chain(3));
            store.limits.graph_depth = depth;
            store.limits.retained_bytes = work;
            let mut facts = ValidationFacts::new(budget);
            store
                .validate_graph_with_facts(reference::<Type>(2).id, None, &mut facts)
                .unwrap();
            assert_eq!(
                store.validate_graph_with_facts(reference::<Type>(3).id, None, &mut facts),
                Err(err(error)),
            );
            assert_eq!(
                store.validate_complete_with_facts(&mut facts),
                Err(err(error))
            );
        }
    }
}

fn corrupt_payload(bytes: &[u8], id: RecordId, mutation: impl FnOnce(&mut [u8])) -> Vec<u8> {
    let mut logical = super::physical::decode(bytes, identity(), Limits::default())
        .unwrap()
        .into_inner();
    let count = u32::from_le_bytes(logical[12..16].try_into().unwrap()) as usize;
    let entry = (0..count)
        .map(|i| super::container::HEADER_SIZE + i * super::container::ENTRY_SIZE)
        .find(|&offset| logical[offset..offset + 32] == id)
        .unwrap();
    let offset = u64::from_le_bytes(logical[entry + 36..entry + 44].try_into().unwrap()) as usize;
    let len = u64::from_le_bytes(logical[entry + 44..entry + 52].try_into().unwrap()) as usize;
    mutation(&mut logical[offset..offset + len]);
    let digest: [u8; 32] = Sha256::digest(&logical[offset..offset + len]).into();
    logical[entry + 60..entry + 92].copy_from_slice(&digest);
    super::physical::encode(&logical, Limits::default()).unwrap()
}

#[test]
fn exact_failure_diagnostics_survive_hits_misses_and_eviction() {
    let mut encoder = MetadataEncoder::new(identity(), Limits::default());
    encoder.insert(reference::<Type>(1), &Type::Int).unwrap();
    encoder
        .insert(reference::<Type>(2), &Type::List(reference(1)))
        .unwrap();
    encoder
        .insert(reference::<Type>(3), &Type::List(reference(3)))
        .unwrap();
    let cycle = encoder.finish().unwrap();
    let malformed = corrupt_payload(&chain(4), reference::<Type>(4).id, |payload| {
        payload[0] = b'!'
    });
    let missing = corrupt_payload(&chain(4), reference::<Type>(4).id, |payload| {
        let old = serde_json::to_vec(&Type::List(reference::<Type>(3))).unwrap();
        let new = serde_json::to_vec(&Type::List(reference::<Type>(99))).unwrap();
        assert_eq!(payload, old);
        payload.copy_from_slice(&new);
    });
    let mut encoder = MetadataEncoder::new(identity(), Limits::default());
    encoder.insert(reference::<Type>(1), &Type::Int).unwrap();
    encoder
        .insert(reference::<Type>(2), &Type::List(reference(1)))
        .unwrap();
    encoder
        .insert(
            reference::<Text>(3),
            &Text {
                value: "text".into(),
            },
        )
        .unwrap();
    let wrong_kind = corrupt_payload(
        &encoder.finish().unwrap(),
        reference::<Type>(2).id,
        |payload| {
            payload
                .copy_from_slice(&serde_json::to_vec(&Type::List(reference::<Type>(3))).unwrap());
        },
    );
    let mut encoder = MetadataEncoder::new(identity(), Limits::default());
    encoder.insert(reference::<Type>(1), &Type::Int).unwrap();
    encoder
        .insert(reference::<Type>(2), &Type::List(reference(1)))
        .unwrap();
    encoder
        .insert(
            reference::<SourceFile>(3),
            &SourceFile {
                relative_path: "../invalid.sifr".into(),
                content_digest: [0; 32],
                byte_length: 10,
            },
        )
        .unwrap();
    let bad_field = encoder.finish().unwrap();
    for (bytes, expected) in [
        (cycle, "cycle in structural type or semantic body records"),
        (malformed, "record decode:"),
        (missing, "missing or wrong-kind complete record reference"),
        (
            wrong_kind,
            "missing or wrong-kind complete record reference",
        ),
        (
            bad_field,
            "source path must be normalized and packaged-source relative",
        ),
    ] {
        let baseline = complete(&bytes, 0).0;
        assert!(baseline.as_ref().unwrap_err().0.contains(expected));
        for budget in [1, 700, 1 << 20] {
            assert_eq!(complete(&bytes, budget).0, baseline);
        }
    }
}

#[test]
fn captured_input_survives_file_mutation_but_new_store_rejects_it() {
    let bytes = chain(4);
    let path = std::env::temp_dir().join(format!(
        "sifr-metadata-facts-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    use std::io::{Seek, Write};
    file.write_all(&bytes).unwrap();
    file.rewind().unwrap();
    let original_time = file.metadata().unwrap().modified().unwrap();
    let captured =
        MetadataStore::open(file.try_clone().unwrap(), identity(), Limits::default()).unwrap();
    captured.validate_complete().unwrap();
    let mut corrupt = bytes.clone();
    let last = corrupt.len() - 1;
    corrupt[last] ^= 0xff;
    file.rewind().unwrap();
    file.write_all(&corrupt).unwrap();
    file.set_times(std::fs::FileTimes::new().set_modified(original_time))
        .unwrap();
    assert_eq!(file.metadata().unwrap().modified().unwrap(), original_time);
    captured.validate_complete().unwrap();
    file.rewind().unwrap();
    let reopened = MetadataStore::open(file, identity(), Limits::default());
    assert!(
        reopened
            .and_then(|store| store.validate_complete())
            .is_err()
    );
    std::fs::remove_file(path).unwrap();
}
