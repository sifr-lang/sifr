use crate::rust_interop_direct::{rust_interop_function_body, rust_interop_method_body};
use crate::{generate_rust_with_metadata, render_expr, RustStmt};
use ruff_text_size::TextRange;
use sifr_ir::{
    HirClass, HirClassKind, HirFunction, HirModule, HirParam, MethodKind,
    RustInteropAbiRequirements, RustInteropDeclaration, RustInteropDecoratorKind,
    RustInteropEffect, RustTargetPath,
};
use sifr_type_system::{FixedIntType, ParamConvention, Type};
use std::collections::HashMap;

#[test]
fn rust_interop_function_body_emits_package_bridge_root() {
    let func = HirFunction {
        name: "digest".to_string(),
        params: vec![HirParam {
            name: "data".to_string(),
            ty: Type::Bytes,
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::Bytes,
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["bridge", "hash", "digest"],
        )],
        type_params: Vec::new(),
    };

    let body =
        rust_interop_function_body(&func).expect("bridge interop metadata should lower to a body");
    let [RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("bridge interop should lower to a return expression");
    };

    assert_eq!(render_expr(expr), "bridge::hash::digest(data)");
}

#[test]
fn rust_interop_method_body_emits_self_handle_call() {
    let func = HirFunction {
        name: "encode".to_string(),
        params: vec![HirParam {
            name: "text".to_string(),
            ty: Type::Str,
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::List(Box::new(Type::FixedInt(FixedIntType::U32))),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["Self", "encode"],
        )],
        type_params: Vec::new(),
    };

    let body =
        rust_interop_method_body(&func).expect("Self interop metadata should lower to a body");
    let [RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("Self interop should lower to a return expression");
    };

    assert_eq!(render_expr(expr), "self._handle.encode(text)");
}

#[test]
fn emitted_direct_result_none_interop_does_not_append_ok_tail() {
    let error_ty = Type::Class {
        name: "ZipError".to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    };
    let module = HirModule {
        functions: vec![HirFunction {
            name: "zip_close".to_string(),
            params: vec![HirParam {
                name: "path".to_string(),
                ty: Type::Str,
                default: None,
                keyword_only: false,
                convention: ParamConvention::borrow(),
            }],
            return_type: Type::Result(Box::new(Type::None), Box::new(error_ty)),
            body: Vec::new(),
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: Vec::new(),
            rust_interop: vec![declaration(
                RustInteropDecoratorKind::Function,
                &["sifr_stdlib", "zip", "zip_close"],
            )],
            type_params: Vec::new(),
        }],
        classes: vec![zip_error_class()],
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module).rust_source;

    assert!(generated.contains("sifr_stdlib::zip::zip_close(path)"));
    assert!(!generated.contains("return Ok(());"), "{generated}");
}

fn zip_error_class() -> HirClass {
    HirClass {
        name: "ZipError".to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: Vec::new(),
        is_hashable: false,
        is_error_type: true,
        kind: HirClassKind::Regular,
        operator_impls: Vec::new(),
        newtype_inner: None,
        implements_protocols: Vec::new(),
        parent_class: Some("Error".to_string()),
        type_params: Vec::new(),
        enum_variants: Vec::new(),
        rust_interop: Vec::new(),
    }
}

fn declaration(kind: RustInteropDecoratorKind, segments: &[&str]) -> RustInteropDeclaration {
    RustInteropDeclaration {
        kind,
        target: Some(RustTargetPath {
            segments: segments
                .iter()
                .map(|segment| (*segment).to_string())
                .collect(),
            span: TextRange::default(),
        }),
        arguments: Vec::new(),
        span: TextRange::default(),
        effect: RustInteropEffect::Sync,
        abi_requirements: RustInteropAbiRequirements::default(),
    }
}
