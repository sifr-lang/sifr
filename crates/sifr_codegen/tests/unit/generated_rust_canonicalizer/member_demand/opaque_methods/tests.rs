#[test]
fn shadowed_standard_namespace_keeps_opaque_ufcs_demand() {
    let mut file = syn::parse_file(
        r#"
            extern crate unknown as std;
            trait SifrGeneratedOpaqueExampleMethods { fn sub(&self); }
            fn run() { ::std::ops::Sub::sub(); }
        "#,
    )
    .expect("test syntax");
    super::prune(&mut file);
    assert!(quote::quote!(#file).to_string().contains("fn sub"));
}
