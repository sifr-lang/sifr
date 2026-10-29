use crate::generate_rust;
use sifr_ir::HirModule;
use sifr_lowering::{ExternalDefs, lower_module, lower_module_with_externals};
use sifr_python_parser::parse_module;
use sifr_stdlib_artifact::{StdlibCode, StdlibRustSource};
use std::collections::HashSet;

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

pub(crate) fn trait_impl_fixture_stdlib_code() -> StdlibCode {
    let mut code = StdlibCode::default();
    code.module_rust_code.insert(
        "sifr.tomllib".to_string(),
        StdlibRustSource {
            module: "sifr.tomllib".to_string(),
            source_path: "stdlib/sifr/tomllib.sifr".to_string(),
            source_sha256: "fixture".to_string(),
            nominal_types: HashSet::new(),
            rust: r#"
struct TOMLDecodeError { message: String }
impl std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
fn loads(_text: String) -> Result<String, TOMLDecodeError> {
    Err(TOMLDecodeError { message: "fixture".to_string() })
}
"#
            .to_string(),
        },
    );
    code
}
