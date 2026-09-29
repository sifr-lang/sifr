// H02a0 authority-carrier wire and container contract cases.

#[test]
fn h02a0_authority_record_roundtrip() {
    let mut encoder = MetadataEncoder::new(identity(), limits());
    let declaration = encoder
        .intern(&Text {
            value: "method".into(),
        })
        .unwrap();
    // The authority record references a typed callable declaration, not its name.
    let callable = encoder
        .intern(&CallableIdentity {
            module: declaration,
            owner: Some(declaration),
            symbol: declaration,
            generic_arguments: vec![],
            signature: declaration,
        })
        .unwrap();
    let mut cases = Vec::new();
    for authority in [
        MethodAuthority::Unclassified,
        MethodAuthority::BuiltinIntrinsic {
            declaration: callable,
        },
        MethodAuthority::Protocol {
            declaration: callable,
        },
        MethodAuthority::LocalNominal {
            declaration: callable,
        },
        MethodAuthority::InheritedNominal {
            declaration: callable,
        },
        MethodAuthority::Imported {
            declaration: callable,
        },
        MethodAuthority::RustAdapted {
            declaration: callable,
        },
    ] {
        let reference = encoder.intern(&authority).unwrap();
        cases.push((reference, authority));
    }
    let store = store(encoder);
    for (reference, expected) in cases {
        assert_eq!(*store.get(reference).unwrap(), expected);
    }
}

#[test]
fn h02a0_old_version_and_invalid_authority_reject() {
    let valid = encoder().finish().unwrap();
    assert_eq!(&valid[8..12], &5_u32.to_le_bytes());
    let mut old = valid.clone();
    old[8..12].copy_from_slice(&4_u32.to_le_bytes());
    assert!(MetadataStore::open(Cursor::new(old), identity(), limits()).is_err());
    let mut logical = super::physical::decode(&valid, identity(), limits())
        .unwrap()
        .into_inner();
    let count = u32::from_le_bytes(logical[12..16].try_into().unwrap()) as usize;
    let mut corrupted = false;
    for index in 0..count {
        let entry = super::container::HEADER_SIZE + index * super::container::ENTRY_SIZE;
        let kind = u16::from_le_bytes(logical[entry + 32..entry + 34].try_into().unwrap());
        if kind != MethodAuthority::KIND {
            continue;
        }
        let offset =
            u64::from_le_bytes(logical[entry + 36..entry + 44].try_into().unwrap()) as usize;
        let len = u64::from_le_bytes(logical[entry + 44..entry + 52].try_into().unwrap()) as usize;
        assert_eq!(&logical[offset..offset + len], b"\"Unclassified\"");
        logical[offset + len - 2] = b'x';
        let digest: [u8; 32] = Sha256::digest(&logical[offset..offset + len]).into();
        logical[entry + 60..entry + 92].copy_from_slice(&digest);
        corrupted = true;
        break;
    }
    assert!(corrupted);
    let forged = super::physical::encode(&logical, limits()).unwrap();
    let store = MetadataStore::open(Cursor::new(forged), identity(), limits()).unwrap();
    assert!(store.validate_complete().is_err());
    for value in [
        r#""FutureAuthority""#,
        r#"{"Protocol":{}}"#,
        r#"{"Imported":{"declaration":null}}"#,
        r#"{"Unclassified":{"unexpected":true}}"#,
    ] {
        assert!(
            serde_json::from_str::<MethodAuthority>(value).is_err(),
            "{value}"
        );
    }
}
