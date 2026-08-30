use super::type_contains_by;
use crate::hir_analysis::traversal::{self, TraversalConfig, TraversalControl};
use sifr_ir::{HirExpr, HirFunction, HirModule, HirStmt};
use sifr_type_system::Type;

fn type_contains_template(ty: &Type) -> bool {
    type_contains_by(ty, |candidate| matches!(candidate, Type::Template(_)))
}

pub(crate) fn module_uses_template(module: &HirModule) -> bool {
    fn function_uses_template(function: &HirFunction) -> bool {
        if function
            .params
            .iter()
            .any(|param| type_contains_template(&param.ty))
            || type_contains_template(&function.return_type)
        {
            return true;
        }
        let mut on_stmt = |_stmt: &HirStmt| TraversalControl::Continue;
        let mut on_expr = |expr: &HirExpr| {
            if matches!(expr, HirExpr::TemplateString(_)) {
                TraversalControl::Stop
            } else {
                TraversalControl::Continue
            }
        };
        matches!(
            traversal::walk_stmts_until(
                &function.body,
                TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
                &mut on_stmt,
                &mut on_expr,
            ),
            TraversalControl::Stop
        )
    }

    module.functions.iter().any(function_uses_template)
        || module.classes.iter().any(|class| {
            class
                .fields
                .iter()
                .any(|(_, field_ty)| type_contains_template(field_ty))
                || class.methods.iter().any(function_uses_template)
        })
        || module.constants.iter().any(|(_, ty, value)| {
            type_contains_template(ty) || matches!(value, HirExpr::TemplateString(_))
        })
}
