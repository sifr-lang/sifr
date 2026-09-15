use super::project_build_check::mktemp_dir;
use crate::{CompileResult, build, build_project, compile, emit_project};
use std::path::Path;

fn emitted(result: CompileResult) -> String {
    match result {
        CompileResult::Success { rust_source } => rust_source,
        CompileResult::Errors { errors } => panic!("error conversion must emit: {errors:?}"),
    }
}

fn run(binary: &Path) {
    let output = std::process::Command::new(binary)
        .output()
        .expect("run native regression");
    assert!(output.status.success(), "native regression: {output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "error-conversions-ok"
    );
}

#[test]
fn async_python_error_channel_native_local_and_transitive_conversions() {
    let source = r#"
class DomainError(Error):
    message: str

class DetailedError(DomainError):
    detail: str

    def __init__(self, message: str, detail: str):
        super().__init__(message)
        self.detail = detail

def direct(own error: DomainError) -> Result[None, Error]:
    raise error

def inherited() -> Result[None, DetailedError]:
    raise DetailedError("inherited", "detail")

def propagate() -> Result[None, Error]:
    return inherited()

def main():
    try:
        _direct: None = direct(DomainError("direct"))
        assert False
    except Error as error:
        assert error.message == "direct"
    try:
        _propagated: None = propagate()
        assert False
    except Error as error:
        assert error.message == "inherited"
    print("error-conversions-ok")
"#;
    let rust = emitted(compile(source));
    assert!(rust.contains("From<DomainError> for Error"), "{rust}");
    assert!(rust.contains("From<DetailedError> for Error"), "{rust}");
    let dir = mktemp_dir("error_channel_local_native");
    let binary = build(source, &dir).expect("local/transitive errors must compile natively");
    run(&binary);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn async_python_error_channel_native_stdlib_nominal_collisions() {
    let source = r#"
from sifr.csv import Error as CsvError
from sifr.configparser import Error as ConfigError

def csv(own error: CsvError) -> Result[None, Error]:
    raise error

def config(own error: ConfigError) -> Result[None, Error]:
    raise error

def main():
    try:
        _csv: None = csv(CsvError("csv"))
        assert False
    except Error as error:
        assert error.message == "csv"
    try:
        _config: None = config(ConfigError("config"))
        assert False
    except Error as error:
        assert error.message == "config"
    print("error-conversions-ok")
"#;
    let rust = emitted(compile(source));
    for identity in ["sifr.csv.Error", "sifr.configparser.Error"] {
        let name = sifr_type_system::class_rust_name(Some(identity), "Error");
        assert_eq!(
            rust.matches(&format!("From<{name}> for Error")).count(),
            1,
            "{rust}"
        );
    }
    let dir = mktemp_dir("error_channel_stdlib_native");
    let binary = build(source, &dir).expect("distinct stdlib errors must compile natively");
    run(&binary);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn async_python_error_channel_native_project_aliases_and_collisions() {
    let dir = mktemp_dir("error_channel_project_native");
    for (name, source) in [
        (
            "left",
            "class DomainError(Error):\n    message: str\n\nclass DetailedError(DomainError):\n    detail: str\n\n    def __init__(self, message: str, detail: str):\n        super().__init__(message)\n        self.detail = detail\n",
        ),
        ("right", "class DomainError(Error):\n    message: str\n"),
        ("api", "from left import DomainError as PublicError\n"),
        ("shadow", "class ValueError(Error):\n    message: str\n"),
        (
            "main",
            r#"
from api import PublicError as LeftError
from left import DetailedError as LeftDetailedError
from right import DomainError as RightError
from shadow import ValueError as ShadowError

def left(own error: LeftError) -> Result[None, Error]:
    raise error

def right(own error: RightError) -> Result[None, Error]:
    raise error

def detailed(own error: LeftDetailedError) -> Result[None, Error]:
    raise error

def shadow(own error: ShadowError) -> Result[None, Error]:
    raise error

def main():
    try:
        _left: None = left(LeftError("left"))
        assert False
    except Error as error:
        assert error.message == "left"
    try:
        _right: None = right(RightError("right"))
        assert False
    except Error as error:
        assert error.message == "right"
    try:
        _detailed: None = detailed(LeftDetailedError("detailed", "detail"))
        assert False
    except Error as error:
        assert error.message == "detailed"
    try:
        _shadow: None = shadow(ShadowError("shadow"))
        assert False
    except Error as error:
        assert error.message == "shadow"
    print("error-conversions-ok")
"#,
        ),
    ] {
        std::fs::write(dir.join(format!("{name}.sifr")), source).expect("write source");
    }
    let main = dir.join("main.sifr");
    let rust = emitted(emit_project(
        &main,
        &mut sifr_frontend::DiskSourceProvider::new(),
    ));
    for name in [
        "crate::left::DomainError",
        "crate::left::DetailedError",
        "crate::right::DomainError",
        "crate::shadow::ValueError",
    ] {
        assert_eq!(
            rust.matches(&format!("From<{name}> for Error")).count(),
            1,
            "{rust}"
        );
    }
    let binary = build_project(
        &main,
        &dir.join("out"),
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("project error identities must compile natively");
    run(&binary);
    let _ = std::fs::remove_dir_all(dir);
}
