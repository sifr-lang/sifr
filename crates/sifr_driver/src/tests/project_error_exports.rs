use super::project_build_check::mktemp_dir;
use crate::{build_project, check_project};
use sifr_diagnostics::DiagnosticCode;

fn write_aliased_error_project(dir: &std::path::Path) {
    std::fs::write(
        dir.join("errors.sifr"),
        "class PackageError(Error):\n    message: str\n",
    )
    .expect("error module should be written");
    std::fs::write(
        dir.join("api.sifr"),
        "from errors import PackageError as ApiError\n",
    )
    .expect("facade module should be written");
    std::fs::write(
        dir.join("main.sifr"),
        "from api import ApiError as PublicError\n\ndef fail() -> Result[int, PublicError]:\n    raise PublicError(\"failed\")\n\ndef main() -> Result[None, PublicError]:\n    try:\n        value: int = fail()\n        assert value == 1\n    except PublicError as error:\n        assert error.message == \"failed\"\n        return None\n    return None\n",
    )
    .expect("main module should be written");
}

#[test]
fn test_check_project_preserves_aliased_error_status_through_facade() {
    let dir = mktemp_dir("aliased_error_facade_check");
    write_aliased_error_project(&dir);

    let errors = check_project(&dir.join("main.sifr"));

    assert!(errors.is_empty(), "aliased error should check: {errors:?}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_project_preserves_aliased_error_status_through_facade() {
    let dir = mktemp_dir("aliased_error_facade_native");
    write_aliased_error_project(&dir);

    let binary = build_project(&dir.join("main.sifr"), &dir.join("build_out"))
        .expect("aliased error project should build natively");
    let status = std::process::Command::new(binary)
        .status()
        .expect("aliased error binary should run");

    assert!(status.success());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_check_project_does_not_leak_error_status_between_modules() {
    let dir = mktemp_dir("module_scoped_error_status");
    std::fs::write(
        dir.join("errors.sifr"),
        "class Record(Error):\n    message: str\n",
    )
    .expect("error module should be written");
    std::fs::write(dir.join("data.sifr"), "class Record:\n    value: int\n")
        .expect("data module should be written");
    std::fs::write(
        dir.join("main.sifr"),
        "from errors import Record as PackageError\nfrom data import Record\n\ndef invalid() -> Result[int, Record]:\n    return 1\n\ndef main() -> None:\n    error: PackageError = PackageError(\"failed\")\n    assert error.message == \"failed\"\n",
    )
    .expect("main module should be written");

    let errors = check_project(&dir.join("main.sifr"));

    assert!(
        errors
            .iter()
            .any(|error| error.code == DiagnosticCode::RESULT_INVALID_ERROR_TYPE.code()),
        "same-name non-error class must remain invalid: {errors:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}
