use crate::lower_module;
use ruff_text_size::{TextRange, TextSize};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

fn range_for(source: &str, needle: &str) -> TextRange {
    let start = source.find(needle).expect("needle should occur");
    TextRange::new(
        TextSize::try_from(start).expect("test source offset fits"),
        TextSize::try_from(start + needle.len()).expect("test source offset fits"),
    )
}

#[test]
fn index_argument_traverses_unresolved_base_footprint() {
    let source = r#"
class Inner:
    values: list[int]

class Owner:
    value: int | None
    inner: Inner

    def pick(self) -> Inner:
        return self.inner

    def update(self, value: int | None) -> int | None:
        self.value = value
        return self.value

    def conflict(self) -> int | None:
        return self.update(self.pick().values[0])
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("index base footprint under the mutable receiver should overlap"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
            && error.primary_range == Some(range_for(source, "self.pick().values[0]"))
    }));
}

#[test]
fn slice_argument_traverses_unresolved_base_footprint() {
    let source = r#"
class Inner:
    values: list[int]

class Owner:
    values: list[int]
    inner: Inner

    def pick(self) -> Inner:
        return self.inner

    def replace(self, own values: list[int]) -> None:
        self.values = values

    def conflict(self) -> None:
        self.replace(self.pick().values[:1])
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("slice base footprint under the mutable receiver should overlap"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
            && error.primary_range == Some(range_for(source, "self.pick().values[:1]"))
    }));
}

#[test]
fn index_and_slice_arguments_accept_disjoint_unresolved_bases() {
    let source = r#"
class Inner:
    values: list[int]

class Source:
    inner: Inner

    def snapshot(self) -> Inner:
        return self.inner

class Owner:
    value: int | None
    values: list[int]

    def update(self, value: int | None) -> int | None:
        self.value = value
        return self.value

    def replace(self, own values: list[int]) -> None:
        self.values = values

class Coordinator:
    owner: Owner
    source: Source

    def apply(self) -> int | None:
        self.owner.replace(self.source.snapshot().values[:1])
        return self.owner.update(self.source.snapshot().values[0])
"#;
    let parsed = parse_module(source).expect("source should parse");
    lower_module(parsed.suite())
        .expect("disjoint unresolved index and slice bases should remain accepted");
}

#[test]
fn nested_index_argument_collects_inner_index_footprint() {
    let source = r#"
class Receiver:
    value: int | None
    index: int

    def update(self, value: int | None) -> int | None:
        self.value = value
        return self.value

class Coordinator:
    receiver: Receiver

    def conflict_index(self, other: tuple[list[int], list[int]]) -> int | None:
        return self.receiver.update(other[self.receiver.index][0])
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("nested index expressions under the mutable receiver should overlap"),
        Err(errors) => errors,
    };

    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
                && error.primary_range == Some(range_for(source, "other[self.receiver.index][0]"))
        }),
        "{errors:?}"
    );
}

#[test]
fn nested_index_and_slice_arguments_accept_disjoint_inner_indexes() {
    let source = r#"
class Receiver:
    value: int | None
    index: int

    def update(self, value: int | None) -> int | None:
        self.value = value
        return self.value

    def replace(self, own values: list[int]) -> None:
        pass

class Source:
    index: int

class Coordinator:
    receiver: Receiver
    source: Source

    def accepted(self, other: tuple[list[int], list[int]]) -> int | None:
        self.receiver.replace(other[self.source.index][:1])
        return self.receiver.update(other[self.source.index][0])
"#;
    let parsed = parse_module(source).expect("source should parse");
    lower_module(parsed.suite())
        .expect("disjoint nested index and slice footprints should remain accepted");
}
