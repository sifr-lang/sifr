use super::support::parse_suite;
use crate::project::ProjectLowering;
use crate::{collect_project_hir_modules, compile_stdlib};
use sifr_ir::{AdapterFieldDefault, StaticProgramValue};
use sifr_type_system::Type;
use std::collections::HashMap;

const CONTRACT: &str = r#"
from sifr.meta import CallableIdentity, DeclarationInput, DeclarationPlan, PlannedField, PlannedIssue, PlannedMetadata, StaticValue

class DefaultDescriptor:
    default_kind: str
    default_value: StaticValue | None
    default_factory: CallableIdentity | None

class NestedDescriptorValue:
    value: str

@class_adapter_provider("fixture.defaults", "DefaultDescriptor")
@const_eval
def adapt_defaults(declaration: DeclarationInput[DefaultDescriptor]) -> DeclarationPlan[DefaultDescriptor]:
    fields: list[PlannedField] = []
    metadata: list[PlannedMetadata[DefaultDescriptor]] = []
    issues: list[PlannedIssue[DefaultDescriptor]] = []
    for item in declaration.declaration.items:
        if item.kind == "field":
            declared_type: str | None = item.declared_type
            if declared_type is not None:
                identity: str = item.identity
                default_kind: str = item.default_kind
                default_value: StaticValue | None = item.default_value
                default_factory: CallableIdentity | None = None
                for descriptor in declaration.descriptors:
                    if descriptor.target_identity == identity:
                        default_kind = descriptor.value.default_kind
                        default_value = descriptor.value.default_value
                        default_factory = descriptor.value.default_factory
                fields.append(PlannedField(identity, declared_type, default_kind, default_value, default_factory, "package"))
        if item.kind == "method" and item.name == "inherited_method":
            if item.identity != "main.Parent.inherited_method":
                raise ValueError("inherited method identity was not preserved")
    return DeclarationPlan(fields, metadata, None, None, issues)

@class_adapter_marker("fixture.defaults", "adapt_defaults")
class Contract:
    pass

@field_descriptor("fixture.defaults", "adapt_defaults")
def contract_field(
    default: StaticValue | None = None,
    default_factory: CallableIdentity | None = None,
) -> DefaultDescriptor:
    if default_factory is not None:
        return DefaultDescriptor("factory", None, default_factory)
    if default is not None:
        return DefaultDescriptor("const", default, None)
    return DefaultDescriptor("required", None, None)

@const_eval
def nested_value(value: str) -> NestedDescriptorValue:
    return NestedDescriptorValue(value)

@field_descriptor("fixture.defaults", "adapt_defaults")
def nested_contract_field(
    values: list[str | NestedDescriptorValue],
) -> DefaultDescriptor:
    if len(values) == 0:
        raise ValueError("nested descriptor values are required")
    return DefaultDescriptor("required", None, None)
"#;

fn compile(main: &str) -> ProjectLowering {
    compile_with_contract(main, CONTRACT)
}

fn compile_with_contract(main: &str, contract: &str) -> ProjectLowering {
    let modules = HashMap::from([
        ("fixture.defaults".to_string(), parse_suite(contract)),
        ("main".to_string(), parse_suite(main)),
    ]);
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    collect_project_hir_modules(&modules, stdlib_defs)
        .expect("adapted defaults project should compile")
}

#[test]
fn required_const_and_factory_defaults_finalize_constructor_parameters() {
    let compiled = compile(
        r#"
from fixture.defaults import Contract, contract_field

class Model(Contract):
    value: int
    enabled: bool = contract_field(default=True)
    tags: list[str] = contract_field(default_factory=list)
    count: int = 3
"#,
    );
    let defaults = compiled
        .external_defs
        .function_defaults
        .get("main")
        .and_then(|functions| functions.get("Model"))
        .expect("normalized constructor defaults are exported");
    assert_eq!(defaults.len(), 3);
    assert_eq!(defaults[0].0, 1);
    assert_eq!(defaults[1].0, 2);
    assert_eq!(defaults[2].0, 3);

    let selection = compiled
        .external_defs
        .class_adapter_selections
        .get("main")
        .and_then(|classes| classes.get("Model"))
        .expect("applied field plan is retained");
    assert!(matches!(
        selection.field_plans[0].default,
        AdapterFieldDefault::Required
    ));
    assert!(matches!(
        selection.field_plans[1].default,
        AdapterFieldDefault::Const(StaticProgramValue::Bool(true))
    ));
    assert!(matches!(
        selection.field_plans[2].default,
        AdapterFieldDefault::Factory(_)
    ));
    assert!(matches!(
        selection.field_plans[3].default,
        AdapterFieldDefault::Const(StaticProgramValue::Integer(ref value)) if value == "3"
    ));
    assert_ne!(selection.adapter_invocation_identity, [0; 32]);
    assert_ne!(selection.post_adapter_identity, [0; 32]);
}

#[test]
fn provisional_descriptor_defaults_do_not_reject_later_required_fields() {
    let compiled = compile(
        r#"
from fixture.defaults import Contract, contract_field

class Model(Contract):
    descriptor_required: int = contract_field()
    plain_required: str
"#,
    );
    let selection = compiled
        .external_defs
        .class_adapter_selections
        .get("main")
        .and_then(|classes| classes.get("Model"))
        .expect("adapted model is finalized");
    assert!(selection
        .field_plans
        .iter()
        .all(|field| matches!(field.default, AdapterFieldDefault::Required)));
}

#[test]
fn descriptor_arguments_evaluate_nested_imported_const_calls() {
    let compiled = compile(
        r#"
from fixture.defaults import Contract, nested_contract_field, nested_value

class Model(Contract):
    value: int = nested_contract_field([nested_value("primary"), "fallback"])
"#,
    );
    let selection = compiled
        .external_defs
        .class_adapter_selections
        .get("main")
        .and_then(|classes| classes.get("Model"))
        .expect("nested const descriptor is applied");
    assert!(matches!(
        selection.field_plans[0].default,
        AdapterFieldDefault::Required
    ));
}

#[test]
fn descriptor_arguments_evaluate_reexported_const_calls() {
    let modules = HashMap::from([
        ("fixture.defaults".to_string(), parse_suite(CONTRACT)),
        (
            "fixture.facade".to_string(),
            parse_suite(
                r#"
from fixture.defaults import Contract, nested_contract_field, nested_value
"#,
            ),
        ),
        (
            "main".to_string(),
            parse_suite(
                r#"
from fixture.facade import Contract, nested_contract_field, nested_value

class Model(Contract):
    value: int = nested_contract_field([nested_value("primary"), "fallback"])
"#,
            ),
        ),
    ]);
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let compiled = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("re-exported const descriptor arguments should compile");
    let selection = compiled
        .external_defs
        .class_adapter_selections
        .get("main")
        .and_then(|classes| classes.get("Model"))
        .expect("re-exported nested const descriptor is applied");
    assert!(matches!(
        selection.field_plans[0].default,
        AdapterFieldDefault::Required
    ));
}

#[test]
fn adapter_identities_ignore_source_movement_and_distinguish_default_states() {
    fn identities(source: &str, contract: &str) -> ([u8; 32], [u8; 32]) {
        let compiled = compile_with_contract(source, contract);
        let selection = compiled
            .external_defs
            .class_adapter_selections
            .get("main")
            .and_then(|classes| classes.get("Model"))
            .expect("adapter identity is exported");
        (
            selection.adapter_invocation_identity,
            selection.post_adapter_identity,
        )
    }

    let constant_source = r#"
from fixture.defaults import Contract, contract_field
class Model(Contract):
    tags: list[str] = contract_field(default=[])
"#;
    let constant = identities(constant_source, CONTRACT);
    let moved = identities(
        r#"


from fixture.defaults import Contract, contract_field

class Model(Contract):
    tags: list[str] = contract_field(default=[])
"#,
        CONTRACT,
    );
    let factory = identities(
        r#"
from fixture.defaults import Contract, contract_field
class Model(Contract):
    tags: list[str] = contract_field(default_factory=list)
"#,
        CONTRACT,
    );
    assert_eq!(constant, moved);
    assert_eq!(
        constant,
        identities(constant_source, &format!("\n\n{CONTRACT}"))
    );
    assert_ne!(constant, factory);
}

#[test]
fn relevant_adapter_edits_invalidate_invocation_and_post_adapter_identity() {
    let source = r#"
from fixture.defaults import Contract
class Model(Contract):
    value: int
"#;
    let base = compile_with_contract(source, CONTRACT);
    let changed_contract = CONTRACT.replace(
        "fields.append(PlannedField(identity, declared_type, default_kind, default_value, default_factory, \"package\"))",
        "fields.append(PlannedField(identity, declared_type, default_kind, default_value, default_factory, \"changed\"))",
    );
    let changed = compile_with_contract(source, &changed_contract);
    let identities = |compiled: &ProjectLowering| {
        let selection = compiled
            .external_defs
            .class_adapter_selections
            .get("main")
            .and_then(|classes| classes.get("Model"))
            .expect("adapter identity exists");
        (
            selection.adapter_invocation_identity,
            selection.post_adapter_identity,
        )
    };
    assert_ne!(identities(&base), identities(&changed));
}

#[test]
fn inherited_fields_keep_parent_identity_and_concrete_generic_types() {
    let compiled = compile(
        r#"
from fixture.defaults import Contract

class Parent[T](Contract):
    inherited: T

    def inherited_method(self, value: T) -> T:
        return value

class Child(Parent[int]):
    local: str
"#,
    );
    let selection = compiled
        .external_defs
        .class_adapter_selections
        .get("main")
        .and_then(|classes| classes.get("Child"))
        .expect("generic adapted child is selected");
    assert_eq!(selection.data_parent.as_deref(), Some("Parent"));
    assert_eq!(selection.provider_module, "fixture.defaults");
    assert_eq!(selection.field_plans.len(), 2);
    assert_eq!(selection.field_plans[0].identity, "main.Parent.inherited");
    assert_eq!(selection.field_plans[0].declared_type, Type::Int);
    assert_eq!(selection.field_plans[1].identity, "main.Child.local");
}

#[test]
fn imported_concrete_generic_parent_keeps_provider_and_field_identity() {
    let modules = HashMap::from([
        ("fixture.defaults".to_string(), parse_suite(CONTRACT)),
        (
            "base".to_string(),
            parse_suite(
                r#"
from fixture.defaults import Contract
class Parent[T](Contract):
    inherited: T
"#,
            ),
        ),
        (
            "main".to_string(),
            parse_suite(
                r#"
from base import Parent
class Child(Parent[int]):
    local: str
"#,
            ),
        ),
    ]);
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let compiled = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("imported generic adapted parent should compile");
    let selection = compiled
        .external_defs
        .class_adapter_selections
        .get("main")
        .and_then(|classes| classes.get("Child"))
        .expect("imported adapted child is selected");
    assert_eq!(selection.provider_module, "fixture.defaults");
    assert_eq!(selection.field_plans[0].identity, "base.Parent.inherited");
    assert_eq!(selection.field_plans[0].declared_type, Type::Int);
}

#[test]
fn incompatible_inherited_field_reannotation_is_rejected() {
    let modules = HashMap::from([
        ("fixture.defaults".to_string(), parse_suite(CONTRACT)),
        (
            "main".to_string(),
            parse_suite(
                r#"
from fixture.defaults import Contract

class Parent(Contract):
    value: int

class Child(Parent):
    value: str
"#,
            ),
        ),
    ]);
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let errors = match collect_project_hir_modules(&modules, stdlib_defs) {
        Ok(_) => panic!("an incompatible inherited field override must fail"),
        Err(errors) => errors,
    };
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::TYPE_MISMATCH.code()
            && error.message.contains("cannot be re-annotated")
    }));
}

#[test]
fn compatible_inherited_reannotation_preserves_local_default_ordering() {
    let compiled = compile(
        r#"
from fixture.defaults import Contract, contract_field

class Parent(Contract):
    value: int

class Child(Parent):
    value: int
    enabled: bool = contract_field(default=True)
"#,
    );
    let defaults = compiled
        .external_defs
        .function_defaults
        .get("main")
        .and_then(|functions| functions.get("Child"))
        .expect("compatible override keeps the local default on enabled");
    assert_eq!(defaults.len(), 1);
    assert_eq!(defaults[0].0, 1);
}

#[test]
fn reexported_default_descriptors_finalize_constructor_parameters() {
    let modules = HashMap::from([
        (
            "fixture.api".to_string(),
            parse_suite(include_str!(
                "../../../../verification/areas/core_language/fixtures/static_class_adapter/fixture/api.sifr"
            )),
        ),
        (
            "fixture.contract_types".to_string(),
            parse_suite(include_str!(
                "../../../../verification/areas/core_language/fixtures/static_class_adapter/fixture/contract_types.sifr"
            )),
        ),
        (
            "fixture.contract".to_string(),
            parse_suite(include_str!(
                "../../../../verification/areas/core_language/fixtures/static_class_adapter/fixture/contract.sifr"
            )),
        ),
        (
            "fixture.facade".to_string(),
            parse_suite(include_str!(
                "../../../../verification/areas/core_language/fixtures/static_class_adapter/fixture/facade.sifr"
            )),
        ),
        (
            "main".to_string(),
            parse_suite(
                r#"
from fixture.facade import Contract, contract_config, contract_field
class Model(Contract):
    _config = contract_config(True)
    value: int
    enabled: bool = contract_field(default=True)
    tags: list[str] = contract_field(default_factory=list)
"#,
            ),
        ),
    ]);
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let compiled = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("reexported default descriptors should compile");
    let defaults = compiled
        .external_defs
        .function_defaults
        .get("main")
        .and_then(|functions| functions.get("Model"))
        .expect("adapted constructor defaults are exported");
    assert_eq!(
        defaults.iter().map(|(index, _)| *index).collect::<Vec<_>>(),
        vec![1, 2]
    );
}

#[test]
fn checked_default_factories_support_functions_static_methods_and_constructors() {
    let compiled = compile(
        r#"
from fixture.defaults import Contract, contract_field

def make_tags() -> list[str]:
    return []

class Factory:
    @staticmethod
    def make_tags() -> list[str]:
        return []

class Token:
    pass

class Model(Contract):
    first: list[str] = contract_field(default_factory=make_tags)
    second: list[str] = contract_field(default_factory=Factory.make_tags)
    token: Token = contract_field(default_factory=Token)
"#,
    );
    let fields = &compiled
        .external_defs
        .class_adapter_selections
        .get("main")
        .and_then(|classes| classes.get("Model"))
        .expect("factory plans are retained")
        .field_plans;
    assert!(fields
        .iter()
        .all(|field| matches!(field.default, AdapterFieldDefault::Factory(_))));
    let model = compiled
        .hir_modules
        .get("main")
        .and_then(|module| module.classes.iter().find(|class| class.name == "Model"))
        .expect("adapted model HIR exists");
    assert_eq!(model.field_default_identities.len(), 3);
    assert!(model
        .field_default_identities
        .iter()
        .all(|(_, identity)| identity.starts_with("callable[")));
}

#[test]
fn adapted_factory_defaults_retain_static_specialization_output() {
    let modules = HashMap::from([
        (
            "fixture.api".to_string(),
            parse_suite(include_str!(
                "../../../../verification/areas/core_language/fixtures/static_class_adapter/fixture/api.sifr"
            )),
        ),
        (
            "fixture.contract_types".to_string(),
            parse_suite(include_str!(
                "../../../../verification/areas/core_language/fixtures/static_class_adapter/fixture/contract_types.sifr"
            )),
        ),
        (
            "fixture.contract".to_string(),
            parse_suite(include_str!(
                "../../../../verification/areas/core_language/fixtures/static_class_adapter/fixture/contract.sifr"
            )),
        ),
        (
            "models".to_string(),
            parse_suite(
                r#"
from fixture.contract import Contract, contract_field

class Model(Contract):
    value: int64
    tags: list[str] = contract_field(default_factory=list)
"#,
            ),
        ),
        (
            "main".to_string(),
            parse_suite(
                r#"
from fixture.api import ContractError
from models import Model

def build() -> Result[Model, ContractError | RustPanicError]:
    return Model.construct()
"#,
            ),
        ),
    ]);
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let compiled = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("adapted factory model should specialize");
    let outputs = compiled
        .external_defs
        .specialization_outputs
        .get("models")
        .expect("model specializations are exported");
    assert!(outputs.iter().any(|output| output.owner == "Model"));
    let owners = sifr_codegen::structural_static_program_owners(
        compiled
            .hir_modules
            .get("models")
            .expect("models HIR exists"),
    );
    assert!(owners.contains("Model"), "structural owners: {owners:?}");
}

#[test]
fn invalid_constant_and_factory_defaults_are_rejected_by_the_adapter_boundary() {
    for (declaration, expected) in [
        (
            "value: int = contract_field(default=\"bad\")",
            "constant default",
        ),
        (
            "value: int = contract_field(default_factory=needs_arg)",
            "must accept no arguments",
        ),
        (
            "value: int = contract_field(default_factory=wrong_result)",
            "does not return its declared type",
        ),
    ] {
        let source = format!(
            r#"
from fixture.defaults import Contract, contract_field
def needs_arg(value: int) -> int:
    return value
def wrong_result() -> str:
    return "bad"
class Invalid(Contract):
    {declaration}
"#
        );
        let modules = HashMap::from([
            ("fixture.defaults".to_string(), parse_suite(CONTRACT)),
            ("main".to_string(), parse_suite(&source)),
        ]);
        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let errors = match collect_project_hir_modules(&modules, stdlib_defs) {
            Ok(_) => panic!("invalid adapter default should fail"),
            Err(errors) => errors,
        };
        assert!(errors.iter().any(|error| error.message.contains(expected)));
    }
}
