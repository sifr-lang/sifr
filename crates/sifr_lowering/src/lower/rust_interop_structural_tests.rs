use crate::{lower_module_with_externals, ExternalDefs, HirDiagnostic, HirModule};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{
    DeclarationMetadataTargetKind, HirExpr, RustInteropDecoratorKind, TypedDeclarationMetadata,
};
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
                "StringStructural".to_string(),
                Type::Class {
                    identity: Some("sifr.meta.StringStructural".to_string()),
                    type_args: Vec::new(),
                    name: "StringStructural".to_string(),
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
            (
                "MethodSlots".to_string(),
                Type::Class {
                    identity: Some("sifr.meta.MethodSlots".to_string()),
                    type_args: Vec::new(),
                    name: "MethodSlots".to_string(),
                    fields: Vec::new(),
                    methods: Vec::new(),
                    parent_class: None,
                },
            ),
            (
                "Context".to_string(),
                Type::Class {
                    identity: Some("sifr.meta.Context".to_string()),
                    type_args: Vec::new(),
                    name: "Context".to_string(),
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
fn structural_method_slots_require_one_context() {
    let source = r"
from sifr.meta import MethodSlots

class CodecError(Error):
    message: str

@rust.structural
@rust(bridge.codec.decode)
def decode[T: MethodSlots](value: T) -> Result[T, CodecError | RustPanicError]: ...
";
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module_with_externals(parsed.suite(), &structural_externals()) {
        Ok(_) => panic!("method-slot bridge without context must fail"),
        Err(errors) => errors,
    };
    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::RUST_SLOT_BOUND)));
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
fn rust_interop_accepts_distinct_structural_input_and_static_output() {
    let source = r"
from sifr.meta import StaticProgram, Structural

class CodecError(Error):
    message: str

@rust.structural
@rust(bridge.codec.convert)
def convert[Output: StaticProgram, Input: Structural](
    value: Input,
) -> Result[Output, CodecError | RustPanicError]: ...
";
    let parsed = parse_module(source).expect("source should parse");
    let module = lower_module_with_externals(parsed.suite(), &structural_externals())
        .map(|result| result.module)
        .expect("distinct structural generics should lower");
    assert_eq!(module.functions[0].type_params, ["Input", "Output"]);
    assert_eq!(
        module.type_param_bounds["convert"]["Output"],
        ["StaticProgram"]
    );
    assert_eq!(module.type_param_bounds["convert"]["Input"], ["Structural"]);
}

#[test]
fn string_structural_bound_accepts_only_recursive_string_leaves() {
    let source = r#"
from sifr.meta import StringStructural

class Leaf:
    value: str

class Payload:
    label: str
    leaves: list[Leaf]
    metadata: dict[str, str]

def accept[T: StringStructural](value: T) -> None:
    pass

def use(payload: Payload) -> None:
    accept("root")
    accept(payload)
    accept({"nested": ["left", "right"]})
"#;
    let parsed = parse_module(source).expect("source should parse");
    lower_module_with_externals(parsed.suite(), &structural_externals())
        .expect("bare and recursively nested string inputs should lower");

    for invalid_type in ["int", "list[bool]", "dict[int, str]", "str | None"] {
        let source = format!(
            r#"
from sifr.meta import StringStructural

def accept[T: StringStructural](value: T) -> None:
    pass

def use(value: {invalid_type}) -> None:
    accept(value)
"#
        );
        let parsed = parse_module(&source).expect("source should parse");
        let errors = match lower_module_with_externals(parsed.suite(), &structural_externals()) {
            Ok(_) => panic!("non-string leaf type {invalid_type} must fail"),
            Err(errors) => errors,
        };
        assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
                && error
                    .message
                    .contains("does not implement protocol 'StringStructural'")
        }));
    }
}

#[test]
fn string_structural_bound_checks_generic_type_arguments() {
    let source = r#"
from sifr.meta import StringStructural

class Phantom[T]:
    pass

def accept[T: StringStructural](value: T) -> None:
    pass

def use(value: Phantom[int]) -> None:
    accept(value)
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module_with_externals(parsed.suite(), &structural_externals()) {
        Ok(_) => panic!("non-string generic type arguments must fail"),
        Err(errors) => errors,
    };
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
            && error
                .message
                .contains("does not implement protocol 'StringStructural'")
    }));
}

#[test]
fn rust_interop_accepts_the_canonical_string_structural_marker() {
    let source = r"
from sifr.meta import StringStructural

class CodecError(Error):
    message: str

@rust.structural
@rust(bridge.codec.observe)
def observe[T: StringStructural](value: T) -> Result[str, CodecError | RustPanicError]: ...
";
    let parsed = parse_module(source).expect("source should parse");
    let module = lower_module_with_externals(parsed.suite(), &structural_externals())
        .map(|result| result.module)
        .expect("canonical string-structural bridge should lower");
    assert_eq!(
        module.type_param_bounds["observe"]["T"],
        ["StringStructural"]
    );
}

#[test]
fn rust_interop_rejects_aliased_string_structural_marker() {
    let source = r"
from sifr.meta import StringStructural as Strings

class CodecError(Error):
    message: str

@rust.structural
@rust(bridge.codec.observe)
def observe[T: StringStructural](value: T) -> Result[str, CodecError | RustPanicError]: ...
";
    let errors = lower_errors_without_marker_import(source);
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::RUST_TYPE_PROBE_FAILURE)
            && error.message.contains("compiler-owned")
    }));
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

    let direct_bytes = eligible
        .replace("value: str", "value: bytes")
        .replace("Record(\"ok\")", "Record(b\"ok\")");
    let parsed = parse_module(&direct_bytes).expect("source should parse");
    lower_module_with_externals(parsed.suite(), &structural_externals())
        .expect("a direct bytes field has one supported scalar encoding");

    let nested_bytes = eligible
        .replace("value: str", "value: list[bytes]")
        .replace("Record(\"ok\")", "Record([b\"ok\"])");
    let parsed = parse_module(&nested_bytes).expect("source should parse");
    let errors = match lower_module_with_externals(parsed.suite(), &structural_externals()) {
        Ok(_) => panic!("nested bytes must not select the sequence encoding"),
        Err(errors) => errors,
    };
    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
            && error
                .message
                .contains("does not implement protocol 'StaticProgram'")
    }));

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

    let unsupported = r#"
from sifr.meta import StaticProgram

class Base:
    base: str

@const_specialize("package.schema", "derive")
class Record(Base):
    value: str

def retain[T: StaticProgram](value: T) -> T:
    return value

def use(value: Record) -> Record:
    return retain(value)
"#;
    let parsed = parse_module(unsupported).expect("source should parse");
    let errors = match lower_module_with_externals(parsed.suite(), &structural_externals()) {
        Ok(_) => panic!("unsupported specialization owner must fail before Rust emission"),
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
fn structural_bound_accepts_enums_and_supported_ordinary_unions() {
    lower_ok(
        r"
from enum import Enum

class Status(Enum):
    READY = 4
    WAITING = 5

def accept[T: Structural](value: T) -> None:
    pass

def use(choice: int | str, status: Status) -> None:
    accept(choice)
    accept(status)
",
    );
}

#[test]
fn structural_bound_accepts_explicitly_mapped_rust_values() {
    lower_ok(
        r"
@rust.opaque(
    type=bridge.token.Token,
    structural=bridge.token.TokenMapping,
    close=none,
)
class Token:
    pass

def accept[T: Structural](value: T) -> None:
    pass

def use(value: Token) -> None:
    accept(value)
",
    );
}

#[test]
fn string_structural_bound_rejects_mapped_rust_values_without_visible_leaf_types() {
    let errors = lower_errors(
        r"
from sifr.meta import StringStructural

@rust.opaque(
    type=bridge.token.Token,
    structural=bridge.token.TokenMapping,
    close=none,
)
class Token:
    pass

def accept[T: StringStructural](value: T) -> None:
    pass

def use(value: Token) -> None:
    accept(value)
",
    );

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
            && error
                .message
                .contains("does not implement protocol 'StringStructural'")
    }));
}

#[test]
fn structurally_mapped_rust_values_reject_fields_parents_and_generics() {
    for declaration in [
        "class Token:\n    value: str",
        "class Token[T]:\n    pass",
        "class Base:\n    pass\n\n@rust.opaque(type=bridge.token.Token, structural=bridge.token.TokenMapping, close=none)\nclass Token(Base):\n    pass",
    ] {
        let source = if declaration.starts_with("class Base") {
            declaration.to_string()
        } else {
            format!(
                "@rust.opaque(type=bridge.token.Token, structural=bridge.token.TokenMapping, close=none)\n{declaration}"
            )
        };
        let errors = lower_errors(&source);
        assert!(errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::RUST_CONFIG_MALFORMED_DECORATOR)
                && error.message.contains("fieldless, non-generic root classes")
        }));
    }
}

#[test]
fn structural_bound_rejects_unmapped_rust_values() {
    let errors = lower_errors(
        r"
@rust.opaque(type=bridge.token.Token, close=none)
class Token:
    pass

def accept[T: Structural](value: T) -> None:
    pass

def use(value: Token) -> None:
    accept(value)
",
    );

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
            && error
                .message
                .contains("does not implement protocol 'Structural'")
    }));
}

#[test]
fn structural_bound_rejects_enum_discriminant_overflow() {
    let errors = lower_errors(
        r"
from enum import Enum

class Status(Enum):
    LAST = 9223372036854775807
    OVERFLOW = 'auto'

def accept[T: Structural](value: T) -> None:
    pass

def use(status: Status) -> None:
    accept(status)
",
    );

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
            && error
                .message
                .contains("does not implement protocol 'Structural'")
    }));
}

#[test]
fn structural_bound_rejects_imported_enum_with_unrepresentable_metadata() {
    let mut externals = structural_externals();
    externals.classes.insert(
        "models".to_string(),
        HashMap::from([(
            "Status".to_string(),
            Type::Enum {
                identity: Some("models.Status".to_string()),
                name: "Status".to_string(),
                variants: vec![("READY".to_string(), Some(1))],
            },
        )]),
    );
    externals.declaration_metadata.insert(
        "models".to_string(),
        vec![TypedDeclarationMetadata {
            owner: "Status".to_string(),
            target_kind: DeclarationMetadataTargetKind::Type,
            target_name: None,
            key: "example.policy".to_string(),
            value_type: Type::Int,
            value: HirExpr::BinOp {
                left: Box::new(HirExpr::IntLiteral(1)),
                op: "+".to_string(),
                right: Box::new(HirExpr::IntLiteral(1)),
                ty: Type::Int,
            },
            range: Default::default(),
        }],
    );
    let source = r"
from sifr.meta import Structural
from models import Status

def accept[T: Structural](value: T) -> None:
    pass

def use(status: Status) -> None:
    accept(status)
";
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module_with_externals(parsed.suite(), &externals) {
        Ok(_) => panic!("unrepresentable imported metadata must fail lowering"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
            && error
                .message
                .contains("does not implement protocol 'Structural'")
    }));
}

#[test]
fn structural_bound_rejects_an_ordinary_union_with_an_unsupported_member() {
    let errors = lower_errors(
        r"
def accept[T: Structural](value: T) -> None:
    pass

def use(value: int | list[bytes]) -> None:
    accept(value)
",
    );

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
            && error
                .message
                .contains("does not implement protocol 'Structural'")
    }));
}

#[test]
fn structural_bound_rejects_platform_integer_union_members() {
    let errors = lower_errors(
        r"
def accept[T: Structural](value: T) -> None:
    pass

def use(value: int | usize) -> None:
    accept(value)
",
    );

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)
            && error
                .message
                .contains("does not implement protocol 'Structural'")
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
