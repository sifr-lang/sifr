use super::*;
use sifr_type_system::{ParamConvention, Type};

fn signature(return_type: Type) -> FuncSignature {
    (vec![(Type::Int, ParamConvention::borrow())], return_type)
}

fn private_import(module: &str, alias: &str) -> HirImport {
    HirImport {
        module: module.to_string(),
        names: vec!["shared_name".to_string()],
        aliases: vec![("shared_name".to_string(), alias.to_string())],
    }
}

#[test]
fn imported_metadata_preserves_aliases_methods_fields_generators_and_constants() {
    let parsed = sifr_python_parser::parse_module("class Container[T]:\n    value: T\n")
        .expect("parse metadata fixture");
    let mut module = sifr_lowering::lower_module(parsed.suite())
        .expect("lower metadata fixture")
        .module;
    let template = std::sync::Arc::new(module.classes.remove(0));
    module.imports.push(HirImport {
        module: "sifr.fixture".into(),
        names: vec![
            "consume".into(),
            "Container".into(),
            "items".into(),
            "LIMIT".into(),
        ],
        aliases: vec![
            ("consume".into(), "borrowed".into()),
            ("Container".into(), "Alias".into()),
            ("LIMIT".into(), "BOUND".into()),
        ],
    });
    let mut code = crate::StdlibCode::default();
    code.func_signatures.insert(
        "sifr.fixture".into(),
        [
            ("consume".into(), signature(Type::Bool)),
            ("Container::read".into(), signature(Type::Int)),
            ("Returned::read".into(), signature(Type::Str)),
        ]
        .into(),
    );
    code.module_class_fields.insert(
        "sifr.fixture".into(),
        [("Container".into(), vec![("value".into(), Type::Int)])].into(),
    );
    code.generic_classes.insert("Container".into());
    code.generic_class_params
        .insert("Container".into(), vec!["T".into()]);
    code.generic_class_templates
        .insert("Container".into(), template.clone());
    code.generator_functions
        .insert("sifr.fixture".into(), ["items".into()].into());
    code.module_constants.insert(
        "sifr.fixture".into(),
        [("LIMIT".into(), (Type::Int, "crate::LIMIT".into()))].into(),
    );

    let mut emitter = RustEmitter::new();
    register_imported_stdlib_metadata(&mut emitter, &module, &code);
    assert_eq!(emitter.func_signatures["borrowed"], signature(Type::Bool));
    assert_eq!(emitter.func_signatures["Alias::read"], signature(Type::Int));
    assert_eq!(
        emitter.func_signatures["Returned::read"],
        signature(Type::Str)
    );
    assert_eq!(emitter.class_field_order["Alias"], vec!["value"]);
    assert_eq!(
        emitter.class_field_types[&("Alias".into(), "value".into())],
        Type::Int
    );
    assert!(emitter.generic_classes.contains("Container"));
    assert_eq!(emitter.generic_class_params["Container"], vec!["T"]);
    assert!(std::sync::Arc::ptr_eq(
        &emitter.generic_class_templates["Container"],
        &template
    ));
    assert!(emitter.generator_functions.contains("items"));
    assert_eq!(
        emitter.module_constants["BOUND"],
        (Type::Int, "crate::LIMIT".into())
    );
}

#[test]
fn private_alias_registration_preserves_local_signatures_without_overwriting_origin() {
    let mut code = crate::StdlibCode::default();
    code.func_signatures.insert(
        "_sifr.first".to_string(),
        [("shared_name".to_string(), signature(Type::Int))].into(),
    );
    code.func_signatures.insert(
        "_sifr.second".to_string(),
        [("shared_name".to_string(), signature(Type::Str))].into(),
    );
    let mut emitter = RustEmitter::new();

    register_imported_stdlib_signature(
        &mut emitter,
        &code,
        &private_import("_sifr.first", "_first_impl"),
        "shared_name",
    );
    register_imported_stdlib_signature(
        &mut emitter,
        &code,
        &private_import("_sifr.second", "_second_impl"),
        "shared_name",
    );

    assert_eq!(emitter.func_signatures["_first_impl"].1, Type::Int);
    assert_eq!(emitter.func_signatures["_second_impl"].1, Type::Str);
    assert_eq!(emitter.func_signatures["shared_name"].1, Type::Int);
}
