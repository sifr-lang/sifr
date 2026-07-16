use super::*;

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn compiler_owned_nominal_paths_and_unions_build_beside_source_collisions() {
    let dir = mktemp_dir("package_compiler_identity_collisions");
    let app = production_package(&dir, "app", "sifr-identity-app", "identity_app");
    write_package_source(
        &app,
        "main.sifr",
        r#"from enum import Enum

class __SifrIoNativeFileHandle:
    value: int

class __SifrIoFileHandle(int):
    pass

class __SifrIoTextFileHandle(Enum):
    READY = 1

class __SifrIoBinaryFileHandle(Protocol):
    def value(self) -> int:
        pass

class LocalValue:
    number: int

    def value(self) -> int:
        return self.number

class std:
    value: int

class Int:
    value: int

class IntOrStr:
    value: int

def read(own value: __SifrIoBinaryFileHandle) -> int:
    return value.value()

def choose(flag: bool) -> int | Int:
    if flag:
        return 1
    return Int(2)

def choose_text(flag: bool) -> int | str:
    if flag:
        return 1
    return "text"

def main() -> None:
    regular = __SifrIoNativeFileHandle(1)
    wrapped = __SifrIoFileHandle(2)
    variant = __SifrIoTextFileHandle.READY
    local_std = std(4)
    local_union_name = IntOrStr(8)
    values: dict[str, int] = {"value": 16}
    first: int | Int = choose(True)
    second: int | str = choose_text(False)
    assert read(LocalValue(32)) == 32
    assert regular.value + wrapped.value() + variant.value() == 4
    assert local_std.value + local_union_name.value == 12
    assert values["value"] == 16
    _ = first
    _ = second
"#,
    );

    let graph = package_graph(&dir, &[&app], &[]);
    let source_map = sifr_package::PackageSourceMap::build(&graph).expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));
    let artifact = build_cached_package_project(&entrypoint)
        .expect("compiler-owned nominal paths and unions should be source-disjoint");

    assert!(artifact.binary_path().exists());
    let _ignored = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn canonical_open_handles_build_beside_local_same_basename_classes() {
    let dir = mktemp_dir("package_open_handle_identity_collision");
    let app = production_package(&dir, "app", "sifr-open-app", "open_app");
    write_package_source(
        &app,
        "main.sifr",
        r#"class FileHandle:
    value: int

class TextFileHandle:
    value: int

class __SifrIoFileHandle:
    value: int

class __SifrIoTextFileHandle:
    value: int

def main() -> None:
    local_binary = FileHandle(1)
    local_text = TextFileHandle(2)
    local_internal_binary = __SifrIoFileHandle(4)
    local_internal_text = __SifrIoTextFileHandle(8)
    try:
        binary = open("/tmp/sifr_open_identity.bin", "wb")
        binary.close()
        text = open("/tmp/sifr_open_identity.txt", "w", encoding="utf-8")
        text.close()
    except IOError as error:
        _ = error.message
    assert local_binary.value + local_text.value + local_internal_binary.value + local_internal_text.value == 15
"#,
    );

    let graph = package_graph(&dir, &[&app], &[]);
    let source_map = sifr_package::PackageSourceMap::build(&graph).expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));
    let artifact = build_cached_package_project(&entrypoint)
        .expect("canonical and local same-basename file handles should not collide");

    assert!(artifact.binary_path().exists());
    let _ignored = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn source_spellable_object_names_build_beside_the_sealed_python_handle() {
    let dir = mktemp_dir("package_python_local_object_collision");
    let app = production_package(&dir, "app", "sifr-object-app", "object_app");
    write_package_source(
        &app,
        "main.sifr",
        r#"from sifr.python import Object as PythonObject, PythonError

class Object:
    value: int

class __SifrPythonObject:
    value: int

class sifr_runtime:
    value: int

@python(builtins.id)
def echo(value: PythonObject) -> Result[PythonObject, PythonError]: ...

def main() -> Result[None, PythonError]:
    try:
        local_object = Object(1)
        local_alias = __SifrPythonObject(2)
        local_runtime = sifr_runtime(3)
        _ = local_object.value + local_alias.value + local_runtime.value
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
        .expect("source-spellable classes and the sealed Python handle should not collide");

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
