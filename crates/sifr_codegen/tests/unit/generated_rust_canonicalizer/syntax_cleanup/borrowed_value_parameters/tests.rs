use super::rewrite_borrow_only_value_parameters;

#[test]
fn borrowed_trait_object_call_inside_assert_rewrites_boxed_argument() {
    let mut file: syn::File = syn::parse_quote! {
        trait Unwrappable {
            fn unwrap(&self) -> i64;
        }
        fn through_protocol(value: Box<dyn Unwrappable>) -> i64 {
            value.unwrap()
        }
        fn main() {
            assert!(through_protocol(Box::new(Carrier::new())) == 5);
        }
    };
    rewrite_borrow_only_value_parameters(&mut file, false);
    let rust = prettyplease::unparse(&file);
    assert!(
        rust.contains("fn through_protocol(value: &dyn Unwrappable)"),
        "{rust}"
    );
    let compact = rust
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    assert!(
        compact.contains("through_protocol(&Carrier::new())"),
        "{rust}"
    );
    assert!(!rust.contains("through_protocol(Box::new"), "{rust}");
}
