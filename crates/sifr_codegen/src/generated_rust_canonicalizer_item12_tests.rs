use super::canonicalize_generated_rust_source;

#[test]
fn item12_qualified_string_calls_and_returns_preserve_unknown_abis() {
    let source = r#"
        mod known {
            pub struct Reader;
            impl Reader { pub fn write(&self, value: &str) -> usize { value.len() } }
            pub fn render() -> String { String::new() }
        }
        fn check(reader: &other::Reader) {
            reader.write(&"message".to_string());
            other::render().to_string();
        }
    "#;
    let canonical =
        canonicalize_generated_rust_source(source).expect("qualified string identities");
    let compact = canonical.split_whitespace().collect::<String>();
    assert!(
        compact.contains("reader.write(&\"message\".to_string())"),
        "{canonical}"
    );
    assert!(
        compact.contains("other::render().to_string()"),
        "{canonical}"
    );
}

#[test]
fn item12_string_callbacks_and_scalar_trait_methods_keep_declared_abis() {
    let source = r#"
        fn length(value: String) -> usize { value.len() }
        trait Compare { fn positive(value: SifrInt) -> bool; }
        struct Number;
        impl Compare for Number { fn positive(value: SifrInt) -> bool { value > SifrInt::from_i64(0) } }
        pub fn run() {
            let callback: fn(String) -> usize = length;
            println!("{} {}", callback("x".to_string()), Number::positive(SifrInt::from_i64(1)));
        }
    "#;
    let canonical =
        canonicalize_generated_rust_source(source).expect("callback and trait contracts");
    assert!(
        canonical.contains("fn length(value: String)"),
        "{canonical}"
    );
    assert!(
        canonical.contains("fn positive(value: SifrInt)"),
        "{canonical}"
    );
    assert!(
        !canonical.contains("positive(value: &SifrInt)"),
        "{canonical}"
    );
}

#[test]
fn item12_project_type_facts_follow_declared_module_imports() {
    let mut left = "pub fn inspect(value: &str) -> usize { value.len() }".to_string();
    let mut right = "pub fn inspect(value: String) -> String { value }".to_string();
    let mut main = r#"
        use crate::left::inspect as borrowed;
        use crate::right::inspect as owned;
        fn main() { println!("{} {}", borrowed(&"left".to_string()), owned("right".to_string())); }
    "#
    .to_string();
    super::generated_rust_canonicalizer::rewrite_named_project_borrows(&mut [
        ("left", &mut left),
        ("right", &mut right),
        ("", &mut main),
    ])
    .expect("project module identities");
    let compact = main.split_whitespace().collect::<String>();
    assert!(compact.contains("borrowed(\"left\")"), "{main}");
    assert!(compact.contains("owned(\"right\".to_string())"), "{main}");
}

#[test]
fn item12_borrow_plans_do_not_match_unrelated_qualified_callees() {
    let source = r#"
        mod local {
            pub struct Reader;
            impl Reader {
                pub fn new(value: Option<SifrInt>) -> bool { value.is_some() }
                pub fn read(&self, value: Option<String>) -> bool { value.is_some() }
                pub fn rows(values: &[String]) -> usize { values.len() }
            }
        }
        use local::Reader as KnownReader;
        fn check(reader: &other::Reader, value: Option<SifrInt>, text: Option<String>, rows: Vec<String>) {
            other::Reader::new(value);
            reader.read(text);
            other::Reader::rows(rows);
            KnownReader::new(Some(SifrInt::from_i64(1)));
        }
    "#;
    let canonical = canonicalize_generated_rust_source(source).expect("qualified callee identity");
    let compact = canonical.split_whitespace().collect::<String>();
    assert!(compact.contains("other::Reader::new(value)"), "{canonical}");
    assert!(compact.contains("reader.read(text)"), "{canonical}");
    assert!(compact.contains("other::Reader::rows(rows)"), "{canonical}");
    assert!(
        compact.contains("KnownReader::new(Some(&SifrInt::from_i64(1)))"),
        "{canonical}"
    );
}

#[test]
fn item12_generic_self_constructor_keeps_optional_scalar_borrow_plan() {
    let source = r#"
        struct Queue<T> { values: Vec<T>, limit: Option<SifrInt> }
        impl<T: Clone> Queue<T> {
            fn new(values: Vec<T>, limit: Option<SifrInt>) -> Self { Self { values, limit } }
            fn copy(&self) -> Self { Self::new(self.values.clone(), self.limit.clone()) }
        }
        fn main() { let queue = Queue::new(vec![true], None); println!("{}", queue.copy().values.len()); }
    "#;
    let canonical = canonicalize_generated_rust_source(source).expect("generic constructor ABI");
    assert!(canonical.contains("limit: Option<&SifrInt>"), "{canonical}");
    assert!(canonical.contains("self.limit.as_ref()"), "{canonical}");
}

#[test]
fn item12_final_project_slice_plan_preserves_lexical_loop_item_borrows() {
    let mut source = r#"
        fn read(row: &Vec<String>) -> usize { row.len() }
        fn rows(rows: &Vec<Vec<String>>) { for row in rows.iter() { println!("{}", read(&row)); } }
    "#
    .to_string();
    super::generated_rust_canonicalizer::rewrite_project_borrowed_string_literals(&mut [
        &mut source,
    ])
    .expect("project borrow plans");
    source = super::generated_rust_canonicalizer::finalize_formatted_generated_rust_source(&source)
        .expect("final API normalization must preserve lexical borrow facts");
    let compact = source.split_whitespace().collect::<String>();
    assert!(compact.contains("read(row)"), "{source}");
}

#[test]
fn item12_tail_empty_collection_keeps_element_type_evidence() {
    let source = r#"
        fn main() {
            let values: Vec<SifrInt> = vec![SifrInt::from_i64(1)];
            assert_eq!(values, { let empty: Vec<SifrInt> = Vec::new(); empty });
        }
    "#;
    let canonical = canonicalize_generated_rust_source(source).expect("typed empty vector");
    assert!(
        canonical
            .split_whitespace()
            .collect::<String>()
            .contains("Vec::<SifrInt>::new()"),
        "{canonical}"
    );
}

#[test]
fn item12_branch_argument_keeps_value_live_for_following_argument() {
    let source = r#"
        fn consume(value: String, other: &String) {}
        fn run(value: String, condition: bool) {
            consume(if condition { value.clone() } else { String::new() }, &value);
        }
    "#;
    let canonical = canonicalize_generated_rust_source(source).expect("branch with a live sibling");
    assert!(canonical.contains("value.clone()"), "{canonical}");
}

#[test]
fn item12_owned_callback_string_conversion_keeps_the_owned_abi() {
    let source = r#"
        fn accept(callback: fn(SifrInt) -> Result<String, Error>) {}
        fn stringify(value: SifrInt) -> Result<String, Error> {
            if value < SifrInt::from_i64(0) { return Err(Error); }
            Ok(value.to_string())
        }
        fn main() { accept(stringify); }
    "#;
    let canonical = canonicalize_generated_rust_source(source).expect("owned callback ABI");
    assert!(
        canonical.contains("fn stringify(value: SifrInt)"),
        "{canonical}"
    );
    assert!(canonical.contains("String::from(value)"), "{canonical}");
}

#[test]
fn item12_external_runtime_namespace_does_not_imply_a_borrowed_string_abi() {
    let canonical = canonicalize_generated_rust_source(
        r#"
        fn main() {
            let range = ::sifr_runtime::SifrRange::new_known_nonzero(
                SifrInt::from_i64(0), SifrInt::from_i64(4), SifrInt::from_i64(1));
            println!("{range:?}");
        }
    "#,
    )
    .expect("external runtime calls retain their declared ABI");
    assert!(!canonical.contains("&SifrInt::from_i64"), "{canonical}");
}

#[test]
fn item12_constructor_borrows_follow_declared_reexports_only() {
    let source = r#"
        mod definitions {
            pub struct Reader;
            impl Reader { pub fn new(value: Option<String>) -> Self { let _ = &value; Self } }
        }
        pub use definitions::Reader as Input;
        mod other {
            pub struct Reader;
            impl Reader { pub fn new(value: Option<String>) -> Option<String> { value } }
        }
        fn main() {
            let input = Input::new(None);
            println!("{}", other::Reader::new(Some("kept".to_string())).is_some());
        }
    "#;
    let canonical = canonicalize_generated_rust_source(source).expect("reexported constructor");
    assert!(canonical.contains("Input::new(&None)"), "{canonical}");
    assert!(
        canonical.contains("other::Reader::new(Some("),
        "{canonical}"
    );
}

#[test]
fn item12_cleanup_keeps_effectful_initializer_lookalikes() {
    let source = r#"
        fn new() -> i64 { println!("effect"); 1 }
        fn main() {
            let mut value = new();
            value = 2;
            println!("{value}");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("effectful initializers must survive assignment cleanup");

    assert!(canonical.contains("let mut value = new()"), "{canonical}");
    assert!(canonical.contains("value = 2"), "{canonical}");
}

#[test]
fn item12_macro_struct_construction_conservatively_keeps_candidate_fields() {
    let source = r#"
        macro_rules! consume { ($value:expr) => { drop($value) }; }
        fn make_number() -> i64 { println!("effect"); 4 }
        struct Config { live: i64, effect: i64 }
        impl Config {
            fn run() {
                consume!(Self { live: 1, effect: make_number() });
            }
        }
        fn main() {
            Config::run();
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("macro-contained struct construction must keep field layout aligned");

    assert!(canonical.contains("live: i64"), "{canonical}");
    assert!(canonical.contains("effect: i64"), "{canonical}");
    assert!(canonical.contains("effect: make_number()"), "{canonical}");
}

#[test]
fn item12_closed_binary_removes_stale_exact_support_function_imports() {
    let source = r#"
        pub(crate) mod sifr_generated_generated_support {
            pub(super) fn live() {}
        }
        mod consumer {
            use crate::sifr_generated_generated_support::{live, message};
            pub fn run() { live(); }
        }
        fn main() { consumer::run(); }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("stale support function import should canonicalize");

    assert!(canonical.contains("live"));
    assert!(!canonical.contains("message"));
}

#[test]
fn item12_residual_clippy_shapes_are_rewritten_inside_macros_and_control_flow() {
    let source = r#"
        use std::collections::HashMap;
        struct SifrInt;
        impl SifrInt { fn normalize_index_or_len(&self, _: usize) -> usize { 0 } }
        fn identity<T: Clone>(value: &T) -> T { value.clone() }
        fn sort_inplace(values: &mut Vec<i64>) { values.sort(); }
        fn borrowed_view(values: &mut Vec<i64>) -> usize { values.len() }
        fn recursive() {
            fn step(value: SifrInt) {
                value.normalize_index_or_len(0);
                if false { step(SifrInt); }
            }
            step(SifrInt);
        }
        fn main() {
            let bytes: Vec<u8> = vec![1];
            assert_eq!(bytes.get(0).map(|value| *value), Some(1));
            let values: HashMap<String, i64> = HashMap::new();
            assert!(!values.contains_key(&"missing".to_string()));
            assert_eq!(identity(&"text".to_string()).to_string(), "text");
            let mut lines: Vec<String> = Vec::new();
            lines = vec!["line".to_string()];
            let mut selected: Option<String> = None;
            if true { selected = Some("value".to_string()); }
            let index: SifrInt = SifrInt;
            let checked = { let sifr_generated_checked_read_index = index.clone(); sifr_generated_checked_read_index.normalize_index_or_len(lines.len()) };
            let normalized = "a\nb\r".replace('\n', " ").replace('\r', " ");
            let count = { let mut sifr_generated_count = 0; sifr_generated_count += 1; sifr_generated_count };
            let power = 2_f64.powf(10_f64);
            let mut sorted = vec![2, 1];
            sort_inplace(&mut sorted);
            let viewed = borrowed_view(&mut sorted);
            recursive();
            println!("{lines:?}{selected:?}{checked}{normalized}{count}{power}{viewed}");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("residual strict-Clippy shapes must canonicalize");

    assert!(canonical.contains(".get(0).copied()"), "{canonical}");
    assert!(
        canonical.contains("contains_key(\"missing\")"),
        "{canonical}"
    );
    assert!(
        !canonical.contains("identity(&\"text\".to_string()).to_string()"),
        "{canonical}"
    );
    assert!(!canonical.contains("let mut lines"), "{canonical}");
    assert!(!canonical.contains("let mut selected"), "{canonical}");
    assert!(
        canonical.contains("let sifr_generated_checked_read_index = &index"),
        "{canonical}"
    );
    assert!(canonical.contains("replace(["), "{canonical}");
    assert!(canonical.contains("'\\n'"), "{canonical}");
    assert!(canonical.contains("'\\r'"), "{canonical}");
    assert!(canonical.contains("saturating_add(1usize)"), "{canonical}");
    assert!(canonical.contains("2_f64.powi(10)"), "{canonical}");
    assert!(
        canonical.contains("fn sort_inplace(values: &mut [i64])"),
        "{canonical}"
    );
    assert!(
        canonical.contains("fn borrowed_view(values: &[i64])"),
        "{canonical}"
    );
    assert!(canonical.contains("borrowed_view(&sorted)"), "{canonical}");
    assert!(
        canonical.contains("fn step(value: &SifrInt)"),
        "{canonical}"
    );
    assert!(canonical.contains("step(&SifrInt)"), "{canonical}");
}

#[test]
fn item12_replacement_collapse_keeps_sequential_semantics_when_replacement_matches() {
    let source = r#"
        fn main() {
            println!("{}", "ab".replace('a', "b").replace('b', "b"));
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("replacement-sensitive chains must remain sequential");

    assert!(canonical.matches(".replace").count() >= 2, "{canonical}");
}

#[test]
fn item12_project_string_rewrites_use_local_signatures_without_cross_module_guesses() {
    let mut borrowed = r#"
        fn resolve(_: &str) {}
        fn local() { resolve(&"local".to_string()); }
    "#
    .to_string();
    let mut owned = r#"
        fn resolve(_: String) {}
        fn external() { other::resolve("external".to_string()); }
    "#
    .to_string();

    super::generated_rust_canonicalizer::rewrite_project_borrowed_string_literals(&mut [
        &mut borrowed,
        &mut owned,
    ])
    .expect("local signatures remain distinct from unresolved project calls");

    assert!(borrowed.contains("resolve(\"local\")"), "{borrowed}");
    assert!(
        owned.contains("other::resolve(\"external\".to_string())"),
        "{owned}"
    );
}

#[test]
fn item12_boolean_cleanup_preserves_pattern_conditions_and_shadowed_types() {
    let source = r#"
        fn consume(_: &str) {}
        fn main() {
            let candidate: Option<i64> = Some(1);
            let matched = if let Some(_value) = candidate { true } else { false };
            let first: Option<i64> = Some(1);
            let second: Option<i64> = Some(2);
            let chained = if let Some(left) = first && let Some(right) = second {
                left < right
            } else {
                false
            };
            let name: String = "text".to_string();
            consume(name.as_str());
            {
                let name: Vec<i64> = Vec::new();
                drop(name);
            }
            println!("{matched} {chained}");
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("pattern conditions and shadowed non-String bindings must remain sound");

    assert!(
        !canonical.contains("let Some(_value) = candidate;"),
        "{canonical}"
    );
    assert!(
        !canonical.contains("let Some(left) = first;"),
        "{canonical}"
    );
    assert!(
        !canonical.contains("let Some(right) = second;"),
        "{canonical}"
    );
    assert!(!canonical.contains("name.as_str().as_str()"), "{canonical}");
}

#[test]
fn item12_borrowed_loop_cleanup_preserves_clone_from_shadow_contract() {
    let source = r#"
        fn update(values: &[Option<String>]) {
            let mut target = String::new();
            for value in values.iter() {
                if let Some(value) = value.clone() {
                    target.clone_from(&value);
                }
            }
            println!("{target}");
        }
        fn main() { update(&[Some("x".to_string())]); }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("borrowed loop cleanup must preserve Clone::clone_from's reference contract");

    assert!(
        canonical.contains("target.clone_from(&value)"),
        "{canonical}"
    );
}

#[test]
fn item12_math_capture_keeps_the_resolved_import_without_fallback_materialization() {
    let source = r#"
        mod constants { pub const VALUE: f64 = 7.25; }
        use constants::VALUE as PI;
        fn main() { println!("{PI}"); }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("format captures must retain their resolved imported constant");

    assert!(canonical.contains("7.25"), "{canonical}");
    assert!(canonical.contains("VALUE as PI"), "{canonical}");
    assert!(!canonical.contains("std::f64::consts::PI"), "{canonical}");
}

#[test]
fn item12_api_cleanup_preserves_receiver_abi_and_module_visibility() {
    let source = r#"
        struct RootPayload;
        mod generated_union {
            pub enum RootEvent { Value(Option<crate::RootPayload>) }
        }
        mod left {
            struct Payload;
            pub(super) enum Event { Value(Option<Payload>) }
            #[derive(Clone, Copy)]
            enum Flag { On, Off }
            impl Flag {
                fn label(&self) -> &'static str {
                    match self { Self::On => "on", Self::Off => "off" }
                }
            }
        }
        mod right { struct Payload; }
        fn main() {}
    "#;

    let canonical =
        super::generated_rust_canonicalizer::finalize_formatted_generated_rust_source(source)
            .expect("API cleanup must preserve receiver ABI and module-owned visibility");

    assert!(
        canonical.contains("pub(super) struct Payload"),
        "{canonical}"
    );
    assert!(canonical.contains("pub struct RootPayload"), "{canonical}");
    assert!(canonical.contains("fn label(&self)"), "{canonical}");
    assert!(
        canonical.contains("trivially_copy_pass_by_ref"),
        "{canonical}"
    );
    let right = canonical
        .split("mod right")
        .nth(1)
        .expect("right module must remain present");
    assert!(!right.contains("pub struct Payload"), "{canonical}");

    let constructor = r#"
        struct Writer;
        impl Writer {
            fn new(values: Vec<String>) -> Self {
                for value in values.iter() { println!("{value}"); }
                Self
            }
        }
        fn main() { let _ = Writer::new(vec!["name".to_string()]); }
    "#;
    let mut constructor = canonicalize_generated_rust_source(constructor)
        .expect("constructor slice signatures must canonicalize");
    super::generated_rust_canonicalizer::rewrite_project_borrowed_string_literals(&mut [
        &mut constructor,
    ])
    .expect("qualified constructors must share the exact owner-qualified slice plan");
    assert!(
        constructor.contains("fn new(values: &[String])"),
        "{constructor}"
    );
    assert!(
        constructor.contains("Writer::new(&[\"name\".to_string()])"),
        "{constructor}"
    );
}

#[test]
fn item12_string_clone_cleanup_uses_exact_binding_type_facts() {
    let source = r#"
        fn borrowed(value: &str) -> usize { value.len() }
        fn consume(value: String) -> String { value }
        fn make_quotechar() -> String { "quoted".to_owned() }
        fn split() -> (bool, String, String) {
            (true, "name".to_string(), "value".to_string())
        }
        struct Dialect { delimiter: String }
        struct Other { delimiter: i64 }
        struct Writer;
        impl Writer {
            fn enter(&self) -> Self { Self }
            fn write(&self, _: &str) {}
        }
        fn clone_delimiter(value: &Dialect) -> String { value.delimiter.to_string() }
        fn render_other(value: &Other) -> String { value.delimiter.to_string() }
        fn use_writer(writer: &Writer) {
            let mut entered = writer.enter();
            entered.write(&"message".to_string());
        }
        fn copy_token(value: &Option<String>) -> String {
            let Some(value) = value.as_ref() else { return String::new(); };
            value.to_string()
        }
        fn main() {
            let quotechar = make_quotechar();
            let (_, _, inline_value) = split();
            let values = vec![inline_value.to_owned()];
            let borrowed_len = borrowed(quotechar.as_str());
            let consumed = consume(quotechar);
            use_writer(&Writer);
            println!("{} {} {} {} {} {}", copy_token(&values.first().cloned()), values.len(), borrowed_len, consumed, clone_delimiter(&Dialect { delimiter: "x".to_string() }), render_other(&Other { delimiter: 1 }));
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("known owned and borrowed String bindings should use explicit clone syntax");

    assert!(canonical.contains("value.clone()"), "{canonical}");
    assert!(
        canonical.contains("borrowed(quotechar.as_str())"),
        "{canonical}"
    );
    assert!(
        !canonical.contains("inline_value.to_owned()"),
        "{canonical}"
    );
    assert!(canonical.contains("value.delimiter.clone()"), "{canonical}");
    assert!(
        canonical.contains("value.delimiter.to_string()"),
        "{canonical}"
    );
    assert!(!canonical.contains("let mut entered"), "{canonical}");
    assert!(
        canonical.contains("entered.write(\"message\")"),
        "{canonical}"
    );

    let mut project_source = r#"
        fn borrowed(value: &str) -> usize { value.len() }
        fn main() {
            let quotechar: &str = "quoted";
            let quotechar = quotechar.to_owned();
            let borrowed_len = borrowed(quotechar);
            println!("{}", borrowed_len);
        }
    "#
    .to_string();
    super::generated_rust_canonicalizer::rewrite_project_borrowed_string_literals(&mut [
        &mut project_source,
    ])
    .expect("lexical project string facts must preserve owned shadow bindings");
    assert!(
        project_source.contains("borrowed(quotechar.as_str())"),
        "{project_source}"
    );
}

#[test]
fn item12_recursive_sifr_int_cleanup_preserves_callable_abi() {
    let source = r#"
        use sifr_runtime::SifrInt;
        fn main() {
            fn recurse(n: SifrInt) -> SifrInt {
                if n > SifrInt::from_i64(0) {
                    return recurse(SifrInt::from_i64(0));
                }
                n
            }
            println!("{}", recurse(SifrInt::from_i64(1)));
        }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("recursive integer cleanup must preserve the callable signature and call sites");

    assert!(canonical.contains("fn recurse(n: SifrInt)"), "{canonical}");
    assert!(
        !canonical.contains("fn recurse(n: &SifrInt)"),
        "{canonical}"
    );
}

#[test]
fn item12_scalar_borrow_plans_are_module_and_owner_qualified() {
    let source = r#"
        use sifr_runtime::SifrInt;
        mod first {
            use super::SifrInt;
            pub fn inspect(value: Option<SifrInt>) -> bool { value.is_some() }
        }
        mod second {
            use super::SifrInt;
            pub fn inspect(value: SifrInt) -> bool { value > SifrInt::from_i64(0) }
        }
        mod third {
            use super::SifrInt;
            pub fn bounded(value: SifrInt, text: &str) -> bool {
                (|| value < text.chars().count())()
            }
            pub fn present<T: Clone + 'static>(value: Option<T>) -> bool {
                if let Some(value) = value.clone() { let _ = value; }
                value.is_some()
            }
        }
        fn main() {
            println!("{} {} {} {}", first::inspect(Some(SifrInt::from_i64(1))), second::inspect(SifrInt::from_i64(1)), third::bounded(SifrInt::from_i64(1), "x"), third::present(Some(SifrInt::from_i64(1))));
        }
    "#;

    let mut canonical = source.to_string();
    super::generated_rust_canonicalizer::rewrite_project_borrowed_string_literals(&mut [
        &mut canonical,
    ])
    .expect("module-qualified scalar borrow plans must remain independent");

    assert!(
        canonical.contains("inspect(value: Option<&SifrInt>)"),
        "{canonical}"
    );
    assert!(
        canonical.contains("inspect(value: &SifrInt)"),
        "{canonical}"
    );
    assert!(canonical.contains("bounded(value: &SifrInt"), "{canonical}");
    assert!(
        canonical.contains("present<T: Clone + 'static>(value: Option<T>)"),
        "{canonical}"
    );
}

#[test]
fn item12_scalar_borrowing_preserves_callback_abis_and_let_chain_bindings() {
    let source = r#"
        use sifr_runtime::SifrInt;
        fn apply<U, F: Fn(SifrInt) -> U>(value: SifrInt, transform: F) -> U {
            transform(value)
        }
        mod callback_owner {
            use super::{apply, SifrInt};
            pub fn transform(value: SifrInt) -> bool {
                value > SifrInt::from_i64(0)
            }
            pub fn run() -> bool {
                apply(SifrInt::from_i64(1), transform)
            }
        }
        mod direct_owner {
            use super::SifrInt;
            pub fn transform(value: SifrInt) -> bool {
                value > SifrInt::from_i64(0)
            }
            pub fn run() -> bool {
                transform(SifrInt::from_i64(1))
            }
        }
        mod option_owner {
            use super::SifrInt;
            pub fn seed(value: Option<SifrInt>) -> bool {
                value.is_some()
            }
        }
        use option_owner::seed;
        mod preborrowed_owner {
            use super::SifrInt;
            pub fn reseed(value: Option<&SifrInt>) -> bool {
                value.is_some()
            }
        }
        use preborrowed_owner::reseed;
        fn bounded(maxlen: Option<SifrInt>, items: Vec<SifrInt>) -> bool {
            if maxlen.is_none() {
                return false;
            }
            if let Some(maxlen) = maxlen
                && SifrInt::from(items.len()) > maxlen
            {
                true
            } else {
                false
            }
        }
        trait Combine: Sized {
            fn combine(self, rhs: Self) -> Self;
        }
        impl Combine for String {
            fn combine(mut self, rhs: Self) -> Self {
                self.push_str(&rhs);
                self
            }
        }
        struct Entry;
        fn present(root: Option<&Entry>) -> bool {
            root.is_some()
        }
        fn main() {
            let entry = Entry;
            println!("{} {} {} {} {} {} {}", callback_owner::run(), direct_owner::run(), bounded(None, Vec::new()), present(Some(&entry)), seed(Some(SifrInt::from_i64(1))), reseed(Some(SifrInt::from_i64(2))), Combine::combine("a".to_string(), "b".to_string()));
        }
    "#;

    let mut canonical = source.to_string();
    super::generated_rust_canonicalizer::rewrite_project_borrowed_string_literals(&mut [
        &mut canonical,
    ])
    .expect("callable values and let-chain shadows must keep their own type identities");

    assert_eq!(
        canonical.matches("fn transform(value: SifrInt)").count(),
        1,
        "{canonical}"
    );
    assert_eq!(
        canonical.matches("fn transform(value: &SifrInt)").count(),
        1,
        "{canonical}"
    );
    assert!(
        canonical.contains("SifrInt::from(items.len()) > maxlen"),
        "{canonical}"
    );
    assert!(
        !canonical.contains("&SifrInt::from(items.len()) > maxlen"),
        "{canonical}"
    );
    assert!(
        canonical.contains("fn bounded(maxlen: Option<&SifrInt>"),
        "{canonical}"
    );
    assert!(
        canonical.contains("let maxlen: Option<SifrInt> = maxlen.cloned();"),
        "{canonical}"
    );
    assert!(
        canonical
            .replace(' ', "")
            .contains("seed(Some(&SifrInt::from_i64(1)))"),
        "{canonical}"
    );
    assert!(
        canonical
            .replace(' ', "")
            .contains("reseed(Some(&SifrInt::from_i64(2)))"),
        "{canonical}"
    );
    let finalized = canonicalize_generated_rust_source(&canonical)
        .expect("borrowed option call sites must remain stable during final cleanup");
    assert!(
        finalized
            .replace(' ', "")
            .contains("seed(Some(&SifrInt::from_i64(1)))"),
        "{finalized}"
    );
    assert!(
        canonical.contains("fn present(root: Option<&Entry>)"),
        "{canonical}"
    );
    let trait_canonical = canonicalize_generated_rust_source(
        r#"
            trait Combine: Sized { fn combine(self, rhs: Self) -> Self; }
            impl Combine for String {
                fn combine(mut self, rhs: Self) -> Self {
                    self.push_str(&rhs);
                    self
                }
            }
        "#,
    )
    .expect("trait implementation signatures must retain their declared ABI");
    assert!(
        trait_canonical.contains("fn combine(mut self, rhs: Self) -> Self"),
        "{trait_canonical}"
    );
}

#[test]
fn item12_assignment_cleanup_folds_recomputed_character_cache() {
    let source = r#"
        fn normalize(mut value: String) -> usize {
            let mut chars: Vec<char> = value.chars().collect::<Vec<char>>();
            if value.starts_with("prefix") {
                value = value.trim_start_matches("prefix").to_string();
                chars = value.chars().collect::<Vec<char>>();
            }
            chars.len()
        }
        fn main() { println!("{}", normalize("prefix-value".to_string())); }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("recomputed character cache should be initialized by the conditional expression");

    assert!(
        canonical.contains("let chars: Vec<char> = if"),
        "{canonical}"
    );
    assert!(!canonical.contains("let mut chars"), "{canonical}");
}

#[test]
fn item12_closed_binary_prunes_unreferenced_marker_trait_components() {
    let source = r#"
        trait Marker {}
        impl Marker for i64 {}
        fn main() { println!("ok"); }
    "#;

    let canonical = canonicalize_generated_rust_source(source)
        .expect("unused marker-trait components should be pruned together");

    assert!(!canonical.contains("trait Marker"), "{canonical}");
    assert!(!canonical.contains("impl Marker"), "{canonical}");
}
