#[cfg(test)]
mod tests {
    #[test]
    fn item12_iterator_items_drive_borrows_without_method_name_inference() {
        let rust = clean(
            r#"
            fn row(values: &[String]) {}
            fn run(values: Vec<SifrInt>, rows: Vec<Vec<String>>, unknown: Other) {
                for x in values.iter() { println!("{}", ::std::ops::Mul::mul(&x, &x)); }
                for value in rows.iter() { row(&value); }
                for value in unknown.iter() { row(&value); }
                for (index, value) in Box::new(rows.iter().cloned().enumerate().map(|pair| (SifrInt::from(pair.0), pair.1))) {
                    let copy: Vec<String> = value.iter().cloned().collect();
                }
            }
        "#,
        );
        let compact = rust.split_whitespace().collect::<String>();
        assert!(compact.contains("::std::ops::Mul::mul(x,x)"), "{rust}");
        assert_eq!(rust.matches("row(value)").count(), 1, "{rust}");
        assert_eq!(rust.matches("row(&value)").count(), 1, "{rust}");
        assert!(
            rust.contains("let copy: Vec<String> = value.clone()"),
            "{rust}"
        );
    }
    #[test]
    fn item12_collection_cleanup_requires_both_collection_types() {
        let rust = clean(
            r#"
            fn run(values: Vec<String>, set: HashSet<String>) {
                let copied: Vec<String> = values.iter().cloned().collect();
                let set_copy: HashSet<String> = values.iter().cloned().collect();
                let flattened: Vec<String> = set.iter().cloned().collect();
                { let values: HashSet<String> = set;
                  let inner: Vec<String> = values.iter().cloned().collect(); }
                let nested: Vec<String> = { let out = values.iter().cloned().collect(); out };
            }
        "#,
        );
        assert!(
            rust.contains("let copied: Vec<String> = values.clone()"),
            "{rust}"
        );
        assert!(
            rust.contains("let set_copy: HashSet<String> = values.iter().cloned().collect()"),
            "{rust}"
        );
        assert!(
            rust.contains("let flattened: Vec<String> = set.iter().cloned().collect()"),
            "{rust}"
        );
        assert!(
            rust.contains("let inner: Vec<String> = values.iter().cloned().collect()"),
            "{rust}"
        );
        assert!(rust.contains("let out = values.clone()"), "{rust}");
    }

    #[test]
    fn item12_field_types_keep_generic_and_drop_boundaries() {
        let rust = clean(
            r#"
            struct Box<T> { value: T }
            struct Owned { message: String }
            struct Dropped { message: String }
            impl Drop for Dropped { fn drop(&mut self) {} }
            fn run() {
                let string: Box<String> = Box::new(String::new());
                let integer: Box<SifrInt> = Box::new(SifrInt::from_i64(1));
                assert_eq!(string.value.to_string(), "");
                assert_eq!(integer.value.to_string(), "1");
                let error: Owned = make_owned();
                let _message: String = error.message.clone();
                let dropped: Dropped = make_dropped();
                let _message: String = dropped.message.clone();
            }
        "#,
        );
        assert!(rust.contains("string.value.clone()"), "{rust}");
        assert!(rust.contains("integer.value.to_string()"), "{rust}");
        assert!(
            rust.contains("let _message: String = error.message;"),
            "{rust}"
        );
        assert!(
            rust.contains("let _message: String = dropped.message.clone()"),
            "{rust}"
        );
    }

    #[test]
    fn item12_result_shadow_and_slice_borrows_keep_exact_types() {
        let rust = clean(
            r#"
            struct Locale;
            impl Locale { fn new(value: &str) -> Self { Self } }
            fn run(raw: Option<String>, values: Vec<SifrInt>) -> Option<Locale> {
                let raw = raw?;
                let [first, rest @ ..] = values.as_slice() else { return None };
                assert_eq!(&first, &SifrInt::from_i64(1));
                Some(Locale::new(raw))
            }
        "#,
        );
        assert!(rust.contains("Locale::new(raw.as_str())"), "{rust}");
        assert!(
            rust.split_whitespace()
                .collect::<String>()
                .contains("assert_eq!(first,&SifrInt::from_i64(1))"),
            "{rust}"
        );
    }
    #[test]
    fn item12_typed_local_callbacks_slice_patterns_and_enum_payloads() {
        let rust = clean(
            r#"
            enum Payload { Value(String) }
            fn read(value: &String) {}
            fn run(values: &Vec<String>, payload: &Payload) {
                let [first, rest @ ..] = values.as_slice() else { return; };
                read(&first);
                let callback = |value: &str| -> String { value.to_string() };
                callback(&"value".to_string());
                match payload { Payload::Value(value) => read(&value) }
                { let first: String = String::new(); read(&first); }
            }
        "#,
        );
        assert!(rust.contains("read(first)"), "{rust}");
        assert!(rust.contains("callback(\"value\")"), "{rust}");
        assert!(
            rust.contains("Payload::Value(value) => read(value)"),
            "{rust}"
        );
        assert!(rust.contains("read(&first)"), "{rust}");
    }

    fn clean(source: &str) -> String {
        let mut file = syn::parse_file(source).expect("valid Rust fixture");
        super::rewrite(&mut file);
        prettyplease::unparse(&file)
    }

    #[test]
    fn item12_typed_borrows_preserve_reference_depth_and_lexical_shadows() {
        let rust = clean(
            r#"
            fn read(value: &str) {}
            fn deep(value: &&str) {}
            fn use_it(value: &str, source: &Option<String>) {
                read(&value);
                deep(&value);
                { let value: String = String::new(); read(&value); }
                if let Some(value) = source { let text: &String = &value; }
                if let Some((value, _)) = None { read(&value); }
                read(&value);
            }
        "#,
        );
        assert_eq!(rust.matches("read(value)").count(), 2, "{rust}");
        assert!(rust.contains("deep(&value)"), "{rust}");
        assert_eq!(rust.matches("read(&value)").count(), 2, "{rust}");
        assert!(rust.contains("let text: &String = value"), "{rust}");
    }

    #[test]
    fn item12_callback_cleanup_uses_receiving_signature_and_callable_identity() {
        let rust = clean(
            r#"
            fn add(x: &String, y: &String) -> usize { x.len() + y.len() }
            fn apply(callback: impl Fn(&String, &String) -> usize) {}
            fn outer(add: impl Fn(String, String) -> usize) {
                let callback = |x, y| add(x, y);
            }
            fn run() { apply(|x, y| add(&x, &y)); }
        "#,
        );
        assert!(rust.contains("apply(add)"), "{rust}");
        assert!(rust.contains("|x, y| add(x, y)"), "{rust}");
    }

    #[test]
    fn item12_owned_option_mapper_moves_string_without_changing_unknown_method() {
        let rust = clean(
            r#"
            fn run(value: Option<String>, other: Other) {
                value.map_or_else(String::new, |value| value.to_string());
                other.map_or_else(String::new, |value| value.to_string());
            }
        "#,
        );
        assert!(
            rust.contains("value.map_or_else(String::new, |value| value)"),
            "{rust}"
        );
        assert!(
            rust.contains("other.map_or_else(String::new, |value| value.to_string())"),
            "{rust}"
        );
    }
}
