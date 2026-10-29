use super::*;
use crate::hir_nodes::{HirStmt, MethodCallSource, MethodKind, MutableReceiverTarget};
use ruff_text_size::{TextRange, TextSize};
use sifr_type_system::ReceiverConvention;

#[test]
fn verifier_rejects_compiler_authored_malformed_source_call() {
    let range = TextRange::new(TextSize::new(1), TextSize::new(5));
    let mut module = HirModule {
        functions: vec![HirFunction {
            name: "broken".to_string(),
            params: Vec::new(),
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::MethodCall {
                    authority: sifr_ir::MethodAuthority::Unclassified,
                    object: Box::new(HirExpr::ListLiteral {
                        elements: Vec::new(),
                        ty: Type::List(Box::new(Type::Int)),
                    }),
                    method: "append".to_string(),
                    args: vec![HirExpr::IntLiteral(1)],
                    receiver_convention: None,
                    receiver_target: None,
                    mutable_arg_places: Vec::new(),
                    source: Some(MethodCallSource {
                        call_range: range,
                        receiver_range: range,
                        arg_ranges: Vec::new(),
                    }),
                    ty: Type::None,
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
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

    let violations = verify_module_method_calls(&mut module, &HashMap::new(), &HashMap::new());
    assert_eq!(violations.len(), 4);
    assert!(
        violations
            .iter()
            .any(|violation| violation.message.contains("no resolved receiver"))
    );
    assert!(
        violations
            .iter()
            .any(|violation| violation.message.contains("source range"))
    );
    assert!(
        violations
            .iter()
            .any(|violation| violation.message.contains("mutable-place proof slot"))
    );
}

#[test]
fn verifier_accepts_aligned_resolved_source_call() {
    let range = TextRange::new(TextSize::new(1), TextSize::new(5));
    let mut module = HirModule {
        functions: vec![HirFunction {
            name: "valid".to_string(),
            params: Vec::new(),
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::MethodCall {
                    authority: sifr_ir::MethodAuthority::BuiltinIntrinsic {
                        declaration: sifr_ir::CallableIdentity {
                            module: "sifr.builtin".to_string(),
                            owner: Some("list[int]".to_string()),
                            symbol: "append".to_string(),
                            generic_arguments: Vec::new(),
                            signature: "(int) -> None".to_string(),
                        },
                    },
                    object: Box::new(HirExpr::ListLiteral {
                        elements: Vec::new(),
                        ty: Type::List(Box::new(Type::Int)),
                    }),
                    method: "append".to_string(),
                    args: vec![HirExpr::IntLiteral(1)],
                    receiver_convention: Some(ReceiverConvention::MutableBorrow),
                    receiver_target: Some(MutableReceiverTarget::OwnedTemporary),
                    mutable_arg_places: vec![None],
                    source: Some(MethodCallSource {
                        call_range: range,
                        receiver_range: range,
                        arg_ranges: vec![range],
                    }),
                    ty: Type::None,
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
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

    let violations = verify_module_method_calls(&mut module, &HashMap::new(), &HashMap::new());
    assert!(violations.is_empty(), "{violations:?}");
}

#[test]
fn verifier_distinguishes_mutable_places_from_owned_temporaries() {
    let binding_id = sifr_ir::BindingId(7);
    let named = HirExpr::Name {
        name: "items".to_string(),
        binding_id: Some(binding_id),
        ty: Type::List(Box::new(Type::Int)),
    };
    let literal = HirExpr::ListLiteral {
        elements: Vec::new(),
        ty: Type::List(Box::new(Type::Int)),
    };
    let mut violations = Vec::new();
    verify_mutable_argument_targets(
        &[named.clone(), literal],
        &[
            Some(MutableArgumentTarget::Place(Place {
                root: binding_id,
                projections: Vec::new(),
            })),
            Some(MutableArgumentTarget::OwnedTemporary),
        ],
        TextRange::default(),
        &mut |message, _| violations.push(message),
    );
    assert!(violations.is_empty(), "{violations:?}");

    verify_mutable_argument_targets(
        &[named],
        &[Some(MutableArgumentTarget::OwnedTemporary)],
        TextRange::default(),
        &mut |message, _| violations.push(message),
    );
    assert_eq!(violations.len(), 1);
    assert!(violations[0].contains("invalid checked target"));
}

#[test]
fn verifier_rejects_conditional_storage_as_an_owned_temporary() {
    let binding_id = sifr_ir::BindingId(11);
    let named = || HirExpr::Name {
        name: "item".to_string(),
        binding_id: Some(binding_id),
        ty: Type::List(Box::new(Type::Int)),
    };
    let conditional = HirExpr::IfExpr {
        condition: Box::new(HirExpr::BoolLiteral(true)),
        then_expr: Box::new(named()),
        else_expr: Box::new(named()),
        ty: Type::List(Box::new(Type::Int)),
    };
    let mut violations = Vec::new();
    verify_mutable_argument_targets(
        &[conditional],
        &[Some(MutableArgumentTarget::OwnedTemporary)],
        TextRange::default(),
        &mut |message, _| violations.push(message),
    );
    assert_eq!(violations.len(), 1);
    assert!(violations[0].contains("invalid checked target"));
}
