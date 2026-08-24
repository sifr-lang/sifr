use super::support::parse_suite;
use crate::{collect_project_hir_modules, compile_stdlib};
use sifr_ir::{MethodKind, StaticMethodSlotContext, StaticProgramValue};
use sifr_python_ast::Suite;
use sifr_type_system::ReceiverConvention;
use std::collections::HashMap;

const TYPES: &str = r#"
class HandlerDescriptor:
    tag: str
"#;

const CONTRACT: &str = r#"
from sifr.meta import CallableIdentity, ConstSpecializationOutcome, DeclarationInput, DeclarationPlan, PlannedField, PlannedHandler, ShapeInput
from fixture.handler_types import HandlerDescriptor

@class_adapter_provider("fixture.handler_types", "HandlerDescriptor")
@const_eval
def adapt_handlers(declaration: DeclarationInput[HandlerDescriptor]) -> DeclarationPlan[HandlerDescriptor]:
    fields: list[PlannedField] = []
    handlers: list[PlannedHandler[HandlerDescriptor]] = []
    for item in declaration.declaration.items:
        declared_type = item.declared_type
        if item.kind == "field" and declared_type is not None:
            fields.append(PlannedField(item.identity, declared_type))
    for descriptor in declaration.descriptors:
        target = descriptor.target_callable
        if descriptor.target_kind == "method" and target is not None:
            handlers.append(PlannedHandler(target, descriptor.value, descriptor.origin))
    return DeclarationPlan(fields, [], "fixture.handlers", "specialize", [], handlers)

@class_adapter_marker("fixture.handlers", "adapt_handlers")
class HandlerContract:
    pass

@method_descriptor("fixture.handlers", "adapt_handlers")
def handler(tag: str) -> HandlerDescriptor:
    return HandlerDescriptor(tag)

class HandlerProgram:
    sifr_method_slots: list[CallableIdentity]

@const_eval
def specialize(shape: ShapeInput[HandlerDescriptor]) -> ConstSpecializationOutcome[HandlerProgram, HandlerDescriptor]:
    slots: list[CallableIdentity] = []
    for method in shape.root.methods:
        target = method.target
        descriptor = method.descriptor
        if target is not None and descriptor is not None:
            slots.append(target)
    return ConstSpecializationOutcome("produced", HandlerProgram(slots), [])
"#;

fn project(main: &str) -> HashMap<String, Suite> {
    HashMap::from([
        ("fixture.handler_types".to_string(), parse_suite(TYPES)),
        ("fixture.handlers".to_string(), parse_suite(CONTRACT)),
        ("main".to_string(), parse_suite(main)),
    ])
}

fn compile_errors(main: &str) -> Vec<sifr_diagnostics::RenderedDiagnostic> {
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    match collect_project_hir_modules(&project(main), stdlib_defs) {
        Ok(_) => panic!("handler fixture must fail"),
        Err(errors) => errors,
    }
}

#[test]
fn handler_descriptors_select_checked_receivers_in_declaration_order() {
    let modules = project(
        r#"
from fixture.handlers import HandlerContract, handler

class Model(HandlerContract):
    value: str

    @staticmethod
    @handler("static")
    def parse_static(own value: str) -> str:
        return value

    @handler("class")
    @classmethod
    def parse_class(cls, own value: str) -> Result[str, ValueError]:
        return value

    @handler("shared")
    def inspect(self) -> str:
        return self.value

    @handler("mutable")
    def normalize(mut self) -> str:
        return self.value

    @handler("owned")
    def finish(own self) -> Self:
        return self
"#,
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let compiled = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("non-Pydantic handler pipeline should compile");
    let selection = compiled
        .external_defs
        .class_adapter_selections
        .get("main")
        .and_then(|classes| classes.get("Model"))
        .expect("handler adapter selection exists");
    assert_eq!(
        selection
            .handler_plans
            .iter()
            .map(|handler| handler.callable.symbol.as_str())
            .collect::<Vec<_>>(),
        [
            "parse_static",
            "parse_class",
            "inspect",
            "normalize",
            "finish"
        ]
    );
    assert_eq!(
        selection
            .handler_plans
            .iter()
            .map(|handler| handler.declaration_order)
            .collect::<Vec<_>>(),
        [0, 1, 2, 3, 4]
    );

    let output = compiled
        .external_defs
        .specialization_outputs
        .get("main")
        .and_then(|outputs| outputs.first())
        .expect("handler specialization output exists");
    assert_eq!(
        output
            .method_slots
            .iter()
            .map(|slot| slot.name.as_str())
            .collect::<Vec<_>>(),
        [
            "parse_static",
            "parse_class",
            "inspect",
            "normalize",
            "finish"
        ]
    );
    assert_eq!(output.method_slots[0].method_kind, MethodKind::StaticMethod);
    assert!(!output.method_slots[0].is_fallible);
    assert_eq!(output.method_slots[1].method_kind, MethodKind::ClassMethod);
    assert!(output.method_slots[1].is_fallible);
    assert_eq!(
        output.method_slots[2].receiver,
        Some(ReceiverConvention::SharedBorrow)
    );
    assert_eq!(
        output.method_slots[3].receiver,
        Some(ReceiverConvention::MutableBorrow)
    );
    assert_eq!(
        output.method_slots[4].receiver,
        Some(ReceiverConvention::Owned)
    );
    assert_eq!(
        output.method_slot_context,
        Some(StaticMethodSlotContext::None)
    );
    assert!(output.method_slots.iter().all(|slot| {
        slot.descriptor_value
            .as_ref()
            .is_some_and(|value| matches!(value, StaticProgramValue::Record(_)))
            && slot.descriptor_origin.is_some()
            && slot.descriptor_range.is_some()
    }));
}

#[test]
fn classmethod_descriptor_must_be_the_outer_adjacent_decorator() {
    let errors = compile_errors(
        r#"
from fixture.handlers import HandlerContract, handler

class Invalid(HandlerContract):
    value: str

    @classmethod
    @handler("wrong-order")
    def parse(cls, own value: str) -> str:
        return value
"#,
    );
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::META_MALFORMED_DECLARATION.code()
            && error
                .message
                .contains("outer decorator with @classmethod directly above")
    }));
}

#[test]
fn staticmethod_descriptor_must_be_the_inner_adjacent_decorator() {
    let errors = compile_errors(
        r#"
from fixture.handlers import HandlerContract, handler

class Invalid(HandlerContract):
    value: str

    @handler("wrong-order")
    @staticmethod
    def parse(own value: str) -> str:
        return value
"#,
    );
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::META_MALFORMED_DECLARATION.code()
            && error
                .message
                .contains("directly above the method with @staticmethod as the outer decorator")
    }));
}

#[test]
fn owned_handler_requires_exact_self_output_at_the_descriptor() {
    let errors = compile_errors(
        r#"
from fixture.handlers import HandlerContract, handler

class Invalid(HandlerContract):
    value: str

    @handler("owned")
    def finish(own self) -> str:
        return self.value
"#,
    );
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::RUST_SLOT_SIGNATURE.code()
            && error.message.contains("must return exactly Self")
    }));
}

#[test]
fn constructors_cannot_be_selected_as_handlers() {
    let errors = compile_errors(
        r#"
from fixture.handlers import HandlerContract, handler

class Invalid(HandlerContract):
    value: str

    @handler("constructor")
    def __init__(self, own value: str):
        self.value = value
"#,
    );
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::RUST_SLOT_METHOD.code()
            && error.message.contains("cannot name a constructor")
    }));
}

#[test]
fn inherited_handlers_precede_local_handlers_deterministically() {
    let modules = project(
        r#"
from fixture.handlers import HandlerContract, handler

class Parent(HandlerContract):
    parent_value: str

    @staticmethod
    @handler("parent")
    def parent_handler(own value: str) -> str:
        return value

class Child(Parent):
    child_value: str

    @staticmethod
    @handler("child")
    def child_handler(own value: str) -> str:
        return value
"#,
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let compiled = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("inherited handlers should compile");
    let child = compiled
        .external_defs
        .class_adapter_selections
        .get("main")
        .and_then(|classes| classes.get("Child"))
        .expect("child selection exists");
    assert_eq!(
        child
            .handler_plans
            .iter()
            .map(|handler| handler.callable.symbol.as_str())
            .collect::<Vec<_>>(),
        ["parent_handler", "child_handler"]
    );
    assert_eq!(
        child
            .handler_plans
            .iter()
            .map(|handler| handler.declaration_order)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    let output = compiled
        .external_defs
        .specialization_outputs
        .get("main")
        .and_then(|outputs| outputs.iter().find(|output| output.owner == "Child"))
        .expect("child specialization output exists");
    assert_eq!(
        output
            .method_slots
            .iter()
            .map(|slot| slot.name.as_str())
            .collect::<Vec<_>>(),
        ["parent_handler", "child_handler"]
    );
}

#[test]
fn imported_inherited_handler_keeps_its_checked_owner() {
    let mut modules = project(
        r#"
from base import Parent
from fixture.handlers import handler

class Child(Parent):
    child_value: str

    @staticmethod
    @handler("child")
    def child_handler(own value: str) -> str:
        return value
"#,
    );
    modules.insert(
        "base".to_string(),
        parse_suite(
            r#"
from fixture.handlers import HandlerContract, handler

class Parent(HandlerContract):
    parent_value: str

    @staticmethod
    @handler("parent")
    def parent_handler(own value: str) -> str:
        return value
"#,
        ),
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let compiled = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("imported inherited handler should compile");
    let output = compiled
        .external_defs
        .specialization_outputs
        .get("main")
        .and_then(|outputs| outputs.iter().find(|output| output.owner == "Child"))
        .expect("child specialization output exists");
    assert_eq!(
        output
            .method_slots
            .iter()
            .map(|slot| (slot.owner_identity.as_str(), slot.name.as_str()))
            .collect::<Vec<_>>(),
        [
            ("base.Parent", "parent_handler"),
            ("main.Child", "child_handler")
        ]
    );
}

#[test]
fn handler_context_must_be_borrowed() {
    let errors = compile_errors(
        r#"
from fixture.handlers import HandlerContract, handler

class AppContext:
    calls: int

class Invalid(HandlerContract):
    value: str

    @staticmethod
    @handler("context")
    def parse(own value: str, own context: AppContext) -> str:
        return value
"#,
    );
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::RUST_SLOT_CONTEXT.code()
            && error
                .message
                .contains("context must be an immutable or mutable borrow")
    }));
}

#[test]
fn handler_target_changes_static_program_identity() {
    fn identity(method_name: &str) -> [u8; 32] {
        let main = format!(
            r#"
from fixture.handlers import HandlerContract, handler

class Model(HandlerContract):
    value: str

    @staticmethod
    @handler("selected")
    def {method_name}(own value: str) -> str:
        return value
"#
        );
        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let compiled = collect_project_hir_modules(&project(&main), stdlib_defs)
            .expect("handler identity fixture compiles");
        compiled
            .external_defs
            .specialization_outputs
            .get("main")
            .and_then(|outputs| outputs.first())
            .expect("specialization output exists")
            .program_identity
    }

    assert_ne!(identity("first"), identity("second"));
}

#[test]
fn owned_handler_receiver_moves_exactly_once() {
    let errors = compile_errors(
        r#"
from fixture.handlers import HandlerContract, handler

class Model(HandlerContract):
    value: str

    @handler("owned")
    def finish(own self) -> Self:
        return self

def invalid_reuse() -> str:
    model: Model = Model("value")
    moved: Model = model.finish()
    return model.value + moved.value
"#,
    );
    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::OWN_USE_AFTER_MOVE.code()
    }));
}

#[test]
fn self_annotation_keeps_the_current_generic_specialization() {
    let modules = project(
        r#"
class Box[T]:
    value: T

    def keep(own self) -> Self:
        return self
"#,
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let compiled = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("generic Self annotation should compile");
    let method = compiled
        .hir_modules
        .get("main")
        .and_then(|module| module.classes.iter().find(|class| class.name == "Box"))
        .and_then(|class| class.methods.iter().find(|method| method.name == "keep"))
        .expect("generic method exists");
    let sifr_type_system::Type::Class {
        name, type_args, ..
    } = method.return_type.resolve_alias()
    else {
        panic!("Self should resolve to the current class specialization");
    };
    assert_eq!(name, "Box");
    assert_eq!(
        type_args,
        &[sifr_type_system::Type::TypeVar("T".to_string())]
    );
    assert_eq!(method.receiver, Some(ReceiverConvention::Owned));
}

#[test]
fn self_annotation_is_rejected_in_static_and_class_methods() {
    let errors = compile_errors(
        r#"
class Invalid:
    @staticmethod
    def static_value(value: Self) -> Self:
        return value

    @classmethod
    def class_value(cls, value: Self) -> Self:
        return value
"#,
    );
    let invalid_self_count = errors
        .iter()
        .filter(|error| {
            error.code == sifr_diagnostics::DiagnosticCode::TYPE_INVALID_ANNOTATION.code()
                && error
                    .message
                    .contains("Self is valid only in an ordinary class method annotation")
        })
        .count();
    assert!(invalid_self_count >= 4, "errors: {errors:#?}");
}
