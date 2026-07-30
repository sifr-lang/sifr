use super::method_receiver_analysis::method_signature;
use crate::hir_nodes::{HirExpr, HirFunction, HirIteratorOp, HirModule};
use ruff_text_size::TextRange;
use sifr_ir::{visit_hir_function_exprs_mut, MutableArgumentTarget, Place, PlaceProjection};
use sifr_type_system::ReceiverConvention;
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

#[derive(Debug)]
pub(super) struct MethodCallInvariantViolation {
    pub message: String,
    pub range: TextRange,
}

pub(super) fn verify_module_method_calls(
    module: &mut HirModule,
    class_types: &HashMap<String, Type>,
    functions: &HashMap<String, FunctionType>,
) -> Vec<MethodCallInvariantViolation> {
    let mut violations = Vec::new();
    for function in &mut module.functions {
        verify_function(function, class_types, functions, &mut violations);
    }
    for class in &mut module.classes {
        for method in class
            .methods
            .iter_mut()
            .chain(class.operator_impls.iter_mut().map(|(_, method)| method))
        {
            verify_function(method, class_types, functions, &mut violations);
        }
    }
    violations
}

fn verify_function(
    function: &mut HirFunction,
    class_types: &HashMap<String, Type>,
    functions: &HashMap<String, FunctionType>,
    violations: &mut Vec<MethodCallInvariantViolation>,
) {
    visit_hir_function_exprs_mut(function, &mut |expr| {
        if let HirExpr::IteratorCall {
            op: HirIteratorOp::Next,
            args,
            mutable_arg_places,
            ..
        } = expr
        {
            if mutable_arg_places.len() != args.len()
                || !mutable_arg_places.first().is_some_and(Option::is_some)
            {
                violations.push(MethodCallInvariantViolation {
                    message:
                        "internal compiler error: iterator next call has no checked mutable place"
                            .to_string(),
                    range: TextRange::default(),
                });
            }
            verify_mutable_argument_targets(
                args,
                mutable_arg_places,
                TextRange::default(),
                &mut |message, range| {
                    violations.push(MethodCallInvariantViolation { message, range });
                },
            );
            return;
        }
        if let HirExpr::Call {
            func,
            args,
            mutable_arg_places,
            ..
        } = expr
        {
            // Regular source calls already carry the targets proven against
            // their lexical signature. Re-resolving `func` through the
            // module-wide function table is unsound for same-named nested
            // helpers in different scopes.
            let requires_first_mutable_place = func == "anext";
            if requires_first_mutable_place
                && !mutable_arg_places.first().is_some_and(Option::is_some)
            {
                violations.push(MethodCallInvariantViolation {
                    message: format!(
                        "internal compiler error: mutable source call '{func}' has no checked argument place"
                    ),
                    range: TextRange::default(),
                });
            }
            verify_mutable_argument_targets(
                args,
                mutable_arg_places,
                TextRange::default(),
                &mut |message, range| {
                    violations.push(MethodCallInvariantViolation { message, range });
                },
            );
            return;
        }
        let HirExpr::MethodCall {
            object,
            method,
            args,
            receiver_convention,
            receiver_target,
            mutable_arg_places,
            source: Some(source),
            ..
        } = expr
        else {
            return;
        };

        if receiver_convention.is_none() {
            violations.push(MethodCallInvariantViolation {
                message: format!(
                    "internal compiler error: source method call '{method}' on '{}' has no resolved receiver convention",
                    object.ty().display_name()
                ),
                range: source.call_range,
            });
        }
        if source.arg_ranges.len() != args.len() {
            violations.push(MethodCallInvariantViolation {
                message: format!(
                    "internal compiler error: source method call '{method}' has {} HIR argument(s) but {} source range(s)",
                    args.len(),
                    source.arg_ranges.len()
                ),
                range: source.call_range,
            });
        }
        if mutable_arg_places.len() != args.len() {
            violations.push(MethodCallInvariantViolation {
                message: format!(
                    "internal compiler error: source method call '{method}' has {} HIR argument(s) but {} mutable-place proof slot(s)",
                    args.len(),
                    mutable_arg_places.len()
                ),
                range: source.call_range,
            });
        }
        verify_mutable_argument_targets(
            args,
            mutable_arg_places,
            source.call_range,
            &mut |message, range| {
                violations.push(MethodCallInvariantViolation { message, range });
            },
        );
        if *receiver_convention == Some(ReceiverConvention::MutableBorrow)
            && receiver_target.is_none()
        {
            violations.push(MethodCallInvariantViolation {
                message: format!(
                    "internal compiler error: mutable source method call '{method}' has no checked receiver target"
                ),
                range: source.receiver_range,
            });
        }
        if receiver_target
            .as_ref()
            .is_some_and(|target| !receiver_target_matches(object, target))
        {
            violations.push(MethodCallInvariantViolation {
                message: format!(
                    "internal compiler error: mutable source method call '{method}' has an invalid checked receiver target"
                ),
                range: source.receiver_range,
            });
        }
        if let Some(signature) = method_signature(object.ty(), method, class_types, functions) {
            if let Some(expected) = signature.receiver {
                if *receiver_convention != Some(expected) {
                    violations.push(MethodCallInvariantViolation {
                    message: format!(
                        "internal compiler error: source method call '{method}' has receiver convention {receiver_convention:?}, expected {expected:?}"
                    ),
                    range: source.call_range,
                });
                }
            }
            for (index, (_, _, convention)) in signature.params.iter().enumerate() {
                if convention.is_mut_borrow()
                    && !mutable_arg_places.get(index).is_some_and(Option::is_some)
                {
                    violations.push(MethodCallInvariantViolation {
                        message: format!(
                            "internal compiler error: mutable argument {} of source method call '{method}' has no checked place",
                            index + 1
                        ),
                        range: source
                            .arg_ranges
                            .get(index)
                            .copied()
                            .unwrap_or(source.call_range),
                    });
                }
            }
        }
    });
}

fn verify_mutable_argument_targets(
    args: &[HirExpr],
    targets: &[Option<MutableArgumentTarget>],
    fallback_range: TextRange,
    report: &mut impl FnMut(String, TextRange),
) {
    for (index, target) in targets.iter().enumerate() {
        let Some(target) = target else {
            continue;
        };
        let Some(argument) = args.get(index) else {
            report(
                format!(
                    "internal compiler error: mutable argument target {} has no matching argument",
                    index + 1
                ),
                fallback_range,
            );
            continue;
        };
        let valid = match target {
            MutableArgumentTarget::Place(place) => expression_matches_place(argument, place),
            MutableArgumentTarget::OwnedTemporary => {
                super::method_receiver_places::is_owned_temporary(argument)
            }
        };
        if !valid {
            report(
                format!(
                    "internal compiler error: mutable argument {} has an invalid checked target",
                    index + 1
                ),
                fallback_range,
            );
        }
    }
}

fn expression_matches_place(expression: &HirExpr, place: &Place) -> bool {
    expression_matches_place_projection(expression, place, place.projections.len())
}

fn receiver_target_matches(expression: &HirExpr, target: &sifr_ir::MutableReceiverTarget) -> bool {
    match target {
        sifr_ir::MutableReceiverTarget::Place(place) => expression_matches_place(expression, place),
        sifr_ir::MutableReceiverTarget::OwnedTemporary => {
            super::method_receiver_places::is_owned_temporary(expression)
        }
        sifr_ir::MutableReceiverTarget::SpecializedIndexedStorage(place) => {
            matches!(
                expression,
                HirExpr::Index { object, .. } if expression_matches_place(object, place)
            )
        }
    }
}

fn expression_matches_place_projection(
    expression: &HirExpr,
    place: &Place,
    projection_count: usize,
) -> bool {
    match expression {
        HirExpr::Name {
            binding_id: Some(binding_id),
            ..
        } => *binding_id == place.root && projection_count == 0,
        HirExpr::FieldAccess { object, field, .. } if projection_count > 0 => {
            let PlaceProjection::Field(identity) = &place.projections[projection_count - 1];
            identity.field == *field
                && expression_matches_place_projection(object, place, projection_count - 1)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(violations.len(), 3);
        assert!(violations
            .iter()
            .any(|violation| violation.message.contains("no resolved receiver")));
        assert!(violations
            .iter()
            .any(|violation| violation.message.contains("source range")));
        assert!(violations
            .iter()
            .any(|violation| violation.message.contains("mutable-place proof slot")));
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
}
