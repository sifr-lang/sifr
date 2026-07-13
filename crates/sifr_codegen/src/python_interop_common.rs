use crate::hir_analysis::traversal::{self, TraversalConfig, TraversalControl};
use sifr_ir::{HirModule, HirStmt, PythonInteropDeclaration, PythonInteropEffect};

pub(crate) fn python_omit_parameter_indices(
    declaration: &PythonInteropDeclaration,
) -> impl Iterator<Item = usize> + '_ {
    declaration
        .parameters
        .iter()
        .enumerate()
        .filter_map(|(index, parameter)| parameter.omit_when_absent.then_some(index))
}

pub(crate) fn module_uses_async_python_declaration(module: &HirModule) -> bool {
    let declaration_uses_async = module
        .functions
        .iter()
        .chain(module.classes.iter().flat_map(|class| class.methods.iter()))
        .any(|function| {
            function
                .python_interop
                .iter()
                .any(|declaration| declaration.effect == PythonInteropEffect::Async)
        });
    if declaration_uses_async {
        return true;
    }
    module
        .functions
        .iter()
        .chain(module.classes.iter().flat_map(|class| class.methods.iter()))
        .any(|function| {
            let mut on_stmt = |stmt: &HirStmt| {
                if matches!(
                    stmt,
                    HirStmt::AsyncWith {
                        kind: sifr_ir::HirAsyncWithKind::Python { .. },
                        ..
                    }
                ) {
                    TraversalControl::Stop
                } else {
                    TraversalControl::Continue
                }
            };
            let mut on_expr = |_: &sifr_ir::HirExpr| TraversalControl::Continue;
            matches!(
                traversal::walk_stmts_until(
                    &function.body,
                    TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
                    &mut on_stmt,
                    &mut on_expr,
                ),
                TraversalControl::Stop
            )
        })
}
