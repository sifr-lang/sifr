use super::*;

#[test]
fn test_mut_on_mutating_method_call() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "items".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                    value: HirExpr::ListLiteral {
                        elements: vec![HirExpr::IntLiteral(1)],
                        ty: Type::List(Box::new(Type::Int)),
                    },
                    is_mutable: true,
                },
                HirStmt::Expr {
                    expr: HirExpr::MethodCall {
                        object: Box::new(HirExpr::Name {
                            name: "items".to_string(),
                            binding_id: None,
                            ty: Type::List(Box::new(Type::Int)),
                        }),
                        method: "append".to_string(),
                        args: vec![HirExpr::IntLiteral(2)],
                        receiver_convention: Some(
                            sifr_type_system::ReceiverConvention::MutableBorrow,
                        ),
                        source: None,
                        ty: Type::None,
                    },
                },
            ],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let rust_code = generate_rust(&module);
    assert!(rust_code.contains("let mut items"));
}

#[test]
fn test_mut_on_local_nested_function_mutborrow_call_argument() {
    let rust_code = generate_rust_from_source(
        r#"def main():
    vals: list[str] = ["x"]
    def dfs(vals: list[str]) -> None:
        vals.pop(0)
    dfs(vals)
"#,
    );

    assert!(
        rust_code.contains("let mut vals: Vec<String>"),
        "local nested mut-borrow call should mark argument binding mutable"
    );
    assert!(rust_code.contains("dfs(&mut vals);"));
}

#[test]
fn inferred_class_receiver_signatures_are_emitted_from_hir_metadata() {
    let rust_code = generate_rust_from_source(
        r#"
class Counter:
    value: int

    def read(self) -> int:
        return self.value

    def bump(self) -> None:
        self.value += 1

class Owner:
    counter: Counter

    def bump(self) -> None:
        self.counter.bump()

class Consumable:
    def close(own self) -> None:
        pass
"#,
    );

    assert!(rust_code.contains("fn read(&self) -> i64"));
    assert!(rust_code.contains("fn bump(&mut self)"));
    assert!(rust_code.contains("fn close(&self)"));
}

#[test]
fn protocol_bridge_uses_protocol_receiver_convention() {
    let rust_code = generate_rust_from_source(
        r#"
class Mutable(Protocol):
    def update(mut self) -> None:
        pass

class SharedImplementation:
    def update(self) -> None:
        pass
"#,
    );

    let protocol_impl = rust_code
        .split("impl Mutable for SharedImplementation")
        .nth(1)
        .expect("protocol implementation should be emitted");
    assert!(protocol_impl.contains("fn update(&mut self)"));
}

#[test]
fn incompatible_inferred_mutable_receiver_does_not_emit_shared_protocol_bridge() {
    let rust_code = generate_rust_from_source(
        r#"
class Shared(Protocol):
    def update(self) -> None:
        pass

class MutableImplementation:
    value: int

    def update(self) -> None:
        self.value += 1
"#,
    );

    assert!(!rust_code.contains("impl Shared for MutableImplementation"));
}
