use crate::lower_module;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};
use sifr_python_parser::parse_module;

#[test]
fn explicit_mutable_receiver_rejects_shared_protocol_conformance() {
    let source = r#"
class Shared(Protocol):
    def update(self) -> None:
        pass

class MutableImplementation:
    value: int

    def update(mut self) -> None:
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

    def update(mut self) -> None:
        self.value += 1

class SecondMutableImplementation:
    value: int

    def update(mut self) -> None:
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

    def bump(mut self) -> None:
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
        .find(|error| error.code == Some(DiagnosticCode::PROTO_FIXED_RECEIVER_VIOLATION))
        .expect("fixed receiver diagnostic should be present");
    assert_eq!(
        error.args.get("class_name"),
        Some(&DiagnosticArg::String("Counter".to_string()))
    );
    assert_eq!(
        error.args.get("method"),
        Some(&DiagnosticArg::String("__eq__".to_string()))
    );
    assert_eq!(
        error.args.get("trait_name"),
        Some(&DiagnosticArg::String("PartialEq".to_string()))
    );
}

#[test]
fn fixed_trait_receivers_require_explicit_source_conventions() {
    let source = r#"
class Addable(Protocol):
    def __add__(self, other: Addable) -> Addable:
        ...

class Value:
    number: int

    def __add__(self, other: Value) -> Value:
        return Value(self.number + other.number)
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("fixed receiver declaration mismatches must fail"),
        Err(errors) => errors,
    };
    let violations = errors
        .iter()
        .filter(|error| error.code == Some(DiagnosticCode::PROTO_FIXED_RECEIVER_VIOLATION))
        .collect::<Vec<_>>();
    assert_eq!(violations.len(), 2, "{errors:?}");
    assert_eq!(
        violations
            .iter()
            .map(|error| error.args.get("class_name"))
            .collect::<Vec<_>>(),
        vec![
            Some(&DiagnosticArg::String("Addable".to_string())),
            Some(&DiagnosticArg::String("Value".to_string())),
        ]
    );
}

#[test]
fn fixed_receiver_diagnostics_are_declaration_ordered_and_class_distinct() {
    let source = r#"
class Zulu:
    value: int

    def __eq__(self, other: Zulu) -> bool:
        self.value += 1
        return self.value == other.value

class Bravo:
    value: int

    def __eq__(self, other: Bravo) -> bool:
        self.value += 1
        return self.value == other.value

class Charlie:
    value: int

    def __eq__(self, other: Charlie) -> bool:
        self.value += 1
        return self.value == other.value

class Delta:
    value: int

    def __eq__(self, other: Delta) -> bool:
        self.value += 1
        return self.value == other.value

class Echo:
    value: int

    def __eq__(self, other: Echo) -> bool:
        self.value += 1
        return self.value == other.value

class Foxtrot:
    value: int

    def __eq__(self, other: Foxtrot) -> bool:
        self.value += 1
        return self.value == other.value
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("fixed receiver mutations must fail"),
        Err(errors) => errors,
    };
    let class_names = errors
        .iter()
        .filter(|error| error.code == Some(DiagnosticCode::PROTO_FIXED_RECEIVER_VIOLATION))
        .map(|error| match error.args.get("class_name") {
            Some(DiagnosticArg::String(class_name)) => class_name.as_str(),
            other => panic!("class_name argument should be populated: {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        class_names,
        ["Zulu", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot"]
    );
}

#[test]
fn immutable_parameter_field_receivers_report_root_binding_argument() {
    let source = r#"
class Helper:
    items: list[int]

    def bump(mut self) -> None:
        self.items.append(1)

class Owner:
    helper: Helper

def borrowed(owner: Owner) -> None:
    owner.helper.bump()

def owned(own owner: Owner) -> None:
    owner.helper.bump()
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("immutable parameter field receivers must fail"),
        Err(errors) => errors,
    };
    let diagnostics = errors
        .iter()
        .filter(|error| error.code == Some(DiagnosticCode::OWN_IMMUTABLE_PARAMETER_MUTATION))
        .collect::<Vec<_>>();
    assert_eq!(diagnostics.len(), 2, "{errors:?}");
    assert!(diagnostics.iter().all(|error| {
        error.args.get("binding") == Some(&DiagnosticArg::String("owner".to_string()))
    }));
}
