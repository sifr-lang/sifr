use super::non_ipc_serializable_reason;
use sifr_type_system::{FunctionType, Type};

#[test]
fn accepts_initial_payload_families() {
    let record = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "EchoRequest".to_string(),
        fields: vec![
            ("message".to_string(), Type::Str),
            ("attempts".to_string(), Type::Int),
            ("tags".to_string(), Type::List(Box::new(Type::Str))),
        ]
        .into(),
        methods: vec![].into(),
        parent_class: None,
    };
    let payloads = [
        Type::Bool,
        Type::Int,
        Type::Float,
        Type::Str,
        Type::Bytes,
        Type::None,
        Type::Union(vec![Type::None, Type::Int]),
        Type::List(Box::new(Type::Int)),
        Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        Type::Tuple(vec![Type::Int, Type::Str, Type::Bytes]),
        Type::Enum {
            identity: None,
            name: "Color".to_string(),
            variants: vec![("RED".to_string(), Some(1)), ("BLUE".to_string(), Some(2))].into(),
        },
        record,
    ];

    for payload in payloads {
        assert_eq!(non_ipc_serializable_reason(&payload), None);
    }
}

#[test]
fn rejects_process_local_and_callable_payloads() {
    let process_reader = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "PipeReader".to_string(),
        fields: vec![("_handle".to_string(), Type::Int)].into(),
        methods: vec![].into(),
        parent_class: None,
    };
    let callable = Type::Callable(vec![Type::Int], vec![], Box::new(Type::Int));
    let function = Type::Function(FunctionType::new(
        vec![("value".to_string(), Type::Int)],
        Type::Int,
    ));

    assert!(
        non_ipc_serializable_reason(&process_reader)
            .is_some_and(|reason| reason.contains("process-local process pipe handle"))
    );
    assert!(
        non_ipc_serializable_reason(&callable)
            .is_some_and(|reason| reason.contains("callables are process-local code"))
    );
    assert!(
        non_ipc_serializable_reason(&function)
            .is_some_and(|reason| reason.contains("callables are process-local code"))
    );
}
