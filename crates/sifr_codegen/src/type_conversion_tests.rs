use crate::{
    render_type, sifr_type_to_rust_field_type, sifr_type_to_rust_type, try_sifr_type_to_rust_type,
    RustTrait, RustType,
};
use sifr_type_system::{FixedIntType, FunctionType, ParamConvention, PythonArrowKind, Type};
use std::collections::HashMap;

fn assert_named_nodes_are_leaf_paths(ty: &RustType) {
    match ty {
        RustType::Named(name) => {
            assert!(
                !name.contains(['<', '>', '[', ']']),
                "non-leaf Rust name: {name}"
            );
            assert!(!name.starts_with("dyn ") && !name.starts_with("impl "));
        }
        RustType::Vec(inner)
        | RustType::HashSet(inner)
        | RustType::VecDeque(inner)
        | RustType::Option(inner)
        | RustType::Boxed(inner)
        | RustType::Ref { inner, .. }
        | RustType::Array { element: inner, .. } => assert_named_nodes_are_leaf_paths(inner),
        RustType::HashMap(left, right) | RustType::Result(left, right) => {
            assert_named_nodes_are_leaf_paths(left);
            assert_named_nodes_are_leaf_paths(right);
        }
        RustType::Tuple(items) | RustType::Generic { params: items, .. } => {
            for item in items {
                assert_named_nodes_are_leaf_paths(item);
            }
        }
        RustType::Fn { params, ret } => {
            for param in params {
                assert_named_nodes_are_leaf_paths(param);
            }
            assert_named_nodes_are_leaf_paths(ret);
        }
        RustType::DynTrait { trait_, .. } | RustType::ImplTrait { trait_, .. } => match trait_ {
            RustTrait::Named {
                params,
                associated_types,
                ..
            } => {
                for param in params {
                    assert_named_nodes_are_leaf_paths(param);
                }
                for (_, ty) in associated_types {
                    assert_named_nodes_are_leaf_paths(ty);
                }
            }
            RustTrait::Callable { params, ret, .. } => {
                for param in params {
                    assert_named_nodes_are_leaf_paths(param);
                }
                if let Some(ret) = ret {
                    assert_named_nodes_are_leaf_paths(ret);
                }
            }
        },
        RustType::I64
        | RustType::F64
        | RustType::Bool
        | RustType::String_
        | RustType::Unit
        | RustType::Never => {}
    }
}

fn function() -> FunctionType {
    FunctionType::new(vec![("value".to_string(), Type::Int)], Type::Str)
}

fn nominal() -> Type {
    Type::Class {
        identity: Some("app.Widget".to_string()),
        type_args: vec![Type::Int],
        name: "Widget".to_string(),
        fields: vec![("value".to_string(), Type::Int)],
        methods: Vec::new(),
        parent_class: None,
    }
}

#[test]
fn every_type_variant_uses_structured_conversion() {
    let types = vec![
        Type::Int,
        Type::FixedInt(FixedIntType::U8),
        Type::Float,
        Type::Bool,
        Type::Str,
        Type::Bytes,
        Type::None,
        Type::Function(function()),
        Type::AsyncFunction(function()),
        Type::Coroutine(Box::new(Type::Int), Box::new(Type::Str)),
        Type::Task(Box::new(Type::Int), Box::new(Type::Never)),
        Type::TaskResult(Box::new(Type::Int), Box::new(Type::Str)),
        Type::Failure(Box::new(Type::Str)),
        Type::TimeoutResult(Box::new(Type::Str)),
        Type::Select2(Box::new(Type::Int), Box::new(Type::Str)),
        Type::BlockingTask(Box::new(Type::Int), Box::new(Type::Str)),
        Type::JoinSet(Box::new(Type::Int), Box::new(Type::Str)),
        Type::Awaitable(Box::new(Type::Int)),
        Type::AsyncIterator(Box::new(Type::Int), Box::new(Type::Str)),
        Type::AsyncGenerator(Box::new(Type::Int), Box::new(Type::Str)),
        Type::PythonBuffer(Box::new(Type::FixedInt(FixedIntType::U8))),
        Type::PythonArrow(PythonArrowKind::Array),
        Type::PythonDlpackTensor(Box::new(Type::Float)),
        Type::PythonDlpackStream,
        Type::List(Box::new(Type::Int)),
        Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        Type::Set(Box::new(Type::Int)),
        Type::Tuple(vec![Type::Int, Type::Str]),
        Type::Range,
        Type::Iterable(Box::new(Type::Int)),
        Type::Iterator(Box::new(Type::Int)),
        Type::Any,
        Type::Never,
        Type::Union(vec![Type::Int, Type::Str]),
        Type::Intersection(vec![Type::Int, Type::Str]),
        Type::LiteralInt(1),
        Type::LiteralStr("x".to_string()),
        Type::LiteralBool(true),
        Type::Alias {
            name: "Count".to_string(),
            type_args: Vec::new(),
            body: Box::new(Type::Int),
        },
        Type::Unknown,
        Type::Result(Box::new(Type::Int), Box::new(Type::Str)),
        nominal(),
        Type::Protocol {
            identity: Some("app.Readable".to_string()),
            name: "Readable".to_string(),
            methods: vec![("read".to_string(), function())],
        },
        Type::Newtype {
            identity: Some("app.UserId".to_string()),
            name: "UserId".to_string(),
            inner: Box::new(Type::Int),
        },
        Type::TypeVar("T".to_string()),
        Type::Callable(
            vec![Type::Str],
            vec![ParamConvention::borrow()],
            Box::new(Type::Int),
        ),
        Type::AsyncCallable(
            vec![Type::Int],
            vec![ParamConvention::own()],
            Box::new(Type::Str),
        ),
        Type::Enum {
            identity: Some("app.Color".to_string()),
            name: "Color".to_string(),
            variants: vec![("RED".to_string(), Some(1))],
        },
        Type::Decimal,
        Type::BigDecimal,
    ];

    for ty in types {
        let rust_ty = try_sifr_type_to_rust_type(&ty).expect("variant must be supported");
        assert_named_nodes_are_leaf_paths(&rust_ty);
        syn::parse_str::<syn::Type>(&render_type(&rust_ty))
            .expect("structured type must render as Rust syntax");
    }
}

#[test]
fn callable_fields_use_boxed_trait_objects() {
    let callable = Type::Callable(
        vec![Type::Str],
        vec![ParamConvention::borrow()],
        Box::new(Type::Int),
    );
    assert!(matches!(
        sifr_type_to_rust_field_type(&callable),
        RustType::Boxed(_)
    ));
    assert!(matches!(
        sifr_type_to_rust_type(&callable),
        RustType::ImplTrait { .. }
    ));
}

#[test]
fn malformed_callable_returns_one_structured_error() {
    let malformed = Type::Callable(vec![Type::Int], Vec::new(), Box::new(Type::Str));
    let error = try_sifr_type_to_rust_type(&malformed).expect_err("shape must fail");
    assert_eq!(
        error.message,
        "unsupported callable type: 1 parameters but 0 conventions"
    );
}

#[test]
fn malformed_signature_emits_one_production_codegen_error() {
    let malformed = Type::Callable(vec![Type::Int], Vec::new(), Box::new(Type::Str));
    let module = sifr_ir::HirModule {
        functions: vec![sifr_ir::HirFunction {
            name: "broken".to_string(),
            params: vec![sifr_ir::HirParam {
                name: "callback".to_string(),
                ty: malformed,
                default: None,
                keyword_only: false,
                convention: ParamConvention::own(),
            }],
            return_type: Type::None,
            body: Vec::new(),
            is_async: false,
            method_kind: sifr_ir::MethodKind::Regular,
            receiver: None,
            decorators: Vec::new(),
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: Vec::new(),
        }],
        classes: Vec::new(),
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    };

    let generated = crate::generate_rust_with_metadata(&module).rust_source;
    assert_eq!(
        generated.matches("compile_error!").count(),
        1,
        "{generated}"
    );
    assert!(generated.contains("unsupported callable type: 1 parameters but 0 conventions"));
}
