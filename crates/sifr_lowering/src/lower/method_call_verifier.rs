use super::method_receiver_analysis::method_signature;
use crate::hir_nodes::{HirExpr, HirFunction, HirModule};
use ruff_text_size::TextRange;
use sifr_ir::visit_hir_function_exprs_mut;
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

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
        let HirExpr::MethodCall {
            object,
            method,
            args,
            receiver_convention,
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
        if let Some(expected) = method_signature(object.ty(), method, class_types, functions)
            .and_then(|sig| sig.receiver)
        {
            if *receiver_convention != Some(expected) {
                violations.push(MethodCallInvariantViolation {
                    message: format!(
                        "internal compiler error: source method call '{method}' has receiver convention {receiver_convention:?}, expected {expected:?}"
                    ),
                    range: source.call_range,
                });
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir_nodes::{HirStmt, MethodCallSource, MethodKind};
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
                        object: Box::new(HirExpr::Name {
                            name: "value".to_string(),
                            binding_id: None,
                            ty: Type::List(Box::new(Type::Int)),
                        }),
                        method: "append".to_string(),
                        args: vec![HirExpr::IntLiteral(1)],
                        receiver_convention: None,
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
        assert_eq!(violations.len(), 2);
        assert!(violations
            .iter()
            .any(|violation| violation.message.contains("no resolved receiver")));
        assert!(violations
            .iter()
            .any(|violation| violation.message.contains("source range")));
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
                        object: Box::new(HirExpr::Name {
                            name: "value".to_string(),
                            binding_id: None,
                            ty: Type::List(Box::new(Type::Int)),
                        }),
                        method: "append".to_string(),
                        args: vec![HirExpr::IntLiteral(1)],
                        receiver_convention: Some(ReceiverConvention::MutableBorrow),
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

        assert!(
            verify_module_method_calls(&mut module, &HashMap::new(), &HashMap::new()).is_empty()
        );
    }
}
