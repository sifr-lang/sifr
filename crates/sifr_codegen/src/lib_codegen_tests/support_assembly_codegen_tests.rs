use super::*;
use std::collections::{HashMap, HashSet};

#[test]
fn multi_module_support_has_one_private_owner_and_a_strict_size_budget() {
    let main = module_calling_shared("main");
    let worker = module_calling_shared("work");
    let generated = generate_rust_multi_with_metadata(
        &[("main", &main), ("worker", &worker)],
        &shared_stdlib(),
    );

    let all_source = std::iter::once(generated.project_union_prelude.as_str())
        .chain(generated.rust_files.values().map(String::as_str))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(all_source.matches("fn shared_operation").count(), 1);
    assert_eq!(
        all_source.matches("mod __sifr_generated_support").count(),
        1
    );
    assert!(
        generated
            .project_union_prelude
            .contains("pub(crate) fn shared_operation")
    );
    assert!(
        !generated
            .project_union_prelude
            .contains("pub fn shared_operation")
    );
    assert!(
        generated.project_union_prelude.len() <= 512,
        "support bridge exceeded its budget:\n{}",
        generated.project_union_prelude
    );

    for source in generated.rust_files.values() {
        assert!(!source.contains("// --- stdlib:"));
        assert!(!source.contains("fn shared_operation"));
        assert_eq!(
            source
                .matches("use crate::__sifr_generated_support::*;")
                .count(),
            1
        );
    }
}

#[test]
fn multi_module_project_omits_support_when_no_module_demands_it() {
    let main = empty_named_module("main");
    let worker = empty_named_module("work");

    let generated = generate_rust_multi_with_metadata(
        &[("main", &main), ("worker", &worker)],
        &StdlibCode::default(),
    );

    assert!(
        !generated
            .project_union_prelude
            .contains("__sifr_generated_support")
    );
    assert!(
        !generated
            .project_union_prelude
            .contains("__sifr_project_nominals")
    );
    assert!(
        generated
            .rust_files
            .values()
            .all(|source| !source.contains("__sifr_generated_support"))
    );
}

#[test]
fn test_project_support_is_rendered_once_for_support_and_test_modules() {
    let support = module_calling_shared("support_value");
    let test = module_calling_shared("test_shared_value");
    let generated = crate::generate_rust_test_project_with_metadata(
        &[("support", &support)],
        &[("test_shared", &test)],
        &shared_stdlib(),
    );

    let all_source = std::iter::once(generated.project_union_prelude.as_str())
        .chain(generated.support_rust_files.values().map(String::as_str))
        .chain(generated.test_rust_files.values().map(String::as_str))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(all_source.matches("fn shared_operation").count(), 1);
    assert_eq!(
        all_source.matches("mod __sifr_generated_support").count(),
        1
    );
    assert!(
        generated
            .support_rust_files
            .values()
            .all(|source| !source.contains("fn shared_operation"))
    );
    assert!(
        generated
            .test_rust_files
            .values()
            .all(|source| !source.contains("fn shared_operation"))
    );
}

fn module_calling_shared(function_name: &str) -> HirModule {
    let mut module = empty_named_module(function_name);
    module.imports.push(HirImport {
        module: "sifr.shared".to_string(),
        names: vec!["shared_operation".to_string()],
        aliases: Vec::new(),
    });
    module.functions[0].body.push(HirStmt::Expr {
        expr: HirExpr::Call {
            mutable_arg_places: Vec::new(),
            func: "shared_operation".to_string(),
            args: Vec::new(),
            ty: Type::None,
        },
    });
    module
}

fn empty_named_module(function_name: &str) -> HirModule {
    HirModule {
        functions: vec![HirFunction {
            name: function_name.to_string(),
            params: Vec::new(),
            return_type: Type::None,
            body: Vec::new(),
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
    }
}

fn shared_stdlib() -> StdlibCode {
    let mut stdlib = StdlibCode::default();
    stdlib.module_rust_code.insert(
        "sifr.shared".to_string(),
        StdlibRustSource {
            module: "sifr.shared".to_string(),
            source_path: "stdlib/sifr/shared.sifr".to_string(),
            source_sha256: "support-assembly-fixture".to_string(),
            nominal_types: HashSet::new(),
            rust: "fn shared_operation() {}\n".to_string(),
        },
    );
    stdlib.func_signatures.insert(
        "sifr.shared".to_string(),
        HashMap::from([("shared_operation".to_string(), (Vec::new(), Type::None))]),
    );
    stdlib
}
