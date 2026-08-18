use super::support::parse_suite;
use crate::{collect_project_hir_modules, compile_stdlib};
use sifr_ir::{DeclarationDescriptorKind, StaticProgramValue};
use std::collections::HashMap;

fn descriptor_project(main: &str) -> HashMap<String, Vec<sifr_python_ast::Stmt>> {
    let mut modules = HashMap::new();
    modules.insert(
        "fixture.contract_types".to_string(),
        parse_suite(
            r#"
from sifr.meta import CallableIdentity

class ContractDescriptor:
    kind: str
    limit: int | None
    tags: list[str]
    callback: CallableIdentity | None
"#,
        ),
    );
    modules.insert(
        "fixture.contract".to_string(),
        parse_suite(
            r#"
from sifr.meta import CallableIdentity, DeclarationInput, DeclarationPlan
from fixture.contract_types import ContractDescriptor

@class_adapter_provider("fixture.contract_types", "ContractDescriptor")
@const_eval
def adapt_contract(value: DeclarationInput[ContractDescriptor]) -> DeclarationPlan[ContractDescriptor]:
    return DeclarationPlan([], [], None, None, [])

@field_descriptor("fixture.contract", "adapt_contract")
def option(limit: int | None, tags: list[str], callback: CallableIdentity | None) -> ContractDescriptor:
    return ContractDescriptor("field", limit, tags, callback)

@class_descriptor("fixture.contract", "adapt_contract")
def config() -> ContractDescriptor:
    tags: list[str] = []
    return ContractDescriptor("class", None, tags, None)

@method_descriptor("fixture.contract", "adapt_contract")
def before(tag: str) -> ContractDescriptor:
    tags: list[str] = [tag]
    return ContractDescriptor("method", None, tags, None)

@type_descriptor("fixture.contract", "adapt_contract")
def bounded(limit: int) -> ContractDescriptor:
    tags: list[str] = []
    return ContractDescriptor("type", limit, tags, None)

@field_descriptor("fixture.contract", "adapt_contract")
def broken() -> ContractDescriptor:
    raise ValueError("broken descriptor")

@field_descriptor("fixture.contract", "adapt_contract")
def forever() -> ContractDescriptor:
    while True:
        pass
    tags: list[str] = []
    return ContractDescriptor("field", None, tags, None)
"#,
        ),
    );
    modules.insert("main".to_string(), parse_suite(main));
    modules
}

#[test]
fn descriptor_calls_in_runtime_locations_are_rejected() {
    let modules = descriptor_project(
        r#"
from fixture.contract import config

def invalid():
    value = config()
"#,
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let errors = match collect_project_hir_modules(&modules, stdlib_defs) {
        Ok(_) => panic!("descriptor call in a runtime function must be rejected"),
        Err(errors) => errors,
    };
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::META_MALFORMED_DECLARATION.code()
            && error
                .message
                .contains("not valid in this declaration location")
    }));
}

#[test]
fn descriptor_const_failure_uses_stable_meta_diagnostic() {
    let modules = descriptor_project(
        r#"
from fixture.contract import broken

class Invalid:
    value: int = broken()
"#,
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let errors = match collect_project_hir_modules(&modules, stdlib_defs) {
        Ok(_) => panic!("failing descriptor const evaluation must be rejected"),
        Err(errors) => errors,
    };
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::META_MALFORMED_DECLARATION.code()
            && error.message.contains("failed bounded const evaluation")
    }));
}

#[test]
fn descriptor_const_budget_failure_is_stable() {
    let modules = descriptor_project(
        r#"
from fixture.contract import forever

class Invalid:
    value: int = forever()
"#,
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let errors = match collect_project_hir_modules(&modules, stdlib_defs) {
        Ok(_) => panic!("descriptor evaluation must enforce the const step budget"),
        Err(errors) => errors,
    };
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::META_MALFORMED_DECLARATION.code()
            && error
                .message
                .contains("const evaluation step limit exceeded")
    }));
}

#[test]
fn mixed_canonical_providers_fail_before_descriptor_evaluation() {
    let mut modules = descriptor_project(
        r#"
from fixture.contract import config
from fixture.other_contract import other_field

class Invalid:
    _config = config()
    value: int = other_field()
"#,
    );
    modules.insert(
        "fixture.other_types".to_string(),
        parse_suite(
            r#"
class OtherDescriptor:
    enabled: bool
"#,
        ),
    );
    modules.insert(
        "fixture.other_contract".to_string(),
        parse_suite(
            r#"
from sifr.meta import DeclarationInput, DeclarationPlan
from fixture.other_types import OtherDescriptor

@class_adapter_provider("fixture.other_types", "OtherDescriptor")
@const_eval
def adapt_other(value: DeclarationInput[OtherDescriptor]) -> DeclarationPlan[OtherDescriptor]:
    return DeclarationPlan([], [], None, None, [])

@field_descriptor("fixture.other_contract", "adapt_other")
def other_field() -> OtherDescriptor:
    return OtherDescriptor(True)
"#,
        ),
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let errors = match collect_project_hir_modules(&modules, stdlib_defs) {
        Ok(_) => panic!("mixed providers on one declaration must be rejected"),
        Err(errors) => errors,
    };
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::TYPE_MISMATCH.code()
            && error.message.contains("same canonical provider")
    }));
    assert!(errors
        .iter()
        .all(|error| !error.message.contains("bounded const evaluation")));
}

#[test]
fn typed_descriptor_project_preserves_all_kinds_and_callable_identity() {
    let modules = descriptor_project(
        r#"
from typing import Annotated
from fixture.contract import before, bounded, config, option as field_option

def normalize(own value: str) -> str:
    return value

class Contract:
    _config = config()
    name: Annotated[str, bounded(16), bounded(32)] = field_option(None, ["public"], normalize)

    @staticmethod
    @before("normalize")
    def parse(own value: str) -> str:
        return value
"#,
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let project = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("typed descriptor project should compile");

    let providers = project
        .external_defs
        .class_adapter_providers
        .get("fixture.contract")
        .expect("provider exports exist");
    assert!(providers.contains_key("adapt_contract"));
    let functions = project
        .external_defs
        .descriptor_functions
        .get("fixture.contract")
        .expect("descriptor exports exist");
    assert_eq!(functions.len(), 6);

    let descriptors = project
        .external_defs
        .declaration_descriptors
        .get("main")
        .expect("consumer descriptors exist");
    assert_eq!(descriptors.len(), 5);
    assert_eq!(descriptors[0].target_kind, DeclarationDescriptorKind::Class);
    assert_eq!(descriptors[1].target_kind, DeclarationDescriptorKind::Type);
    assert_eq!(descriptors[2].target_kind, DeclarationDescriptorKind::Type);
    assert_eq!(descriptors[3].target_kind, DeclarationDescriptorKind::Field);
    assert_eq!(
        descriptors[4].target_kind,
        DeclarationDescriptorKind::Method
    );
    assert!(descriptors[4].target_identity.ends_with(":static"));

    let StaticProgramValue::Record(field_value) = &descriptors[3].value else {
        panic!("field descriptor should retain a typed record");
    };
    let callable = field_value
        .iter()
        .find(|(name, _)| name == "callback")
        .map(|(_, value)| value)
        .expect("callback field exists");
    let StaticProgramValue::CallableIdentity(callable) = callable else {
        panic!("callback should retain a sealed callable identity");
    };
    assert_eq!(callable.module, "main");
    assert_eq!(callable.owner, None);
    assert_eq!(callable.symbol, "normalize");
    assert!(!callable.signature.is_empty());

    let main = project.hir_modules.get("main").expect("main HIR exists");
    let contract = main
        .classes
        .iter()
        .find(|class| class.name == "Contract")
        .expect("Contract exists");
    assert_eq!(contract.fields.len(), 1);
    assert!(contract.field_defaults.is_empty());
}

#[test]
fn unrelated_same_basename_is_not_a_descriptor() {
    let mut modules = descriptor_project(
        r#"
from typing import Annotated
from unrelated import bounded

class Plain:
    value: Annotated[str, bounded(4)]
"#,
    );
    modules.insert(
        "unrelated".to_string(),
        parse_suite(
            r#"
def bounded(value: int) -> int:
    return value
"#,
        ),
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let project = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("an unrelated same-basename call should remain ordinary annotation metadata");
    assert!(!project
        .external_defs
        .declaration_descriptors
        .contains_key("main"));
}

#[test]
fn callable_identities_support_static_methods_and_type_constructors() {
    let modules = descriptor_project(
        r#"
from fixture.contract import option

class Builder:
    value: int

class Callbacks:
    @staticmethod
    def normalize(own value: str) -> str:
        return value

class Model:
    constructor: int = option(None, [], Builder)
    callback: int = option(None, [], Callbacks.normalize)
"#,
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let project = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("checked constructors and static methods should be callable identities");
    let uses = project
        .external_defs
        .declaration_descriptors
        .get("main")
        .expect("callable descriptors exist");
    assert_eq!(uses.len(), 2);
    let identities = uses
        .iter()
        .map(|descriptor| {
            let StaticProgramValue::Record(fields) = &descriptor.value else {
                panic!("descriptor result should remain a record");
            };
            let value = fields
                .iter()
                .find(|(name, _)| name == "callback")
                .map(|(_, value)| value)
                .expect("callback exists");
            let StaticProgramValue::CallableIdentity(identity) = value else {
                panic!("callback should be sealed");
            };
            identity
        })
        .collect::<Vec<_>>();
    assert_eq!(identities[0].owner.as_deref(), Some("main.Builder"));
    assert_eq!(identities[0].symbol, "__init__");
    assert_eq!(identities[1].owner.as_deref(), Some("main.Callbacks"));
    assert_eq!(identities[1].symbol, "normalize");
    assert_ne!(identities[0].signature, identities[1].signature);
}

#[test]
fn ordinary_class_assignment_keeps_its_existing_diagnostic() {
    let modules = descriptor_project(
        r#"
def ordinary() -> int:
    return 1

class Invalid:
    value = ordinary()
"#,
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let errors = match collect_project_hir_modules(&modules, stdlib_defs) {
        Ok(_) => panic!("ordinary class assignments remain unsupported"),
        Err(errors) => errors,
    };
    assert!(errors.iter().any(|error| error.code
        == sifr_diagnostics::DiagnosticCode::CLASS_UNSUPPORTED_DECLARATION.code()));
}

#[test]
fn provider_descriptor_union_accepts_assignable_record_members() {
    let mut modules = HashMap::new();
    modules.insert(
        "fixture.union_types".to_string(),
        parse_suite(
            r#"
class FieldIntent:
    required: bool

class TypeIntent:
    limit: int

type Descriptor = FieldIntent | TypeIntent
"#,
        ),
    );
    modules.insert(
        "fixture.union_contract".to_string(),
        parse_suite(
            r#"
from sifr.meta import DeclarationInput, DeclarationPlan
from fixture.union_types import Descriptor, FieldIntent, TypeIntent

@class_adapter_provider("fixture.union_types", "Descriptor")
@const_eval
def adapt(value: DeclarationInput[Descriptor]) -> DeclarationPlan[Descriptor]:
    return DeclarationPlan([], [], None, None, [])

@field_descriptor("fixture.union_contract", "adapt")
def required() -> FieldIntent:
    return FieldIntent(True)

@type_descriptor("fixture.union_contract", "adapt")
def bounded(limit: int) -> TypeIntent:
    return TypeIntent(limit)
"#,
        ),
    );
    modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from typing import Annotated
from fixture.union_contract import bounded, required

class Model:
    value: Annotated[int, bounded(8)] = required()
"#,
        ),
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let project = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("provider union members should remain typed descriptor results");
    let uses = project
        .external_defs
        .declaration_descriptors
        .get("main")
        .expect("union descriptors should be retained");
    assert_eq!(uses.len(), 2);
    assert!(uses.iter().all(|descriptor| {
        descriptor.provider_module == "fixture.union_contract"
            && descriptor.provider_function == "adapt"
    }));
}

#[test]
fn descriptor_return_type_is_checked_against_provider_before_use() {
    let mut modules = descriptor_project("def main():\n    pass\n");
    modules.insert(
        "fixture.contract".to_string(),
        parse_suite(
            r#"
from sifr.meta import DeclarationInput, DeclarationPlan
from fixture.contract_types import ContractDescriptor

@class_adapter_provider("fixture.contract_types", "ContractDescriptor")
@const_eval
def adapt_contract(value: DeclarationInput[ContractDescriptor]) -> DeclarationPlan[ContractDescriptor]:
    return DeclarationPlan([], [], None, None, [])

@field_descriptor("fixture.contract", "adapt_contract")
def wrong() -> int:
    return 1
"#,
        ),
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let errors = match collect_project_hir_modules(&modules, stdlib_defs) {
        Ok(_) => panic!("descriptor return type must match the provider descriptor type"),
        Err(errors) => errors,
    };
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::META_MALFORMED_DECLARATION.code()
            && error.message.contains("return type is not assignable")
    }));
}
