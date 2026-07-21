use crate::python_interop_direct::{
    python_interop_function_body_with_retained_errors,
    python_interop_method_body_with_retained_errors,
};
use crate::RustStmt;
use sifr_ir::{HirFunction, PythonInteropDeclaration};
use std::collections::HashMap;

pub(crate) fn python_interop_function_body(
    func: &HirFunction,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
) -> Option<Vec<RustStmt>> {
    python_interop_function_body_with_retained_errors(func, opaque_classes, &HashMap::new())
}

pub(crate) fn python_interop_method_body(
    func: &HirFunction,
    opaque_classes: &HashMap<String, PythonInteropDeclaration>,
    owner_declaration: Option<&PythonInteropDeclaration>,
) -> Option<Vec<RustStmt>> {
    python_interop_method_body_with_retained_errors(
        func,
        opaque_classes,
        owner_declaration,
        &HashMap::new(),
        &[],
    )
}
