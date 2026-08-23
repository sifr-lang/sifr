use super::{PythonRuntimeConfig, PythonRuntimeError, py_error};
use pyo3::types::PyAnyMethods;
use pyo3::{Bound, PyAny, prelude::Python};
use std::path::Path;

pub(super) fn verify_interpreter_config(
    py: Python<'_>,
    config: &PythonRuntimeConfig,
) -> Result<(), PythonRuntimeError> {
    verify_interpreter_version(py, config)?;
    let sys = py.import("sys").map_err(|error| py_error(&error))?;
    verify_sys_attr(&sys, "executable", &config.executable)?;
    verify_sys_attr(&sys, "prefix", &config.sys_prefix)?;
    verify_sys_attr(&sys, "base_prefix", &config.sys_base_prefix)
}

fn verify_interpreter_version(
    py: Python<'_>,
    config: &PythonRuntimeConfig,
) -> Result<(), PythonRuntimeError> {
    if config.cpython_version_tuple.len() < 2 {
        return Ok(());
    }
    let version = py.version_info();
    let expected_major = config.cpython_version_tuple[0];
    let expected_minor = config.cpython_version_tuple[1];
    if u64::from(version.major) == expected_major && u64::from(version.minor) == expected_minor {
        return Ok(());
    }
    Err(PythonRuntimeError::InterpreterVersionMismatch {
        expected: format!("{expected_major}.{expected_minor}"),
        actual: format!("{}.{}", version.major, version.minor),
    })
}

fn verify_sys_attr(
    sys: &Bound<'_, PyAny>,
    field: &'static str,
    expected: &str,
) -> Result<(), PythonRuntimeError> {
    if expected.is_empty() {
        return Ok(());
    }
    let actual = sys
        .getattr(field)
        .and_then(|value| value.extract::<String>())
        .map_err(|error| py_error(&error))?;
    if python_path_value_matches(expected, &actual) {
        return Ok(());
    }
    Err(PythonRuntimeError::InterpreterConfigMismatch {
        field,
        expected: expected.to_string(),
        actual,
    })
}

fn python_path_value_matches(expected: &str, actual: &str) -> bool {
    if expected == actual {
        return true;
    }
    matches!(
        (canonical_path(expected), canonical_path(actual)),
        (Some(expected), Some(actual)) if expected == actual
    )
}

fn canonical_path(value: &str) -> Option<std::path::PathBuf> {
    Path::new(value).canonicalize().ok()
}

#[cfg(test)]
mod tests {
    use super::python_path_value_matches;

    #[test]
    fn python_path_value_matches_existing_path_aliases() {
        let cwd = std::env::current_dir().expect("current dir should exist");

        assert!(python_path_value_matches(".", &cwd.display().to_string()));
    }

    #[test]
    fn python_path_value_rejects_missing_paths_with_different_spellings() {
        assert!(!python_path_value_matches(
            "/sifr/missing/python-a",
            "/sifr/missing/python-b"
        ));
    }
}
