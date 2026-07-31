use crate::{
    lower_module, HirExpr, HirFunction, HirModule, HirStmt, MutableReceiverTarget, PlaceProjection,
};
use ruff_text_size::{TextRange, TextSize};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;
use sifr_type_system::ReceiverConvention;

fn lower(source: &str) -> HirModule {
    let parsed = parse_module(source).expect("source should parse");
    lower_module(parsed.suite())
        .expect("source should lower")
        .module
}

fn method<'a>(module: &'a HirModule, class: &str, name: &str) -> &'a HirFunction {
    module
        .classes
        .iter()
        .find(|candidate| candidate.name == class)
        .and_then(|class| {
            class
                .methods
                .iter()
                .find(|candidate| candidate.name == name)
        })
        .expect("method should exist")
}

fn expression_method_call(function: &HirFunction) -> &HirExpr {
    function
        .body
        .iter()
        .find_map(|statement| match statement {
            HirStmt::Expr { expr } => Some(expr),
            _ => None,
        })
        .expect("method should contain an expression statement")
}

fn range_for(source: &str, needle: &str) -> TextRange {
    let start = source.find(needle).expect("needle should occur");
    TextRange::new(
        TextSize::try_from(start).expect("test source offset fits"),
        TextSize::try_from(start + needle.len()).expect("test source offset fits"),
    )
}

#[test]
fn constructor_self_use_requires_materialized_storage() {
    let source = r#"
class Deferred:
    value: int

    def __init__(self, flag: bool):
        if flag:
            self.value = 1
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("constructor should be rejected"),
        Err(errors) => errors,
    };

    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::OWN_UNSUPPORTED_MUTABLE_RECEIVER_PLACE)
                && error
                    .message
                    .contains("before field storage is initialized: self.value")
                && error
                    .primary_range
                    .is_some_and(|range| range.start() == range_for(source, "if flag:").start())
                && matches!(
                    error.args.get("place"),
                    Some(sifr_diagnostics::DiagnosticArg::String(place)) if place == "self"
                )
        }),
        "{errors:?}"
    );
}

#[test]
fn constructor_missing_parent_diagnostic_is_source_facing_and_statement_anchored() {
    let source = r#"
class Base:
    value: int

class Child(Base):
    own: int

    def __init__(self, own: int):
        self.own = own
        self.own += 1
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("constructor should be rejected"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_UNSUPPORTED_MUTABLE_RECEIVER_PLACE)
            && error.message.contains("call super().__init__(...) first")
            && !error.message.contains("__sifr_parent")
            && error.primary_range == Some(range_for(source, "self.own += 1"))
            && matches!(
                error.args.get("place"),
                Some(sifr_diagnostics::DiagnosticArg::String(place)) if place == "self"
            )
    }));
}

#[test]
fn constructor_same_named_parameter_remains_seeded_before_explicit_assignment() {
    let source = r#"
class Holder:
    a: int
    items: list[int]

    def __init__(self, a: int):
        self.items = []
        self.items.append(1)
        self.a = a
"#;
    let parsed = parse_module(source).expect("source should parse");
    if let Err(errors) = lower_module(parsed.suite()) {
        panic!("same-named constructor parameter should seed storage: {errors:?}");
    }
}

#[test]
fn constructor_mid_initialization_read_names_fields_and_statement() {
    let source = r#"
class Pair:
    a: int
    b: int

    def __init__(self):
        self.b = self.a + 1
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("constructor should be rejected"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_UNSUPPORTED_MUTABLE_RECEIVER_PLACE)
            && error.message.contains("self.a, self.b")
            && error.primary_range == Some(range_for(source, "self.b = self.a + 1"))
            && !error.message.contains("__sifr_")
    }));
}

#[test]
fn constructor_repeated_field_before_complete_storage_is_rejected() {
    let source = r#"
class Counter:
    count: int
    items: list[int]

    def __init__(self, n: int):
        self.count = 0
        self.count = n
        self.items = []
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("constructor should be rejected"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_UNSUPPORTED_MUTABLE_RECEIVER_PLACE)
            && error.message.contains("self.items")
            && error.primary_range == Some(range_for(source, "self.count = n"))
    }));
}

#[test]
fn same_call_place_conflict_populates_canonical_binding_argument() {
    let source = r#"
class Helper:
    items: list[int]

    def absorb(self, mut other: Helper) -> None:
        self.items.append(1)
        other.items.append(2)

class Owner:
    helper: Helper

def conflict(mut owner: Owner) -> None:
    owner.helper.absorb(owner.helper)
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("overlapping receiver and argument should be rejected"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
            && matches!(
                error.args.get("binding"),
                Some(sifr_diagnostics::DiagnosticArg::String(binding))
                    if binding == "owner.helper"
            )
    }));
}

#[test]
fn unsupported_callable_field_footprint_rejects_same_root_overlap() {
    let source = r#"
class Owner:
    callback: Callable[[int], int]

def touch(mut owner: Owner, callback: Callable[[int], int]) -> None:
    pass

def conflict(mut owner: Owner) -> None:
    touch(owner, owner.callback)
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("callable field under a mutable root should overlap"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
            && error.primary_range == Some(range_for(source, "owner.callback"))
    }));
}

#[test]
fn unsupported_recursive_field_footprint_rejects_same_root_overlap() {
    let source = r#"
class Node:
    next: Node | None

    def absorb(mut self, other: Node | None) -> None:
        self.next = other

def conflict(mut node: Node) -> None:
    node.absorb(node.next)
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("recursive field under a mutable root should overlap"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
            && error.primary_range == Some(range_for(source, "node.next"))
    }));
}

#[test]
fn unsupported_callable_field_footprint_accepts_disjoint_sibling_place() {
    let source = r#"
class Inner:
    value: int

class Owner:
    inner: Inner
    callback: Callable[[int], int]

def take(mut inner: Inner, callback: Callable[[int], int]) -> None:
    pass

def accepted(mut owner: Owner) -> None:
    take(owner.inner, owner.callback)
"#;
    let parsed = parse_module(source).expect("source should parse");
    lower_module(parsed.suite()).expect("disjoint callable sibling should not overlap");
}

#[test]
fn invoked_callable_field_footprint_rejects_receiver_prefix_overlap() {
    let source = r#"
class Owner:
    value: int
    callback: Callable[[int], int]

    def update(self, value: int) -> int:
        self.value = value
        return self.value

    def conflict(self) -> int:
        return self.update(self.callback(2))
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("invoked callable field under the mutable receiver should overlap"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
            && error.primary_range == Some(range_for(source, "self.callback(2)"))
    }));
}

#[test]
fn invoked_callable_field_footprint_accepts_disjoint_sibling_place() {
    let source = r#"
class Helper:
    value: int

    def update(self, value: int) -> int:
        self.value = value
        return self.value

class Owner:
    helper: Helper
    callback: Callable[[int], int]

    def accepted(self) -> int:
        return self.helper.update(self.callback(2))
"#;
    let parsed = parse_module(source).expect("source should parse");
    lower_module(parsed.suite()).expect("invoked callable sibling should not overlap");
}

#[test]
fn actual_method_shadowing_callable_field_stays_conservative() {
    let source = r#"
class Helper:
    value: int

    def update(self, value: int) -> int:
        self.value = value
        return self.value

class Base:
    value: int

    def run(self, value: int) -> int:
        self.value = value
        return self.value

class Child(Base):
    helper: Helper
    run: Callable[[int], int]

    def conflict(self) -> int:
        return self.helper.update(self.run(2))
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("actual method call should retain the conservative object footprint"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
            && error.primary_range == Some(range_for(source, "self.run(2)"))
    }));
}

#[test]
fn callable_field_on_dynamic_base_keeps_conservative_object_footprint() {
    let source = r#"
class Inner:
    callback: Callable[[int], int]

class Owner:
    value: int
    inner: Inner

    def pick(self) -> Inner:
        return self.inner

    def update(self, value: int) -> int:
        self.value = value
        return self.value

    def conflict(self) -> int:
        return self.update(self.pick().callback(2))
"#;
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("dynamic callable-field base should conservatively overlap"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW)
            && error.primary_range == Some(range_for(source, "self.pick().callback(2)"))
    }));
}

#[test]
fn unsupported_recursive_field_footprint_accepts_disjoint_sibling_place() {
    let source = r#"
class Inner:
    value: int

class Node:
    inner: Inner
    next: Node | None

def take(mut inner: Inner, next: Node | None) -> None:
    pass

def accepted(mut node: Node) -> None:
    take(node.inner, node.next)
"#;
    let parsed = parse_module(source).expect("source should parse");
    lower_module(parsed.suite()).expect("disjoint recursive sibling should not overlap");
}

#[test]
fn receiver_inference_closes_transitive_delegation_and_attaches_call_metadata() {
    let source = r#"
class Leaf:
    value: int

    def bump(self) -> None:
        self.value += 1

class Middle:
    leaf: Leaf

    def bump(self) -> None:
        self.leaf.bump()

class Root:
    middle: Middle

    def bump(self) -> None:
        self.middle.bump()
"#;
    let module = lower(source);

    for class in ["Leaf", "Middle", "Root"] {
        assert_eq!(
            method(&module, class, "bump").receiver,
            Some(ReceiverConvention::MutableBorrow)
        );
    }
    let HirExpr::MethodCall {
        receiver_convention,
        source: Some(source),
        ..
    } = expression_method_call(method(&module, "Root", "bump"))
    else {
        panic!("delegating expression should be an annotated method call");
    };
    assert_eq!(
        *receiver_convention,
        Some(ReceiverConvention::MutableBorrow)
    );
    assert!(!source.call_range.is_empty());
    assert!(!source.receiver_range.is_empty());
}

#[test]
fn inherited_receiver_metadata_uses_declaring_method_origin() {
    let source = r#"
class Base:
    value: int

    def bump(self) -> None:
        self.value += 1

class Child(Base):
    pass

class Owner:
    child: Child

    def bump(self) -> None:
        self.child.bump()
"#;
    let module = lower(source);

    assert_eq!(
        method(&module, "Base", "bump").receiver,
        Some(ReceiverConvention::MutableBorrow)
    );
    assert_eq!(
        method(&module, "Owner", "bump").receiver,
        Some(ReceiverConvention::MutableBorrow)
    );
}

#[test]
fn shared_calls_do_not_spuriously_mutate_enclosing_receiver() {
    let source = r#"
class Leaf:
    value: int

    def read(self) -> int:
        return self.value

class Owner:
    leaf: Leaf

    def read(self) -> int:
        return self.leaf.read()
"#;
    let module = lower(source);

    assert_eq!(
        method(&module, "Leaf", "read").receiver,
        Some(ReceiverConvention::SharedBorrow)
    );
    assert_eq!(
        method(&module, "Owner", "read").receiver,
        Some(ReceiverConvention::SharedBorrow)
    );
}

#[test]
fn generic_receiver_specializations_keep_inferred_conventions() {
    let source = r#"
class Cell[T]:
    value: T
    updates: int

    def bump(self) -> None:
        self.updates += 1

class Owner[T]:
    cell: Cell[T]

    def bump(self) -> None:
        self.cell.bump()

def mutate(mut owner: Owner[int]) -> None:
    owner.bump()
"#;
    let module = lower(source);

    assert_eq!(
        method(&module, "Cell", "bump").receiver,
        Some(ReceiverConvention::MutableBorrow)
    );
    assert_eq!(
        method(&module, "Owner", "bump").receiver,
        Some(ReceiverConvention::MutableBorrow)
    );
    let HirExpr::MethodCall {
        receiver_convention,
        ..
    } = expression_method_call(&module.functions[0])
    else {
        panic!("generic call should remain a method call");
    };
    assert_eq!(
        *receiver_convention,
        Some(ReceiverConvention::MutableBorrow)
    );
}

#[test]
fn protocol_calls_use_declared_receiver_metadata() {
    let source = r#"
class Mutable(Protocol):
    def bump(mut self) -> None:
        pass

class Counter:
    value: int

    def bump(self) -> None:
        self.value += 1

def invoke(own mut entity: Mutable) -> None:
    entity.bump()
"#;
    let module = lower(source);
    let counter = module
        .classes
        .iter()
        .find(|class| class.name == "Counter")
        .expect("counter class should exist");
    assert!(counter
        .implements_protocols
        .contains(&"Mutable".to_string()));
    let HirExpr::MethodCall {
        receiver_convention,
        source: Some(_),
        ..
    } = expression_method_call(&module.functions[0])
    else {
        panic!("protocol call should be fully annotated");
    };
    assert_eq!(
        *receiver_convention,
        Some(ReceiverConvention::MutableBorrow)
    );
}

#[test]
fn inferred_mutable_receiver_rejects_shared_protocol_conformance() {
    let source = r#"
class Shared(Protocol):
    def update(self) -> None:
        pass

class MutableImplementation:
    value: int

    def update(self) -> None:
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
    assert!(errors.iter().any(|error| {
        error.code == Some(sifr_diagnostics::DiagnosticCode::PROTO_RECEIVER_CONVENTION_MISMATCH)
    }));
}

#[test]
fn fixed_trait_receiver_rejects_transitive_receiver_mutation() {
    let source = r#"
class Counter:
    value: int

    def bump(self) -> None:
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
    assert!(errors.iter().any(|error| {
        error.code == Some(sifr_diagnostics::DiagnosticCode::PROTO_FIXED_RECEIVER_MUTATION)
    }));
}

#[test]
fn builtin_method_calls_carry_canonical_receiver_conventions_and_source_ranges() {
    let source = r#"
def mutate(mut values: list[int], text: str) -> None:
    values.append(1)
    text.upper()
"#;
    let module = lower(source);
    let function = &module.functions[0];
    let calls: Vec<_> = function
        .body
        .iter()
        .filter_map(|statement| match statement {
            HirStmt::Expr {
                expr:
                    call @ HirExpr::MethodCall {
                        receiver_convention,
                        source,
                        ..
                    },
            } => Some((call, receiver_convention, source)),
            _ => None,
        })
        .collect();

    assert_eq!(calls.len(), 2);
    assert_eq!(*calls[0].1, Some(ReceiverConvention::MutableBorrow));
    assert_eq!(*calls[1].1, Some(ReceiverConvention::SharedBorrow));
    assert!(calls.iter().all(|(_, _, source)| source.is_some()));
}

#[test]
fn keyword_and_default_method_arguments_keep_ranges_aligned_with_hir_args() {
    let source = r#"
class Helper:
    def absorb(mut self, source: list[int], limit: int = 1) -> None:
        pass

def invoke(own mut helper: Helper, stock: list[int]) -> None:
    helper.absorb(source=stock)
"#;
    let module = lower(source);
    let HirExpr::MethodCall {
        args,
        source: Some(call_source),
        ..
    } = expression_method_call(&module.functions[0])
    else {
        panic!("keyword method call should carry source metadata");
    };

    assert_eq!(args.len(), 2);
    assert_eq!(call_source.arg_ranges.len(), args.len());
    assert_eq!(u32::from(call_source.arg_ranges[0].len()), 5);
    assert!(!call_source.arg_ranges[1].is_empty());
}

#[test]
fn declaration_first_consuming_method_and_call_keep_owned_receiver_metadata() {
    let source = r#"
class BridgeError(Error):
    message: str

class Plain:
    @rust(bridge.consume, panic=trusted_no_panic)
    def consume(own self) -> Result[str, BridgeError]:
        ...

def invoke(own value: Plain) -> Result[str, BridgeError]:
    return value.consume()
"#;
    let module = lower(source);
    assert_eq!(
        method(&module, "Plain", "consume").receiver,
        Some(ReceiverConvention::Owned)
    );
    let HirStmt::Return {
        value:
            Some(HirExpr::MethodCall {
                receiver_convention,
                source: Some(call_source),
                args,
                ..
            }),
    } = &module.functions[0].body[0]
    else {
        panic!("consuming call should be retained in the return expression");
    };
    assert_eq!(*receiver_convention, Some(ReceiverConvention::Owned));
    assert_eq!(call_source.arg_ranges.len(), args.len());
}

#[test]
fn instance_syntax_static_method_call_does_not_trip_receiver_invariant() {
    let source = r#"
class Utility:
    @staticmethod
    def ping() -> None:
        pass

def invoke() -> None:
    utility = Utility()
    utility.ping()
"#;
    let module = lower(source);
    let HirExpr::MethodCall {
        receiver_convention,
        source: Some(_),
        ..
    } = expression_method_call(&module.functions[0])
    else {
        panic!("instance-syntax static call should remain annotated");
    };
    assert_eq!(*receiver_convention, Some(ReceiverConvention::SharedBorrow));
}

#[test]
fn nested_receiver_and_regular_mut_argument_share_checked_place_metadata() {
    let source = r#"
class Leaf:
    value: int

    def bump(self) -> None:
        self.value += 1

class Mid:
    leaf: Leaf

class Root:
    mid: Mid

def bump_leaf(mut leaf: Leaf) -> None:
    leaf.bump()

def mutate(mut root: Root) -> None:
    root.mid.leaf.bump()
    bump_leaf(root.mid.leaf)
"#;
    let module = lower(source);
    let function = &module.functions[1];
    let HirStmt::Expr {
        expr:
            HirExpr::MethodCall {
                object,
                receiver_target: Some(MutableReceiverTarget::Place(receiver_place)),
                ..
            },
    } = &function.body[0]
    else {
        panic!("nested mutable receiver should carry a checked place");
    };
    let HirExpr::FieldAccess {
        object: middle_object,
        ..
    } = object.as_ref()
    else {
        panic!("receiver should have a leaf projection");
    };
    let HirExpr::FieldAccess {
        object: root_object,
        ..
    } = middle_object.as_ref()
    else {
        panic!("receiver should have a middle projection");
    };
    let HirExpr::Name {
        binding_id: Some(root_id),
        ..
    } = root_object.as_ref()
    else {
        panic!("nested receiver should retain its root binding id");
    };
    assert_eq!(receiver_place.root, *root_id);
    assert_eq!(receiver_place.projections.len(), 2);
    let projection_fields = receiver_place
        .projections
        .iter()
        .map(|projection| match projection {
            PlaceProjection::Field(identity) => identity.field.as_str(),
        })
        .collect::<Vec<_>>();
    assert_eq!(projection_fields, ["mid", "leaf"]);

    let HirStmt::Expr {
        expr: HirExpr::Call {
            mutable_arg_places, ..
        },
    } = &function.body[1]
    else {
        panic!("regular mutable call should carry checked argument places");
    };
    let Some(sifr_ir::MutableArgumentTarget::Place(argument_place)) =
        mutable_arg_places[0].as_ref()
    else {
        panic!("mutable field argument should carry a checked place");
    };
    assert_eq!(argument_place, receiver_place);
}
