use super::{
    canonicalize_generated_rust_source, discover_project_const_function_names,
    finalize_formatted_generated_rust_source_with_project_consts,
};

#[test]
fn preserves_result_default_closures_that_use_the_error_parameter() {
    let source = r#"
        fn message(result: Result<String, String>) -> String {
            result.unwrap_or_else(|error| error)
        }
        fn main() { println!("{}", message(Err("failed".to_string()))); }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("a Result default that uses its error must remain lazy and scoped");

    assert!(
        canonical.contains("unwrap_or_else(|error| error)"),
        "{canonical}"
    );
    syn::parse_file(&canonical).expect("the preserved closure must remain valid Rust");
}

#[test]
fn unused_owned_bindings_keep_their_scope_drop_point() {
    let source = r#"
        struct Guard;
        impl Drop for Guard {
            fn drop(&mut self) { println!("dropped"); }
        }
        fn main() {
            let guard = Guard;
            println!("body");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("unused owned values must stay bound until the enclosing scope exits");

    assert!(canonical.contains("let _guard = Guard;"), "{canonical}");
    assert!(!canonical.contains("let _ = Guard;"), "{canonical}");
}

#[test]
fn unused_fallible_arithmetic_is_still_evaluated() {
    let source = r#"
        fn divide(left: i64, right: i64) {
            let quotient = left / right;
            println!("evaluated");
        }
        fn main() { divide(1, 1); }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("discarded arithmetic must preserve evaluation and failure timing");

    assert!(
        canonical.contains("let _quotient = left / right;"),
        "{canonical}"
    );
}

#[test]
fn unused_clones_are_not_assumed_to_be_effect_free() {
    let source = r#"
        #[derive(Clone)]
        struct Value;
        fn duplicate(value: Value) {
            let copy = value.clone();
            println!("cloned");
        }
        fn main() { duplicate(Value); }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("Clone implementations are not a syntactic purity boundary");

    assert!(
        canonical.contains("let _copy = value.clone();"),
        "{canonical}"
    );
}

#[test]
fn option_none_patterns_use_the_canonical_option_query() {
    let source = r#"
        fn describe(value: Option<i64>) -> &'static str {
            if let None = value { "none" } else { "some" }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("the prelude None variant must not be renamed as an unused binding");

    assert!(canonical.contains("value.is_none()"), "{canonical}");
    assert!(!canonical.contains("_None"), "{canonical}");
}

#[test]
fn source_expectations_cover_nested_dense_names_and_constant_assertions_only() {
    let source = r#"
        const PI: f64 = 3.14159;
        fn constants() { assert!(true); assert!(PI > 3.14); }
        fn dynamic(condition: bool) { assert!(condition); }
        fn dense() {
            if true {
                let a = 1; let b = 2; let c = 3; let d = 4; let e = 5;
                println!("{}", a + b + c + d + e);
            }
        }
        fn disjoint() {
            { let a = 1; let b = 2; println!("{}", a + b); }
            { let c = 3; let d = 4; let e = 5; println!("{}", c + d + e); }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("exact source-contract lint shapes must be recognized through nested blocks");
    let parsed = syn::parse_file(&canonical).expect("canonical output must parse");
    let attrs = |name: &str| {
        parsed
            .items
            .iter()
            .find_map(|item| match item {
                syn::Item::Fn(function) if function.sig.ident == name => Some(
                    function
                        .attrs
                        .iter()
                        .map(quote::ToTokens::to_token_stream)
                        .map(|tokens| tokens.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                ),
                _ => None,
            })
            .expect("test function must remain")
    };
    assert!(
        attrs("constants").contains("assertions_on_constants"),
        "{canonical}"
    );
    assert!(
        !attrs("dynamic").contains("assertions_on_constants"),
        "{canonical}"
    );
    assert!(
        attrs("dense").contains("many_single_char_names"),
        "{canonical}"
    );
    assert!(
        !attrs("disjoint").contains("many_single_char_names"),
        "{canonical}"
    );
}

#[test]
fn folds_tail_bindings_and_empty_else_blocks() {
    let source = r#"
        fn selected(value: Option<i64>) -> bool {
            if let None = value { false } else { true }
        }
        fn describe(value: &str) -> String {
            if value.is_empty() { println!("empty"); } else {}
            let result = value.to_string();
            result
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("canonical control flow should not retain trivial pattern or tail scaffolding");

    assert!(canonical.contains("value.is_none()"), "{canonical}");
    assert!(!canonical.contains("if let None"), "{canonical}");
    assert!(!canonical.contains("else {}"), "{canonical}");
    assert!(!canonical.contains("let result ="), "{canonical}");
}

#[test]
fn rewrites_only_identity_constructors_with_structural_proof() {
    let source = r#"
        fn convert(value: Option<i64>, values: Vec<i64>) {
            let _ = value.map_or_else(|| Err::<i64, ()>(()), |item| Ok(item));
            assert!(value != None);
            assert!(None == value);
            let _ = values.into_iter().map(|item| item + 1).into_iter().collect::<Vec<_>>();
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("identity adapters and Option comparisons should use canonical Rust APIs");

    assert!(
        canonical.contains("map_or_else(|| Err::<i64, ()>(()), Ok)"),
        "{canonical}"
    );
    assert!(canonical.contains("assert_ne!(value, None)"), "{canonical}");
    assert!(canonical.contains("assert_eq!(None, value)"), "{canonical}");
    assert!(!canonical.contains(".is_none()"), "{canonical}");
    assert!(!canonical.contains(".is_some()"), "{canonical}");
    assert_eq!(canonical.matches(".into_iter()").count(), 2, "{canonical}");
}

#[test]
fn factors_shared_branch_prefix_after_condition_evaluation() {
    let source = r#"
        fn report(result: Result<String, String>) {
            if result.is_ok() {
                let value = result.clone();
                println!("ok {value:?}");
            } else {
                let value = result.clone();
                println!("error {value:?}");
            }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("a common branch prefix should have one evaluation after the condition");

    assert_eq!(
        canonical.matches("let value = result.clone()").count(),
        1,
        "{canonical}"
    );
    assert!(
        canonical.contains("sifr_generated_shared_branch_condition"),
        "{canonical}"
    );
}

#[test]
fn factors_shared_if_let_prefix_without_invalid_let_expression() {
    let source = r#"
        fn report(value: Option<String>) {
            if let Some(value) = value.clone() {
                println!("result:");
                println!("{value}");
            } else {
                println!("result:");
                println!("missing");
            }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("an if-let prefix should preserve the pattern condition and its binding");

    assert_eq!(
        canonical.matches("println!(\"result:\")").count(),
        1,
        "{canonical}"
    );
    assert!(
        canonical.contains("let sifr_generated_shared_branch_value = value.clone()"),
        "{canonical}"
    );
    assert!(
        canonical.contains("if let Some(value) = sifr_generated_shared_branch_value"),
        "{canonical}"
    );
}

#[test]
fn preserves_if_let_prefix_that_reads_a_shadowing_pattern_binding() {
    let source = r#"
        fn report(value: Option<String>) {
            let item = "outer";
            if let Some(item) = value {
                println!("{item}");
                println!("present");
            } else {
                println!("{item}");
                println!("missing");
            }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("branch factoring must preserve if-let pattern binding scope");

    assert_eq!(
        canonical.matches("println!(\"{item}\")").count(),
        2,
        "{canonical}"
    );
    assert!(
        !canonical.contains("sifr_generated_shared_branch_value"),
        "{canonical}"
    );
}

#[test]
fn retains_shared_branch_suffix_before_branch_local_drop() {
    let source = r#"
        fn advance(flag: bool, mut left: i64, mut right: i64) {
            if flag {
                println!("short");
                left += 1;
                right += 1;
            } else {
                let detail = "long";
                println!("{detail}");
                left += 1;
                right += 1;
            }
            println!("{left} {right}");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("common branch suffixes must not cross a branch-local drop boundary");

    assert_eq!(canonical.matches("left += 1").count(), 2, "{canonical}");
    assert_eq!(canonical.matches("right += 1").count(), 2, "{canonical}");
    assert!(canonical.contains("let detail = \"long\""), "{canonical}");
}

#[test]
fn preserves_if_let_suffix_that_reads_a_shadowing_pattern_binding() {
    let source = r#"
        fn report(value: Option<String>) {
            let item = "outer";
            if let Some(item) = value {
                println!("present");
                println!("{item}");
            } else {
                println!("missing");
                println!("{item}");
            }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("suffix factoring must preserve if-let pattern binding scope");

    assert_eq!(
        canonical.matches("println!(\"{item}\")").count(),
        2,
        "{canonical}"
    );
}

#[test]
fn folds_generated_conditional_initialization_without_losing_branch_effects() {
    let source = r#"
        fn choose(flag: bool, nested: bool, input: &str) -> (String, Vec<i64>, i64) {
            let mut text = {
                let mut sifr_generated_concat = String::with_capacity(input.len());
                sifr_generated_concat.push_str(input);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
            if flag { text = "yes".to_string(); } else { text = "no".to_string(); }
            let mut side = 0;
            let mut values = Vec::new();
            if flag { values = vec![1]; side = 1; } else { values = vec![2]; side = 2; }
            let mut origin = 0;
            if flag { origin = 1; } else if nested { origin = 2; } else { return (text, values, side); }
            (text, values, origin + side)
        }
        fn wrapped(value: String) -> String {
            let mut selected = String::new();
            { selected = value; }
            selected
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("generated replace-then-branch scaffolding should become conditional initializers");

    assert!(canonical.contains("let text = if flag"), "{canonical}");
    assert!(canonical.contains("let values = if flag"), "{canonical}");
    assert!(canonical.contains("let origin = if flag"), "{canonical}");
    assert!(
        canonical.contains("const fn wrapped(value: String) -> String {\n    value\n}"),
        "{canonical}"
    );
    assert!(!canonical.contains("selected = value"), "{canonical}");
}

#[test]
fn inlines_assert_format_arguments_and_terminates_unit_macro_tails() {
    let source = r#"
        fn close(left: f64, right: f64, tolerance: f64) {
            {
                assert!(left == right, "failed: {} != {} ({})", left, right, tolerance)
            };
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("assert diagnostics should use format capture and a terminated unit tail");

    assert!(
        canonical.contains("\"failed: {left} != {right} ({tolerance})\""),
        "{canonical}"
    );
    assert!(
        canonical.contains("assert!(left == right, \"failed: {left} != {right} ({tolerance})\");"),
        "{canonical}"
    );
}

#[test]
fn preserves_bindings_used_only_by_inlined_assert_format_captures() {
    let source = r#"
        fn fail() {
            let error = "failure".to_string();
            assert!(false, "{}", error);
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("assert format capture liveness must survive subsequent canonical passes");

    assert!(canonical.contains("let error ="), "{canonical}");
    assert!(
        canonical.contains("assert!(false, \"{error}\")"),
        "{canonical}"
    );
    assert!(!canonical.contains("let _error"), "{canonical}");
}

#[test]
fn prunes_dead_items_when_only_generated_external_modules_are_present() {
    let source = r#"
        pub mod sifr_generated_bridge;
        const LIVE: i64 = 1;
        struct Dead;
        impl std::fmt::Display for Dead {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("dead")
            }
        }
        fn main() { println!("{LIVE}"); }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("compiler-owned external modules must not disable closed-binary demand pruning");

    assert!(
        canonical.contains("mod sifr_generated_bridge"),
        "{canonical}"
    );
    assert!(!canonical.contains("struct Dead"), "{canonical}");
    assert!(canonical.contains("const LIVE"), "{canonical}");
}

#[test]
fn removes_mutability_when_local_method_facts_prove_a_shared_receiver() {
    let source = r#"
        struct TextHandle;
        struct BinaryHandle;
        struct TextContext;
        struct BinaryContext;
        impl TextHandle { fn write(&self, _text: &str) {} }
        impl TextContext { fn enter(&self) -> TextHandle { TextHandle } }
        impl BinaryContext { fn enter(&self) -> BinaryHandle { BinaryHandle } }
        fn use_context(context: &TextContext) {
            let mut handle = context.enter();
            handle.write("value");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("local receiver and return facts should override name-only mutability guesses");

    assert!(
        canonical.contains("let handle = context.enter();"),
        "{canonical}"
    );
    assert!(!canonical.contains("let mut handle"), "{canonical}");
}

#[test]
fn removes_only_proven_residual_unit_iterator_and_static_format_ceremony() {
    let source = r#"
        struct Generated;
        impl Generated {
            fn sifr_generated_iter__(&self) -> Box<dyn Iterator<Item = i64>> {
                Box::new(vec![1].into_iter())
            }
        }
        fn main() {
            let mut values = vec![0];
            { values.extend(vec![1, 2].into_iter()); () };
            let paired = vec![1].into_iter().zip(vec![2].into_iter());
            let boxed = Box::new(Generated.sifr_generated_iter__().into_iter());
            println!("{} {:?} {:?}", format!("ready"), paired.collect::<Vec<_>>(), boxed.collect::<Vec<_>>());
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("residual unit, iterator, and static-format ceremony should canonicalize");

    assert!(
        canonical.contains(".extend(vec![1, 2].into_iter())"),
        "{canonical}"
    );
    assert!(
        canonical.contains(".zip(vec![2].into_iter())"),
        "{canonical}"
    );
    assert!(
        !canonical.contains("sifr_generated_iter__().into_iter()"),
        "{canonical}"
    );
    assert!(!canonical.contains("format!(\"ready\")"), "{canonical}");
    assert!(!canonical.contains("\n        ()\n"), "{canonical}");
}

#[test]
fn leaves_post_parse_len_closures_unchanged_and_rewrites_nested_format_arguments() {
    let source = r#"
        struct Vec2 { x: i64, y: i64 }
        impl std::fmt::Display for Vec2 {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", format!("Vec2({}, {})", self.x, self.y))
            }
        }
        fn safe_len(items: &Option<Vec<String>>) -> usize {
            items.as_ref().map_or(0, |value| value.len())
        }
        fn main() {
            println!("{}", Vec2 { x: 1, y: 2 });
            println!("{}", safe_len(&Some(vec!["x".to_string()])));
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("typed closures and nested formatting should canonicalize");

    assert!(canonical.contains("|value| value.len()"), "{canonical}");
    assert!(
        !canonical.contains("write!(f, \"{}\", format!"),
        "{canonical}"
    );
    assert!(
        canonical.contains("write!(f, \"Vec2({}, {})\""),
        "{canonical}"
    );
}

#[test]
fn canonicalizes_source_enum_variants_without_changing_their_names() {
    let source = r#"
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(i64)]
        enum Direction { NORTH = 1, SOUTH = 2, NOT_FOUND = 3 }
        impl std::fmt::Display for Direction {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{self:?}")
            }
        }
        impl Direction {
            fn name(&self) -> String { format!("{self:?}") }
            fn value(&self) -> i64 { *self as i64 }
            fn is_vertical(&self) -> bool {
                match self { Direction::NORTH | Direction::SOUTH => true, Direction::NOT_FOUND => false }
            }
        }
        fn main() {
            let direction: Direction = Direction::NORTH;
            println!("{} {} {}", direction.name(), direction.value(), direction.is_vertical());
            println!("{}", Direction::NOT_FOUND);
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("source enum names should have idiomatic Rust identifiers and stable values");

    assert!(canonical.contains("North = 1"), "{canonical}");
    assert!(canonical.contains("NotFound = 3"), "{canonical}");
    assert!(!canonical.contains("SOUTH"), "{canonical}");
    assert!(canonical.contains("impl ::std::fmt::Debug"), "{canonical}");
    assert!(canonical.contains(".write_str("), "{canonical}");
    assert!(canonical.contains("\"NORTH\""), "{canonical}");
    assert!(canonical.contains("const fn value"), "{canonical}");
}

#[test]
fn retains_unread_drop_fields_and_prunes_same_named_undemanded_methods() {
    let source = r#"
        struct Config { value: i64, callback: Box<dyn Fn(i64) -> i64> }
        impl Config {
            fn new(value: i64, callback: impl Fn(i64) -> i64 + 'static) -> Self {
                let stored = Box::new(callback);
                Self { value, callback: stored }
            }
        }
        struct Base;
        impl Base { fn describe(&self) -> &'static str { "base" } }
        struct Used;
        impl Used { fn describe(&self) -> &'static str { "used" } }
        fn main() {
            let config = Config::new(1, |value| value);
            let base = Base;
            let used: Used = Used;
            println!("{} {}", config.value, used.describe());
            let _ = base;
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("closed binaries should retain only demanded private members");

    assert!(canonical.contains("callback: Box"), "{canonical}");
    assert!(
        !canonical.contains("impl Base {\n    fn describe"),
        "{canonical}"
    );
    assert!(canonical.contains("impl Used"), "{canonical}");
    assert!(canonical.contains("fn describe"), "{canonical}");
}

#[test]
fn folds_overwritten_sifr_integer_initializers() {
    let source = r#"
        use sifr_runtime::SifrInt;
        fn main() {
            let mut p: SifrInt = SifrInt::from_i64(0);
            let mut q: SifrInt = SifrInt::from_i64(0);
            let mut r: SifrInt = SifrInt::from_i64(0);
            r = SifrInt::from_i64(99);
            q = r.clone();
            p = q.clone();
            println!("{p} {q} {r}");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("pure overwritten initializers should fold into their first live assignments");

    assert!(!canonical.contains("from_i64(0)"), "{canonical}");
    assert!(canonical.contains("let r"), "{canonical}");
    assert!(canonical.contains("let q"), "{canonical}");
    assert!(canonical.contains("let p"), "{canonical}");
}

#[test]
fn propagates_const_apis_across_generated_project_files() {
    let provider = "pub const fn provided() -> i64 { 1 }";
    let consumer = "pub use crate::provider::provided; pub fn value() -> i64 { provided() }";
    let initial = discover_project_const_function_names([provider, consumer])
        .expect("project const discovery should parse every generated module");
    let consumer = finalize_formatted_generated_rust_source_with_project_consts(consumer, &initial)
        .expect("an imported const function should make its wrapper const");
    assert!(consumer.contains("pub const fn value"), "{consumer}");

    let expanded = discover_project_const_function_names([provider, consumer.as_str()])
        .expect("new const wrappers should join the project-wide fixed point");
    assert!(expanded.contains("provided"));
    assert!(expanded.contains("value"));
}

#[test]
fn suppresses_unused_bindings_across_let_chain_conditions() {
    let source = r#"
        fn main() {
            let first_value = Some(1_i64);
            let second_value = Some(2_i64);
            if let Some(first_value) = first_value
                && let Some(second_value) = second_value
            {
                println!("present");
            }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("unused let-chain payloads should not remain warning-producing bindings");

    assert!(canonical.contains("Some(_first_value)"), "{canonical}");
    assert!(canonical.contains("Some(_second_value)"), "{canonical}");
}

#[test]
fn cascades_pruned_enum_generics_through_dependent_structs_and_impls() {
    let source = r#"
        struct BlockingTask<T, E> {
            handle: TaskResult<T, E>,
        }
        enum TaskResult<T, E> {
            Ok(T),
            Error(E),
        }
        impl<T, E> BlockingTask<T, E> {
            fn join(self) -> TaskResult<T, E> {
                self.handle
            }
        }
        fn spawn<T>(value: T) -> BlockingTask<T, ::std::convert::Infallible> {
            BlockingTask { handle: TaskResult::Ok(value) }
        }
        fn main() {
            let task: BlockingTask<i64, ::std::convert::Infallible> = spawn(1);
            if let TaskResult::Ok(value) = task.join() {
                println!("{value}");
            }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("aggregate generic pruning must cascade through users");

    assert!(canonical.contains("struct BlockingTask<T>"), "{canonical}");
    assert!(canonical.contains("enum TaskResult<T>"), "{canonical}");
    assert!(canonical.contains("impl<T> BlockingTask<T>"), "{canonical}");
    assert!(!canonical.contains("BlockingTask<T, E>"), "{canonical}");
    assert!(!canonical.contains("TaskResult<T, E>"), "{canonical}");
    assert!(!canonical.contains("Infallible"), "{canonical}");
}

#[test]
fn preserves_diverging_result_fallback_closure_coercion() {
    let source = r#"
        fn recover<T>(result: Result<T, Box<dyn ::std::any::Any + Send>>) -> T {
            result.unwrap_or_else(|payload| ::std::panic::resume_unwind(payload))
        }
        fn main() {
            println!("{}", recover::<i64>(Ok(1)));
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("a diverging closure body must retain its contextual return coercion");

    assert!(
        canonical.contains("unwrap_or_else(|payload| ::std::panic::resume_unwind(payload))"),
        "{canonical}"
    );
}

#[test]
fn rewrites_exhaustive_option_match_as_if_let() {
    let source = r#"
        fn inspect(value: Option<i64>) -> i64 {
            let output = match value {
                Some(inner) => {
                    println!("{inner}");
                    inner
                }
                None => {
                    println!("missing");
                    0
                }
            };
            println!("{output}");
            output
        }
        fn main() {
            println!("{}", inspect(Some(1)));
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("an exhaustive Option destructure should use canonical if-let control flow");

    assert!(
        canonical.contains("if let Some(inner) = value")
            || canonical.contains("let Some(inner) = value else")
            || canonical.contains(".map_or_else("),
        "{canonical}"
    );
    assert!(!canonical.contains("match value"), "{canonical}");
}

#[test]
fn option_match_with_await_stays_in_the_async_control_region() {
    let source = r#"
        async fn inspect(value: Option<i64>) -> i64 {
            let output = match value {
                Some(inner) => async move { inner }.await,
                None => async move { 0 }.await,
            };
            println!("{output}");
            output
        }
        fn main() {
            let _future = inspect(Some(1));
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("await must not move into a synchronous Option fallback closure");

    assert!(
        canonical.contains("if let Some(inner) = value"),
        "{canonical}"
    );
    assert!(!canonical.contains("match value"), "{canonical}");
    assert!(!canonical.contains("map_or_else"), "{canonical}");
}
