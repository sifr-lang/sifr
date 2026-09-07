use super::*;

#[test]
fn required_error_message_python_native_five_field_contract() {
    let dir = mktemp_dir("required_error_message_python");
    let app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    write_package_source(
        &app,
        "main.sifr",
        r#"
from sifr.python import PythonError

def take(own error: Error) -> str:
    return error.message

def main():
    error = PythonError("python", "kind", "Exception", "traceback", "context")
    assert error.message == "python"
    assert error.kind == "kind"
    assert error.exception_type == "Exception"
    assert error.traceback == "traceback"
    assert error.context == "context"
    assert take(error) == "python"
    print("required-error-message-ok")
"#,
    );
    let graph = package_graph(&dir, &[&app], &[]);
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map");
    let mut entrypoint =
        package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));
    // Canonical PythonError links the Python runtime even without invoking
    // Python. Use the existing probed package fixture, including native trust.
    entrypoint.python_runtime = Some(local_python_runtime(&dir));
    let artifact =
        build_cached_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new())
            .expect("canonical PythonError must build with the selected runtime");
    let output = std::process::Command::new(artifact.binary_path())
        .output()
        .expect("execute native PythonError assertions");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "required-error-message-ok"
    );
    let _ = std::fs::remove_dir_all(dir);
}
