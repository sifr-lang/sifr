use crate::canonicalize_generated_rust_source;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

fn canonical_and_compile(source: &str) -> String {
    let canonical = canonicalize_generated_rust_source(source).expect("canonical Rust");
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "sifr-record-variant-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&directory).expect("unique native evidence directory");
    let mut child = Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-name=record_variant_demand",
            "--crate-type=lib",
            "--emit=metadata",
            "--out-dir",
        ])
        .arg(&directory)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rustc");
    child
        .stdin
        .take()
        .expect("compiler stdin")
        .write_all(canonical.as_bytes())
        .expect("write canonical Rust");
    let result = child.wait_with_output().expect("compiler result");
    assert!(
        result.status.success(),
        "{}\n{canonical}",
        String::from_utf8_lossy(&result.stderr)
    );
    canonical
}

#[test]
fn record_variant_demand_preserves_nested_macro_constructors() {
    let canonical = canonical_and_compile(
        r#"
        enum Part { Literal { value: String }, Nested { child: Box<Part> }, Dead }
        fn main() {
            let parts = vec![Part::Nested {
                child: Box::new(Part::Literal { value: String::from("live") })
            }];
            std::hint::black_box(parts);
        }
        "#,
    );
    assert!(canonical.contains("Literal {"), "{canonical}");
    assert!(canonical.contains("Nested {"), "{canonical}");
    assert!(!canonical.contains("Dead"), "{canonical}");
}

#[test]
fn record_variant_demand_preserves_self_constructors_inside_macros() {
    let canonical = canonical_and_compile(
        r#"
        enum Part { Live { value: i64 }, Dead }
        impl Part {
            fn make() -> Vec<Self> { vec![Self::Live { value: 7 }] }
        }
        fn main() { std::hint::black_box(Part::make()); }
        "#,
    );
    assert!(canonical.contains("Live {"), "{canonical}");
    assert!(!canonical.contains("Dead"), "{canonical}");
}

#[test]
fn record_variant_demand_preserves_cross_scope_constructors_and_aliases() {
    let canonical = canonical_and_compile(
        r#"
        mod support {
            #[derive(Clone)]
            pub enum Part {
                Direct { value: i64 }, Aliased { value: i64 }, Repeated { value: i64 }, Dead
            }
        }
        use support::Part as ImportedPart;
        fn main() {
            let direct = support::Part::Direct { value: 1 };
            let aliased = vec![ImportedPart::Aliased { value: 2 }];
            let repeated = vec![ImportedPart::Repeated { value: 3 }; 2];
            std::hint::black_box((direct, aliased, repeated));
        }
        "#,
    );
    assert!(canonical.contains("Direct {"), "{canonical}");
    assert!(canonical.contains("Aliased {"), "{canonical}");
    assert!(canonical.contains("Repeated {"), "{canonical}");
    assert!(!canonical.contains("Dead"), "{canonical}");
}

#[test]
fn record_variant_demand_compiles_nested_template_runtime() {
    let source = "def retain(value: Template) -> bool:\n    return True\n\ndef main():\n    precision: str = \"2\"\n    result: bool = retain(t\"score={3.5:.{precision}f}\")\n    assert result\n";
    let parsed = sifr_python_parser::parse_module(source).expect("template source parses");
    let lowered = sifr_lowering::lower_module(parsed.suite()).expect("template source lowers");
    let canonical = canonical_and_compile(&crate::generate_rust(&lowered.module));
    assert!(canonical.contains("SifrGeneratedTemplateFormatSpecPart"));
    assert!(canonical.contains("Literal {"), "{canonical}");
    assert!(canonical.contains("Interpolation {"), "{canonical}");
}

#[test]
fn record_variant_demand_preserves_macro_initializer_effects() {
    let canonical = canonical_and_compile(
        r#"
        struct Carrier { effect: i64, unused: i64 }
        fn next() -> i64 { std::hint::black_box(7) }
        fn main() {
            let values = vec![Carrier { effect: next(), unused: 0 }];
            std::hint::black_box(values);
        }
        "#,
    );
    assert!(canonical.contains("effect: i64"), "{canonical}");
    let tokens = canonical
        .parse::<proc_macro2::TokenStream>()
        .expect("canonical tokens")
        .to_string();
    assert!(tokens.contains("effect : next ()"), "{canonical}");
    assert!(!canonical.contains("unused"), "{canonical}");
}

#[test]
fn removed_union_patterns_are_rewritten_inside_local_and_external_macros() {
    let canonical = canonical_and_compile(
        r#"
        mod support {
            pub enum __SifrUnionExternal { Live(String), Dead(String) }
        }
        use support::__SifrUnionExternal as Imported;
        enum __SifrUnionLocal { Live(String), Dead(String) }
        fn main() {
            assert_eq!(match Imported::Live(String::from("external")) {
                Imported::Live(value) => value,
                Imported::Dead(value) => value,
            }, "external");
            assert_eq!(match __SifrUnionLocal::Live(String::from("local")) {
                __SifrUnionLocal::Live(value) => value,
                __SifrUnionLocal::Dead(value) => value,
            }, "local");
        }
        "#,
    );
    assert!(!canonical.contains("Dead"), "{canonical}");
    assert_eq!(canonical.matches("assert_eq!").count(), 2, "{canonical}");
}

#[test]
fn const_promotion_proves_owned_inputs_and_locals_are_transferred_not_dropped() {
    let canonical = canonical_and_compile(
        r#"
        pub fn discard(_: String) -> i64 { 1 }
        pub fn discard_result(_: Result<String, String>) -> i64 { 2 }
        pub fn discard_option(_: Option<String>) -> i64 { 3 }
        pub fn discard_generic<T>(_: T) -> i64 { 4 }
        pub fn discard_local() -> i64 { let text = String::new(); 5 }
        pub fn discard_alias(value: String) -> i64 { let alias = value; 6 }
        pub fn discard_shadow(value: String) -> i64 { let value = 10; value }
        pub fn transfer(value: String) -> String { value }
        pub fn transfer_local(value: String) -> String { let alias = value; alias }
        pub fn scalar(_: i64) -> i64 { 7 }
        pub fn scalar_alias(value: i64) -> i64 { let alias = value; alias }
        pub fn choose(value: String, flag: bool) -> String {
            if flag { value } else { value }
        }
        pub fn conditional_discard(value: String, flag: bool) -> String {
            if flag { value } else { String::new() }
        }
        pub fn borrowed(_: &String) -> i64 { 8 }
        pub struct Carrier { pub value: String }
        pub fn replace_field(value: Carrier) -> Carrier {
            Carrier { value: String::new(), ..value }
        }
        pub const fn length(_: &String) -> usize { 0 }
        pub fn temporary_borrow() -> usize { length(&String::new()) }
        impl Carrier {
            pub fn new(value: String) -> Self { Self { value } }
            pub fn discard_self(self) -> i64 { 9 }
        }
        "#,
    );
    for name in [
        "discard",
        "discard_result",
        "discard_option",
        "discard_generic",
        "discard_local",
        "discard_alias",
        "discard_shadow",
        "discard_self",
        "conditional_discard",
        "replace_field",
        "temporary_borrow",
    ] {
        assert!(
            !canonical.contains(&format!("const fn {name}(")),
            "{canonical}"
        );
        assert!(
            !canonical.contains(&format!("const fn {name}<")),
            "{canonical}"
        );
    }
    for name in [
        "transfer",
        "transfer_local",
        "scalar",
        "scalar_alias",
        "borrowed",
        "new",
        "choose",
    ] {
        assert!(
            canonical.contains(&format!("const fn {name}(")),
            "{canonical}"
        );
    }
}

#[test]
fn const_promotion_preserves_control_flow_and_checks_every_owner_exit() {
    let canonical = canonical_and_compile(
        r#"
        #[derive(Clone, Copy)]
        pub enum Direction { North, East }
        impl Direction {
            pub fn is_vertical(&self) -> bool {
                match self { Direction::North => true, Direction::East => false }
            }
        }
        pub fn early_scalar(flag: bool) -> i64 {
            if flag { return 1; }
            2
        }
        pub fn early_transfer(value: String, flag: bool) -> String {
            if flag { let alias = value; return alias; }
            value
        }
        pub fn nested_transfer(value: String, first: bool, second: bool) -> String {
            if first {
                if second { return value; }
                let alias = value;
                return alias;
            }
            value
        }
        pub fn match_transfer(value: String, flag: bool) -> String {
            match flag { true => value, false => value }
        }
        pub fn guarded_transfer(value: String, first: bool, second: bool) -> String {
            match first { true if second => return value, _ => {} }
            value
        }
        pub fn borrowed_match(value: &Option<String>, flag: bool) -> i64 {
            match value { Some(_) if flag => 1, Some(_) => 2, None => 0 }
        }
        pub fn early_discard(value: String, flag: bool) -> String {
            if flag { return value; }
            String::new()
        }
        pub fn match_discard(value: String, flag: bool) -> String {
            match flag { true => value, false => String::new() }
        }
        pub fn scoped_discard(flag: bool) -> i64 {
            if flag { let value = String::new(); }
            3
        }
        pub fn statement_discard(flag: bool) -> i64 {
            if flag { String::new(); }
            4
        }
        pub fn temporary_match() -> i64 {
            match &String::new() { _ => 5 }
        }
        pub fn boolean_operator_boundary(value: String, first: bool, second: bool) -> String {
            if first && second { return value; }
            value
        }
        pub fn method_call_boundary(value: &Option<String>) -> i64 {
            if value.is_none() { return 0; }
            1
        }
        "#,
    );
    for name in [
        "is_vertical",
        "early_scalar",
        "early_transfer",
        "nested_transfer",
        "match_transfer",
        "guarded_transfer",
        "borrowed_match",
    ] {
        assert!(
            canonical.contains(&format!("const fn {name}(")),
            "{canonical}"
        );
    }
    for name in [
        "early_discard",
        "match_discard",
        "scoped_discard",
        "statement_discard",
        "temporary_match",
        "boolean_operator_boundary",
        "method_call_boundary",
    ] {
        assert!(
            !canonical.contains(&format!("const fn {name}(")),
            "{canonical}"
        );
    }
}

#[test]
fn const_promotion_preserves_trivial_container_let_else_and_owner_exits() {
    let canonical = canonical_and_compile(
        r#"
        pub fn option_bool(value: Option<bool>) -> bool {
            let Some(value) = value else { return false; };
            value
        }
        pub fn option_float(value: Option<f64>) -> f64 {
            let Some(value) = value else { return 0.0; };
            value
        }
        pub fn result_scalar(value: Result<i64, bool>) -> i64 {
            let Ok(value) = value else { return 0; };
            value
        }
        pub fn nested_container(value: Option<Result<bool, i64>>) -> bool {
            let Some(value) = value else { return false; };
            let Ok(value) = value else { return false; };
            value
        }
        pub fn borrowed_option(value: &Option<String>) -> Option<&String> {
            let Some(value) = value else { return None; };
            Some(value)
        }
        pub fn let_else_transfer(value: String, flag: Option<bool>) -> String {
            let Some(flag) = flag else { return value; };
            if flag { return value; }
            value
        }
        pub fn let_else_discard(value: String, flag: Option<bool>) -> String {
            let Some(flag) = flag else { return String::new(); };
            if flag { return value; }
            value
        }
        pub fn owned_container(value: Option<String>) -> bool {
            let Some(value) = value else { return false; };
            false
        }
        pub fn generic_container<T>(_: Option<T>) -> bool { false }
        pub fn owned_result(_: Result<bool, String>) -> bool { false }
        pub fn shadow_with_owner(value: bool) -> i64 {
            let value = String::new();
            let alias = value;
            1
        }
        "#,
    );
    for name in [
        "option_bool",
        "option_float",
        "result_scalar",
        "nested_container",
        "borrowed_option",
        "let_else_transfer",
    ] {
        assert!(
            canonical.contains(&format!("const fn {name}(")),
            "{canonical}"
        );
    }
    for name in [
        "let_else_discard",
        "owned_container",
        "generic_container",
        "owned_result",
        "shadow_with_owner",
    ] {
        assert!(
            !canonical.contains(&format!("const fn {name}(")),
            "{canonical}"
        );
        assert!(
            !canonical.contains(&format!("const fn {name}<")),
            "{canonical}"
        );
    }
}

#[test]
fn const_promotion_does_not_confuse_shadowed_containers_with_standard_types() {
    let local = canonical_and_compile(
        r#"
        pub struct Option<T>(pub T);
        impl<T> Drop for Option<T> { fn drop(&mut self) {} }
        pub fn local_container(_: Option<bool>) -> i64 { 0 }
        pub fn standard_container(value: ::core::option::Option<bool>) -> bool {
            let Some(value) = value else { return false; };
            value
        }
        "#,
    );
    assert!(!local.contains("const fn local_container("), "{local}");
    assert!(local.contains("const fn standard_container("), "{local}");
    let imported = canonical_and_compile(
        r#"
        use std::vec::Vec as Option;
        pub fn imported_container(_: Option<bool>) -> i64 { 0 }
        "#,
    );
    assert!(
        !imported.contains("const fn imported_container("),
        "{imported}"
    );
}

#[test]
fn option_question_mark_requires_owned_values_and_preserves_borrowing_patterns() {
    use syn::visit::{self, Visit};
    let canonical = canonical_and_compile(
        r#"
        pub fn owned(value: Option<String>) -> Option<String> {
            let Some(value) = value else { return None; };
            Some(value)
        }
        pub fn owned_alias(value: Option<String>) -> Option<String> {
            let alias = value;
            let Some(value) = alias else { return None; };
            Some(value)
        }
        pub fn shared_view(value: Option<String>) -> Option<String> {
            let Some(value) = value.as_ref() else { return None; };
            Some(value.to_owned())
        }
        pub fn mutable_view(mut value: Option<String>) -> Option<String> {
            let Some(value) = value.as_mut() else { return None; };
            Some(value.to_owned())
        }
        pub fn borrowed(value: &Option<String>) -> Option<&String> {
            let Some(value) = value else { return None; };
            Some(value)
        }
        pub fn borrowed_alias(value: &Option<String>) -> Option<&String> {
            let alias = value;
            let Some(value) = alias else { return None; };
            Some(value)
        }
        pub fn borrowed_typed(value: &Option<String>) -> Option<&String> {
            let alias: &Option<String> = value;
            let Some(value) = alias else { return None; };
            Some(value)
        }
        pub fn block_shadow(value: Option<String>, borrowed: &Option<String>) -> Option<&String> {
            let value = borrowed;
            let Some(value) = value else { return None; };
            Some(value)
        }
        pub fn closure_shadow<'a>(value: Option<String>, borrowed: &'a Option<String>) -> Option<&'a String> {
            let inspect = |value: &'a Option<String>| {
                let Some(value) = value else { return None; };
                Some(value)
            };
            inspect(borrowed)
        }
        pub fn by_ref_pattern(value: Option<String>) -> Option<usize> {
            let Some(ref value) = value else { return None; };
            Some(value.len())
        }
        pub fn supplier(value: &Option<String>) -> &Option<String> { value }
        pub fn borrowed_call(value: &Option<String>) -> Option<&String> {
            let Some(value) = supplier(value) else { return None; };
            Some(value)
        }
        "#,
    );
    struct Tries(usize);
    impl<'ast> Visit<'ast> for Tries {
        fn visit_expr_try(&mut self, expression: &'ast syn::ExprTry) {
            self.0 += 1;
            visit::visit_expr_try(self, expression);
        }
    }
    let file = syn::parse_file(&canonical).expect("canonical file");
    let mut observed = std::collections::HashMap::new();
    for item in file.items {
        if let syn::Item::Fn(function) = item {
            let mut tries = Tries(0);
            tries.visit_block(&function.block);
            observed.insert(function.sig.ident.to_string(), tries.0);
        }
    }
    for name in ["owned", "owned_alias", "shared_view", "mutable_view"] {
        assert_eq!(observed.get(name), Some(&1), "{canonical}");
    }
    for name in [
        "borrowed",
        "borrowed_alias",
        "borrowed_typed",
        "block_shadow",
        "closure_shadow",
        "by_ref_pattern",
        "borrowed_call",
    ] {
        assert_eq!(observed.get(name), Some(&0), "{canonical}");
    }
    let competing = canonical_and_compile(
        r#"
        pub trait View { fn as_ref(&self) -> &Option<String>; }
        impl View for &Option<String> { fn as_ref(&self) -> &Option<String> { self } }
        pub fn competing_view(value: &Option<String>) -> Option<&String> {
            let Some(value) = value.as_ref() else { return None; };
            Some(value)
        }
        "#,
    );
    let mut tries = Tries(0);
    tries.visit_file(&syn::parse_file(&competing).expect("competing method source"));
    assert_eq!(tries.0, 0, "{competing}");
}

#[test]
fn option_question_mark_preserves_clone_and_declared_lookup_ownership() {
    use syn::visit::{self, Visit};
    let canonical = canonical_and_compile(
        r#"
        mod data {
            use ::std::collections::HashMap as Table;
            pub struct Registry { pub values: Table<String, String> }
            impl Registry {
                pub fn lookup(&self, key: &str) -> Option<String> {
                    let Some(value) = self.values.get(key) else { return None; };
                    Some(value.clone())
                }
            }
        }
        pub use data::Registry as RegistryAlias;
        pub fn alias_lookup(value: &RegistryAlias, key: &str) -> Option<String> {
            let Some(found) = value.values.get(key) else { return None; };
            Some(found.clone())
        }
        pub fn clone_owned(value: Option<String>) -> Option<String> {
            let Some(found) = value.clone() else { return None; };
            Some(found)
        }
        pub fn clone_alias(value: Option<String>) -> Option<String> {
            let alias = value;
            let Some(found) = alias.clone() else { return None; };
            Some(found)
        }
        pub fn sequence_lookup(value: &[String], index: usize) -> Option<&String> {
            let Some(found) = value.get(index) else { return None; };
            Some(found)
        }
        pub fn mutable_lookup<'a>(value: &'a mut ::std::collections::BTreeMap<String, String>, key: &str) -> Option<&'a mut String> {
            let Some(found) = value.get_mut(key) else { return None; };
            Some(found)
        }
        pub struct NotClone(pub String);
        pub fn cloned_reference(value: &Option<NotClone>) -> Option<&NotClone> {
            let Some(found) = value.clone() else { return None; };
            Some(found)
        }
        pub type Borrowed<'a> = &'a Option<NotClone>;
        pub fn reference_alias(value: Borrowed<'_>) -> Option<&NotClone> {
            let Some(found) = value else { return None; };
            Some(found)
        }
        use ::std::collections::HashMap;
        pub fn local_lookup_shadow(value: &Option<String>) -> Option<&String> {
            struct HashMap<'a>(&'a Option<String>);
            impl<'a> HashMap<'a> {
                fn get(&self, _: usize) -> &'a Option<String> { self.0 }
            }
            let map: HashMap<'_> = HashMap(value);
            let Some(found) = map.get(0) else { return None; };
            Some(found)
        }
        "#,
    );
    #[derive(Default)]
    struct Tries(usize);
    impl<'ast> Visit<'ast> for Tries {
        fn visit_expr_try(&mut self, expression: &'ast syn::ExprTry) {
            self.0 += 1;
            visit::visit_expr_try(self, expression);
        }
    }
    #[derive(Default)]
    struct Functions(std::collections::HashMap<String, usize>);
    impl<'ast> Visit<'ast> for Functions {
        fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
            let mut count = Tries::default();
            count.visit_block(&function.block);
            self.0.insert(function.sig.ident.to_string(), count.0);
            visit::visit_item_fn(self, function);
        }
        fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
            let mut count = Tries::default();
            count.visit_block(&function.block);
            self.0.insert(function.sig.ident.to_string(), count.0);
            visit::visit_impl_item_fn(self, function);
        }
    }
    let mut functions = Functions::default();
    functions.visit_file(&syn::parse_file(&canonical).expect("canonical source"));
    for name in [
        "lookup",
        "alias_lookup",
        "clone_owned",
        "clone_alias",
        "sequence_lookup",
        "mutable_lookup",
    ] {
        assert_eq!(functions.0.get(name), Some(&1), "{name}: {canonical}");
    }
    for name in ["cloned_reference", "reference_alias", "local_lookup_shadow"] {
        assert_eq!(functions.0.get(name), Some(&0), "{name}: {canonical}");
    }
}

#[test]
fn option_question_mark_rejects_competing_clone_and_opaque_imports() {
    use syn::visit::{self, Visit};
    let canonical = canonical_and_compile(
        r#"
        pub struct NotClone(pub String);
        pub trait BorrowingClone { fn clone(&self) -> &Option<NotClone>; }
        impl BorrowingClone for Option<NotClone> {
            fn clone(&self) -> &Option<NotClone> { self }
        }
        pub fn clone_returns_reference(value: Option<NotClone>) -> Option<usize> {
            let Some(found) = value.clone() else { return None; };
            Some(found.0.len())
        }
        "#,
    );
    #[derive(Default)]
    struct Tries(usize);
    impl<'ast> Visit<'ast> for Tries {
        fn visit_expr_try(&mut self, expression: &'ast syn::ExprTry) {
            self.0 += 1;
            visit::visit_expr_try(self, expression);
        }
    }
    let mut count = Tries::default();
    count.visit_file(&syn::parse_file(&canonical).expect("canonical source"));
    assert_eq!(count.0, 0, "{canonical}");
    // An unavailable external trait cannot supply receiver ownership evidence.
    // This checks the fail-closed syntax decision, not an invented native package.
    let opaque = canonicalize_generated_rust_source(
        r#"
        use unknown_external::Extension;
        pub fn inspect(value: Option<String>) -> Option<String> {
            let Some(found) = value.clone() else { return None; };
            Some(found)
        }
        "#,
    )
    .expect("opaque-import syntax");
    let mut count = Tries::default();
    count.visit_file(&syn::parse_file(&opaque).expect("opaque syntax"));
    assert_eq!(count.0, 0, "{opaque}");
}
