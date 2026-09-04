use super::*;

#[test]
fn item10a_single_file_user_error_shadow_keeps_one_local_identity() {
    let generated = generate_rust_from_source(
        r#"
class ValueError(Error):
    message: str

def message() -> str:
    try:
        raise ValueError("local")
    except ValueError as error:
        return error.message
"#,
    );

    assert_eq!(
        generated.matches("struct ValueError").count(),
        1,
        "{generated}"
    );
    assert!(generated.contains("Err(ValueError::new(\"local\".to_string()))"));
}
