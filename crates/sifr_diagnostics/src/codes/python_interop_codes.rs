use super::DiagnosticCode;
use crate::model::Severity;

impl DiagnosticCode {
    pub const PYENV_INVALID_CONFIG: Self = Self::new("SIFR-PYENV-0001", Severity::Error);
    pub const PYENV_MULTIPLE_SELECTIONS: Self = Self::new("SIFR-PYENV-0002", Severity::Error);
    pub const PYENV_MISSING_SELECTION: Self = Self::new("SIFR-PYENV-0003", Severity::Error);
    pub const PYENV_PROBE_FAILED: Self = Self::new("SIFR-PYENV-0004", Severity::Error);
    pub const PYENV_UNSUPPORTED_INTERPRETER: Self = Self::new("SIFR-PYENV-0005", Severity::Error);
    pub const PYENV_VENV_PREFIX_MISMATCH: Self = Self::new("SIFR-PYENV-0006", Severity::Error);
    pub const PYENV_SITE_PACKAGES_MISSING: Self = Self::new("SIFR-PYENV-0007", Severity::Error);
    pub const PYENV_DECLARED_IMPORT_MISSING: Self = Self::new("SIFR-PYENV-0008", Severity::Error);
    pub const PYENV_NATIVE_IMPORT_FAILED: Self = Self::new("SIFR-PYENV-0009", Severity::Error);
    pub const PYENV_FREE_THREADED_UNSUPPORTED: Self = Self::new("SIFR-PYENV-0010", Severity::Error);
    pub const PYENV_LOCK_OR_PROJECT_STALE: Self = Self::new("SIFR-PYENV-0011", Severity::Error);

    pub const PYIMP_INVALID_TARGET: Self = Self::new("SIFR-PYIMP-0001", Severity::Error);
    pub const PYIMP_INVALID_BRIDGE_SOURCE: Self = Self::new("SIFR-PYIMP-0002", Severity::Error);
    pub const PYCALL_INVALID_SHAPE: Self = Self::new("SIFR-PYCALL-0001", Severity::Error);
    pub const PYCONV_UNSUPPORTED_DECLARATION_TYPE: Self =
        Self::new("SIFR-PYCONV-0001", Severity::Error);
    pub const PYRES_UNIMPLEMENTED_DECLARATION: Self = Self::new("SIFR-PYRES-0002", Severity::Error);
    pub const PYCTX_INVALID_DECLARATION: Self = Self::new("SIFR-PYCTX-0001", Severity::Error);

    pub const PYTRUST_WILDCARD_REJECTED: Self = Self::new("SIFR-PYTRUST-0001", Severity::Error);
    pub const PYTRUST_REQUIRED_IMPORT_UNAUTHORIZED: Self =
        Self::new("SIFR-PYTRUST-0005", Severity::Error);
    pub const PYTRUST_UNTRUSTED_NATIVE_IMPORT: Self =
        Self::new("SIFR-PYTRUST-0003", Severity::Error);
    pub const PYTRUST_DYNAMIC_IMPORT_REQUIRES_TRUST: Self =
        Self::new("SIFR-PYTRUST-0004", Severity::Error);
}
