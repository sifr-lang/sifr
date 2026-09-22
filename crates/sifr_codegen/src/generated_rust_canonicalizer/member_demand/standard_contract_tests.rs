#[test]
fn standard_spelling_does_not_replace_local_vector_methods() {
    let output = canonical_and_compile(
        r#"
        pub struct Vec<T>(pub T);
        impl Vec<i32> {
            pub fn to_vec(&self) -> Self { Self(self.0 + 1) }
            pub fn clone(&self) -> Self { Self(self.0) }
        }
        pub fn convert(value: Vec<i32>) -> Vec<i32> { value.to_vec() }
    "#,
    );
    assert!(output.contains("value.to_vec()"), "{output}");
}

#[test]
fn standard_spelling_does_not_replace_local_string_views() {
    let output = canonical_and_compile(
        r#"
        pub struct String;
        impl String {
            pub fn as_str(&self) -> &'static str { "wide" }
            pub fn len(&self) -> usize { 1 }
        }
        pub fn measure(value: &String) -> usize { value.as_str().len() }
    "#,
    );
    assert!(output.contains("value.as_str().len()"), "{output}");
}

#[test]
fn standard_spelling_does_not_combine_local_option_methods() {
    let output = canonical_and_compile(
        r#"
        pub struct Option<T>(pub T);
        impl Option<i32> {
            pub fn map(&self, f: fn(i32) -> i32) -> Self { Self(f(self.0) + 1) }
            pub fn unwrap_or_else(&self, f: fn() -> i32) -> i32 { self.0 + f() }
            pub fn map_or_else(&self, f: fn() -> i32, g: fn(i32) -> i32) -> i32 { g(self.0) + f() }
        }
        pub fn process(value: Option<i32>) -> i32 {
            value.map(|x| x).unwrap_or_else(|| 1)
        }
    "#,
    );
    assert!(
        output.contains(".map(") && output.contains(".unwrap_or_else("),
        "{output}"
    );
}

#[test]
fn temporary_clone_elision_requires_inert_derived_record_semantics() {
    let output = canonical_and_compile(
        r#"
        #[derive(Clone)]
        pub struct Plain { pub text: String }
        pub fn make_plain() -> Plain { Plain { text: String::from("plain") } }
        pub fn take_plain() -> Plain { make_plain().clone() }
        #[derive(Clone)]
        pub struct Observed { pub text: String }
        impl Drop for Observed {
            fn drop(&mut self) { std::hint::black_box(&self.text); }
        }
        pub fn make_observed() -> Observed { Observed { text: String::from("observed") } }
        pub fn keep_drop() -> Observed { make_observed().clone() }
        pub struct Custom { pub value: i32 }
        impl Clone for Custom {
            fn clone(&self) -> Self { Self { value: self.value + 1 } }
        }
        pub fn make_custom() -> Custom { Custom { value: 1 } }
        pub fn keep_custom() -> Custom { make_custom().clone() }
    "#,
    );
    assert!(!output.contains("make_plain().clone()"), "{output}");
    assert!(output.contains("make_observed().clone()"), "{output}");
    assert!(output.contains("make_custom().clone()"), "{output}");
}

#[test]
fn unrelated_map_or_method_keeps_its_declared_api() {
    let output = canonical_and_compile(
        r#"
        pub struct Widget;
        impl Widget {
            pub fn map_or(&self, fallback: Option<i32>, map: fn(i32) -> Option<i32>) -> Option<i32> {
                map(7).or(fallback)
            }
        }
        pub fn test(value: Widget) -> Option<i32> { value.map_or(None, |x| Some(x)) }
    "#,
    );
    assert!(output.contains("value.map_or("), "{output}");
}

#[test]
fn optional_custom_conversion_keeps_its_declared_result_type() {
    canonical_and_compile(
        r#"
        #[derive(Clone)]
        pub struct Intermediate;
        impl Intermediate { pub fn to_string(&self) -> String { String::from("rendered") } }
        pub struct Source;
        impl Source { pub fn to_string(&self) -> Intermediate { Intermediate } }
        pub fn first(value: Option<Source>) -> String {
            value.map(|x| x.to_string()).map_or_else(String::new, |x| x.to_string())
        }
        pub fn second(value: Option<&Intermediate>) -> String {
            value.cloned().map_or_else(String::new, |x| x.to_string())
        }
    "#,
    );
}

#[test]
fn vector_temporary_preserves_observable_element_clone() {
    let output = canonical_and_compile(
        r#"
        pub struct Custom(pub i32);
        impl Clone for Custom { fn clone(&self) -> Self { Self(self.0 + 1) } }
        pub fn values() -> Vec<Custom> { vec![Custom(1)].clone() }
    "#,
    );
    assert!(output.contains("vec![Custom(1)].clone()"), "{output}");
}

fn canonical_and_run(source: &str) {
    fn run(directory: &std::path::Path, kind: &str, source: &str) {
        let input = directory.join(format!("{kind}.rs"));
        let binary = directory.join(format!("program-{kind}"));
        std::fs::write(&input, source).expect("native source");
        let compiled = Command::new("rustc")
            .args(["--edition=2024", "--crate-name=standard_contract"])
            .arg(&input)
            .arg("-o")
            .arg(&binary)
            .output()
            .expect("rustc");
        assert!(
            compiled.status.success(),
            "{kind}: {}\n{source}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let result = Command::new(&binary).output().expect("native contract");
        assert!(
            result.status.success(),
            "{kind}: {}\n{source}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    static NEXT_NATIVE: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "sifr-standard-native-{}-{}",
        std::process::id(),
        NEXT_NATIVE.fetch_add(1, Ordering::Relaxed),
    ));
    std::fs::create_dir(&directory).expect("owned native directory");
    run(&directory, "original", source);
    let canonical = canonicalize_generated_rust_source(source).expect("canonical native source");
    run(&directory, "canonical", &canonical);
    std::fs::remove_dir_all(directory).expect("remove owned completed native evidence");
}

#[test]
fn standard_cleanup_preserves_native_clone_and_drop_effects() {
    canonical_and_run(
        r#"
        use std::sync::atomic::{AtomicUsize, Ordering};
        static DROPS: AtomicUsize = AtomicUsize::new(0);
        pub struct Custom(pub i32);
        impl Clone for Custom { fn clone(&self) -> Self { Self(self.0 + 1) } }
        #[derive(Clone)]
        pub struct Observed(pub i32);
        impl Drop for Observed {
            fn drop(&mut self) { DROPS.fetch_add(1, Ordering::SeqCst); }
        }
        fn make() -> Observed { Observed(1) }
        fn main() {
            let values = vec![Custom(1)].clone();
            assert_eq!(values.first().map(|value| value.0), Some(2));
            drop(make().clone());
            assert_eq!(DROPS.load(Ordering::SeqCst), 2);
        }
    "#,
    );
}

#[test]
fn option_fallback_retains_eager_effects_and_custom_default_values() {
    canonical_and_run(
        r#"
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        fn fallback() -> i32 { CALLS.fetch_add(1, Ordering::SeqCst); 9 }
        #[derive(Default)]
        struct Value(i32);
        impl Value { fn new() -> Self { Self(7) } }
        fn choose(value: Option<i32>) -> i32 { value.map_or(fallback(), |item| item) }
        fn custom(value: Option<Value>) -> Value { value.unwrap_or(Value::new()) }
        fn main() {
            assert_eq!(choose(Some(3)), 3);
            assert_eq!(CALLS.load(Ordering::SeqCst), 1);
            assert_eq!(custom(None).0, 7);
        }
    "#,
    );
}

#[test]
fn custom_integer_constructor_keeps_lazy_fallback_effects() {
    canonical_and_run(
        r#"
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        struct SifrInt;
        impl SifrInt {
            fn from_i64(value: i64) -> i64 { CALLS.fetch_add(1, Ordering::SeqCst); value }
        }
        fn choose(value: Option<i64>) -> i64 { value.unwrap_or_else(|| SifrInt::from_i64(0)) }
        fn main() {
            assert_eq!(choose(Some(3)), 3);
            assert_eq!(CALLS.load(Ordering::SeqCst), 0);
            assert_eq!(choose(None), 0);
            assert_eq!(CALLS.load(Ordering::SeqCst), 1);
        }
    "#,
    );
}

#[test]
fn custom_empty_string_constructor_retains_its_value_and_effects() {
    canonical_and_run(
        r#"
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        struct String;
        impl String {
            fn new() -> &'static str { CALLS.fetch_add(1, Ordering::SeqCst); "custom" }
        }
        fn read(value: &str) -> usize { value.len() }
        fn main() {
            assert_eq!(read(&String::new()), 6);
            assert_eq!(CALLS.load(Ordering::SeqCst), 1);
        }
    "#,
    );
}

#[test]
fn unused_custom_integer_constructor_retains_side_effects() {
    canonical_and_run(
        r#"
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        pub struct SifrInt;
        impl SifrInt {
            pub fn from_i64(_value: i64) -> Self {
                CALLS.fetch_add(1, Ordering::SeqCst);
                Self
            }
        }
        fn main() {
            let _unused = SifrInt::from_i64(1);
            assert_eq!(CALLS.load(Ordering::SeqCst), 1);
        }
    "#,
    );
}

#[test]
fn overwritten_custom_initializers_and_cache_assignments_preserve_effects() {
    canonical_and_run(
        r#"
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        static DROPS: AtomicUsize = AtomicUsize::new(0);
        pub struct String { pub value: usize }
        impl String {
            pub fn new() -> Self { CALLS.fetch_add(1, Ordering::SeqCst); Self { value: 1 } }
        }
        impl Drop for String {
            fn drop(&mut self) { DROPS.fetch_add(1, Ordering::SeqCst); }
        }
        fn cache() -> Vec<char> { CALLS.fetch_add(1, Ordering::SeqCst); vec!['x'] }
        fn main() {
            let mut value: String = String::new();
            value = String { value: 2 };
            assert_eq!(value.value, 2);
            assert_eq!(DROPS.load(Ordering::SeqCst), 1);
            let mut sifr_generated_chars_value = vec!['a'];
            std::hint::black_box(&sifr_generated_chars_value);
            sifr_generated_chars_value = cache();
            assert_eq!(CALLS.load(Ordering::SeqCst), 2);
            let mut ready = false;
            if ready { ready = true; }
            assert!(!ready);
        }
    "#,
    );
}

#[test]
fn custom_character_collection_keeps_overwritten_effects() {
    canonical_and_run(
        r#"
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        pub struct String;
        impl String {
            pub fn chars(&self) -> std::vec::IntoIter<char> {
                CALLS.fetch_add(1, Ordering::SeqCst);
                vec!['x'].into_iter()
            }
        }
        fn main() {
            let value = String;
            let mut chars: Vec<char> = value.chars().collect::<Vec<char>>();
            if std::hint::black_box(true) { chars = vec!['y']; }
            assert_eq!(chars, vec!['y']);
            assert_eq!(CALLS.load(Ordering::SeqCst), 1);
        }
    "#,
    );
}

#[test]
fn dead_assignment_liveness_preserves_implicit_format_captures_and_shadowed_outer_values() {
    canonical_and_run(
        r#"
        fn main() {
            let mut ready: bool = std::hint::black_box(false);
            if std::hint::black_box(true) { ready = true; }
            assert_eq!(format!("ready={ready}"), "ready=true");
            let source: String = String::from("y");
            let mut cache: Vec<char> = vec!['x'];
            std::hint::black_box(&cache);
            if std::hint::black_box(true) { cache = source.chars().collect::<Vec<char>>(); }
            assert_eq!(format!("cache={cache:?}"), "cache=['y']");
            let mut outer: bool = std::hint::black_box(false);
            outer = true;
            if std::hint::black_box(true) {
                std::hint::black_box(1usize);
                let outer: bool = false;
                std::hint::black_box((outer, 1usize));
            } else {
                std::hint::black_box(2usize);
                let outer: bool = false;
                std::hint::black_box((outer, 2usize));
            }
            assert!(outer);
        }
    "#,
    );
}

#[test]
fn identical_branch_prefixes_converge_and_preserve_effect_and_drop_order() {
    canonical_and_run(
        r#"
        use std::sync::atomic::{AtomicUsize, Ordering};
        static TRACE: AtomicUsize = AtomicUsize::new(1);
        fn mark(value: usize) {
            let _ = TRACE.fetch_update(Ordering::SeqCst, Ordering::SeqCst,
                |state| Some(state * 10 + value));
        }
        struct Guard;
        impl Drop for Guard { fn drop(&mut self) { mark(6); } }
        fn main() {
            if { mark(2); std::hint::black_box(true) } {
                let guard = Guard;
                mark(3); mark(4); mark(5);
                drop(guard);
            } else {
                let guard = Guard;
                mark(3); mark(4); mark(5);
                drop(guard);
            }
            assert_eq!(TRACE.load(Ordering::SeqCst), 123456);
            let mut outer: bool = std::hint::black_box(false);
            outer = true;
            if std::hint::black_box(true) {
                let outer: bool = false;
                std::hint::black_box(outer);
            } else {
                let outer: bool = false;
                std::hint::black_box(outer);
            }
            assert!(outer);
        }
    "#,
    );
}

#[test]
fn maximal_shared_prefix_stops_before_if_let_binding_uses() {
    canonical_and_run(
        r#"
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        fn choose(input: Option<i32>) -> i32 {
            let value = 10;
            if let Some(value) = input {
                CALLS.fetch_add(1, Ordering::SeqCst);
                value
            } else {
                CALLS.fetch_add(1, Ordering::SeqCst);
                value
            }
        }
        fn main() {
            assert_eq!(choose(Some(7)), 7);
            assert_eq!(choose(None), 10);
            assert_eq!(CALLS.load(Ordering::SeqCst), 2);
        }
    "#,
    );
}

#[test]
fn initializer_motion_preserves_macro_captures_and_conditional_reads() {
    canonical_and_run(
        r#"
        fn main() {
            let mut ready: bool = false;
            let before = format!("ready={ready}");
            ready = true;
            assert_eq!(before, "ready=false");
            assert!(ready);
            let mut flag: bool = false;
            if format!("flag={flag}") == "flag=false" { flag = true; }
            assert!(flag);
            let mut nested: bool = false;
            if std::hint::black_box(true) {
                let before = format!("nested={nested}");
                assert_eq!(before, "nested=false");
                nested = true;
            } else { nested = false; }
            assert!(nested);
            let mut captured: bool = false;
            macro_rules! observe { () => { format!("captured={captured}") } }
            let observation = observe!();
            captured = true;
            assert_eq!(observation, "captured=false");
            assert!(captured);
            macro_rules! produce { () => { 3_i32 } }
            std::hint::black_box(0_i32);
            fn macro_value() -> i32 { produce!() }
            assert_eq!(macro_value(), 3);
        }
    "#,
    );
}

#[test]
fn initializer_motion_preserves_shadowed_vector_macro_capture() {
    canonical_and_run(
        r#"
        fn main() {
            let mut values: Vec<String> = Vec::new();
            macro_rules! vec { () => {{
                assert!(values.is_empty());
                ::std::vec![String::from("kept")]
            }} }
            values = vec![];
            assert_eq!(values, ::std::vec![String::from("kept")]);
        }
    "#,
    );
}

#[test]
fn initializer_motion_preserves_nested_condition_read() {
    canonical_and_run(
        r#"
        fn main() {
            let mut ready: bool = false;
            if std::hint::black_box(false) { ready = true; }
            else if format!("ready={ready}") == "ready=false" { ready = true; }
            else { ready = false; }
            assert!(ready);
        }
    "#,
    );
}

#[test]
fn borrowed_method_receiver_preserves_custom_clone_effects() {
    canonical_and_run(
        r#"
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CLONES: AtomicUsize = AtomicUsize::new(0);
        struct Value { value: String }
        impl Clone for Value {
            fn clone(&self) -> Self {
                CLONES.fetch_add(1, Ordering::SeqCst);
                Self { value: self.value.clone() }
            }
        }
        impl Value { fn get(&self) -> &str { &self.value } }
        fn main() {
            let value = Value { value: String::from("kept") };
            assert_eq!(value.clone().get(), "kept");
            assert_eq!(CLONES.load(Ordering::SeqCst), 1);
        }
    "#,
    );
}

#[test]
fn clone_receiver_preserves_mutation_and_identity_observation() {
    canonical_and_run(
        r#"
        #[derive(Clone)] struct Value { value: String }
        impl Value {
            fn change(&mut self) -> usize { self.value.push('!'); self.value.len() }
            fn address(&self) -> usize { self as *const Self as usize }
        }
        fn main() {
            let mut value = Value { value: String::from("kept") };
            assert_eq!(value.clone().change(), 5);
            assert_eq!(value.value, "kept");
            assert_ne!(value.clone().address(), value.address());
        }
    "#,
    );
}
#[test]
fn fresh_optional_shadow_transfer_preserves_diverging_borrows() {
    canonical_and_run(
        r#"
        fn read(input: Option<&String>) -> String {
            let value: Option<String> = input.cloned();
            let Some(value) = value.clone() else { return String::from("none"); };
            value
        }
        fn retained(input: Option<String>) -> String {
            let value: Option<String> = input;
            let captured = || value.is_none();
            let Some(value) = value.clone() else { assert!(captured()); return String::from("none"); };
            assert!(!captured());
            value
        }
        fn main() {
            assert_eq!(read(Some(&String::from("kept"))), "kept");
            assert_eq!(read(None), "none");
            assert_eq!(retained(Some(String::from("kept"))), "kept");
            assert_eq!(retained(None), "none");
        }
    "#,
    );
}

#[test]
fn inert_projection_search_retains_values_and_custom_clone_effects() {
    canonical_and_run(
        r#"
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CLONES: AtomicUsize = AtomicUsize::new(0);
        struct Pair { key: String, value: String }
        impl Clone for Pair {
            fn clone(&self) -> Self {
                CLONES.fetch_add(1, Ordering::SeqCst);
                Self { key: self.key.clone(), value: self.value.clone() }
            }
        }
        fn custom(pairs: &[Pair], name: &str) -> Option<String> {
            for pair in pairs.iter().cloned() {
                if pair.key == name { return Some(pair.value.clone()); }
            }
            None
        }
        fn plain(pairs: &[(String, String)], name: &str) -> Option<String> {
            for pair in pairs.iter().cloned() {
                if pair.0 == name { return Some(pair.1.clone()); }
            }
            None
        }
        fn main() {
            let pairs = vec![(String::from("a"), String::from("first")), (String::from("b"), String::from("second"))];
            assert_eq!(plain(&pairs, "b"), Some(String::from("second")));
            assert_eq!(plain(&pairs, "c"), None);
            assert_eq!(pairs[0].1, "first");
            let custom_pairs = vec![Pair { key: String::from("a"), value: String::from("first") }];
            assert_eq!(custom(&custom_pairs, "a"), Some(String::from("first")));
            assert_eq!(CLONES.load(Ordering::SeqCst), 1);
        }
    "#,
    );
}

#[test]
fn borrowed_getter_preserves_cloned_storage_identity() {
    canonical_and_run(
        r#"
        #[derive(Clone)] struct Value { value: String }
        impl Value { fn get(&self) -> &str { &self.value } }
        fn main() {
            let value = Value { value: String::from("kept") };
            assert_ne!(value.clone().get().as_ptr(), value.get().as_ptr());
        }
    "#,
    );
}
#[test]
fn projected_field_preserves_custom_parent_clone() {
    canonical_and_run(
        r#"
        struct Value { kind: String }
        impl Clone for Value {
            fn clone(&self) -> Self { Self { kind: format!("{}!", self.kind) } }
        }
        fn main() {
            let value = Value { kind: String::from("kept") };
            let projected = value.clone().kind.clone();
            assert_eq!(projected, "kept!");
            assert_eq!(value.kind, "kept");
        }
    "#,
    );
}

#[test]
fn projection_search_preserves_custom_comparison_identity() {
    canonical_and_run(
        r#"
        struct Other(*const u8);
        impl PartialEq<Other> for String {
            fn eq(&self, other: &Other) -> bool { self.as_ptr() != other.0 }
        }
        fn search(pairs: &[(String, String)], name: Other) -> Option<String> {
            for pair in pairs.iter().cloned() {
                if pair.0 == name { return Some(pair.1.clone()); }
            }
            None
        }
        fn main() {
            let pairs = vec![(String::from("key"), String::from("value"))];
            let name = Other(pairs[0].0.as_ptr());
            assert_eq!(search(&pairs, name), Some(String::from("value")));
        }
    "#,
    );
}

#[test]
fn borrowed_projection_comparison_preserves_custom_parent_clone() {
    canonical_and_run(
        r#"
        struct Value { kind: String }
        impl Clone for Value {
            fn clone(&self) -> Self { Self { kind: format!("{}!", self.kind) } }
        }
        fn main() {
            let value = Value { kind: String::from("kept") };
            assert!(value.clone().kind == "kept!");
            assert_eq!(value.kind, "kept");
            let pair = (String::from("key"), String::from("value"));
            assert!(pair.clone().1 == "value");
            assert_eq!(pair.0, "key");
        }
    "#,
    );
}

#[test]
fn borrowed_projection_keeps_the_snapshot_before_sibling_mutation() {
    canonical_and_run(
        r#"
        #[derive(Clone)] struct Value { text: String }
        fn replace(value: &mut Value) -> String {
            value.text = String::from("after");
            String::from("before")
        }
        fn main() {
            let mut value = Value { text: String::from("before") };
            let equal = value.clone().text == replace(&mut value);
            assert!(equal);
            assert_eq!(value.text, "after");
            let mut formatted = Value { text: String::from("before") };
            let output = format!("{} {}", formatted.clone().text, replace(&mut formatted));
            assert_eq!(output, "before before");
            assert_eq!(formatted.text, "after");
        }
    "#,
    );
}
