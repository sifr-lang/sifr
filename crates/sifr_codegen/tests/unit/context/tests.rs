use super::*;

#[test]
fn codegen_error_formats_with_location() {
    let err = CodegenError::with_location("unsupported node", Some(10), Some(3));
    assert_eq!(err.to_string(), "unsupported node (at 10:3)");
}
