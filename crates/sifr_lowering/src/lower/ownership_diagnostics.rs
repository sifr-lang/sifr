use super::LowerCtx;
use ruff_text_size::TextRange;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode};
use sifr_type_system::Type;
use std::collections::BTreeMap;

pub(in crate::lower) fn use_after_move(ctx: &mut LowerCtx, name: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_USE_AFTER_MOVE,
        format!("use of moved value: '{name}'"),
        range,
    );
}

pub(in crate::lower) fn double_mutable_borrow(
    ctx: &mut LowerCtx,
    name: &str,
    func_name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW,
        format!(
            "cannot borrow '{name}' as mutable more than once in the same call to '{func_name}'"
        ),
        range,
    );
}

pub(in crate::lower) fn mutable_borrow_after_immutable(
    ctx: &mut LowerCtx,
    name: &str,
    func_name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW,
        format!(
            "cannot borrow '{name}' as mutable because it is already borrowed as immutable in the same call to '{func_name}'"
        ),
        range,
    );
}

pub(in crate::lower) fn immutable_borrow_after_mutable(
    ctx: &mut LowerCtx,
    name: &str,
    func_name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW,
        format!(
            "cannot borrow '{name}' as immutable because it is already borrowed as mutable in the same call to '{func_name}'"
        ),
        range,
    );
}

pub(in crate::lower) fn borrowed_parameter_store_escape(
    ctx: &mut LowerCtx,
    name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES,
        format!(
            "cannot store borrowed parameter `{name}`: borrowed parameters cannot escape -- add `own` at the signature boundary or store `{name}.clone()`"
        ),
        range,
    );
}

pub(in crate::lower) fn borrowed_parameter_return_escape(
    ctx: &mut LowerCtx,
    name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES,
        format!(
            "cannot return borrowed parameter `{name}`: borrowed parameters cannot escape -- add `own` at the signature boundary or return `{name}.clone()`"
        ),
        range,
    );
}

pub(in crate::lower) fn borrowed_affine_parameter_escape(
    ctx: &mut LowerCtx,
    name: &str,
    action: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::PYZC_INVALID_DECLARATION,
        format!(
            "cannot {action} borrowed affine Python resource parameter '{name}'; accept it with `own` before transferring the resource"
        ),
        range,
    );
}

pub(in crate::lower) fn must_use_reusable_callable_capture(
    ctx: &mut LowerCtx,
    callable_kind: &str,
    name: &str,
    ty: &Type,
    range: TextRange,
) {
    if ctx.python_context_borrows.contains_key(name) {
        ctx.error_with_code_at(
            DiagnosticCode::PYCTX_INVALID_DECLARATION,
            format!(
                "invalid Python context declaration: entered binding '{name}' is a context-scoped borrow and cannot escape through a {callable_kind} capture"
            ),
            range,
        );
        return;
    }
    let captured_resource =
        must_use_resource_type_name(ctx, ty).unwrap_or_else(|| ty.display_name());
    ctx.error_with_code_at(
        DiagnosticCode::PYZC_INVALID_DECLARATION,
        format!(
            "{callable_kind} cannot capture '{name}' of type '{captured_resource}' because reusable callables cannot own or repeatedly expose an affine or must-use Python resource"
        ),
        range,
    );
}

fn must_use_resource_type_name(ctx: &LowerCtx, ty: &Type) -> Option<String> {
    match ty.resolve_alias() {
        Type::Class { name, .. }
            if ctx
                .python_opaque_classes
                .get(name)
                .and_then(|declaration| declaration.cleanup)
                .is_some_and(|cleanup| cleanup != sifr_ir::PythonCleanupPolicy::Drop) =>
        {
            Some(ty.display_name())
        }
        Type::List(item) | Type::Result(item, _) => must_use_resource_type_name(ctx, item),
        Type::Tuple(items) | Type::Union(items) => items
            .iter()
            .find_map(|item| must_use_resource_type_name(ctx, item)),
        Type::Dict(key, value) => must_use_resource_type_name(ctx, key)
            .or_else(|| must_use_resource_type_name(ctx, value)),
        _ => None,
    }
}

pub(in crate::lower) fn reject_affine_nested_function_capture(
    ctx: &mut LowerCtx,
    function_name: &str,
    range: TextRange,
) {
    let capture = ctx
        .nested_function_captures
        .get(function_name)
        .and_then(|captures| {
            captures.iter().find(|(_, ty)| {
                ty.contains_affine_resource() || ctx.must_use_obligation_for_type(ty).is_some()
            })
        })
        .cloned();
    if let Some((capture_name, capture_ty)) = capture {
        must_use_reusable_callable_capture(
            ctx,
            "nested function",
            &capture_name,
            &capture_ty,
            range,
        );
    }
}

pub(in crate::lower) fn sync_guard_return_escape(
    ctx: &mut LowerCtx,
    label: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES,
        format!("cannot return {label}: synchronization guards cannot escape their local critical section"),
        range,
    );
}

pub(in crate::lower) fn moved_across_loop(ctx: &mut LowerCtx, name: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_MOVED_ACROSS_LOOP,
        format!(
            "value '{name}' is moved inside loop body; it would be unavailable on subsequent iterations"
        ),
        range,
    );
}

pub(in crate::lower) fn report_moved_across_loop(
    ctx: &mut LowerCtx,
    snapshot: &crate::scope::MovedSnapshot,
    range: TextRange,
) {
    for name in ctx.scope.moved_since(snapshot) {
        moved_across_loop(ctx, &name, range);
    }
}

pub(in crate::lower) fn immutable_parameter_mutation(
    ctx: &mut LowerCtx,
    name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_IMMUTABLE_PARAMETER_MUTATION,
        format!("cannot mutate through immutable parameter `{name}`: add `mut` to the parameter declaration"),
        range,
    );
}

pub(in crate::lower) fn same_call_place_conflict(
    ctx: &mut LowerCtx,
    place: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_DOUBLE_MUTABLE_BORROW,
        format!("borrow conflict for {place} in the same call"),
        range,
    );
}

pub(in crate::lower) fn unsupported_mutable_receiver_place(
    ctx: &mut LowerCtx,
    place: &str,
    range: TextRange,
) {
    let mut args = BTreeMap::new();
    args.insert(
        "place".to_string(),
        DiagnosticArg::String(place.to_string()),
    );
    ctx.error_with_code_args_help_at(
        DiagnosticCode::OWN_UNSUPPORTED_MUTABLE_RECEIVER_PLACE,
        format!("mutable method receiver {place} is not a supported storage place"),
        args,
        None,
        range,
    );
}

pub(in crate::lower) fn constructor_storage_unavailable(
    ctx: &mut LowerCtx,
    missing_fields: &[String],
    missing_parent: bool,
    range: TextRange,
) {
    let field_places = missing_fields
        .iter()
        .map(|field| format!("self.{field}"))
        .collect::<Vec<_>>();
    let message = match (missing_parent, field_places.is_empty()) {
        (true, true) => {
            "constructor uses self before inherited storage is initialized; call super().__init__(...) first"
                .to_string()
        }
        (true, false) => format!(
            "constructor uses self before storage is initialized; call super().__init__(...) first and initialize {}",
            field_places.join(", ")
        ),
        (false, false) => format!(
            "constructor uses self before field storage is initialized: {}",
            field_places.join(", ")
        ),
        (false, true) => "constructor uses self before its storage is initialized".to_string(),
    };
    let mut args = BTreeMap::new();
    args.insert(
        "place".to_string(),
        DiagnosticArg::String("self".to_string()),
    );
    ctx.error_with_code_args_help_at(
        DiagnosticCode::OWN_UNSUPPORTED_MUTABLE_RECEIVER_PLACE,
        message,
        args,
        Some(
            "initialize every declared field and inherited storage before the first statement that reads or mutates self"
                .to_string(),
        ),
        range,
    );
}

pub(in crate::lower) fn immutable_parameter_reassignment(
    ctx: &mut LowerCtx,
    name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_IMMUTABLE_PARAMETER_REASSIGNMENT,
        format!(
            "cannot reassign immutable parameter `{name}`: add `mut` to the parameter declaration"
        ),
        range,
    );
}

pub(in crate::lower) fn reject_borrowed_affine_parameter_reassignment(
    ctx: &mut LowerCtx,
    name: &str,
    is_parameter: bool,
    ty: &Type,
    range: TextRange,
) -> bool {
    if !is_parameter || !ctx.borrowed_params.contains(name) || !ty.contains_affine_resource() {
        return false;
    }
    ctx.error_with_code_at(
        DiagnosticCode::PYZC_INVALID_DECLARATION,
        format!(
            "cannot reassign borrowed affine Python resource parameter '{name}'; mutable parameter shadowing would require cloning it"
        ),
        range,
    );
    true
}

pub(in crate::lower) fn immutable_bytes_subscript_assignment(ctx: &mut LowerCtx, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_IMMUTABLE_BYTES_ASSIGNMENT,
        "bytes is immutable; subscript assignment is not supported".to_string(),
        range,
    );
}

pub(in crate::lower) fn immutable_bytes_augmented_subscript_assignment(
    ctx: &mut LowerCtx,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_IMMUTABLE_BYTES_AUGMENTED_ASSIGNMENT,
        "bytes is immutable; augmented subscript assignment is not supported".to_string(),
        range,
    );
}

pub(in crate::lower) fn mutable_borrow_across_await(
    ctx: &mut LowerCtx,
    name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_BORROW_ACROSS_AWAIT,
        format!(
            "mutable borrow `{name}` cannot cross await; finish the mutation before awaiting or transfer ownership with `own`"
        ),
        range,
    );
}

pub(in crate::lower) fn mutable_borrow_across_yield(
    ctx: &mut LowerCtx,
    name: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_BORROW_ACROSS_AWAIT,
        format!(
            "mutable borrow `{name}` cannot cross async generator yield; finish the mutation before yielding or transfer ownership with `own`"
        ),
        range,
    );
}

pub(in crate::lower) fn sync_guard_across_await(
    ctx: &mut LowerCtx,
    name: &str,
    label: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_BORROW_ACROSS_AWAIT,
        format!("{label} `{name}` cannot cross await; release the guard before awaiting"),
        range,
    );
}

pub(in crate::lower) fn non_send_task_capture(
    ctx: &mut LowerCtx,
    value: &str,
    ty: &str,
    reason: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_NON_SEND_TASK_CAPTURE,
        format!(
            "scope.spawn() cannot move `{value}` of type `{ty}` across a task boundary because {reason}; use an explicit synchronization primitive or keep the value in the current task"
        ),
        range,
    );
}

pub(in crate::lower) fn non_send_channel_element(
    ctx: &mut LowerCtx,
    value: &str,
    ty: &str,
    reason: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_NON_SEND_CHANNEL_ELEMENT,
        format!(
            "channel send cannot transfer `{value}` of type `{ty}` because {reason}; use an explicit synchronization primitive or keep the value in the current task"
        ),
        range,
    );
}

pub(in crate::lower) fn non_share_safe_shared_value(
    ctx: &mut LowerCtx,
    value: &str,
    ty: &str,
    reason: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_NON_SHARE_SAFE_SHARED_VALUE,
        format!(
            "Shared cannot publish `{value}` of type `{ty}` because {reason}; wrap mutable state in `sync.Lock`/`sync.RwLock` or keep ownership local"
        ),
        range,
    );
}

pub(in crate::lower) fn non_ipc_serializable_payload(
    ctx: &mut LowerCtx,
    value: &str,
    ty: &str,
    reason: &str,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::OWN_NON_IPC_SERIALIZABLE_PAYLOAD,
        format!(
            "typed IPC payload cannot transfer `{value}` of type `{ty}` because {reason}; pass owned schema data instead of process-local resources"
        ),
        range,
    );
}
