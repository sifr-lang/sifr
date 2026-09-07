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
            .contains("pub fn shared_operation")
    );
    assert!(
        !generated
            .project_union_prelude
            .contains("pub mod __sifr_generated_support")
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
                .matches("use crate::__sifr_generated_support::{shared_operation};")
                .count(),
            1
        );
    }
}

#[test]
fn support_visibility_keeps_helper_functions_and_imports_private() {
    let source = "use std::fmt::Debug; fn helper() {} fn shared() { helper(); }";
    let visible =
        crate::crate_visible_generated_support_source(source, &["fn call() { shared(); }"]);
    assert!(visible.contains("pub fn shared()"), "{visible}");
    assert!(visible.contains("\nfn helper()"), "{visible}");
    assert!(visible.starts_with("use std::fmt::Debug;"), "{visible}");
    assert!(!visible.contains("pub(crate)"));
}

#[test]
fn support_visibility_exposes_signature_types_but_not_private_implementation_types() {
    let source = "struct ResultValue { value: i64 } struct Internal;\n\
        fn helper(_: Internal) {} fn shared() -> ResultValue { helper(Internal); ResultValue { value: 1 } }";
    let visible =
        crate::crate_visible_generated_support_source(source, &["fn call() { shared(); }"]);
    assert!(visible.contains("pub struct ResultValue"), "{visible}");
    assert!(visible.contains("pub value: i64"), "{visible}");
    assert!(visible.contains("\nstruct Internal;"), "{visible}");
    assert!(!visible.contains("pub fn helper"), "{visible}");
}

#[test]
fn support_imports_resolve_lexical_bindings_and_module_boundaries() {
    let support = "fn channel() {} fn message() {} fn helper() {} struct Value;";
    for consumer in [
        "fn consume(channel: i64) { sink(channel); }",
        "fn consume() { let channel = 0; sink(channel); }",
        "fn consume() { let callback = |channel| sink(channel); }",
        "fn consume() { for channel in values { sink(channel); } }",
        "fn consume() { match value { Some(channel) => sink(channel), _ => () } }",
        "fn consume() { if let Some(channel) = value { sink(channel); } }",
        "fn consume() { while let Some(channel) = value { sink(channel); } }",
        "fn consume() { let channel = 0; println!(\"{}\", channel); }",
        "fn consume() { let channel = 0; println!(\"{channel}\"); }",
        "struct Generic<Value> { value: Value }",
        "fn consume<Value>(value: Value) { sink(value); }",
        "mod child { fn consume() { channel(); } }",
        "fn channel() {} fn consume() { channel(); }",
    ] {
        assert_eq!(
            crate::generated_visibility::generated_support_import(consumer, support),
            "",
            "{consumer}"
        );
    }
    for consumer in [
        "fn consume() { let channel = channel(); sink(channel); }",
        "fn consume() { { let channel = 0; sink(channel); } channel(); }",
        "fn consume() { let channel = 0; fn nested() { channel(); } nested(); }",
        "fn consume() { if let Some(channel) = value { sink(channel); } else { channel(); } }",
    ] {
        assert_eq!(
            crate::generated_visibility::generated_support_import(consumer, support),
            "use crate::__sifr_generated_support::{channel};",
            "{consumer}"
        );
    }
}

#[test]
fn support_imports_keep_implicit_format_captures_after_canonicalization() {
    let source = "mod __sifr_generated_support { pub const ACTIVE: i64 = 1; }\n\
        use crate::__sifr_generated_support::{ACTIVE};\n\
        fn main() { println!(\"{}\", ACTIVE); }";
    let canonical = crate::canonicalize_generated_rust_source(source).expect("canonical source");
    assert!(canonical.contains("{ACTIVE}"), "{canonical}");
    assert!(
        canonical.contains("use crate::sifr_generated_generated_support::{ACTIVE};"),
        "{canonical}"
    );
}

#[test]
fn support_imports_drop_reverse_nominal_edges_of_pruned_helpers() {
    let source = "struct Used; struct Unused;\n\
        mod __sifr_generated_support { use crate::{Used, Unused};\n\
        pub fn create() -> Used { Used } fn unused() -> Unused { Unused } }\n\
        use crate::__sifr_generated_support::{create}; fn main() { consume(create()); }\n\
        fn consume<T>(_: T) {}";
    let canonical = crate::canonicalize_generated_rust_source(source).expect("canonical source");
    assert!(!canonical.contains("fn unused"), "{canonical}");
    assert!(canonical.contains("use crate::{Used};"), "{canonical}");
    assert!(!canonical.contains("Unused"), "{canonical}");
    assert_eq!(
        crate::canonicalize_generated_rust_source(&canonical).expect("idempotent source"),
        canonical
    );
}

#[test]
fn support_imports_are_refreshed_after_reference_removing_canonicalization() {
    let source = "mod __sifr_generated_support { pub struct Value; pub struct Task;\n\
        impl Task { pub async fn result(self) -> Value { Value } } pub fn create() -> Task { Task } }\n\
        use crate::__sifr_generated_support::{Value, create};\n\
        #[tokio::main] async fn main() { let value: Value = create().result().await; consume(value); } fn consume<T>(_: T) {}";
    let canonical = crate::canonicalize_generated_rust_source(source).expect("canonical source");
    let file = syn::parse_file(&canonical).expect("final syntax");
    let imports = file
        .items
        .iter()
        .filter_map(|item| {
            if let syn::Item::Use(item) = item {
                Some(quote::quote!(#item).to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert!(
        imports.iter().all(|import| !import.contains("Value")),
        "{canonical}"
    );
    assert!(
        imports.iter().any(|import| import.contains("create")),
        "{canonical}"
    );
}

#[test]
fn support_imports_include_trait_methods_and_macro_statics_without_qualified_names() {
    let support = "struct Qualified; trait Action { fn act(&self); }\n\
        tokio::task_local! { static ACTIVE: String; } fn helper() {}";
    let consumer = "fn call(x: crate::__sifr_generated_support::Qualified) {\n\
        x.act(); ACTIVE.try_with(|_| ()); }";
    assert_eq!(
        crate::generated_visibility::generated_support_import(consumer, support),
        "use crate::__sifr_generated_support::{ACTIVE, Action};"
    );
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
