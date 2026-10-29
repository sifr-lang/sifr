use super::rust_source_uses_python_runtime;

#[test]
fn runtime_dependency_detection_covers_both_python_namespaces() {
    assert!(rust_source_uses_python_runtime(
        "::sifr_stdlib::python::PythonError"
    ));
    assert!(rust_source_uses_python_runtime(
        "::sifr_runtime::python::PythonError"
    ));
    assert!(!rust_source_uses_python_runtime("struct PythonError;"));
}
