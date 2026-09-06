use super::project_build_check::mktemp_dir;
use crate::{CompileResult, build, build_project, check, compile, emit_project, run_tests};
use sifr_frontend::DiskSourceProvider;
use std::path::Path;

fn emitted(result: CompileResult) -> String {
    match result {
        CompileResult::Success { rust_source } => rust_source,
        CompileResult::Errors { errors } => panic!("required error message must emit: {errors:?}"),
    }
}

fn run(binary: &Path) {
    let output = std::process::Command::new(binary)
        .output()
        .expect("execute native assertions");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "required-error-message-ok"
    );
}

#[test]
fn required_error_message_rejects_invalid_source_before_emission() {
    for source in [
        "class EmptyError(Error):\n    pass\ndef main():\n    _error = EmptyError()\n",
        "class CodeError(Error):\n    code: int\ndef main():\n    _error = CodeError(3)\n",
        "class CodeError(Error):\n    message: int\ndef main():\n    _error = CodeError(3)\n",
        "class CodeError(Error):\n    def __init__(self, code: int):\n        self.code = code\ndef main():\n    _error = CodeError(3)\n",
    ] {
        let errors = check(source);
        assert!(!errors.is_empty(), "source check must reject: {source}");
        assert!(
            matches!(compile(source), CompileResult::Errors { .. }),
            "emission must reject: {source}"
        );
    }
}

#[test]
fn required_error_message_native_default_custom_and_inherited() {
    let source = r#"
class EmptyError(Error):
    pass

class CodeError(Error):
    code: int

class OwnError(Error):
    code: int
    message: str

class CustomError(Error):
    code: int

    def __init__(self, text: str, code: int):
        self.message = text + "!"
        self.code = code

class ChildError(CustomError):
    pass

class LeafError(ChildError):
    pass

class SuperError(Error):
    code: int

    def __init__(self, message: str, code: int):
        super().__init__(message)
        self.code = code

class ImplicitError(Error):
    def __init__(self, message: str):
        pass

class Payload:
    code: int

class MixedError(Payload, Error):
    def __init__(self, message: str, code: int):
        super().__init__(code)
        self.message = message

class MessageData:
    message: str

class MessageMid(MessageData):
    def __init__(self, message: str):
        super().__init__(message)

class TaggedError(MessageMid, Error):
    pass

def specific() -> Result[None, CodeError]:
    raise CodeError("specific", 3)

def unrelated() -> Result[None, Error]:
    raise ValueError("unrelated")

def root(own error: CodeError) -> Result[None, Error]:
    raise error

def inherited() -> Result[None, LeafError]:
    raise LeafError("inherited", 7)

def propagate() -> Result[None, Error]:
    return inherited()

def take(own error: Error) -> str:
    return error.message

def main():
    assert EmptyError("empty").message == "empty"
    assert OwnError(4, "own").message == "own"
    assert SuperError("super", 5).message == "super"
    assert ImplicitError("implicit").message == "implicit"
    assert str(LeafError("display", 7)) == "display!"
    assert take(MixedError("mixed", 8)) == "mixed"
    assert take(TaggedError("nested-data")) == "nested-data"
    assert take(SuperError("super-root", 5)) == "super-root"
    try:
        _specific: None = specific()
        assert False
    except CodeError as error:
        assert error.code == 3
        assert error.message == "specific"
    try:
        _root: None = root(CodeError("root", 9))
        assert False
    except Error as error:
        assert error.message == "root"
    try:
        _propagated: None = propagate()
        assert False
    except Error as error:
        assert error.message == "inherited!"
    print("required-error-message-ok")
"#;
    let rust = emitted(compile(source));
    for name in [
        "CodeError",
        "EmptyError",
        "OwnError",
        "CustomError",
        "ChildError",
        "LeafError",
        "SuperError",
        "MixedError",
    ] {
        assert_eq!(
            rust.matches(&format!("From<{name}> for Error")).count(),
            1,
            "{rust}"
        );
    }
    assert!(
        !rust.contains("new(err.to_string())"),
        "conversion must consume string storage"
    );
    let dir = mktemp_dir("required_error_message_local");
    let binary = build(source, &dir).expect("native local message contract");
    run(&binary);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn required_error_message_native_project_and_test_project_identities() {
    let dir = mktemp_dir("required_error_message_project");
    let files = [
        (
            "left",
            "class CodeError(Error):\n    code: int\nclass Inherited(CodeError):\n    pass\n",
        ),
        ("right", "class CodeError(Error):\n    code: int\n"),
        ("facade", "from left import Inherited as PublicError\n"),
        ("shadow", "class ValueError(Error):\n    pass\n"),
        (
            "csv_child",
            "from sifr.csv import Error\nclass CsvChild(Error):\n    pass\n",
        ),
        ("ordinary", "class Error:\n    code: int\n"),
        (
            "ordinary_child",
            "from ordinary import Error\nclass OrdinaryChild(Error):\n    def __init__(self, code: int):\n        super().__init__(code)\n",
        ),
        (
            "checks",
            r#"
from facade import PublicError as LeftError
from right import CodeError as RightError
from shadow import ValueError as ShadowError
from csv_child import CsvChild
from sifr.configparser import NoSectionError
from ordinary_child import OrdinaryChild

def take(own error: Error) -> str:
    return error.message

def fail() -> Result[None, LeftError]:
    raise LeftError("project", 3)

def propagate() -> Result[None, Error]:
    return fail()

def verify():
    assert OrdinaryChild(17).code == 17
    assert LeftError("left", 1).code == 1
    assert take(LeftError("left", 1)) == "left"
    assert take(RightError("right", 2)) == "right"
    assert take(ShadowError("shadow")) == "shadow"
    assert take(CsvChild("csv")) == "csv"
    assert take(NoSectionError("section")) == "no section: section"
    try:
        _result: None = propagate()
        assert False
    except Error as error:
        assert error.message == "project"
"#,
        ),
        (
            "main",
            "from checks import verify\ndef main():\n    verify()\n    print(\"required-error-message-ok\")\n",
        ),
        (
            "test_messages",
            "from checks import verify\ndef test_required_message():\n    verify()\n",
        ),
    ];
    for (name, source) in files {
        std::fs::write(dir.join(format!("{name}.sifr")), source).expect("write owned test fixture");
    }
    let main = dir.join("main.sifr");
    let rust = emitted(emit_project(&main, &mut DiskSourceProvider::new()));
    for name in [
        "crate::left::CodeError",
        "crate::left::Inherited",
        "crate::right::CodeError",
        "crate::shadow::ValueError",
        "crate::csv_child::CsvChild",
    ] {
        assert_eq!(
            rust.matches(&format!("From<{name}> for Error")).count(),
            1,
            "{rust}"
        );
    }
    let binary = build_project(&main, &dir.join("out"), &mut DiskSourceProvider::new())
        .expect("native imported message contract");
    run(&binary);
    assert!(
        run_tests(&dir, &mut DiskSourceProvider::new())
            .expect("native test-project message contract")
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn required_error_message_imported_constructor_rejects_missing_message() {
    let dir = mktemp_dir("required_error_message_negative_project");
    std::fs::write(
        dir.join("errors.sifr"),
        "class CodeError(Error):\n    code: int\nclass LeafError(CodeError):\n    pass\n",
    )
    .expect("write declarations");
    std::fs::write(
        dir.join("main.sifr"),
        "from errors import LeafError\ndef main():\n    _error = LeafError(3)\n",
    )
    .expect("write invalid call");
    let result = emit_project(&dir.join("main.sifr"), &mut DiskSourceProvider::new());
    assert!(
        matches!(result, CompileResult::Errors { .. }),
        "invalid imported call must fail before Rust"
    );
    let _ = std::fs::remove_dir_all(dir);
}
