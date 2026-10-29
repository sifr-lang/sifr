use super::*;

#[test]
fn result_callback_adapter_maps_values_and_display_errors() {
    let error = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "EventError".to_string(),
        fields: vec![("message".to_string(), Type::Str)].into(),
        methods: Vec::new().into(),
        parent_class: Some("Error".to_string()),
    };
    let param = HirParam {
        name: "handler".to_string(),
        ty: Type::Callable(
            vec![Type::Str],
            vec![ParamConvention::borrow()],
            Box::new(Type::Result(Box::new(Type::None), Box::new(error))),
        ),
        default: None,
        keyword_only: false,
        convention: ParamConvention::borrow(),
    };

    let rendered = render_expr(&call_scoped_callback_adapter_expr(&param));

    assert!(rendered.contains("CallScopedCallbackBridge::new"));
    assert!(
        rendered.contains("handler(::std::string::String::as_str(&__sifr_callback_arg_0))")
    );
    assert!(rendered.contains("Ok(__sifr_callback_ok) => Ok(())"));
    assert!(rendered.contains("__sifr_callback_error.to_string()"));
}
