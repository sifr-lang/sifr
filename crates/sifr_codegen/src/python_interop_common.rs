use crate::hir_analysis::traversal::{self, TraversalConfig, TraversalControl};
use sifr_ir::{HirModule, HirStmt, PythonInteropDeclaration, PythonInteropEffect};
use sifr_type_system::Type;

pub(crate) fn python_omit_parameter_indices(
    declaration: &PythonInteropDeclaration,
) -> impl Iterator<Item = usize> + '_ {
    declaration
        .parameters
        .iter()
        .enumerate()
        .filter_map(|(index, parameter)| parameter.omit_when_absent.then_some(index))
}

pub(crate) fn module_uses_python_declaration(module: &HirModule) -> bool {
    module
        .functions
        .iter()
        .chain(module.classes.iter().flat_map(|class| class.methods.iter()))
        .any(|function| !function.python_interop.is_empty())
        || module
            .classes
            .iter()
            .any(|class| class.python_opaque_declaration().is_some())
}

pub(crate) fn rust_source_uses_python_runtime(source: &str) -> bool {
    source.contains("::sifr_stdlib::python::") || source.contains("::sifr_runtime::python::")
}

pub(crate) fn python_error_contract_types(
    module: &HirModule,
) -> std::collections::BTreeMap<String, Type> {
    let mut rust_types = std::collections::BTreeMap::new();
    for function in module
        .functions
        .iter()
        .chain(module.classes.iter().flat_map(|class| class.methods.iter()))
    {
        if !function.python_interop.is_empty() {
            record_python_error_contract(&function.return_type, &mut rust_types);
        }
    }
    rust_types
}

fn record_python_error_contract(
    ty: &Type,
    rust_types: &mut std::collections::BTreeMap<String, Type>,
) {
    match ty.resolve_alias() {
        Type::Result(_, error) => record_python_error_contract(error, rust_types),
        Type::Union(members) => {
            for member in members {
                record_python_error_contract(member, rust_types);
            }
        }
        class @ Type::Class { .. } if class.is_python_error_contract() => {
            rust_types
                .entry(crate::render_type(&crate::sifr_type_to_rust_type(class)))
                .or_insert_with(|| class.clone());
        }
        _ => {}
    }
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

#[cfg(test)]
mod tests {
    use super::rust_source_uses_python_runtime;

    #[test]
    fn runtime_dependency_detection_covers_both_python_namespaces() {
        assert!(rust_source_uses_python_runtime(
            "::sifr_stdlib::python::PythonError"
        ));
        assert!(rust_source_uses_python_runtime(
            "::sifr_runtime::python::PythonError"
        ));
        assert!(!rust_source_uses_python_runtime("struct PythonError;"));
    }
}
