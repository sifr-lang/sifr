use super::*;

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn local_object_record_builds_beside_the_sealed_python_handle() {
    let dir = mktemp_dir("package_python_local_object_collision");
    let app = production_package(&dir, "app", "sifr-object-app", "object_app");
    write_package_source(
        &app,
        "main.sifr",
        r#"from sifr.python import PythonError

class Object:
    value: int

@python(builtins.id)
def echo(value: Object) -> Result[Object, PythonError]: ...

@python.coroutine(builtins.id)
async def echo_async(value: Object) -> Result[Object, PythonError]: ...

async def main() -> Result[None, PythonError]:
    try:
        source = Object(1)
        _result = await echo_async(source)
        return None
    except PythonError as error:
        raise error
"#,
    );

    let graph = package_graph(&dir, &[&app], &[]);
    let source_map = sifr_package::PackageSourceMap::build(&graph).expect("source map builds");
    let mut entrypoint =
        package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));
    entrypoint.python_runtime = Some(local_python_runtime(&app.root));
    let artifact = build_cached_package_project(&entrypoint)
        .expect("local Object and sealed Python handle should build with distinct Rust names");

    assert!(artifact.binary_path().exists());
    let _ignored = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn raw_coroutine_api_builds_and_runs_on_the_owned_loop() {
    let dir = mktemp_dir("package_python_owned_async_loop");
    let app = production_package(&dir, "app", "sifr-async-app", "async_app");
    write_package_source(
        &app,
        "main.sifr",
        r#"from sifr.python import Object, PythonError, call_attr, close, from_float, from_int, import_module, run_coroutine_blocking, to_int

@trust_python_dynamic
@blocking_io
def python_value() -> Result[int, PythonError]:
    try:
        asyncio: Object = import_module("asyncio")
        delay: Object = from_float(0.0)
        expected: Object = from_int(42)
        coroutine: Object = call_attr(asyncio, "sleep", [delay], [("result", expected)])
        result_object: Object = run_coroutine_blocking(coroutine)
        result: int = to_int(result_object)
        _closed_result: None = close(result_object)
        _closed_coroutine: None = close(coroutine)
        _closed_asyncio: None = close(asyncio)
        return result
    except PythonError as error:
        raise error

@blocking_io
def main() -> Result[None, PythonError]:
    try:
        value: int = python_value()
        assert value == 42
        print("sifr-python-interop:owned-async-loop:value=42")
    except PythonError as error:
        raise error
    return None
"#,
    );

    let graph = package_graph(&dir, &[&app], &[]);
    let source_map = sifr_package::PackageSourceMap::build(&graph).expect("source map builds");
    let mut entrypoint =
        package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));
    entrypoint.python_runtime = Some(local_python_runtime_with_roots(&app.root, &["asyncio"]));
    let artifact = build_cached_package_project(&entrypoint)
        .expect("raw coroutine package should build with the owned loop");
    let output = std::process::Command::new(artifact.binary_path())
        .output()
        .expect("raw coroutine package should run");

    assert!(
        output.status.success(),
        "binary should pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "sifr-python-interop:owned-async-loop:value=42"
    );
    let _ignored = std::fs::remove_dir_all(dir);
}
