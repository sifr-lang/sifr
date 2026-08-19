use super::{diagnostic, DescriptorResolver};
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_lowering::{DeclarationDescriptorKind, HirDiagnostic};
use sifr_python_ast::{Expr, StmtFunctionDef};

pub(super) fn validate(
    function: &StmtFunctionDef,
    resolver: &DescriptorResolver,
    errors: &mut Vec<HirDiagnostic>,
) {
    let classmethod_index = decorator_index(function, "classmethod");
    let staticmethod_index = decorator_index(function, "staticmethod");
    for (index, decorator) in function.decorator_list.iter().enumerate() {
        let Some((declaration, call)) = resolver.call(&decorator.expression) else {
            continue;
        };
        if declaration.kind != DeclarationDescriptorKind::Method {
            continue;
        }
        if classmethod_index.is_some_and(|classmethod_index| {
            index + 1 != classmethod_index || classmethod_index + 1 != function.decorator_list.len()
        }) {
            errors.push(diagnostic(
                DiagnosticCode::META_MALFORMED_DECLARATION,
                format!(
                    "method descriptor '{}' must be the outer decorator with @classmethod directly above the method",
                    declaration.function
                ),
                call.range(),
            ));
        }
        if staticmethod_index.is_some_and(|staticmethod_index| {
            staticmethod_index != 0
                || index != staticmethod_index + 1
                || index + 1 != function.decorator_list.len()
        }) {
            errors.push(diagnostic(
                DiagnosticCode::META_MALFORMED_DECLARATION,
                format!(
                    "method descriptor '{}' must be directly above the method with @staticmethod as the outer decorator",
                    declaration.function
                ),
                call.range(),
            ));
        }
    }
}

fn decorator_index(function: &StmtFunctionDef, name: &str) -> Option<usize> {
    function.decorator_list.iter().position(
        |decorator| matches!(&decorator.expression, Expr::Name(found) if found.id.as_str() == name),
    )
}
