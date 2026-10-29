use super::{LowerCtx, collect_rust_opaque_close_methods};
use sifr_python_parser::parse_module;

fn prescan(source: &str) -> LowerCtx<'static> {
    let parsed = parse_module(source).expect("source must parse");
    let mut ctx = LowerCtx::new();
    collect_rust_opaque_close_methods(parsed.suite(), &mut ctx);
    ctx
}

#[test]
fn opaque_prescan_gates_capabilities_on_complete_declarations() {
    let incomplete = prescan(
        r#"
@rust.opaque(
    type=bridge.token.Token,
    structural=bridge.token.TokenMapping,
    close="invalid",
)
class Token:
    pass
"#,
    );
    assert!(incomplete.rust_opaque_classes.contains("Token"));
    assert!(incomplete.rust_structural_classes.is_empty());
    assert!(incomplete.rust_consuming_methods.is_empty());
    assert!(incomplete.errors.is_empty());

    let complete = prescan(
        r"
@rust.opaque(
    type=bridge.token.Token,
    structural=bridge.token.TokenMapping,
    close=none,
)
class Token:
    pass
",
    );
    assert!(complete.rust_opaque_classes.contains("Token"));
    assert!(complete.rust_structural_classes.contains("Token"));
    assert!(complete.errors.is_empty());
}
