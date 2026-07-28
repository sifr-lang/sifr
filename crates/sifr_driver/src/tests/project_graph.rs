use super::support::parse_suite;
use crate::{
    assemble_project_main_rs, check, collect_project_hir_modules, compile_frontend_modules,
    compile_stdlib, compute_module_compile_order, FrontendDiagnosticStyle,
};
use sifr_diagnostics::DiagnosticCode;
use sifr_lowering::{HirExpr, HirStmt};
use sifr_type_system::Type;
use std::collections::HashMap;

#[test]
fn test_compile_frontend_modules_uses_explicit_diagnostic_style() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
def main():
    print(missing_name)
"#,
        ),
    );

    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let bare_errors = compile_frontend_modules(
        &parsed_modules,
        stdlib_defs.clone(),
        FrontendDiagnosticStyle::Bare,
    )
    .err()
    .expect("bare diagnostic style should still report type errors");
    let prefixed_errors = compile_frontend_modules(
        &parsed_modules,
        stdlib_defs,
        FrontendDiagnosticStyle::ModulePrefixed,
    )
    .err()
    .expect("module-prefixed diagnostic style should report type errors");

    assert!(bare_errors
        .iter()
        .any(|e| !e.message.starts_with("[main] ")));
    assert!(prefixed_errors
        .iter()
        .all(|e| e.message.starts_with("[main] ")));
}

#[test]
fn test_check_and_project_lowering_share_typecheck_rules() {
    let source = r#"
def main():
    print(unknown_symbol)
"#;
    let check_errors = check(source);
    assert!(!check_errors.is_empty(), "check should report type errors");

    let mut parsed_modules = HashMap::new();
    parsed_modules.insert("main".to_string(), parse_suite(source));
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let project_errors = collect_project_hir_modules(&parsed_modules, stdlib_defs)
        .err()
        .expect("project lowering should report same frontend type errors");

    let check_messages: Vec<String> = check_errors.into_iter().map(|e| e.message).collect();
    let normalized_project_messages: Vec<String> = project_errors
        .into_iter()
        .map(|e| {
            e.message
                .strip_prefix("[main] ")
                .unwrap_or(&e.message)
                .to_string()
        })
        .collect();
    assert_eq!(check_messages, normalized_project_messages);
}

#[test]
fn test_project_lowering_propagates_imported_rust_opaque_close_ownership() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "resources".to_string(),
        parse_suite(
            r#"
class ResourceError(Error):
    message: str

@rust.opaque(type=bridge.resources.Resource, close=close)
class Resource:
    @rust(bridge.resources.close)
    def close(own self) -> Result[None, ResourceError]:
        ...
"#,
        ),
    );
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from resources import Resource

def close_borrowed(resource: Resource) -> None:
    resource.close()
"#,
        ),
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;

    let errors = match collect_project_hir_modules(&parsed_modules, stdlib_defs) {
        Ok(_) => panic!("borrowed imported close must fail before codegen"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES.code()
            && error.message.contains("borrowed parameter 'resource'")
    }));
}

#[test]
fn test_project_lowering_propagates_reexported_rust_opaque_close_ownership() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "resources".to_string(),
        parse_suite(
            r#"
class ResourceError(Error):
    message: str

@rust.opaque(type=bridge.resources.Resource, close=close)
class Resource:
    @rust(bridge.resources.close)
    def close(own self) -> Result[None, ResourceError]:
        ...
"#,
        ),
    );
    parsed_modules.insert(
        "facade".to_string(),
        parse_suite("from resources import Resource as ManagedResource\n"),
    );
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from facade import ManagedResource

def close_borrowed(resource: ManagedResource) -> None:
    resource.close()
"#,
        ),
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;

    let errors = match collect_project_hir_modules(&parsed_modules, stdlib_defs) {
        Ok(_) => panic!("borrowed reexported close must fail before codegen"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES.code()
            && error.message.contains("borrowed parameter 'resource'")
    }));
}

#[test]
fn test_collect_project_modules_supports_single_level_relative_import() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from .helper import value

def main():
    print(value())
"#,
        ),
    );
    parsed_modules.insert(
        "helper".to_string(),
        parse_suite(
            r#"
def value() -> int:
    return 42
"#,
        ),
    );

    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let result = collect_project_hir_modules(&parsed_modules, stdlib_defs)
        .expect("single-level relative imports should resolve in project lowering");
    assert!(result.hir_modules.contains_key("main"));
    assert!(result.hir_modules.contains_key("helper"));
}

#[test]
fn test_collect_project_modules_allows_non_main_stdlib_imports() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from helper import area_like

def main():
    print(area_like(2.0))
"#,
        ),
    );
    parsed_modules.insert(
        "helper".to_string(),
        parse_suite(
            r#"
from sifr.math import pi

def area_like(r: float) -> float:
    return r * pi
"#,
        ),
    );

    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let result = collect_project_hir_modules(&parsed_modules, stdlib_defs)
        .expect("project lowering should resolve non-main stdlib imports");
    assert!(result.hir_modules.contains_key("main"));
    assert!(result.hir_modules.contains_key("helper"));
}

#[test]
fn test_collect_project_modules_resolves_non_main_local_dependencies() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from a_consumer import fetch

def main():
    print(fetch())
"#,
        ),
    );
    parsed_modules.insert(
        "a_consumer".to_string(),
        parse_suite(
            r#"
from z_provider import value

def fetch() -> int:
    return value()
"#,
        ),
    );
    parsed_modules.insert(
        "z_provider".to_string(),
        parse_suite(
            r#"
def value() -> int:
    return 41
"#,
        ),
    );

    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let result = collect_project_hir_modules(&parsed_modules, stdlib_defs)
        .expect("project lowering should resolve non-main local imports");
    assert!(result.hir_modules.contains_key("main"));
    assert!(result.hir_modules.contains_key("a_consumer"));
    assert!(result.hir_modules.contains_key("z_provider"));
}

#[test]
fn test_compute_module_compile_order_is_dependency_safe() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from consumer import value

def main():
    print(value())
"#,
        ),
    );
    parsed_modules.insert(
        "consumer".to_string(),
        parse_suite(
            r#"
from provider import value_provider

def value() -> int:
    return value_provider()
"#,
        ),
    );
    parsed_modules.insert(
        "provider".to_string(),
        parse_suite(
            r#"
def value_provider() -> int:
    return 42
"#,
        ),
    );

    let order = compute_module_compile_order(&parsed_modules)
        .expect("compile order should be computed for acyclic graph");
    assert_eq!(
        order,
        vec![
            "provider".to_string(),
            "consumer".to_string(),
            "main".to_string()
        ]
    );
}

#[test]
fn test_compute_module_compile_order_is_deterministic_across_hashmap_insertion_order() {
    let mut parsed_modules_a = HashMap::new();
    parsed_modules_a.insert(
        "main".to_string(),
        parse_suite(
            r#"
from consumer import value

def main():
    print(value())
"#,
        ),
    );
    parsed_modules_a.insert(
        "consumer".to_string(),
        parse_suite(
            r#"
from provider import value_provider

def value() -> int:
    return value_provider()
"#,
        ),
    );
    parsed_modules_a.insert(
        "provider".to_string(),
        parse_suite(
            r#"
def value_provider() -> int:
    return 42
"#,
        ),
    );

    let mut parsed_modules_b = HashMap::new();
    parsed_modules_b.insert(
        "provider".to_string(),
        parse_suite(
            r#"
def value_provider() -> int:
    return 42
"#,
        ),
    );
    parsed_modules_b.insert(
        "main".to_string(),
        parse_suite(
            r#"
from consumer import value

def main():
    print(value())
"#,
        ),
    );
    parsed_modules_b.insert(
        "consumer".to_string(),
        parse_suite(
            r#"
from provider import value_provider

def value() -> int:
    return value_provider()
"#,
        ),
    );

    let order_a = compute_module_compile_order(&parsed_modules_a)
        .expect("compile order should be computed for acyclic graph");
    let order_b = compute_module_compile_order(&parsed_modules_b)
        .expect("compile order should be deterministic across map insertion order");
    assert_eq!(order_a, order_b);
    assert_eq!(
        order_a,
        vec![
            "provider".to_string(),
            "consumer".to_string(),
            "main".to_string()
        ]
    );
}

#[test]
fn test_assemble_project_main_rs_is_deterministic_against_hashmap_order() {
    let compile_order = vec![
        "provider".to_string(),
        "consumer".to_string(),
        "main".to_string(),
    ];

    let mut rust_files_a = HashMap::new();
    rust_files_a.insert("main".to_string(), "fn main() {}\n".to_string());
    rust_files_a.insert("consumer".to_string(), "pub fn c() {}\n".to_string());
    rust_files_a.insert("provider".to_string(), "pub fn p() {}\n".to_string());

    let mut rust_files_b = HashMap::new();
    rust_files_b.insert("provider".to_string(), "pub fn p() {}\n".to_string());
    rust_files_b.insert("main".to_string(), "fn main() {}\n".to_string());
    rust_files_b.insert("consumer".to_string(), "pub fn c() {}\n".to_string());

    let main_a = assemble_project_main_rs(&compile_order, &rust_files_a);
    let main_b = assemble_project_main_rs(&compile_order, &rust_files_b);
    assert_eq!(main_a, main_b);
    assert_eq!(main_a, "mod consumer;\nmod provider;\n\nfn main() {}\n");
}

#[test]
fn test_assemble_project_main_rs_declares_dotted_modules_by_top_level_namespace() {
    let compile_order = vec!["helpers.nodes".to_string(), "main".to_string()];

    let mut rust_files = HashMap::new();
    rust_files.insert("main".to_string(), "fn main() {}\n".to_string());
    rust_files.insert(
        "helpers.nodes".to_string(),
        "pub struct LinkedNode;\n".to_string(),
    );

    let main_rs = assemble_project_main_rs(&compile_order, &rust_files);

    assert_eq!(main_rs, "mod helpers;\n\nfn main() {}\n");
}

#[test]
fn test_collect_project_modules_reports_unknown_module_in_non_main() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from helper import get

def main():
    print(get())
"#,
        ),
    );
    parsed_modules.insert(
        "helper".to_string(),
        parse_suite(
            r#"
from missing_mod import value

def get() -> int:
    return value()
"#,
        ),
    );

    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let errors = collect_project_hir_modules(&parsed_modules, stdlib_defs)
        .err()
        .expect("project lowering should fail when non-main imports missing module");
    assert!(errors.iter().any(|e| {
        e.message.contains("unknown import target: 'missing_mod'")
            && e.code == DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE.code()
    }));
}

#[test]
fn test_project_lowering_preserves_retained_callback_contract_through_reexports() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "subscriptions".to_string(),
        parse_suite(
            r#"
class SubscriptionError(Error):
    message: str

class Subscription:
    lifecycle_token: int

@rust.callback(backpressure=bounded(2), overflow=error, shutdown=drain)
@rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
def subscribe(
    own handler: Callable[[str], None],
) -> Result[Subscription, SubscriptionError | RustPanicError]:
    ...
"#,
        ),
    );
    parsed_modules.insert(
        "api".to_string(),
        parse_suite(
            r#"
from subscriptions import subscribe as retained_subscribe
"#,
        ),
    );
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from api import retained_subscribe as subscribe

class LocalState(NonSend):
    value: int

def attach(state: LocalState):
    def handler(event: str):
        _value: int = state.value

    _subscription = subscribe(handler)
"#,
        ),
    );

    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let errors = collect_project_hir_modules(&parsed_modules, stdlib_defs)
        .err()
        .expect("retained callback capture should be rejected across module reexports");

    assert!(errors.iter().any(|error| {
        error.code == DiagnosticCode::RUST_CALLBACK_CONTRACT.code()
            && error.message.contains("handler `handler` capture `state`")
            && error.message.contains("not sendable")
    }));
}

#[test]
fn test_project_lowering_preserves_imported_method_callback_contract() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "subscriptions".to_string(),
        parse_suite(
            r#"
class Subscription:
    lifecycle_token: int

class Registrar:
    @rust.callback(backpressure=bounded(2), overflow=error, shutdown=drain)
    @rust(bridge.events.subscribe, panic=map_error(bridge.events.map_panic))
    def subscribe(
        self,
        own handler: Callable[[str], None],
    ) -> Subscription:
        ...
"#,
        ),
    );
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from subscriptions import Registrar

class LocalState(NonSend):
    value: int

def attach(registrar: Registrar, state: LocalState):
    def handler(event: str):
        _value: int = state.value

    _subscription = registrar.subscribe(handler)
"#,
        ),
    );

    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let errors = collect_project_hir_modules(&parsed_modules, stdlib_defs)
        .err()
        .expect("imported method callback capture should be rejected");

    assert!(errors.iter().any(|error| {
        error.code == DiagnosticCode::RUST_CALLBACK_CONTRACT.code()
            && error.message.contains("handler `handler` capture `state`")
            && error.message.contains("not sendable")
    }));
}

#[test]
fn test_collect_project_modules_cycle_reports_error() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from a import value_a

def main():
    print(value_a())
"#,
        ),
    );
    parsed_modules.insert(
        "a".to_string(),
        parse_suite(
            r#"
from b import value_b

def value_a() -> int:
    return value_b()
"#,
        ),
    );
    parsed_modules.insert(
        "b".to_string(),
        parse_suite(
            r#"
from a import value_a

def value_b() -> int:
    return value_a()
"#,
        ),
    );

    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let errors = collect_project_hir_modules(&parsed_modules, stdlib_defs)
        .err()
        .expect("project lowering should fail when there is a dependency cycle");
    assert!(errors
        .iter()
        .any(|e| e.message.contains("module dependency cycle detected")));
    assert!(errors.iter().any(|e| e.message.contains("a -> b -> a")));
}

#[test]
fn test_compute_module_compile_order_cycle_diagnostics_are_canonical_and_stable() {
    let mut parsed_modules_a = HashMap::new();
    parsed_modules_a.insert(
        "main".to_string(),
        parse_suite(
            r#"
from a import value_a

def main():
    print(value_a())
"#,
        ),
    );
    parsed_modules_a.insert(
        "a".to_string(),
        parse_suite(
            r#"
from b import value_b

def value_a() -> int:
    return value_b()
"#,
        ),
    );
    parsed_modules_a.insert(
        "b".to_string(),
        parse_suite(
            r#"
from c import value_c

def value_b() -> int:
    return value_c()
"#,
        ),
    );
    parsed_modules_a.insert(
        "c".to_string(),
        parse_suite(
            r#"
from a import value_a

def value_c() -> int:
    return value_a()
"#,
        ),
    );

    let mut parsed_modules_b = HashMap::new();
    parsed_modules_b.insert(
        "c".to_string(),
        parse_suite(
            r#"
from a import value_a

def value_c() -> int:
    return value_a()
"#,
        ),
    );
    parsed_modules_b.insert(
        "b".to_string(),
        parse_suite(
            r#"
from c import value_c

def value_b() -> int:
    return value_c()
"#,
        ),
    );
    parsed_modules_b.insert(
        "main".to_string(),
        parse_suite(
            r#"
from a import value_a

def main():
    print(value_a())
"#,
        ),
    );
    parsed_modules_b.insert(
        "a".to_string(),
        parse_suite(
            r#"
from b import value_b

def value_a() -> int:
    return value_b()
"#,
        ),
    );

    let error_a = compute_module_compile_order(&parsed_modules_a)
        .err()
        .expect("cycle graph should fail compile ordering");
    let error_b = compute_module_compile_order(&parsed_modules_b)
        .err()
        .expect("cycle graph should fail compile ordering");

    let message_a = &error_a[0].message;
    let message_b = &error_b[0].message;
    assert_eq!(message_a, message_b);
    assert!(message_a.contains("module dependency cycle detected: a -> b -> c -> a"));
    assert!(message_a.contains("import chain: a imports b, b imports c, c imports a"));
}

#[test]
fn test_collect_project_modules_exports_local_constants() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from consumer import get

def main():
    print(get())
"#,
        ),
    );
    parsed_modules.insert(
        "consumer".to_string(),
        parse_suite(
            r#"
from constants_mod import ANSWER

def get() -> int:
    return ANSWER
"#,
        ),
    );
    parsed_modules.insert(
        "constants_mod".to_string(),
        parse_suite(
            r#"
ANSWER: int = 42
"#,
        ),
    );

    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let result = collect_project_hir_modules(&parsed_modules, stdlib_defs)
        .expect("project lowering should resolve local constant imports");
    let constants = result
        .external_defs
        .constants
        .get("constants_mod")
        .expect("constants module exports should exist");
    assert_eq!(constants.get("ANSWER"), Some(&Type::Int));
    let constant_values = result
        .external_defs
        .constant_integer_values
        .get("constants_mod")
        .expect("integer constant values should be exported");
    assert_eq!(
        constant_values
            .get("ANSWER")
            .map(std::string::ToString::to_string),
        Some("42".to_string())
    );
}

#[test]
fn test_project_lowering_fits_imported_integer_constants() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from constants_mod import BASE as LIMIT

def main() -> uint8:
    value: uint8 = LIMIT + 1
    return value
"#,
        ),
    );
    parsed_modules.insert(
        "constants_mod".to_string(),
        parse_suite(
            r#"
BASE: int = 250 + 4
"#,
        ),
    );

    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let result = collect_project_hir_modules(&parsed_modules, stdlib_defs)
        .expect("project lowering should fit imported integer constants");
    let main_module = result
        .hir_modules
        .get("main")
        .expect("main module should lower");
    let main_fn = main_module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function should lower");
    let HirStmt::Let { ty, value, .. } = &main_fn.body[0] else {
        panic!("expected first statement to be fitted let");
    };
    assert_eq!(ty.display_name(), "uint8");
    assert!(matches!(value, HirExpr::IntLiteral(255)));
}

#[test]
fn test_project_lowering_does_not_fold_shadowed_imported_integer_constant() {
    let mut parsed_modules = HashMap::new();
    parsed_modules.insert(
        "main".to_string(),
        parse_suite(
            r#"
from constants_mod import BASE

def main():
    BASE: int = 100
    value: uint8 = BASE + 1
"#,
        ),
    );
    parsed_modules.insert(
        "constants_mod".to_string(),
        parse_suite(
            r#"
BASE: int = 254
"#,
        ),
    );

    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let Err(errors) = collect_project_hir_modules(&parsed_modules, stdlib_defs) else {
        panic!("shadowed imported integer constant should not fit");
    };
    assert!(errors.iter().any(|error| {
        error.code == DiagnosticCode::TYPE_MISMATCH.code()
            && error
                .message
                .contains("[main] type mismatch: expected 'uint8', got 'int'")
    }));
}
