use super::*;

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn typed_raw_api_builds_runs_and_releases_ordinary_objects() {
    let dir = mktemp_dir("package_python_typed_raw_api");
    let app = production_package(&dir, "app", "sifr-raw-app", "raw_app");
    write_package_source(
        &app,
        "main.sifr",
        r#"from sifr.python import Object, PythonError, from_value, kwarg, resource_diagnostics, to_value
from sifr.python_core import ResourceDiagnostics

class Payload:
    name: str
    values: list[int]

    def __init__(self, name: str, values: list[int]):
        self.name = name
        self.values = values

@blocking_io
def exercise_raw_objects() -> Result[str, PythonError]:
    try:
        payload_object: Object = from_value(Payload("sifr", [1, 2, 3]))
        payload: Payload = to_value(payload_object)
        assert payload.name == "sifr"
        assert payload.values == [1, 2, 3]

        item_object: Object = payload_object.get_item("name")
        item: str = to_value(item_object)
        assert item == "sifr"

        template: Object = from_value("{name}:{count}")
        name_kwarg: tuple[str, Object] = kwarg("name", "typed")
        count_kwarg: tuple[str, Object] = kwarg("count", 3)
        formatted_object: Object = template.call_method(
            "format",
            [],
            [name_kwarg, count_kwarg],
        )
        formatted: str = to_value(formatted_object)
        assert formatted == "typed:3"

        upper_callable: Object = item_object.get_attr("upper")
        upper_object: Object = upper_callable.call([], [])
        return to_value(upper_object)
    except PythonError as error:
        raise error

@blocking_io
def checked_failure() -> Result[bool, PythonError]:
    try:
        text: Object = from_value("not an integer")
        invalid: int = to_value(text)
        return invalid == 0
    except PythonError as error:
        return error.kind == "conversion" and error.exception_type == "TypeError"

@blocking_io
def main() -> Result[None, PythonError]:
    try:
        before: ResourceDiagnostics = resource_diagnostics()
        result: str = exercise_raw_objects()
        assert result == "SIFR"
        failed_safely: bool = checked_failure()
        assert failed_safely
        after: ResourceDiagnostics = resource_diagnostics()
        assert after.live_objects == before.live_objects
        print("sifr-python-interop:typed-raw-api:released")
        return None
    except PythonError as error:
        raise error
"#,
    );

    let graph = package_graph(&dir, &[&app], &[]);
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let mut entrypoint =
        package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));
    entrypoint.python_runtime = Some(local_python_runtime(&app.root));
    let artifact =
        build_cached_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new())
            .expect("typed raw Python API package should build");
    let output = std::process::Command::new(artifact.binary_path())
        .output()
        .expect("typed raw Python API package should run");

    assert!(
        output.status.success(),
        "binary should pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "sifr-python-interop:typed-raw-api:released"
    );
    let _ignored = std::fs::remove_dir_all(dir);
}
