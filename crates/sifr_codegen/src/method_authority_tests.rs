use crate::{HirExpr, RustEmitter, RustExpr, render_expr};
use sifr_ir::{BindingId, CallableIdentity, MethodAuthority, MutableReceiverTarget, Place};
use sifr_type_system::{ReceiverConvention, Type};

pub(crate) fn builtin_authority(method: &str) -> MethodAuthority {
    MethodAuthority::BuiltinIntrinsic {
        declaration: identity("sifr.builtin", method),
    }
}

pub(crate) fn contextual_authority(method: &str) -> MethodAuthority {
    MethodAuthority::RustAdapted {
        declaration: identity("fixture.bridge", method),
    }
}

fn identity(module: &str, method: &str) -> CallableIdentity {
    CallableIdentity {
        module: module.to_string(),
        owner: Some("Owner".to_string()),
        symbol: method.to_string(),
        generic_arguments: Vec::new(),
        signature: "() -> None".to_string(),
    }
}

fn call(
    receiver_ty: Type,
    method: &str,
    authority: MethodAuthority,
    args: Vec<HirExpr>,
) -> HirExpr {
    let mutable = method == "append";
    HirExpr::MethodCall {
        object: Box::new(HirExpr::Name {
            name: "value".to_string(),
            binding_id: Some(BindingId(1)),
            ty: receiver_ty,
        }),
        method: method.to_string(),
        mutable_arg_places: vec![None; args.len()],
        args,
        authority,
        receiver_convention: Some(if mutable {
            ReceiverConvention::MutableBorrow
        } else {
            ReceiverConvention::SharedBorrow
        }),
        receiver_target: mutable.then(|| {
            MutableReceiverTarget::Place(Place {
                root: BindingId(1),
                projections: Vec::new(),
            })
        }),
        source: None,
        ty: Type::None,
    }
}

fn builtin(receiver_ty: Type, method: &str, args: Vec<HirExpr>) -> HirExpr {
    call(
        receiver_ty,
        method,
        MethodAuthority::BuiltinIntrinsic {
            declaration: identity("sifr.builtin", method),
        },
        args,
    )
}

#[test]
fn typed_builtin_dispatch_and_strict_decline() {
    let list_ty = Type::List(Box::new(Type::Int));
    let accepted = builtin(list_ty.clone(), "append", vec![HirExpr::IntLiteral(1)]);
    assert_eq!(
        crate::method_call_emitter::source_method_path(&accepted).unwrap(),
        crate::method_call_emitter::SourceMethodPath::BuiltinIntrinsic
    );
    let emitted = RustEmitter::new()
        .lower_stmt_expr_for_ir(&accepted)
        .unwrap()
        .unwrap();
    assert_eq!(render_expr(&emitted), "value.push(SifrInt::from_i64(1))");

    let declined = builtin(list_ty.clone(), "invented", Vec::new());
    let error = RustEmitter::new()
        .lower_stmt_expr_for_ir(&declined)
        .unwrap_err();
    assert!(
        error
            .message
            .contains("builtin method 'invented' is unsupported")
    );
    assert!(
        crate::method_call_emitter::source_method_path(&call(
            list_ty.clone(),
            "append",
            MethodAuthority::Unclassified,
            vec![HirExpr::IntLiteral(1)]
        ))
        .unwrap_err()
        .message
        .contains("unclassified source method")
    );
    assert!(
        crate::method_call_emitter::source_method_path(&call(
            list_ty,
            "append",
            MethodAuthority::BuiltinIntrinsic {
                declaration: identity("other.module", "append"),
            },
            vec![HirExpr::IntLiteral(1)]
        ))
        .is_err()
    );
}

#[test]
fn user_protocol_and_contextual_paths() {
    let nominal_ty = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Owner".to_string(),
        fields: Default::default(),
        methods: Default::default(),
        parent_class: None,
    };
    for authority in [
        MethodAuthority::LocalNominal {
            declaration: identity("user", "append"),
        },
        MethodAuthority::Protocol {
            declaration: identity("user", "append"),
        },
        MethodAuthority::RustAdapted {
            declaration: identity("bridge", "append"),
        },
    ] {
        let expr = call(
            nominal_ty.clone(),
            "append",
            authority.clone(),
            vec![HirExpr::IntLiteral(1)],
        );
        let expected = if matches!(authority, MethodAuthority::RustAdapted { .. }) {
            crate::method_call_emitter::SourceMethodPath::ContextualRust
        } else {
            crate::method_call_emitter::SourceMethodPath::UserProtocol
        };
        assert_eq!(
            crate::method_call_emitter::source_method_path(&expr).unwrap(),
            expected
        );
        let emitted = RustEmitter::new()
            .lower_stmt_expr_for_ir(&expr)
            .unwrap()
            .unwrap();
        assert!(render_expr(&emitted).contains("value.append("));
    }

    let rust_ir_call = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("value".to_string())),
        method: "clone".to_string(),
        args: Vec::new(),
    };
    assert_eq!(render_expr(&rust_ir_call), "value.clone()");
}

#[test]
fn list_append_cloned_decline_regression() {
    let list_ty = Type::List(Box::new(Type::Int));
    let object = RustExpr::Ident("value".to_string());
    assert!(crate::methods::lower_method(&list_ty, "append", &object, &[]).is_none());
    assert!(
        crate::methods::lower_method(
            &list_ty,
            "cloned",
            &object,
            &[RustExpr::Ident("extra".to_string())],
        )
        .is_none()
    );

    let malformed_append = builtin(list_ty.clone(), "append", Vec::new());
    let malformed_cloned = builtin(list_ty.clone(), "cloned", vec![HirExpr::IntLiteral(1)]);
    for expr in [&malformed_append, &malformed_cloned] {
        let error = RustEmitter::new().lower_stmt_expr_for_ir(expr).unwrap_err();
        assert!(error.message.contains("builtin method"));
    }

    let cloned = builtin(list_ty, "cloned", Vec::new());
    let emitted = RustEmitter::new()
        .lower_stmt_expr_for_ir(&cloned)
        .unwrap()
        .unwrap();
    assert_eq!(render_expr(&emitted), "value.clone()");
}
