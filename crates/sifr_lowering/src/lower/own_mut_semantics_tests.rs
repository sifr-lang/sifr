use crate::{lower_module, HirDiagnostic, HirModule};
use ruff_text_size::{TextRange, TextSize};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;

fn lower_source(source: &str) -> Result<HirModule, Vec<HirDiagnostic>> {
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).map(|result| result.module)
}

fn lower_errors(source: &str) -> Vec<HirDiagnostic> {
    lower_source(source).expect_err("expected lowering error")
}

fn range_for_after(source: &str, after: &str, needle: &str) -> TextRange {
    let after_start = source.find(after).expect("anchor should exist");
    let relative_start = source[after_start..]
        .find(needle)
        .expect("needle should exist after anchor");
    let start = (after_start + relative_start) as u32;
    TextRange::new(
        TextSize::new(start),
        TextSize::new(start + needle.len() as u32),
    )
}

#[test]
fn test_own_mut_parameter_allows_mutation_and_return() {
    let result = lower_source(
        "def mutate_and_return(own mut items: list[int]) -> list[int]:\n    items[0] = 7\n    return items\n",
    );

    assert!(
        result.is_ok(),
        "own mut parameters should be mutable and returnable"
    );
}

#[test]
fn test_mut_borrow_parameter_cannot_escape_via_return() {
    let source = "def borrowed_return(mut items: list[int]) -> list[int]:\n    return items\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message
            == "cannot return borrowed parameter `items`: borrowed parameters cannot escape -- add `own` at the signature boundary or return `items.clone()`"
            && error.code == Some(DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES)
            && error.primary_range == Some(range_for_after(source, "return ", "items"))
    }));
}

#[test]
fn test_mut_borrow_parameter_cannot_escape_via_local_binding() {
    let source = "def borrowed_store(mut items: list[int]) -> int:\n    captured: list[int] = items\n    return len(captured)\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message
            == "cannot store borrowed parameter `items`: borrowed parameters cannot escape -- add `own` at the signature boundary or store `items.clone()`"
            && error.code == Some(DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES)
            && error.primary_range == Some(range_for_after(source, "= ", "items"))
    }));
}

#[test]
fn test_own_parameter_cannot_be_mutated_without_mut() {
    let source = "def owned_immutable_mutation(own items: list[int]) -> list[int]:\n    items[0] = 7\n    return items\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message
            == "cannot mutate through immutable parameter `items`: add `mut` to the parameter declaration"
            && error.code == Some(DiagnosticCode::OWN_IMMUTABLE_PARAMETER_MUTATION)
            && error.primary_range == Some(range_for_after(source, "    items[0]", "items"))
    }));
}

#[test]
fn test_own_parameter_mutating_method_requires_mut() {
    let source = "def owned_immutable_append(own items: list[int] = [1]) -> list[int]:\n    items.append(5)\n    return items\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message
            == "cannot mutate through immutable parameter `items`: add `mut` to the parameter declaration"
            && error.code == Some(DiagnosticCode::OWN_IMMUTABLE_PARAMETER_MUTATION)
            && error.primary_range == Some(range_for_after(source, "    items.append", "items"))
    }));
}

#[test]
fn test_borrowed_parameter_cannot_be_reassigned_without_mut() {
    let source =
        "def borrowed_reassign(items: list[int]) -> int:\n    items = [1, 2, 3]\n    return len(items)\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message
            == "cannot reassign immutable parameter `items`: add `mut` to the parameter declaration"
            && error.code == Some(DiagnosticCode::OWN_IMMUTABLE_PARAMETER_REASSIGNMENT)
            && error.primary_range == Some(range_for_after(source, "    items = ", "items"))
    }));
}

#[test]
fn test_borrowed_parameter_cannot_be_augassigned_without_mut() {
    let source = "def borrowed_augassign(count: int) -> int:\n    count += 1\n    return count\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message
            == "cannot reassign immutable parameter `count`: add `mut` to the parameter declaration"
            && error.code == Some(DiagnosticCode::OWN_IMMUTABLE_PARAMETER_REASSIGNMENT)
            && error.primary_range == Some(range_for_after(source, "    count += ", "count"))
    }));
}

#[test]
fn test_borrowed_parameter_cannot_be_tuple_reassigned_without_mut() {
    let source = "def borrowed_tuple_reassign(items: list[int], other: list[int]) -> int:\n    items, other = other, items\n    return len(items)\n";
    let errors = lower_errors(source);

    assert!(errors.iter().any(|error| {
        error.message
            == "cannot reassign immutable parameter `items`: add `mut` to the parameter declaration"
            && error.code == Some(DiagnosticCode::OWN_IMMUTABLE_PARAMETER_REASSIGNMENT)
            && error.primary_range == Some(range_for_after(source, "    items, other", "items"))
    }));
}
