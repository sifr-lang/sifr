#[test]
fn public_reexports_expose_owners_but_not_private_support_functions() {
    let file: syn::File = syn::parse_quote! {
        mod support { pub fn helper() {} pub struct Owner; }
        pub use support::Owner as PublicOwner;
        pub mod visible { pub fn public_function() {} }
    };
    let exports = super::Exports::collect(&file.items);
    assert!(exports.owner(&[], &syn::parse_quote!(PublicOwner)));
    assert!(exports.owner(&["support".to_owned()], &syn::parse_quote!(Owner)));
    assert!(!exports.function(&["support".to_owned()], &syn::parse_quote!(helper)));
    assert!(exports.function(&["visible".to_owned()], &syn::parse_quote!(public_function)));
}
