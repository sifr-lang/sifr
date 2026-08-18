pub(crate) struct PrivateReExportRule {
    pub(crate) public_module: &'static str,
    pub(crate) private_module: &'static str,
    pub(crate) names: &'static [&'static str],
    pub(crate) semantic_evidence: &'static str,
}

const MATH_EXPORTS: &[&str] = &[
    "sqrt",
    "floor",
    "ceil",
    "log",
    "cbrt",
    "sin",
    "cos",
    "tan",
    "pi",
    "e",
    "asin",
    "acos",
    "atan",
    "atan2",
    "sinh",
    "cosh",
    "tanh",
    "log10",
    "log2",
    "exp2",
    "degrees",
    "radians",
    "isnan",
    "isinf",
    "trunc",
    "copysign",
    "signbit",
    "fmod",
    "remainder",
    "hypot",
    "fma",
    "fmax",
    "fmin",
    "tau",
    "inf",
    "nan",
    "exp",
    "expm1",
    "log1p",
    "fabs",
    "isfinite",
    "isnormal",
    "issubnormal",
    "acosh",
    "asinh",
    "atanh",
    "isqrt",
    "erf",
    "erfc",
    "gamma",
    "lgamma",
    "frexp",
    "ldexp",
    "modf",
    "nextafter",
    "ulp",
];

const OS_FS_EXPORTS: &[&str] = &[
    "getcwd",
    "listdir",
    "mkdir",
    "rmdir",
    "remove_file",
    "rename",
    "is_file",
    "is_dir",
    "chdir",
    "disk_usage",
];

pub(crate) const PRIVATE_RE_EXPORT_RULES: &[PrivateReExportRule] = &[
    PrivateReExportRule {
        public_module: "sifr.http",
        private_module: "_sifr.http",
        names: &["HeaderError", "HttpError"],
        semantic_evidence: "typed public HTTP errors used by the wrapper API",
    },
    PrivateReExportRule {
        public_module: "sifr.math",
        private_module: "_sifr.math",
        names: MATH_EXPORTS,
        semantic_evidence:
            "canonical scalar math operations and constants with no second public wrapper",
    },
    PrivateReExportRule {
        public_module: "sifr.net",
        private_module: "_sifr.net",
        names: &["NetError"],
        semantic_evidence: "typed public network error used by the wrapper API",
    },
    PrivateReExportRule {
        public_module: "sifr.os",
        private_module: "_sifr.sys",
        names: &["run_command", "getpid", "cpu_count", "which"],
        semantic_evidence: "canonical process and host operations with no public alias",
    },
    PrivateReExportRule {
        public_module: "sifr.os",
        private_module: "_sifr.fs",
        names: OS_FS_EXPORTS,
        semantic_evidence:
            "canonical module-level filesystem operations; stat_size is excluded in favor of stat",
    },
    PrivateReExportRule {
        public_module: "sifr.process",
        private_module: "_sifr.process",
        names: &["ProcessError"],
        semantic_evidence: "typed public process error used by the wrapper API",
    },
    PrivateReExportRule {
        public_module: "sifr.python_core",
        private_module: "_sifr.python",
        names: &[
            "ExitCause",
            "ExitCauseKind",
            "ExitDecision",
            "Object",
            "PythonError",
        ],
        semantic_evidence: "canonical public Python handle, error, and context-exit contracts",
    },
    PrivateReExportRule {
        public_module: "sifr.runtime",
        private_module: "_sifr.runtime",
        names: &["DiagnosticError"],
        semantic_evidence: "typed public diagnostic emission error",
    },
    PrivateReExportRule {
        public_module: "sifr.shutil",
        private_module: "_sifr.fs",
        names: &["disk_usage"],
        semantic_evidence: "canonical disk-usage query distinct from file mutation wrappers",
    },
    PrivateReExportRule {
        public_module: "sifr.shutil",
        private_module: "_sifr.sys",
        names: &["which"],
        semantic_evidence: "canonical executable lookup with no second public wrapper",
    },
    PrivateReExportRule {
        public_module: "sifr.signal",
        private_module: "_sifr.signal",
        names: &["SignalError"],
        semantic_evidence: "typed public signal error used by the wrapper API",
    },
    PrivateReExportRule {
        public_module: "sifr.tempfile",
        private_module: "_sifr.fs",
        names: &["gettempdir"],
        semantic_evidence: "canonical temporary-directory query distinct from creation helpers",
    },
    PrivateReExportRule {
        public_module: "sifr.time",
        private_module: "_sifr.time",
        names: &["sleep", "perf_counter", "monotonic"],
        semantic_evidence:
            "canonical timer operations distinct from wall-clock time and calendar wrappers",
    },
    PrivateReExportRule {
        public_module: "sifr.tls",
        private_module: "_sifr.tls",
        names: &["TlsError"],
        semantic_evidence: "typed public TLS error used by the wrapper API",
    },
    PrivateReExportRule {
        public_module: "sifr.uuid",
        private_module: "_sifr.uuid",
        names: &["uuid4"],
        semantic_evidence:
            "uuid4 returns canonical text while uuid4_obj returns a typed UUID value",
    },
];

pub(crate) fn approved_private_re_export(
    public_module: &str,
    private_module: &str,
    source_name: &str,
    local_name: &str,
) -> bool {
    private_re_export_evidence(public_module, private_module, source_name, local_name).is_some()
}

pub(crate) fn private_re_export_evidence(
    public_module: &str,
    private_module: &str,
    source_name: &str,
    local_name: &str,
) -> Option<&'static str> {
    (source_name == local_name)
        .then(|| {
            PRIVATE_RE_EXPORT_RULES.iter().find_map(|rule| {
                (rule.public_module == public_module
                    && rule.private_module == private_module
                    && rule.names.contains(&source_name))
                .then_some(rule.semantic_evidence)
            })
        })
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn private_re_export_rules_are_unique_and_evidenced() {
        let mut keys = HashSet::new();
        for rule in PRIVATE_RE_EXPORT_RULES {
            assert!(!rule.semantic_evidence.trim().is_empty());
            assert!(!rule.names.is_empty());
            for name in rule.names {
                assert!(!name.starts_with('_'));
                assert!(
                    keys.insert((rule.public_module, rule.private_module, *name)),
                    "duplicate private re-export rule for {}:{}:{name}",
                    rule.public_module,
                    rule.private_module
                );
            }
        }
    }

    #[test]
    fn aliases_and_unapproved_direct_imports_are_not_public() {
        assert!(!approved_private_re_export(
            "sifr.math",
            "_sifr.math",
            "pow_val",
            "pow"
        ));
        assert!(!approved_private_re_export(
            "sifr.glob",
            "_sifr.fs",
            "listdir",
            "listdir"
        ));
        assert!(approved_private_re_export(
            "sifr.math",
            "_sifr.math",
            "sqrt",
            "sqrt"
        ));
    }
}
