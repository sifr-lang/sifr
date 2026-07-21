use super::LowerCtx;
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_type_system::Type;

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

pub(in crate::lower) fn affine_reusable_callable_capture(
    ctx: &mut LowerCtx,
    callable_kind: &str,
    name: &str,
    ty: &Type,
    range: TextRange,
) {
    ctx.error_with_code_at(
        DiagnosticCode::PYZC_INVALID_DECLARATION,
        format!(
            "{callable_kind} cannot capture '{name}' of type '{}' because reusable callables cannot own or repeatedly expose an affine Python resource",
            ty.display_name()
        ),
        range,
    );
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
            captures
                .iter()
                .find(|(_, ty)| ty.contains_affine_resource())
        })
        .cloned();
    if let Some((capture_name, capture_ty)) = capture {
        affine_reusable_callable_capture(ctx, "nested function", &capture_name, &capture_ty, range);
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
