use super::*;

fn sample_param(name: &str, ty: Type, convention: ParamConvention) -> HirParam {
    HirParam {
        name: name.to_string(),
        ty,
        default: None,
        keyword_only: false,
        convention,
    }
}

#[test]
fn function_type_from_params_preserves_named_conventions() {
    let params = vec![
        sample_param("value", Type::Int, ParamConvention::borrow()),
        sample_param("count", Type::Int, ParamConvention::own()),
    ];

    let function_type = function_type_from_params(&params, &Type::Bool);

    assert_eq!(
        function_type,
        FunctionType {
            params: vec![
                ("value".to_string(), Type::Int, ParamConvention::borrow()),
                ("count".to_string(), Type::Int, ParamConvention::own()),
            ],
            return_type: Box::new(Type::Bool),
        }
    );
}

#[test]
fn signature_params_can_override_constructor_conventions() {
    let params = vec![sample_param("self", Type::Str, ParamConvention::borrow())];

    assert_eq!(
        signature_params(&params, Some(ParamConvention::own())),
        vec![(Type::Str, ParamConvention::own())]
    );
}

#[test]
fn public_constant_integer_value_exports_filter_to_public_recorded_values() {
    let mut values = HashMap::new();
    values.insert("ANSWER".to_string(), 42);
    values.insert("_PRIVATE".to_string(), 99);
    values.insert("STALE".to_string(), 100);

    let exports =
        collect_public_constant_integer_value_exports(["ANSWER", "MISSING"].into_iter(), &values);

    assert_eq!(exports.len(), 1);
    assert_eq!(exports.get("ANSWER"), Some(&42));
    assert!(!exports.contains_key("_PRIVATE"));
    assert!(!exports.contains_key("STALE"));
}

#[test]
fn stdlib_class_exports_preserve_parent_markers() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let object_ty = compiled
        .defs
        .classes
        .get("sifr.python")
        .and_then(|classes| classes.get("Object"))
        .expect("sifr.python.Object should be exported");

    assert!(matches!(
        object_ty,
        Type::Class {
            parent_class: Some(parent),
            ..
        } if parent.split('|').any(|name| name == "NonSend")
    ));
}

#[test]
fn python_core_re_exports_preserve_callable_metadata() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let workloads = compiled
        .defs
        .function_workloads
        .get("sifr.python")
        .expect("sifr.python should export workload metadata");

    assert_eq!(
        workloads.get("threadsafe_callback_echo"),
        Some(&"blocking_io".to_string())
    );
    assert_eq!(
        workloads.get("close_local_callback"),
        Some(&"blocking_io".to_string())
    );

    let defaults = compiled
        .defs
        .function_defaults
        .get("sifr.python")
        .and_then(|module_defaults| module_defaults.get("PythonError"))
        .expect("PythonError constructor defaults should be re-exported");
    assert_eq!(defaults.len(), 4);
}
