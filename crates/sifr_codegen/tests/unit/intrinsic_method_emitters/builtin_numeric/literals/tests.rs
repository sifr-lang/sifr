#[test]
fn bigdecimal_literal_uses_a_borrowed_coefficient_array() {
    let expression =
        super::bigdecimal_literal_expr(&super::HirExpr::StringLiteral("-1.9".to_owned()))
            .expect("exact decimal literal");
    let rendered = crate::render_expr(&expression);
    assert!(
        rendered.contains("from_signed_bytes_be(&[237])"),
        "{rendered}"
    );
    assert!(!rendered.contains("vec!"), "{rendered}");
}
