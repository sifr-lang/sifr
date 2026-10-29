use crate::generate_rust;
use sifr_ir::HirModule;
use sifr_lowering::{ExternalDefs, lower_module, lower_module_with_externals};
use sifr_python_parser::parse_module;

pub(crate) fn generate_rust_from_source(source: &str) -> String {
    let parsed = parse_module(source).expect("parse failed");
    let lowering = lower_module(parsed.suite()).expect("lowering failed");
    generate_rust(&lowering.module)
}

pub(crate) fn generate_rust_from_source_with_stdlib_collections(source: &str) -> String {
    let parsed = parse_module(source).expect("parse failed");
    let mut externals = ExternalDefs::default();
    externals
        .functions
        .entry("sifr.collections".to_string())
        .or_default();
    let lowering =
        lower_module_with_externals(parsed.suite(), &externals).expect("lowering failed");
    generate_rust(&lowering.module)
}

pub(crate) fn empty_module() -> HirModule {
    HirModule {
        functions: vec![],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    }
}
