use super::early_adapters::{attached_contract, compile_errors, project};

#[test]
fn attached_api_public_name_cannot_collide_with_a_field() {
    let modules = project(
        r#"
from fixture.contract import Contract, contract_config

class Invalid(Contract):
    _config = contract_config(True)
    describe: int
"#,
        &attached_contract(),
    );

    let errors = compile_errors(&modules, "an attached API and field collision must fail");
    assert_eq!(
        errors
            .iter()
            .filter(
                |error| error.message.contains("attached API") && error.message.contains("collid")
            )
            .count(),
        2,
        "expected both collision diagnostics: {errors:#?}"
    );
}

#[test]
fn imported_private_attached_function_is_not_exposed() {
    let contract = attached_contract().replace(
        "@class_adapter_provider",
        r#"@attached_api("fixture.contract", "ContractApi", public_name="hidden", receiver="type", owner="T")
def _hidden[T: StaticProgram]() -> int:
    return 1

@class_adapter_provider"#,
    );
    let modules = project(
        r#"
from fixture.contract import Contract, contract_config

class Model(Contract):
    _config = contract_config(True)
    value: int

def invalid():
    Model.hidden()
"#,
        &contract,
    );

    let errors = compile_errors(
        &modules,
        "an imported private attached API must stay hidden",
    );
    assert!(
        errors.iter().any(|error| {
            error.message.contains("hidden")
                && (error.message.contains("unknown") || error.message.contains("has no"))
        }),
        "expected a missing-member diagnostic: {errors:#?}"
    );
}

#[test]
fn adapted_owner_requires_a_structurally_eligible_real_data_parent() {
    let modules = project(
        r#"
from fixture.contract import Contract, contract_config

class Grandparent:
    value: int

class Parent(Grandparent):
    pass

class Model(Parent, Contract):
    _config = contract_config(True)

def invalid():
    Model.describe()
"#,
        &attached_contract(),
    );

    let errors = compile_errors(
        &modules,
        "an adapted owner with an ineligible real data parent must fail StaticProgram",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("StaticProgram")),
        "expected a StaticProgram eligibility diagnostic: {errors:#?}"
    );
}

#[test]
fn attached_api_rejects_method_slots_bound_on_the_wrong_type_parameter() {
    let contract = attached_contract()
        .replace(
            "from sifr.meta import StaticProgram, StringStructural, Structural, ",
            "from sifr.meta import MethodSlots, StaticProgram, StringStructural, Structural, ",
        )
        .replace(
            "@class_adapter_provider",
            r#"@attached_api("fixture.contract", "ContractApi", public_name="misbound", receiver="type", owner="T")
def misbound[T: Structural, Slots: MethodSlots]() -> int:
    return 1

@class_adapter_provider"#,
        );
    let modules = project(
        r#"
from fixture.contract import Contract

class Model(Contract):
    value: int
"#,
        &contract,
    );

    let errors = compile_errors(
        &modules,
        "a MethodSlots bound on a non-owner parameter must not authorize the owner",
    );
    assert!(
        errors.iter().any(|error| {
            error
                .message
                .contains("owner type parameter must be bounded by StaticProgram or MethodSlots")
        }),
        "expected the owner-bound diagnostic: {errors:#?}"
    );
}
