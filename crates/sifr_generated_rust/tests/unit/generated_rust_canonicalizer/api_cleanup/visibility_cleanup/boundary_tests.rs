use super::publicize_public_enum_field_owners;

#[test]
fn ancestry_visibility_resolves_aliases_and_keeps_generic_names_private() {
    let mut file: syn::File = syn::parse_quote! {
        mod hidden { pub(crate) struct Parent; }
        mod other { struct Parent; }
        use crate::hidden::Parent as ImportedParent;
        pub struct Child;
        impl ::std::ops::Deref for Child {
            type Target = ImportedParent;
            fn deref(&self) -> &Self::Target { &ImportedParent }
        }
        struct Parameter;
        pub struct Generic<Parameter>(Parameter);
        impl<Parameter> ::std::ops::Deref for Generic<Parameter> {
            type Target = Parameter;
            fn deref(&self) -> &Parameter { &self.0 }
        }
        struct PrivatePayload;
        trait PrivateContract { type Target; }
        impl PrivateContract for Child { type Target = PrivatePayload; }
        pub fn run() { let _ = &*Child; let _ = &*Generic(1); }
    };
    publicize_public_enum_field_owners(&mut file.items);
    let once = prettyplease::unparse(&file);
    publicize_public_enum_field_owners(&mut file.items);
    assert_eq!(
        once,
        prettyplease::unparse(&file),
        "visibility closure must be idempotent"
    );
    assert_eq!(once.matches("pub struct Parent").count(), 1, "{once}");
    assert!(once.contains("struct Parameter;"), "{once}");
    assert!(!once.contains("pub struct Parameter;"), "{once}");
    assert!(!once.contains("pub struct PrivatePayload;"), "{once}");
    assert!(once.contains("struct Parent;"), "{once}");
    compile_native(&once);
}

#[test]
fn ancestry_visibility_keeps_restricted_audiences_and_external_paths() {
    let mut file: syn::File = syn::parse_quote! {
        mod outer {
            pub(crate) enum Value { Child(inner::Child) }
            mod inner {
                pub(super) struct Child;
                struct Parent;
                impl ::std::ops::Deref for Child {
                    type Target = Parent;
                    fn deref(&self) -> &Parent { &Parent }
                }
            }
            struct External;
            pub enum Generic<External> { Value(External) }
        }
        mod external { struct Value; }
        pub enum Opaque { Value(::external::Value) }
    };
    publicize_public_enum_field_owners(&mut file.items);
    let rust = prettyplease::unparse(&file);
    assert!(rust.contains("pub(crate) struct Child"), "{rust}");
    assert!(rust.contains("pub(crate) struct Parent"), "{rust}");
    assert!(!rust.contains("pub struct External"), "{rust}");
    assert!(!rust.contains("pub struct Value"), "{rust}");
    // The unresolved external crate is intentionally an AST boundary control.
}

fn compile_native(source: &str) {
    let directory = tempfile::Builder::new()
        .prefix("sifr-ancestry-visibility-")
        .tempdir()
        .expect("native inputs");
    let input = directory.path().join("input.rs");
    let output = directory.path().join("output.rlib");
    std::fs::write(&input, source).expect("native source");
    let result =
        std::process::Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .args([
                "--edition=2024",
                "--crate-type=lib",
                "--crate-name=ancestry_visibility",
            ])
            .arg(&input)
            .arg("-o")
            .arg(output)
            .output()
            .expect("rustc");
    std::fs::write(directory.path().join("stderr"), &result.stderr).expect("native diagnostics");
    if !result.status.success() {
        let path = directory.keep();
        panic!(
            "native visibility regression: {}\n{}",
            String::from_utf8_lossy(&result.stderr),
            path.display()
        );
    }
}
