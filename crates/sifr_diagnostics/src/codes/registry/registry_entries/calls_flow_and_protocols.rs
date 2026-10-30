//! Call, ownership, flow, match, protocol, class, result, and stdlib diagnostics.

use super::super::DiagnosticRegistryEntry;
use crate::model::Severity;

pub(super) const ENTRIES: &[DiagnosticRegistryEntry] = &[
    active_entry!(
        "SIFR-CALL-0001",
        "CALL",
        "Wrong positional argument count.",
        Severity::Error,
        "{callable} takes {quantifier} {expected_count} argument(s), got {actual_count}",
        "sifr_lowering::lower",
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
        "{callable} got an unexpected keyword argument '{keyword}'",
        "sifr_lowering::lower",
        [arg!("callable"), arg!("keyword")],
        ["callable", "keyword"]
    ),
    active_entry!(
        "SIFR-CALL-0003",
        "CALL",
        "Duplicate argument from positional and keyword overlap.",
        Severity::Error,
        "{callable} got multiple values for argument '{argument}'",
        "sifr_lowering::lower",
        [arg!("callable"), arg!("argument")],
        ["callable", "argument"]
    ),
    active_entry!(
        "SIFR-CALL-0004",
        "CALL",
        "Missing required argument.",
        Severity::Error,
        "{callable} missing required argument '{argument}'",
        "sifr_lowering::lower",
        [arg!("callable"), arg!("argument")],
        ["callable", "argument"]
    ),
    active_entry!(
        "SIFR-CALL-0005",
        "CALL",
        "Callable arity failure or expression is not callable.",
        Severity::Error,
        "{callable} callable expects {expected_count} argument(s), got {actual_count} iterable(s)",
        "sifr_lowering::lower",
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
        "use of moved value {binding}",
        "sifr_lowering::lower",
        [arg!("binding")],
        ["binding"]
    ),
    active_entry!(
        "SIFR-OWN-0002",
        "OWN",
        "Conflicting borrow.",
        Severity::Error,
        "borrow conflict for {binding}",
        "sifr_lowering::lower",
        [arg!("binding")],
        ["binding"]
    ),
    active_entry!(
        "SIFR-OWN-0003",
        "OWN",
        "Borrowed parameter escapes by return or store.",
        Severity::Error,
        "borrowed parameter {binding} escapes",
        "sifr_lowering::lower",
        [arg!("binding"), json_arg!("escape_kind")],
        ["binding", "escape_kind"]
    ),
    active_entry!(
        "SIFR-OWN-0004",
        "OWN",
        "Moved value is reused across loop iterations.",
        Severity::Error,
        "moved value {binding} is reused across loop iterations",
        "sifr_lowering::lower",
        [arg!("binding")],
        ["binding"]
    ),
    active_entry!(
        "SIFR-OWN-0005",
        "OWN",
        "Immutable parameter is mutated.",
        Severity::Error,
        "cannot mutate through immutable parameter {binding}",
        "sifr_lowering::lower",
        [arg!("binding")],
        ["binding"]
    ),
    active_entry!(
        "SIFR-OWN-0006",
        "OWN",
        "Immutable parameter is reassigned.",
        Severity::Error,
        "cannot reassign immutable parameter {binding}",
        "sifr_lowering::lower",
        [arg!("binding")],
        ["binding"]
    ),
    active_entry!(
        "SIFR-OWN-0007",
        "OWN",
        "Immutable bytes value is mutated.",
        Severity::Error,
        "bytes is immutable; subscript assignment is not supported",
        "sifr_lowering::lower::statements",
        [],
        []
    ),
    active_entry!(
        "SIFR-OWN-0008",
        "OWN",
        "Immutable bytes value is mutated by augmented assignment.",
        Severity::Error,
        "bytes is immutable; augmented subscript assignment is not supported",
        "sifr_lowering::lower::aug_assign_lowering",
        [],
        []
    ),
    active_entry!(
        "SIFR-OWN-0009",
        "OWN",
        "Mutable borrow remains live across an await point.",
        Severity::Error,
        "mutable borrow {binding} cannot cross await",
        "sifr_lowering::lower",
        [arg!("binding")],
        ["binding"]
    ),
    active_entry!(
        "SIFR-OWN-0010",
        "OWN",
        "Non-sendable value crosses a spawned task boundary.",
        Severity::Error,
        "scope.spawn() cannot move {value} of type {type_name} across a task boundary",
        "sifr_lowering::lower::task_scope_calls",
        [arg!("value"), arg!("type_name"), json_arg!("reason")],
        ["value", "type_name", "reason"]
    ),
    active_entry!(
        "SIFR-OWN-0011",
        "OWN",
        "Non-sendable value is sent through a channel.",
        Severity::Error,
        "channel send cannot transfer {value} of type {type_name}",
        "sifr_lowering::lower",
        [arg!("value"), arg!("type_name"), json_arg!("reason")],
        ["value", "type_name", "reason"]
    ),
    active_entry!(
        "SIFR-OWN-0012",
        "OWN",
        "Non-share-safe value is wrapped in sync.Shared.",
        Severity::Error,
        "Shared cannot publish {value} of type {type_name}",
        "sifr_lowering::lower",
        [arg!("value"), arg!("type_name"), json_arg!("reason")],
        ["value", "type_name", "reason"]
    ),
    active_entry!(
        "SIFR-OWN-0013",
        "OWN",
        "Non-IPC-serializable value is used as a typed IPC payload.",
        Severity::Error,
        "typed IPC payload cannot transfer {value} of type {type_name}",
        "sifr_lowering::lower::ipc_payload_calls",
        [arg!("value"), arg!("type_name"), json_arg!("reason")],
        ["value", "type_name", "reason"]
    ),
    active_entry!(
        "SIFR-OWN-0014",
        "OWN",
        "Unsupported mutable receiver place.",
        Severity::Error,
        "mutable method receiver {place} is not a supported storage place",
        "sifr_lowering::lower::method_receiver_places",
        [arg!("place")],
        ["place"]
    ),
    active_entry!(
        "SIFR-FLOW-0001",
        "FLOW",
        "Break outside a loop.",
        Severity::Error,
        "'break' outside of loop",
        "sifr_lowering::lower::statements",
        [],
        []
    ),
    active_entry!(
        "SIFR-FLOW-0002",
        "FLOW",
        "Continue outside a loop.",
        Severity::Error,
        "'continue' outside of loop",
        "sifr_lowering::lower::statements",
        [],
        []
    ),
    active_entry!(
        "SIFR-FLOW-0003",
        "FLOW",
        "Invalid nonlocal or nested-function flow.",
        Severity::Error,
        "recursive nested function '{function}' cannot mutate captured state with nonlocal yet",
        "sifr_lowering::lower",
        [arg!("function")],
        ["function"]
    ),
    active_entry!(
        "SIFR-FLOW-0004",
        "FLOW",
        "Function may finish without returning a required value.",
        Severity::Error,
        "function '{function}' must return a value of type '{return_type}' on all control-flow paths",
        "sifr_lowering::lower::typing_and_functions",
        [arg!("function"), arg!("return_type")],
        ["function", "return_type"]
    ),
    active_entry!(
        "SIFR-FLOW-0005",
        "FLOW",
        "Control-flow condition has an unsupported type.",
        Severity::Error,
        "{keyword} condition must be bool or collection/string truthiness, got '{actual}'",
        "sifr_lowering::lower::control_flow_conditions",
        [arg!("keyword"), arg!("actual")],
        ["keyword", "actual"]
    ),
    active_entry!(
        "SIFR-FLOW-0006",
        "FLOW",
        "Statement form is unsupported by HIR lowering.",
        Severity::Error,
        "unsupported statement form: {form}",
        "sifr_lowering::lower::statements",
        [arg!("form")],
        ["form"]
    ),
    active_entry!(
        "SIFR-FLOW-0007",
        "FLOW",
        "Assignment target form is unsupported by HIR lowering.",
        Severity::Error,
        "invalid assignment target: {target}",
        "sifr_lowering::lower::statements",
        [arg!("target")],
        ["target"]
    ),
    active_entry!(
        "SIFR-FLOW-0008",
        "FLOW",
        "For-loop iteration form or source is invalid.",
        Severity::Error,
        "invalid for-loop iteration: {reason}",
        "sifr_lowering::lower::statements",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-FLOW-0901",
        "FLOW",
        "Unreachable statement ignored during lowering.",
        Severity::Warning,
        "unreachable statement ignored",
        "sifr_lowering::lower::statements",
        [],
        []
    ),
    active_entry!(
        "SIFR-MATCH-0001",
        "MATCH",
        "Non-exhaustive match.",
        Severity::Error,
        "non-exhaustive match: enum {enum_name} has uncovered variants: {uncovered}",
        "sifr_lowering::lower::statements",
        [arg!("enum_name"), arg!("uncovered")],
        ["enum_name", "uncovered"]
    ),
    active_entry!(
        "SIFR-MATCH-0002",
        "MATCH",
        "Match guard must be bool.",
        Severity::Error,
        "match guard must be a bool expression, got {actual}",
        "sifr_lowering::lower::statements",
        [arg!("actual")],
        ["actual"]
    ),
    active_entry!(
        "SIFR-MATCH-0003",
        "MATCH",
        "Invalid class pattern field.",
        Severity::Error,
        "class {class_name} has no field {field}",
        "sifr_lowering::lower::statements",
        [arg!("field"), arg!("class_name")],
        ["field", "class_name"]
    ),
    active_entry!(
        "SIFR-MATCH-0004",
        "MATCH",
        "Invalid or unsupported match pattern form.",
        Severity::Error,
        "invalid match pattern: {reason}",
        "sifr_lowering::lower::statements",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-PROTO-0001",
        "PROTO",
        "Protocol bound or conformance failure.",
        Severity::Error,
        "type '{actual}' does not implement protocol '{protocol}' required by type parameter '{type_param}'",
        "sifr_lowering::lower",
        [arg!("actual"), arg!("protocol"), arg!("type_param")],
        ["actual", "protocol", "type_param"]
    ),
    active_entry!(
        "SIFR-PROTO-0002",
        "PROTO",
        "Invalid iterator or reversible protocol signature.",
        Severity::Error,
        "class '{type_name}' must return {expected}",
        "sifr_lowering::lower::classes",
        [arg!("type_name"), arg!("expected")],
        ["type_name", "expected"]
    ),
    active_entry!(
        "SIFR-PROTO-0003",
        "PROTO",
        "Context-manager protocol is missing.",
        Severity::Error,
        "type '{type_name}' does not implement the ContextManager protocol (missing __enter__ and __exit__ methods)",
        "sifr_lowering::lower::statements",
        [arg!("type_name")],
        ["type_name"]
    ),
    active_entry!(
        "SIFR-PROTO-0004",
        "PROTO",
        "Hashable or comparable protocol is required.",
        Severity::Error,
        "hash() argument must be hashable, got '{type_name}'",
        "sifr_lowering::lower::expressions",
        [arg!("type_name")],
        ["type_name"]
    ),
    active_entry!(
        "SIFR-PROTO-0005",
        "PROTO",
        "Protocol receiver convention mismatch.",
        Severity::Error,
        "class '{class_name}' method '{method}' requires a mutable receiver but protocol '{protocol}' declares a shared receiver",
        "sifr_lowering::lower::method_receiver_analysis",
        [arg!("class_name"), arg!("method"), arg!("protocol")],
        ["class_name", "method", "protocol"]
    ),
    active_entry!(
        "SIFR-PROTO-0006",
        "PROTO",
        "Fixed Rust trait receiver convention is violated.",
        Severity::Error,
        "class '{class_name}' method '{method}' violates the receiver convention fixed by Rust trait '{trait_name}'",
        "sifr_lowering::lower::method_receiver_analysis",
        [arg!("class_name"), arg!("method"), arg!("trait_name")],
        ["class_name", "method", "trait_name"]
    ),
    active_entry!(
        "SIFR-CLASS-0001",
        "CLASS",
        "Class fields require an initializer or super initializer.",
        Severity::Error,
        "class '{class_name}' has fields but no __init__; parent fields will not be initialized. Define an explicit __init__ with super().__init__(...)",
        "sifr_lowering::lower::classes",
        [arg!("class_name")],
        ["class_name"]
    ),
    active_entry!(
        "SIFR-CLASS-0002",
        "CLASS",
        "Required field declared after a defaulted field.",
        Severity::Error,
        "class '{class_name}': required field '{field}' declared after field with default value",
        "sifr_lowering::lower::classes",
        [arg!("class_name"), arg!("field")],
        ["class_name", "field"]
    ),
    active_entry!(
        "SIFR-CLASS-0003",
        "CLASS",
        "Duplicate enum or class value, or invalid variant.",
        Severity::Error,
        "enum '{enum_name}' has duplicate value {value}: variants '{existing_variant}' and '{duplicate_variant}'",
        "sifr_lowering::lower::classes",
        [
            arg!("enum_name"),
            arg!("value"),
            arg!("existing_variant"),
            arg!("duplicate_variant")
        ],
        [
            "enum_name",
            "value",
            "existing_variant",
            "duplicate_variant"
        ]
    ),
    active_entry!(
        "SIFR-CLASS-0004",
        "CLASS",
        "Missing class field.",
        Severity::Error,
        "type '{type_name}' has no field '{field}'",
        "sifr_lowering::lower::expressions",
        [arg!("type_name"), arg!("field")],
        ["type_name", "field"]
    ),
    active_entry!(
        "SIFR-CLASS-0005",
        "CLASS",
        "Invalid class base.",
        Severity::Error,
        "invalid base class for '{class_name}': {reason}",
        "sifr_lowering::lower::classes",
        [arg!("class_name"), arg!("reason")],
        ["class_name", "reason"]
    ),
    active_entry!(
        "SIFR-CLASS-0006",
        "CLASS",
        "Unsupported class declaration.",
        Severity::Error,
        "unsupported class declaration in '{class_name}': {detail}",
        "sifr_lowering::lower::classes",
        [arg!("class_name"), arg!("detail")],
        ["class_name", "detail"]
    ),
    active_entry!(
        "SIFR-RESULT-0001",
        "RESULT",
        "Unused Result value.",
        Severity::Error,
        "unused Result value",
        "sifr_lowering::lower",
        [],
        []
    ),
    active_entry!(
        "SIFR-RESULT-0002",
        "RESULT",
        "Invalid Result error type.",
        Severity::Error,
        "invalid Result error type {error_type}",
        "sifr_lowering::lower",
        [arg!("error_type")],
        ["error_type"]
    ),
    active_entry!(
        "SIFR-RESULT-0003",
        "RESULT",
        "Invalid raise expression.",
        Severity::Error,
        "invalid raise expression of type {actual}",
        "sifr_lowering::lower::statements",
        [arg!("actual")],
        ["actual"]
    ),
    active_entry!(
        "SIFR-RESULT-0004",
        "RESULT",
        "Except arm references an unknown error type.",
        Severity::Error,
        "unknown except error type '{error_type}'",
        "sifr_lowering::lower::statements",
        [arg!("error_type")],
        ["error_type"]
    ),
    active_entry!(
        "SIFR-RESULT-0005",
        "RESULT",
        "Try body error types are not fully covered by except arms.",
        Severity::Error,
        "except arms do not cover all error types from try body: {uncovered}",
        "sifr_lowering::lower::statements",
        [arg!("uncovered")],
        ["uncovered"]
    ),
    active_entry!(
        "SIFR-RESULT-0006",
        "RESULT",
        "Except arm type expression has an unsupported form.",
        Severity::Error,
        "invalid except error type: {reason}",
        "sifr_lowering::lower::statements",
        [arg!("reason")],
        ["reason"]
    ),
    active_entry!(
        "SIFR-STDLIB-0001",
        "STDLIB",
        "Unsupported standard-library constructor, method, or surface.",
        Severity::Error,
        "defaultdict() does not support keyword arguments",
        "sifr_lowering::lower::builtin_calls",
        [],
        []
    ),
    active_entry!(
        "SIFR-STDLIB-0003",
        "STDLIB",
        "Embedded standard-library bootstrap failure.",
        Severity::Error,
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
        "standard-library cache failed during {operation}",
        "sifr_driver::stdlib::cache",
        [arg!("operation")],
        ["operation"]
    ),
];
