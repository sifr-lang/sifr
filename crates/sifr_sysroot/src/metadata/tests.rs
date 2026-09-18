use super::*;
use std::io::Cursor;
use std::sync::Arc;
include!("test_dispatch.rs");

fn identity() -> Compatibility {
    Compatibility {
        compiler: [1; 32],
        semantic_target: [2; 32],
        stdlib_inputs: [3; 32],
    }
}
fn limits() -> Limits {
    Limits::default()
}
fn fixture<T: Record>() -> Ref<T> {
    Ref::anchor(&[b"fixture"])
}
fn roundtrip<T: Record + PartialEq>(bytes: &[u8]) -> Vec<u8> {
    assert!(
        std::mem::size_of::<T>() + 256
            <= usize::try_from(super::container::RECORD_FIXED_BOUND).unwrap()
    );
    let value: T = serde_json::from_slice(bytes).expect("decode explicit schema case");
    let encoded = serde_json::to_vec(&value).expect("encode explicit schema case");
    let decoded: T = serde_json::from_slice(&encoded).expect("roundtrip case");
    assert_eq!(value, decoded);
    encoded
}
fn insert<T: Record>(encoder: &mut MetadataEncoder, bytes: &[u8]) {
    encoder
        .insert(fixture::<T>(), &serde_json::from_slice::<T>(bytes).unwrap())
        .unwrap();
}
fn get<T: Record>(store: &MetadataStore) -> Vec<u8> {
    serde_json::to_vec(store.get(fixture::<T>()).unwrap().as_ref()).unwrap()
}
fn cases() -> Vec<serde_json::Value> {
    serde_json::from_str(include_str!("record_cases.json")).unwrap()
}
fn encoder() -> MetadataEncoder {
    let mut encoder = MetadataEncoder::new(identity(), limits());
    let mut seen = BTreeSet::new();
    for case in cases() {
        let kind = u16::try_from(case["kind"].as_u64().unwrap()).unwrap();
        if seen.insert(kind) {
            insert_default(
                &mut encoder,
                kind,
                &serde_json::to_vec(&case["value"]).unwrap(),
            );
        }
    }
    assert_eq!(seen.len(), KIND_COUNT as usize);
    encoder
}
fn store(encoder: MetadataEncoder) -> MetadataStore {
    MetadataStore::open(Cursor::new(encoder.finish().unwrap()), identity(), limits()).unwrap()
}

#[test]
fn all_explicit_record_families_and_variants_roundtrip() {
    let mut kinds = BTreeSet::new();
    let cases = cases();
    assert!(cases.len() > 250);
    for case in &cases {
        let kind = u16::try_from(case["kind"].as_u64().unwrap()).unwrap();
        kinds.insert(kind);
        let bytes = serde_json::to_vec(&case["value"]).unwrap();
        let _ = roundtrip_case(kind, &bytes);
    }
    assert_eq!(kinds.len(), KIND_COUNT as usize);
    let store = store(encoder());
    let mut seen = BTreeSet::new();
    for case in cases {
        let kind = u16::try_from(case["kind"].as_u64().unwrap()).unwrap();
        if seen.insert(kind) {
            let expected = roundtrip_case(kind, &serde_json::to_vec(&case["value"]).unwrap());
            assert_eq!(get_default(&store, kind), expected, "{}", case["name"]);
        }
    }
}

#[test]
fn i05_generated_record_permutations_have_identical_payload_hashes() {
    let defaults = cases().into_iter().fold(BTreeMap::new(), |mut m, case| {
        m.entry(u16::try_from(case["kind"].as_u64().unwrap()).unwrap())
            .or_insert(case["value"].clone());
        m
    });
    let expected = encoder().finish().unwrap();
    for seed in 0..32_u64 {
        let mut order = defaults.keys().copied().collect::<Vec<_>>();
        let mut state = seed + 1;
        for i in (1..order.len()).rev() {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            order.swap(i, usize::try_from(state % (i + 1) as u64).unwrap());
        }
        let mut candidate = MetadataEncoder::new(identity(), limits());
        for kind in order {
            insert_default(
                &mut candidate,
                kind,
                &serde_json::to_vec(&defaults[&kind]).unwrap(),
            );
        }
        let bytes = candidate.finish().unwrap();
        assert_eq!(Sha256::digest(&bytes), Sha256::digest(&expected));
    }
}

#[test]
fn i06_nominal_package_version_source_and_binder_anchors_do_not_alias() {
    let a = Ref::<Declaration>::anchor(&[b"pkg", b"1", b"source-a", b"module", b"Class"]);
    let b = Ref::<Declaration>::anchor(&[b"pkg", b"2", b"source-a", b"module", b"Class"]);
    let c = Ref::<Declaration>::anchor(&[b"other", b"1", b"source-a", b"module", b"Class"]);
    let d = Ref::<Declaration>::anchor(&[b"pkg", b"1", b"source-b", b"module", b"Class"]);
    assert_eq!(BTreeSet::from([a, b, c, d]).len(), 4);
    assert_ne!(
        Ref::<Declaration>::anchor(&[b"ab", b"c"]),
        Ref::<Declaration>::anchor(&[b"a", b"bc"])
    );
    let mut encoded = encoder();
    let mut binder = serde_json::from_value::<Binder>(
        cases().into_iter().find(|v| v["name"] == "Binder").unwrap()["value"].clone(),
    )
    .unwrap();
    let first = encoded.intern(&binder).unwrap();
    binder.nested_path = vec![1];
    let second = encoded.intern(&binder).unwrap();
    let t1 = encoded
        .intern(&Type::TypeVar {
            binder: first,
            slot: 0,
        })
        .unwrap();
    let t2 = encoded
        .intern(&Type::TypeVar {
            binder: second,
            slot: 0,
        })
        .unwrap();
    assert_ne!(t1, t2);
    let store = store(encoded);
    assert_ne!(store.get(t1).unwrap(), store.get(t2).unwrap());
}

fn class_scaling(signatures: usize) -> (usize, usize, u64) {
    let mut encoded = encoder();
    let text = fixture::<Text>();
    let primitive = fixture::<Type>();
    let mut fields = Vec::new();
    for i in 0..500 {
        let name = encoded
            .intern(&Text {
                value: format!("field_{i}"),
            })
            .unwrap();
        fields.push((name, primitive));
    }
    let full = encoded
        .intern(&NominalView {
            name: text,
            identity: None,
            fields,
            methods: Vec::new(),
            parent_class: None,
            enum_variants: Vec::new(),
            newtype_inner: None,
        })
        .unwrap();
    let partial = encoded
        .intern(&NominalView {
            name: text,
            identity: None,
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
            enum_variants: Vec::new(),
            newtype_inner: None,
        })
        .unwrap();
    let ty = encoded
        .intern(&Type::Class {
            declaration: fixture::<Declaration>(),
            view: full,
            type_args: Vec::new(),
        })
        .unwrap();
    let other = encoded
        .intern(&Type::Class {
            declaration: fixture::<Declaration>(),
            view: partial,
            type_args: Vec::new(),
        })
        .unwrap();
    assert_ne!(ty, other);
    let mut functions = Vec::new();
    for i in 0..signatures {
        let name = encoded
            .intern(&Text {
                value: format!("arg_{i}"),
            })
            .unwrap();
        functions.push(
            encoded
                .intern(&FunctionType {
                    receiver: None,
                    params: vec![(name, ty, fixture::<ParamConvention>())],
                    return_type: ty,
                })
                .unwrap(),
        );
    }
    let bytes = encoded.finish().unwrap();
    let size = bytes.len();
    let store = MetadataStore::open(Cursor::new(bytes), identity(), limits()).unwrap();
    assert_eq!(store.retained_records().unwrap(), 0);
    let handle = store.get(ty).unwrap();
    assert!(Arc::ptr_eq(&handle, &store.get(ty).unwrap()));
    for function in functions {
        let signature = store.get(function).unwrap();
        assert_eq!(signature.return_type, ty);
    }
    // Lookup of thousands of signatures does not retain the large declaration view.
    assert_eq!(store.retained_records().unwrap(), signatures + 1);
    let retained = store.retained_bound().unwrap();
    let count = store.retained_records().unwrap();
    assert_eq!(store.get(full).unwrap().fields.len(), 500);
    assert!(store.get(partial).unwrap().fields.is_empty());
    (size, count, retained)
}
#[test]
#[allow(clippy::print_stderr)]
fn m16_shared_large_class_retention_is_unique_records_plus_references() {
    let small = class_scaling(10);
    let large = class_scaling(1000);
    eprintln!(
        "M16 (encoded bytes, retained records, retained bound bytes): 10 signatures {small:?}; 1000 signatures {large:?}"
    );
    assert!(large.0 - small.0 < 1_000_000);
    assert_eq!(large.1 - small.1, 990);
    assert!(large.2 - small.2 < 20_000_000);
}

#[test]
fn m16_nominal_recursion_terminates_but_structural_cycles_and_excess_depth_reject() {
    let mut encoded = encoder();
    let recursive = Ref::<Type>::anchor(&[b"recursive"]);
    let view = Ref::<NominalView>::anchor(&[b"recursive-view"]);
    encoded
        .insert(
            recursive,
            &Type::Class {
                declaration: fixture::<Declaration>(),
                view,
                type_args: Vec::new(),
            },
        )
        .unwrap();
    encoded
        .insert(
            view,
            &NominalView {
                name: fixture::<Text>(),
                identity: None,
                fields: vec![(fixture::<Text>(), recursive)],
                methods: Vec::new(),
                parent_class: None,
                enum_variants: Vec::new(),
                newtype_inner: None,
            },
        )
        .unwrap();
    let good = store(encoded);
    assert!(good.get(recursive).is_ok());
    assert!(good.get(view).is_ok());
    let mut encoded = encoder();
    let cycle = Ref::<Type>::anchor(&[b"invalid-cycle"]);
    encoded.insert(cycle, &Type::List(cycle)).unwrap();
    assert!(store(encoded).get(cycle).unwrap_err().0.contains("cycle"));
    let mut encoded = encoder();
    let mut last = fixture::<Type>();
    for _ in 0..300 {
        last = encoded.intern(&Type::List(last)).unwrap();
    }
    assert!(store(encoded).get(last).unwrap_err().0.contains("depth"));
}

#[test]
fn m06_header_index_payload_and_reference_fuzz_seeds_fail_closed() {
    let original = encoder().finish().unwrap();
    for end in [0, 7, 8, 119, 120, original.len() - 1] {
        assert!(
            MetadataStore::open(Cursor::new(original[..end].to_vec()), identity(), limits())
                .is_err()
        );
    }
    for offset in [0, 8, 12, 16, 48, 80, 112, 120] {
        let mut bytes = original.clone();
        bytes[offset] ^= 0xff;
        assert!(MetadataStore::open(Cursor::new(bytes), identity(), limits()).is_err());
    }
    let logical = super::physical::decode(&original, identity(), limits())
        .unwrap()
        .into_inner();
    for offset in [120 + 32, 120 + 34, 120 + 36, 120 + 44, 120 + 52] {
        let mut bytes = logical.clone();
        bytes[offset] ^= 0xff;
        let bytes = super::physical::encode(&bytes, limits()).unwrap();
        assert!(
            MetadataStore::open(Cursor::new(bytes), identity(), limits()).is_err(),
            "offset {offset}"
        );
    }
    let mut bytes = logical;
    let count = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let start = 120 + count * 92;
    bytes[start] ^= 1;
    let bytes = super::physical::encode(&bytes, limits()).unwrap();
    let store = MetadataStore::open(Cursor::new(bytes), identity(), limits()).unwrap();
    let first = store.directory.iter().next().unwrap();
    assert!(store.read_payload(first.1).is_err());
    let mut encoded = encoder();
    let missing = Ref::<Type>::anchor(&[b"missing"]);
    encoded
        .insert(Ref::<Type>::anchor(&[b"bad"]), &Type::List(missing))
        .unwrap();
    assert!(encoded.finish().is_err());
    for raw in [
        br#""00""#.as_slice(),
        br#"[]"#.as_slice(),
        br#"null"#.as_slice(),
    ] {
        assert!(serde_json::from_slice::<Ref<Type>>(raw).is_err());
    }
}

#[test]
fn m06_source_ranges_binders_and_fragment_assurance_are_checked() {
    let mut encoded = encoder();
    let bad = encoded
        .intern(&SourceFile {
            relative_path: "../producer/secret.sifr".into(),
            content_digest: [0; 32],
            byte_length: 10,
        })
        .unwrap();
    assert!(store(encoded).get(bad).is_err());
    let mut encoded = encoder();
    let bad = encoded
        .intern(&SourceLocation {
            source: fixture::<SourceFile>(),
            start: 10,
            end: 1000,
            documentation: None,
        })
        .unwrap();
    assert!(store(encoded).get(bad).is_err());
    let mut encoded = encoder();
    let bad = encoded
        .intern(&Type::TypeVar {
            binder: fixture::<Binder>(),
            slot: 10,
        })
        .unwrap();
    assert!(store(encoded).get(bad).is_err());
    let mut encoded = encoder();
    let bad_source = encoded
        .intern(&Text {
            value: "invalidated fragment".into(),
        })
        .unwrap();
    let bad = encoded
        .intern(&RustPayload {
            module: fixture::<Module>(),
            source: bad_source,
            names: BTreeMap::new(),
            mappings: Vec::new(),
            validation: fixture::<FragmentValidation>(),
            generators: BTreeSet::new(),
        })
        .unwrap();
    assert!(store(encoded).get(bad).is_err());
}

#[test]
fn generated_valid_type_graph_property_cases_preserve_references() {
    for seed in 1..=32_u64 {
        let mut encoder = encoder();
        let mut types = vec![fixture::<Type>()];
        let mut state = seed;
        for i in 0..64 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let a = types[usize::try_from(state % types.len() as u64).unwrap()];
            let b = types[usize::try_from((state >> 32) % types.len() as u64).unwrap()];
            let value = match i % 4 {
                0 => Type::List(a),
                1 => Type::Dict(a, b),
                2 => Type::Tuple(vec![a, b]),
                _ => Type::LiteralInt(i64::from_ne_bytes(state.to_ne_bytes())),
            };
            types.push(encoder.intern(&value).unwrap());
        }
        let store = store(encoder);
        for reference in types {
            let first = store.get(reference).unwrap();
            assert!(Arc::ptr_eq(&first, &store.get(reference).unwrap()));
        }
    }
}

#[test]
fn i05_indexed_file_relocation_and_separate_store_owners_preserve_identity() {
    let root = std::env::temp_dir().join(format!(
        "sifr-dx5-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let first = root.join("first.sifrmeta");
    let second = root.join("moved.sifrmeta");
    let bytes = encoder().finish().unwrap();
    std::fs::write(&first, &bytes).unwrap();
    std::fs::rename(&first, &second).unwrap();
    let a =
        MetadataStore::open(std::fs::File::open(&second).unwrap(), identity(), limits()).unwrap();
    let b = MetadataStore::open(Cursor::new(bytes), identity(), limits()).unwrap();
    let x = a.get(fixture::<SourceFile>()).unwrap();
    let y = b.get(fixture::<SourceFile>()).unwrap();
    assert_eq!(x, y);
    assert!(!Arc::ptr_eq(&x, &y));
    drop(a);
    std::fs::remove_file(second).unwrap();
    std::fs::remove_dir(root).unwrap();
}

fn replace_payload(bytes: Vec<u8>, id: RecordId, replacement: impl FnOnce(&mut [u8])) -> Vec<u8> {
    let mut bytes = super::physical::decode(&bytes, identity(), limits())
        .unwrap()
        .into_inner();
    let count = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let index = (0..count)
        .map(|i| 120 + i * 92)
        .find(|i| bytes[*i..*i + 32] == id)
        .unwrap();
    let start = usize::try_from(u64::from_le_bytes(
        bytes[index + 36..index + 44].try_into().unwrap(),
    ))
    .unwrap();
    let len = usize::try_from(u64::from_le_bytes(
        bytes[index + 44..index + 52].try_into().unwrap(),
    ))
    .unwrap();
    replacement(&mut bytes[start..start + len]);
    let digest = Sha256::digest(&bytes[start..start + len]);
    bytes[index + 60..index + 92].copy_from_slice(&digest);
    super::physical::encode(&bytes, limits()).unwrap()
}
#[test]
fn m06_checks_corrupt_reference_ids_even_when_payload_digest_matches() {
    let mut encoder = encoder();
    let root = encoder.intern(&Type::List(fixture::<Type>())).unwrap();
    let bytes = encoder.finish().unwrap();
    for wrong in [
        Ref::<Type>::anchor(&[b"absent"]).id(),
        fixture::<Text>().id(),
    ] {
        let text = id_text(&wrong);
        let bytes = replace_payload(bytes.clone(), root.id(), |payload| {
            let start = payload
                .windows(64)
                .position(|p| p.iter().all(u8::is_ascii_hexdigit))
                .unwrap();
            payload[start..start + 64].copy_from_slice(text.as_bytes());
        });
        let store = MetadataStore::open(Cursor::new(bytes), identity(), limits()).unwrap();
        assert!(store.get(root).is_err());
    }
}
#[test]
fn m06_mutation_corpus_never_panics_or_allocates_from_unchecked_lengths() {
    let original = encoder().finish().unwrap();
    let mut state = 7_u64;
    for _ in 0..128 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let mut bytes = original.clone();
        let pos = usize::try_from(state % bytes.len() as u64).unwrap();
        bytes[pos] ^= state.to_le_bytes()[4] | 1;
        let result = std::panic::catch_unwind(|| {
            if let Ok(store) = MetadataStore::open(Cursor::new(bytes), identity(), limits()) {
                let _ = store.get(fixture::<SemanticExports>());
                let _ = store.get(fixture::<Type>());
            }
        });
        assert!(result.is_ok());
    }
    let mut small = limits();
    small.record_bytes = 1;
    assert!(MetadataStore::open(Cursor::new(original.clone()), identity(), small).is_err());
    small = limits();
    small.retained_bytes = 1;
    assert!(MetadataStore::open(Cursor::new(original), identity(), small).is_err());
}

fn default_record<T: Record>() -> T {
    let case = cases()
        .into_iter()
        .find(|case| case["kind"].as_u64() == Some(u64::from(T::KIND)))
        .unwrap();
    serde_json::from_value(case["value"].clone()).unwrap()
}
#[test]
fn i06_encoded_same_name_declarations_keep_package_version_and_source_identity() {
    let mut encoder = encoder();
    let mut types = Vec::new();
    for (version, source) in [("1", [1; 32]), ("2", [1; 32]), ("1", [2; 32])] {
        let version = encoder
            .intern(&Text {
                value: version.into(),
            })
            .unwrap();
        let package = encoder
            .intern(&Package {
                name: fixture::<Text>(),
                version,
                source_identity: source,
            })
            .unwrap();
        let module = Ref::<Module>::anchor(&[b"module", &package.id()]);
        let declaration = Ref::<Declaration>::anchor(&[b"declaration", &package.id()]);
        let mut module_value = default_record::<Module>();
        module_value.package = package;
        module_value
            .declarations
            .insert(fixture::<Text>(), declaration);
        module_value.exports.insert(fixture::<Text>(), declaration);
        encoder.insert(module, &module_value).unwrap();
        let mut value = default_record::<Declaration>();
        value.module = module;
        encoder.insert(declaration, &value).unwrap();
        types.push(
            encoder
                .intern(&Type::Class {
                    declaration,
                    view: fixture::<NominalView>(),
                    type_args: Vec::new(),
                })
                .unwrap(),
        );
    }
    let store = store(encoder);
    let decoded = types
        .into_iter()
        .map(|reference| store.get(reference).unwrap())
        .collect::<Vec<_>>();
    assert_ne!(decoded[0], decoded[1]);
    assert_ne!(decoded[0], decoded[2]);
    let mut packages = BTreeSet::new();
    for ty in decoded {
        let Type::Class { declaration, .. } = ty.as_ref() else {
            panic!("expected class")
        };
        let declaration = store.get(*declaration).unwrap();
        let module = store.get(declaration.module).unwrap();
        packages.insert(module.package);
    }
    assert_eq!(packages.len(), 3);
}

#[test]
fn m16_mutually_recursive_nominal_views_stay_lazy() {
    let mut encoder = encoder();
    let a = Ref::<Type>::anchor(&[b"mutual-a"]);
    let b = Ref::<Type>::anchor(&[b"mutual-b"]);
    let av = Ref::<NominalView>::anchor(&[b"view-a"]);
    let bv = Ref::<NominalView>::anchor(&[b"view-b"]);
    for (ty, view, other) in [(a, av, b), (b, bv, a)] {
        encoder
            .insert(
                ty,
                &Type::Class {
                    declaration: fixture::<Declaration>(),
                    view,
                    type_args: Vec::new(),
                },
            )
            .unwrap();
        let mut value = default_record::<NominalView>();
        value.fields = vec![(fixture::<Text>(), other)];
        encoder.insert(view, &value).unwrap();
    }
    let store = store(encoder);
    assert!(store.get(a).is_ok());
    assert!(store.get(b).is_ok());
    assert_eq!(store.retained_records().unwrap(), 2);
    assert_eq!(store.get(av).unwrap().fields[0].1, b);
    assert_eq!(store.get(bv).unwrap().fields[0].1, a);
}

#[test]
fn bounded_retention_evicts_only_unreferenced_records() {
    let mut encoder = encoder();
    let mut refs = Vec::new();
    for i in 0..80 {
        refs.push(
            encoder
                .intern(&Text {
                    value: format!("text {i}"),
                })
                .unwrap(),
        );
    }
    let bytes = encoder.finish().unwrap();
    let logical = super::physical::decode(&bytes, identity(), limits())
        .unwrap()
        .into_inner();
    let count = u32::from_le_bytes(logical[12..16].try_into().unwrap()) as usize;
    let maximum = (0..count)
        .map(|i| {
            u64::from_le_bytes(
                logical[120 + i * 92 + 52..120 + i * 92 + 60]
                    .try_into()
                    .unwrap(),
            )
        })
        .max()
        .unwrap();
    let mut limited = limits();
    limited.retained_bytes = maximum + 10_000;
    let store = MetadataStore::open(Cursor::new(bytes), identity(), limited).unwrap();
    let pinned = store.get(refs[0]).unwrap();
    for reference in &refs[1..] {
        let _ = store.get(*reference).unwrap();
    }
    assert!(store.retained_records().unwrap() < refs.len());
    assert!(Arc::ptr_eq(&pinned, &store.get(refs[0]).unwrap()));
    assert!(store.retained_bound().unwrap() <= limited.retained_bytes);
    let mut pinned_records = Vec::new();
    let mut exhausted = false;
    for reference in &refs[1..] {
        match store.get(*reference) {
            Ok(record) => pinned_records.push(record),
            Err(error) => {
                assert!(error.0.contains("active decoded handles"));
                exhausted = true;
                break;
            }
        }
    }
    assert!(exhausted);
    drop(pinned_records);
    assert!(store.get(refs[79]).is_ok());
}

#[test]
fn dx6_class_shaped_constructor_views_preserve_declared_nominal_kinds() {
    for (kind, valid) in [
        (DeclarationKind::Class, true),
        (DeclarationKind::Enum, true),
        (DeclarationKind::Newtype, true),
        (DeclarationKind::Protocol, true),
        (DeclarationKind::Function, false),
        (DeclarationKind::Alias, false),
    ] {
        let mut encoder = encoder();
        let kind = encoder.intern(&kind).unwrap();
        let declaration = encoder
            .intern(&Declaration {
                module: fixture(),
                symbol: fixture(),
                kind,
                binder: None,
                full_view: None,
                location: fixture(),
            })
            .unwrap();
        let occurrence = encoder
            .intern(&Type::Class {
                declaration,
                view: fixture(),
                type_args: Vec::new(),
            })
            .unwrap();
        let store = store(encoder);
        assert_eq!(store.get(occurrence).is_ok(), valid);
    }
}

#[test]
fn dx8_portable_digest_preserves_semantic_payload_changes() {
    let digest = |text: &str| {
        let mut encoder = MetadataEncoder::new(identity(), limits());
        encoder
            .intern(&Text {
                value: text.to_owned(),
            })
            .unwrap();
        let bytes = encoder.finish().unwrap();
        MetadataStore::open(Cursor::new(bytes), identity(), limits())
            .unwrap()
            .portable_payload_digest()
            .unwrap()
    };
    assert_ne!(digest("declaration-a"), digest("declaration-b"));
    assert_eq!(digest("declaration-a"), digest("declaration-a"));
}

#[test]
fn dx15_physical_frames_are_bounded_canonical_and_versioned() {
    let original = encoder().finish().unwrap();
    for expanded in [0_u64, 119, u64::MAX, limits().file_bytes + 1] {
        let mut bytes = original.clone();
        bytes[120..128].copy_from_slice(&expanded.to_le_bytes());
        assert!(MetadataStore::open(Cursor::new(bytes), identity(), limits()).is_err());
    }
    for suffix in [vec![0], original[128..].to_vec()] {
        let mut bytes = original.clone();
        bytes.extend_from_slice(&suffix);
        let size = bytes.len() as u64;
        bytes[112..120].copy_from_slice(&size.to_le_bytes());
        assert!(MetadataStore::open(Cursor::new(bytes), identity(), limits()).is_err());
    }
    let mut old = original.clone();
    old[8..12].copy_from_slice(&1_u32.to_le_bytes());
    assert!(MetadataStore::open(Cursor::new(old), identity(), limits()).is_err());
    let logical = super::physical::decode(&original, identity(), limits())
        .unwrap()
        .into_inner();
    assert_eq!(
        super::physical::encode(&logical, limits()).unwrap(),
        original
    );
    let mut undersized = original.clone();
    undersized[120..128].copy_from_slice(&((logical.len() - 1) as u64).to_le_bytes());
    assert!(MetadataStore::open(Cursor::new(undersized), identity(), limits()).is_err());
    let mut oversized = original;
    oversized[120..128].copy_from_slice(&((logical.len() + 1) as u64).to_le_bytes());
    assert!(MetadataStore::open(Cursor::new(oversized), identity(), limits()).is_err());
}

#[test]
fn dx15_self_contained_distribution_fixture_uses_a_valid_raw_zstd_frame() {
    let encoded = MetadataEncoder::new(identity(), limits()).finish().unwrap();
    let logical = super::physical::decode(&encoded, identity(), limits())
        .unwrap()
        .into_inner();
    assert_eq!(logical.len(), 120);
    let mut frame = vec![0x28, 0xb5, 0x2f, 0xfd, 0x20, 0x78, 0xc1, 0x03, 0x00];
    frame.extend_from_slice(&logical);
    frame.extend_from_slice(&[0x28, 0xb5, 0x2f, 0xfd, 0x20, 0x00, 0x01, 0x00, 0x00]);
    let mut fixture = logical[..112].to_vec();
    fixture.extend_from_slice(&((128 + frame.len()) as u64).to_le_bytes());
    fixture.extend_from_slice(&120_u64.to_le_bytes());
    fixture.extend_from_slice(&frame);
    let store = MetadataStore::open_bytes(fixture, identity(), limits()).unwrap();
    assert_eq!(store.retained_records().unwrap(), 0);
}

#[test]
fn dx15_payload_frame_is_demand_loaded_once_without_decoding_unrequested_records() {
    let store = store(encoder());
    assert_eq!(store.physical_payload_decode_us(), 0);
    assert_eq!(store.retained_records().unwrap(), 0);
    let first = get::<SemanticExports>(&store);
    let elapsed = store.physical_payload_decode_us();
    assert!(elapsed > 0);
    assert_eq!(store.retained_records().unwrap(), 1);
    assert_eq!(store.decoded_count::<HirModule>(), 0);
    assert_eq!(first, get::<SemanticExports>(&store));
    assert_eq!(store.physical_payload_decode_us(), elapsed);
}
