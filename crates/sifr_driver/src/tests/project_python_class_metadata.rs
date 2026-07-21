use super::project_build_check::mktemp_dir;
use crate::check_project;
use sifr_diagnostics::DiagnosticCode;

#[test]
fn test_check_project_preserves_imported_affine_class_call_metadata() {
    let dir = mktemp_dir("imported_affine_class_metadata");
    std::fs::write(
        dir.join("main.sifr"),
        r#"
from helper import Holder, Tools, Wrap

def misuse_arrow(own value: python.ArrowArray) -> None:
    first: Holder = Holder(value)
    second: Holder = Holder(value)

def misuse_buffer(own value: python.Buffer[uint8]) -> None:
    first: Wrap = Wrap(value)
    second: Wrap = Wrap(value)

def misuse_class_style(own value: python.ArrowArray) -> None:
    Holder.consume(value)

def valid_class_style() -> int:
    return Tools.identity(1) + Tools.class_identity(2)

def main() -> None:
    return None
"#,
    )
    .expect("main module should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        r#"
class Holder:
    value: python.ArrowArray

    def __init__(self, own value: python.ArrowArray):
        self.value = value

    def consume(self, own value: python.ArrowArray) -> None:
        return None

class Wrap:
    value: python.Buffer[uint8]

class Tools:
    @staticmethod
    def identity(value: int) -> int:
        return value

    @classmethod
    def class_identity(cls, value: int) -> int:
        return value
"#,
    )
    .expect("helper module should be written");

    let errors = check_project(&dir.join("main.sifr"));
    assert!(
        errors
            .iter()
            .filter(|error| error.code == DiagnosticCode::OWN_USE_AFTER_MOVE.code())
            .count()
            >= 2,
        "{errors:?}"
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.code == DiagnosticCode::CLASS_MISSING_MEMBER.code())
            .count(),
        1,
        "{errors:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}
