use crate::lower_module;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};
use sifr_python_parser::parse_module;

#[test]
fn inferred_mutable_receiver_rejects_shared_protocol_conformance() {
    let source = r#"
class Shared(Protocol):
    def update(self) -> None:
        pass

class MutableImplementation:
    value: int

    def update(self) -> None:
        self.value += 1

class SharedImplementation:
    def update(self) -> None:
        pass
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("receiver mismatch must be rejected"),
        Err(errors) => errors,
    };
    let error = errors
        .iter()
        .find(|error| error.code == Some(DiagnosticCode::PROTO_RECEIVER_CONVENTION_MISMATCH))
        .expect("receiver mismatch diagnostic should be present");
    assert_eq!(
        error.args.get("class_name"),
        Some(&DiagnosticArg::String("MutableImplementation".to_string()))
    );
    assert_eq!(
        error.args.get("method"),
        Some(&DiagnosticArg::String("update".to_string()))
    );
    assert_eq!(
        error.args.get("protocol"),
        Some(&DiagnosticArg::String("Shared".to_string()))
    );
}

#[test]
fn protocol_receiver_mismatches_retain_distinct_structured_arguments() {
    let source = r#"
class Shared(Protocol):
    def update(self) -> None:
        pass

class FirstMutableImplementation:
    value: int

    def update(self) -> None:
        self.value += 1

class SecondMutableImplementation:
    value: int

    def update(self) -> None:
        self.value += 1
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("receiver mismatches must be rejected"),
        Err(errors) => errors,
    };
    let mut class_names = errors
        .iter()
        .filter(|error| error.code == Some(DiagnosticCode::PROTO_RECEIVER_CONVENTION_MISMATCH))
        .map(|error| {
            assert_eq!(
                error.args.get("method"),
                Some(&DiagnosticArg::String("update".to_string()))
            );
            assert_eq!(
                error.args.get("protocol"),
                Some(&DiagnosticArg::String("Shared".to_string()))
            );
            match error.args.get("class_name") {
                Some(DiagnosticArg::String(class_name)) => class_name.clone(),
                other => panic!("class_name argument should be populated: {other:?}"),
            }
        })
        .collect::<Vec<_>>();
    class_names.sort();
    assert_eq!(
        class_names,
        vec![
            "FirstMutableImplementation".to_string(),
            "SecondMutableImplementation".to_string(),
        ]
    );
}

#[test]
fn fixed_trait_receiver_rejects_transitive_receiver_mutation() {
    let source = r#"
class Counter:
    value: int

    def bump(self) -> None:
        self.value += 1

    def __eq__(self, other: Counter) -> bool:
        self.bump()
        return self.value == other.value
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("fixed receiver mutation must fail"),
        Err(errors) => errors,
    };
    let error = errors
        .iter()
        .find(|error| error.code == Some(DiagnosticCode::PROTO_FIXED_RECEIVER_MUTATION))
        .expect("fixed receiver diagnostic should be present");
    assert_eq!(
        error.args.get("method"),
        Some(&DiagnosticArg::String("__eq__".to_string()))
    );
    assert_eq!(
        error.args.get("trait_name"),
        Some(&DiagnosticArg::String("PartialEq".to_string()))
    );
}
