use super::declarations;
use crate::stdlib_filter::{filter_stdlib_ir_to_needed, rust_source_defined_item_names};
use quote::ToTokens;
use std::collections::HashSet;

fn locals(source: &str) -> Vec<super::TaskLocal> {
    syn::parse_file(source)
        .expect("valid Rust")
        .items
        .into_iter()
        .flat_map(|item| {
            if let syn::Item::Macro(item) = item {
                declarations(&item.mac).map_or_else(Vec::new, |locals| locals.0)
            } else {
                Vec::new()
            }
        })
        .collect()
}

#[test]
fn task_local_support_visibility_preserves_names_types_attributes_and_boundaries() {
    let source = r#"
        ::tokio::task_local! {
            #[doc = "static __SIFR_TASK_CANCELLATION: unchanged text"]
            static __SIFR_TASK_CANCELLATION: Carrier;
            pub static __SIFR_TASK_CONTEXT_LABEL: String
        }
        other::task_local! { static PRIVATE: String; }
        task_local! { static UNQUALIFIED: String; }
        mod nested { tokio::task_local! { static NESTED: String; } }
    "#;
    let visible = crate::crate_visible_generated_support_source(source);
    let declared = locals(&visible);
    assert_eq!(declared.len(), 2);
    assert_eq!(declared[0].name, "__SIFR_TASK_CANCELLATION");
    assert_eq!(declared[1].name, "__SIFR_TASK_CONTEXT_LABEL");
    for declaration in &declared {
        assert_eq!(
            declaration.visibility.to_token_stream().to_string(),
            "pub (crate)"
        );
    }
    assert_eq!(declared[0].ty.to_token_stream().to_string(), "Carrier");
    assert_eq!(declared[1].ty.to_token_stream().to_string(), "String");
    assert_eq!(declared[0].attrs.len(), 1);
    let original = syn::parse_file(source).expect("valid source");
    let rewritten = syn::parse_file(&visible).expect("valid rewrite");
    for (original, rewritten) in original.items[1..].iter().zip(&rewritten.items[1..]) {
        assert_eq!(
            original.to_token_stream().to_string(),
            rewritten.to_token_stream().to_string()
        );
    }
    assert_eq!(
        crate::crate_visible_generated_support_source(&visible),
        visible
    );
    // Public module rewriting must not turn support macro statics into a public API.
    assert!(matches!(
        locals(&crate::publicize_generated_module_source(source))[0].visibility,
        syn::Visibility::Inherited
    ));
}

#[test]
fn task_local_support_discovery_includes_all_macro_owned_symbols() {
    let names = rust_source_defined_item_names(
        "tokio::task_local! { static FIRST: String; static SECOND: u64; }\n\
         other::task_local! { static PRIVATE: String; }",
    );
    assert_eq!(names, HashSet::from(["FIRST".into(), "SECOND".into()]));
}

#[test]
fn task_local_support_pruning_tracks_direct_transitive_and_absent_demand() {
    let source = "struct Carrier; struct Unused;\n\
        tokio::task_local! { static ACTIVE: Carrier; static IDLE: Unused; }\n\
        fn accessor() { ACTIVE.try_with(|_| ()); }";
    for root in ["ACTIVE", "accessor"] {
        let filtered = filter_stdlib_ir_to_needed(source, &HashSet::from([root.into()]));
        let names = rust_source_defined_item_names(&filtered);
        assert!(names.contains("ACTIVE"), "{filtered}");
        assert!(names.contains("Carrier"), "{filtered}");
        assert!(!names.contains("IDLE"), "{filtered}");
        assert!(!names.contains("Unused"), "{filtered}");
        let consumer = if root == "accessor" {
            "fn consumer() { accessor(); }"
        } else {
            "fn consumer() { ACTIVE.try_with(|_| ()); }"
        };
        let (_, pruned) = crate::prune_generated_project_owners("", source, &[consumer])
            .expect("support pruning");
        assert!(
            rust_source_defined_item_names(&pruned).contains("ACTIVE"),
            "{pruned}"
        );
        assert!(!pruned.contains("IDLE"), "{pruned}");
    }
    let (_, empty) = crate::prune_generated_project_owners("", source, &["fn unrelated() {}"])
        .expect("unused support pruning");
    assert!(empty.is_empty(), "{empty}");
}

#[test]
fn task_local_support_macro_only_owner_survives_pruning() {
    let (_, support) = crate::prune_generated_project_owners(
        "",
        "tokio::task_local! { static ACTIVE: String; }",
        &["async fn consumer() { ACTIVE.scope(String::new(), async {}).await; }"],
    )
    .expect("macro-only support owner");
    assert_eq!(locals(&support)[0].name, "ACTIVE");
}

fn async_module(name: &str) -> sifr_ir::HirModule {
    let source = format!(
        "class Resource_{name}:\n\
         \x20   async def __aenter__(self) -> Result[None, ValueError]:\n\
         \x20       return None\n\n\
         \x20   async def __aexit__(self, cause: AsyncExitCause) -> Result[None, ValueError]:\n\
         \x20       return None\n\n\
         async def {name}() -> Result[None, ValueError]:\n\
         \x20   async with Resource_{name}():\n\
         \x20       await task.sleep(0.0)\n\
         \x20   return None\n"
    );
    sifr_lowering::lower_module(
        sifr_python_parser::parse_module(&source)
            .expect("parse async source")
            .suite(),
    )
    .expect("lower async source")
    .module
}

fn assert_cancellation_owner(prelude: &str) {
    let file = syn::parse_file(prelude).expect("project prelude");
    let support = file
        .items
        .into_iter()
        .find_map(|item| match item {
            syn::Item::Mod(module) if module.ident == "__sifr_generated_support" => module.content,
            _ => None,
        })
        .expect("one support owner")
        .1;
    let mut file = syn::parse_file("").expect("empty file");
    file.items = support;
    let declared = locals(&prettyplease::unparse(&file));
    let cancellation = declared
        .iter()
        .filter(|local| local.name == "__SIFR_TASK_CANCELLATION")
        .collect::<Vec<_>>();
    assert_eq!(cancellation.len(), 1, "{prelude}");
    assert_eq!(
        cancellation[0].visibility.to_token_stream().to_string(),
        "pub (crate)"
    );
    assert_eq!(prelude.matches("mod __sifr_generated_support").count(), 1);
}

#[test]
fn task_local_support_binary_project_preserves_cancellation_owner_and_consumers() {
    let main = async_module("main");
    let worker = async_module("work");
    let project = crate::generate_rust_multi_with_metadata(
        &[("main", &main), ("worker", &worker)],
        &crate::StdlibCode::default(),
    );
    assert_cancellation_owner(&project.project_union_prelude);
    let main = &project.rust_files["main"];
    assert!(
        main.split_whitespace()
            .collect::<String>()
            .contains("__SIFR_TASK_CANCELLATION.scope"),
        "{main}"
    );
    assert!(
        main.contains("use crate::__sifr_generated_support::"),
        "{main}"
    );
    assert!(!main.contains("static __SIFR_TASK_CANCELLATION"));
}

#[test]
fn task_local_support_test_project_preserves_cancellation_owner_and_consumers() {
    let support = async_module("main");
    let test = async_module("test_async");
    let project = crate::generate_rust_test_project_with_metadata(
        &[("support", &support)],
        &[("test_async", &test)],
        &crate::StdlibCode::default(),
    );
    assert_cancellation_owner(&project.project_union_prelude);
    let support = &project.support_rust_files["support"];
    assert!(
        support
            .split_whitespace()
            .collect::<String>()
            .contains("__SIFR_TASK_CANCELLATION.scope"),
        "{support}"
    );
    assert!(
        support.contains("use crate::__sifr_generated_support::"),
        "{support}"
    );
    assert!(!support.contains("static __SIFR_TASK_CANCELLATION"));
}

#[test]
fn task_local_support_sync_projects_emit_no_cancellation_owner() {
    let module = sifr_lowering::lower_module(
        sifr_python_parser::parse_module("def main() -> None:\n    pass\n")
            .expect("parse sync source")
            .suite(),
    )
    .expect("lower sync source")
    .module;
    let normal = crate::generate_rust_multi_with_metadata(
        &[("main", &module)],
        &crate::StdlibCode::default(),
    );
    let tests = crate::generate_rust_test_project_with_metadata(
        &[],
        &[("test_sync", &module)],
        &crate::StdlibCode::default(),
    );
    assert!(!normal.project_union_prelude.contains("task_local"));
    assert!(!tests.project_union_prelude.contains("task_local"));
}

#[test]
#[should_panic(expected = "invalid compiler-owned tokio::task_local declaration")]
fn task_local_support_rejects_invalid_owned_macro_syntax() {
    crate::crate_visible_generated_support_source("tokio::task_local! { static BAD: u64 = 1; }");
}
