use super::canonicalize_generated_rust_source;

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
        !canonical.contains("clippy::single_match_else"),
        "{canonical}"
    );
}

#[test]
fn canonicalizes_wildcard_result_fallbacks_without_capturing_enclosing_returns() {
    let source = r#"
        pub fn recover(result: Result<i64, String>, early: bool) -> i64 {
            match result {
                Ok(value) => value,
                Err(_) => {
                    if early {
                        return 1;
                    }
                    2
                }
            }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("wildcard fallback control flow should remain in the enclosing function");

    assert!(
        canonical.contains("if let Ok(value) = result"),
        "{canonical}"
    );
    assert!(canonical.contains("return 1"), "{canonical}");
    assert!(!canonical.contains("match result"), "{canonical}");
    assert!(!canonical.contains("unwrap_or_else"), "{canonical}");
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
fn prunes_undemanded_generated_trait_methods_and_enum_variants() {
    let source = r#"
        trait GeneratedActions {
            fn used(&self) -> i64;
            fn dead(&self) -> i64;
        }
        struct GeneratedValue;
        impl GeneratedActions for GeneratedValue {
            fn used(&self) -> i64 { 1 }
            fn dead(&self) -> i64 { 2 }
        }
        enum GeneratedState {
            Ready(i64),
            Dead(String),
        }
        #[derive(Debug)]
        enum MacroState {
            Printed,
            Dead,
        }
        fn main() {
            let value = GeneratedValue;
            let state = GeneratedState::Ready(value.used());
            assert!(matches!(state, GeneratedState::Ready(_)));
            if let GeneratedState::Ready(result) = state {
                println!("{result}");
            }
            println!("{:?}", MacroState::Printed);
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("closed generated binaries should retain only demanded members");

    assert!(canonical.contains("fn used"), "{canonical}");
    assert!(!canonical.contains("fn dead"), "{canonical}");
    assert!(canonical.contains("Ready(i64)"), "{canonical}");
    assert!(!canonical.contains("Dead(String)"), "{canonical}");
    assert!(canonical.contains("Printed"), "{canonical}");
}

#[test]
fn preserves_method_demand_when_pattern_refinement_hides_the_receiver_owner() {
    let source = r#"
        struct GeneratedValue;
        impl GeneratedValue {
            fn live(&self) -> i64 { 1 }
            fn dead(&self) -> i64 { 2 }
        }
        fn make_value() -> Option<GeneratedValue> { Some(GeneratedValue) }
        fn main() {
            let mut value: Option<GeneratedValue> = None;
            value = make_value();
            if let Some(value) = value {
                println!("{}", value.live());
            }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("unresolved refined receivers must conservatively retain live methods");

    assert!(canonical.contains("fn live"), "{canonical}");
    assert!(!canonical.contains("fn dead"), "{canonical}");
    syn::parse_file(&canonical).expect("method-demand pruning must preserve valid Rust syntax");
}

#[test]
fn external_associated_constructors_do_not_retain_same_named_local_methods() {
    let source = r#"
        struct GeneratedFailure;
        impl GeneratedFailure {
            fn new() -> Self { Self }
            fn with_secondary() -> Self { Self }
        }
        fn main() {
            let _external = String::new();
            let _failure = GeneratedFailure::with_secondary();
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("exact external constructors must not retain unrelated local methods");

    assert!(!canonical.contains("fn new"), "{canonical}");
    assert!(canonical.contains("fn with_secondary"), "{canonical}");
}

#[test]
fn preserves_crate_root_items_imported_by_inline_support_modules() {
    let source = r#"
        struct GeneratedHandle(i64);
        impl GeneratedHandle {
            fn new(value: i64) -> Self { Self(value) }
        }
        struct DeadRootItem;
        mod support {
            use crate::GeneratedHandle;
            pub fn make_handle() -> GeneratedHandle { GeneratedHandle::new(1) }
        }
        fn main() {
            let handle = support::make_handle();
            println!("{}", handle.0);
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("inline module imports must demand their crate-root definitions");

    assert!(canonical.contains("struct GeneratedHandle"), "{canonical}");
    assert!(canonical.contains("fn new"), "{canonical}");
    assert!(!canonical.contains("DeadRootItem"), "{canonical}");
    syn::parse_file(&canonical).expect("cross-scope demand must preserve valid Rust syntax");
}

#[test]
fn removes_unobserved_literal_resets_of_boolean_locals() {
    let source = r#"
        fn parse(input: bool) {
            let mut in_quotes: bool = input;
            if in_quotes {
                println!("quoted");
            }
            if in_quotes {
                in_quotes = false;
            }
            println!("done");
        }
        fn main() { parse(true); }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("unobserved boolean resets should be removed structurally");

    assert!(!canonical.contains("in_quotes = false"), "{canonical}");
    assert!(!canonical.contains("if in_quotes {}"), "{canonical}");
    syn::parse_file(&canonical).expect("dead boolean cleanup must preserve valid Rust syntax");
}

#[test]
fn inverts_negated_if_else_conditions() {
    let source = r#"
        fn select(empty: bool) -> i64 {
            if !empty { 1 } else { 2 }
        }
        fn main() { println!("{}", select(false)); }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("negated if-else conditions should be expressed positively");

    assert!(
        canonical.contains("if empty { 2 } else { 1 }"),
        "{canonical}"
    );
    assert!(!canonical.contains("if !empty"), "{canonical}");
}

#[test]
fn combines_adjacent_if_branches_with_identical_bodies() {
    let source = r#"
        fn select(first: bool, second: bool) -> i64 {
            if first { 1 } else if second { 1 } else { 2 }
        }
        fn main() { println!("{}", select(false, true)); }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("adjacent identical branches should share one condition");

    assert!(canonical.contains("first || second"), "{canonical}");
    assert_eq!(canonical.matches("{ 1 }").count(), 1, "{canonical}");
}

#[test]
fn scopes_member_demand_to_each_inline_module() {
    let source = r#"
        mod first {
            pub enum State { First, Dead }
        }
        mod second {
            pub enum State { Second, Dead }
        }
        fn main() {
            let first = first::State::First;
            let second = second::State::Second;
            match first { first::State::First => {} }
            match second { second::State::Second => {} }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("member demand must not merge same-named types from different modules");

    assert!(canonical.contains("First"), "{canonical}");
    assert!(canonical.contains("Second"), "{canonical}");
    assert!(!canonical.contains("Dead"), "{canonical}");
}

#[test]
fn routes_reexported_union_construction_demand_into_its_module() {
    let source = r#"
        mod unions {
            pub enum __SifrUnionValue { Int(i64), Str(String) }
            impl std::fmt::Display for __SifrUnionValue {
                fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        Self::Int(value) => write!(formatter, "{value}"),
                        Self::Str(value) => write!(formatter, "{value}"),
                    }
                }
            }
        }
        pub use unions::__SifrUnionValue;
        fn describe(value: __SifrUnionValue) -> String {
            match value {
                __SifrUnionValue::Int(value) => value.to_string(),
                __SifrUnionValue::Str(value) => value,
            }
        }
        fn main() {
            println!("{}", describe(__SifrUnionValue::Int(1)));
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("external construction demand must retain and route the live union variant");

    assert!(canonical.contains("Int(i64)"), "{canonical}");
    assert!(!canonical.contains("Str(String)"), "{canonical}");
    assert!(!canonical.contains("::Str("), "{canonical}");
}

#[test]
fn keeps_root_union_arms_when_a_module_prunes_a_same_named_union() {
    let source = r#"
        mod generated {
            pub enum __SifrUnionValue { Nested(i64), Dead(String) }
            pub fn make() -> __SifrUnionValue { __SifrUnionValue::Nested(1) }
        }
        enum __SifrUnionValue { Root(i64), Dead(String) }
        impl std::fmt::Display for __SifrUnionValue {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::Root(value) => write!(formatter, "{value}"),
                    Self::Dead(value) => write!(formatter, "{value}"),
                }
            }
        }
        fn main() {
            let _nested = generated::make();
            println!("{}", __SifrUnionValue::Root(1));
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("module pruning must not mutate a distinct same-named root union");

    assert!(canonical.contains("Root(i64)"), "{canonical}");
    assert!(canonical.contains("Self::Root(value)"), "{canonical}");
    assert!(!canonical.contains("Dead(String)"), "{canonical}");
}

#[test]
fn removes_stale_wildcards_after_generated_enum_variant_pruning() {
    let source = r#"
        enum State { Ready(i64), Text(String), Dead(bool) }
        enum Single { Ready, Dead }
        enum PairState { Ok, Error }
        fn main() {
            let first = State::Ready(1);
            match first {
                State::Ready(value) => println!("{value}"),
                value => println!("{value:?}"),
            }
            let second = State::Text("ok".to_string());
            match second {
                State::Ready(_) => {},
                value => println!("{value:?}"),
            }
            let single = Single::Ready;
            match single {
                Single::Ready => println!("ready"),
                _ => println!("fallback"),
            }
            let pair = (PairState::Error, PairState::Error);
            match pair {
                (PairState::Error, PairState::Error) => println!("error"),
                _ => println!("pair fallback"),
            }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("variant pruning must leave explicit exhaustive match patterns");

    assert!(!canonical.contains("Dead"), "{canonical}");
    assert!(canonical.contains("value @ State::Text"), "{canonical}");
    assert_eq!(canonical.matches("value =>").count(), 0, "{canonical}");
    assert!(!canonical.contains("\"fallback\""), "{canonical}");
    assert!(canonical.contains("pair fallback"), "{canonical}");
    assert!(
        canonical.contains("(PairState::Error, PairState::Error)"),
        "{canonical}"
    );
}

#[test]
fn prunes_unused_module_function_that_shares_a_live_wrapper_name() {
    let source = r#"
        mod generated {
            pub struct Message { pub message: String }
            impl Message { pub fn new() -> Self { Self { message: String::new() } } }
            impl std::fmt::Display for Message {
                fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(formatter, "{}", self.message)
                }
            }
            pub fn message() -> Message { Message::new() }
        }
        pub use generated::Message;
        fn message() -> Message { Message::new() }
        fn main() { let _ = message(); }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("module demand must distinguish a wrapper from its same-named export");

    assert_eq!(canonical.matches("fn message").count(), 1, "{canonical}");
}

#[test]
fn removes_identity_wrapping_around_sifr_int_class_methods() {
    let source = r#"
        struct HeaderMap;
        impl HeaderMap {
            fn len(&self) -> SifrInt { SifrInt::from_i64(0) }
        }
        fn main() {
            let headers: HeaderMap = HeaderMap;
            assert_eq!(SifrInt::from(headers.len()), SifrInt::from_i64(0));
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("known SifrInt methods must not receive an identity conversion");

    assert!(canonical.contains("headers.len()"), "{canonical}");
    assert!(
        !canonical.contains("SifrInt::from(headers.len())"),
        "{canonical}"
    );
}

#[test]
fn removes_generic_parameters_owned_only_by_undemanded_enum_variants() {
    let source = r#"
        enum GeneratedState<T, E> {
            Ready(T),
            Dead(E),
        }
        fn main() {
            let state: GeneratedState<i64, String> = GeneratedState::Ready(1);
            let nested = Ok::<GeneratedState<i64, String>, String>(state);
            match nested {
                Ok(GeneratedState::Ready(value)) => println!("{value}"),
                Ok(GeneratedState::Dead(_)) => println!("unreachable"),
                Err(_) => println!("error"),
            }
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("variant pruning should preserve valid generic declarations and references");

    assert!(!canonical.contains("Dead"), "{canonical}");
    assert!(canonical.contains("enum GeneratedState<T>"), "{canonical}");
    assert!(canonical.contains("GeneratedState<i64>"), "{canonical}");
}

#[test]
fn removes_only_mutability_that_the_rendered_rust_body_does_not_require() {
    let source = r#"
        struct Buffer { values: Vec<i64> }
        impl Buffer {
            fn append(&mut self, value: i64) { self.values.push(value); }
        }
        pub fn inspect(mut text: String, mut buffer: Buffer) -> usize {
            let mut error = text.clone();
            let found = error.contains("x");
            buffer.append(1);
            usize::from(found)
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("rendered mutability should follow actual mutable Rust uses");

    assert!(canonical.contains("text: &str"), "{canonical}");
    assert!(!canonical.contains("mut text"), "{canonical}");
    assert!(!canonical.contains("mut error"), "{canonical}");
    assert!(canonical.contains("mut buffer: Buffer"), "{canonical}");
}

#[test]
fn preserves_mutability_for_external_task_set_operations() {
    let source = r#"
        pub async fn run(
            mut join_set: ExternalJoinSet,
            mut values: ExternalCollection,
            mut receiver: ExternalReceiver,
        ) {
            join_set.spawn(async move {});
            while join_set.join_next().await.is_some() {}
            let _ = values.get_mut(0);
            let _ = receiver.recv().await;
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("external APIs that require mutable receivers must retain mutability");

    assert!(
        canonical.contains("mut join_set: ExternalJoinSet"),
        "{canonical}"
    );
    assert!(
        canonical.contains("mut values: ExternalCollection"),
        "{canonical}"
    );
    assert!(
        canonical.contains("mut receiver: ExternalReceiver"),
        "{canonical}"
    );
}

#[test]
fn canonicalizes_result_extraction_defaults_and_single_element_ranges() {
    let source = r#"
        pub fn extract(value: Result<usize, String>) -> Result<Vec<usize>, String> {
            let value = match value {
                Ok(value) => value,
                Err(_) => return Err("invalid".to_string()),
            };
            let missing: Option<Vec<usize>> = None;
            let mut values = missing.unwrap_or(Vec::new());
            values.extend(value..value + 1);
            Ok(values)
        }
        pub fn select(value: Option<String>) -> Option<String> {
            let Some(value) = value else { return None; };
            Some(value)
        }
        pub fn direct(value: Option<String>) -> Option<String> {
            let Some(value) = value else { return None };
            Some(value)
        }
        pub fn propagate(value: Result<(), String>) -> Result<(), String> {
            if let Err(error) = value { return Err(error); }
            Ok(())
        }
        pub fn lazy(value: Option<i64>, values: &[i64]) -> i64 {
            value.unwrap_or(SifrInt::from(values.len()))
        }
        pub fn result_default(value: Result<i64, String>) -> i64 {
            value.unwrap_or_else(|_| SifrInt::from_i64(0))
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("common generated control and default shapes should be idiomatic");

    assert!(
        canonical.contains("let Ok(value) = value else"),
        "{canonical}"
    );
    assert!(canonical.contains("unwrap_or_default()"), "{canonical}");
    assert!(canonical.contains("value..=value"), "{canonical}");
    assert!(canonical.contains("let value = value?"), "{canonical}");
    assert_eq!(
        canonical.matches("let value = value?").count(),
        2,
        "{canonical}"
    );
    assert!(canonical.contains("value?;"), "{canonical}");
    assert!(canonical.contains("unwrap_or_else(||"), "{canonical}");
    assert!(
        canonical.contains("value.unwrap_or(SifrInt::from_i64(0))"),
        "{canonical}"
    );
    assert!(
        !canonical.contains("value.unwrap_or_else(|| SifrInt::from_i64(0))"),
        "{canonical}"
    );
}

#[test]
fn disambiguates_nested_bindings_against_enclosing_bindings() {
    let source = r#"
        fn main() {
            struct Task { cancellation: i64 }
            let dynamic_encoder = 1_i64;
            let cancellations = Vec::<i64>::new();
            let result = (|| {
                let dynamic_encoded = dynamic_encoder + 1;
                let Task { cancellation } = Task { cancellation: 1 };
                println!("{cancellation}");
                dynamic_encoded
            })();
            println!("{cancellations:?}");
            println!("{result}");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("nested bindings should be distinct from similar enclosing names");

    assert!(canonical.contains("dynamic_encoded_value_"), "{canonical}");
    assert!(
        canonical.contains("cancellation: cancellation_value_"),
        "{canonical}"
    );
}

#[test]
fn scopes_control_carriers_to_the_closure_that_owns_them() {
    let source = r#"
        pub fn run() -> Result<(), String> {
            let nested: Result<Result<(), String>, String> = (|| {
                let inner: Result<(), String> = (|| {
                    Err::<(), String>("failure".to_string())?;
                    Ok(())
                })();
                Ok(inner)
            })();
            nested??;
            Ok(())
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("a nested closure's carrier must not block its enclosing closure cleanup");

    assert_eq!(canonical.matches("(||").count(), 1, "{canonical}");
}

#[test]
fn emits_exact_expectations_for_macro_constants_and_dense_math_names() {
    let source = r#"
        pub fn math(a: i64, b: i64) -> Vec<bool> {
            let x = a;
            let y = b;
            let g = x + y;
            vec![g > 3.14]
        }
        pub fn chars(value: &str) -> Vec<String> {
            value.chars().map(|character| character.to_string()).collect()
        }
        pub fn char_refs(value: &[char]) -> Vec<String> {
            value.iter().map(|character| character.to_string()).collect()
        }
        pub fn first_char(value: &[char]) -> Option<String> {
            { let index = 0; value.get(index) }.map(|character| character.to_string())
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("source-contract expectations should match exact rendered lint shapes");

    assert!(canonical.contains("clippy::approx_constant"), "{canonical}");
    assert!(
        canonical.contains("clippy::many_single_char_names"),
        "{canonical}"
    );
    assert!(
        !canonical.contains("clippy::redundant_closure_for_method_calls"),
        "{canonical}"
    );
    assert!(
        canonical.contains(".map(::std::string::ToString::to_string)"),
        "{canonical}"
    );
}

#[test]
fn adds_eq_only_when_every_retained_field_type_is_proven_eq() {
    let source = r#"
        #[derive(PartialEq)]
        struct Floating { value: f64 }
        #[derive(PartialEq)]
        struct Exact { value: i64 }
        #[derive(PartialEq)]
        struct Generic<T> { values: Vec<T> }
        #[derive(PartialEq)]
        enum Values {
            Floating(Floating),
            Exact(Exact),
        }
        fn main() {
            let value = Values::Floating(Floating { value: 1.5 });
            println!("{}", value == value);
            let exact = Exact { value: 1 };
            println!("{}", exact == exact);
            let generic = Generic { values: vec![1] };
            println!("{}", generic == generic);
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("Eq derives must be added only from structural proof");

    assert!(canonical.contains("struct Exact"), "{canonical}");
    assert!(
        canonical.contains("#[derive(PartialEq, Eq)]\nstruct Exact"),
        "{canonical}"
    );
    assert!(
        canonical.contains("#[derive(PartialEq, Eq)]\nstruct Generic"),
        "{canonical}"
    );
    assert!(
        canonical.contains("#[derive(PartialEq)]\nstruct Floating"),
        "{canonical}"
    );
    assert!(
        canonical.contains("#[derive(PartialEq)]\nenum Values"),
        "{canonical}"
    );
}

#[test]
fn preserves_mutability_of_fn_mut_closure_bindings() {
    let source = r#"
        fn main() {
            let mut total = 0;
            let mut apply = || {
                total += 1;
            };
            apply();
            println!("{total}");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("FnMut closure bindings must retain callable mutability");

    assert!(canonical.contains("let mut apply = ||"), "{canonical}");
}

#[test]
fn preserves_mutability_after_folding_an_assignment_conditional() {
    let source = r#"
        fn main() {
            let value = Some(1.0_f64);
            let mut selected: f64 = 0.0_f64;
            if let Some(value) = value {
                selected = value;
            }
            if true {
                if let Some(increment) = Some(2.0_f64) {
                    selected += increment;
                }
            }
            println!("{selected}");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("folded assignment conditionals must preserve later mutability");

    assert!(canonical.contains("let mut selected"), "{canonical}");
    assert!(
        canonical.contains("value.unwrap_or(0.0_f64)"),
        "{canonical}"
    );
}

#[test]
fn preserves_mutability_after_folding_a_delayed_initialization() {
    let source = r#"
        fn main() {
            let mut selected: f64 = 0.0_f64;
            println!("initializing");
            selected = 1.0_f64;
            if true {
                selected += 2.0_f64;
            }
            println!("{selected}");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("folded delayed initializations must preserve later mutability");

    assert!(canonical.contains("let mut selected"), "{canonical}");
}

#[test]
fn retains_only_concretely_instantiated_generated_trait_impls() {
    let source = r#"
        trait GeneratedAdd: Sized {
            fn add(self, rhs: Self) -> Self;
        }
        impl GeneratedAdd for String {
            fn add(mut self, rhs: Self) -> Self { self.push_str(&rhs); self }
        }
        impl GeneratedAdd for f64 {
            fn add(self, rhs: Self) -> Self { self + rhs }
        }
        fn add_same<T: GeneratedAdd>(left: T, right: T) -> T {
            GeneratedAdd::add(left, right)
        }
        fn relay<U: GeneratedAdd>(left: U, right: U) -> U {
            add_same(left, right)
        }
        fn main() {
            println!("{}", relay("sifr".to_string(), " rust".to_string()));
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("concrete generic calls must demand their generated trait implementation");

    assert!(
        canonical.contains("impl GeneratedAdd for String"),
        "{canonical}"
    );
    assert!(
        !canonical.contains("impl GeneratedAdd for f64"),
        "{canonical}"
    );
}
