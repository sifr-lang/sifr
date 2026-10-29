#[test]
fn mutable_copy_receiver_does_not_get_shared_receiver_expectation() {
    let signature = syn::parse_quote!(fn change(&mut self));
    let body = syn::parse_quote!({});
    let mut attrs = Vec::new();
    super::refresh_function_expectations(
        &mut attrs,
        &signature,
        &body,
        super::FunctionExpectationContext {
            owner_has_display: false,
            copy_receiver_lint: true,
            trait_impl: false,
            restricted_api: true,
            ref_option_lint: true,
        },
    );
    assert!(
        !quote::quote!(#(#attrs)*)
            .to_string()
            .contains("trivially_copy_pass_by_ref")
    );
}
