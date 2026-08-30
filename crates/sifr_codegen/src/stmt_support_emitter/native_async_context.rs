use super::python_context::{rewrite_context_control_flow, rust_stmts_always_exit};
use crate::{HirExpr, HirStmt, RustEmitter, RustExpr, RustStmt, Type};

const CLEANUP_BUDGET_SECONDS: u64 = 5;

impl RustEmitter {
    pub(crate) fn try_lower_native_async_context_for_ir(
        &mut self,
        context: &HirExpr,
        exit_error_ty: &Type,
        declared_active_error_ty: &Type,
        target: Option<&str>,
        body: &[HirStmt],
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let suffix = self.python_context_counter;
        self.python_context_counter += 1;
        let mut names = NativeAsyncContextNames::new(suffix);
        let context_is_nested = self.python_context_envelope_depth > 0;
        let manager_setup = if let HirExpr::Name { name, .. } = context {
            names.manager.clone_from(name);
            String::new()
        } else {
            let Some(value) = self.lower_rendered_expr_for_ir(context)? else {
                return Ok(None);
            };
            format!(
                "let mut {} = {};",
                names.manager,
                crate::render_expr(&value)
            )
        };

        self.python_context_envelope_depth += 1;
        let lowered_body = self.try_lower_scoped_stmt_block_for_ir(body);
        self.python_context_envelope_depth -= 1;
        let mut rewritten = rewrite_context_control_flow(
            lowered_body?.ok_or_else(|| {
                crate::CodegenError::new("native async context body could not be lowered")
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
        let return_expression_type = self.native_context_return_expression_type(&active_error_rust);
        let body_always_exits = rust_stmts_always_exit(&rewritten);
        let normal_outcome_type = if can_return {
            format!("Option<{return_expression_type}>")
        } else if body_always_exits {
            "std::convert::Infallible".to_string()
        } else {
            "()".to_string()
        };
        if !body_always_exits {
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
        let can_loop = !self.loop_else_stack.is_empty();
        let loop_control_type = if can_loop {
            "bool"
        } else {
            "std::convert::Infallible"
        };
        let normal_exit = native_exit_call(&names.manager, "AsyncExitCause::Normal");
        let return_exit = native_exit_call(&names.manager, "AsyncExitCause::Return");
        let return_arm = if !can_return {
            String::new()
        } else if context_is_nested {
            format!(
                "Some(Ok(Ok(Ok(Some(value))))) => {{ {return_exit} return Ok(Ok(Some(value))); }},"
            )
        } else {
            format!("Some(Ok(Ok(Ok(Some(value))))) => {{ {return_exit} return value; }},")
        };
        let loop_arms = if can_loop && context_is_nested {
            format!(
                "Some(Ok(Ok(Err(false)))) => {{ {normal_exit} return Ok(Err(false)); }}, Some(Ok(Ok(Err(true)))) => {{ {normal_exit} return Ok(Err(true)); }},"
            )
        } else if can_loop {
            format!(
                "Some(Ok(Ok(Err(false)))) => {{ {normal_exit} break; }}, Some(Ok(Ok(Err(true)))) => {{ {normal_exit} continue; }},"
            )
        } else {
            format!("Some(Ok(Ok(Err(control)))) => {{ {normal_exit} match control {{}} }},")
        };
        let error_cause = native_error_cause(active_error_ty, &names.body_error);
        let error_cleanup = abnormal_exit(
            &names,
            &error_cause,
            &context.ty().display_name(),
            "return Err(body_error);",
        );
        let cancellation_cleanup = abnormal_exit(
            &names,
            "AsyncExitCause::Cancellation",
            &context.ty().display_name(),
            &format!(
                "{} match std::future::pending::<std::convert::Infallible>().await {{}}",
                resume_parent_cancellation(&names.scope)
            ),
        );
        let runtime_fault_cleanup = abnormal_exit(
            &names,
            "AsyncExitCause::RuntimeFault(\"runtime panic\".to_string())",
            &context.ty().display_name(),
            "std::panic::resume_unwind(panic_payload);",
        );
        let entered_target = target.unwrap_or("_");
        let normal_arm = if body_always_exits {
            if can_return {
                "Some(Ok(Ok(Ok(None)))) => match std::future::pending::<std::convert::Infallible>().await {},".to_string()
            } else {
                "Some(Ok(Ok(Ok(normal)))) => match normal {},".to_string()
            }
        } else {
            let normal_pattern = if can_return { "None" } else { "()" };
            format!("Some(Ok(Ok(Ok({normal_pattern})))) => {{ {normal_exit} }},")
        };
        let rendered = format!(
            r#"{{
{manager_setup}
let {parent} = match __sifr_current_task_cancellation() {{
    Some(carrier) => carrier,
    None => std::future::pending().await,
}};
let {scope} = match ::sifr_runtime::cancellation::CancellationScopeLease::claim(&{parent}) {{
    Ok(scope) => scope,
    Err(_) => std::future::pending().await,
}};
let {child} = {scope}.child().clone();
let mut {enter_future} = Box::pin(__SIFR_TASK_CANCELLATION.scope(
    {child}.clone(),
    {manager}.__aenter__(),
));
let mut {enter_cancel} = Box::pin({scope}.notification());
let {enter_outcome} = tokio::select! {{
    biased;
    result = &mut {enter_future} => Some(result),
    _ = &mut {enter_cancel} => None,
}};
drop({enter_future});
drop({enter_cancel});
let {entered_target} = match {enter_outcome} {{
    Some(result) => result?,
    None => {{
        {enter_cancel_resume}
        match std::future::pending::<std::convert::Infallible>().await {{}}
    }},
}};
let {body_future} = async {{
    {body_source}
}};
let {caught_body} = ::sifr_runtime::async_cleanup::catch_unwind_future({body_future});
let mut {scoped_body} = Box::pin(__SIFR_TASK_CANCELLATION.scope({child}, {caught_body}));
let mut {body_cancel} = Box::pin({scope}.notification());
let {outcome}: Option<Result<Result<Result<{normal_outcome_type}, {loop_control_type}>, {active_error_rust}>, Box<dyn std::any::Any + Send>>> = tokio::select! {{
    biased;
    _ = &mut {body_cancel} => None,
    result = &mut {scoped_body} => Some(result),
}};
drop({scoped_body});
drop({body_cancel});
match {outcome} {{
    {normal_arm}
    {return_arm}
    {loop_arms}
    Some(Ok(Err({body_error}))) => {{ {error_cleanup} }},
    Some(Err({panic_payload})) => {{ {runtime_fault_cleanup} }},
    None => {{ {cancellation_cleanup} }},
}}
drop({scope});
}}"#,
            parent = names.parent,
            scope = names.scope,
            child = names.child,
            manager = names.manager,
            enter_future = names.enter_future,
            enter_cancel = names.enter_cancel,
            enter_outcome = names.enter_outcome,
            enter_cancel_resume = resume_parent_cancellation(&names.scope),
            body_future = names.body_future,
            caught_body = names.caught_body,
            scoped_body = names.scoped_body,
            body_cancel = names.body_cancel,
            outcome = names.outcome,
            body_error = names.body_error,
            panic_payload = names.panic_payload,
        );
        let _ = exit_error_ty;
        Ok(Some(RustStmt::Verbatim(rendered)))
    }

    fn native_context_return_expression_type(&self, error_type: &str) -> String {
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

struct NativeAsyncContextNames {
    manager: String,
    parent: String,
    scope: String,
    child: String,
    body_future: String,
    enter_future: String,
    enter_cancel: String,
    enter_outcome: String,
    caught_body: String,
    scoped_body: String,
    body_cancel: String,
    outcome: String,
    body_error: String,
    panic_payload: String,
    cleanup_carrier: String,
    cleanup_result: String,
}

impl NativeAsyncContextNames {
    fn new(suffix: usize) -> Self {
        let name = |label: &str| format!("__sifr_native_async_context_{label}_{suffix}");
        Self {
            manager: name("manager"),
            parent: name("parent"),
            scope: name("scope"),
            child: name("child"),
            body_future: name("body_future"),
            enter_future: name("enter_future"),
            enter_cancel: name("enter_cancel"),
            enter_outcome: name("enter_outcome"),
            caught_body: name("caught_body"),
            scoped_body: name("scoped_body"),
            body_cancel: name("body_cancel"),
            outcome: name("outcome"),
            body_error: "body_error".to_string(),
            panic_payload: "panic_payload".to_string(),
            cleanup_carrier: name("cleanup_carrier"),
            cleanup_result: name("cleanup_result"),
        }
    }
}

fn native_exit_call(manager: &str, cause: &str) -> String {
    format!("{manager}.__aexit__(&{cause}).await?;")
}

fn native_error_cause(error_ty: &Type, error: &str) -> String {
    match error_ty.resolve_alias() {
        union @ Type::Union(members) => {
            let arms = members
                .iter()
                .map(|member| {
                    let variant = member.union_variant_name();
                    let cause = native_error_cause(member, "variant_error");
                    format!(
                        "{}::{variant}(variant_error) => {cause}",
                        union.union_enum_name()
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("match &{error} {{ {arms} }}")
        }
        Type::Class {
            identity: Some(identity),
            ..
        } if identity == "sifr.builtin.TimeoutError" => "AsyncExitCause::Timeout".to_string(),
        Type::Class { name, .. } if name == "TimeoutError" => "AsyncExitCause::Timeout".to_string(),
        Type::Class {
            identity: Some(identity),
            ..
        } if identity == "sifr.builtin.CancellationError" => {
            "AsyncExitCause::Cancellation".to_string()
        }
        Type::Class { name, .. } if name == "WorkerRuntimeError" => {
            format!("AsyncExitCause::RuntimeFault(format!(\"{{}}\", {error}))")
        }
        _ => format!("AsyncExitCause::OrdinaryError(format!(\"{{}}\", {error}))"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error_type(name: &str, identity: &str) -> Type {
        Type::Class {
            identity: Some(identity.to_string()),
            type_args: Vec::new(),
            name: name.to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: Vec::new(),
            parent_class: Some("Error".to_string()),
        }
    }

    #[test]
    fn union_body_error_is_classified_before_conversion() {
        let timeout = error_type("TimeoutError", "sifr.builtin.TimeoutError");
        let ordinary = error_type("ResourceError", "project.ResourceError");
        let union = Type::Union(vec![timeout.clone(), ordinary.clone()]);

        let cause = native_error_cause(&union, "body_error");

        assert!(cause.contains(&format!(
            "{}::{}(variant_error) => AsyncExitCause::Timeout",
            union.union_enum_name(),
            timeout.union_variant_name()
        )));
        assert!(cause.contains(&format!(
            "{}::{}(variant_error) => AsyncExitCause::OrdinaryError",
            union.union_enum_name(),
            ordinary.union_variant_name()
        )));
    }
}

fn abnormal_exit(
    names: &NativeAsyncContextNames,
    cause: &str,
    resource: &str,
    primary_continuation: &str,
) -> String {
    let resource = format!("{resource:?}");
    format!(
        r#"let {cleanup_carrier} = ::sifr_runtime::cancellation::CancellationCarrier::new();
let {cleanup_result} = tokio::time::timeout(
    std::time::Duration::from_secs({CLEANUP_BUDGET_SECONDS}),
    ::sifr_runtime::async_cleanup::catch_unwind_future(
        __SIFR_TASK_CANCELLATION.scope(
            {cleanup_carrier},
            {manager}.__aexit__(&{cause}),
        ),
    ),
).await;
match {cleanup_result} {{
    Ok(Ok(Ok(()))) => {{}},
    Ok(Ok(Err(cleanup_error))) => {{
        {parent}.record_async_cleanup_failed(
            format!("{{}}", cleanup_error),
            file!().to_string(),
            {resource}.to_string(),
            "__aexit__".to_string(),
            std::time::Duration::from_secs({CLEANUP_BUDGET_SECONDS}),
        );
    }},
    Ok(Err(_)) => {{
        {parent}.record_async_cleanup_failed(
            "asynchronous cleanup panicked".to_string(),
            file!().to_string(),
            {resource}.to_string(),
            "__aexit__".to_string(),
            std::time::Duration::from_secs({CLEANUP_BUDGET_SECONDS}),
        );
    }},
    Err(_) => {{
        {parent}.record_async_cleanup_timed_out(
            file!().to_string(),
            {resource}.to_string(),
            "__aexit__".to_string(),
            std::time::Duration::from_secs({CLEANUP_BUDGET_SECONDS}),
        );
    }},
}}
{primary_continuation}"#,
        cleanup_carrier = names.cleanup_carrier,
        cleanup_result = names.cleanup_result,
        manager = names.manager,
        parent = names.parent,
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
