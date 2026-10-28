use crate::{CompileResult, build_project, emit_project};

use super::project_build_check::mktemp_dir;

#[test]
fn exported_borrowed_value_calls_build_across_modules() {
    let dir = mktemp_dir("borrowed_value_modules");
    let main_file = dir.join("main.sifr");
    let build_out = dir.join("build_out");
    std::fs::write(
        dir.join("helper.sifr"),
        r#"class Unwrappable(Protocol):
    def unwrap(self) -> int: ...

class Carrier:
    value: int

    def __init__(self, value: int):
        self.value = value

    def unwrap(self) -> int:
        return self.value

def through_protocol(own value: Unwrappable) -> int:
    return value.unwrap()

def same_module() -> int:
    return through_protocol(Carrier(2))

def read_optional(value: str | None) -> str:
    if value is None:
        return "none"
    return value.upper()

def forwarded_optional(own value: str | None) -> str:
    return read_optional(value)

def outer(own value: str | None) -> str:
    return forwarded_optional(value)
"#,
    )
    .expect("helper source should be written");
    std::fs::write(
        &main_file,
        r#"from helper import Carrier, through_protocol, same_module, forwarded_optional, outer

def main():
    print(through_protocol(Carrier(3)))
    print(same_module())
    print(forwarded_optional("hello"))
    print(forwarded_optional(None))
    print(outer("chain"))
    print(outer(None))
"#,
    )
    .expect("main source should be written");

    let emitted = emit_project(
        &crate::CompilerContext::for_test(),
        &main_file,
        &mut sifr_frontend::DiskSourceProvider::new(),
    );
    let CompileResult::Success { rust_source } = emitted else {
        panic!("borrowed-value project emit should succeed: {emitted:?}");
    };
    let compact = rust_source
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    assert!(
        compact.contains("through_protocol(value:&dynUnwrappable)"),
        "{rust_source}"
    );
    assert_eq!(
        compact.matches("through_protocol(&").count(),
        2,
        "{rust_source}"
    );
    assert!(
        compact.contains("forwarded_optional(value:&Option<String>)"),
        "{rust_source}"
    );
    assert!(
        compact.contains("forwarded_optional(&Some(")
            && compact.contains("forwarded_optional(&None)"),
        "{rust_source}"
    );
    assert!(
        compact.contains("outer(value:&Option<String>)"),
        "{rust_source}"
    );
    assert!(
        compact.contains("outer(&Some(") && compact.contains("outer(&None)"),
        "{rust_source}"
    );
    assert!(
        compact.contains("forwarded_optional(value)"),
        "{rust_source}"
    );

    let binary = build_project(
        &crate::CompilerContext::for_test(),
        &main_file,
        &build_out,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("cross-module borrowed-value project should build");
    let output = std::process::Command::new(&binary)
        .output()
        .expect("generated project binary should run");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "3\n2\nHELLO\nnone\nCHAIN\nnone\n"
    );
    let _ = std::fs::remove_dir_all(dir);
}
