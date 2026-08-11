use crate::{lower_module_with_externals, ExternalDefs, HirDiagnostic, HirModule};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::RustInteropDecoratorKind;
use sifr_python_parser::parse_module;
use sifr_type_system::Type;
use std::collections::HashMap;

fn structural_externals() -> ExternalDefs {
    let mut externals = ExternalDefs::default();
    externals.classes.insert(
        "sifr.meta".to_string(),
        HashMap::from([
            (
                "Structural".to_string(),
                Type::Class {
                    identity: Some("sifr.meta.Structural".to_string()),
                    type_args: Vec::new(),
                    name: "Structural".to_string(),
                    fields: Vec::new(),
                    methods: Vec::new(),
                    parent_class: None,
                },
            ),
            (
                "StaticProgram".to_string(),
                Type::Class {
                    identity: Some("sifr.meta.StaticProgram".to_string()),
                    type_args: Vec::new(),
                    name: "StaticProgram".to_string(),
                    fields: Vec::new(),
                    methods: Vec::new(),
                    parent_class: None,
                },
            ),
        ]),
    );
    externals
}

#[test]
fn rust_interop_accepts_compiler_owned_static_program_bound() {
    let source = r"
from sifr.meta import StaticProgram

class CodecError(Error):
    message: str

@rust.structural
@rust(bridge.codec.decode)
def decode[T: StaticProgram]() -> Result[T, CodecError | RustPanicError]: ...
";
    let parsed = parse_module(source).expect("source should parse");
    let module = lower_module_with_externals(parsed.suite(), &structural_externals())
        .map(|result| result.module)
        .expect("static program bound should lower");
    assert_eq!(module.type_param_bounds["decode"]["T"], ["StaticProgram"]);
}

#[test]
fn static_program_bound_requires_a_structural_specialization_owner() {
    let eligible = r#"
from sifr.meta import StaticProgram

@const_specialize("package.schema", "derive")
class Record:
    value: str

def retain[T: StaticProgram](value: T) -> T:
    return value

def use() -> Record:
    return retain(Record("ok"))
"#;
    let parsed = parse_module(eligible).expect("source should parse");
    lower_module_with_externals(parsed.suite(), &structural_externals())
        .expect("specialized structural class should satisfy StaticProgram");

    let ineligible = eligible.replace("@const_specialize(\"package.schema\", \"derive\")\n", "");
    let parsed = parse_module(&ineligible).expect("source should parse");
    let errors = match lower_module_with_externals(parsed.suite(), &structural_externals()) {
        Ok(_) => panic!("ordinary class must not receive a static-program fallback"),
        Err(errors) => errors,
    };
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
            && error
                .message
                .contains("does not implement protocol 'StaticProgram'")
    }));
}

fn lower_ok(source: &str) -> HirModule {
    let source = format!("from sifr.meta import Structural\n{source}");
    let parsed = parse_module(&source).expect("source should parse");
    lower_module_with_externals(parsed.suite(), &structural_externals())
        .map(|result| result.module)
        .expect("source should lower")
}

fn lower_errors(source: &str) -> Vec<HirDiagnostic> {
    let source = format!("from sifr.meta import Structural\n{source}");
    lower_errors_without_marker_import(&source)
}

fn lower_errors_without_marker_import(source: &str) -> Vec<HirDiagnostic> {
    let parsed = parse_module(source).expect("source should parse");
    match lower_module_with_externals(parsed.suite(), &structural_externals()) {
        Ok(_) => panic!("source should fail lowering"),
        Err(errors) => errors,
    }
}

fn assert_malformed(errors: &[HirDiagnostic]) {
    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR)));
}

#[test]
fn rust_interop_accepts_bare_structural_marker_with_exact_bound_and_target() {
    let module = lower_ok(
        r"
class CodecError(Error):
    message: str

@rust.structural
@rust(bridge.codec.encode)
def encode[T: Structural](value: T) -> Result[bytes, CodecError | RustPanicError]: ...
",
    );

    let function = &module.functions[0];
    assert!(function.rust_interop.iter().any(|declaration| {
        declaration.kind == RustInteropDecoratorKind::Structural
            && declaration.target.is_none()
            && declaration.arguments.is_empty()
    }));
}

#[test]
fn rust_interop_rejects_incomplete_structural_error_and_panic_contracts() {
    let panic_only = lower_errors(
        r"
@rust.structural
@rust(bridge.codec.encode)
def encode[T: Structural](value: T) -> Result[bytes, RustPanicError]: ...
",
    );
    assert!(panic_only.iter().any(|error| {
        error.code == Some(DiagnosticCode::RUST_TYPE_PROBE_FAILURE)
            && error.message.contains("ordinary error")
    }));

    let ordinary_only = lower_errors(
        r"
class CodecError(Error):
    message: str

@rust.structural
@rust(bridge.codec.encode)
def encode[T: Structural](value: T) -> Result[bytes, CodecError]: ...
",
    );
    assert!(ordinary_only.iter().any(|error| {
        error.code == Some(DiagnosticCode::RUST_TYPE_PROBE_FAILURE)
            && error.message.contains("RustPanicError")
    }));

    let non_result = lower_errors(
        r"
@rust.structural
@rust(bridge.codec.encode)
def encode[T: Structural](value: T) -> bytes: ...
",
    );
    assert!(non_result.iter().any(|error| {
        error.code == Some(DiagnosticCode::RUST_TYPE_PROBE_FAILURE)
            && error.message.contains("must return `Result")
    }));
}

#[test]
fn rust_interop_rejects_nonrecoverable_structural_panic_policies() {
    for policy in ["trusted_no_panic", "abort"] {
        let source = format!(
            r"
class CodecError(Error):
    message: str

@rust.structural
@rust(bridge.codec.encode, panic={policy})
def encode[T: Structural](value: T) -> Result[bytes, CodecError | RustPanicError]: ...
"
        );
        let errors = lower_errors(&source);
        assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::RUST_TYPE_PROBE_FAILURE)
                && error.message.contains("recoverable panic translation")
        }));
    }
}

#[test]
fn rust_interop_rejects_missing_aliased_and_shadowed_structural_markers() {
    for source in [
        r"
class CodecError(Error):
    message: str
@rust.structural
@rust(bridge.codec.encode)
def encode[T: Structural](value: T) -> Result[bytes, CodecError | RustPanicError]: ...
",
        r"
from sifr.meta import Structural as Shape
class CodecError(Error):
    message: str
@rust.structural
@rust(bridge.codec.encode)
def encode[T: Structural](value: T) -> Result[bytes, CodecError | RustPanicError]: ...
",
        r"
from sifr.meta import Structural
class Structural:
    pass
class CodecError(Error):
    message: str
@rust.structural
@rust(bridge.codec.encode)
def encode[T: Structural](value: T) -> Result[bytes, CodecError | RustPanicError]: ...
",
    ] {
        let errors = lower_errors_without_marker_import(source);
        assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::RUST_TYPE_PROBE_FAILURE)
                && error.message.contains("compiler-owned")
        }));
    }
}

#[test]
fn rust_interop_rejects_structural_marker_arguments_and_missing_target() {
    let argument_errors = lower_errors(
        r"
@rust.structural(enabled=True)
@rust(bridge.codec.encode)
def encode[T: Structural](value: T) -> Result[bytes, RustPanicError]: ...
",
    );
    assert_malformed(&argument_errors);

    let missing_target_errors = lower_errors(
        r"
@rust.structural
def encode[T: Structural](value: T) -> Result[bytes, RustPanicError]: ...
",
    );
    assert_malformed(&missing_target_errors);
}

#[test]
fn rust_interop_rejects_duplicate_structural_markers_and_invalid_generic_positions() {
    let duplicate_errors = lower_errors(
        r"
@rust.structural
@rust.structural
@rust(bridge.codec.encode)
def encode[T: Structural](value: T) -> Result[bytes, RustPanicError]: ...
",
    );
    assert_malformed(&duplicate_errors);

    let nested_errors = lower_errors(
        r"
@rust.structural
@rust(bridge.codec.encode)
def encode[T: Structural](value: list[T]) -> Result[bytes, RustPanicError]: ...
",
    );
    assert!(nested_errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::RUST_TYPE_PROBE_FAILURE)));
}
