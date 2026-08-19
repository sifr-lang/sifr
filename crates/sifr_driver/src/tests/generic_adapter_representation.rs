use super::project_build_check::mktemp_dir;
use crate::{build_project, check_project};
use sifr_diagnostics::DiagnosticCode;

#[test]
fn nested_optional_generic_parent_is_rejected_before_rust_codegen() {
    let dir = mktemp_dir("nested_optional_generic_parent");
    let main_file = dir.join("main.sifr");
    std::fs::write(
        &main_file,
        r#"
class Parent[T]:
    value: T | None

class Concrete(Parent[str | None]):
    label: int

    def __init__(self, value: str | None, label: int):
        super().__init__(value)
        self.label = label

def main():
    value: Concrete = Concrete(None, 1)
    assert value.value is None
"#,
    )
    .expect("main module should be written");

    let errors = check_project(&main_file, &mut sifr_frontend::DiskSourceProvider::new());
    assert!(errors.iter().any(|error| {
        error.code == DiagnosticCode::CLASS_INVALID_BASE.code()
            && error.message.contains("union's member topology")
    }));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn transitive_nested_generic_union_parent_is_rejected_before_rust_codegen() {
    let dir = mktemp_dir("transitive_nested_union_generic_parent");
    let main_file = dir.join("main.sifr");
    std::fs::write(
        &main_file,
        r#"
class Inner[U]:
    value: U | None

class Parent[T]:
    inner: Inner[T]

class Concrete(Parent[str | None]):
    def __init__(self, inner: Inner[str | None]):
        super().__init__(inner)

def main():
    pass
"#,
    )
    .expect("main module should be written");

    let errors = check_project(&main_file, &mut sifr_frontend::DiskSourceProvider::new());
    assert!(errors.iter().any(|error| {
        error.code == DiagnosticCode::CLASS_INVALID_BASE.code()
            && error.message.contains("union's member topology")
    }));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn union_bearing_generic_parent_accepts_a_topology_preserving_argument() {
    let dir = mktemp_dir("stable_union_generic_parent");
    let main_file = dir.join("main.sifr");
    let build_out = dir.join("build_out");
    std::fs::write(
        &main_file,
        r#"
class Parent[T]:
    value: T | None

class Concrete(Parent[str]):
    def __init__(self, value: str | None):
        super().__init__(value)

def main():
    value: Concrete = Concrete("ready")
    assert value.value == "ready"
"#,
    )
    .expect("main module should be written");

    let result = build_project(
        &main_file,
        &build_out,
        &mut sifr_frontend::DiskSourceProvider::new(),
    );
    assert!(
        result.is_ok(),
        "stable generic union topology must build: {result:?}"
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn consumer_class_identity_survives_imported_generic_parent_substitution() {
    let dir = mktemp_dir("consumer_generic_parent_identity");
    let main_file = dir.join("main.sifr");
    let build_out = dir.join("build_out");
    std::fs::write(
        dir.join("models.sifr"),
        r#"
class Payload:
    label: str

class Parent[T]:
    value: T
"#,
    )
    .expect("models module should be written");
    std::fs::write(
        &main_file,
        r#"
from models import Parent

class Payload:
    value: int

class Concrete(Parent[Payload]):
    def __init__(self, value: Payload):
        super().__init__(value)

def main():
    value: Concrete = Concrete(Payload(1))
    assert value.value.value == 1
"#,
    )
    .expect("main module should be written");

    let result = build_project(
        &main_file,
        &build_out,
        &mut sifr_frontend::DiskSourceProvider::new(),
    );
    assert!(
        result.is_ok(),
        "consumer class identity must survive substitution: {result:?}"
    );

    let _ = std::fs::remove_dir_all(dir);
}
