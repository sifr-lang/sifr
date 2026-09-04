use super::*;

#[test]
fn item10a_single_file_user_error_shadow_keeps_one_local_identity() {
    let generated = generate_rust_from_source(
        r#"
class ValueError(Error):
    message: str
    detail: str

    def __init__(self, message: str, detail: str):
        self.message = message
        self.detail = detail

def message() -> str:
    try:
        raise ValueError("local", "single-file-detail")
    except ValueError as error:
        return error.detail
"#,
    );

    assert_eq!(
        generated.matches("struct ValueError").count(),
        1,
        "{generated}"
    );
    assert!(generated.contains("detail: String"), "{generated}");
    assert!(generated.contains("\"single-file-detail\".to_string()"));
}
