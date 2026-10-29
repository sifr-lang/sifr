use super::*;
use crate::lower::lower_module;
use sifr_python_parser::parse_module;

fn errors<T>(result: Result<T, Vec<crate::HirDiagnostic>>) -> Vec<crate::HirDiagnostic> {
    match result {
        Ok(_) => panic!("lowering unexpectedly succeeded"),
        Err(errors) => errors,
    }
}

#[test]
fn collects_typed_metadata_for_all_declaration_target_families() {
    let source = r#"
@metadata("fixture.model", "record")
@metadata("field", "value", "fixture.description", "payload")
class Model:
    value: int

    @metadata("fixture.callback", True)
    @metadata("parameter", "scale", "fixture.unit", "ratio")
    def transform(self, scale: int) -> int:
        return self.value * scale

@metadata("fixture.function", 7)
@metadata("parameter", "value", "fixture.unit", "count")
def inspect(value: int) -> int:
    return value
"#;
    let parsed = parse_module(source).expect("fixture parses");
    let lowered = lower_module(parsed.suite()).expect("fixture lowers");
    assert_eq!(lowered.declaration_metadata.len(), 6);
    assert!(
        lowered
            .declaration_metadata
            .iter()
            .any(|item| item.target_kind == DeclarationMetadataTargetKind::Field)
    );
    assert!(
        lowered
            .declaration_metadata
            .iter()
            .any(|item| item.target_kind == DeclarationMetadataTargetKind::Parameter)
    );
}

#[test]
fn rejects_dynamic_metadata_values_with_meta_code() {
    let source = "@metadata(\"fixture.value\", build())\ndef inspect(value: int) -> int:\n    return value\n";
    let parsed = parse_module(source).expect("fixture parses");
    let diagnostics = errors(lower_module(parsed.suite()));
    assert_eq!(
        diagnostics[0].code,
        Some(DiagnosticCode::META_MALFORMED_DECLARATION)
    );
}

#[test]
fn rejects_metadata_for_missing_declaration_child() {
    let source = r#"
@metadata("field", "missing", "fixture.description", "payload")
class Model:
    value: int

@metadata("parameter", "missing", "fixture.unit", "count")
def inspect(value: int) -> int:
    return value
"#;
    let parsed = parse_module(source).expect("fixture parses");
    let diagnostics = errors(lower_module(parsed.suite()));
    assert_eq!(diagnostics.len(), 2);
    assert!(
        diagnostics.iter().all(|diagnostic| {
            diagnostic.code == Some(DiagnosticCode::META_MALFORMED_DECLARATION)
        })
    );
}

#[test]
fn rejects_duplicate_const_specialization_decorators() {
    let source = r#"
@const_specialize("fixture.first", "derive")
@const_specialize("fixture.second", "derive")
class Model:
    value: int
"#;
    let parsed = parse_module(source).expect("fixture parses");
    let diagnostics = errors(lower_module(parsed.suite()));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == Some(DiagnosticCode::META_MALFORMED_DECLARATION)
            && diagnostic.message.contains("exactly one @const_specialize")
    }));
}
