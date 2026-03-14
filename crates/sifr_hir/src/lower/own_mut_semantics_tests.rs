use crate::{lower_module, HirModule, LoweringError};
use sifr_python_parser::parse_module;

fn lower_source(source: &str) -> Result<HirModule, Vec<LoweringError>> {
    let parsed = parse_module(source).expect("parse failed");
    lower_module(parsed.suite()).map(|result| result.module)
}

fn lower_error_messages(source: &str) -> Vec<String> {
    lower_source(source)
        .expect_err("expected lowering error")
        .into_iter()
        .map(|error| error.message)
        .collect()
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
    let errors = lower_error_messages(
        "def borrowed_return(mut items: list[int]) -> list[int]:\n    return items\n",
    );

    assert!(errors.iter().any(|message| {
        message
            == "cannot return borrowed parameter `items`: borrowed parameters cannot escape -- add `own` at the signature boundary or return `items.clone()`"
    }));
}

#[test]
fn test_mut_borrow_parameter_cannot_escape_via_local_binding() {
    let errors = lower_error_messages(
        "def borrowed_store(mut items: list[int]) -> int:\n    captured: list[int] = items\n    return len(captured)\n",
    );

    assert!(errors.iter().any(|message| {
        message
            == "cannot store borrowed parameter `items`: borrowed parameters cannot escape -- add `own` at the signature boundary or store `items.clone()`"
    }));
}

#[test]
fn test_own_parameter_cannot_be_mutated_without_mut() {
    let errors = lower_error_messages(
        "def owned_immutable_mutation(own items: list[int]) -> list[int]:\n    items[0] = 7\n    return items\n",
    );

    assert!(errors.iter().any(|message| {
        message
            == "cannot mutate through immutable parameter `items`: add `mut` to the parameter declaration"
    }));
}

#[test]
fn test_own_parameter_mutating_method_requires_mut() {
    let errors = lower_error_messages(
        "def owned_immutable_append(own items: list[int] = [1]) -> list[int]:\n    items.append(5)\n    return items\n",
    );

    assert!(errors.iter().any(|message| {
        message
            == "cannot mutate through immutable parameter `items`: add `mut` to the parameter declaration"
    }));
}

#[test]
fn test_borrowed_parameter_cannot_be_reassigned_without_mut() {
    let errors = lower_error_messages(
        "def borrowed_reassign(items: list[int]) -> int:\n    items = [1, 2, 3]\n    return len(items)\n",
    );

    assert!(errors.iter().any(|message| {
        message
            == "cannot reassign immutable parameter `items`: add `mut` to the parameter declaration"
    }));
}
