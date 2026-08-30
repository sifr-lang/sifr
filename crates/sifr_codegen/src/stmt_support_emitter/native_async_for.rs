use super::python_context::{rewrite_context_control_flow, rust_stmts_always_exit};
use crate::{HirExpr, HirStmt, RustEmitter, RustExpr, RustStmt, Type};

const CLEANUP_BUDGET_SECONDS: u64 = 5;

impl RustEmitter {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn try_lower_closable_native_async_for_for_ir(
        &mut self,
        target: &str,
        iter: &HirExpr,
        iter_error_ty: &Type,
        close_error_ty: &Type,
        declared_active_error_ty: &Type,
        body: &[HirStmt],
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let suffix = self.python_context_counter;
        self.python_context_counter += 1;
        let mut names = NativeAsyncForNames::new(suffix);
        let context_is_nested = self.python_context_envelope_depth > 0;
        let receiver_setup = if let HirExpr::Name { name, .. } = iter {
            names.receiver.clone_from(name);
            String::new()
        } else {
            let Some(value) = self.lower_rendered_expr_for_ir(iter)? else {
                return Ok(None);
            };
            format!(
                "let mut {} = {};",
                names.receiver,
                crate::render_expr(&value)
            )
        };

        self.python_context_envelope_depth += 1;
        let lowered_body =
            self.lower_checked_sequence_loop_body_for_ir(body, &[], &RustStmt::Continue, &[]);
        self.python_context_envelope_depth -= 1;
        let mut rewritten = rewrite_context_control_flow(
            lowered_body?.ok_or_else(|| {
                crate::CodegenError::new("closable native async-for body could not be lowered")
            })?,
            0,
        );
        let can_return = crate::hir_analysis::queries::body_contains_return(body);
        let effective_active_error_ty = self
            .try_closure_error_type_info
            .last()
            .and_then(Clone::clone);
        let active_error_ty = effective_active_error_ty
            .as_ref()
            .unwrap_or(declared_active_error_ty);
        let active_error_rust = crate::render_type(&crate::sifr_type_to_rust_type(active_error_ty));
        let return_expression_type =
            self.native_async_for_return_expression_type(&active_error_rust);
        let normal_outcome_type = if can_return {
            format!("Option<{return_expression_type}>")
        } else {
            "()".to_string()
        };
        if !rust_stmts_always_exit(&rewritten) {
            rewritten.push(RustStmt::Return(Some(RustExpr::Verbatim(
                if can_return {
                    "Ok(Ok(None))"
                } else {
                    "Ok(Ok(()))"
                }
                .to_string(),
            ))));
        }
        let body_source = crate::render_stmts(&rewritten);
        let target_pattern = if crate::hir_analysis::queries::stmts_reference_var(body, target) {
            target.to_string()
        } else {
            format!("_{target}")
        };
        let next_expr = if matches!(iter_error_ty.resolve_alias(), Type::Never) {
            format!("{}.anext().await", names.receiver)
        } else {
            format!("{}.anext().await?", names.receiver)
        };
        let close_primary = primary_close(&names.receiver, close_error_ty);
        let return_arm = if !can_return {
            String::new()
        } else if context_is_nested {
            format!(
                "Some(Ok(Ok(Some(Ok(Some(value)))))) => {{ {close_primary} return Ok(Ok(Some(value))); }},"
            )
        } else {
            format!("Some(Ok(Ok(Some(Ok(Some(value)))))) => {{ {close_primary} return value; }},")
        };
        let normal_pattern = if can_return { "None" } else { "()" };
        let error_cause = native_error_cause(active_error_ty, &names.body_error);
        let error_cleanup = abnormal_close(
            &names,
            close_error_ty,
            &error_cause,
            &iter.ty().display_name(),
            "return Err(body_error);",
        );
        let cancellation_cleanup = abnormal_close(
            &names,
            close_error_ty,
            "Cancellation",
            &iter.ty().display_name(),
            &format!(
                "{} match std::future::pending::<std::convert::Infallible>().await {{}}",
                resume_parent_cancellation(&names.scope)
            ),
        );
        let runtime_fault_cleanup = abnormal_close(
            &names,
            close_error_ty,
            "RuntimeFault",
            &iter.ty().display_name(),
            "std::panic::resume_unwind(panic_payload);",
        );
        let rendered = format!(
            r#"{{
{receiver_setup}
let {parent} = match __sifr_current_task_cancellation() {{
    Some(carrier) => carrier,
    None => std::future::pending().await,
}};
let {scope} = match ::sifr_runtime::cancellation::CancellationScopeLease::claim(&{parent}) {{
    Ok(scope) => scope,
    Err(_) => std::future::pending().await,
}};
let {child} = {scope}.child().clone();
loop {{
    let {iteration_future} = async {{
        let {next} = {next_expr};
        let Some({target_pattern}) = {next} else {{
            return Ok(None);
        }};
        let {body_outcome}: Result<Result<{normal_outcome_type}, bool>, {active_error_rust}> = async {{
            {body_source}
        }}.await;
        match {body_outcome} {{
            Ok(control) => Ok(Some(control)),
            Err(error) => Err(error),
        }}
    }};
    let {caught_iteration} = ::sifr_runtime::async_cleanup::catch_unwind_future({iteration_future});
    let mut {scoped_iteration} = Box::pin(__SIFR_TASK_CANCELLATION.scope(
        {child}.clone(),
        {caught_iteration},
    ));
    let mut {iteration_cancel} = Box::pin({scope}.notification());
    let {outcome}: Option<Result<Result<Option<Result<{normal_outcome_type}, bool>>, {active_error_rust}>, Box<dyn std::any::Any + Send>>> = tokio::select! {{
        biased;
        _ = &mut {iteration_cancel} => None,
        result = &mut {scoped_iteration} => Some(result),
    }};
    drop({scoped_iteration});
    drop({iteration_cancel});
    match {outcome} {{
        Some(Ok(Ok(None))) => break,
        Some(Ok(Ok(Some(Ok({normal_pattern}))))) => continue,
        {return_arm}
        Some(Ok(Ok(Some(Err(false))))) => {{ {close_primary} break; }},
        Some(Ok(Ok(Some(Err(true))))) => continue,
        Some(Ok(Err({body_error}))) => {{ {error_cleanup} }},
        Some(Err({panic_payload})) => {{ {runtime_fault_cleanup} }},
        None => {{ {cancellation_cleanup} }},
    }}
}}
drop({scope});
}}"#,
            parent = names.parent,
            scope = names.scope,
            child = names.child,
            iteration_future = names.iteration_future,
            next = names.next,
            body_outcome = names.body_outcome,
            caught_iteration = names.caught_iteration,
            scoped_iteration = names.scoped_iteration,
            iteration_cancel = names.iteration_cancel,
            outcome = names.outcome,
            body_error = names.body_error,
            panic_payload = names.panic_payload,
        );
        Ok(Some(RustStmt::Verbatim(rendered)))
    }

    fn native_async_for_return_expression_type(&self, error_type: &str) -> String {
        let function_return = self.current_return_type.as_ref().map_or_else(
            || "()".to_string(),
            |ty| crate::render_type(&crate::sifr_type_to_rust_type(ty)),
        );
        if self.try_closure_depth == 0 {
            return function_return;
        }
        let ok_type = match self.try_closure_return_wrap.last() {
            Some(crate::TryClosureReturnWrap::Optional) => format!("Option<{function_return}>"),
            Some(crate::TryClosureReturnWrap::ControlFlow { continue_type }) => {
                format!("std::ops::ControlFlow<{function_return}, {continue_type}>")
            }
            Some(crate::TryClosureReturnWrap::Direct) | None => function_return,
        };
        format!("Result<{ok_type}, {error_type}>")
    }
}

struct NativeAsyncForNames {
    receiver: String,
    parent: String,
    scope: String,
    child: String,
    iteration_future: String,
    next: String,
    body_outcome: String,
    caught_iteration: String,
    scoped_iteration: String,
    iteration_cancel: String,
    outcome: String,
    body_error: String,
    panic_payload: String,
    cleanup_carrier: String,
    cleanup_result: String,
}

impl NativeAsyncForNames {
    fn new(suffix: usize) -> Self {
        let name = |label: &str| format!("__sifr_native_async_for_{label}_{suffix}");
        Self {
            receiver: name("receiver"),
            parent: name("parent"),
            scope: name("scope"),
            child: name("child"),
            iteration_future: name("iteration_future"),
            next: name("next"),
            body_outcome: name("body_outcome"),
            caught_iteration: name("caught_iteration"),
            scoped_iteration: name("scoped_iteration"),
            iteration_cancel: name("iteration_cancel"),
            outcome: name("outcome"),
            body_error: "body_error".to_string(),
            panic_payload: "panic_payload".to_string(),
            cleanup_carrier: name("cleanup_carrier"),
            cleanup_result: name("cleanup_result"),
        }
    }
}

fn primary_close(receiver: &str, close_error_ty: &Type) -> String {
    if matches!(close_error_ty.resolve_alias(), Type::Never) {
        format!("{receiver}.aclose().await;")
    } else {
        format!("{receiver}.aclose().await?;")
    }
}

fn native_error_cause(error_ty: &Type, error: &str) -> String {
    match error_ty.resolve_alias() {
        Type::Class {
            identity: Some(identity),
            ..
        } if identity == "sifr.builtin.TimeoutError" => "Timeout".to_string(),
        Type::Class {
            identity: Some(identity),
            ..
        } if identity == "sifr.builtin.CancellationError" => "Cancellation".to_string(),
        Type::Class { name, .. } if name == "WorkerRuntimeError" => "RuntimeFault".to_string(),
        _ => {
            let _ = error;
            "OrdinaryError".to_string()
        }
    }
}

fn abnormal_close(
    names: &NativeAsyncForNames,
    close_error_ty: &Type,
    cause: &str,
    resource: &str,
    primary_continuation: &str,
) -> String {
    let resource = format!("{resource:?}");
    let evidence_format = format!("{cause}: {{}}");
    let evidence_format = format!("{evidence_format:?}");
    let call = format!("{}.aclose()", names.receiver);
    let outcome = if matches!(close_error_ty.resolve_alias(), Type::Never) {
        format!(
            r#"match {cleanup_result} {{
    Ok(Ok(())) => {{}},
    Ok(Err(_)) => {parent}.record_async_cleanup_failed(
        "asynchronous cleanup panicked".to_string(),
        file!().to_string(),
        {resource}.to_string(),
        "aclose".to_string(),
        std::time::Duration::from_secs({CLEANUP_BUDGET_SECONDS}),
    ),
    Err(_) => {parent}.record_async_cleanup_timed_out(
        file!().to_string(),
        {resource}.to_string(),
        "aclose".to_string(),
        std::time::Duration::from_secs({CLEANUP_BUDGET_SECONDS}),
    ),
}}"#,
            cleanup_result = names.cleanup_result,
            parent = names.parent,
        )
    } else {
        format!(
            r#"match {cleanup_result} {{
    Ok(Ok(Ok(()))) => {{}},
    Ok(Ok(Err(cleanup_error))) => {parent}.record_async_cleanup_failed(
        format!({evidence_format}, cleanup_error),
        file!().to_string(),
        {resource}.to_string(),
        "aclose".to_string(),
        std::time::Duration::from_secs({CLEANUP_BUDGET_SECONDS}),
    ),
    Ok(Err(_)) => {parent}.record_async_cleanup_failed(
        "asynchronous cleanup panicked".to_string(),
        file!().to_string(),
        {resource}.to_string(),
        "aclose".to_string(),
        std::time::Duration::from_secs({CLEANUP_BUDGET_SECONDS}),
    ),
    Err(_) => {parent}.record_async_cleanup_timed_out(
        file!().to_string(),
        {resource}.to_string(),
        "aclose".to_string(),
        std::time::Duration::from_secs({CLEANUP_BUDGET_SECONDS}),
    ),
}}"#,
            cleanup_result = names.cleanup_result,
            parent = names.parent,
        )
    };
    format!(
        r#"let {cleanup_carrier} = ::sifr_runtime::cancellation::CancellationCarrier::new();
let {cleanup_result} = tokio::time::timeout(
    std::time::Duration::from_secs({CLEANUP_BUDGET_SECONDS}),
    ::sifr_runtime::async_cleanup::catch_unwind_future(
        __SIFR_TASK_CANCELLATION.scope({cleanup_carrier}, {call}),
    ),
).await;
{outcome}
{primary_continuation}"#,
        cleanup_carrier = names.cleanup_carrier,
        cleanup_result = names.cleanup_result,
    )
}

fn resume_parent_cancellation(scope: &str) -> String {
    format!(
        r#"match {scope}.release_and_resume_parent() {{
    ::sifr_runtime::cancellation::CancellationResume::Invoked
    | ::sifr_runtime::cancellation::CancellationResume::AlreadyResumed => {{
        tokio::task::yield_now().await;
    }},
    ::sifr_runtime::cancellation::CancellationResume::NotRequested
    | ::sifr_runtime::cancellation::CancellationResume::ExactClaimActive
    | ::sifr_runtime::cancellation::CancellationResume::FallbackUnavailable
    | ::sifr_runtime::cancellation::CancellationResume::StateUnavailable => {{}},
}}"#
    )
}
