use crate::canonicalize_generated_rust_source;

#[test]
fn support_impls_do_not_define_builtin_import_bindings() {
    let support = r#"
        trait Length { fn length(&self) -> usize; }
        impl Length for String { fn length(&self) -> usize { self.len() } }
        trait Number { fn number(&self) -> f64; }
        impl Number for f64 { fn number(&self) -> f64 { *self } }
        struct Local;
        impl Local { fn make() -> Self { Self } }
    "#;
    let names = crate::stdlib_filter::rust_source_defined_item_names(support);
    assert!(!names.contains("String"));
    assert!(!names.contains("f64"));
    assert!(names.contains("Local"));
    assert!(names.contains("Length"));
    let imports = crate::generated_visibility::generated_support_import(
        "fn consume(value: String, number: f64) -> Local { Local::make() }",
        support,
    );
    assert!(imports.contains("Local"), "{imports}");
    assert!(!imports.contains("String"), "{imports}");
    assert!(!imports.contains("f64"), "{imports}");
}

#[test]
fn nominal_method_imports_root_union_without_generated_support() {
    let prelude = r#"
        enum RootUnion { Value(i64) }
        mod __sifr_project_nominals {
            pub struct Value;
            impl Value { pub fn get(&self) -> RootUnion { RootUnion::Value(1) } }
        }
        pub use __sifr_project_nominals::Value;
    "#;
    let output = crate::import_root_bindings_in_project_nominals(prelude).unwrap();
    let parsed = syn::parse_file(&output).unwrap();
    let module = parsed
        .items
        .iter()
        .find_map(|item| match item {
            syn::Item::Mod(module) => Some(module),
            _ => None,
        })
        .unwrap();
    let items = &module.content.as_ref().unwrap().1;
    assert!(matches!(&items[0], syn::Item::Use(_)), "{output}");
    assert!(output.contains("use crate::RootUnion;"), "{output}");
    assert!(!output.contains("use crate::Value;"), "{output}");
    assert_eq!(
        crate::import_root_bindings_in_project_nominals(&output).unwrap(),
        output
    );
}

#[test]
fn allocating_unwrap_defaults_remain_lazy() {
    let output = canonicalize_generated_rust_source(
        r#"
        fn allocated(value: Option<String>) -> String {
            value.unwrap_or_else(|| "Context".to_string())
        }
        fn constant(value: Option<i64>) -> i64 { value.unwrap_or_else(|| 42) }
        fn main() { println!("{} {}", allocated(None), constant(None)); }
    "#,
    )
    .unwrap();
    assert!(
        output.contains("unwrap_or_else(|| \"Context\".to_string())"),
        "{output}"
    );
    assert!(output.contains("unwrap_or(42)"), "{output}");
}

#[test]
fn unused_local_trait_forwarders_do_not_demand_themselves() {
    let output = canonicalize_generated_rust_source(
        r#"
        struct Foreign;
        impl Foreign {
            fn unused_external(&self) -> bool { false }
            fn retained_external(&self) -> bool { true }
        }
        trait __SifrOpaqueForeignMethods {
            fn entry(&self) -> bool;
            fn helper(&self) -> bool;
            fn unused_external(&self) -> bool;
        }
        impl __SifrOpaqueForeignMethods for Foreign {
            fn entry(&self) -> bool { self.helper() }
            fn helper(&self) -> bool { self.retained_external() }
            fn unused_external(&self) -> bool { self.unused_external() }
        }
        fn main() { println!("{}", Foreign.entry()); }
    "#,
    )
    .unwrap();
    assert!(output.contains("fn entry("), "{output}");
    assert!(output.contains("fn helper("), "{output}");
    assert!(output.contains("fn retained_external("), "{output}");
    assert!(!output.contains("fn unused_external("), "{output}");
}

#[test]
fn opaque_trait_and_nominal_implementation_share_only_reachable_methods() {
    let output = canonicalize_generated_rust_source(
        r#"
        mod support {
            pub trait __SifrOpaqueForeignMethods {
                fn search(&self) -> bool;
                fn unused(&self) -> bool;
            }
        }
        mod nominals {
            use crate::support::__SifrOpaqueForeignMethods;
            pub struct Foreign;
            impl __SifrOpaqueForeignMethods for Foreign {
                fn search(&self) -> bool { true }
                fn unused(&self) -> bool { false }
            }
        }
        use support::__SifrOpaqueForeignMethods;
        pub use nominals::Foreign;
        fn main() { let value: Foreign = Foreign; let found = value.search(); println!("{}", found); }
    "#,
    )
    .unwrap();
    assert!(output.contains("fn search("), "{output}");
    assert!(!output.contains("fn unused("), "{output}");
}

#[test]
fn proven_inherent_calls_do_not_require_same_named_extension_traits() {
    let support = "trait Extension { fn search(&self); }";
    let mut inherent = crate::stdlib_filter::InherentMethods::new();
    crate::stdlib_filter::collect_inherent_methods(
        &syn::parse_file("mod sifr_generated_project_nominals { struct SifrGeneratedStdlibWrapper; impl SifrGeneratedStdlibWrapper { fn search(&self) {} } }").unwrap(),
        &mut inherent,
    );
    let required = crate::stdlib_filter::rust_source_required_trait_names_with_inherent(
        "fn main() { let value: SifrGeneratedStdlibWrapper = SifrGeneratedStdlibWrapper; value.search(); }",
        support,
        &inherent,
    )
    .unwrap();
    assert!(required.is_empty(), "{required:?}");
    let required = crate::stdlib_filter::rust_source_required_trait_names_with_inherent(
        "fn run(value: Foreign) { value.search(); }",
        support,
        &inherent,
    )
    .unwrap();
    assert!(required.contains("Extension"));
}

#[test]
fn shadowed_and_macro_receiver_calls_keep_required_extension_imports() {
    let support = "trait Extension { fn search(&self); }";
    let inherent = crate::stdlib_filter::InherentMethods::from([(
        "Wrapper".to_string(),
        "search".to_string(),
    )]);
    for source in [
        "fn run(value: Wrapper) { { let value = foreign(); value.search(); } }",
        "fn run(value: Wrapper) { let callback = |value| value.search(); }",
        "fn run(value: Wrapper) { match foreign() { Some(value) => value.search(), _ => () } }",
        "fn run(value: Wrapper) { for value in foreign() { value.search(); } }",
        "fn run(value: Wrapper) { if let Some(value) = foreign() { value.search(); } }",
        "fn run(value: Foreign) { println!(\"{}\", value.search()); }",
    ] {
        let required = crate::stdlib_filter::rust_source_required_trait_names_with_inherent(
            source, support, &inherent,
        )
        .unwrap();
        assert!(required.contains("Extension"), "{source}");
    }
}

#[test]
fn unrelated_user_type_basenames_do_not_prove_inherent_dispatch() {
    let source = syn::parse_file("mod first { struct Wrapper; impl Wrapper { fn search(&self) {} } } mod second { struct Wrapper; }").unwrap();
    let mut inherent = crate::stdlib_filter::InherentMethods::new();
    crate::stdlib_filter::collect_inherent_methods(&source, &mut inherent);
    assert!(inherent.is_empty());
    let required = crate::stdlib_filter::rust_source_required_trait_names_with_inherent(
        "fn run(value: Wrapper) { value.search(); }",
        "trait Extension { fn search(&self); }",
        &inherent,
    )
    .unwrap();
    assert!(required.contains("Extension"));
}

#[test]
fn opaque_methods_referenced_only_in_custom_macro_syntax_remain_live() {
    let output = canonicalize_generated_rust_source(r#"
        mod support {
            pub trait __SifrOpaqueForeignMethods { fn search(&self) -> bool; fn unused(&self) -> bool; }
        }
        mod nominals {
            use crate::support::__SifrOpaqueForeignMethods;
            pub struct Foreign;
            impl __SifrOpaqueForeignMethods for Foreign {
                fn search(&self) -> bool { true }
                fn unused(&self) -> bool { false }
            }
        }
        pub use nominals::Foreign;
        use support::__SifrOpaqueForeignMethods;
        fn main() {
            let value: Foreign = Foreign;
            custom_select! { biased; result = value.search() => { consume(result); } }
        }
    "#).unwrap();
    assert!(output.contains("fn search("), "{output}");
    assert!(!output.contains("fn unused("), "{output}");
}

#[test]
fn ordinary_bindings_do_not_demand_opaque_methods() {
    let output = canonicalize_generated_rust_source(r#"
        mod support {
            pub trait __SifrOpaqueForeignMethods { fn search(&self) -> bool; fn pattern(&self) -> bool; }
        }
        mod nominals {
            use crate::support::__SifrOpaqueForeignMethods;
            pub struct Foreign;
            impl __SifrOpaqueForeignMethods for Foreign {
                fn search(&self) -> bool { true }
                fn pattern(&self) -> bool { false }
            }
        }
        pub use nominals::Foreign;
        use support::__SifrOpaqueForeignMethods;
        fn main() {
            let pattern = true;
            println!("{} {}", Foreign.search(), pattern);
        }
    "#).unwrap();
    assert!(output.contains("fn search("), "{output}");
    assert!(!output.contains("fn pattern("), "{output}");
}

#[test]
fn external_type_paths_and_sibling_trait_parameters_do_not_prove_inherent_dispatch() {
    let inherent = crate::stdlib_filter::InherentMethods::from([(
        "SifrGeneratedStdlibWrapper".to_string(),
        "search".to_string(),
    )]);
    for source in [
        "fn run(value: external::SifrGeneratedStdlibWrapper) { value.search(); }",
        "fn run(value: ::SifrGeneratedStdlibWrapper) { value.search(); }",
        "static value: Foreign = Foreign; trait User { fn first(value: SifrGeneratedStdlibWrapper); fn second() { value.search(); } }",
    ] {
        let required = crate::stdlib_filter::rust_source_required_trait_names_with_inherent(
            source,
            "trait Extension { fn search(&self); }",
            &inherent,
        )
        .unwrap();
        assert!(required.contains("Extension"), "{source}");
    }
}

#[test]
fn qualified_receiver_method_paths_preserve_opaque_demand() {
    let output = canonicalize_generated_rust_source(r#"
        mod support {
            pub trait __SifrOpaqueForeignMethods { fn search(&self) -> bool; fn unused(&self) -> bool; }
        }
        mod nominals {
            use crate::support::__SifrOpaqueForeignMethods;
            pub struct Foreign;
            impl __SifrOpaqueForeignMethods for Foreign {
                fn search(&self) -> bool { true }
                fn unused(&self) -> bool { false }
            }
        }
        pub use nominals::Foreign;
        use support::__SifrOpaqueForeignMethods;
        fn main() { let value: Foreign = Foreign; let found = <Foreign>::search(&value); println!("{}", found); }
    "#).unwrap();
    assert!(output.contains("fn search("), "{output}");
    assert!(!output.contains("fn unused("), "{output}");
}

#[test]
fn unused_empty_generated_traits_and_their_impls_disappear() {
    let output = canonicalize_generated_rust_source(
        r#"
        mod support {
            trait __SifrAdd: Sized {}
            impl __SifrAdd for String {}
            pub fn consume(value: String) { println!("{}", value); }
        }
        fn main() { support::consume("value".to_string()); }
    "#,
    )
    .unwrap();
    assert!(!output.contains("SifrGeneratedAdd"), "{output}");
    assert!(output.contains("fn consume("), "{output}");
}

#[test]
fn empty_generated_traits_retain_transitive_bound_contracts() {
    let output = canonicalize_generated_rust_source(
        r#"
        struct Value;
        trait __SifrBase {}
        trait __SifrLeaf: __SifrBase {}
        impl __SifrBase for Value {}
        impl __SifrLeaf for Value {}
        fn consume<T: __SifrLeaf>(value: &T) { println!("used"); }
        fn main() { consume(&Value); }
    "#,
    )
    .unwrap();
    assert!(output.contains("trait SifrGeneratedBase"), "{output}");
    assert!(output.contains("trait SifrGeneratedLeaf"), "{output}");
    assert!(
        output.contains("impl SifrGeneratedBase for Value"),
        "{output}"
    );
}

#[test]
fn mutable_apis_do_not_acquire_spurious_must_use_contracts() {
    let output = canonicalize_generated_rust_source(
        r#"
        pub fn mutate(values: &mut Vec<i64>) -> Option<i64> { values.pop() }
        pub fn observe(values: &[i64]) -> Option<&i64> { values.first() }
        #[must_use]
        pub fn explicit(values: &mut Vec<i64>) -> Option<i64> { values.pop() }
    "#,
    )
    .unwrap();
    let file = syn::parse_file(&output).unwrap();
    for item in file.items {
        if let syn::Item::Fn(function) = item {
            let must_use = function
                .attrs
                .iter()
                .any(|attribute| attribute.path().is_ident("must_use"));
            assert_eq!(must_use, function.sig.ident != "mutate", "{output}");
        }
    }
}

#[test]
fn borrowed_tuple_field_getters_are_const_without_owned_drop_or_deref_assumptions() {
    let output = canonicalize_generated_rust_source(
        r#"
        pub fn borrowed(item: &(f64, String)) -> f64 { item.0 }
        pub fn owned(item: (f64, String)) -> f64 { item.0 }
        pub fn boxed(item: &Box<(f64, String)>) -> f64 { item.0 }
        pub fn shadow(item: &(f64, String), boxed: &Box<(f64, String)>) -> f64 {
            let item = boxed;
            item.0
        }
    "#,
    )
    .unwrap();
    let file = syn::parse_file(&output).unwrap();
    for item in file.items {
        if let syn::Item::Fn(function) = item {
            assert_eq!(
                function.sig.constness.is_some(),
                function.sig.ident == "borrowed",
                "{output}"
            );
        }
    }
}
