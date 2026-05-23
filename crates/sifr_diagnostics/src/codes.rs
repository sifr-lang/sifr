use crate::model::Severity;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticCode {
    code: &'static str,
    declared_severity: Severity,
}

impl DiagnosticCode {
    pub const PARSE_EXPECTED_TOKEN_OR_RECOVERY: Self =
        Self::new("SIFR-PARSE-0002", Severity::Error);
    pub const PARSE_LEXICAL_OR_STRING: Self = Self::new("SIFR-PARSE-0003", Severity::Error);
    pub const PARSE_LAYOUT: Self = Self::new("SIFR-PARSE-0004", Severity::Error);
    pub const PARSE_INVALID_TARGET: Self = Self::new("SIFR-PARSE-0005", Severity::Error);
    pub const PARSE_INVALID_CALL_ARGUMENTS: Self = Self::new("SIFR-PARSE-0006", Severity::Error);
    pub const PARSE_MALFORMED_DECLARATION_LIST: Self =
        Self::new("SIFR-PARSE-0007", Severity::Error);
    pub const PARSE_INVALID_PATTERN: Self = Self::new("SIFR-PARSE-0008", Severity::Error);
    pub const PARSE_UNSUPPORTED_SYNTAX: Self = Self::new("SIFR-PARSE-0009", Severity::Error);

    pub const NAME_UNDEFINED_VARIABLE: Self = Self::new("SIFR-NAME-0001", Severity::Error);
    pub const NAME_UNDEFINED_CALLABLE: Self = Self::new("SIFR-NAME-0002", Severity::Error);
    pub const NAME_UNKNOWN_TYPE: Self = Self::new("SIFR-NAME-0003", Severity::Error);
    pub const NAME_MISSING_MODULE_MEMBER: Self = Self::new("SIFR-NAME-0004", Severity::Error);
    pub const NAME_DUPLICATE_DEFINITION: Self = Self::new("SIFR-NAME-0005", Severity::Error);
    pub const NAME_UNINITIALIZED_VARIABLE: Self = Self::new("SIFR-NAME-0006", Severity::Error);

    pub const IMPORT_FORBIDDEN_INTRINSIC: Self = Self::new("SIFR-IMPORT-0001", Severity::Error);
    pub const IMPORT_UNKNOWN_SOURCE_MODULE: Self = Self::new("SIFR-IMPORT-0002", Severity::Error);
    pub const IMPORT_UNSUPPORTED_FORM: Self = Self::new("SIFR-IMPORT-0003", Severity::Error);
    pub const IMPORT_PRIVATE_MEMBER: Self = Self::new("SIFR-IMPORT-0004", Severity::Error);

    pub const TYPE_MISMATCH: Self = Self::new("SIFR-TYPE-0002", Severity::Error);
    pub const TYPE_IF_BRANCH_MISMATCH: Self = Self::new("SIFR-TYPE-0003", Severity::Error);
    pub const TYPE_MISSING_ANNOTATION: Self = Self::new("SIFR-TYPE-0004", Severity::Error);
    pub const TYPE_UNSUPPORTED_OPERATOR: Self = Self::new("SIFR-TYPE-0005", Severity::Error);
    pub const TYPE_INT_BIGINT_MIXED: Self = Self::new("SIFR-TYPE-0006", Severity::Error);
    pub const TYPE_INVALID_ANNOTATION: Self = Self::new("SIFR-TYPE-0007", Severity::Error);
    pub const TYPE_CONTAINER_ELEMENT_CONFLICT: Self = Self::new("SIFR-TYPE-0008", Severity::Error);
    pub const TYPE_UNPACK_SHAPE_MISMATCH: Self = Self::new("SIFR-TYPE-0009", Severity::Error);
    pub const TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED: Self =
        Self::new("SIFR-TYPE-0010", Severity::Error);
    pub const TYPE_UNSUPPORTED_DEFAULT_ARGUMENT: Self =
        Self::new("SIFR-TYPE-0011", Severity::Error);
    pub const TYPE_UNSUPPORTED_EXPRESSION_FORM: Self = Self::new("SIFR-TYPE-0012", Severity::Error);
    pub const TYPE_ARITHMETIC_OVERFLOW_RISK: Self = Self::new("SIFR-TYPE-0901", Severity::Warning);
    pub const TYPE_REVEAL_TYPE: Self = Self::new("SIFR-TYPE-0902", Severity::Note);
    pub const ASYNC_NO_SUSPEND: Self = Self::new("SIFR-ASYNC-0001", Severity::Error);
    pub const ASYNC_AWAIT_NO_SUSPEND: Self = Self::new("SIFR-ASYNC-0002", Severity::Error);
    pub const ASYNC_DIRECT_BLOCKING_IO_CALL: Self = Self::new("SIFR-ASYNC-0003", Severity::Error);
    pub const ASYNC_DIRECT_CPU_HEAVY_CALL: Self = Self::new("SIFR-ASYNC-0004", Severity::Error);
    pub const ASYNC_UNCLASSIFIED_BLOCKING_OFFLOAD_TARGET: Self =
        Self::new("SIFR-ASYNC-0005", Severity::Error);
    pub const ASYNC_WORKLOAD_ANNOTATION_ON_ASYNC_DEF: Self =
        Self::new("SIFR-ASYNC-0006", Severity::Error);

    pub const DECIMAL_INVALID_LITERAL: Self = Self::new("SIFR-DECIMAL-0001", Severity::Error);
    pub const DECIMAL_BIGDECIMAL_INVALID_LITERAL: Self =
        Self::new("SIFR-DECIMAL-0002", Severity::Error);
    pub const DECIMAL_FLOAT_MIXED: Self = Self::new("SIFR-DECIMAL-0003", Severity::Error);
    pub const DECIMAL_MIXED_WITH_BIGDECIMAL: Self = Self::new("SIFR-DECIMAL-0004", Severity::Error);
    pub const DECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN: Self =
        Self::new("SIFR-DECIMAL-0005", Severity::Error);
    pub const DECIMAL_BIGDECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN: Self =
        Self::new("SIFR-DECIMAL-0006", Severity::Error);
    pub const DECIMAL_SCALE_INVALID: Self = Self::new("SIFR-DECIMAL-0007", Severity::Error);
    pub const DECIMAL_BIGDECIMAL_SCALE_OR_CONTEXT_INVALID: Self =
        Self::new("SIFR-DECIMAL-0008", Severity::Error);

    pub const INT_FIXED_WIDTH_OUT_OF_RANGE: Self = Self::new("SIFR-INT-0001", Severity::Error);
    pub const INT_RESERVED_WIDTH_NAME: Self = Self::new("SIFR-INT-0003", Severity::Error);
    pub const INT_EVAL_BUDGET_EXCEEDED: Self = Self::new("SIFR-INT-0004", Severity::Error);
    pub const INT_EXACT_DIVISION_REQUIRES_HANDLING: Self =
        Self::new("SIFR-INT-0005", Severity::Error);
    pub const INT_EXACT_TO_FLOAT_REQUIRES_HANDLING: Self =
        Self::new("SIFR-INT-0006", Severity::Error);
    pub const INT_BOOL_INTEGER_COMPARISON: Self = Self::new("SIFR-INT-0007", Severity::Error);
    pub const INT_BIGINT_TRANSITION_ALIAS: Self = Self::new("SIFR-INT-0011", Severity::Warning);

    pub const CALL_WRONG_POSITIONAL_COUNT: Self = Self::new("SIFR-CALL-0001", Severity::Error);
    pub const CALL_UNEXPECTED_KEYWORD: Self = Self::new("SIFR-CALL-0002", Severity::Error);
    pub const CALL_DUPLICATE_ARGUMENT: Self = Self::new("SIFR-CALL-0003", Severity::Error);
    pub const CALL_MISSING_REQUIRED_ARGUMENT: Self = Self::new("SIFR-CALL-0004", Severity::Error);
    pub const CALL_NOT_CALLABLE_OR_ARITY: Self = Self::new("SIFR-CALL-0005", Severity::Error);

    pub const OWN_USE_AFTER_MOVE: Self = Self::new("SIFR-OWN-0001", Severity::Error);
    pub const OWN_DOUBLE_MUTABLE_BORROW: Self = Self::new("SIFR-OWN-0002", Severity::Error);
    pub const OWN_BORROWED_PARAMETER_ESCAPES: Self = Self::new("SIFR-OWN-0003", Severity::Error);
    pub const OWN_MOVED_ACROSS_LOOP: Self = Self::new("SIFR-OWN-0004", Severity::Error);
    pub const OWN_IMMUTABLE_PARAMETER_MUTATION: Self = Self::new("SIFR-OWN-0005", Severity::Error);
    pub const OWN_IMMUTABLE_PARAMETER_REASSIGNMENT: Self =
        Self::new("SIFR-OWN-0006", Severity::Error);
    pub const OWN_IMMUTABLE_BYTES_ASSIGNMENT: Self = Self::new("SIFR-OWN-0007", Severity::Error);
    pub const OWN_IMMUTABLE_BYTES_AUGMENTED_ASSIGNMENT: Self =
        Self::new("SIFR-OWN-0008", Severity::Error);
    pub const OWN_BORROW_ACROSS_AWAIT: Self = Self::new("SIFR-OWN-0009", Severity::Error);
    pub const OWN_NON_SEND_TASK_CAPTURE: Self = Self::new("SIFR-OWN-0010", Severity::Error);
    pub const OWN_NON_SEND_CHANNEL_ELEMENT: Self = Self::new("SIFR-OWN-0011", Severity::Error);
    pub const OWN_NON_SHARE_SAFE_SHARED_VALUE: Self = Self::new("SIFR-OWN-0012", Severity::Error);

    pub const FLOW_BREAK_OUTSIDE_LOOP: Self = Self::new("SIFR-FLOW-0001", Severity::Error);
    pub const FLOW_CONTINUE_OUTSIDE_LOOP: Self = Self::new("SIFR-FLOW-0002", Severity::Error);
    pub const FLOW_INVALID_NONLOCAL: Self = Self::new("SIFR-FLOW-0003", Severity::Error);
    pub const FLOW_MISSING_RETURN_VALUE: Self = Self::new("SIFR-FLOW-0004", Severity::Error);
    pub const FLOW_INVALID_CONDITION_TYPE: Self = Self::new("SIFR-FLOW-0005", Severity::Error);
    pub const FLOW_UNSUPPORTED_STATEMENT_FORM: Self = Self::new("SIFR-FLOW-0006", Severity::Error);
    pub const FLOW_INVALID_ASSIGNMENT_TARGET: Self = Self::new("SIFR-FLOW-0007", Severity::Error);
    pub const FLOW_INVALID_ITERATION: Self = Self::new("SIFR-FLOW-0008", Severity::Error);
    pub const FLOW_UNREACHABLE_STATEMENT: Self = Self::new("SIFR-FLOW-0901", Severity::Warning);

    pub const FMT_FORMATTING_DRIFT: Self = Self::new("SIFR-FMT-0001", Severity::Error);

    pub const LINT_UNKNOWN_SUPPRESSION: Self = Self::new("SIFR-LINT-0001", Severity::Warning);
    pub const LINT_UNUSED_SUPPRESSION: Self = Self::new("SIFR-LINT-0002", Severity::Warning);
    pub const LINT_BLANKET_SUPPRESSION: Self = Self::new("SIFR-LINT-0003", Severity::Warning);
    pub const LINT_TRAILING_WHITESPACE: Self = Self::new("SIFR-LINT-0004", Severity::Warning);

    pub const MATCH_NON_EXHAUSTIVE: Self = Self::new("SIFR-MATCH-0001", Severity::Error);
    pub const MATCH_GUARD_NOT_BOOL: Self = Self::new("SIFR-MATCH-0002", Severity::Error);
    pub const MATCH_INVALID_CLASS_PATTERN_FIELD: Self =
        Self::new("SIFR-MATCH-0003", Severity::Error);
    pub const MATCH_INVALID_PATTERN_FORM: Self = Self::new("SIFR-MATCH-0004", Severity::Error);

    pub const PROTO_BOUND_NOT_SATISFIED: Self = Self::new("SIFR-PROTO-0001", Severity::Error);
    pub const PROTO_INVALID_ITERATOR_SIGNATURE: Self =
        Self::new("SIFR-PROTO-0002", Severity::Error);
    pub const PROTO_CONTEXT_MANAGER_MISSING: Self = Self::new("SIFR-PROTO-0003", Severity::Error);
    pub const PROTO_HASHABLE_OR_COMPARABLE_REQUIRED: Self =
        Self::new("SIFR-PROTO-0004", Severity::Error);

    pub const CLASS_MISSING_INITIALIZER: Self = Self::new("SIFR-CLASS-0001", Severity::Error);
    pub const CLASS_REQUIRED_FIELD_AFTER_DEFAULT: Self =
        Self::new("SIFR-CLASS-0002", Severity::Error);
    pub const CLASS_DUPLICATE_OR_INVALID_VALUE: Self =
        Self::new("SIFR-CLASS-0003", Severity::Error);
    pub const CLASS_MISSING_MEMBER: Self = Self::new("SIFR-CLASS-0004", Severity::Error);
    pub const CLASS_INVALID_BASE: Self = Self::new("SIFR-CLASS-0005", Severity::Error);
    pub const CLASS_UNSUPPORTED_DECLARATION: Self = Self::new("SIFR-CLASS-0006", Severity::Error);

    pub const RESULT_UNUSED_VALUE: Self = Self::new("SIFR-RESULT-0001", Severity::Error);
    pub const RESULT_INVALID_ERROR_TYPE: Self = Self::new("SIFR-RESULT-0002", Severity::Error);
    pub const RESULT_INVALID_RAISE: Self = Self::new("SIFR-RESULT-0003", Severity::Error);
    pub const RESULT_UNKNOWN_EXCEPT_TYPE: Self = Self::new("SIFR-RESULT-0004", Severity::Error);
    pub const RESULT_UNCOVERED_TRY_ERRORS: Self = Self::new("SIFR-RESULT-0005", Severity::Error);
    pub const RESULT_INVALID_EXCEPT_TYPE: Self = Self::new("SIFR-RESULT-0006", Severity::Error);

    pub const STDLIB_UNSUPPORTED_SURFACE: Self = Self::new("SIFR-STDLIB-0001", Severity::Error);
    pub const STDLIB_BOOTSTRAP_FAILURE: Self = Self::new("SIFR-STDLIB-0003", Severity::Error);
    pub const STDLIB_CACHE_FAILURE: Self = Self::new("SIFR-STDLIB-0004", Severity::Error);

    pub const WORKSPACE_MALFORMED_MANIFEST: Self =
        Self::new("SIFR-WORKSPACE-0001", Severity::Error);
    pub const WORKSPACE_SOURCE_ROOT_ESCAPES: Self =
        Self::new("SIFR-WORKSPACE-0002", Severity::Error);
    pub const WORKSPACE_SOURCE_ROOT_NOT_DIRECTORY: Self =
        Self::new("SIFR-WORKSPACE-0003", Severity::Error);
    pub const WORKSPACE_INVALID_SOURCE_ROOT: Self =
        Self::new("SIFR-WORKSPACE-0004", Severity::Error);
    pub const WORKSPACE_UNRESOLVED_IMPORT: Self = Self::new("SIFR-WORKSPACE-0101", Severity::Error);
    pub const WORKSPACE_AMBIGUOUS_IMPORT: Self = Self::new("SIFR-WORKSPACE-0102", Severity::Error);
    pub const WORKSPACE_NAMESPACE_COLLISION: Self =
        Self::new("SIFR-WORKSPACE-0103", Severity::Error);
    pub const WORKSPACE_IMPORT_CYCLE: Self = Self::new("SIFR-WORKSPACE-0104", Severity::Error);

    pub const PACKAGE_MISSING_OR_INVALID_CARGO_METADATA: Self =
        Self::new("SIFR-PACKAGE-0001", Severity::Error);
    pub const PACKAGE_MISSING_OR_INVALID_SIFR_MANIFEST: Self =
        Self::new("SIFR-PACKAGE-0002", Severity::Error);
    pub const PACKAGE_UNSUPPORTED_CARGO_SIFR_METADATA: Self =
        Self::new("SIFR-PACKAGE-0003", Severity::Error);
    pub const PACKAGE_CARGO_COMMAND_FAILED: Self = Self::new("SIFR-PACKAGE-0101", Severity::Error);
    pub const PACKAGE_SELECTED_RUST_ONLY: Self = Self::new("SIFR-PACKAGE-0102", Severity::Error);
    pub const PACKAGE_METADATA_PARSE: Self = Self::new("SIFR-PACKAGE-0103", Severity::Error);
    pub const PACKAGE_SOURCE_UNAVAILABLE_OFFLINE: Self =
        Self::new("SIFR-PACKAGE-0104", Severity::Error);
    pub const PACKAGE_RUST_ONLY_DEPENDS_ON_SIFR: Self =
        Self::new("SIFR-PACKAGE-0106", Severity::Error);
    pub const PACKAGE_AMBIGUOUS_IMPORT_ROOT: Self = Self::new("SIFR-PACKAGE-0201", Severity::Error);
    pub const PACKAGE_UNDECLARED_DIRECT_IMPORT: Self =
        Self::new("SIFR-PACKAGE-0202", Severity::Error);
    pub const PACKAGE_PRIVATE_MODULE_ACCESS: Self = Self::new("SIFR-PACKAGE-0203", Severity::Error);
    pub const PACKAGE_TYPE_IDENTITY_MISMATCH: Self =
        Self::new("SIFR-PACKAGE-0204", Severity::Error);
    pub const PACKAGE_CIRCULAR_PATH_DEPENDENCY: Self =
        Self::new("SIFR-PACKAGE-0205", Severity::Error);
    pub const PACKAGE_BACKEND_TRUST_VIOLATION: Self =
        Self::new("SIFR-PACKAGE-0301", Severity::Error);
    pub const PACKAGE_FEATURE_CARGO_PACKAGE_UNAVAILABLE: Self =
        Self::new("SIFR-PACKAGE-0303", Severity::Error);
    pub const PACKAGE_FEATURE_CARGO_FEATURE_UNAVAILABLE: Self =
        Self::new("SIFR-PACKAGE-0304", Severity::Error);
    pub const PACKAGE_TRUST_NON_DIRECT_DEPENDENCY: Self =
        Self::new("SIFR-PACKAGE-0305", Severity::Error);
    pub const PACKAGE_ARCHIVE_MISSING_SIFR_SOURCE: Self =
        Self::new("SIFR-PACKAGE-0401", Severity::Error);
    pub const PACKAGE_PUBLISH_VALIDATION_FAILED: Self =
        Self::new("SIFR-PACKAGE-0402", Severity::Error);
    pub const PACKAGE_INCLUDE_EXCLUDE_OMITS_SOURCE: Self =
        Self::new("SIFR-PACKAGE-0403", Severity::Error);
    pub const PACKAGE_ARCHIVE_TRAVERSAL: Self = Self::new("SIFR-PACKAGE-0404", Severity::Error);
    pub const PACKAGE_NON_TRIVIAL_PURE_MARKER: Self =
        Self::new("SIFR-PACKAGE-0501", Severity::Error);
    pub const PACKAGE_SELECTOR_AMBIGUOUS: Self = Self::new("SIFR-PACKAGE-0601", Severity::Error);
    pub const PACKAGE_DUPLICATE_WORKSPACE_IMPORT_ROOT: Self =
        Self::new("SIFR-PACKAGE-0602", Severity::Error);
    pub const PACKAGE_CHANGED_FILE_MAPPING_FAILED: Self =
        Self::new("SIFR-PACKAGE-0603", Severity::Error);
    pub const PACKAGE_OUTDATED_QUERY_UNSUPPORTED: Self =
        Self::new("SIFR-PACKAGE-0604", Severity::Error);
    pub const PACKAGE_RUN_TARGET_AMBIGUOUS: Self = Self::new("SIFR-PACKAGE-0605", Severity::Error);
    pub const PACKAGE_INVALID_APP_TARGET_NAME: Self =
        Self::new("SIFR-PACKAGE-0606", Severity::Error);
    pub const PACKAGE_DUPLICATE_WORKSPACE_SIFR_NAME: Self =
        Self::new("SIFR-PACKAGE-0607", Severity::Error);
    pub const PACKAGE_MANIFEST_EXPORTS_NOT_PRODUCTION: Self =
        Self::new("SIFR-PACKAGE-0701", Severity::Error);
    pub const PACKAGE_PROJECTION_MANIFEST_POINTER_DRIFT: Self =
        Self::new("SIFR-PACKAGE-0703", Severity::Error);
    pub const PACKAGE_PROJECTION_INCLUDE_DRIFT: Self =
        Self::new("SIFR-PACKAGE-0704", Severity::Error);
    pub const PACKAGE_PROJECTION_PURE_MARKER_MISSING: Self =
        Self::new("SIFR-PACKAGE-0709", Severity::Error);
    pub const PACKAGE_EXPLICIT_FILE_OUTSIDE_SOURCE_ROOT: Self =
        Self::new("SIFR-PACKAGE-0710", Severity::Error);
    pub const PACKAGE_MANIFEST_BIN_TABLES_NOT_PRODUCTION: Self =
        Self::new("SIFR-PACKAGE-0711", Severity::Error);
    pub const PACKAGE_DUPLICATE_PUBLIC_API_SYMBOL: Self =
        Self::new("SIFR-PACKAGE-0713", Severity::Error);
    pub const PACKAGE_SCRIPT_RECURSION: Self = Self::new("SIFR-PACKAGE-0714", Severity::Error);

    pub const BUILD_MATERIALIZATION_FAILURE: Self = Self::new("SIFR-BUILD-0002", Severity::Error);
    pub const BUILD_TEMP_WORKSPACE_FAILURE: Self = Self::new("SIFR-BUILD-0003", Severity::Error);
    pub const BUILD_CARGO_MANIFEST_FAILURE: Self = Self::new("SIFR-BUILD-0004", Severity::Error);
    pub const BUILD_RUSTC_OR_CARGO_FAILURE: Self = Self::new("SIFR-BUILD-0005", Severity::Error);
    pub const BUILD_ARTIFACT_MISSING: Self = Self::new("SIFR-BUILD-0006", Severity::Error);

    pub const INTERNAL_COMPILER_PANIC: Self = Self::new("SIFR-INTERNAL-0001", Severity::Error);
    pub const INTERNAL_RECOVERY_OMISSION_SUMMARY: Self =
        Self::new("SIFR-INTERNAL-0002", Severity::Note);

    #[cfg(test)]
    pub(crate) const TEST_INTERNAL_ERROR: Self = Self::new("SIFR-INTERNAL-9998", Severity::Error);
    #[cfg(test)]
    pub(crate) const TEST_NOTE: Self = Self::new("SIFR-INTERNAL-9999", Severity::Note);
    #[cfg(test)]
    pub(crate) const TEST_SOURCE_ERROR: Self = Self::new("SIFR-NAME-9999", Severity::Error);

    const fn new(code: &'static str, declared_severity: Severity) -> Self {
        Self {
            code,
            declared_severity,
        }
    }

    #[must_use]
    pub const fn code(self) -> &'static str {
        self.code
    }

    #[must_use]
    pub const fn declared_severity(self) -> Severity {
        self.declared_severity
    }

    #[must_use]
    pub fn docs_url(self) -> String {
        format!("https://sifr.sh/docs/errors/{}", self.code())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DiagnosticState {
    Active,
    Reserved,
}

impl DiagnosticState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Reserved => "Reserved",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DiagnosticArgFormat {
    MessageAndJson,
    JsonOnly,
}

impl DiagnosticArgFormat {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MessageAndJson => "message+json",
            Self::JsonOnly => "json-only",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DiagnosticArgDeclaration {
    pub name: &'static str,
    pub format: DiagnosticArgFormat,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DiagnosticTooling {
    pub tool_actions: &'static [&'static str],
    pub fix_all_eligible: bool,
}

impl DiagnosticTooling {
    pub const DEFAULT: Self = Self {
        tool_actions: &[],
        fix_all_eligible: false,
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DiagnosticFamily {
    pub name: &'static str,
    pub summary: &'static str,
    pub reserved_base: &'static str,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DiagnosticRegistryEntry {
    pub id: &'static str,
    pub family: &'static str,
    pub summary: &'static str,
    pub state: DiagnosticState,
    pub docs_path: &'static str,
    pub representative_fixture_path: Option<&'static str>,
    pub message_template: Option<&'static str>,
    pub owner_module: Option<&'static str>,
    pub declared_args: &'static [DiagnosticArgDeclaration],
    pub dedupe_args: &'static [&'static str],
    pub declared_severity: Option<Severity>,
    pub tooling: DiagnosticTooling,
}

pub const DIAGNOSTIC_FAMILIES: &[DiagnosticFamily] = &[
    DiagnosticFamily {
        name: "PARSE",
        summary: "Parsing and source syntax diagnostics.",
        reserved_base: "SIFR-PARSE-0000",
    },
    DiagnosticFamily {
        name: "NAME",
        summary: "Name binding and resolution diagnostics.",
        reserved_base: "SIFR-NAME-0000",
    },
    DiagnosticFamily {
        name: "IMPORT",
        summary: "Module import and path resolution diagnostics.",
        reserved_base: "SIFR-IMPORT-0000",
    },
    DiagnosticFamily {
        name: "TYPE",
        summary: "Static type compatibility and inference diagnostics.",
        reserved_base: "SIFR-TYPE-0000",
    },
    DiagnosticFamily {
        name: "ASYNC",
        summary: "Async effect, awaitability, and blocking-offload diagnostics.",
        reserved_base: "SIFR-ASYNC-0000",
    },
    DiagnosticFamily {
        name: "DECIMAL",
        summary: "Decimal literal and fixed-point arithmetic diagnostics.",
        reserved_base: "SIFR-DECIMAL-0000",
    },
    DiagnosticFamily {
        name: "INT",
        summary: "Exact and fixed-width integer model diagnostics.",
        reserved_base: "SIFR-INT-0000",
    },
    DiagnosticFamily {
        name: "CALL",
        summary: "Function, method, constructor, and overload call diagnostics.",
        reserved_base: "SIFR-CALL-0000",
    },
    DiagnosticFamily {
        name: "OWN",
        summary: "Ownership, borrow, move, and lifetime diagnostics.",
        reserved_base: "SIFR-OWN-0000",
    },
    DiagnosticFamily {
        name: "FLOW",
        summary: "Control-flow, reachability, and narrowing diagnostics.",
        reserved_base: "SIFR-FLOW-0000",
    },
    DiagnosticFamily {
        name: "FMT",
        summary: "Source formatting diagnostics.",
        reserved_base: "SIFR-FMT-0000",
    },
    DiagnosticFamily {
        name: "LINT",
        summary: "Suppressible policy-rule diagnostics.",
        reserved_base: "SIFR-LINT-0000",
    },
    DiagnosticFamily {
        name: "MATCH",
        summary: "Pattern matching and exhaustiveness diagnostics.",
        reserved_base: "SIFR-MATCH-0000",
    },
    DiagnosticFamily {
        name: "PROTO",
        summary: "Protocol and structural conformance diagnostics.",
        reserved_base: "SIFR-PROTO-0000",
    },
    DiagnosticFamily {
        name: "CLASS",
        summary: "Class declaration, constructor, field, and method diagnostics.",
        reserved_base: "SIFR-CLASS-0000",
    },
    DiagnosticFamily {
        name: "RESULT",
        summary: "Result, Option, and checked error-flow diagnostics.",
        reserved_base: "SIFR-RESULT-0000",
    },
    DiagnosticFamily {
        name: "STDLIB",
        summary: "Standard-library surface and intrinsic contract diagnostics.",
        reserved_base: "SIFR-STDLIB-0000",
    },
    DiagnosticFamily {
        name: "WORKSPACE",
        summary: "Workspace, package, manifest, and project discovery diagnostics.",
        reserved_base: "SIFR-WORKSPACE-0000",
    },
    DiagnosticFamily {
        name: "PACKAGE",
        summary: "Cargo-backed Sifr package coordination diagnostics.",
        reserved_base: "SIFR-PACKAGE-0000",
    },
    DiagnosticFamily {
        name: "CODEGEN",
        summary: "Rust lowering and backend code-generation diagnostics.",
        reserved_base: "SIFR-CODEGEN-0000",
    },
    DiagnosticFamily {
        name: "BUILD",
        summary: "Build orchestration, rustc, linker, and artifact diagnostics.",
        reserved_base: "SIFR-BUILD-0000",
    },
    DiagnosticFamily {
        name: "INTERNAL",
        summary: "Compiler invariant and internal failure diagnostics.",
        reserved_base: "SIFR-INTERNAL-0000",
    },
];

macro_rules! arg {
    ($name:literal) => {
        DiagnosticArgDeclaration {
            name: $name,
            format: DiagnosticArgFormat::MessageAndJson,
        }
    };
}

macro_rules! json_arg {
    ($name:literal) => {
        DiagnosticArgDeclaration {
            name: $name,
            format: DiagnosticArgFormat::JsonOnly,
        }
    };
}

macro_rules! active_entry {
    ($id:literal, $family:literal, $summary:literal, $severity:expr, $fixture:literal, $template:literal, $owner:literal, [$($arg:expr),* $(,)?], [$($dedupe:literal),* $(,)?]) => {
        DiagnosticRegistryEntry {
            id: $id,
            family: $family,
            summary: $summary,
            state: DiagnosticState::Active,
            docs_path: concat!("docs/errors/", $id, ".md"),
            representative_fixture_path: Some($fixture),
            message_template: Some($template),
            owner_module: Some($owner),
            declared_args: &[$($arg),*],
            dedupe_args: &[$($dedupe),*],
            declared_severity: Some($severity),
            tooling: DiagnosticTooling::DEFAULT,
        }
    };
}

pub const DIAGNOSTIC_REGISTRY: &[DiagnosticRegistryEntry] = &[
    reserved_family_base("SIFR-PARSE-0000", "PARSE"),
    reserved_family_base("SIFR-NAME-0000", "NAME"),
    reserved_family_base("SIFR-IMPORT-0000", "IMPORT"),
    reserved_family_base("SIFR-TYPE-0000", "TYPE"),
    reserved_family_base("SIFR-ASYNC-0000", "ASYNC"),
    reserved_family_base("SIFR-DECIMAL-0000", "DECIMAL"),
    reserved_family_base("SIFR-INT-0000", "INT"),
    reserved_code(
        "SIFR-INT-0002",
        "INT",
        "Reserved for implicit narrowing from exact or fixed-width integer sources to narrower fixed-width targets.",
    ),
    reserved_code(
        "SIFR-INT-0008",
        "INT",
        "Reserved for fixed-width array, tensor, or dataframe arithmetic without an explicit overflow policy.",
    ),
    reserved_code(
        "SIFR-INT-0009",
        "INT",
        "Reserved for JSON or web-safe integer serialization policy failures.",
    ),
    reserved_code(
        "SIFR-INT-0010",
        "INT",
        "Reserved for bytes or bytearray construction and mutation values that do not fit uint8.",
    ),
    reserved_code(
        "SIFR-TYPE-0903",
        "TYPE",
        "Retired: direct annotated workload calls from async code are now ASYNC-family errors.",
    ),
    reserved_family_base("SIFR-CALL-0000", "CALL"),
    reserved_family_base("SIFR-OWN-0000", "OWN"),
    reserved_family_base("SIFR-FLOW-0000", "FLOW"),
    reserved_family_base("SIFR-FMT-0000", "FMT"),
    reserved_family_base("SIFR-LINT-0000", "LINT"),
    reserved_family_base("SIFR-MATCH-0000", "MATCH"),
    reserved_family_base("SIFR-PROTO-0000", "PROTO"),
    reserved_family_base("SIFR-CLASS-0000", "CLASS"),
    reserved_family_base("SIFR-RESULT-0000", "RESULT"),
    reserved_family_base("SIFR-STDLIB-0000", "STDLIB"),
    reserved_family_base("SIFR-WORKSPACE-0000", "WORKSPACE"),
    reserved_family_base("SIFR-PACKAGE-0000", "PACKAGE"),
    reserved_code(
        "SIFR-PACKAGE-0105",
        "PACKAGE",
        "Retired: Cargo credential failures are wrapped by SIFR-PACKAGE-0101.",
    ),
    reserved_code(
        "SIFR-PACKAGE-0302",
        "PACKAGE",
        "Reserved for future backend trust diagnostics.",
    ),
    reserved_code(
        "SIFR-PACKAGE-0306",
        "PACKAGE",
        "Reserved for future backend trust and feature diagnostics.",
    ),
    reserved_code(
        "SIFR-PACKAGE-0307",
        "PACKAGE",
        "Reserved for future backend trust and feature diagnostics.",
    ),
    reserved_code(
        "SIFR-PACKAGE-0308",
        "PACKAGE",
        "Reserved for future backend trust and feature diagnostics.",
    ),
    reserved_code(
        "SIFR-PACKAGE-0309",
        "PACKAGE",
        "Reserved for future backend trust and feature diagnostics.",
    ),
    reserved_family_base("SIFR-CODEGEN-0000", "CODEGEN"),
    reserved_family_base("SIFR-BUILD-0000", "BUILD"),
    reserved_family_base("SIFR-INTERNAL-0000", "INTERNAL"),
    active_entry!(
        "SIFR-PARSE-0002",
        "PARSE",
        "Expected token or generic parser recovery failure.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/parser_expected_token.sifr",
        "syntax error: expected {expected}",
        "sifr_driver::frontend::api",
        [arg!("expected"), json_arg!("parser_category")],
        ["expected", "parser_category"]
    ),
    active_entry!(
        "SIFR-PARSE-0003",
        "PARSE",
        "Lexical or interpolated string parser failure.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/parser_malformed_string.sifr",
        "lexical error: {reason}",
        "sifr_driver::frontend::api",
        [arg!("reason"), json_arg!("parser_category")],
        ["reason", "parser_category"]
    ),
    active_entry!(
        "SIFR-PARSE-0004",
        "PARSE",
        "Indentation or same-line statement layout parser failure.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/parser_invalid_layout.sifr",
        "invalid source layout: {reason}",
        "sifr_driver::frontend::api",
        [arg!("reason"), json_arg!("parser_category")],
        ["reason", "parser_category"]
    ),
    active_entry!(
        "SIFR-PARSE-0005",
        "PARSE",
        "Invalid assignment, delete, starred, or named-expression target syntax.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/parser_invalid_target.sifr",
        "invalid target syntax: {target_kind}",
        "sifr_driver::frontend::api",
        [arg!("target_kind"), json_arg!("parser_category")],
        ["target_kind", "parser_category"]
    ),
    active_entry!(
        "SIFR-PARSE-0006",
        "PARSE",
        "Invalid call argument order or unpacking syntax.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/parser_invalid_call_arguments.sifr",
        "invalid call argument syntax: {reason}",
        "sifr_driver::frontend::api",
        [arg!("reason"), json_arg!("parser_category")],
        ["reason", "parser_category"]
    ),
    active_entry!(
        "SIFR-PARSE-0007",
        "PARSE",
        "Empty or malformed declaration list syntax.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/parser_malformed_declaration_list.sifr",
        "malformed declaration list: {declaration_kind}",
        "sifr_driver::frontend::api",
        [arg!("declaration_kind"), json_arg!("parser_category")],
        ["declaration_kind", "parser_category"]
    ),
    active_entry!(
        "SIFR-PARSE-0008",
        "PARSE",
        "Invalid match-pattern syntax.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/parser_invalid_match_pattern.sifr",
        "invalid match pattern syntax: {reason}",
        "sifr_driver::frontend::api",
        [arg!("reason"), json_arg!("parser_category")],
        ["reason", "parser_category"]
    ),
    active_entry!(
        "SIFR-PARSE-0009",
        "PARSE",
        "Unsupported parser syntax or interactive-only syntax.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/parser_unsupported_syntax.sifr",
        "unsupported syntax: {syntax_kind}",
        "sifr_driver::frontend::api",
        [arg!("syntax_kind"), json_arg!("parser_category")],
        ["syntax_kind", "parser_category"]
    ),
    active_entry!(
        "SIFR-NAME-0001",
        "NAME",
        "Undefined variable.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/undefined_var.sifr",
        "undefined variable: '{name}'",
        "sifr_hir::lower",
        [arg!("name")],
        ["name"]
    ),
    active_entry!(
        "SIFR-NAME-0002",
        "NAME",
        "Undefined function or callable.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/undefined_function.sifr",
        "undefined function: '{name}'",
        "sifr_hir::lower",
        [arg!("name")],
        ["name"]
    ),
    active_entry!(
        "SIFR-NAME-0003",
        "NAME",
        "Unknown type or generic type name.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/generic_class_missing_type_arg.sifr",
        "unknown type: {name}",
        "sifr_hir::lower::typing_and_functions",
        [arg!("name")],
        ["name"]
    ),
    active_entry!(
        "SIFR-NAME-0004",
        "NAME",
        "Missing module or class member.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/stdlib_missing_function.sifr",
        "module '{container}' has no member '{member}'",
        "sifr_hir::lower",
        [arg!("member"), arg!("container")],
        ["member", "container"]
    ),
    active_entry!(
        "SIFR-NAME-0005",
        "NAME",
        "Duplicate function definition in a module.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/duplicate_function_definition.sifr",
        "duplicate function definition in module: '{name}'",
        "sifr_hir::lower::module_function_registry",
        [arg!("name")],
        ["name"]
    ),
    active_entry!(
        "SIFR-NAME-0006",
        "NAME",
        "Variable declaration lacks a required initializer.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/annotated_variable_requires_initializer.sifr",
        "variable '{name}' must be initialized",
        "sifr_hir::lower::statements",
        [arg!("name")],
        ["name"]
    ),
    active_entry!(
        "SIFR-IMPORT-0001",
        "IMPORT",
        "Forbidden intrinsic import.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/stdlib_intrinsic_direct_import.sifr",
        "cannot import from '{module}' — _sifr.* modules are internal compiler intrinsics",
        "sifr_hir::lower",
        [arg!("module")],
        ["module"]
    ),
    active_entry!(
        "SIFR-IMPORT-0002",
        "IMPORT",
        "Unknown source module import target.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/import_nonexistent_local.sifr",
        "unknown import target: '{module}'",
        "sifr_hir::lower",
        [arg!("module")],
        ["module"]
    ),
    active_entry!(
        "SIFR-IMPORT-0003",
        "IMPORT",
        "Unsupported import statement form.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/unsupported_import_statement.sifr",
        "unsupported import form: {form}",
        "sifr_hir::lower",
        [arg!("form")],
        ["form"]
    ),
    active_entry!(
        "SIFR-IMPORT-0004",
        "IMPORT",
        "Private module member import.",
        Severity::Error,
        "crates/sifr_hir/src/lower/name_import_diagnostics_tests.rs",
        "cannot import private name '{name}' from module '{module}'",
        "sifr_hir::lower",
        [arg!("name"), arg!("module")],
        ["name", "module"]
    ),
    active_entry!(
        "SIFR-TYPE-0002",
        "TYPE",
        "Expected and actual types are incompatible.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/type_mismatch.sifr",
        "type mismatch: expected {expected}, got {actual}",
        "sifr_hir::lower",
        [arg!("expected"), arg!("actual")],
        ["expected", "actual"]
    ),
    active_entry!(
        "SIFR-TYPE-0003",
        "TYPE",
        "If-expression or conditional branches have incompatible types.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/ternary_type_mismatch.sifr",
        "conditional branches have incompatible types: {then_type} and {else_type}",
        "sifr_hir::lower::if_expression",
        [arg!("then_type"), arg!("else_type")],
        ["then_type", "else_type"]
    ),
    active_entry!(
        "SIFR-TYPE-0004",
        "TYPE",
        "A required type annotation is missing.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/missing_type_annotation.sifr",
        "missing type annotation for {name}",
        "sifr_hir::lower::typing_and_functions",
        [arg!("name"), json_arg!("declaration_kind")],
        ["name", "declaration_kind"]
    ),
    active_entry!(
        "SIFR-TYPE-0005",
        "TYPE",
        "Unsupported operator or operand types.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/optional_arithmetic_without_narrowing.sifr",
        "unsupported operator {operator} for {operand_types}",
        "sifr_type_system",
        [arg!("operator"), arg!("operand_types")],
        ["operator", "operand_types"]
    ),
    active_entry!(
        "SIFR-TYPE-0006",
        "TYPE",
        "Int and bigint are mixed without an explicit conversion.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/bigint_int_mixed_arithmetic.sifr",
        "cannot mix int and bigint with operator {operator}",
        "sifr_type_system",
        [arg!("operator")],
        ["operator"]
    ),
    active_entry!(
        "SIFR-TYPE-0007",
        "TYPE",
        "Invalid type annotation shape.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/invalid_type_annotation.sifr",
        "invalid type annotation for {annotation_kind}",
        "sifr_hir::lower::typing_and_functions",
        [arg!("annotation_kind")],
        ["annotation_kind"]
    ),
    active_entry!(
        "SIFR-TYPE-0008",
        "TYPE",
        "Container literal elements, keys, or values have conflicting types.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/container_literal_type_conflict.sifr",
        "container literal has conflicting {element_kind} types: {expected} and {actual}",
        "sifr_hir::lower::container_literal_specialization",
        [arg!("element_kind"), arg!("expected"), arg!("actual")],
        ["element_kind", "expected", "actual"]
    ),
    active_entry!(
        "SIFR-TYPE-0009",
        "TYPE",
        "Tuple or list unpacking shape mismatch.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/tuple_unpack_shape_mismatch.sifr",
        "tuple unpacking: expected {expected_count} values, got {actual_count}",
        "sifr_hir::lower::tuple_unpack",
        [arg!("expected_count"), arg!("actual_count")],
        ["expected_count", "actual_count"]
    ),
    active_entry!(
        "SIFR-TYPE-0010",
        "TYPE",
        "TypeVar constraints are not satisfied by the inferred concrete type.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/typevar_constraints_violation.sifr",
        "type '{actual}' does not satisfy constraints ({constraints}) required by type parameter '{type_param}'",
        "sifr_hir::lower::expressions",
        [arg!("actual"), arg!("constraints"), arg!("type_param")],
        ["actual", "constraints", "type_param"]
    ),
    active_entry!(
        "SIFR-TYPE-0011",
        "TYPE",
        "Unsupported default argument expression.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/unsupported_default_expr_call.sifr",
        "function {function}: unsupported default argument expression for parameter {parameter}",
        "sifr_hir::lower::typing_and_functions",
        [arg!("function"), arg!("parameter")],
        ["function", "parameter"]
    ),
    active_entry!(
        "SIFR-TYPE-0012",
        "TYPE",
        "Unsupported expression form.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/unsupported_yield_expression.sifr",
        "unsupported expression form: {form}",
        "sifr_hir::lower::expressions",
        [arg!("form")],
        ["form"]
    ),
    active_entry!(
        "SIFR-TYPE-0901",
        "TYPE",
        "Integer arithmetic may overflow at runtime.",
        Severity::Warning,
        "crates/sifr_driver/src/tests/single_file_frontend.rs::test_type_check_source_surfaces_arithmetic_overflow_as_structured_warning",
        "integer {operation} may overflow at runtime",
        "sifr_hir::lower::arithmetic_warnings",
        [arg!("operation")],
        ["operation"]
    ),
    active_entry!(
        "SIFR-TYPE-0902",
        "TYPE",
        "Reveal the inferred static type of an expression.",
        Severity::Note,
        "crates/sifr_driver/src/tests/single_file_frontend.rs::test_type_check_source_surfaces_reveal_type_as_structured_note",
        "revealed type is {revealed_type}",
        "sifr_hir::lower::builtin_calls",
        [arg!("revealed_type")],
        ["revealed_type"]
    ),
    active_entry!(
        "SIFR-ASYNC-0001",
        "ASYNC",
        "Async function body has no real suspension effect.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/async_no_suspend_rejected.sifr",
        "{message}",
        "sifr_hir::lower::typing_and_functions",
        [arg!("message")],
        ["message"]
    ),
    active_entry!(
        "SIFR-ASYNC-0002",
        "ASYNC",
        "Awaited same-task coroutine has no real suspension effect.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/async_transitive_no_suspend_await_rejected.sifr",
        "{message}",
        "sifr_hir::lower::async_await",
        [arg!("message")],
        ["message"]
    ),
    active_entry!(
        "SIFR-ASYNC-0003",
        "ASYNC",
        "Blocking I/O function called directly from async context.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/blocking_io_direct_call_in_async_rejected.sifr",
        "{message}",
        "sifr_hir::lower::workload_annotations",
        [arg!("message")],
        ["message"]
    ),
    active_entry!(
        "SIFR-ASYNC-0004",
        "ASYNC",
        "CPU-heavy function called directly from async context.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/cpu_heavy_direct_call_in_async_rejected.sifr",
        "{message}",
        "sifr_hir::lower::workload_annotations",
        [arg!("message")],
        ["message"]
    ),
    active_entry!(
        "SIFR-ASYNC-0005",
        "ASYNC",
        "Blocking offload target is not classified as blocking I/O or CPU-heavy work.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/spawn_blocking_unannotated_rejected.sifr",
        "{message}",
        "sifr_hir::lower::workload_annotations",
        [arg!("message")],
        ["message"]
    ),
    active_entry!(
        "SIFR-ASYNC-0006",
        "ASYNC",
        "Synchronous workload annotation applied to async function.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/blocking_io_on_async_def_rejected.sifr",
        "{message}",
        "sifr_hir::lower::typing_and_functions",
        [arg!("message")],
        ["message"]
    ),
    active_entry!(
        "SIFR-DECIMAL-0001",
        "DECIMAL",
        "Invalid Decimal exact literal.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/decimal_invalid_literal_string.sifr",
        "invalid Decimal literal: {literal}",
        "sifr_hir::lower",
        [arg!("literal")],
        ["literal"]
    ),
    active_entry!(
        "SIFR-DECIMAL-0002",
        "DECIMAL",
        "Invalid BigDecimal exact literal.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/bigdecimal_invalid_literal_string.sifr",
        "invalid BigDecimal literal: {literal}",
        "sifr_hir::lower",
        [arg!("literal")],
        ["literal"]
    ),
    active_entry!(
        "SIFR-DECIMAL-0003",
        "DECIMAL",
        "Float mixed with a decimal numeric type.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/decimal_float_mixed_arithmetic.sifr",
        "cannot mix float with {decimal_type}",
        "sifr_type_system",
        [arg!("decimal_type")],
        ["decimal_type"]
    ),
    active_entry!(
        "SIFR-DECIMAL-0004",
        "DECIMAL",
        "Decimal and BigDecimal mixed in one operation.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/decimal_bigdecimal_mixed_arithmetic.sifr",
        "cannot mix Decimal and BigDecimal",
        "sifr_type_system",
        [],
        []
    ),
    active_entry!(
        "SIFR-DECIMAL-0005",
        "DECIMAL",
        "Decimal float construction or conversion is forbidden.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/decimal_constructor_float.sifr",
        "Decimal cannot be constructed from float value {value}",
        "sifr_hir::lower",
        [arg!("value")],
        ["value"]
    ),
    active_entry!(
        "SIFR-DECIMAL-0006",
        "DECIMAL",
        "BigDecimal float construction or conversion is forbidden.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/bigdecimal_constructor_float.sifr",
        "BigDecimal cannot be constructed from float value {value}",
        "sifr_hir::lower",
        [arg!("value")],
        ["value"]
    ),
    active_entry!(
        "SIFR-DECIMAL-0007",
        "DECIMAL",
        "Decimal scale argument is invalid.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/decimal_round_scale_out_of_range.sifr",
        "invalid Decimal scale {scale}",
        "sifr_hir::lower::decimal_methods",
        [arg!("scale"), json_arg!("operation")],
        ["scale", "operation"]
    ),
    active_entry!(
        "SIFR-DECIMAL-0008",
        "DECIMAL",
        "BigDecimal scale or context argument is invalid.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/bigdecimal_quantize_negative_scale_context.sifr",
        "invalid BigDecimal {argument}: {value}",
        "sifr_hir::lower::decimal_methods",
        [arg!("argument"), arg!("value"), json_arg!("operation")],
        ["argument", "value", "operation"]
    ),
    active_entry!(
        "SIFR-INT-0001",
        "INT",
        "Fixed-width integer literal or const expression is out of range.",
        Severity::Error,
        "crates/sifr_hir/src/lower/expressions_tests.rs::test_fixed_width_literal_assignment_out_of_range_has_int_code",
        "integer value {value} does not fit target type {target_type}; valid range is {min_value}..={max_value}",
        "sifr_hir::lower::fixed_width_fitting",
        [
            arg!("value"),
            arg!("target_type"),
            arg!("min_value"),
            arg!("max_value")
        ],
        ["value", "target_type", "min_value", "max_value"]
    ),
    active_entry!(
        "SIFR-INT-0003",
        "INT",
        "Reserved integer width name used before support lands.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/reserved_int128_annotation.sifr",
        "reserved integer width name {name} is not supported yet",
        "sifr_hir::lower::typing_and_functions",
        [arg!("name")],
        ["name"]
    ),
    active_entry!(
        "SIFR-INT-0004",
        "INT",
        "Compile-time integer evaluation budget exceeded.",
        Severity::Error,
        "crates/sifr_hir/src/lower/expressions_tests.rs::test_large_integer_literal_over_budget_has_int_code",
        "integer literal exceeds compile-time evaluation budget: {digits} decimal digits (max {max_digits})",
        "sifr_hir::lower::integer_literal_diagnostics",
        [arg!("digits"), arg!("max_digits")],
        ["digits", "max_digits"]
    ),
    active_entry!(
        "SIFR-INT-0005",
        "INT",
        "Integer division, modulo, or exponentiation requires handling a typed failure.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/exact_int_division_requires_handling.sifr",
        "integer division, modulo, or exponentiation requires handling a typed integer failure unless the compiler can prove this operation is safe",
        "sifr_hir::lower::integer_failure_diagnostics",
        [],
        []
    ),
    active_entry!(
        "SIFR-INT-0006",
        "INT",
        "Exact integer to float conversion requires handling precision loss.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/exact_int_true_division_requires_handling.sifr",
        "exact integer to float conversion requires handling possible overflow or precision loss",
        "sifr_type_system",
        [],
        []
    ),
    active_entry!(
        "SIFR-INT-0007",
        "INT",
        "Bool and integer comparison requires explicit conversion.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/bool_integer_comparison.sifr",
        "cannot compare bool and integer values without explicit conversion",
        "sifr_type_system",
        [],
        []
    ),
    active_entry!(
        "SIFR-INT-0011",
        "INT",
        "Temporary bigint transition alias used.",
        Severity::Warning,
        "crates/sifr_driver/src/tests/single_file_frontend.rs::test_type_check_source_surfaces_bigint_transition_warning",
        "bigint is a temporary transition alias; use int for exact integers or an explicit fixed-width type for representation-sensitive values",
        "sifr_hir::lower::typing_and_functions",
        [],
        []
    ),
    active_entry!(
        "SIFR-CALL-0001",
        "CALL",
        "Wrong positional argument count.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/stdlib_wrong_arg_count.sifr",
        "{callable} takes {quantifier} {expected_count} argument(s), got {actual_count}",
        "sifr_hir::lower",
        [
            arg!("callable"),
            arg!("quantifier"),
            arg!("expected_count"),
            arg!("actual_count")
        ],
        ["callable", "quantifier", "expected_count", "actual_count"]
    ),
    active_entry!(
        "SIFR-CALL-0002",
        "CALL",
        "Unexpected keyword argument.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/sorted_unexpected_keyword.sifr",
        "{callable} got an unexpected keyword argument '{keyword}'",
        "sifr_hir::lower",
        [arg!("callable"), arg!("keyword")],
        ["callable", "keyword"]
    ),
    active_entry!(
        "SIFR-CALL-0003",
        "CALL",
        "Duplicate argument from positional and keyword overlap.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/keyword_after_positional_error.sifr",
        "{callable} got multiple values for argument '{argument}'",
        "sifr_hir::lower",
        [arg!("callable"), arg!("argument")],
        ["callable", "argument"]
    ),
    active_entry!(
        "SIFR-CALL-0004",
        "CALL",
        "Missing required argument.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/missing_required_argument.sifr",
        "{callable} missing required argument '{argument}'",
        "sifr_hir::lower",
        [arg!("callable"), arg!("argument")],
        ["callable", "argument"]
    ),
    active_entry!(
        "SIFR-CALL-0005",
        "CALL",
        "Callable arity failure or expression is not callable.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/map_callable_arity_mismatch.sifr",
        "{callable} callable expects {expected_count} argument(s), got {actual_count} iterable(s)",
        "sifr_hir::lower",
        [
            arg!("callable"),
            arg!("expected_count"),
            arg!("actual_count")
        ],
        ["callable", "expected_count", "actual_count"]
    ),
    active_entry!(
        "SIFR-OWN-0001",
        "OWN",
        "Use after move.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/use_after_move.sifr",
        "use of moved value {binding}",
        "sifr_hir::lower",
        [arg!("binding")],
        ["binding"]
    ),
    active_entry!(
        "SIFR-OWN-0002",
        "OWN",
        "Same-call borrow conflict.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/double_mut_borrow.sifr",
        "borrow conflict for {binding} in the same call",
        "sifr_hir::lower",
        [arg!("binding")],
        ["binding"]
    ),
    active_entry!(
        "SIFR-OWN-0003",
        "OWN",
        "Borrowed parameter escapes by return or store.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/borrow_escape_return.sifr",
        "borrowed parameter {binding} escapes",
        "sifr_hir::lower",
        [arg!("binding"), json_arg!("escape_kind")],
        ["binding", "escape_kind"]
    ),
    active_entry!(
        "SIFR-OWN-0004",
        "OWN",
        "Moved value is reused across loop iterations.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/use_after_move_loop.sifr",
        "moved value {binding} is reused across loop iterations",
        "sifr_hir::lower",
        [arg!("binding")],
        ["binding"]
    ),
    active_entry!(
        "SIFR-OWN-0005",
        "OWN",
        "Immutable parameter is mutated.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/own_parameter_mutation_requires_mut.sifr",
        "cannot mutate through immutable parameter {binding}",
        "sifr_hir::lower",
        [arg!("binding")],
        ["binding"]
    ),
    active_entry!(
        "SIFR-OWN-0006",
        "OWN",
        "Immutable parameter is reassigned.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/own_parameter_reassignment_requires_mut.sifr",
        "cannot reassign immutable parameter {binding}",
        "sifr_hir::lower",
        [arg!("binding")],
        ["binding"]
    ),
    active_entry!(
        "SIFR-OWN-0007",
        "OWN",
        "Immutable bytes value is mutated.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/bytes_subscript_assignment_unsupported.sifr",
        "bytes is immutable; subscript assignment is not supported",
        "sifr_hir::lower::statements",
        [],
        []
    ),
    active_entry!(
        "SIFR-OWN-0008",
        "OWN",
        "Immutable bytes value is mutated by augmented assignment.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/bytes_augmented_subscript_assignment_unsupported.sifr",
        "bytes is immutable; augmented subscript assignment is not supported",
        "sifr_hir::lower::aug_assign_lowering",
        [],
        []
    ),
    active_entry!(
        "SIFR-OWN-0009",
        "OWN",
        "Mutable borrow remains live across an await point.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/borrow_across_await_rejected.sifr",
        "mutable borrow {binding} cannot cross await",
        "sifr_hir::lower",
        [arg!("binding")],
        ["binding"]
    ),
    active_entry!(
        "SIFR-OWN-0010",
        "OWN",
        "Non-sendable value crosses a spawned task boundary.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/spawn_non_send_field_rejected.sifr",
        "scope.spawn() cannot move {value} of type {type_name} across a task boundary",
        "sifr_hir::lower::task_scope_calls",
        [arg!("value"), arg!("type_name"), json_arg!("reason")],
        ["value", "type_name", "reason"]
    ),
    active_entry!(
        "SIFR-OWN-0011",
        "OWN",
        "Non-sendable value is sent through a channel.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/channel_non_send_element_rejected.sifr",
        "channel send cannot transfer {value} of type {type_name}",
        "sifr_hir::lower",
        [arg!("value"), arg!("type_name"), json_arg!("reason")],
        ["value", "type_name", "reason"]
    ),
    active_entry!(
        "SIFR-OWN-0012",
        "OWN",
        "Non-share-safe value is wrapped in sync.Shared.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/shared_mut_without_lock_rejected.sifr",
        "Shared cannot publish {value} of type {type_name}",
        "sifr_hir::lower",
        [arg!("value"), arg!("type_name"), json_arg!("reason")],
        ["value", "type_name", "reason"]
    ),
    active_entry!(
        "SIFR-FLOW-0001",
        "FLOW",
        "Break outside a loop.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/break_outside_loop.sifr",
        "'break' outside of loop",
        "sifr_hir::lower::statements",
        [],
        []
    ),
    active_entry!(
        "SIFR-FLOW-0002",
        "FLOW",
        "Continue outside a loop.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/continue_outside_loop.sifr",
        "'continue' outside of loop",
        "sifr_hir::lower::statements",
        [],
        []
    ),
    active_entry!(
        "SIFR-FLOW-0003",
        "FLOW",
        "Invalid nonlocal or nested-function flow.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/nested_function_recursive_nonlocal_unsupported.sifr",
        "recursive nested function '{function}' cannot mutate captured state with nonlocal yet",
        "sifr_hir::lower",
        [arg!("function")],
        ["function"]
    ),
    active_entry!(
        "SIFR-FLOW-0004",
        "FLOW",
        "Function may finish without returning a required value.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/missing_return_value.sifr",
        "function '{function}' must return a value of type '{return_type}' on all control-flow paths",
        "sifr_hir::lower::typing_and_functions",
        [arg!("function"), arg!("return_type")],
        ["function", "return_type"]
    ),
    active_entry!(
        "SIFR-FLOW-0005",
        "FLOW",
        "Control-flow condition has an unsupported type.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/if_condition_numeric_truthiness.sifr",
        "{keyword} condition must be bool or collection/string truthiness, got '{actual}'",
        "sifr_hir::lower::control_flow_conditions",
        [arg!("keyword"), arg!("actual")],
        ["keyword", "actual"]
    ),
    active_entry!(
        "SIFR-FLOW-0006",
        "FLOW",
        "Statement form is unsupported by HIR lowering.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/yield_without_value.sifr",
        "unsupported statement form: {form}",
        "sifr_hir::lower::statements",
        [arg!("form")],
        ["form"]
    ),
    active_entry!(
        "SIFR-FLOW-0007",
        "FLOW",
        "Assignment target form is unsupported by HIR lowering.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/invalid_assignment_target_attribute_base.sifr",
        "invalid assignment target: {target}",
        "sifr_hir::lower::statements",
        [arg!("target")],
        ["target"]
    ),
    active_entry!(
        "SIFR-FLOW-0008",
        "FLOW",
        "For-loop iteration form or source is invalid.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/for_loop_invalid_iterable.sifr",
        "invalid for-loop iteration: {reason}",
        "sifr_hir::lower::statements",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-FLOW-0901",
        "FLOW",
        "Unreachable statement ignored during lowering.",
        Severity::Warning,
        "crates/sifr_driver/src/tests/single_file_frontend.rs::test_type_check_source_surfaces_unreachable_statement_as_structured_warning",
        "unreachable statement ignored",
        "sifr_hir::lower::statements",
        [],
        []
    ),
    active_entry!(
        "SIFR-MATCH-0001",
        "MATCH",
        "Non-exhaustive match.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/enum_match_non_exhaustive.sifr",
        "non-exhaustive match: enum {enum_name} has uncovered variants: {uncovered}",
        "sifr_hir::lower::statements",
        [arg!("enum_name"), arg!("uncovered")],
        ["enum_name", "uncovered"]
    ),
    active_entry!(
        "SIFR-MATCH-0002",
        "MATCH",
        "Match guard must be bool.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/match_type_mismatch_guard.sifr",
        "match guard must be a bool expression, got {actual}",
        "sifr_hir::lower::statements",
        [arg!("actual")],
        ["actual"]
    ),
    active_entry!(
        "SIFR-MATCH-0003",
        "MATCH",
        "Invalid class pattern field.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/match_invalid_field_name.sifr",
        "class {class_name} has no field {field}",
        "sifr_hir::lower::statements",
        [arg!("field"), arg!("class_name")],
        ["field", "class_name"]
    ),
    active_entry!(
        "SIFR-MATCH-0004",
        "MATCH",
        "Invalid or unsupported match pattern form.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/match_tuple_pattern_requires_tuple_subject.sifr",
        "invalid match pattern: {reason}",
        "sifr_hir::lower::statements",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-PROTO-0001",
        "PROTO",
        "Protocol bound or conformance failure.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/generic_bounds_not_satisfied.sifr",
        "type '{actual}' does not implement protocol '{protocol}' required by type parameter '{type_param}'",
        "sifr_hir::lower",
        [arg!("actual"), arg!("protocol"), arg!("type_param")],
        ["actual", "protocol", "type_param"]
    ),
    active_entry!(
        "SIFR-PROTO-0002",
        "PROTO",
        "Invalid iterator or reversible protocol signature.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/invalid_iter_signature.sifr",
        "class '{type_name}' must return {expected}",
        "sifr_hir::lower::classes",
        [arg!("type_name"), arg!("expected")],
        ["type_name", "expected"]
    ),
    active_entry!(
        "SIFR-PROTO-0003",
        "PROTO",
        "Context-manager protocol is missing.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/with_non_context_manager.sifr",
        "type '{type_name}' does not implement the ContextManager protocol (missing __enter__ and __exit__ methods)",
        "sifr_hir::lower::statements",
        [arg!("type_name")],
        ["type_name"]
    ),
    active_entry!(
        "SIFR-PROTO-0004",
        "PROTO",
        "Hashable or comparable protocol is required.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/unhashable_dict_key.sifr",
        "hash() argument must be hashable, got '{type_name}'",
        "sifr_hir::lower::expressions",
        [arg!("type_name")],
        ["type_name"]
    ),
    active_entry!(
        "SIFR-CLASS-0001",
        "CLASS",
        "Class fields require an initializer or super initializer.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/auto_init_inheritance_missing_super.sifr",
        "class '{class_name}' has fields but no __init__; parent fields will not be initialized. Define an explicit __init__ with super().__init__(...)",
        "sifr_hir::lower::classes",
        [arg!("class_name")],
        ["class_name"]
    ),
    active_entry!(
        "SIFR-CLASS-0002",
        "CLASS",
        "Required field declared after a defaulted field.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/auto_init_required_after_default.sifr",
        "class '{class_name}': required field '{field}' declared after field with default value",
        "sifr_hir::lower::classes",
        [arg!("class_name"), arg!("field")],
        ["class_name", "field"]
    ),
    active_entry!(
        "SIFR-CLASS-0003",
        "CLASS",
        "Duplicate enum or class value, or invalid variant.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/enum_duplicate_value.sifr",
        "enum '{enum_name}' has duplicate value {value}: variants '{existing_variant}' and '{duplicate_variant}'",
        "sifr_hir::lower::classes",
        [
            arg!("enum_name"),
            arg!("value"),
            arg!("existing_variant"),
            arg!("duplicate_variant")
        ],
        ["enum_name", "value", "existing_variant", "duplicate_variant"]
    ),
    active_entry!(
        "SIFR-CLASS-0004",
        "CLASS",
        "Missing class field.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/missing_field.sifr",
        "type '{type_name}' has no field '{field}'",
        "sifr_hir::lower::expressions",
        [arg!("type_name"), arg!("field")],
        ["type_name", "field"]
    ),
    active_entry!(
        "SIFR-CLASS-0005",
        "CLASS",
        "Invalid class base.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/class_unknown_parent.sifr",
        "invalid base class for '{class_name}': {reason}",
        "sifr_hir::lower::classes",
        [arg!("class_name"), arg!("reason")],
        ["class_name", "reason"]
    ),
    active_entry!(
        "SIFR-CLASS-0006",
        "CLASS",
        "Unsupported class declaration.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/class_unsupported_field_default.sifr",
        "unsupported class declaration in '{class_name}': {detail}",
        "sifr_hir::lower::classes",
        [arg!("class_name"), arg!("detail")],
        ["class_name", "detail"]
    ),
    active_entry!(
        "SIFR-RESULT-0001",
        "RESULT",
        "Unused Result value.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/unused_result.sifr",
        "unused Result value",
        "sifr_hir::lower",
        [],
        []
    ),
    active_entry!(
        "SIFR-RESULT-0002",
        "RESULT",
        "Invalid Result error type.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/error_str_not_allowed.sifr",
        "invalid Result error type {error_type}",
        "sifr_hir::lower",
        [arg!("error_type")],
        ["error_type"]
    ),
    active_entry!(
        "SIFR-RESULT-0003",
        "RESULT",
        "Invalid raise expression.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/error_raise_str.sifr",
        "invalid raise expression of type {actual}",
        "sifr_hir::lower::statements",
        [arg!("actual")],
        ["actual"]
    ),
    active_entry!(
        "SIFR-RESULT-0004",
        "RESULT",
        "Except arm references an unknown error type.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/unknown_except_error_type.sifr",
        "unknown except error type '{error_type}'",
        "sifr_hir::lower::statements",
        [arg!("error_type")],
        ["error_type"]
    ),
    active_entry!(
        "SIFR-RESULT-0005",
        "RESULT",
        "Try body error types are not fully covered by except arms.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/try_except_uncovered_error_types.sifr",
        "except arms do not cover all error types from try body: {uncovered}",
        "sifr_hir::lower::statements",
        [arg!("uncovered")],
        ["uncovered"]
    ),
    active_entry!(
        "SIFR-RESULT-0006",
        "RESULT",
        "Except arm type expression has an unsupported form.",
        Severity::Error,
        "crates/sifr_hir/src/lower/statement_diagnostics_tests.rs",
        "invalid except error type: {reason}",
        "sifr_hir::lower::statements",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-STDLIB-0001",
        "STDLIB",
        "Unsupported standard-library constructor, method, or surface.",
        Severity::Error,
        "crates/sifr/tests/e2e/fail/defaultdict_keyword_constructor_unsupported.sifr",
        "defaultdict() does not support keyword arguments",
        "sifr_hir::lower::builtin_calls",
        [],
        []
    ),
    active_entry!(
        "SIFR-STDLIB-0003",
        "STDLIB",
        "Embedded standard-library bootstrap failure.",
        Severity::Error,
        "crates/sifr_driver/src/tests/stdlib_exports.rs",
        "embedded standard library bootstrap failed during {operation}",
        "sifr_driver::stdlib::bootstrap",
        [arg!("operation")],
        ["operation"]
    ),
    active_entry!(
        "SIFR-STDLIB-0004",
        "STDLIB",
        "Standard-library cache build or reuse failure.",
        Severity::Error,
        "crates/sifr_driver/src/tests/project_build_check.rs",
        "standard-library cache failed during {operation}",
        "sifr_driver::stdlib::cache",
        [arg!("operation")],
        ["operation"]
    ),
    active_entry!(
        "SIFR-WORKSPACE-0001",
        "WORKSPACE",
        "Malformed workspace manifest.",
        Severity::Error,
        "crates/sifr/tests/verification/project/workspace_malformed_manifest",
        "could not parse workspace manifest at {path}: {reason}",
        "sifr_driver::workspace",
        [arg!("path"), arg!("reason")],
        ["path", "reason"]
    ),
    active_entry!(
        "SIFR-WORKSPACE-0002",
        "WORKSPACE",
        "Workspace source root escapes the workspace root.",
        Severity::Error,
        "crates/sifr_driver/src/tests/discovery_and_workspace.rs",
        "source root {path} escapes the workspace root",
        "sifr_driver::workspace",
        [arg!("path")],
        ["path"]
    ),
    active_entry!(
        "SIFR-WORKSPACE-0003",
        "WORKSPACE",
        "Workspace source root is not a directory.",
        Severity::Error,
        "crates/sifr_driver/src/tests/discovery_and_workspace.rs",
        "source root {path} is not a directory",
        "sifr_driver::workspace",
        [arg!("path")],
        ["path"]
    ),
    active_entry!(
        "SIFR-WORKSPACE-0004",
        "WORKSPACE",
        "Workspace source root entry has an invalid shape or path.",
        Severity::Error,
        "crates/sifr_driver/src/tests/discovery_and_workspace.rs",
        "invalid source root entry {entry}",
        "sifr_driver::workspace",
        [arg!("entry")],
        ["entry"]
    ),
    active_entry!(
        "SIFR-WORKSPACE-0101",
        "WORKSPACE",
        "Workspace import target could not be resolved.",
        Severity::Error,
        "crates/sifr/tests/verification/project/workspace_unresolved_import",
        "could not resolve import {module}",
        "sifr_driver::project::discovery",
        [arg!("module"), json_arg!("searched_paths")],
        ["module", "searched_paths"]
    ),
    active_entry!(
        "SIFR-WORKSPACE-0102",
        "WORKSPACE",
        "Workspace import target is ambiguous.",
        Severity::Error,
        "crates/sifr/tests/verification/project/workspace_ambiguous_import",
        "module {module} is ambiguous in workspace {workspace}",
        "sifr_driver::project::discovery",
        [
            arg!("module"),
            arg!("workspace"),
            json_arg!("candidate_paths")
        ],
        ["module", "workspace", "candidate_paths"]
    ),
    active_entry!(
        "SIFR-WORKSPACE-0103",
        "WORKSPACE",
        "Workspace namespace package collision.",
        Severity::Error,
        "crates/sifr_driver/src/tests/discovery_and_workspace.rs",
        "module {module} collides with namespace path {path}",
        "sifr_driver::project::discovery",
        [arg!("module"), arg!("path")],
        ["module", "path"]
    ),
    active_entry!(
        "SIFR-WORKSPACE-0104",
        "WORKSPACE",
        "Workspace import graph contains a cycle.",
        Severity::Error,
        "crates/sifr_driver/src/tests/project_graph.rs",
        "workspace import cycle detected: {cycle}",
        "sifr_driver::project::compile_order",
        [arg!("cycle")],
        ["cycle"]
    ),
    active_entry!(
        "SIFR-BUILD-0002",
        "BUILD",
        "Build file materialization failed.",
        Severity::Error,
        "crates/sifr_driver/src/tests/project_build_check.rs",
        "failed to materialize build file {path}",
        "sifr_driver::build::materialize",
        [arg!("path")],
        ["path"]
    ),
    active_entry!(
        "SIFR-BUILD-0003",
        "BUILD",
        "Temporary build workspace creation failed.",
        Severity::Error,
        "crates/sifr_driver/src/tests/project_build_check.rs",
        "failed to create temporary build workspace {path}",
        "sifr_driver::build::workspace",
        [arg!("path")],
        ["path"]
    ),
    active_entry!(
        "SIFR-BUILD-0004",
        "BUILD",
        "Cargo manifest generation failed.",
        Severity::Error,
        "crates/sifr_driver/src/tests/project_build_check.rs",
        "failed to generate Cargo manifest at {path}",
        "sifr_driver::build::workspace",
        [arg!("path")],
        ["path"]
    ),
    active_entry!(
        "SIFR-BUILD-0005",
        "BUILD",
        "Rustc or Cargo execution failed.",
        Severity::Error,
        "crates/sifr_driver/src/tests/project_build_check.rs",
        "{tool} failed with exit status {status}",
        "sifr_driver::build::workspace",
        [arg!("tool"), arg!("status")],
        ["tool", "status"]
    ),
    active_entry!(
        "SIFR-BUILD-0006",
        "BUILD",
        "Expected build artifact was not produced.",
        Severity::Error,
        "crates/sifr_driver/src/tests/project_build_check.rs",
        "expected build artifact {path} was not produced",
        "sifr_driver::build::workspace",
        [arg!("path")],
        ["path"]
    ),
    active_entry!(
        "SIFR-INTERNAL-0001",
        "INTERNAL",
        "Unclassified compiler panic after a panic boundary.",
        Severity::Error,
        "crates/sifr_driver/src/tests/panic_boundary.rs::planned_internal_0001",
        "internal compiler error",
        "sifr_driver::diagnostics",
        [],
        []
    ),
    active_entry!(
        "SIFR-INTERNAL-0002",
        "INTERNAL",
        "Structured recovery-cap omission summary.",
        Severity::Note,
        "crates/sifr_driver/src/tests/diagnostics.rs::test_apply_diagnostic_recovery_limits_summarizes_similar_diagnostics",
        "{omitted_count} additional {omitted_kind} omitted by recovery cap ({cap_kind})",
        "sifr_driver::diagnostics",
        [arg!("omitted_count"), arg!("omitted_kind"), arg!("cap_kind")],
        ["cap_kind"]
    ),
    active_entry!(
        "SIFR-FMT-0001",
        "FMT",
        "Source formatting drift detected by sifr fmt --check.",
        Severity::Error,
        "crates/sifr_format/src/lib.rs::check_reports_formatting_drift",
        "source is not formatted with sifr fmt",
        "sifr_format",
        [arg!("path")],
        ["path"]
    ),
    active_entry!(
        "SIFR-LINT-0001",
        "LINT",
        "Suppression references an unknown policy rule id.",
        Severity::Warning,
        "crates/sifr_lint/src/lib.rs::unknown_and_unused_suppressions_are_reported",
        "unknown Sifr policy rule id '{rule}'",
        "sifr_lint::suppressions",
        [arg!("rule")],
        ["rule"]
    ),
    active_entry!(
        "SIFR-LINT-0002",
        "LINT",
        "Suppression did not suppress any diagnostic.",
        Severity::Warning,
        "crates/sifr_lint/src/lib.rs::unknown_and_unused_suppressions_are_reported",
        "unused Sifr suppression for policy rule '{rule}'",
        "sifr_lint::suppressions",
        [arg!("rule")],
        ["rule"]
    ),
    active_entry!(
        "SIFR-LINT-0003",
        "LINT",
        "Suppression must list explicit Sifr policy rule ids.",
        Severity::Warning,
        "crates/sifr_lint/src/lib.rs::blanket_suppression_is_reported",
        "sifr suppression must list explicit policy rule ids",
        "sifr_lint::suppressions",
        [arg!("rule")],
        ["rule"]
    ),
    active_entry!(
        "SIFR-LINT-0004",
        "LINT",
        "Line ends with trailing horizontal whitespace.",
        Severity::Warning,
        "crates/sifr_lint/src/lib.rs::suppression_only_suppresses_matching_policy_rule",
        "line has trailing whitespace",
        "sifr_lint::rules::trailing_whitespace",
        [arg!("rule")],
        ["rule"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0001",
        "PACKAGE",
        "Missing or invalid Cargo Sifr discovery metadata.",
        Severity::Error,
        "crates/sifr_package/src/manifest/metadata.rs::tests",
        "invalid [package.metadata.sifr]: {reason}",
        "sifr_package::manifest::metadata",
        [arg!("reason"), json_arg!("cargo_package_id")],
        ["cargo_package_id", "reason"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0002",
        "PACKAGE",
        "Missing or invalid sifr.toml package manifest.",
        Severity::Error,
        "crates/sifr_package/src/manifest/sifr.rs::tests",
        "invalid sifr.toml: {reason}",
        "sifr_package::manifest::sifr",
        [arg!("reason"), json_arg!("cargo_package_id"), json_arg!("manifest_path")],
        ["cargo_package_id", "manifest_path", "reason"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0003",
        "PACKAGE",
        "Unsupported Sifr compiler metadata appears in Cargo metadata.",
        Severity::Error,
        "crates/sifr_package/src/manifest/metadata.rs::tests",
        "unsupported Sifr compiler metadata in Cargo metadata: {key}",
        "sifr_package::manifest::metadata",
        [arg!("key"), json_arg!("cargo_package_id")],
        ["cargo_package_id", "key"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0103",
        "PACKAGE",
        "Cargo metadata parsing or normalization failed.",
        Severity::Error,
        "crates/sifr_package/src/cargo/metadata.rs::tests",
        "could not parse cargo metadata: {reason}",
        "sifr_package::cargo::metadata",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0101",
        "PACKAGE",
        "Cargo command invocation failed; Sifr reports the redacted Cargo excerpt and safe Sifr-owned recovery context.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_4_tests.rs::cargo_failure_mapping_redacts_private_credentials",
        "cargo {action} failed",
        "sifr_package::cargo::errors",
        [arg!("action"), arg!("reason")],
        ["action", "reason"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0102",
        "PACKAGE",
        "A selected Cargo package is Rust-only.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_5_tests.rs::explicit_rust_only_selection_reports_0102",
        "selected Rust-only package '{package_name}'",
        "sifr_package::graph::workspace",
        [arg!("package_name"), json_arg!("cargo_package_id")],
        ["cargo_package_id", "package_name"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0104",
        "PACKAGE",
        "Package source is unavailable in offline or frozen mode.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_4_tests.rs::offline_mode_reports_missing_sifr_source_package",
        "package source unavailable in {lock_mode} mode",
        "sifr_package::cargo::lock_modes",
        [
            arg!("lock_mode"),
            json_arg!("cargo_package_id"),
            json_arg!("package_path")
        ],
        ["cargo_package_id", "package_path", "lock_mode"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0106",
        "PACKAGE",
        "Rust-only package depends directly on a Sifr source package.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_5_tests.rs::rust_only_member_depending_on_sifr_reports_0106",
        "Rust-only package depends on Sifr package",
        "sifr_package::graph::workspace",
        [
            json_arg!("from_cargo_package_id"),
            json_arg!("to_cargo_package_id")
        ],
        ["from_cargo_package_id", "to_cargo_package_id"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0201",
        "PACKAGE",
        "Direct package import root resolves to multiple package instances.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_2_tests.rs::duplicate_direct_import_root_in_one_scope_reports_0201",
        "ambiguous package import root '{import_root}'",
        "sifr_package::graph::scopes",
        [
            arg!("import_root"),
            json_arg!("cargo_package_id"),
            json_arg!("candidates")
        ],
        ["cargo_package_id", "import_root", "candidates"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0202",
        "PACKAGE",
        "Package imports a module outside its direct dependency scope.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_3_tests.rs::transitive_dependency_import_reports_0202",
        "undeclared direct package import '{import_path}'",
        "sifr_package::imports::source_map",
        [
            arg!("import_path"),
            json_arg!("cargo_package_id"),
            json_arg!("package_id")
        ],
        ["cargo_package_id", "package_id", "import_path"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0203",
        "PACKAGE",
        "Package imports a private module from another package.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_3_tests.rs::private_dependency_module_reports_0203",
        "private package module access '{import_path}'",
        "sifr_package::imports::source_map",
        [
            arg!("import_path"),
            json_arg!("cargo_package_id"),
            json_arg!("package_id"),
            json_arg!("target_package_id")
        ],
        ["cargo_package_id", "package_id", "target_package_id", "import_path"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0204",
        "PACKAGE",
        "Type identity crosses resolved package instances.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_2_tests.rs::type_identity_mismatch_reports_0204_for_distinct_package_instances",
        "package type identity mismatch: expected {expected}, got {actual}",
        "sifr_package::graph::type_identity",
        [
            arg!("expected"),
            arg!("actual"),
            json_arg!("cargo_package_id")
        ],
        ["cargo_package_id", "expected", "actual"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0301",
        "PACKAGE",
        "Backend Rust crate is not allowed by the Sifr trust policy.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_4_tests.rs::backend_trust_reports_untrusted_direct_backend_crate",
        "untrusted backend crate '{backend_name}'",
        "sifr_package::cargo::trust",
        [
            arg!("backend_name"),
            json_arg!("cargo_package_id"),
            json_arg!("package_id")
        ],
        ["cargo_package_id", "package_id", "backend_name"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0305",
        "PACKAGE",
        "Trust policy names a backend crate that is not a direct dependency.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_4_tests.rs::backend_trust_rejects_stale_non_direct_trust_entry",
        "trusted backend crate '{backend_name}' is not direct",
        "sifr_package::cargo::trust",
        [
            arg!("backend_name"),
            json_arg!("cargo_package_id"),
            json_arg!("package_id")
        ],
        ["cargo_package_id", "package_id", "backend_name"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0401",
        "PACKAGE",
        "Cargo package archive is missing required Sifr source.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_6_tests.rs::archive_missing_sifr_source_reports_0401",
        "package '{package_id}' archive contains no .sifr source files",
        "sifr_package::cargo::package",
        [json_arg!("cargo_package_id"), arg!("package_id")],
        ["cargo_package_id", "package_id"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0402",
        "PACKAGE",
        "Package publish or archive validation failed.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_6_tests.rs::publish_validation_failed_reports_0402",
        "package publish validation failed: {reason}",
        "sifr_package::cargo::package",
        [arg!("reason"), json_arg!("cargo_package_id")],
        ["cargo_package_id", "reason"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0403",
        "PACKAGE",
        "Cargo include/exclude rules omit required Sifr files.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_6_tests.rs::archive_missing_required_entry_reports_0403",
        "Cargo package include/exclude rules omit required Sifr file '{path}'",
        "sifr_package::cargo::package",
        [arg!("path"), json_arg!("cargo_package_id")],
        ["cargo_package_id", "path"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0404",
        "PACKAGE",
        "Cargo package archive contains an unsafe path.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_6_tests.rs::archive_traversal_reports_0404",
        "Cargo package archive entry escapes the package root: {path}",
        "sifr_package::cargo::package",
        [arg!("path"), json_arg!("cargo_package_id")],
        ["cargo_package_id", "path"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0501",
        "PACKAGE",
        "Pure Sifr Rust marker contains implementation.",
        Severity::Error,
        "crates/sifr_package/src/source/layout.rs::tests",
        "pure Sifr package marker contains Rust implementation: {reason}",
        "sifr_package::source::layout",
        [arg!("reason"), json_arg!("cargo_package_id"), json_arg!("marker_path")],
        ["cargo_package_id", "marker_path", "reason"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0601",
        "PACKAGE",
        "Package selector is ambiguous or invalid.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_5_tests.rs::ambiguous_filter_reports_0601",
        "package selector '{selector}' is ambiguous or invalid",
        "sifr_package::graph::filters",
        [arg!("selector"), json_arg!("candidates")],
        ["selector", "candidates"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0602",
        "PACKAGE",
        "Workspace selection contains duplicate import roots.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_5_tests.rs::workspace_duplicate_import_roots_report_0602",
        "duplicate workspace import root '{import_root}'",
        "sifr_package::graph::workspace",
        [arg!("import_root"), json_arg!("packages")],
        ["import_root", "packages"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0603",
        "PACKAGE",
        "Changed file could not be mapped to a package.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_5_tests.rs::changed_file_mapping_reports_0603",
        "changed path '{path}' does not map to one Sifr package",
        "sifr_package::graph::changed",
        [arg!("path")],
        ["path"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0604",
        "PACKAGE",
        "Outdated query cannot inspect this Cargo source.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_5_tests.rs::outdated_unknown_source_reports_0604",
        "outdated query unsupported for source '{source}'",
        "sifr_package::ops::read",
        [arg!("source"), json_arg!("cargo_package_id")],
        ["cargo_package_id", "source"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0605",
        "PACKAGE",
        "Runnable package target or script selection is missing or ambiguous.",
        Severity::Error,
        "crates/sifr_package/src/milestone_adhoc_pkg_3_tests.rs::package_session_reports_script_target_ambiguity",
        "ambiguous package run target: {selector}",
        "sifr_package::ops::session",
        [arg!("selector"), json_arg!("candidates")],
        ["selector", "candidates"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0606",
        "PACKAGE",
        "Discovered app target name is invalid.",
        Severity::Error,
        "crates/sifr_package/src/milestone_adhoc_pkg_3_tests.rs::package_session_rejects_invalid_nested_target_name",
        "invalid package app target name: {target}",
        "sifr_package::ops::session",
        [arg!("target")],
        ["target"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0607",
        "PACKAGE",
        "Selected workspace members use the same Sifr package name.",
        Severity::Error,
        "crates/sifr_package/src/milestone_37_5_tests.rs::workspace_duplicate_sifr_names_report_0607",
        "duplicate Sifr package name in workspace: {package}",
        "sifr_package::graph::workspace",
        [arg!("package"), json_arg!("members")],
        ["package", "members"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0701",
        "PACKAGE",
        "Production sifr.toml uses manifest-level exports.",
        Severity::Error,
        "crates/sifr_package/src/milestone_adhoc_pkg_1_tests.rs::production_manifest_exports_report_0701",
        "production sifr.toml uses [exports].modules",
        "sifr_package::manifest::sifr",
        [json_arg!("cargo_package_id"), json_arg!("manifest_path")],
        ["cargo_package_id", "manifest_path"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0703",
        "PACKAGE",
        "Sifr-managed Cargo projection manifest pointer drift.",
        Severity::Error,
        "crates/sifr_package/src/milestone_adhoc_pkg_2_tests.rs::repair_check_reports_missing_manifest_pointer_0703",
        "Cargo projection manifest pointer drift",
        "sifr_package::projection",
        [
            json_arg!("cargo_package_id"),
            json_arg!("path"),
            arg!("reason")
        ],
        ["cargo_package_id", "path", "reason"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0704",
        "PACKAGE",
        "Sifr-managed Cargo projection include rules omit required package files.",
        Severity::Error,
        "crates/sifr_package/src/milestone_adhoc_pkg_2_tests.rs::repair_check_reports_missing_required_include_0704",
        "Cargo projection include rules omit required entry '{required}'",
        "sifr_package::projection",
        [
            json_arg!("cargo_package_id"),
            json_arg!("path"),
            arg!("required")
        ],
        ["cargo_package_id", "path", "required"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0709",
        "PACKAGE",
        "Pure package marker is missing from Sifr-managed projection.",
        Severity::Error,
        "crates/sifr_package/src/milestone_adhoc_pkg_2_tests.rs::repair_regenerates_missing_pure_marker",
        "pure Sifr package marker is missing",
        "sifr_package::projection",
        [json_arg!("cargo_package_id"), json_arg!("marker_path")],
        ["cargo_package_id", "marker_path"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0710",
        "PACKAGE",
        "Explicit Sifr file target is outside the package source root.",
        Severity::Error,
        "crates/sifr_package/src/milestone_adhoc_pkg_3_tests.rs::package_session_rejects_explicit_file_outside_source_root",
        "explicit file is outside package source root",
        "sifr_package::ops::session",
        [arg!("file"), arg!("source_root")],
        ["file", "source_root"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0711",
        "PACKAGE",
        "Production sifr.toml uses manifest binary target tables.",
        Severity::Error,
        "crates/sifr_package/src/milestone_adhoc_pkg_1_tests.rs::production_manifest_bin_tables_report_0711",
        "production sifr.toml uses [[bin]]",
        "sifr_package::manifest::sifr",
        [json_arg!("cargo_package_id"), json_arg!("manifest_path")],
        ["cargo_package_id", "manifest_path"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0713",
        "PACKAGE",
        "Public API symbol is exported more than once.",
        Severity::Error,
        "crates/sifr_package/src/milestone_adhoc_pkg_1_tests.rs::duplicate_init_public_symbol_reports_0713",
        "duplicate public API symbol '{symbol}'",
        "sifr_package::imports::namespace_api",
        [
            arg!("symbol"),
            json_arg!("cargo_package_id"),
            json_arg!("manifest_path")
        ],
        ["cargo_package_id", "manifest_path", "symbol"]
    ),
    active_entry!(
        "SIFR-PACKAGE-0714",
        "PACKAGE",
        "Package script expansion attempted to invoke another script.",
        Severity::Error,
        "crates/sifr_package/src/milestone_adhoc_pkg_3_tests.rs::package_session_rejects_nested_script_expansion",
        "package script recursion is not allowed: {script}",
        "sifr_package::ops::session",
        [arg!("script")],
        ["script"]
    ),
];

pub const ACTIVE_DIAGNOSTIC_CODES: &[DiagnosticCode] = &[
    DiagnosticCode::PARSE_EXPECTED_TOKEN_OR_RECOVERY,
    DiagnosticCode::PARSE_LEXICAL_OR_STRING,
    DiagnosticCode::PARSE_LAYOUT,
    DiagnosticCode::PARSE_INVALID_TARGET,
    DiagnosticCode::PARSE_INVALID_CALL_ARGUMENTS,
    DiagnosticCode::PARSE_MALFORMED_DECLARATION_LIST,
    DiagnosticCode::PARSE_INVALID_PATTERN,
    DiagnosticCode::PARSE_UNSUPPORTED_SYNTAX,
    DiagnosticCode::NAME_UNDEFINED_VARIABLE,
    DiagnosticCode::NAME_UNDEFINED_CALLABLE,
    DiagnosticCode::NAME_UNKNOWN_TYPE,
    DiagnosticCode::NAME_MISSING_MODULE_MEMBER,
    DiagnosticCode::NAME_DUPLICATE_DEFINITION,
    DiagnosticCode::NAME_UNINITIALIZED_VARIABLE,
    DiagnosticCode::IMPORT_FORBIDDEN_INTRINSIC,
    DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE,
    DiagnosticCode::IMPORT_UNSUPPORTED_FORM,
    DiagnosticCode::IMPORT_PRIVATE_MEMBER,
    DiagnosticCode::TYPE_MISMATCH,
    DiagnosticCode::TYPE_IF_BRANCH_MISMATCH,
    DiagnosticCode::TYPE_MISSING_ANNOTATION,
    DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
    DiagnosticCode::TYPE_INT_BIGINT_MIXED,
    DiagnosticCode::TYPE_INVALID_ANNOTATION,
    DiagnosticCode::TYPE_CONTAINER_ELEMENT_CONFLICT,
    DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH,
    DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED,
    DiagnosticCode::TYPE_UNSUPPORTED_DEFAULT_ARGUMENT,
    DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
    DiagnosticCode::TYPE_ARITHMETIC_OVERFLOW_RISK,
    DiagnosticCode::TYPE_REVEAL_TYPE,
    DiagnosticCode::ASYNC_NO_SUSPEND,
    DiagnosticCode::ASYNC_AWAIT_NO_SUSPEND,
    DiagnosticCode::ASYNC_DIRECT_BLOCKING_IO_CALL,
    DiagnosticCode::ASYNC_DIRECT_CPU_HEAVY_CALL,
    DiagnosticCode::ASYNC_UNCLASSIFIED_BLOCKING_OFFLOAD_TARGET,
    DiagnosticCode::ASYNC_WORKLOAD_ANNOTATION_ON_ASYNC_DEF,
    DiagnosticCode::DECIMAL_INVALID_LITERAL,
    DiagnosticCode::DECIMAL_BIGDECIMAL_INVALID_LITERAL,
    DiagnosticCode::DECIMAL_FLOAT_MIXED,
    DiagnosticCode::DECIMAL_MIXED_WITH_BIGDECIMAL,
    DiagnosticCode::DECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN,
    DiagnosticCode::DECIMAL_BIGDECIMAL_FLOAT_CONSTRUCTION_FORBIDDEN,
    DiagnosticCode::DECIMAL_SCALE_INVALID,
    DiagnosticCode::DECIMAL_BIGDECIMAL_SCALE_OR_CONTEXT_INVALID,
    DiagnosticCode::INT_FIXED_WIDTH_OUT_OF_RANGE,
    DiagnosticCode::INT_RESERVED_WIDTH_NAME,
    DiagnosticCode::INT_EVAL_BUDGET_EXCEEDED,
    DiagnosticCode::INT_EXACT_DIVISION_REQUIRES_HANDLING,
    DiagnosticCode::INT_EXACT_TO_FLOAT_REQUIRES_HANDLING,
    DiagnosticCode::INT_BOOL_INTEGER_COMPARISON,
    DiagnosticCode::INT_BIGINT_TRANSITION_ALIAS,
    DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT,
    DiagnosticCode::CALL_UNEXPECTED_KEYWORD,
    DiagnosticCode::CALL_DUPLICATE_ARGUMENT,
    DiagnosticCode::CALL_MISSING_REQUIRED_ARGUMENT,
    DiagnosticCode::CALL_NOT_CALLABLE_OR_ARITY,
    DiagnosticCode::OWN_USE_AFTER_MOVE,
    DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW,
    DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES,
    DiagnosticCode::OWN_MOVED_ACROSS_LOOP,
    DiagnosticCode::OWN_IMMUTABLE_PARAMETER_MUTATION,
    DiagnosticCode::OWN_IMMUTABLE_PARAMETER_REASSIGNMENT,
    DiagnosticCode::OWN_IMMUTABLE_BYTES_ASSIGNMENT,
    DiagnosticCode::OWN_IMMUTABLE_BYTES_AUGMENTED_ASSIGNMENT,
    DiagnosticCode::OWN_BORROW_ACROSS_AWAIT,
    DiagnosticCode::OWN_NON_SEND_TASK_CAPTURE,
    DiagnosticCode::OWN_NON_SEND_CHANNEL_ELEMENT,
    DiagnosticCode::OWN_NON_SHARE_SAFE_SHARED_VALUE,
    DiagnosticCode::FLOW_BREAK_OUTSIDE_LOOP,
    DiagnosticCode::FLOW_CONTINUE_OUTSIDE_LOOP,
    DiagnosticCode::FLOW_INVALID_NONLOCAL,
    DiagnosticCode::FLOW_MISSING_RETURN_VALUE,
    DiagnosticCode::FLOW_INVALID_CONDITION_TYPE,
    DiagnosticCode::FLOW_UNSUPPORTED_STATEMENT_FORM,
    DiagnosticCode::FLOW_INVALID_ASSIGNMENT_TARGET,
    DiagnosticCode::FLOW_INVALID_ITERATION,
    DiagnosticCode::FLOW_UNREACHABLE_STATEMENT,
    DiagnosticCode::FMT_FORMATTING_DRIFT,
    DiagnosticCode::LINT_UNKNOWN_SUPPRESSION,
    DiagnosticCode::LINT_UNUSED_SUPPRESSION,
    DiagnosticCode::LINT_BLANKET_SUPPRESSION,
    DiagnosticCode::LINT_TRAILING_WHITESPACE,
    DiagnosticCode::MATCH_NON_EXHAUSTIVE,
    DiagnosticCode::MATCH_GUARD_NOT_BOOL,
    DiagnosticCode::MATCH_INVALID_CLASS_PATTERN_FIELD,
    DiagnosticCode::MATCH_INVALID_PATTERN_FORM,
    DiagnosticCode::PROTO_BOUND_NOT_SATISFIED,
    DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE,
    DiagnosticCode::PROTO_CONTEXT_MANAGER_MISSING,
    DiagnosticCode::PROTO_HASHABLE_OR_COMPARABLE_REQUIRED,
    DiagnosticCode::CLASS_MISSING_INITIALIZER,
    DiagnosticCode::CLASS_REQUIRED_FIELD_AFTER_DEFAULT,
    DiagnosticCode::CLASS_DUPLICATE_OR_INVALID_VALUE,
    DiagnosticCode::CLASS_MISSING_MEMBER,
    DiagnosticCode::CLASS_INVALID_BASE,
    DiagnosticCode::CLASS_UNSUPPORTED_DECLARATION,
    DiagnosticCode::RESULT_UNUSED_VALUE,
    DiagnosticCode::RESULT_INVALID_ERROR_TYPE,
    DiagnosticCode::RESULT_INVALID_RAISE,
    DiagnosticCode::RESULT_UNKNOWN_EXCEPT_TYPE,
    DiagnosticCode::RESULT_UNCOVERED_TRY_ERRORS,
    DiagnosticCode::RESULT_INVALID_EXCEPT_TYPE,
    DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
    DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
    DiagnosticCode::STDLIB_CACHE_FAILURE,
    DiagnosticCode::WORKSPACE_MALFORMED_MANIFEST,
    DiagnosticCode::WORKSPACE_SOURCE_ROOT_ESCAPES,
    DiagnosticCode::WORKSPACE_SOURCE_ROOT_NOT_DIRECTORY,
    DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
    DiagnosticCode::WORKSPACE_UNRESOLVED_IMPORT,
    DiagnosticCode::WORKSPACE_AMBIGUOUS_IMPORT,
    DiagnosticCode::WORKSPACE_NAMESPACE_COLLISION,
    DiagnosticCode::WORKSPACE_IMPORT_CYCLE,
    DiagnosticCode::PACKAGE_MISSING_OR_INVALID_CARGO_METADATA,
    DiagnosticCode::PACKAGE_MISSING_OR_INVALID_SIFR_MANIFEST,
    DiagnosticCode::PACKAGE_UNSUPPORTED_CARGO_SIFR_METADATA,
    DiagnosticCode::PACKAGE_CARGO_COMMAND_FAILED,
    DiagnosticCode::PACKAGE_SELECTED_RUST_ONLY,
    DiagnosticCode::PACKAGE_METADATA_PARSE,
    DiagnosticCode::PACKAGE_SOURCE_UNAVAILABLE_OFFLINE,
    DiagnosticCode::PACKAGE_RUST_ONLY_DEPENDS_ON_SIFR,
    DiagnosticCode::PACKAGE_AMBIGUOUS_IMPORT_ROOT,
    DiagnosticCode::PACKAGE_UNDECLARED_DIRECT_IMPORT,
    DiagnosticCode::PACKAGE_PRIVATE_MODULE_ACCESS,
    DiagnosticCode::PACKAGE_TYPE_IDENTITY_MISMATCH,
    DiagnosticCode::PACKAGE_BACKEND_TRUST_VIOLATION,
    DiagnosticCode::PACKAGE_TRUST_NON_DIRECT_DEPENDENCY,
    DiagnosticCode::PACKAGE_ARCHIVE_MISSING_SIFR_SOURCE,
    DiagnosticCode::PACKAGE_PUBLISH_VALIDATION_FAILED,
    DiagnosticCode::PACKAGE_INCLUDE_EXCLUDE_OMITS_SOURCE,
    DiagnosticCode::PACKAGE_ARCHIVE_TRAVERSAL,
    DiagnosticCode::PACKAGE_NON_TRIVIAL_PURE_MARKER,
    DiagnosticCode::PACKAGE_SELECTOR_AMBIGUOUS,
    DiagnosticCode::PACKAGE_DUPLICATE_WORKSPACE_IMPORT_ROOT,
    DiagnosticCode::PACKAGE_CHANGED_FILE_MAPPING_FAILED,
    DiagnosticCode::PACKAGE_OUTDATED_QUERY_UNSUPPORTED,
    DiagnosticCode::PACKAGE_RUN_TARGET_AMBIGUOUS,
    DiagnosticCode::PACKAGE_INVALID_APP_TARGET_NAME,
    DiagnosticCode::PACKAGE_DUPLICATE_WORKSPACE_SIFR_NAME,
    DiagnosticCode::PACKAGE_MANIFEST_EXPORTS_NOT_PRODUCTION,
    DiagnosticCode::PACKAGE_PROJECTION_MANIFEST_POINTER_DRIFT,
    DiagnosticCode::PACKAGE_PROJECTION_INCLUDE_DRIFT,
    DiagnosticCode::PACKAGE_PROJECTION_PURE_MARKER_MISSING,
    DiagnosticCode::PACKAGE_EXPLICIT_FILE_OUTSIDE_SOURCE_ROOT,
    DiagnosticCode::PACKAGE_MANIFEST_BIN_TABLES_NOT_PRODUCTION,
    DiagnosticCode::PACKAGE_DUPLICATE_PUBLIC_API_SYMBOL,
    DiagnosticCode::PACKAGE_SCRIPT_RECURSION,
    DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
    DiagnosticCode::BUILD_TEMP_WORKSPACE_FAILURE,
    DiagnosticCode::BUILD_CARGO_MANIFEST_FAILURE,
    DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE,
    DiagnosticCode::BUILD_ARTIFACT_MISSING,
    DiagnosticCode::INTERNAL_COMPILER_PANIC,
    DiagnosticCode::INTERNAL_RECOVERY_OMISSION_SUMMARY,
];

#[must_use]
pub fn registry_entry(id: &str) -> Option<&'static DiagnosticRegistryEntry> {
    DIAGNOSTIC_REGISTRY.iter().find(|entry| entry.id == id)
}

pub fn active_registry_entries() -> impl Iterator<Item = &'static DiagnosticRegistryEntry> {
    DIAGNOSTIC_REGISTRY
        .iter()
        .filter(|entry| entry.state == DiagnosticState::Active)
}

const fn reserved_family_base(id: &'static str, family: &'static str) -> DiagnosticRegistryEntry {
    reserved_code(
        id,
        family,
        "Reserved family base; not emitted as a diagnostic.",
    )
}

const fn reserved_code(
    id: &'static str,
    family: &'static str,
    summary: &'static str,
) -> DiagnosticRegistryEntry {
    DiagnosticRegistryEntry {
        id,
        family,
        summary,
        state: DiagnosticState::Reserved,
        docs_path: "docs/errors/diagnostic-codes.md",
        representative_fixture_path: None,
        message_template: None,
        owner_module: None,
        declared_args: &[],
        dedupe_args: &[],
        declared_severity: None,
        tooling: DiagnosticTooling::DEFAULT,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::PathBuf;

    use super::{
        active_registry_entries, DiagnosticCode, DiagnosticState, ACTIVE_DIAGNOSTIC_CODES,
        DIAGNOSTIC_FAMILIES, DIAGNOSTIC_REGISTRY,
    };

    #[test]
    fn docs_url_is_derived_from_code() {
        assert_eq!(
            DiagnosticCode::TEST_SOURCE_ERROR.docs_url(),
            "https://sifr.sh/docs/errors/SIFR-NAME-9999"
        );
    }

    #[test]
    fn registry_skeleton_is_internally_consistent() {
        let families_by_name = families_by_name();
        let mut ids = BTreeSet::new();

        for entry in DIAGNOSTIC_REGISTRY {
            assert!(ids.insert(entry.id), "duplicate diagnostic id {}", entry.id);
            assert_canonical_code(entry.id);
            assert_eq!(entry.family, parse_family(entry.id));
            assert!(
                families_by_name.contains_key(entry.family),
                "unknown diagnostic family {} for {}",
                entry.family,
                entry.id
            );
            assert_dedupe_args_are_declared(entry);
            assert_template_placeholders_are_declared(entry);
            assert_registry_strings_are_markdown_safe(entry);

            match entry.state {
                DiagnosticState::Active => {
                    assert!(
                        entry.declared_severity.is_some(),
                        "active diagnostic {} must declare severity",
                        entry.id
                    );
                    assert!(
                        entry.owner_module.is_some(),
                        "active diagnostic {} must declare owner module",
                        entry.id
                    );
                    assert!(
                        entry.message_template.is_some(),
                        "active diagnostic {} must declare message template",
                        entry.id
                    );
                    assert!(
                        entry.representative_fixture_path.is_some(),
                        "active diagnostic {} must declare representative fixture path",
                        entry.id
                    );
                    assert!(
                        entry.docs_path == format!("docs/errors/{}.md", entry.id),
                        "active diagnostic {} must use its canonical docs page",
                        entry.id
                    );
                }
                DiagnosticState::Reserved => {
                    assert!(
                        entry.representative_fixture_path.is_none(),
                        "reserved diagnostic {} must not claim a fixture",
                        entry.id
                    );
                }
            }
        }

        for family in DIAGNOSTIC_FAMILIES {
            assert_family_name(family.name);
            assert_eq!(family.reserved_base, format!("SIFR-{}-0000", family.name));
            let base = registry_entry_for(family.reserved_base);
            assert_eq!(base.state, DiagnosticState::Reserved);
            assert_eq!(
                base.declared_severity, None,
                "reserved family base {} must not declare severity",
                base.id
            );
        }

        let active_ids: BTreeSet<_> = active_registry_entries().map(|entry| entry.id).collect();
        let constant_ids: BTreeSet<_> = ACTIVE_DIAGNOSTIC_CODES
            .iter()
            .map(|code| code.code())
            .collect();
        assert_eq!(
            active_ids, constant_ids,
            "active registry entries and DiagnosticCode constants must stay in sync"
        );

        for code in ACTIVE_DIAGNOSTIC_CODES {
            let entry = registry_entry_for(code.code());
            assert_eq!(
                entry.declared_severity,
                Some(code.declared_severity()),
                "DiagnosticCode severity must match registry severity for {}",
                code.code()
            );
        }
    }

    #[test]
    fn active_diagnostic_docs_pages_exist_with_exact_casing() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("crate must live under workspace crates directory")
            .to_path_buf();
        let errors_dir = repo_root.join("docs/errors");
        let directory_entries = fs::read_dir(&errors_dir)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", errors_dir.display()))
            .map(|entry| {
                entry
                    .expect("failed to read docs/errors directory entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect::<BTreeSet<_>>();

        for entry in active_registry_entries() {
            let expected_file = format!("{}.md", entry.id);
            assert!(
                directory_entries.contains(&expected_file),
                "active diagnostic {} is missing exact docs page {}",
                entry.id,
                expected_file
            );
        }
    }

    fn registry_entry_for(id: &str) -> &'static super::DiagnosticRegistryEntry {
        super::registry_entry(id).unwrap_or_else(|| panic!("missing registry entry for {id}"))
    }

    fn families_by_name() -> BTreeMap<&'static str, &'static super::DiagnosticFamily> {
        DIAGNOSTIC_FAMILIES
            .iter()
            .map(|family| (family.name, family))
            .collect()
    }

    fn assert_family_name(name: &str) {
        assert!(
            (3..=12).contains(&name.len()),
            "family name {name} must be 3-12 ASCII letters"
        );
        assert!(
            name.bytes().all(|byte| byte.is_ascii_uppercase()),
            "family name {name} must contain uppercase ASCII letters only"
        );
    }

    fn assert_canonical_code(id: &str) {
        let mut parts = id.split('-');
        assert_eq!(parts.next(), Some("SIFR"));
        let family = parts.next().expect("diagnostic id must include family");
        let local = parts.next().expect("diagnostic id must include local code");
        assert!(parts.next().is_none(), "diagnostic id has too many parts");
        assert_family_name(family);
        assert_eq!(local.len(), 4, "diagnostic local code must be four digits");
        assert!(
            local.bytes().all(|byte| byte.is_ascii_digit()),
            "diagnostic local code must contain digits only"
        );
    }

    fn parse_family(id: &str) -> &str {
        id.split('-')
            .nth(1)
            .expect("canonical diagnostic id must include family")
    }

    fn assert_dedupe_args_are_declared(entry: &super::DiagnosticRegistryEntry) {
        let declared_args = entry
            .declared_args
            .iter()
            .map(|arg| arg.name)
            .collect::<BTreeSet<_>>();
        for dedupe_arg in entry.dedupe_args {
            assert!(
                declared_args.contains(dedupe_arg),
                "dedupe arg {dedupe_arg} is not declared for {}",
                entry.id
            );
        }
    }

    fn assert_template_placeholders_are_declared(entry: &super::DiagnosticRegistryEntry) {
        let Some(template) = entry.message_template else {
            return;
        };
        for placeholder in placeholders(template) {
            let declaration = entry
                .declared_args
                .iter()
                .find(|arg| arg.name == placeholder)
                .unwrap_or_else(|| {
                    panic!(
                        "template placeholder {{{placeholder}}} is not declared for {}",
                        entry.id
                    )
                });
            assert_eq!(
                declaration.format,
                super::DiagnosticArgFormat::MessageAndJson,
                "json-only arg {placeholder} must not appear in the message template for {}",
                entry.id
            );
        }
    }

    fn assert_registry_strings_are_markdown_safe(entry: &super::DiagnosticRegistryEntry) {
        for value in [
            entry.id,
            entry.family,
            entry.docs_path,
            entry.summary,
            entry.owner_module.unwrap_or_default(),
            entry.message_template.unwrap_or_default(),
            entry.representative_fixture_path.unwrap_or_default(),
        ] {
            assert!(
                !value.contains('`'),
                "registry string for {} must not contain backticks: {value}",
                entry.id
            );
        }
        for arg in entry.declared_args {
            assert!(
                !arg.name.contains('`'),
                "declared arg for {} must not contain backticks: {}",
                entry.id,
                arg.name
            );
        }
        for value in entry
            .dedupe_args
            .iter()
            .chain(entry.tooling.tool_actions.iter())
        {
            assert!(
                !value.contains('`'),
                "registry metadata for {} must not contain backticks: {value}",
                entry.id
            );
        }
    }

    fn placeholders(template: &str) -> Vec<String> {
        let mut placeholders = Vec::new();
        let mut chars = template.char_indices().peekable();

        while let Some((_, ch)) = chars.next() {
            if ch != '{' {
                continue;
            }
            if matches!(chars.peek(), Some((_, '{'))) {
                chars.next();
                continue;
            }

            let mut placeholder = String::new();
            let mut closed = false;
            for (_, next) in chars.by_ref() {
                if next == '}' {
                    closed = true;
                    break;
                }
                placeholder.push(next);
            }
            assert!(closed, "unclosed template placeholder in {template}");
            assert!(
                !placeholder.is_empty(),
                "empty template placeholder in {template}"
            );
            placeholders.push(placeholder);
        }

        placeholders
    }
}
