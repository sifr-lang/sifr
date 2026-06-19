use crate::{ExternalDefs, HirDiagnostic};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_parser::parse_module;
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

fn object_ty() -> Type {
    Type::Class {
        name: "Object".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: Some("NonSend".to_string()),
    }
}

fn error_ty() -> Type {
    Type::Class {
        name: "PythonError".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    }
}

fn python_externals() -> ExternalDefs {
    let object_ty = object_ty();
    let error_ty = error_ty();
    let mut functions = HashMap::new();
    functions.insert(
        "from_int".to_string(),
        FunctionType::all_borrow(
            vec![("value".to_string(), Type::Int)],
            Type::Result(Box::new(object_ty.clone()), Box::new(error_ty.clone())),
        ),
    );
    functions.insert(
        "run_coroutine_blocking".to_string(),
        FunctionType::all_borrow(
            vec![("coro".to_string(), object_ty.clone())],
            Type::Result(Box::new(object_ty.clone()), Box::new(error_ty.clone())),
        ),
    );
    functions.insert(
        "to_int".to_string(),
        FunctionType::all_borrow(
            vec![("obj".to_string(), object_ty.clone())],
            Type::Result(Box::new(Type::Int), Box::new(error_ty.clone())),
        ),
    );
    functions.insert(
        "close".to_string(),
        FunctionType::new(
            vec![("obj".to_string(), object_ty.clone())],
            Type::Result(Box::new(Type::None), Box::new(error_ty.clone())),
        ),
    );

    let mut classes = HashMap::new();
    classes.insert("Object".to_string(), object_ty);
    classes.insert("PythonError".to_string(), error_ty);

    let mut workloads = HashMap::new();
    for name in ["from_int", "run_coroutine_blocking", "to_int", "close"] {
        workloads.insert(name.to_string(), "blocking_io".to_string());
    }

    let mut externals = ExternalDefs::default();
    externals
        .functions
        .insert("sifr.python".to_string(), functions);
    externals.classes.insert("sifr.python".to_string(), classes);
    externals
        .function_workloads
        .insert("sifr.python".to_string(), workloads);
    externals.error_types.insert("PythonError".to_string());
    externals
}

fn lower(source: &str) -> Result<(), Vec<HirDiagnostic>> {
    let parsed = parse_module(source).expect("source should parse");
    crate::lower_module_with_externals(parsed.suite(), &python_externals()).map(|_| ())
}

#[test]
fn async_python_call_is_rejected_without_offload() {
    let source = "from sifr.python import Object, PythonError, from_int\n\nasync def main() -> Result[None, PythonError]:\n    obj: Object = from_int(1)\n    return None\n";
    let errors = lower(source).expect_err("direct Python call should fail in async code");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::ASYNC_DIRECT_BLOCKING_IO_CALL)
            && error.message.contains("blocking_io function 'from_int'")
    }));
}

#[test]
fn run_coroutine_blocking_is_rejected_without_offload() {
    let source = "from sifr.python import Object, PythonError, run_coroutine_blocking\n\nasync def main(coro: Object) -> Result[None, PythonError]:\n    result: Object = run_coroutine_blocking(coro)\n    return None\n";
    let errors = lower(source).expect_err("run_coroutine_blocking should fail in async code");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::ASYNC_DIRECT_BLOCKING_IO_CALL)
            && error
                .message
                .contains("blocking_io function 'run_coroutine_blocking'")
    }));
}

#[test]
fn offloaded_python_worker_is_allowed_when_returning_send_value() {
    let source = "from sifr.python import Object, PythonError, from_int, to_int\n\n@blocking_io\ndef build_value() -> Result[int, PythonError]:\n    try:\n        obj: Object = from_int(41)\n        value: int = to_int(obj)\n        return value\n    except PythonError as e:\n        raise e\n\nasync def main() -> Result[None, ScopeFailure]:\n    handle = task.spawn_blocking(build_value)\n    result = await handle\n    return None\n";

    lower(source).expect("annotated Python worker returning sendable data should lower");
}

#[test]
fn offloaded_python_worker_cannot_return_object_handle() {
    let source = "from sifr.python import Object, PythonError, from_int\n\n@blocking_io\ndef build_object() -> Result[Object, PythonError]:\n    try:\n        obj: Object = from_int(41)\n        return obj\n    except PythonError as e:\n        raise e\n\nasync def main() -> Result[None, ScopeFailure]:\n    handle = task.spawn_blocking(build_object)\n    result = await handle\n    return None\n";
    let errors = lower(source).expect_err("py.Object should not cross worker boundary");

    assert!(errors.iter().any(|error| {
        error
            .message
            .contains("task.spawn_blocking() cannot return non-send value type 'Object'")
            && error
                .message
                .contains("`Object` inherits the `NonSend` marker")
    }));
}

#[test]
fn offloaded_python_worker_must_be_classified() {
    let source = "from sifr.python import Object, PythonError, from_int, to_int\n\ndef build_value() -> Result[int, PythonError]:\n    try:\n        obj: Object = from_int(41)\n        value: int = to_int(obj)\n        return value\n    except PythonError as e:\n        raise e\n\nasync def main() -> Result[None, ScopeFailure]:\n    handle = task.spawn_blocking(build_value)\n    result = await handle\n    return None\n";
    let errors = lower(source).expect_err("unclassified Python worker should fail offload");

    assert!(errors.iter().any(|error| {
        error.code == Some(DiagnosticCode::ASYNC_UNCLASSIFIED_BLOCKING_OFFLOAD_TARGET)
            && error
                .message
                .contains("target 'build_value' is not classified")
    }));
}
