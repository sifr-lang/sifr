use super::publicize_public_enum_field_owners;

#[test]
fn ancestry_visibility_follows_transitive_associated_types() {
    let mut file: syn::File = syn::parse_quote! {
        mod nodes {
            pub enum Value { Child(Child) }
            struct Root;
            struct Mid(Root);
            struct Child(Mid);
            struct Unrelated;
            impl ::std::ops::Deref for Child { type Target = Mid; fn deref(&self) -> &Mid { &self.0 } }
            impl ::std::ops::Deref for Mid { type Target = Root; fn deref(&self) -> &Root { &self.0 } }
        }
    };
    publicize_public_enum_field_owners(&mut file.items);
    let rust = prettyplease::unparse(&file);
    for name in ["Root", "Mid", "Child"] {
        assert!(rust.contains(&format!("pub struct {name}")), "{rust}");
    }
    assert!(!rust.contains("pub struct Unrelated"), "{rust}");
}
