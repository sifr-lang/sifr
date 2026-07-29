use crate::{lower_module, HirExpr, HirFunction, HirModule, HirStmt};
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
fn final_inferred_receivers_refresh_protocol_implementation_membership() {
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
    let module = lower(source);
    let mutable = module
        .classes
        .iter()
        .find(|class| class.name == "MutableImplementation")
        .expect("mutable implementation should exist");
    let shared = module
        .classes
        .iter()
        .find(|class| class.name == "SharedImplementation")
        .expect("shared implementation should exist");

    assert!(!mutable.implements_protocols.contains(&"Shared".to_string()));
    assert!(shared.implements_protocols.contains(&"Shared".to_string()));
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
