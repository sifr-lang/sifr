use crate::{
    lower_module, HirExpr, HirFunction, HirModule, HirStmt, MutableReceiverTarget, PlaceProjection,
};
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

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::OWN_UNSUPPORTED_MUTABLE_RECEIVER_PLACE)
            && error
                .message
                .contains("before constructor storage initialization")
            && error.message.contains("value")
    }));
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
