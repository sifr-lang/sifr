use super::{import_project_prelude_bindings, prune_generated_project_owners};

#[test]
fn project_nominal_methods_retain_their_transitive_support_helpers() {
    let prelude = r#"
        mod __sifr_project_nominals {
            pub struct Child;
            impl Child {
                pub async fn wait(&self) -> i64 {
                    process_async_wait().await
                }
            }
        }
        pub use __sifr_project_nominals::Child;
    "#;
    let support = r#"
        async fn process_async_wait() -> i64 {
            status_from_parts()
        }
        fn status_from_parts() -> i64 {
            0
        }
        fn unused_support() {}
    "#;
    let body = "async fn run(child: Child) -> i64 { child.wait().await }";

    let (pruned_prelude, pruned_support) =
        prune_generated_project_owners(prelude, support, &[body])
            .expect("generated owners should prune");

    assert!(pruned_prelude.contains("pub async fn wait"));
    assert!(pruned_support.contains("async fn process_async_wait"));
    assert!(pruned_support.contains("fn status_from_parts"));
    assert!(!pruned_support.contains("fn unused_support"));
}

#[test]
fn consumer_method_calls_retain_support_trait_contracts() {
    let support = r#"
        struct Wrapper;
        trait RenderSupport {
            fn render(&self) -> i64;
        }
        impl RenderSupport for Wrapper {
            fn render(&self) -> i64 { 1 }
        }
        fn unused_support() {}
    "#;
    let body = "fn run(value: Wrapper) -> i64 { value.render() }";

    let (_, pruned_support) = prune_generated_project_owners("", support, &[body])
        .expect("generated owners should prune");

    assert!(pruned_support.contains("trait RenderSupport"));
    assert!(pruned_support.contains("impl RenderSupport for Wrapper"));
    assert!(!pruned_support.contains("fn unused_support"));
}

#[test]
fn support_imports_only_referenced_crate_root_reexports() {
    let prelude = "mod nominals { pub struct ParseError; pub struct Other; } pub use nominals::{ParseError, Other};";
    let support = "pub(crate) fn parse() -> ParseError { ParseError }";

    let imported = import_project_prelude_bindings(prelude, support)
        .expect("generated owners should parse");

    assert!(imported.contains("use crate::ParseError;"));
    assert!(!imported.contains("crate::Other"));
}

#[test]
fn support_does_not_import_already_qualified_root_reexports() {
    let prelude = "mod nominals { pub struct ParseError; } pub use nominals::ParseError;";
    let support = "pub(crate) fn parse() -> crate::ParseError { crate::ParseError }";

    let imported = import_project_prelude_bindings(prelude, support)
        .expect("generated owners should parse");

    assert_eq!(imported, support);
}

#[test]
fn finalized_prelude_imports_include_transitive_types_in_non_union_consumers() {
    let prelude = "pub struct BinaryHandle; mod nominals { pub struct Encoding; pub struct Unused; } pub use nominals::{Encoding, Unused};";
    let body = "pub fn open() { let value = BinaryHandle; let encoding = Encoding; sink(value, encoding); }";
    let imported = import_project_prelude_bindings(prelude, body).expect("final owner imports");
    assert!(
        imported.contains("use crate::{BinaryHandle, Encoding};"),
        "{imported}"
    );
    assert!(!imported.contains("Unused"));
    assert_eq!(
        import_project_prelude_bindings(prelude, &imported).expect("idempotent imports"),
        imported
    );
}

#[test]
fn finalized_prelude_imports_respect_local_declarations_imports_and_type_parameters() {
    let prelude = "pub struct Encoding; pub struct BinaryHandle;";
    for body in [
        "struct Encoding; pub fn read(value: Encoding) { sink(value); }",
        "use dependency::Encoding; pub fn read(value: Encoding) { sink(value); }",
        "pub fn read<Encoding>(value: Encoding) { sink(value); }",
        "pub fn read() { let BinaryHandle = 1; sink(BinaryHandle); }",
        "mod child { pub fn read(value: Encoding) { sink(value); } }",
    ] {
        assert_eq!(
            import_project_prelude_bindings(prelude, body).expect("lexical ownership"),
            body
        );
    }
}
