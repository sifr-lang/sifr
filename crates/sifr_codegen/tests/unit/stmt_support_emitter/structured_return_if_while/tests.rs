use super::*;

#[test]
fn result_none_bare_return_uses_unit_inside_control_flow_carrier() {
    let mut emitter = RustEmitter::new();
    emitter
        .try_closure_return_wrap
        .push(crate::TryClosureReturnWrap::ControlFlow {
            continue_type: "(SifrInt,)".to_string(),
        });
    let return_ty = Type::Result(Box::new(Type::None), Box::new(Type::Str));

    let lowered = emitter.try_closure_unit_return_for_ir(Some(&return_ty));

    assert_eq!(
        crate::render_expr(&lowered),
        "Ok(::std::ops::ControlFlow::Break(Ok(())))"
    );
}
