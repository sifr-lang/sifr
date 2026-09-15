use super::{canonicalize_generated_rust_project_with_names, canonicalize_generated_rust_source};
use std::collections::BTreeMap;

#[test]
fn external_absolute_paths_preserve_symbols_but_rewrite_local_generic_arguments() {
    let source = r#"
        pub struct __Payload;
        pub fn __declaration_object_result(value: __Payload) -> __Payload {
            ::sifr_runtime::python::__declaration_object_result::<__Payload>(value)
        }
    "#;
    let output = canonicalize_generated_rust_source(source).expect("owned and external identities");
    assert!(
        output.contains("fn sifr_generated_declaration_object_result"),
        "{output}"
    );
    assert!(
        output.split_whitespace().collect::<String>().contains(
            "::sifr_runtime::python::__declaration_object_result::<SifrGeneratedPayload>"
        ),
        "{output}"
    );
    assert!(!output.contains("python::sifr_generated_declaration_object_result"));
}

#[test]
fn external_imports_keep_the_original_symbol_and_introduce_canonical_local_aliases() {
    let source = r#"
        use ::external::__module::{__read, __write as __local_write};
        pub fn run() { __read(); __local_write(); }
    "#;
    let output = canonicalize_generated_rust_source(source).expect("external imports");
    assert!(output.contains("__read as sifr_generated_read"), "{output}");
    assert!(
        output.contains("__write as sifr_generated_local_write"),
        "{output}"
    );
    assert!(output.contains("::external::__module::"), "{output}");
    assert!(output.contains("sifr_generated_read();"), "{output}");
    assert!(output.contains("sifr_generated_local_write();"), "{output}");
}

#[test]
fn parsed_and_unparsed_macro_arguments_preserve_external_paths() {
    let source = r#"
        pub fn run(_local: i64) {
            consume!(::external::__read(_local));
            let values = vec![::external::__read(_local); 3];
            consume!(values);
        }
    "#;
    let output = canonicalize_generated_rust_source(source).expect("macro path ownership");
    let output = output.split_whitespace().collect::<String>();
    assert_eq!(output.matches("::external::__read").count(), 2, "{output}");
    assert_eq!(
        output
            .matches("::external::__read(sifr_generated_local)")
            .count(),
        2,
        "{output}"
    );
}

#[test]
fn physical_module_segments_share_the_injective_project_spelling_map() {
    let sources = BTreeMap::from([
        (
            String::new(),
            "pub mod __sifr_bridge; pub fn __sifr_encoding() {}".to_string(),
        ),
        (
            "__sifr_bridge".to_string(),
            "pub mod _sifr_encoding; pub mod sifr_generated_encoding;".to_string(),
        ),
        (
            "__sifr_bridge::_sifr_encoding".to_string(),
            "pub struct Encoding;".to_string(),
        ),
        (
            "__sifr_bridge::sifr_generated_encoding".to_string(),
            "pub struct UserEncoding;".to_string(),
        ),
    ]);
    let (output, names) =
        canonicalize_generated_rust_project_with_names(&sources).expect("complete name map");
    let generated = &names["_sifr_encoding"];
    let user = &names["sifr_generated_encoding"];
    assert_ne!(generated, user);
    assert_ne!(generated, &names["__sifr_encoding"]);
    assert!(output["__sifr_bridge"].contains(&format!("pub mod {generated};")));
    assert!(output["__sifr_bridge"].contains(&format!("pub mod {user};")));
    assert!(output[""].contains(&format!("pub mod {};", names["__sifr_bridge"])));
}

#[test]
fn module_keys_participate_even_before_synthetic_namespace_declarations_exist() {
    let sources = BTreeMap::from([
        (String::new(), "pub fn __owner() {}".to_string()),
        (
            "_owner::nested".to_string(),
            "pub struct Value;".to_string(),
        ),
    ]);
    let (_, names) =
        canonicalize_generated_rust_project_with_names(&sources).expect("implicit namespace names");
    assert_ne!(names["__owner"], names["_owner"]);
}
