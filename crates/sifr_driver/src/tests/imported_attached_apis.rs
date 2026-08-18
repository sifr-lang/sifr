use super::early_adapters::{attached_contract, compile_errors, project};
use super::support::parse_suite;
use crate::{collect_project_hir_modules, compile_stdlib};

const MODELS: &str = r#"
from fixture.contract import Contract, contract_config

class Selected(Contract):
    _config = contract_config(True)
    value: int

class Plain(Contract):
    _config = contract_config(True)
    value: int
"#;

fn selective_contract() -> String {
    attached_contract().replace(
        "return DeclarationPlan(fields, metadata, \"fixture.contract\", \"specialize\", issues, [], \"fixture.contract\", \"ContractApi\")",
        r#"attached_module: str | None = "fixture.contract"
    attached_symbol: str | None = "ContractApi"
    if declaration.declaration.name == "Plain":
        attached_module = None
        attached_symbol = None
    return DeclarationPlan(fields, metadata, "fixture.contract", "specialize", issues, [], attached_module, attached_symbol)"#,
    )
}

#[test]
fn imported_selected_owner_uses_finalized_set_through_type_and_instance_aliases() {
    let contract = selective_contract();
    let mut modules = project(
        r#"
from fixture.models import Plain as PlainAlias, Selected as SelectedAlias

def main():
    selected: SelectedAlias = SelectedAlias(1)
    plain: PlainAlias = PlainAlias(2)
    assert SelectedAlias.describe() == "attached"
    assert SelectedAlias.echo("residual") == "residual"
    assert selected.touch() == 1
    assert plain.value == 2
"#,
        &contract,
    );
    modules.insert("fixture.models".to_string(), parse_suite(MODELS));
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let compiled = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("imported selected attached APIs should lower through aliases");
    let main = compiled
        .hir_modules
        .get("main")
        .expect("main module exists");
    let debug_hir = format!("{main:?}");
    assert!(
        debug_hir.contains("describe::<SelectedAlias>"),
        "{debug_hir}"
    );
    assert!(
        debug_hir.contains("echo::<String, SelectedAlias>"),
        "{debug_hir}"
    );
    assert!(debug_hir.contains("touch::<SelectedAlias>"), "{debug_hir}");
}

#[test]
fn imported_unselected_owner_exposes_no_provisional_attached_api() {
    let contract = selective_contract();
    let mut modules = project(
        r#"
from fixture.models import Plain as PlainAlias, Selected as SelectedAlias

def invalid():
    plain: PlainAlias = PlainAlias(1)
    assert SelectedAlias.describe() == "attached"
    PlainAlias.describe()
    plain.touch()
"#,
        &contract,
    );
    modules.insert("fixture.models".to_string(), parse_suite(MODELS));
    let errors = compile_errors(
        &modules,
        "an imported finalized owner without a selected set must expose no attached API",
    );
    let missing_members = errors
        .iter()
        .filter(|error| {
            error.code == sifr_diagnostics::DiagnosticCode::CLASS_MISSING_MEMBER.code()
                && error.message.contains("PlainAlias")
                && (error.message.contains("describe") || error.message.contains("touch"))
        })
        .count();
    assert_eq!(
        missing_members, 2,
        "expected imported type and instance missing-member diagnostics: {errors:#?}"
    );
}

#[test]
fn imported_unbound_generic_owner_still_fails_static_program_bound() {
    let contract = attached_contract().replace(
        "return DeclarationPlan(fields, metadata, \"fixture.contract\", \"specialize\", issues, [], \"fixture.contract\", \"ContractApi\")",
        r#"specialization_module: str | None = "fixture.contract"
    specialization_function: str | None = "specialize"
    if declaration.declaration.name == "Generic":
        specialization_module = None
        specialization_function = None
    return DeclarationPlan(fields, metadata, specialization_module, specialization_function, issues, [], "fixture.contract", "ContractApi")"#,
    );
    let mut modules = project(
        r#"
from fixture.models import Generic as GenericAlias

def invalid():
    GenericAlias.describe()
"#,
        &contract,
    );
    modules.insert(
        "fixture.models".to_string(),
        parse_suite(
            r#"
from fixture.contract import Contract, contract_config

class Generic[T](Contract):
    _config = contract_config(True)
    value: T
"#,
        ),
    );
    let errors = compile_errors(
        &modules,
        "an imported unbound generic owner must not satisfy StaticProgram",
    );
    assert!(
        errors.iter().any(|error| {
            error.message.contains("StaticProgram") || error.message.contains("concrete")
        }),
        "expected imported StaticProgram diagnostic: {errors:#?}"
    );
}
