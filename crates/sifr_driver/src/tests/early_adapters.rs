use super::support::parse_suite;
use crate::{collect_project_hir_modules, compile_stdlib};
use sifr_ir::{DeclarationMetadataTargetKind, StaticProgramValue};
use std::collections::HashMap;

const TYPES: &str = r#"
class ContractDescriptor:
    kind: str
    enabled: bool
"#;

const CONTRACT: &str = r#"
from sifr.meta import ConstSpecializationOutcome, DeclarationInput, DeclarationPlan, PlannedField, PlannedIssue, PlannedIssueLabel, PlannedMetadata, ShapeInput
from fixture.contract_types import ContractDescriptor

@class_adapter_provider("fixture.contract_types", "ContractDescriptor")
@const_eval
@metadata("sifr.meta.issue_template", ("adapter_warning", ["enabled", "kind"]))
def adapt_contract(declaration: DeclarationInput[ContractDescriptor]) -> DeclarationPlan[ContractDescriptor]:
    fields: list[PlannedField] = []
    metadata: list[PlannedMetadata[ContractDescriptor]] = []
    issues: list[PlannedIssue[ContractDescriptor]] = []
    if declaration.provider_module != "fixture.contract" or declaration.provider_function != "adapt_contract":
        raise ValueError("provider identity was not canonical")
    for item in declaration.declaration.items:
        if item.kind == "field":
            declared_type: str | None = item.declared_type
            if declared_type is not None:
                fields.append(PlannedField(item.identity, declared_type))
        if item.kind == "method" and item.name == "parse":
            signature: str | None = item.signature
            if signature is None:
                raise ValueError("checked method signature was not retained")
    for descriptor in declaration.descriptors:
        if descriptor.target_kind == "class":
            metadata.append(PlannedMetadata("class", declaration.declaration.identity, "fixture.contract", descriptor.value))
    parent_identity: str | None = declaration.data_parent
    if parent_identity is not None:
        metadata.append(PlannedMetadata("class", declaration.declaration.identity, "fixture.parent", ContractDescriptor(parent_identity, True)))
    return DeclarationPlan(fields, metadata, "fixture.contract", "specialize", issues)

@class_adapter_marker("fixture.contract", "adapt_contract")
class Contract:
    pass

@class_descriptor("fixture.contract", "adapt_contract")
def contract_config(enabled: bool) -> ContractDescriptor:
    return ContractDescriptor("class", enabled)

@const_eval
def specialize(shape: ShapeInput[ContractDescriptor]) -> ConstSpecializationOutcome[str, ContractDescriptor]:
    for metadata in shape.root.metadata:
        if metadata.key == "fixture.contract":
            if metadata.value.enabled:
                return ConstSpecializationOutcome("produced", metadata.value.kind, [])
    raise ValueError("adapter metadata was not applied")
"#;

fn project(main: &str, contract: &str) -> HashMap<String, Vec<sifr_python_ast::Stmt>> {
    HashMap::from([
        ("fixture.contract_types".to_string(), parse_suite(TYPES)),
        ("fixture.contract".to_string(), parse_suite(contract)),
        ("main".to_string(), parse_suite(main)),
    ])
}

fn compile_errors(
    modules: &HashMap<String, Vec<sifr_python_ast::Stmt>>,
    message: &str,
) -> Vec<sifr_diagnostics::RenderedDiagnostic> {
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    match collect_project_hir_modules(modules, stdlib_defs) {
        Ok(_) => panic!("{message}"),
        Err(errors) => errors,
    }
}

fn attached_contract() -> String {
    CONTRACT
        .replace(
            "from sifr.meta import ",
            "from sifr.meta import StaticProgram, Structural, ",
        )
        .replace(
            "@class_adapter_provider",
            r#"@attached_api_set
class ContractApi:
    pass

@attached_api("fixture.contract", "ContractApi", public_name="describe", receiver="type", owner="T")
def describe[T: StaticProgram]() -> str:
    return "attached"

@attached_api("fixture.contract", "ContractApi", public_name="echo", receiver="type", owner="T")
def echo[T: StaticProgram, Input: Structural](input: Input) -> Input:
    return input

@attached_api("fixture.contract", "ContractApi", public_name="touch", receiver="mutable", owner="T")
def touch[T: StaticProgram](mut target: Self) -> int:
    return 1

@attached_api("fixture.contract", "ContractApi", public_name="finish", receiver="owned", owner="T")
def finish[T: StaticProgram](own target: Self) -> int:
    return 2

@class_adapter_provider"#,
        )
        .replace(
            "return DeclarationPlan(fields, metadata, \"fixture.contract\", \"specialize\", issues)",
            "return DeclarationPlan(fields, metadata, \"fixture.contract\", \"specialize\", issues, [], \"fixture.contract\", \"ContractApi\")",
        )
}

#[test]
fn erased_marker_runs_adapter_and_specializes_without_layout_or_constructor_cost() {
    let modules = project(
        r#"
from fixture.contract import Contract, contract_config

class Model(Contract):
    _config = contract_config(True)
    value: int

    @staticmethod
    def parse(own value: str) -> str:
        return value
"#,
        CONTRACT,
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let compiled = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("the non-Pydantic adapter should derive and specialize the class");

    let package = compiled
        .hir_modules
        .get("fixture.contract")
        .expect("contract module exists");
    assert!(package.classes.iter().all(|class| class.name != "Contract"));

    let main = compiled
        .hir_modules
        .get("main")
        .expect("main module exists");
    let model = main
        .classes
        .iter()
        .find(|class| class.name == "Model")
        .expect("adapted class exists");
    assert_eq!(model.parent_class, None);
    assert_eq!(model.fields.len(), 1);
    assert_eq!(model.fields[0].0, "value");
    assert!(main
        .imports
        .iter()
        .all(|import| import.names.iter().all(|name| name != "Contract")));

    let exported = compiled
        .external_defs
        .classes
        .get("main")
        .and_then(|classes| classes.get("Model"))
        .expect("adapted class is exported");
    let sifr_type_system::Type::Class { fields, .. } = exported else {
        panic!("adapted export should remain a class");
    };
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].0, "value");

    let selection = compiled
        .external_defs
        .class_adapter_selections
        .get("main")
        .and_then(|classes| classes.get("Model"))
        .expect("adapter selection is retained");
    assert_eq!(selection.provider_module, "fixture.contract");
    assert_eq!(selection.provider_function, "adapt_contract");
    assert_eq!(selection.marker_identities, ["fixture.contract.Contract"]);

    let metadata = compiled
        .external_defs
        .applied_adapter_metadata
        .get("main")
        .expect("adapter metadata is retained");
    assert_eq!(metadata.len(), 1);
    assert_eq!(metadata[0].target_kind, DeclarationMetadataTargetKind::Type);
    assert_eq!(metadata[0].key, "fixture.contract");

    let output = compiled
        .external_defs
        .specialization_outputs
        .get("main")
        .and_then(|outputs| outputs.first())
        .expect("adapter-requested specialization should run");
    assert_eq!(output.package_module, "fixture.contract");
    assert_eq!(output.function, "specialize");
    assert_eq!(
        output.value,
        StaticProgramValue::String("class".to_string())
    );
}

#[test]
fn adapter_plan_retains_selected_attached_api_set() {
    let contract = attached_contract();
    let modules = project(
        r#"
from fixture.contract import Contract, contract_config

class Model(Contract):
    _config = contract_config(True)
    value: int

def main():
    assert Model.describe() == "attached"
    assert Model.echo("residual") == "residual"
"#,
        &contract,
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let compiled = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("the adapter should select its attached API set");
    let selection = compiled
        .external_defs
        .class_adapter_selections
        .get("main")
        .and_then(|classes| classes.get("Model"))
        .expect("adapter selection is retained");
    assert_eq!(
        selection
            .attached_api_set
            .as_ref()
            .map(|set| (set.module.as_str(), set.symbol.as_str())),
        Some(("fixture.contract", "ContractApi"))
    );
    let main = compiled
        .hir_modules
        .get("main")
        .expect("main module exists");
    let debug_hir = format!("{main:?}");
    assert!(debug_hir.contains("describe::<Model>"));
    assert!(
        debug_hir.contains("echo::<String, Model>"),
        "attached residual identity missing from {debug_hir}"
    );
    let model = main
        .classes
        .iter()
        .find(|class| class.name == "Model")
        .expect("adapted model exists");
    assert!(model.methods.iter().all(|method| {
        !matches!(
            method.name.as_str(),
            "describe" | "echo" | "touch" | "finish"
        )
    }));
}

#[test]
fn attached_api_collision_reports_both_selected_declarations() {
    let contract = attached_contract();
    let modules = project(
        r#"
from fixture.contract import Contract, contract_config

class Invalid(Contract):
    _config = contract_config(True)
    value: int

    @staticmethod
    def describe() -> str:
        return "native"
"#,
        &contract,
    );
    let errors = compile_errors(&modules, "attached/native collision must fail");
    let collisions = errors
        .iter()
        .filter(|error| error.message.contains("attached API") && error.message.contains("collid"))
        .count();
    assert_eq!(
        collisions, 2,
        "expected both declaration diagnostics: {errors:#?}"
    );
}

#[test]
fn attached_owned_receiver_uses_normal_move_tracking() {
    let contract = attached_contract();
    let modules = project(
        r#"
from fixture.contract import Contract, contract_config

class Model(Contract):
    _config = contract_config(True)
    value: int

def invalid():
    model: Model = Model(1)
    result: int = model.finish()
    assert model.value == result
"#,
        &contract,
    );
    let errors = compile_errors(&modules, "owned attached receiver must move its owner");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("use of moved value: 'model'")),
        "expected move diagnostic: {errors:#?}"
    );
}

#[test]
fn unbound_generic_attached_owner_cannot_request_static_program() {
    let contract = attached_contract();
    let modules = project(
        r#"
from fixture.contract import Contract, contract_config

class GenericModel[T](Contract):
    _config = contract_config(True)
    value: T

def invalid():
    GenericModel.describe()
"#,
        &contract,
    );
    let errors = compile_errors(
        &modules,
        "an unbound generic owner must not satisfy StaticProgram",
    );
    assert!(errors.iter().any(|error| {
        error.message.contains("StaticProgram") || error.message.contains("concrete")
    }));
}

#[test]
fn concrete_generic_adapted_child_receives_concrete_attached_signature() {
    let contract = attached_contract().replace(
        "return DeclarationPlan(fields, metadata, \"fixture.contract\", \"specialize\", issues, [], \"fixture.contract\", \"ContractApi\")",
        r#"specialization_module: str | None = "fixture.contract"
    specialization_function: str | None = "specialize"
    if declaration.declaration.name == "Parent":
        specialization_module = None
        specialization_function = None
    return DeclarationPlan(fields, metadata, specialization_module, specialization_function, issues, [], "fixture.contract", "ContractApi")"#,
    );
    let modules = project(
        r#"
from fixture.contract import Contract, contract_config

class Parent[T](Contract):
    value: T

class Concrete(Parent[int]):
    _config = contract_config(True)
    label: str

def main():
    value: Concrete = Concrete(1, "ready")
    assert Concrete.echo("input") == "input"
    assert value.touch() == 1
"#,
        &contract,
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let compiled = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("concrete generic adapted child should receive attached APIs");
    let main = compiled
        .hir_modules
        .get("main")
        .expect("main module exists");
    let debug_hir = format!("{main:?}");
    assert!(debug_hir.contains("echo::<String, Concrete>"));
    assert!(debug_hir.contains("touch::<Concrete>"));
}

#[test]
fn conflicting_marker_providers_fail_at_the_base_declaration() {
    let other_contract = CONTRACT.replace(
        "@class_adapter_marker(\"fixture.contract\", \"adapt_contract\")\nclass Contract:\n    pass",
        r#"@class_adapter_provider("fixture.contract_types", "ContractDescriptor")
@const_eval
def adapt_other(declaration: DeclarationInput[ContractDescriptor]) -> DeclarationPlan[ContractDescriptor]:
    fields: list[PlannedField] = []
    metadata: list[PlannedMetadata[ContractDescriptor]] = []
    issues: list[PlannedIssue[ContractDescriptor]] = []
    return DeclarationPlan(fields, metadata, None, None, issues)

@class_adapter_marker("fixture.contract", "adapt_contract")
class Contract:
    pass

@class_adapter_marker("fixture.contract", "adapt_other")
class OtherContract:
    pass"#,
    );
    let modules = project(
        r#"
from fixture.contract import Contract, OtherContract

class Invalid(Contract, OtherContract):
    value: int
"#,
        &other_contract,
    );
    let errors = compile_errors(&modules, "conflicting providers must be rejected");
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::META_MALFORMED_DECLARATION.code()
            && error
                .message
                .contains("conflicting canonical adapter providers")
    }));
}

#[test]
fn adapter_plan_cannot_add_or_retype_fields() {
    let contract = CONTRACT.replace(
        "return DeclarationPlan(fields, metadata, \"fixture.contract\", \"specialize\", issues)",
        r#"fields.append(PlannedField(declaration.declaration.identity + ".generated", "str"))
    return DeclarationPlan(fields, metadata, "fixture.contract", "specialize", issues)"#,
    );
    let modules = project(
        r#"
from fixture.contract import Contract

class Invalid(Contract):
    value: int
"#,
        &contract,
    );
    let errors = compile_errors(&modules, "field additions must fail plan validation");
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::META_MALFORMED_DECLARATION.code()
            && error
                .message
                .contains("preserve every declared field identity")
    }));
}

#[test]
fn adapter_plan_cannot_remove_or_retype_fields() {
    let field_append = "fields.append(PlannedField(item.identity, declared_type))";
    for contract in [
        CONTRACT.replace(
            field_append,
            "fields.append(PlannedField(declaration.declaration.identity + \".\" + item.name, \"str\"))",
        ),
        CONTRACT.replace(field_append, "pass"),
    ] {
        let modules = project(
            r#"
from fixture.contract import Contract

class Invalid(Contract):
    value: int
"#,
            &contract,
        );
        let errors = compile_errors(&modules, "field removal and retyping must fail validation");
        assert!(
            errors.iter().any(|error| {
                error.code == sifr_diagnostics::DiagnosticCode::META_MALFORMED_DECLARATION.code()
                    && error
                        .message
                        .contains("preserve every declared field identity")
            }),
            "unexpected diagnostics: {errors:#?}"
        );
    }
}

#[test]
fn marker_declaration_is_erased_and_cannot_be_constructed() {
    let modules = project(
        r#"
from fixture.contract import Contract

def invalid() -> Contract:
    return Contract()
"#,
        CONTRACT,
    );
    let errors = compile_errors(&modules, "an erased marker must not have a runtime value");
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::META_MALFORMED_DECLARATION.code()
            && error
                .message
                .contains("is erased and is valid only as a class base")
    }));
}

#[test]
fn marker_reexports_keep_canonical_selection_but_not_runtime_imports() {
    let mut modules = project(
        r#"
from fixture.facade import Contract, contract_config

class Model(Contract):
    _config = contract_config(True)
    value: int
"#,
        CONTRACT,
    );
    modules.insert(
        "fixture.facade".to_string(),
        parse_suite("from fixture.contract import Contract, contract_config\n"),
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let compiled = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("a public marker re-export should remain a compile-time base");
    let selection = compiled
        .external_defs
        .class_adapter_selections
        .get("main")
        .and_then(|classes| classes.get("Model"))
        .expect("the re-exported marker selects its canonical provider");
    assert_eq!(selection.marker_identities, ["fixture.contract.Contract"]);
    let facade = compiled
        .hir_modules
        .get("fixture.facade")
        .expect("facade module exists");
    assert!(facade
        .imports
        .iter()
        .all(|import| import.names.iter().all(|name| name != "Contract")));
}

#[test]
fn marker_declaration_must_be_fieldless() {
    let contract = CONTRACT.replace(
        "class Contract:\n    pass",
        "class Contract:\n    value: int",
    );
    let modules = project(
        r#"
from fixture.contract import Contract
"#,
        &contract,
    );
    let errors = compile_errors(&modules, "marker fields must be rejected");
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::META_MALFORMED_DECLARATION.code()
            && error.message.contains("requires a field-less class")
    }));
}

#[test]
fn adapter_const_budget_failure_is_stable() {
    let contract = CONTRACT.replace(
        "fields: list[PlannedField] = []",
        "while True:\n        pass\n    fields: list[PlannedField] = []",
    );
    let modules = project(
        r#"
from fixture.contract import Contract

class Invalid(Contract):
    value: int
"#,
        &contract,
    );
    let errors = compile_errors(
        &modules,
        "adapter evaluation must enforce the const step budget",
    );
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::META_MALFORMED_DECLARATION.code()
            && error
                .message
                .contains("const evaluation step limit exceeded")
    }));
}

#[test]
fn marker_does_not_consume_the_optional_data_parent_and_adapter_sees_its_identity() {
    let modules = project(
        r#"
from fixture.contract import Contract, contract_config

class Parent:
    pass

class Child(Contract, Parent):
    _config = contract_config(True)
    value: int

    def __init__(self, value: int):
        self.value = value
"#,
        CONTRACT,
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let compiled = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("marker and one data parent should coexist");
    let child = compiled
        .hir_modules
        .get("main")
        .and_then(|module| module.classes.iter().find(|class| class.name == "Child"))
        .expect("adapted child exists");
    assert_eq!(child.parent_class.as_deref(), Some("Parent"));
    let metadata = compiled
        .external_defs
        .applied_adapter_metadata
        .get("main")
        .expect("parent-derived metadata exists");
    let parent = metadata
        .iter()
        .find(|item| item.key == "fixture.parent")
        .expect("adapter saw the parent identity");
    let StaticProgramValue::Record(fields) = &parent.value else {
        panic!("parent metadata should retain the descriptor record");
    };
    let kind = fields
        .iter()
        .find(|(name, _)| name == "kind")
        .map(|(_, value)| value)
        .expect("descriptor kind exists");
    let StaticProgramValue::String(parent_identity) = kind else {
        panic!("parent identity should be a string");
    };
    assert!(parent_identity.contains("main.Parent"));
}

#[test]
fn adapter_issue_limit_is_deterministic() {
    let contract = CONTRACT.replace(
        "issues: list[PlannedIssue[ContractDescriptor]] = []",
        r#"issues: list[PlannedIssue[ContractDescriptor]] = []
    count: int = 0
    while count < 33:
        labels: list[PlannedIssueLabel] = []
        notes: list[str] = []
        arguments: ContractDescriptor = ContractDescriptor("issue", True)
        issues.append(PlannedIssue("fixture.contract", "adapter_warning", "warning", arguments, declaration.declaration.origin, labels, notes))
        count += 1"#,
    );
    let modules = project(
        r#"
from fixture.contract import Contract

class Invalid(Contract):
    value: int
"#,
        &contract,
    );
    let errors = compile_errors(&modules, "adapter issue count must be bounded");
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::META_MALFORMED_DECLARATION.code()
            && error.message.contains("at most 32 issues")
    }));
}

#[test]
fn marker_is_rejected_in_annotation_position() {
    let modules = project(
        r#"
from fixture.contract import Contract

class Invalid:
    value: Contract
"#,
        CONTRACT,
    );
    let errors = compile_errors(&modules, "marker annotations must fail locally");
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::META_MALFORMED_DECLARATION.code()
            && error.message.contains("base-only declarations")
    }));
}

#[test]
fn private_markers_are_not_exported() {
    let contract = CONTRACT.replace("class Contract:", "class _Contract:");
    let modules = project(
        r#"
from fixture.contract import _Contract

class Invalid(_Contract):
    value: int
"#,
        &contract,
    );
    let errors = compile_errors(&modules, "private markers must not cross module exports");
    assert!(
        errors.iter().any(|error| {
            error.message.contains("_Contract")
                && (error.message.contains("cannot import private")
                    || error.message.contains("unknown type"))
        }),
        "{errors:#?}"
    );
}

#[test]
fn adapter_edits_but_not_consumer_source_movement_invalidate_program_identity() {
    fn identities(main: &str, contract: &str) -> ([u8; 32], [u8; 32]) {
        let modules = project(main, contract);
        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let compiled = collect_project_hir_modules(&modules, stdlib_defs)
            .expect("adapter identity project should compile");
        let invocation = compiled
            .external_defs
            .class_adapter_selections
            .get("main")
            .and_then(|classes| classes.get("Model"))
            .expect("adapter selection exists")
            .adapter_invocation_identity;
        let program = compiled
            .external_defs
            .specialization_outputs
            .get("main")
            .and_then(|outputs| outputs.first())
            .expect("adapter-requested program exists")
            .program_identity;
        (invocation, program)
    }

    let source = r#"
from fixture.contract import Contract, contract_config
class Model(Contract):
    _config = contract_config(True)
    value: int
"#;
    let moved = format!("\n\n{source}");
    let edited = CONTRACT.replace(
        "    fields: list[PlannedField] = []",
        "    adapter_revision: int = 2\n    fields: list[PlannedField] = []",
    );
    let base = identities(source, CONTRACT);
    assert_eq!(base, identities(&moved, CONTRACT));
    assert_ne!(base, identities(source, &edited));
}
