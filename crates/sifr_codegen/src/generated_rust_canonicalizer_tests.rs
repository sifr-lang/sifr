use super::{
    IdentifierCollector, canonicalize_generated_rust_source,
    finalize_formatted_generated_rust_source,
};
use syn::visit::Visit;

#[test]
fn canonicalizes_declarations_references_and_macro_tokens() {
    let source = r#"
        struct __Widget { pub _value: i64 }
        fn __make(_input: i64) -> __Widget {
            let __result = __Widget { _value: _input };
            assert_eq!(__result._value, _input);
            __result
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("valid generated Rust should canonicalize");

    let parsed = syn::parse_file(&canonical).expect("canonical Rust should parse");
    let mut identifiers = IdentifierCollector::default();
    identifiers.visit_file(&parsed);
    assert!(
        ["__Widget", "_value", "_input", "__result"]
            .iter()
            .all(|name| !identifiers.names.contains(*name))
    );
    assert!(canonical.contains("SifrGeneratedWidget"));
    assert!(canonical.contains("pub value: i64"));
    assert!(canonical.contains("sifr_generated_input"));
    assert!(canonical.contains("sifr_generated_result.value"));
}

#[test]
fn removes_redundant_generated_struct_field_prefixes_without_renaming_values() {
    let source = r#"
        struct __Handle {
            pub _handle: i64,
            pub _mode: String,
            pub _closed: bool,
        }
        fn __make(_handle: i64, mode: String) -> __Handle {
            __Handle { _handle, _mode: mode, _closed: false }
        }
        fn __close(value: &mut __Handle) {
            value._closed = true;
            println!("{}", value._handle);
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("type-owned fields should not retain the global generated namespace");

    assert!(canonical.contains("pub handle: i64"), "{canonical}");
    assert!(canonical.contains("pub mode: String"), "{canonical}");
    assert!(canonical.contains("pub closed: bool"), "{canonical}");
    assert!(
        canonical.contains("handle: sifr_generated_handle"),
        "{canonical}"
    );
    assert!(canonical.contains("value.closed = true"), "{canonical}");
    assert!(canonical.contains("value.handle"), "{canonical}");
}

#[test]
fn disambiguates_similar_function_parameters_and_their_references() {
    let source = r#"
        pub fn calendar(tm_mon: i64, tm_min: i64, tm_mday: i64, tm_wday: i64, tm_yday: i64) -> i64 {
            tm_mon + tm_min + tm_mday + tm_wday + tm_yday
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("similar source parameter names should become unambiguous Rust bindings");

    assert!(canonical.contains("tm_min_argument_"), "{canonical}");
    assert!(canonical.contains("tm_mday_argument_"), "{canonical}");
    assert!(canonical.contains("tm_wday_argument_"), "{canonical}");
    assert!(canonical.contains("tm_yday_argument_"), "{canonical}");
    assert!(
        canonical.matches("tm_min_argument_").count() >= 2,
        "{canonical}"
    );
}

#[test]
fn preserves_literals_comments_and_the_wildcard_pattern() {
    let source = r#"
        // __comment stays untouched
        fn demo(value: Option<i64>) {
            let _ = value;
            let text = "__literal stays untouched";
            println!("{text}");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("valid generated Rust should canonicalize");

    assert!(
        canonical.contains("// __comment stays untouched"),
        "{canonical}"
    );
    assert!(canonical.contains("\"__literal stays untouched\""));
    assert!(canonical.contains("let _ = value;"));
}

#[test]
fn escapes_the_reserved_namespace_injectively() {
    let source = r#"
        fn sifr_generated_x() {}
        fn _x() {}
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("valid generated Rust should canonicalize");
    let functions = syn::parse_file(&canonical)
        .expect("canonical Rust should parse")
        .items
        .into_iter()
        .filter_map(|item| match item {
            syn::Item::Fn(function) => Some(function.sig.ident.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(functions.len(), 2);
    assert_ne!(functions[0], functions[1]);
    assert!(
        functions
            .iter()
            .all(|name| name.starts_with("sifr_generated_"))
    );
    assert!(functions.iter().any(|name| name == "sifr_generated_x"));
    assert!(
        functions
            .iter()
            .any(|name| name.starts_with("sifr_generated_x_user_"))
    );
}

#[test]
fn improves_and_prunes_a_closed_generated_binary_structurally() {
    let source = r#"
        #[derive(PartialEq)]
        pub struct __Thing { value: i64 }

        impl __Thing {
            pub fn new(value: i64) -> Self { Self { value } }
            pub fn value(&self) -> i64 { self.value }
            pub fn fail(&self) -> Result<(), String> { Ok(()) }
            pub fn dead(&self) -> i64 { 0 }
        }

        fn __dead_function() {}
        fn external_name_collision() {}

        struct __DeadDisplay;
        impl std::fmt::Display for __DeadDisplay {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "dead")
            }
        }

        fn main() {
            let __thing = __Thing::new(1);
            ::external_crate::external_name_collision();
            let _result: () = (|| { println!("{}", __thing.value()); })();
            let _ = __thing.fail();
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("valid closed generated binary should canonicalize");

    assert!(!canonical.contains("dead_function"));
    assert!(!canonical.contains("fn external_name_collision"));
    assert!(!canonical.contains("DeadDisplay"));
    assert!(!canonical.contains("fn dead"));
    assert!(canonical.contains("fn value"));
    assert!(canonical.contains("#[derive(PartialEq, Eq)]"));
    assert!(canonical.contains("pub const fn new"));
    assert!(canonical.contains("#[must_use]"));
    assert!(canonical.contains("# Errors"));
    assert!(canonical.contains("println!(\"{}\""));
    assert!(!canonical.contains("const fn main"));
    assert!(!canonical.contains("let sifr_generated_result: ()"));
    assert!(canonical.contains("let _ = sifr_generated_thing.fail();"));
}

#[test]
fn preserves_associated_methods_referenced_through_self_in_trait_implementations() {
    let source = r#"
        struct GeneratedError { message: String }
        impl GeneratedError {
            fn new(message: String) -> Self { Self { message } }
        }
        impl From<String> for GeneratedError {
            fn from(message: String) -> Self { Self::new(message) }
        }
        fn main() {
            let error: GeneratedError = "failure".to_string().into();
            println!("{}", error.message);
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("trait implementations should retain associated methods reached through Self");

    assert!(canonical.contains("fn new"), "{canonical}");
    assert!(canonical.contains("Self::new(message)"), "{canonical}");
}

#[test]
fn rejects_invalid_assembled_source() {
    let error = canonicalize_generated_rust_source("fn broken(")
        .expect_err("invalid assembled Rust must fail closed");

    assert!(error.starts_with("failed to parse assembled generated Rust:"));
}

#[test]
fn rewrites_format_captures_nested_inside_assertion_macros() {
    let source = r#"
        fn main() {
            let value = 1;
            assert_eq!(format!("{:?}", value), "1");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("nested generated format macro should canonicalize");

    assert!(canonical.contains("format!(\"{value:?}\")"), "{canonical}");
}

#[test]
fn folds_tail_option_if_let_into_one_match_expression() {
    let source = r#"
        struct __Task;
        impl __Task {
            async fn join(receiver: Option<i64>) -> i64 {
                if let Some(receiver) = receiver {
                    return receiver;
                }
                0
            }
        }
        async fn main() { let _ = __Task::join(None).await; }
    "#;

    let canonical =
        canonicalize_generated_rust_source(source).expect("tail option branch should canonicalize");

    assert!(
        canonical.contains("let Some(receiver) = receiver else"),
        "{canonical}"
    );
    assert!(!canonical.contains("if let Some"), "{canonical}");
}

#[test]
fn preserves_match_guards_when_single_pattern_simplification_is_not_equivalent() {
    let source = r#"
        fn main() {
            let value = 1;
            match value {
                candidate if candidate > 0 => println!("positive"),
                _ => println!("other"),
            }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("guarded generated match should remain valid Rust");

    assert!(
        canonical.contains("candidate if candidate > 0"),
        "{canonical}"
    );
    assert!(!canonical.contains("if let candidate if"), "{canonical}");
}

#[test]
fn does_not_promote_type_dependent_operator_dispatch_to_const() {
    let source = r#"
        fn double(value: SifrInt) -> SifrInt {
            &value * &SifrInt::from_i64(2)
        }
        fn main() { let _ = double(SifrInt::from_i64(1)); }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("operator-bearing generated function should remain valid Rust");

    assert!(canonical.contains("fn double"), "{canonical}");
    assert!(!canonical.contains("const fn double"), "{canonical}");
}

#[test]
fn does_not_promote_owned_field_extraction_with_drop_semantics_to_const() {
    let source = r#"
        struct WorkerRuntimeError { message: String }
        struct WorkerError { message: String }
        impl WorkerError {
            fn new(message: String) -> Self { Self { message } }
        }
        fn convert(error: WorkerRuntimeError) -> WorkerError {
            WorkerError::new(error.message)
        }
        fn main() {
            let _ = convert(WorkerRuntimeError { message: String::new() });
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("owned field extraction should remain valid generated Rust");

    assert!(canonical.contains("fn convert"), "{canonical}");
    assert!(!canonical.contains("const fn convert"), "{canonical}");
}

#[test]
fn preserves_trait_dependencies_complex_renames_mutability_and_pattern_liveness() {
    let source = r#"
        trait Describable { fn describe(&self) -> i64; }
        struct Item;
        impl Describable for Item { fn describe(&self) -> i64 { 1 } }
        struct Task { observed: i64, error: i64 }
        fn main() {
            let first_observed = 0;
            let first_receiver = Some(1);
            let second_receiver = Some(2);
            let (Some(mut first_receiver), Some(second_receiver)) =
                (first_receiver, second_receiver) else { return; };
            select! { value = &mut first_receiver => { let _ = value; } }
            let mut observer_count = 0;
            observer_count += 1;
            let Task { observed, error } = Task { observed: 1, error: 2 };
            println!("{} {} {} {}", Item.describe(), first_observed, second_receiver, observed);
            println!("{observer_count}");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("structured cleanup should preserve every declaration dependency");

    assert!(canonical.contains("trait Describable"), "{canonical}");
    assert!(
        canonical.contains("impl Describable for Item"),
        "{canonical}"
    );
    assert!(
        canonical.contains("mut first_receiver_value_"),
        "{canonical}"
    );
    assert!(
        canonical.matches("first_receiver_value_").count() >= 2,
        "{canonical}"
    );
    assert!(canonical.contains("let mut observer_count"), "{canonical}");
    assert!(canonical.contains("error: _"), "{canonical}");
}

#[test]
fn simplifies_wildcard_option_tests_and_empty_conditionals() {
    let source = r#"
        fn main() {
            let value = Some(1);
            if let Some(unused) = value { println!("present"); }
            let result: Result<(), ()> = Ok(());
            if result.is_err() {}
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("wildcard tests and empty conditionals should simplify structurally");

    assert!(canonical.contains("value.is_some()"), "{canonical}");
    assert!(!canonical.contains("if let Some(_)"), "{canonical}");
    assert!(!canonical.contains("if result.is_err()"), "{canonical}");
    assert!(
        canonical.contains("let _ = result.is_err();"),
        "{canonical}"
    );
}

#[test]
fn preserves_result_error_bindings_when_rewriting_identity_matches() {
    let source = r#"
        pub fn recover(result: Result<i64, String>) -> i64 {
            match result {
                Ok(value) => value,
                Err(error) => error.len() as i64,
            }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("result identity match should canonicalize without losing bindings");

    assert!(canonical.contains("unwrap_or_else"), "{canonical}");
    assert!(
        canonical.contains("|error| error.len() as i64"),
        "{canonical}"
    );
}

#[test]
fn preserves_result_matches_that_return_from_the_enclosing_function() {
    let source = r#"
        pub fn propagate(result: Result<i64, String>) -> Result<i64, String> {
            let value = match result {
                Ok(value) => value,
                Err(error) => return Err(error),
            };
            Ok(value)
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("enclosing control flow must not move into a closure");

    assert!(canonical.contains("match result"), "{canonical}");
    assert!(canonical.contains("return Err(error)"), "{canonical}");
    assert!(!canonical.contains("map_or_else"), "{canonical}");
    assert!(
        canonical.contains("clippy::single_match_else"),
        "{canonical}"
    );
}

#[test]
fn canonicalizes_discarded_result_match_without_capturing_enclosing_return() {
    let source = r#"
        pub async fn wait(result: Result<(), String>) -> Result<(), String> {
            match async { result }.await {
                Ok(value) => value,
                Err(error) => { return Err(error); }
            };
            Ok(())
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("discarded result matches should preserve enclosing control flow");

    assert!(
        canonical.contains("async { result }.await?;"),
        "{canonical}"
    );
    assert!(!canonical.contains("return Err(error)"), "{canonical}");
    assert!(!canonical.contains("unwrap_or_else"), "{canonical}");
    assert!(
        !canonical.contains("clippy::single_match_else"),
        "{canonical}"
    );
}

#[test]
fn canonicalizes_discarded_wildcard_result_match_as_an_error_test() {
    let source = r#"
        pub async fn wait(result: Result<(), String>) -> Result<(), String> {
            match async { result }.await {
                Ok(value) => value,
                Err(_) => { return Err("failed".to_string()); }
            };
            Ok(())
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("wildcard result matches should become direct error tests");

    assert!(canonical.contains(".await.is_err()"), "{canonical}");
    assert!(!canonical.contains("if let Err(_)"), "{canonical}");
    assert!(canonical.contains("return Err"), "{canonical}");
}

#[test]
fn preserves_assignment_updates_that_read_the_previous_value() {
    let source = r#"
        pub fn pad(mut text: String, needs_padding: bool) -> String {
            let mut padded = text;
            if needs_padding {
                padded = "0".to_string() + padded.as_str();
            }
            padded
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("self-transforming assignment must remain sequenced");

    assert!(canonical.contains("let mut padded = text"), "{canonical}");
    assert!(canonical.contains("padded.as_str()"), "{canonical}");
}

#[test]
fn canonicalizes_expressions_nested_in_formatting_macros() {
    let source = r#"
        pub fn show(value: Option<i64>) {
            println!("{}", value.map_or("None".to_string().to_string(), |item| item.to_string()));
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("formatting macro arguments should use canonical expressions");

    assert!(
        !canonical.contains("to_string().to_string()"),
        "{canonical}"
    );
    assert!(canonical.contains("map_or_else"), "{canonical}");
    assert!(canonical.contains("item.to_string()"), "{canonical}");
    assert!(!canonical.contains("ToString::to_string"), "{canonical}");
}

#[test]
fn preserves_bindings_moved_into_format_captures() {
    let source = r#"
        pub fn show() {
            let score_result: Result<i64, String> = Ok(21);
            assert_eq!(format!("{:?}", score_result), "Ok(21)");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("format capture inlining must preserve binding liveness");

    assert!(canonical.contains("let score_result"), "{canonical}");
    assert!(canonical.contains("{score_result:?}"), "{canonical}");
}

#[test]
fn canonicalizes_negative_if_else_and_option_let_else() {
    let source = r#"
        pub fn signal(value: i64, item: Option<String>) -> Option<String> {
            let Some(item) = item.as_ref() else { return None; };
            if value != 0 { Some(item.to_owned()) } else { None }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("canonical condition and question-mark rewrites should apply");

    assert!(
        canonical.contains("let item = item.as_ref()?"),
        "{canonical}"
    );
    assert!(canonical.contains("if value == 0"), "{canonical}");
}

#[test]
fn canonicalizes_option_if_let_expressions_and_vector_push_initialization() {
    let source = r#"
        pub fn inspect(value: Option<String>) -> (bool, Vec<Option<String>>) {
            let present = if let Some(item) = value {
                !item.is_empty()
            } else {
                false
            };
            let mut values: Vec<Option<String>> = vec![];
            values.push(Some("first".to_string()));
            values.push(None);
            (present, values)
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("generated option branches and vector initialization should canonicalize");

    assert!(canonical.contains(".is_some_and("), "{canonical}");
    assert!(
        canonical
            .contains("let values: Vec<Option<String>> = vec![Some(\"first\".to_string()), None]"),
        "{canonical}"
    );
    assert!(!canonical.contains("values.push"), "{canonical}");
}

#[test]
fn preserves_fallible_control_when_an_option_branch_cannot_become_a_bool_closure() {
    let source = r#"
        fn parse(value: &str) -> Result<i64, String> { Ok(value.len() as i64) }
        fn check(value: Option<String>) -> Result<bool, String> {
            let valid = if let Some(value) = value {
                let _parsed = parse(&value)?;
                true
            } else {
                false
            };
            Ok(valid)
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("fallible control must remain in the enclosing Result function");

    assert!(
        canonical.contains("if let Some(value) = value"),
        "{canonical}"
    );
    assert!(canonical.contains("parse(&value)?"), "{canonical}");
    assert!(!canonical.contains("is_some_and"), "{canonical}");
}

#[test]
fn preserves_mutability_for_method_calls_nested_in_assertion_macros() {
    let source = r#"
        pub fn check() {
            let mut values = vec![1_i64].into_iter();
            assert_eq!(values.next(), Some(1_i64));
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("macro arguments should participate in mutating-use analysis");

    assert!(canonical.contains("let mut values"), "{canonical}");
}

#[test]
fn preserves_statement_semicolons_when_folding_literal_result_bindings() {
    let source = r#"
        pub fn consume() {
            let generated: Result<String, String> = Ok("ready".to_string());
            generated.unwrap_or_else(|_| "fallback".to_string());
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("literal result scaffolding should fold without changing block type");

    assert!(!canonical.contains("generated"), "{canonical}");
    assert!(canonical.contains("\"ready\".to_string();"), "{canonical}");
}

#[test]
fn removes_infallible_result_scaffolding_and_folds_initial_assignment() {
    let source = r#"
        pub fn collect() -> Vec<String> {
            let mut values: Vec<String> = Vec::new();
            let generated_result: Result<(), String> = {
                values = vec!["ready".to_string()];
                Ok(())
            };
            if generated_result.is_err() {
                assert!(false);
            }
            values
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("infallible generated result scaffolding should disappear");

    assert!(!canonical.contains("generated_result"), "{canonical}");
    assert!(!canonical.contains("assert!(false)"), "{canonical}");
    assert!(
        canonical.contains("let values: Vec<String> = vec![\"ready\".to_string()]"),
        "{canonical}"
    );
}

#[test]
fn moves_default_declarations_to_their_first_straight_line_assignment() {
    let source = r#"
        fn side_effect() {}
        pub fn check() -> bool {
            let mut ready: bool = false;
            side_effect();
            ready = true;
            ready
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("dead default initialization should not survive before a guaranteed assignment");

    assert!(!canonical.contains("ready: bool = false"), "{canonical}");
    assert!(canonical.contains("let ready: bool = true"), "{canonical}");
    assert!(
        canonical.find("side_effect();") < canonical.find("let ready: bool = true"),
        "{canonical}"
    );
}

#[test]
fn removes_generated_character_cache_rebuilds_with_no_live_successor() {
    let source = r#"
        pub fn trim(mut base: String) -> String {
            let mut sifr_generated_chars_base = base.chars().collect::<Vec<char>>();
            if !sifr_generated_chars_base.is_empty() {
                base = base.trim().to_string();
                sifr_generated_chars_base = base.chars().collect::<Vec<char>>();
            }
            base
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("a generated character cache should only rebuild before a later cache read");

    assert_eq!(
        canonical.matches("sifr_generated_chars_base =").count(),
        1,
        "{canonical}"
    );
    assert!(
        !canonical.contains("let mut sifr_generated_chars_base"),
        "{canonical}"
    );
}

#[test]
fn ignores_references_to_nested_shadow_bindings_when_cleaning_let_else() {
    let source = r#"
        pub fn check(first: Option<i64>, second: Option<i64>) {
            let Some(sifr_generated_checked_value) = first else { return; };
            if true {
                let Some(sifr_generated_checked_value) = second else { return; };
                println!("{sifr_generated_checked_value}");
            }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("nested shadow bindings must not keep an unused outer proof binding alive");

    assert!(canonical.contains("if first.is_none()"), "{canonical}");
    assert_eq!(canonical.matches("let Some(").count(), 1, "{canonical}");
}

#[test]
fn wildcards_unused_generated_parameters_and_expects_constant_assertions_with_messages() {
    let source = r#"
        pub fn generated(sifr_generated_context: &mut i64) {
            assert!(false, "typed source failure");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("unused support parameters and deliberate source assertions should be explicit");

    assert!(
        canonical.contains("pub fn generated(_: &mut i64)"),
        "{canonical}"
    );
    assert!(
        canonical.contains("clippy::assertions_on_constants"),
        "{canonical}"
    );
}

#[test]
fn wildcards_unused_source_parameters_in_generated_function_bodies() {
    let source = r#"
        pub fn generated(value: i64) -> i64 {
            7
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("unused source parameters should not create Rust warnings");

    assert!(
        canonical.contains("pub const fn generated(_: i64) -> i64"),
        "{canonical}"
    );
}

#[test]
fn emits_legal_unparenthesized_let_chains_when_collapsing_nested_conditions() {
    let source = r#"
        pub fn smaller(left: Option<i64>, right: Option<i64>) -> bool {
            if let Some(left) = left {
                if let Some(right) = right {
                    return right < left;
                }
            }
            false
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("nested option conditions should canonicalize as a legal let chain");

    assert!(
        canonical.contains("if let Some(left) = left && let Some(right) = right"),
        "{canonical}"
    );
    assert!(!canonical.contains("&& (let"), "{canonical}");
}

#[test]
fn preserves_references_to_pattern_bindings_that_shadow_renamed_locals() {
    let source = r#"
        fn main() {
            let tab_lines = vec!["hello".to_string()];
            let tab_line0 = tab_lines.first().cloned();
            if let Some(tab_line0) = tab_line0 {
                assert_eq!(tab_line0.chars().count(), 5);
            }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("local-name cleanup must respect nested lexical shadowing");

    assert!(canonical.contains("let tab_line0_value_"), "{canonical}");
    assert!(
        canonical.contains("if let Some(tab_line0) = tab_line0_value_"),
        "{canonical}"
    );
    assert!(canonical.contains("tab_line0.chars()"), "{canonical}");
}

#[test]
fn removes_an_unused_result_contract_from_infallible_main() {
    let source = r#"
        struct Error;
        fn main() -> Result<(), Error> {
            println!("done");
            Ok(())
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("infallible main should use Rust's unit entrypoint contract");

    assert!(canonical.contains("fn main()"), "{canonical}");
    assert!(!canonical.contains("Result<(), Error>"), "{canonical}");
    assert!(!canonical.contains("struct Error"), "{canonical}");
}

#[test]
fn documents_source_signatures_over_clippys_argument_threshold() {
    let source = r#"
        pub fn combine(a: i64, b: i64, c: i64, d: i64, e: i64, f: i64, g: i64, h: i64) -> i64 {
            a + b + c + d + e + f + g + h
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("source-shaped generated APIs should carry an exact lint expectation");

    assert!(
        canonical.contains("clippy::too_many_arguments"),
        "{canonical}"
    );
    assert!(
        canonical.contains("preserves the typed Sifr callable contract"),
        "{canonical}"
    );
}

#[test]
fn documents_only_function_bodies_over_clippys_line_threshold() {
    fn source_with_body_lines(lines: usize) -> String {
        let statements = "consume();\n".repeat(lines);
        format!("fn generated() {{\n{statements}}}\n")
    }

    let at_limit = canonicalize_generated_rust_source(&source_with_body_lines(100))
        .expect("a function at Clippy's body limit should canonicalize");
    let over_limit = canonicalize_generated_rust_source(&source_with_body_lines(101))
        .expect("a function over Clippy's body limit should canonicalize");

    assert!(!at_limit.contains("clippy::too_many_lines"), "{at_limit}");
    assert!(
        over_limit.contains("clippy::too_many_lines"),
        "{over_limit}"
    );

    let stale = format!(
        "#[expect(clippy::too_many_lines, reason = \"one generated Rust function preserves one typed Sifr function\")]\n{}",
        source_with_body_lines(100)
    );
    let refreshed = canonicalize_generated_rust_source(&stale)
        .expect("compiler-owned expectations should be recomputed on every pass");
    assert!(!refreshed.contains("clippy::too_many_lines"), "{refreshed}");
}

#[test]
fn measures_function_line_budget_after_structured_rendering() {
    let branches = "if condition() {\n    consume();\n}\n".repeat(34);
    let source = format!(
        "fn generated() {{\n{branches}}}\nfn condition() -> bool {{ true }}\nfn consume() {{}}\n"
    );

    let canonical = finalize_formatted_generated_rust_source(&source)
        .expect("line-budget expectations must follow the formatted Rust body");

    assert!(canonical.contains("clippy::too_many_lines"), "{canonical}");
}
