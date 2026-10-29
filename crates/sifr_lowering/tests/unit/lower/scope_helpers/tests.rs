
use super::LowerCtx;
use sifr_type_system::Type;

#[test]
fn with_pushed_scope_restores_scope_after_none() {
    let mut ctx = LowerCtx::new();
    assert_eq!(ctx.scope.frame_count(), 1);

    let result: Option<()> = ctx.with_pushed_scope(|ctx| {
        ctx.scope.define("temp".to_string(), Type::Int);
        None
    });

    assert!(result.is_none());
    assert_eq!(ctx.scope.frame_count(), 1);
    assert!(ctx.scope.lookup("temp").is_none());
}
