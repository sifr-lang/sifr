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
                            binding_id: Some(sifr_ir::BindingId(1)),
                            ty: Type::List(Box::new(Type::Int)),
                        }),
                        method: "append".to_string(),
                        args: vec![HirExpr::IntLiteral(2)],
                        receiver_convention: Some(
                            sifr_type_system::ReceiverConvention::MutableBorrow,
                        ),
                        receiver_target: Some(sifr_ir::MutableReceiverTarget::Place(
                            sifr_ir::Place {
                                root: sifr_ir::BindingId(1),
                                projections: Vec::new(),
                            },
                        )),
                        mutable_arg_places: vec![None],
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
fn checked_field_receivers_emit_original_storage_without_clone_or_temporary() {
    let rust_code = generate_rust_from_source(
        r#"
class Helper:
    items: list[int]

    def bump(self) -> None:
        self.items.append(1)

class Mid:
    helper: Helper

class Base:
    mid: Mid

class Child(Base):
    def __init__(self, mid: Mid):
        super().__init__(mid)

    def run(self) -> None:
        self.mid.helper.bump()
"#,
    );

    assert!(
        rust_code.contains("self.base.mid.helper.bump()"),
        "{rust_code}"
    );
    assert!(!rust_code.contains("clone().bump()"), "{rust_code}");
    assert!(!rust_code.contains("cloned().bump()"), "{rust_code}");
    assert!(!rust_code.contains("take().bump()"), "{rust_code}");
}

#[test]
fn constructors_materialize_mutable_storage_before_receiver_calls() {
    let rust_code = generate_rust_from_source(
        r#"
class Helper:
    items: list[int]

    def __init__(self):
        self.items = []
        self.items.append(1)

    def bump(self) -> None:
        self.items.append(1)

class Base:
    helper: Helper

    def __init__(self, helper: Helper):
        self.helper = helper

class Child(Base):
    def __init__(self):
        super().__init__(Helper())
        self.helper.bump()
"#,
    );

    assert!(
        rust_code.contains("let mut __sifr_self = Self {"),
        "{rust_code}"
    );
    assert!(
        rust_code.contains("__sifr_self.items.push(1_i64)"),
        "{rust_code}"
    );
    assert!(
        rust_code.contains("__sifr_self.base.helper.bump()"),
        "{rust_code}"
    );
    assert!(!rust_code.contains("self.helper.bump()"), "{rust_code}");
}

#[test]
fn constructors_preserve_statement_order_and_rewrite_storage_roots() {
    let rust_code = generate_rust_from_source(
        r#"
class Helper:
    value: int

    def __init__(self):
        self.value = 0

class Owner:
    helper: Helper
    count: int

    def __init__(self, flag: bool):
        self.helper = Helper()
        self.count = 0
        self.helper.value = 7
        n: int = 0
        if flag:
            self.count += 1
            n = n + 1
        assert n == 1
        doubled: int = self.count * 2
        assert doubled == 2
"#,
    );

    let instance = rust_code
        .find("let mut __sifr_self = Self {")
        .expect("constructor should materialize its instance");
    let nested_assignment = rust_code
        .find("__sifr_self.helper.value = 7_i64")
        .expect("nested assignment should use the synthetic receiver");
    let branch = rust_code
        .find("if flag")
        .expect("self-dependent branch should be emitted");
    let local_assert = rust_code
        .find("assert!(n == (1_i64))")
        .expect("self-free statement after the branch should retain its order");
    let reverse_dependency = rust_code
        .find("let doubled: i64 = __sifr_self.count * (2_i64)")
        .expect("self-dependent local should use the synthetic receiver");
    let dependency_assert = rust_code
        .find("assert!(doubled == (2_i64))")
        .expect("dependent statement should follow its binding");

    assert!(
        instance < nested_assignment
            && nested_assignment < branch
            && branch < local_assert
            && local_assert < reverse_dependency
            && reverse_dependency < dependency_assert,
        "{rust_code}"
    );
    assert!(
        !rust_code.contains("\n        self.helper.value"),
        "{rust_code}"
    );
}
