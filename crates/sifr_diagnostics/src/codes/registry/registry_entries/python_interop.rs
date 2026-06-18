//! Embedded Python interop diagnostics.

use super::super::DiagnosticRegistryEntry;
use crate::model::Severity;

pub(super) const ENTRIES: &[DiagnosticRegistryEntry] = &[
    active_entry!(
        "SIFR-PYENV-0001",
        "PYENV",
        "Python environment configuration is malformed.",
        Severity::Error,
        "crates/sifr_package/src/python/tests.rs::invalid_python_manifest_config_reports_pyenv_0001",
        "invalid Python environment configuration: {reason}",
        "sifr_package::python",
        [
            arg!("reason"),
            json_arg!("cargo_package_id"),
            json_arg!("manifest_path"),
            json_arg!("key")
        ],
        ["cargo_package_id", "manifest_path", "key", "reason"]
    ),
    active_entry!(
        "SIFR-PYENV-0002",
        "PYENV",
        "Package graph selects more than one Python virtual environment.",
        Severity::Error,
        "crates/sifr_package/src/python/tests.rs::multiple_python_environment_selections_report_pyenv_0002",
        "multiple Python environments are selected: {venvs}",
        "sifr_package::python",
        [arg!("venvs"), json_arg!("package_ids")],
        ["package_ids", "venvs"]
    ),
    active_entry!(
        "SIFR-PYENV-0003",
        "PYENV",
        "A package graph requires Python but no root environment is selected.",
        Severity::Error,
        "crates/sifr_package/src/python/tests.rs::missing_python_environment_selection_reports_pyenv_0003",
        "Python imports are required but no Python environment is selected",
        "sifr_package::python",
        [json_arg!("package_ids"), json_arg!("imports")],
        ["package_ids", "imports"]
    ),
    active_entry!(
        "SIFR-PYENV-0004",
        "PYENV",
        "Selected Python interpreter probe failed or returned invalid JSON.",
        Severity::Error,
        "crates/sifr_package/src/python/tests.rs::probe_rejects_missing_interpreter_with_pyenv_0004",
        "Python environment probe failed: {reason}",
        "sifr_package::python",
        [arg!("reason"), json_arg!("interpreter"), json_arg!("venv")],
        ["interpreter", "venv", "reason"]
    ),
    active_entry!(
        "SIFR-PYENV-0005",
        "PYENV",
        "Selected Python interpreter is not supported CPython.",
        Severity::Error,
        "crates/sifr_package/src/python/tests.rs::probe_rejects_non_cpython_json_with_pyenv_0005",
        "unsupported Python interpreter: {implementation}",
        "sifr_package::python",
        [arg!("implementation"), json_arg!("interpreter"), json_arg!("venv")],
        ["interpreter", "venv", "implementation"]
    ),
    active_entry!(
        "SIFR-PYENV-0006",
        "PYENV",
        "Selected interpreter does not belong to the configured virtual environment.",
        Severity::Error,
        "crates/sifr_package/src/python/tests.rs::probe_rejects_prefix_outside_venv_with_pyenv_0006",
        "Python sys.prefix is outside the selected virtual environment",
        "sifr_package::python",
        [
            json_arg!("interpreter"),
            json_arg!("venv"),
            json_arg!("sys_prefix")
        ],
        ["interpreter", "venv", "sys_prefix"]
    ),
    active_entry!(
        "SIFR-PYENV-0007",
        "PYENV",
        "Selected Python environment has no site-packages path.",
        Severity::Error,
        "crates/sifr_package/src/python/tests.rs::probe_rejects_missing_site_packages_with_pyenv_0007",
        "selected Python environment has no site-packages path",
        "sifr_package::python",
        [json_arg!("interpreter"), json_arg!("venv")],
        ["interpreter", "venv"]
    ),
    active_entry!(
        "SIFR-PYENV-0008",
        "PYENV",
        "Declared Python import root is missing from the selected environment.",
        Severity::Error,
        "crates/sifr_package/src/python/tests.rs::probe_rejects_missing_declared_import_with_pyenv_0008",
        "declared Python import root is missing: {import_root}",
        "sifr_package::python",
        [arg!("import_root"), json_arg!("interpreter"), json_arg!("venv")],
        ["interpreter", "venv", "import_root"]
    ),
    active_entry!(
        "SIFR-PYENV-0009",
        "PYENV",
        "Trusted native Python import root failed to load.",
        Severity::Error,
        "crates/sifr_package/src/python/tests.rs::probe_rejects_native_import_failure_with_pyenv_0009",
        "trusted native Python import root failed to load: {import_root}",
        "sifr_package::python",
        [
            arg!("import_root"),
            arg!("reason"),
            json_arg!("interpreter"),
            json_arg!("venv")
        ],
        ["interpreter", "venv", "import_root", "reason"]
    ),
    active_entry!(
        "SIFR-PYENV-0010",
        "PYENV",
        "Free-threaded CPython is not supported for embedded interop.",
        Severity::Error,
        "crates/sifr_package/src/python/tests.rs::probe_rejects_free_threaded_cpython_with_pyenv_0010",
        "free-threaded CPython is not supported",
        "sifr_package::python",
        [json_arg!("interpreter"), json_arg!("venv")],
        ["interpreter", "venv"]
    ),
    active_entry!(
        "SIFR-PYENV-0011",
        "PYENV",
        "Configured Python project or lockfile is missing or stale.",
        Severity::Error,
        "crates/sifr_package/src/python/tests.rs::probe_rejects_missing_lock_digest_with_pyenv_0011",
        "Python environment metadata is stale: {reason}",
        "sifr_package::python",
        [arg!("reason"), json_arg!("interpreter"), json_arg!("venv")],
        ["interpreter", "venv", "reason"]
    ),
];
