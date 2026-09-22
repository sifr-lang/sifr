#[cfg(test)]
mod tests {
    #[test]
    fn generic_receiver_cleanup_uses_declared_receiver_without_inferring_generic_arguments() {
        let rust = clean(
            r#"
            struct Sender<T> { value: T }
            impl<T> Sender<T> { fn send(&mut self, value: T) { self.value = value; } }
            fn run(mut sender: Sender<String>, value: String) {
                (&mut sender).send(value);
            }
        "#,
        );
        assert!(rust.contains("sender.send(value)"), "{rust}");
    }

    #[test]
    fn mutable_receiver_cleanup_requires_declared_borrow_contract() {
        let rust = clean(
            r#"
            struct Owner;
            impl Owner { fn change(&mut self) {} }
            fn run(mut owner: Owner, mut opaque: Other) {
                (&mut owner).change();
                (&mut opaque).change();
            }
        "#,
        );
        assert!(rust.contains("owner.change()"), "{rust}");
        assert!(rust.contains("(&mut opaque).change()"), "{rust}");
    }

    #[test]
    fn iterator_items_drive_borrows_without_method_name_inference() {
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
    fn collection_cleanup_requires_both_collection_types() {
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
    fn field_types_keep_generic_and_drop_boundaries() {
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
            !rust.contains("let _message: String = error.message;"),
            "{rust}"
        );
        assert!(
            rust.contains("let _message: String = dropped.message.clone()"),
            "{rust}"
        );
    }

    #[test]
    fn result_shadow_and_slice_borrows_keep_exact_types() {
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
    fn typed_local_callbacks_slice_patterns_and_enum_payloads() {
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
    fn typed_borrows_preserve_reference_depth_and_lexical_shadows() {
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
    fn callback_cleanup_uses_receiving_signature_and_callable_identity() {
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
    fn owned_option_mapper_moves_string_without_changing_unknown_method() {
        let rust = clean(
            r#"
            fn run(value: Option<String>, other: Other) {
                value.map_or_else(String::new, |value| value.to_string());
                other.map_or_else(String::new, |value| value.to_string());
            }
        "#,
        );
        assert!(rust.contains("value.unwrap_or_else(String::new)"), "{rust}");
        assert!(
            rust.contains("other.map_or_else(String::new, |value| value.to_string())"),
            "{rust}"
        );
    }

    #[test]
    fn empty_string_borrow_cleanup_requires_a_string_slice_boundary() {
        let rust = clean(
            r#"
        fn borrowed(value: &str) {}
        fn owned(value: &String) {}
        fn main() {
            borrowed(&String::new());
            owned(&String::new());
        }
    "#,
        );
        assert!(rust.contains("borrowed(\"\")"), "{rust}");
        assert!(rust.contains("owned(&String::new())"), "{rust}");
    }
    #[test]
    fn task_local_declaration_keeps_known_standard_string_contract() {
        let rust = clean(
            r#"
            mod support {
                ::tokio::task_local! {
                    static LABEL: String;
                    pub static VALUE: usize
                }
                fn borrowed(value: &str) {}
                fn run() { borrowed(&String::new()); }
            }
        "#,
        );
        assert!(rust.contains("borrowed(\"\")"), "{rust}");
    }

    #[test]
    fn unknown_task_local_macros_keep_their_opaque_type_contract() {
        for declaration in [
            "tokio::task_local! { static LABEL: String; }",
            "::other::task_local! { static LABEL: String; }",
            "::tokio::task_local! { unknown syntax }",
            "::tokio::task_local! { #[unknown_attribute] static LABEL: String; }",
            "::tokio::task_local! { static String: usize; }",
        ] {
            let source = format!(
                "{declaration}\nfn borrowed(value: &str) {{}}\nfn run() {{ borrowed(&String::new()); }}"
            );
            let rust = clean(&source);
            assert!(rust.contains("borrowed(&String::new())"), "{rust}");
        }
    }
    #[test]
    fn task_local_declaration_does_not_obscure_standard_clone_methods() {
        let rust = clean(
            r#"
            ::tokio::task_local! { static LABEL: String; }
            fn run(value: String) -> String { value.to_string() }
        "#,
        );
        assert!(!rust.contains("value.to_string()"), "{rust}");
    }
    #[test]
    fn exact_float_equality_has_a_scoped_language_contract() {
        let rust = clean(
            r#"
            fn compare(left: f64, right: f64) -> bool { left == right }
            fn power(value: f64) -> bool { value.powi(3) != 8.0_f64 }
            fn zero(value: f64) -> bool { value == 0.0_f64 }
            fn infinity(value: f64) -> bool { value == f64::INFINITY }
            struct Custom;
            fn custom(value: Custom) -> bool { value == 8.0_f64 }
        "#,
        );
        assert_eq!(rust.matches("clippy::float_cmp").count(), 2, "{rust}");
        assert!(rust.contains("left == right"), "{rust}");
        assert!(rust.contains("value.powi(3) != 8.0_f64"), "{rust}");
    }
    #[test]
    fn generic_calls_preserve_concrete_parameter_borrow_contracts() {
        let rust = clean(
            r#"
            fn generic<T>(value: T, bound: Option<&String>) {}
            fn run(value: &String) { generic(1_i32, Some(&value)); }
        "#,
        );
        assert!(rust.contains("generic(1_i32, Some(value))"), "{rust}");
    }

    #[test]
    fn runtime_integer_parser_uses_its_exact_string_slice_boundary() {
        let rust = clean(
            r#"
            fn run(value: &str) { SifrInt::parse_decimal(&value, 100usize); }
        "#,
        );
        assert!(
            rust.contains("SifrInt::parse_decimal(value, 100usize)"),
            "{rust}"
        );
        let shadowed = clean(
            r#"
            struct SifrInt;
            fn run(value: &str) { SifrInt::parse_decimal(&value, 100usize); }
        "#,
        );
        assert!(
            shadowed.contains("SifrInt::parse_decimal(&value, 100usize)"),
            "{shadowed}"
        );
    }

    #[test]
    fn standard_primitive_slice_iteration_uses_copy() {
        let rust = clean(
            r#"
            fn run(values: &[u8], strings: &[String]) {
                let bytes = values.iter().cloned();
                let text = strings.iter().cloned();
            }
        "#,
        );
        assert!(rust.contains("values.iter().copied()"), "{rust}");
        assert!(rust.contains("strings.iter().cloned()"), "{rust}");
    }
}
