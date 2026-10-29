use super::*;

fn wrapped(inner: Type) -> Type {
    Type::Newtype {
        identity: Some("main.__SifrIoFileHandle".to_string()),
        name: "__SifrIoFileHandle".to_string(),
        inner: Box::new(inner),
    }
}

fn enumeration() -> Type {
    Type::Enum {
        identity: Some("main.__SifrIoTextFileHandle".to_string()),
        name: "__SifrIoTextFileHandle".to_string(),
        variants: vec![("READY".to_string(), Some(1))].into(),
    }
}

#[test]
fn nominal_accessors_use_typed_registry_for_direct_and_alias_receivers() {
    for (receiver, method, result) in [
        (wrapped(Type::Int), "value", Type::Int),
        (wrapped(Type::Str), "value", Type::Str),
        (enumeration(), "value", Type::Int),
        (enumeration(), "name", Type::Str),
    ] {
        for receiver in [
            receiver.clone(),
            Type::Alias {
                name: "AccessorOwner".to_string(),
                type_args: Vec::new(),
                body: Box::new(receiver),
            },
        ] {
            let mut expression = builtin(receiver, method, Vec::new());
            if let HirExpr::MethodCall { ty, .. } = &mut expression {
                *ty = result.clone();
            }
            let emitted = RustEmitter::new()
                .lower_stmt_expr_for_ir(&expression)
                .expect("typed accessor should lower")
                .expect("typed accessor should produce an expression");
            assert_eq!(render_expr(&emitted), format!("value.{method}()"));
        }
    }
}

#[test]
fn nominal_accessor_registry_rejects_wrong_receiver_method_and_arity() {
    for expression in [
        builtin(Type::Int, "value", Vec::new()),
        builtin(Type::Str, "name", Vec::new()),
        builtin(wrapped(Type::Int), "name", Vec::new()),
        builtin(enumeration(), "invented", Vec::new()),
        builtin(wrapped(Type::Int), "value", vec![HirExpr::IntLiteral(1)]),
        builtin(enumeration(), "name", vec![HirExpr::IntLiteral(1)]),
        builtin(enumeration(), "value", vec![HirExpr::IntLiteral(1)]),
    ] {
        let error = RustEmitter::new()
            .lower_stmt_expr_for_ir(&expression)
            .expect_err("unsupported accessors must not fall back to method-name dispatch");
        assert!(
            error
                .message
                .contains("is unsupported on resolved receiver type")
        );
    }
}

#[test]
fn nominal_accessors_require_valid_declared_authority() {
    for (receiver, method) in [(wrapped(Type::Int), "value"), (enumeration(), "name")] {
        for authority in [
            MethodAuthority::Unclassified,
            MethodAuthority::BuiltinIntrinsic {
                declaration: identity("foreign.builtin", method),
            },
            MethodAuthority::BuiltinIntrinsic {
                declaration: identity("sifr.builtin", "different_method"),
            },
        ] {
            let expression = call(receiver.clone(), method, authority, Vec::new());
            assert!(
                RustEmitter::new()
                    .lower_stmt_expr_for_ir(&expression)
                    .is_err()
            );
        }
    }
}
