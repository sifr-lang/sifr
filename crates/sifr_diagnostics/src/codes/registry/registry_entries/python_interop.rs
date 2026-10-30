//! Embedded Python interop diagnostics.

use super::super::DiagnosticRegistryEntry;
use crate::model::Severity;

pub(super) const ENTRIES: &[DiagnosticRegistryEntry] = &[
    active_entry!(
        "SIFR-PYENV-0001",
        "PYENV",
        "Python environment configuration is malformed.",
        Severity::Error,
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
        "unsupported Python interpreter: {implementation}",
        "sifr_package::python",
        [
            arg!("implementation"),
            json_arg!("interpreter"),
            json_arg!("venv")
        ],
        ["interpreter", "venv", "implementation"]
    ),
    active_entry!(
        "SIFR-PYENV-0006",
        "PYENV",
        "Selected interpreter does not belong to the configured virtual environment.",
        Severity::Error,
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
        "declared Python import root is missing: {import_root}",
        "sifr_package::python",
        [
            arg!("import_root"),
            json_arg!("interpreter"),
            json_arg!("venv")
        ],
        ["interpreter", "venv", "import_root"]
    ),
    active_entry!(
        "SIFR-PYENV-0009",
        "PYENV",
        "Trusted native Python import root failed to load.",
        Severity::Error,
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
        "Python environment metadata is stale: {reason}",
        "sifr_package::python",
        [arg!("reason"), json_arg!("interpreter"), json_arg!("venv")],
        ["interpreter", "venv", "reason"]
    ),
    active_entry!(
        "SIFR-PYTRUST-0001",
        "PYTRUST",
        "Dependency package declares a wildcard Python trust root.",
        Severity::Error,
        "Python wildcard import root is rejected",
        "sifr_package::python",
        [json_arg!("cargo_package_id")],
        ["cargo_package_id"]
    ),
    active_entry!(
        "SIFR-PYTRUST-0003",
        "PYTRUST",
        "Native Python import root is trusted but not required.",
        Severity::Error,
        "native Python import root is trusted but not required: {import_root}",
        "sifr_package::python",
        [arg!("import_root")],
        ["import_root"]
    ),
    active_entry!(
        "SIFR-PYTRUST-0004",
        "PYTRUST",
        "Dynamic Python import requires an explicit trust annotation.",
        Severity::Error,
        "dynamic Python import requires @trust_python_dynamic",
        "sifr_lowering",
        [],
        []
    ),
    active_entry!(
        "SIFR-PYTRUST-0005",
        "PYTRUST",
        "Required Python import root is not authorized by the root application.",
        Severity::Error,
        "required Python import root is not authorized by the root application: {import_root}",
        "sifr_package::python",
        [arg!("import_root")],
        ["import_root"]
    ),
    active_entry!(
        "SIFR-PYIMP-0001",
        "PYIMP",
        "A declaration-first Python target path is invalid or unresolved.",
        Severity::Error,
        "invalid Python declaration target: {reason}",
        "sifr_lowering",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-PYIMP-0002",
        "PYIMP",
        "A package-local Python bridge source or import inventory is invalid.",
        Severity::Error,
        "invalid package-local Python bridge: {reason}",
        "sifr_package::python",
        [
            arg!("reason"),
            json_arg!("cargo_package_id"),
            json_arg!("bridge_path")
        ],
        ["cargo_package_id", "bridge_path", "reason"]
    ),
    active_entry!(
        "SIFR-PYIMP-0003",
        "PYIMP",
        "The reserved embedded Python bridge namespace was already populated.",
        Severity::Error,
        "reserved Python bridge namespace collision at '{module}'",
        "sifr_runtime::python::bridge_loader",
        [arg!("module")],
        ["module"]
    ),
    active_entry!(
        "SIFR-PYCALL-0001",
        "PYCALL",
        "A Python declaration or call shape is unsupported or incompatible.",
        Severity::Error,
        "invalid Python declaration call shape: {reason}",
        "sifr_lowering",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-PYCONV-0001",
        "PYCONV",
        "A Sifr type has no active declaration-first Python conversion.",
        Severity::Error,
        "unsupported Python declaration conversion type: {reason}",
        "sifr_lowering",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-PYRES-0002",
        "PYRES",
        "A Python resource declaration uses an unsupported ownership or target shape.",
        Severity::Error,
        "unsupported Python resource declaration: {reason}",
        "sifr_lowering",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-PYCTX-0001",
        "PYCTX",
        "A Python context-manager entry, exit, or entered-value declaration is invalid.",
        Severity::Error,
        "invalid Python context declaration: {reason}",
        "sifr_lowering",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-PYCB-0001",
        "PYCB",
        "A Python callback lifetime, dispatch, concurrency, or owner declaration is invalid.",
        Severity::Error,
        "invalid Python callback declaration: {reason}",
        "sifr_lowering",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-PYZC-0001",
        "PYZC",
        "An advanced-data ownership, buffer layout, or hidden-copy declaration is invalid.",
        Severity::Error,
        "invalid Python zero-copy declaration: {reason}",
        "sifr_lowering",
        [arg!("reason")],
        ["reason"]
    ),
];
