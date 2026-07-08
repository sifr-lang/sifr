//! Parse, name, import, type, async, decimal, and integer diagnostics.

use super::super::DiagnosticRegistryEntry;
use crate::model::Severity;

pub(super) const ENTRIES: &[DiagnosticRegistryEntry] = &[
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
            "sifr_lowering::lower",
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
            "sifr_lowering::lower",
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
            "sifr_lowering::lower::typing_and_functions",
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
            "sifr_lowering::lower",
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
            "sifr_lowering::lower::module_function_registry",
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
            "sifr_lowering::lower::statements",
            [arg!("name")],
            ["name"]
        ),
    active_entry!(
            "SIFR-IMPORT-0001",
            "IMPORT",
            "Forbidden private sysroot declaration import.",
            Severity::Error,
            "crates/sifr/tests/e2e/fail/stdlib_intrinsic_direct_import.sifr",
            "cannot import from '{module}' — private sysroot declarations can only be imported by public sysroot stdlib source",
            "sifr_lowering::lower",
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
            "sifr_lowering::lower",
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
            "sifr_lowering::lower",
            [arg!("form")],
            ["form"]
        ),
    active_entry!(
            "SIFR-IMPORT-0004",
            "IMPORT",
            "Private module member import.",
            Severity::Error,
            "crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs",
            "cannot import private name '{name}' from module '{module}'",
            "sifr_lowering::lower",
            [arg!("name"), arg!("module")],
            ["name", "module"]
        ),
    active_entry!(
            "SIFR-IMPORT-0005",
            "IMPORT",
            "Ambiguous source module import target.",
            Severity::Error,
            "verification/areas/project_workspace/fixtures/project/workspace_ambiguous_import_canonical",
            "ambiguous import target: '{module}'",
            "sifr_driver::project::discovery",
            [
                arg!("module"),
                json_arg!("candidate_paths"),
                json_arg!("resolution_scope")
            ],
            ["module", "candidate_paths", "resolution_scope"]
        ),
    active_entry!(
            "SIFR-IMPORT-0006",
            "IMPORT",
            "Source module namespace and file import collision.",
            Severity::Error,
            "verification/areas/project_workspace/fixtures/project/workspace_namespace_collision_canonical",
            "import target '{module}' collides with a namespace package",
            "sifr_driver::project::discovery",
            [
                arg!("module"),
                json_arg!("resolved_path"),
                json_arg!("parent_path")
            ],
            ["module", "resolved_path", "parent_path"]
        ),
    active_entry!(
            "SIFR-IMPORT-0007",
            "IMPORT",
            "Circular source module import graph.",
            Severity::Error,
            "verification/areas/project_workspace/fixtures/project/import_cycle_source_spans",
            "circular import detected: {cycle}",
            "sifr_driver::project::compile_order",
            [arg!("cycle"), json_arg!("cycle_edges")],
            ["cycle", "cycle_edges"]
        ),
    active_entry!(
            "SIFR-IMPORT-0008",
            "IMPORT",
            "Bare CPython-style stdlib import attempt.",
            Severity::Error,
            "crates/sifr/tests/e2e/fail/bare_stdlib_from_math.sifr",
            "bare stdlib import '{bare_module}'; Sifr stdlib lives under 'sifr.*'",
            "sifr_lowering::lower / sifr_driver::project::discovery",
            [
                arg!("bare_module"),
                json_arg!("suggested_module"),
                json_arg!("imported_names")
            ],
            ["bare_module", "suggested_module", "imported_names"]
        ),
    active_entry!(
            "SIFR-IMPORT-0009",
            "IMPORT",
            "Unsupported legacy Sifr stdlib module import.",
            Severity::Error,
            "crates/sifr/tests/e2e/fail/legacy_sifr_asyncio_removed.sifr",
            "legacy stdlib module '{legacy_module}' is unsupported; use '{suggested_module}'",
            "sifr_lowering::lower",
            [
                arg!("legacy_module"),
                arg!("suggested_module"),
                json_arg!("imported_names"),
                json_arg!("reason")
            ],
            ["legacy_module", "suggested_module", "imported_names", "reason"]
        ),
    active_entry!(
            "SIFR-TYPE-0002",
            "TYPE",
            "Expected and actual types are incompatible.",
            Severity::Error,
            "crates/sifr/tests/e2e/fail/type_mismatch.sifr",
            "type mismatch: expected {expected}, got {actual}",
            "sifr_lowering::lower",
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
            "sifr_lowering::lower::if_expression",
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
            "sifr_lowering::lower::typing_and_functions",
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
            "sifr_lowering::lower::typing_and_functions",
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
            "sifr_lowering::lower::container_literal_specialization",
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
            "sifr_lowering::lower::tuple_unpack",
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
            "sifr_lowering::lower::expressions",
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
            "sifr_lowering::lower::typing_and_functions",
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
            "sifr_lowering::lower::expressions",
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
            "sifr_lowering::lower::arithmetic_warnings",
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
            "sifr_lowering::lower::builtin_calls",
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
            "sifr_lowering::lower::typing_and_functions",
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
            "sifr_lowering::lower::async_await",
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
            "sifr_lowering::lower::workload_annotations",
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
            "sifr_lowering::lower::workload_annotations",
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
            "sifr_lowering::lower::workload_annotations",
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
            "sifr_lowering::lower::typing_and_functions",
            [arg!("message")],
            ["message"]
        ),
    active_entry!(
            "SIFR-ASYNC-0007",
            "ASYNC",
            "Shell execution function called directly from async context.",
            Severity::Error,
            "crates/sifr/tests/e2e/fail/process_shell_exec_direct_async_rejected.sifr",
            "{message}",
            "sifr_lowering::lower::workload_annotations",
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
            "sifr_lowering::lower",
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
            "sifr_lowering::lower",
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
            "sifr_lowering::lower",
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
            "sifr_lowering::lower",
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
            "sifr_lowering::lower::decimal_methods",
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
            "sifr_lowering::lower::decimal_methods",
            [arg!("argument"), arg!("value"), json_arg!("operation")],
            ["argument", "value", "operation"]
        ),
    active_entry!(
            "SIFR-INT-0001",
            "INT",
            "Fixed-width integer literal or const expression is out of range.",
            Severity::Error,
            "crates/sifr_lowering/src/lower/expressions_tests.rs::test_fixed_width_literal_assignment_out_of_range_has_int_code",
            "integer value {value} does not fit target type {target_type}; valid range is {min_value}..={max_value}",
            "sifr_lowering::lower::fixed_width_fitting",
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
            "sifr_lowering::lower::typing_and_functions",
            [arg!("name")],
            ["name"]
        ),
    active_entry!(
            "SIFR-INT-0004",
            "INT",
            "Compile-time integer evaluation budget exceeded.",
            Severity::Error,
            "crates/sifr_lowering/src/lower/expressions_tests.rs::test_large_integer_literal_over_budget_has_int_code",
            "integer literal exceeds compile-time evaluation budget: {digits} decimal digits (max {max_digits})",
            "sifr_lowering::lower::integer_literal_diagnostics",
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
            "sifr_lowering::lower::integer_failure_diagnostics",
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
            "sifr_lowering::lower::typing_and_functions",
            [],
            []
        ),
];
