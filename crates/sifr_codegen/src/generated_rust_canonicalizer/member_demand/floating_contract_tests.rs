
#[test]
fn floating_source_operations_preserve_rounding_overflow_and_signed_zero() {
    canonical_and_run(r#"
        fn separate(a: f64, b: f64, c: f64) -> f64 { a * b + c }
        fn midpoint(a: f64, b: f64) -> f64 { (a + b) / 2.0_f64 }
        fn power(value: f64) -> f64 { value.powf(0.5_f64) }
        fn main() {
            let a = std::hint::black_box(1e16_f64);
            let b = std::hint::black_box(1.0000000000000002_f64);
            assert_ne!(separate(a, b, -a).to_bits(), a.mul_add(b, -a).to_bits());
            let maximum = std::hint::black_box(f64::MAX);
            assert!(midpoint(maximum, maximum).is_infinite());
            assert!(maximum.midpoint(maximum).is_finite());
            let zero = std::hint::black_box(-0.0_f64);
            assert_ne!(power(zero).to_bits(), zero.sqrt().to_bits());
        }
    "#);
}

#[test]
fn owned_field_snapshot_survives_a_prior_borrow_used_later() {
    canonical_and_run(r#"
        #[derive(Clone)] struct Record { text: String }
        fn main() {
            let value = Record { text: String::from("kept") };
            let borrowed = &value;
            let copied = value.text.clone();
            assert_eq!(copied, borrowed.text);
        }
    "#);
}

#[test]
fn inert_block_results_and_assert_fields_keep_native_effects() {
    canonical_and_run(r#"
        #[derive(Clone)] struct Record { message: String }
        fn build() -> Vec<i64> {
            { let mut values = vec![1_i64]; values.extend([2_i64]); values }.clone()
        }
        fn main() {
            let record = Record { message: String::from("detail") };
            let mut counter = 0_i64;
            assert!({ counter += 1; true }, "{}", record.message.clone());
            assert_eq!(counter, 1_i64);
            assert_eq!(build(), vec![1_i64, 2_i64]);
        }
    "#);
}

#[test]
fn typed_initializer_boolean_folding_is_complete() {
    let canonical = canonical_and_compile(r#"
        fn choose(a: bool, b: bool) -> bool {
            let mut result: bool = false;
            if a || b { result = true; }
            result
        }
        fn main() { assert!(choose(false, true)); }
    "#);
    assert!(!canonical.contains("else"), "{canonical}");
}


#[test]
fn resolved_string_call_contracts_borrow_only_concrete_str_inputs() {
    let canonical = canonical_and_compile(r#"
        fn read(value: &str) -> usize { value.len() }
        fn owned_view(value: &String) -> usize { value.capacity() }
        fn positive(value: String) -> usize { read(&value) }
        fn preserve(value: String) -> usize { owned_view(&value) }
        fn callback(value: String, read: fn(&String) -> usize) -> usize { read(&value) }
        fn main() {
            assert_eq!(positive(String::from("abc")), 3);
            assert!(preserve(String::from("abc")) >= 3);
            assert!(callback(String::from("abc"), owned_view) >= 3);
        }
    "#);
    assert!(canonical.contains("fn positive(value: &str)"), "{canonical}");
    assert!(canonical.contains("fn preserve(value: String)"), "{canonical}");
    assert!(canonical.contains("fn callback(value: String"), "{canonical}");
}

#[test]
fn resolved_string_calls_keep_import_and_callable_value_identity() {
    canonical_and_run(r#"
        mod strings { pub fn read(value: &str) -> usize { value.len() } }
        use strings::read as imported;
        fn positive(value: String) -> usize { imported(&value) }
        fn retained(value: String) -> usize { imported(&value) }
        fn main() {
            let callable: fn(String) -> usize = retained;
            assert_eq!(positive(String::from("abc")), 3);
            assert_eq!(callable(String::from("abcd")), 4);
        }
    "#);
}

#[test]
fn unused_explicit_drop_binding_is_not_discarded() {
    canonical_and_run(r#"
        static DROPS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        struct Effect;
        impl Drop for Effect {
            fn drop(&mut self) { DROPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst); }
        }
        fn main() {
            let value = Effect;
            {
                let _unused: Effect = value;
            }
            assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst), 1);
        }
    "#);
}


#[test]
fn resolved_string_borrowing_preserves_opaque_macro_owned_contracts() {
    let canonical = canonicalize_generated_rust_source(r#"
        fn read(value: &str) -> usize { value.len() }
        fn retain(value: String) -> usize {
            external::observe_context!();
            read(&value)
        }
    "#).expect("opaque macro syntax remains opaque");
    assert!(canonical.contains("fn retain(value: String)"), "{canonical}");
}


#[test]
fn string_borrow_planning_converges_through_deep_call_chains() {
    let mut source = String::new();
    for index in 0..20 {
        source.push_str(&format!("fn step{index}(value: String) -> usize {{ step{}(value) }}\n", index + 1));
    }
    source.push_str(r#"
        fn step20(value: String) -> usize { value.len() }
        fn main() { assert_eq!(step0(String::from("complete")), 8); }
    "#);
    let canonical = canonical_and_compile(&source);
    assert!(!canonical.contains("value: String"), "{canonical}");
}

#[test]
fn string_borrow_planning_adapts_only_standard_string_views() {
    canonical_and_run(r#"
        fn read(value: &str) -> usize { value.len() }
        fn forwarded(value: String) -> usize { read(value.as_str()) }
        fn main() { assert_eq!(forwarded(String::from("view")), 4); }
    "#);
    let canonical = canonical_and_compile(r#"
        struct String;
        impl String { fn as_str(&self) -> &str { "custom" } }
        fn read(value: &str) -> usize { value.len() }
        fn retain(value: String) -> usize { read(value.as_str()) }
        fn main() { assert_eq!(retain(String), 6); }
    "#);
    assert!(canonical.contains("fn retain(value: String)"), "{canonical}");
}


#[test]
fn owned_ingress_fields_move_only_before_any_borrow_or_capture() {
    let output = canonical_and_compile(r#"
        #[derive(Clone)] struct Record { text: String }
        fn take(value: Record) -> String { value.text.clone() }
        fn borrowed(value: &Record) -> String { value.text.clone() }
        fn aliased(value: Record) -> String {
            let alias = &value;
            let copied = value.text.clone();
            assert_eq!(alias.text, copied);
            copied
        }
        fn main() {
            assert_eq!(take(Record { text: String::from("take") }), "take");
            assert_eq!(borrowed(&Record { text: String::from("borrow") }), "borrow");
            assert_eq!(aliased(Record { text: String::from("alias") }), "alias");
        }
    "#);
    let file = syn::parse_file(&output).expect("canonical parses");
    let take = file.items.iter().find_map(|item| match item {
        syn::Item::Fn(function) if function.sig.ident == "take" => Some(function),
        _ => None,
    }).expect("take retained");
    assert!(!quote::ToTokens::to_token_stream(&take.block).to_string().contains("clone"), "{output}");
}

#[test]
fn owned_iterator_pattern_transfers_a_terminal_inert_value() {
    canonical_and_run(r#"
        fn choose(values: &[String], index: usize) -> String {
            for (current, value) in Box::new(values.iter().cloned().enumerate().map(|pair| (pair.0, pair.1))) {
                if current == index { return value.clone(); }
            }
            String::new()
        }
        fn main() { assert_eq!(choose(&[String::from("first"), String::from("second")], 1), "second"); }
    "#);
}


#[test]
fn constructor_self_results_keep_concrete_inert_identity() {
    let canonical = canonical_and_compile(r#"
        mod records {
            #[derive(Clone)] pub struct Record { pub value: String }
            impl Record { pub fn new(value: String) -> Self { Self { value } } }
        }
        fn main() { assert_eq!(records::Record::new(String::from("owned")).clone().value, "owned"); }
    "#);
    assert!(!canonical.contains(".clone()"), "{canonical}");
}

#[test]
fn float_match_payloads_keep_exact_source_arithmetic() {
    let canonical = canonical_and_compile(r#"
        fn compute(input: Result<(f64, f64), ()>, value: f64) -> f64 {
            let (left, right) = match input {
                Ok(pair) => pair,
                Err(_) => { return 0.0_f64; }
            };
            value - left * right
        }
        fn main() { assert_eq!(compute(Ok((2.0_f64, 3.0_f64)), 10.0_f64), 4.0_f64); }
    "#);
    assert!(canonical.contains("clippy::suboptimal_flops"), "{canonical}");
    assert!(canonical.contains("value - left * right"), "{canonical}");
}


#[test]
fn generic_callback_borrow_adaptation_preserves_native_results() {
    canonical_and_run(r#"
        fn fold<T: Clone, U: Clone>(callback: impl Fn(&U, &T) -> U, values: &[T], initial: &U) -> U {
            let mut result = initial.clone();
            for value in values { result = callback(&result, value); }
            result
        }
        fn add(left: &i64, right: &i64) -> i64 { *left + *right }
        fn main() {
            let values: Vec<i64> = vec![1_i64, 2_i64, 3_i64];
            assert_eq!(fold(|left, right| add(&left, &right), &values, &0_i64), 6_i64);
        }
    "#);
}


#[test]
fn inert_string_choice_keeps_snapshots_before_effectful_conditions() {
    canonical_and_run(r#"
        fn mutate(value: &mut String) -> bool { value.push_str("-changed"); false }
        fn choose(flag: bool, token: &str, inline: String) -> String {
            let mut selected: String = token.to_string();
            if flag { selected = inline; }
            selected
        }
        fn main() {
            assert_eq!(choose(false, "base", String::from("inline")), "base");
            assert_eq!(choose(true, "base", String::from("inline")), "inline");
            let mut token = String::from("before");
            let inline = String::from("inline");
            let mut selected: String = token.to_string();
            if mutate(&mut token) { selected = inline; }
            assert_eq!(selected, "before");
            assert_eq!(token, "before-changed");
        }
    "#);
}


#[test]
fn repeated_macro_values_keep_module_qualified_constructors() {
    canonical_and_run(r#"
        mod records {
            #[derive(Clone)] pub struct Record { pub value: i32 }
            impl Record { pub fn new(value: i32) -> Self { Self { value } } }
        }
        fn main() {
            let values = vec![records::Record::new(7); 2];
            assert_eq!(values[0].value + values[1].value, 14);
        }
    "#);
}


#[test]
fn borrowed_string_parameters_preserve_owned_copies_and_later_views() {
    let source = r#"
        fn valid(text: &str) -> bool { text == "kept" }
        fn snapshot(text: String) -> String {
            let mut copied: String = text.clone();
            if !valid(&text) { copied = String::from("invalid"); }
            copied
        }
        fn main() {
            assert_eq!(snapshot(String::from("kept")), "kept");
            assert_eq!(snapshot(String::from("other")), "invalid");
        }
    "#;
    canonical_and_run(source);
    let canonical = canonicalize_generated_rust_source(source).expect("canonical snapshot");
    assert!(canonical.contains("fn snapshot(text: &str)"), "{canonical}");
    assert!(canonical.contains("text.to_string()"), "{canonical}");
}


#[test]
fn standard_option_fallback_result_retains_owned_payload_facts() {
    let source = r#"
        fn choose(values: &[String], fallback: String) -> String {
            let value = values.get(0).cloned().unwrap_or(fallback);
            value.clone()
        }
        fn main() {
            assert_eq!(choose(&[String::from("found")], String::from("fallback")), "found");
            assert_eq!(choose(&[], String::from("fallback")), "fallback");
        }
    "#;
    canonical_and_run(source);
    let canonical = canonicalize_generated_rust_source(source).expect("typed Option fallback");
    assert!(!canonical.contains("value.clone()"), "{canonical}");
}


#[test]
fn local_context_guard_fields_keep_entered_reference_contracts() {
    let source = r#"
        static DROPS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        struct File;
        impl File {
            fn enter(&self) -> &Self { self }
            fn write(&self, text: &str) { assert_eq!(text, "works"); }
        }
        fn main() {
            {
                struct Guard { ctx: File }
                impl Drop for Guard {
                    fn drop(&mut self) { DROPS.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
                }
                let guard = Guard { ctx: File };
                let file = guard.ctx.enter();
                file.write(&"works".to_string());
            }
            assert_eq!(DROPS.load(std::sync::atomic::Ordering::Relaxed), 1);
        }
    "#;
    canonical_and_run(source);
    let canonical = canonicalize_generated_rust_source(source).expect("local guard facts");
    assert!(!canonical.contains(r#""works".to_string()"#), "{canonical}");
}

#[test]
fn local_type_shadow_keeps_its_distinct_string_receiving_contract() {
    let source = r#"
        struct File;
        impl File { fn write(&self, text: &str) { assert_eq!(text, "works"); } }
        fn main() {
            struct File;
            impl File { fn enter(&self) -> &Self { self }
                fn write(&self, text: &String) { assert!(text.capacity() >= 5); } }
            struct Guard { ctx: File }
            let guard = Guard { ctx: File };
            let file = guard.ctx.enter();
            file.write(&"works".to_string());
        }
    "#;
    canonical_and_run(source);
    let canonical = canonicalize_generated_rust_source(source).expect("local shadow contracts");
    assert!(canonical.contains(r#""works".to_string()"#), "{canonical}");
}

#[test]
fn explicit_owned_assertion_consumption_preserves_payload_drop_timing() {
    canonical_and_run(r#"
        #[derive(Clone)] struct Payload(std::sync::Arc<std::sync::atomic::AtomicUsize>);
        impl Drop for Payload {
            fn drop(&mut self) { self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst); }
        }
        fn consume_some<T>(value: Option<T>) { assert!(value.is_some()); std::mem::drop(value); }
        fn consume_none<T>(value: Option<T>) { assert!(value.is_none()); std::mem::drop(value); }
        fn main() {
            let drops = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            consume_some(Some(Payload(drops.clone())));
            assert_eq!(drops.load(std::sync::atomic::Ordering::SeqCst), 1);
            drops.store(0, std::sync::atomic::Ordering::SeqCst);
            let observed = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(usize::MAX));
            let observed_hook = observed.clone();
            let drops_hook = drops.clone();
            let previous = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |_| {
                observed_hook.store(drops_hook.load(std::sync::atomic::Ordering::SeqCst),
                    std::sync::atomic::Ordering::SeqCst);
            }));
            let result = std::panic::catch_unwind(|| consume_none(Some(Payload(drops.clone()))));
            std::panic::set_hook(previous);
            assert!(result.is_err());
            assert_eq!(observed.load(std::sync::atomic::Ordering::SeqCst), 0);
            assert_eq!(drops.load(std::sync::atomic::Ordering::SeqCst), 1);
        }
    "#);
}

#[test]
fn local_imports_keep_module_definitions_and_distinct_guard_contracts() {
    for imports in [
        "use other::File;",
        "use other::{File as Imported}; type File = Imported;",
        "use other::*;",
    ] {
        for retained in ["", "let _ = std::hint::black_box(other::File); let _ = std::hint::black_box(crate::File);"] {
            let source = format!(r#"
                struct File;
                impl File {{
                    fn enter(&self) -> &Self {{ self }}
                    fn write(&self, text: &str) {{ assert_eq!(text, "works"); }}
                }}
                mod other {{
                    pub struct File;
                    impl File {{
                        pub fn enter(&self) -> &Self {{ self }}
                        pub fn write(&self, text: &String) {{ assert!(text.capacity() >= 5); }}
                    }}
                }}
                fn main() {{
                    {imports}
                    {retained}
                    struct Guard {{ ctx: File }}
                    let guard: Guard = Guard {{ ctx: File }};
                    let file = guard.ctx.enter();
                    file.write(&"works".to_string());
                }}
            "#);
            // A type alias is not a unit-struct constructor.
            let source = if imports.contains("Imported") {
                source.replace("ctx: File };", "ctx: Imported };")
            } else { source };
            canonical_and_run(&source);
            let canonical = canonicalize_generated_rust_source(&source).expect("local import contracts");
            assert!(canonical.contains(r#""works".to_string()"#), "{canonical}");
        }
    }
}

#[test]
fn generic_and_macro_declared_local_guards_keep_distinct_contracts() {
    for declaration in [
        "struct Guard<T> { ctx: File, extra: T }",
        "macro_rules! local_file { () => { struct File; impl File { fn enter(&self) -> &Self { self } fn write(&self, text: &String) { assert!(text.capacity() >= 5); } } } } local_file!(); struct Guard<T> { ctx: File, extra: T }",
    ] {
        let source = format!(r#"
            struct File;
            impl File {{
                fn enter(&self) -> &Self {{ self }}
                fn write(&self, text: &str) {{ assert_eq!(text, "works"); }}
            }}
            fn main() {{
                {declaration}
                let _ = std::hint::black_box(crate::File);
                let guard: Guard<u8> = Guard {{ ctx: File, extra: 1 }};
                let file = guard.ctx.enter();
                file.write(&"works".to_string());
                assert_eq!(guard.extra, 1);
            }}
        "#);
        canonical_and_run(&source);
    }
}
