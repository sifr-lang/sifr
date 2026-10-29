
use super::*;

#[test]
fn default_file_handle_writes_are_shared_and_close_is_mutable() {
    let mut ty = Type::Class {
        identity: Some(FILE_HANDLE_IDENTITY.to_string()),
        type_args: vec![],
        name: "FileHandle".to_string(),
        fields: vec![].into(),
        methods: ["write", "write_bytes", "close", "__exit__"]
            .into_iter()
            .map(|name| {
                (
                    name.to_string(),
                    FunctionType::all_borrow(vec![], Type::None),
                )
            })
            .collect(),
        parent_class: None,
    };
    fill_file_handle_receiver_conventions(&mut ty);
    let Type::Class { methods, .. } = ty else {
        unreachable!()
    };
    for (name, signature) in methods {
        let expected = if name == "close" || name == "__exit__" {
            ReceiverConvention::MutableBorrow
        } else {
            ReceiverConvention::SharedBorrow
        };
        assert_eq!(signature.receiver, Some(expected), "{name}");
    }
}

#[test]
fn explicit_imported_receiver_contract_is_preserved() {
    let mut ty = Type::Class {
        identity: Some(FILE_HANDLE_IDENTITY.to_string()),
        type_args: vec![],
        name: "FileHandle".to_string(),
        fields: vec![].into(),
        methods: vec![(
            "write".to_string(),
            FunctionType::all_borrow(vec![], Type::None)
                .with_receiver(ReceiverConvention::MutableBorrow),
        )]
        .into(),
        parent_class: None,
    };
    fill_file_handle_receiver_conventions(&mut ty);
    let Type::Class { methods, .. } = ty else {
        unreachable!()
    };
    assert_eq!(
        methods[0].1.receiver,
        Some(ReceiverConvention::MutableBorrow)
    );
}
