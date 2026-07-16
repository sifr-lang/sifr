/// Render the generated task-local scope call used for cancellation-masked
/// Python cleanup. Keeping this syntax authority separate prevents the sync
/// context emitter from acquiring Tokio-specific responsibilities.
pub(super) fn cleanup_scope_call(carrier: &str, manager: &str, cause: &str) -> String {
    format!(
        "__SIFR_TASK_CANCELLATION.scope({carrier}.clone(), ::sifr_runtime::python::submit_async_context_exit_with_callbacks({manager}.__sifr_python_object, {cause}, Some(&{carrier}), {manager}.__sifr_python_callbacks))"
    )
}
