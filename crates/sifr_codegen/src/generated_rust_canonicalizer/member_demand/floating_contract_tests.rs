
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
