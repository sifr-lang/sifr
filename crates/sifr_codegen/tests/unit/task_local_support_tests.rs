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
    let mut cancellation = Vec::new();
    for item in &file.items {
        if let syn::Item::Macro(item) = item
            && item
                .mac
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                == ["tokio", "task_local"]
        {
            let tokens = item.mac.tokens.clone().into_iter().collect::<Vec<_>>();
            for (index, pair) in tokens.windows(2).enumerate() {
                if matches!(&pair[0], proc_macro2::TokenTree::Ident(name) if name == "static")
                    && matches!(&pair[1], proc_macro2::TokenTree::Ident(name) if name == "__SIFR_TASK_CANCELLATION")
                {
                    cancellation.push(index > 0 && matches!(&tokens[index - 1], proc_macro2::TokenTree::Ident(name) if name == "pub"));
                }
            }
        }
    }
    assert_eq!(cancellation.len(), 1, "{prelude}");
    assert!(
        cancellation[0],
        "cancellation declaration must be public: {prelude}"
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
    )
    .expect("project generation should succeed");
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
    )
    .expect("project generation should succeed");
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
    )
    .expect("project generation should succeed");
    let tests = crate::generate_rust_test_project_with_metadata(
        &[],
        &[("test_sync", &module)],
        &crate::StdlibCode::default(),
    )
    .expect("project generation should succeed");
    assert!(!normal.project_union_prelude.contains("task_local"));
    assert!(!tests.project_union_prelude.contains("task_local"));
}

#[test]
#[should_panic(expected = "invalid compiler-owned tokio::task_local declaration")]
fn task_local_support_rejects_invalid_owned_macro_syntax() {
    crate::crate_visible_generated_support_source(
        "tokio::task_local! { static BAD: u64 = 1; }",
        &[],
    );
}
