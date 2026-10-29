use super::*;

#[test]
fn runtime_methods_keep_strict_source_authority_and_checked_receiver_places() {
    let receiver = Type::AsyncGenerator(Box::new(Type::Int), Box::new(Type::Never));
    let mut expression = builtin(receiver.clone(), "aclose", Vec::new());
    if let HirExpr::MethodCall {
        receiver_convention,
        receiver_target,
        ..
    } = &mut expression
    {
        *receiver_convention = Some(ReceiverConvention::MutableBorrow);
        *receiver_target = Some(MutableReceiverTarget::Place(Place {
            root: BindingId(1),
            projections: Vec::new(),
        }));
    }
    let output = RustEmitter::new()
        .lower_stmt_expr_for_ir(&expression)
        .unwrap()
        .unwrap();
    assert_eq!(render_expr(&output), "value.aclose()");
    for authority in [
        MethodAuthority::Unclassified,
        MethodAuthority::BuiltinIntrinsic {
            declaration: identity("forged", "aclose"),
        },
    ] {
        let mut invalid = expression.clone();
        if let HirExpr::MethodCall {
            authority: current, ..
        } = &mut invalid
        {
            *current = authority;
        }
        assert!(RustEmitter::new().lower_stmt_expr_for_ir(&invalid).is_err());
    }
    let mut invalid_place = expression;
    if let HirExpr::MethodCall {
        receiver_target, ..
    } = &mut invalid_place
    {
        *receiver_target = None;
    }
    assert!(
        RustEmitter::new()
            .lower_stmt_expr_for_ir(&invalid_place)
            .is_err()
    );
    for invalid in [
        builtin(Type::Int, "aclose", Vec::new()),
        builtin(receiver, "aclose", vec![HirExpr::IntLiteral(1)]),
    ] {
        assert!(RustEmitter::new().lower_stmt_expr_for_ir(&invalid).is_err());
    }
}
