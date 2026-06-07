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
    pub const IMPORT_AMBIGUOUS_SOURCE_MODULE: Self = Self::new("SIFR-IMPORT-0005", Severity::Error);
    pub const IMPORT_NAMESPACE_COLLISION: Self = Self::new("SIFR-IMPORT-0006", Severity::Error);
    pub const IMPORT_CYCLE: Self = Self::new("SIFR-IMPORT-0007", Severity::Error);
    pub const IMPORT_BARE_STDLIB: Self = Self::new("SIFR-IMPORT-0008", Severity::Error);
    pub const IMPORT_UNSUPPORTED_LEGACY_STDLIB: Self =
        Self::new("SIFR-IMPORT-0009", Severity::Error);

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
    pub const ASYNC_DIRECT_SHELL_EXEC_CALL: Self = Self::new("SIFR-ASYNC-0007", Severity::Error);

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

    pub const IO_TEXT_OPEN_REQUIRES_ENCODING: Self = Self::new("SIFR-IO-0801", Severity::Error);
    pub const IO_OPEN_MODE_REQUIRES_LITERAL: Self = Self::new("SIFR-IO-0802", Severity::Error);

    pub const ENCODING_HANDLER_REQUIRES_STATIC_VALUE: Self =
        Self::new("SIFR-ENCODING-0803", Severity::Error);

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
    pub const LINT_TODO_COMMENT: Self = Self::new("SIFR-LINT-0005", Severity::Warning);
    pub const LINT_BOOLEAN_POSITIONAL_ARGUMENT: Self =
        Self::new("SIFR-LINT-0006", Severity::Warning);
    pub const LINT_LARGE_PARAMETER_LIST: Self = Self::new("SIFR-LINT-0007", Severity::Warning);
    pub const LINT_DUPLICATE_IMPORT: Self = Self::new("SIFR-LINT-0008", Severity::Warning);

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
    pub const SELF_UPDATE_UNMANAGED_RECEIPT: Self = Self::new("SIFR-BUILD-0901", Severity::Error);

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
        name: "IO",
        summary: "File and stream text/binary boundary diagnostics.",
        reserved_base: "SIFR-IO-0000",
    },
    DiagnosticFamily {
        name: "ENCODING",
        summary: "Text encoding and error-handler diagnostics.",
        reserved_base: "SIFR-ENCODING-0000",
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
        $crate::codes::DiagnosticArgDeclaration {
            name: $name,
            format: $crate::codes::DiagnosticArgFormat::MessageAndJson,
        }
    };
}

macro_rules! json_arg {
    ($name:literal) => {
        $crate::codes::DiagnosticArgDeclaration {
            name: $name,
            format: $crate::codes::DiagnosticArgFormat::JsonOnly,
        }
    };
}

macro_rules! active_entry {
    ($id:literal, $family:literal, $summary:literal, $severity:expr, $fixture:literal, $template:literal, $owner:literal, [$($arg:expr),* $(,)?], [$($dedupe:literal),* $(,)?]) => {
        $crate::codes::DiagnosticRegistryEntry {
            id: $id,
            family: $family,
            summary: $summary,
            state: $crate::codes::DiagnosticState::Active,
            docs_path: concat!("docs/errors/", $id, ".md"),
            representative_fixture_path: Some($fixture),
            message_template: Some($template),
            owner_module: Some($owner),
            declared_args: &[$($arg),*],
            dedupe_args: &[$($dedupe),*],
            declared_severity: Some($severity),
            tooling: $crate::codes::DiagnosticTooling::DEFAULT,
        }
    };
}

mod registry_entries;

pub use registry_entries::DIAGNOSTIC_REGISTRY;
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
    DiagnosticCode::IMPORT_AMBIGUOUS_SOURCE_MODULE,
    DiagnosticCode::IMPORT_NAMESPACE_COLLISION,
    DiagnosticCode::IMPORT_CYCLE,
    DiagnosticCode::IMPORT_BARE_STDLIB,
    DiagnosticCode::IMPORT_UNSUPPORTED_LEGACY_STDLIB,
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
    DiagnosticCode::ASYNC_DIRECT_SHELL_EXEC_CALL,
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
    DiagnosticCode::IO_TEXT_OPEN_REQUIRES_ENCODING,
    DiagnosticCode::IO_OPEN_MODE_REQUIRES_LITERAL,
    DiagnosticCode::ENCODING_HANDLER_REQUIRES_STATIC_VALUE,
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
    DiagnosticCode::LINT_TODO_COMMENT,
    DiagnosticCode::LINT_BOOLEAN_POSITIONAL_ARGUMENT,
    DiagnosticCode::LINT_LARGE_PARAMETER_LIST,
    DiagnosticCode::LINT_DUPLICATE_IMPORT,
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
    DiagnosticCode::SELF_UPDATE_UNMANAGED_RECEIPT,
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
