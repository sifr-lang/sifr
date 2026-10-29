use super::*;

#[test]
fn borrowed_views_and_owned_materialization_are_structural() {
    assert_eq!(
        shared_borrowed_param_type(&Type::Str, RustType::String_),
        RustType::Ref {
            mutable: false,
            inner: Box::new(RustType::Str),
        }
    );
    assert_eq!(
        shared_borrowed_param_type(
            &Type::List(Box::new(Type::Bool)),
            RustType::Vec(Box::new(RustType::Bool)),
        ),
        RustType::Ref {
            mutable: false,
            inner: Box::new(RustType::Slice(Box::new(RustType::Bool))),
        }
    );

    let string = materialize_owned_value(&Type::Str, RustExpr::Ident("value".to_string()));
    assert!(matches!(
        string,
        RustExpr::MethodCall { method, .. } if method == "to_owned"
    ));
    let list = materialize_owned_value(
        &Type::List(Box::new(Type::Bool)),
        RustExpr::Ident("values".to_string()),
    );
    assert!(matches!(
        list,
        RustExpr::MethodCall { method, .. } if method == "to_vec"
    ));

    let once = materialize_owned_value(&Type::Str, RustExpr::Ident("value".to_string()));
    assert_eq!(materialize_owned_value(&Type::Str, once.clone()), once);
    assert_eq!(
        materialize_owned_value(
            &Type::Str,
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("value".to_string())),
                method: "clone".to_string(),
                args: Vec::new(),
            },
        ),
        once,
    );
}

#[test]
fn clone_plan_never_compounds_an_existing_clone() {
    let once = RustExpr::Clone(Box::new(RustExpr::Ident("value".to_string())));
    assert_eq!(clone_once(once.clone()), once);
}
