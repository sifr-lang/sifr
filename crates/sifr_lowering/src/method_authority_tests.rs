use crate::{ExternalDefs, HirExpr, HirModule, lower_module, lower_module_with_externals};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{MethodAuthority, visit_hir_function_exprs_mut};
use sifr_python_parser::parse_module;
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

fn lower(source: &str) -> HirModule {
    let parsed = parse_module(source).expect("source should parse");
    lower_module(parsed.suite())
        .expect("source should lower")
        .module
}

fn method_authorities(module: &HirModule, function_name: &str) -> Vec<MethodAuthority> {
    let mut function = module
        .functions
        .iter()
        .find(|function| function.name == function_name)
        .expect("function should exist")
        .clone();
    let mut authorities = Vec::new();
    visit_hir_function_exprs_mut(&mut function, &mut |expr| {
        if let HirExpr::MethodCall { authority, .. } = expr {
            authorities.push(authority.clone());
        }
    });
    authorities
}

#[test]
fn typed_dispatch_classification() {
    let source = r"
class Base:
    def value(self) -> int:
        return 1

class Child(Base):
    pass

class Face(Protocol):
    def value(self) -> int: ...

def local(x: Base) -> int:
    return x.value()

def inherited(x: Child) -> int:
    return x.value()

def protocol(x: Face) -> int:
    return x.value()

def builtin(mut x: list[int]) -> None:
    x.append(1)
";
    let module = lower(source);
    assert!(
        matches!(method_authorities(&module, "local").as_slice(), [MethodAuthority::LocalNominal { declaration }] if declaration.owner.as_deref() == Some("Base") && declaration.symbol == "value")
    );
    assert!(
        matches!(method_authorities(&module, "inherited").as_slice(), [MethodAuthority::InheritedNominal { declaration }] if declaration.owner.as_deref() == Some("Base") && declaration.symbol == "value")
    );
    assert!(
        matches!(method_authorities(&module, "protocol").as_slice(), [MethodAuthority::Protocol { declaration }] if declaration.owner.as_deref() == Some("Face") && declaration.symbol == "value")
    );
    assert!(
        matches!(method_authorities(&module, "builtin").as_slice(), [MethodAuthority::BuiltinIntrinsic { declaration }] if declaration.owner.as_deref() == Some("list[int]") && declaration.symbol == "append")
    );

    let user_enum = r#"
class Color(Enum):
    RED = 1

    def label(self) -> str:
        return "red"

def enum_label(color: Color) -> str:
    return color.label()

def enum_name(color: Color) -> str:
    return color.name()
"#;
    let module = lower(user_enum);
    assert!(
        matches!(method_authorities(&module, "enum_label").as_slice(), [MethodAuthority::LocalNominal { declaration }] if declaration.owner.as_deref() == Some("Color") && declaration.symbol == "label")
    );
    assert!(
        matches!(method_authorities(&module, "enum_name").as_slice(), [MethodAuthority::BuiltinIntrinsic { declaration }] if declaration.owner.as_deref() == Some("Color") && declaration.symbol == "name")
    );

    let imported = r"
from provider import Gadget

def imported(x: Gadget) -> int:
    return x.get()
";
    let parsed = parse_module(imported).expect("imported source should parse");
    let mut externals = ExternalDefs::default();
    externals.classes.insert(
        "provider".to_string(),
        HashMap::from([(
            "Gadget".to_string(),
            Type::Class {
                identity: Some("provider.Gadget".to_string()),
                type_args: Vec::new(),
                name: "Gadget".to_string(),
                fields: Vec::new().into(),
                methods: vec![("get".to_string(), FunctionType::new(Vec::new(), Type::Int))].into(),
                parent_class: None,
            },
        )]),
    );
    let module = lower_module_with_externals(parsed.suite(), &externals)
        .expect("imported method should lower")
        .module;
    assert!(
        matches!(method_authorities(&module, "imported").as_slice(), [MethodAuthority::Imported { declaration }] if declaration.module == "provider" && declaration.owner.as_deref() == Some("Gadget") && declaration.symbol == "get")
    );

    let synthesized = r"
async def worker() -> int:
    await task.sleep(0.0)
    return 1

async def spawn() -> Result[None, ScopeFailure]:
    async with task.TaskGroup() as group:
        task.spawn_scoped(worker())
    return None
";
    let module = lower(synthesized);
    assert!(
        method_authorities(&module, "spawn")
            .iter()
            .any(|authority| {
                matches!(authority, MethodAuthority::BuiltinIntrinsic { declaration }
            if declaration.owner.as_deref() == Some("TaskGroup")
                && declaration.symbol == "__sifr_spawn_infallible")
            })
    );

    let same_named_user_class = r"
class TaskGroup:
    def spawn(self) -> int:
        return 1

def user_spawn(group: TaskGroup) -> int:
    return group.spawn()
";
    let module = lower(same_named_user_class);
    assert!(
        matches!(method_authorities(&module, "user_spawn").as_slice(), [MethodAuthority::LocalNominal { declaration }] if declaration.owner.as_deref() == Some("TaskGroup") && declaration.symbol == "spawn")
    );

    let rust_adapted = r"
class TokenError(Error):
    message: str

@rust.opaque(type=bridge.token.Token, close=none)
class Token:
    @rust(bridge.token.inspect)
    def inspect(self) -> Result[int, TokenError]: ...

def inspect_token(token: Token) -> Result[int, TokenError]:
    return token.inspect()
";
    let module = lower(rust_adapted);
    assert!(
        matches!(method_authorities(&module, "inspect_token").as_slice(), [MethodAuthority::RustAdapted { declaration }] if declaration.owner.as_deref() == Some("Token") && declaration.symbol == "inspect")
    );
}

#[test]
fn unsupported_method_declines_with_diagnostic() {
    let source = "def bad(x: int) -> None:\n    x.nonexistent()\n";
    let parsed = parse_module(source).expect("source should parse");
    let errors = match lower_module(parsed.suite()) {
        Ok(_) => panic!("unsupported method must decline"),
        Err(errors) => errors,
    };
    assert!(
        errors.iter().any(|error| {
            error.code == Some(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE)
                && error.message.contains("no method 'nonexistent'")
                && error.primary_range.is_some()
        }),
        "{errors:?}"
    );
}

#[test]
fn unclassified_method_cannot_leave_lowering() {
    let source = "def main(mut x: list[int]) -> None:\n    x.append(1)\n";
    let mut module = lower(source);
    assert!(
        method_authorities(&module, "main")
            .iter()
            .all(|authority| !matches!(authority, MethodAuthority::Unclassified))
    );
    let function = module
        .functions
        .iter_mut()
        .find(|function| function.name == "main")
        .expect("main exists");
    visit_hir_function_exprs_mut(function, &mut |expr| {
        if let HirExpr::MethodCall {
            authority, source, ..
        } = expr
        {
            *authority = MethodAuthority::Unclassified;
            *source = None;
        }
    });
    let violations = crate::lower::verify_method_authority_for_tests(&mut module);
    assert!(
        violations
            .iter()
            .any(|message| message.contains("no classified authority")),
        "{violations:?}"
    );
}
