use super::cancellation_scope::cleanup_scope_call;
use super::outcome::{cause_label, cause_variant};
use super::sync::rewrite_context_control_flow;
use crate::python_interop_async::{async_output_value, output_schema};
use crate::python_interop_callbacks::failure_reconciliation_stmt;
use crate::rust_interop_error_mapping::bridge_error_expr;
use crate::{HirExpr, HirStmt, RustEmitter, RustExpr, RustStmt, Type};

impl RustEmitter {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn try_lower_python_async_context_for_ir(
        &mut self,
        context: &HirExpr,
        manager_class: &str,
        entered_type: &Type,
        enter_error_type: &Type,
        exit_error_type: &Type,
        _entered_is_opaque_borrow: bool,
        active_error_type: &Type,
        target: Option<&str>,
        body: &[HirStmt],
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let Some(manager_value) = self.lower_rendered_expr_for_ir(context)? else {
            return Ok(None);
        };
        let Some(schema) = output_schema(entered_type, &self.python_opaque_classes) else {
            return Err(crate::CodegenError::new(
                "Python async context entered value has no transport schema",
            ));
        };

        let suffix = self.python_context_counter;
        self.python_context_counter += 1;
        let context_is_nested = self.python_context_envelope_depth > 0;
        let names = AsyncContextNames::new(suffix);
        let owner_retained_errors = self
            .python_retained_callback_errors
            .get(manager_class)
            .cloned()
            .unwrap_or_default();
        let manager = crate::render_expr(&manager_value);
        let schema = crate::render_expr(&schema);
        let entered_rust_type = crate::render_type(&crate::sifr_type_to_rust_type(entered_type));
        let enter_error_rust_type =
            crate::render_type(&crate::sifr_type_to_rust_type(enter_error_type));
        let active_error_rust_type =
            crate::render_type(&crate::sifr_type_to_rust_type(active_error_type));
        let return_expression_type = self.context_return_expression_type(&active_error_rust_type);
        let target = target.unwrap_or("_");
        let target = if self.mutated_vars.contains(target) {
            format!("mut {target}")
        } else {
            target.to_string()
        };

        let Some(converted) = async_output_value(
            &names.entered_raw,
            entered_type,
            enter_error_type,
            &self.python_opaque_classes,
        ) else {
            return Err(crate::CodegenError::new(
                "Python async context entered value cannot be converted",
            ));
        };
        let converted = crate::render_expr(&converted);
        self.python_context_envelope_depth += 1;
        let lowered_body = self.try_lower_stmt_block_for_ir(body);
        self.python_context_envelope_depth -= 1;
        let mut rewritten = rewrite_context_control_flow(
            lowered_body?.ok_or_else(|| {
                crate::CodegenError::new("Python async context body could not be lowered")
            })?,
            0,
        );
        rewritten.push(RustStmt::Expr(RustExpr::Ident(
            "return Ok(Ok(None));".to_string(),
        )));
        let body = crate::render_stmts(&rewritten);

        let internal_error = mapped_internal_error(active_error_type);
        let enter_error = mapped_result_error(enter_error_type);
        let resume_parent_cancellation = resume_parent_cancellation(&names.scope);
        let owner_observer_setup = owner_observer_setup(&names, &owner_retained_errors);
        let normal_exit = normal_exit(&names, exit_error_type, &owner_retained_errors);
        let conversion_exit = sifr_error_exit(
            &names,
            &names.conversion_error,
            "OrdinaryError",
            &enter_error_rust_type,
            false,
        );
        let active_is_python = active_error_type.is_python_error_contract();
        let active_exit = if active_is_python {
            python_error_exit(&names)
        } else {
            sifr_error_exit(
                &names,
                &names.body_error,
                cause_variant(active_error_type),
                &active_error_rust_type,
                true,
            )
        };
        let loop_arms = loop_control_arms(&normal_exit, !self.loop_else_stack.is_empty());
        let return_arm = if self.try_closure_depth > 0 && context_is_nested {
            format!(
                "Some(Ok(Ok(Some(__sifr_context_return)))) => {{ {normal_exit} return Ok(Ok(Some(__sifr_context_return))); }},"
            )
        } else if self.try_closure_depth > 0 {
            format!(
                "Some(Ok(Ok(Some(__sifr_context_return)))) => {{ {normal_exit} return __sifr_context_return; }},"
            )
        } else {
            format!(
                "Some(Ok(Ok(Some(_)))) => {{ {normal_exit} unreachable!(\"Python async context captured a return in a non-returning try\"); }},"
            )
        };

        let rendered = format!(
            r#"{{
let {manager_name} = {manager};
{owner_observer_setup}
let {parent} = match __sifr_current_task_cancellation() {{
    Some(carrier) => carrier,
    None => return Err({internal_error}),
}};
let {scope} = match sifr_runtime::cancellation::CancellationScopeLease::claim(&{parent}) {{
    Ok(scope) => scope,
    Err(_) => return Err({internal_error}),
}};
let {child} = {scope}.child().clone();
let {entered_raw} = match __SIFR_TASK_CANCELLATION.scope(
    {child}.clone(),
    sifr_runtime::python::submit_async_context_enter(
        &{manager_name}.__sifr_python_object,
        {schema},
        Some(&{child}),
    ),
    ).await {{
    Ok(value) => value,
    Err(error) => {{
        let error = sifr_runtime::python::abandon_callback_owner_after_error_async(
            error,
            &{manager_name}.__sifr_python_callbacks,
        ).await;
        sifr_runtime::python::poison_object({manager_name}.__sifr_python_object);
        match {scope}.release_and_resume_parent() {{
            sifr_runtime::cancellation::CancellationResume::Invoked
            | sifr_runtime::cancellation::CancellationResume::AlreadyResumed => {{
                tokio::task::yield_now().await;
                return Err({internal_error});
            }},
            sifr_runtime::cancellation::CancellationResume::NotRequested => {{
                return Err(({enter_error}).into());
            }},
            sifr_runtime::cancellation::CancellationResume::ExactClaimActive
            | sifr_runtime::cancellation::CancellationResume::FallbackUnavailable
            | sifr_runtime::cancellation::CancellationResume::StateUnavailable => {{
                return Err({internal_error});
            }},
        }}
    }},
}};
let {conversion}: Result<{entered_rust_type}, {enter_error_rust_type}> = (|| {{ Ok({converted}) }})();
let {target} = match {conversion} {{
    Ok(value) => value,
    Err(mut {conversion_error}) => {{
        {conversion_exit}
        return Err({conversion_error}.into());
    }},
}};
let {body_future} = async move {{
    {body}
}};
let mut {scoped_body} = Box::pin(__SIFR_TASK_CANCELLATION.scope({child}.clone(), {body_future}));
let mut {body_cancel} = Box::pin({scope}.notification());
let {outcome}: Option<Result<Result<Option<{return_expression_type}>, bool>, {active_error_rust_type}>> = tokio::select! {{
    biased;
    _ = &mut {body_cancel} => None,
    result = &mut {scoped_body} => Some(result),
}};
match {outcome} {{
    Some(Ok(Ok(None))) => {{ {normal_exit} }},
    {return_arm}
    {loop_arms}
    Some(Err(mut {body_error})) => {{ {active_exit} }},
    None => {{
        let {cleanup_carrier} = sifr_runtime::cancellation::CancellationCarrier::new();
        let {cleanup_result} = __SIFR_TASK_CANCELLATION.scope(
            {cleanup_carrier}.clone(),
            sifr_runtime::python::submit_async_context_exit_with_callbacks(
                {manager_name}.__sifr_python_object,
                sifr_runtime::python::PythonAsyncExitCause::Sifr(
                    sifr_runtime::python::SifrExitCause {{
                        kind: sifr_runtime::python::SifrExitCauseKind::Cancellation,
                        sifr_type: "CancellationError".to_string(),
                        message: "task cancellation".to_string(),
                    }},
                ),
                Some(&{cleanup_carrier}),
                {manager_name}.__sifr_python_callbacks,
            ),
        ).await;
        match {cleanup_result} {{
            Ok(sifr_runtime::python::PythonExitDecision::Suppress) => {{
                sifr_runtime::python::record_context_ignored_suppression(
                    "cancellation:CancellationError",
                );
            }},
            Ok(sifr_runtime::python::PythonExitDecision::Propagate) => {{}},
            Err({cleanup_error}) => sifr_runtime::python::record_context_cleanup_evidence(
                "cancellation:CancellationError",
                &{cleanup_error},
            ),
        }}
        {resume_parent_cancellation}
        return Err({internal_error});
    }},
}}
drop({scope});
}}"#,
            manager_name = names.manager,
            owner_observer_setup = owner_observer_setup,
            parent = names.parent,
            scope = names.scope,
            child = names.child,
            entered_raw = names.entered_raw,
            conversion = names.conversion,
            conversion_error = names.conversion_error,
            body_future = names.body_future,
            scoped_body = names.scoped_body,
            body_cancel = names.body_cancel,
            outcome = names.outcome,
            body_error = names.body_error,
            cleanup_carrier = names.cleanup_carrier,
            cleanup_result = names.cleanup_result,
            cleanup_error = names.cleanup_error,
            resume_parent_cancellation = resume_parent_cancellation,
        );
        Ok(Some(RustStmt::Expr(RustExpr::Ident(rendered))))
    }
}

fn resume_parent_cancellation(scope: &str) -> String {
    format!(
        r#"match {scope}.release_and_resume_parent() {{
    sifr_runtime::cancellation::CancellationResume::Invoked
    | sifr_runtime::cancellation::CancellationResume::AlreadyResumed => {{
        tokio::task::yield_now().await;
    }},
    sifr_runtime::cancellation::CancellationResume::NotRequested
    | sifr_runtime::cancellation::CancellationResume::ExactClaimActive
    | sifr_runtime::cancellation::CancellationResume::FallbackUnavailable
    | sifr_runtime::cancellation::CancellationResume::StateUnavailable => {{}},
}}"#
    )
}

struct AsyncContextNames {
    manager: String,
    parent: String,
    scope: String,
    child: String,
    entered_raw: String,
    conversion: String,
    conversion_error: String,
    body_future: String,
    scoped_body: String,
    body_cancel: String,
    outcome: String,
    body_error: String,
    cleanup_carrier: String,
    cleanup_result: String,
    cleanup_error: String,
}

impl AsyncContextNames {
    fn new(suffix: usize) -> Self {
        let name = |label: &str| format!("__sifr_python_async_context_{label}_{suffix}");
        Self {
            manager: name("manager"),
            parent: name("parent"),
            scope: name("scope"),
            child: name("child"),
            entered_raw: name("entered"),
            conversion: name("conversion"),
            conversion_error: name("conversion_error"),
            body_future: name("body_future"),
            scoped_body: name("scoped_body"),
            body_cancel: name("body_cancel"),
            outcome: name("outcome"),
            body_error: name("body_error"),
            cleanup_carrier: name("cleanup_carrier"),
            cleanup_result: name("cleanup_result"),
            cleanup_error: name("cleanup_error"),
        }
    }
}

fn owner_observer_setup(names: &AsyncContextNames, errors: &[Type]) -> String {
    if errors.is_empty() {
        return String::new();
    }
    let owner = format!("{}_callback_owner", names.manager);
    let mut statements = vec![RustStmt::Let {
        mutable: false,
        name: owner,
        ty: None,
        value: RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Field {
                expr: Box::new(RustExpr::Ident(names.manager.clone())),
                field: "__sifr_python_callbacks".to_string(),
            }),
            method: "owner".to_string(),
            args: Vec::new(),
        },
    }];
    for index in 0..errors.len() {
        statements.push(RustStmt::Let {
            mutable: false,
            name: format!("{}_callback_failure_{index}", names.manager),
            ty: None,
            value: RustExpr::Clone(Box::new(RustExpr::Field {
                expr: Box::new(RustExpr::Ident(names.manager.clone())),
                field: format!("__sifr_python_callback_failure_{index}"),
            })),
        });
    }
    crate::render_stmts(&statements)
}

fn normal_exit(
    names: &AsyncContextNames,
    exit_error_type: &Type,
    owner_retained_errors: &[Type],
) -> String {
    let exit = RustExpr::Await(Box::new(RustExpr::Ident(cleanup_scope_call(
        &names.cleanup_carrier,
        &names.manager,
        "sifr_runtime::python::PythonAsyncExitCause::Normal",
    ))));
    let mapped = crate::python_interop_direct::mapped_try(exit, exit_error_type);
    let reconciliation = if owner_retained_errors.is_empty() {
        String::new()
    } else {
        let owner_value = format!("{}_callback_owner_value", names.manager);
        crate::render_stmts(&[RustStmt::IfLet {
            pattern: format!("Some(ref {owner_value})"),
            expr: RustExpr::Ident(format!("{}_callback_owner", names.manager)),
            then_body: owner_retained_errors
                .iter()
                .enumerate()
                .map(|(index, handler_error_type)| {
                    failure_reconciliation_stmt(
                        &format!("{}_callback_failure_{index}", names.manager),
                        handler_error_type,
                        exit_error_type,
                        RustExpr::Deref(Box::new(RustExpr::Ident(owner_value.clone()))),
                    )
                })
                .collect(),
            else_body: None,
        }])
    };
    format!(
        "let {cleanup} = sifr_runtime::cancellation::CancellationCarrier::new(); let _decision = {}; {reconciliation}",
        crate::render_expr(&mapped),
        cleanup = names.cleanup_carrier,
    )
}

fn python_error_exit(names: &AsyncContextNames) -> String {
    format!(
        r#"let {cleanup_carrier} = sifr_runtime::cancellation::CancellationCarrier::new();
let {cause} = match {body_error}.__sifr_python_error.as_ref() {{
    Some(replay) => sifr_runtime::python::PythonAsyncExitCause::Python(replay.clone()),
    None => sifr_runtime::python::PythonAsyncExitCause::Sifr(sifr_runtime::python::SifrExitCause {{
        kind: sifr_runtime::python::SifrExitCauseKind::OrdinaryError,
        sifr_type: "PythonError".to_string(),
        message: format!("{{}}", {body_error}),
    }}),
}};
match __SIFR_TASK_CANCELLATION.scope(
    {cleanup_carrier}.clone(),
    sifr_runtime::python::submit_async_context_exit_with_callbacks(
        {manager}.__sifr_python_object,
        {cause},
        Some(&{cleanup_carrier}),
        {manager}.__sifr_python_callbacks,
    ),
).await {{
    Ok(sifr_runtime::python::PythonExitDecision::Suppress) => {{}},
    Ok(sifr_runtime::python::PythonExitDecision::Propagate) => return Err({body_error}),
    Err({cleanup_error}) => {{
        if let Some(primary) = {body_error}.__sifr_python_error.as_mut() {{
            sifr_runtime::python::attach_secondary_python_error(primary, &{cleanup_error});
            {body_error}.context = primary.context.to_string();
        }} else {{
            sifr_runtime::python::record_context_cleanup_evidence("ordinary-error:PythonError", &{cleanup_error});
        }}
        return Err({body_error});
    }},
}}"#,
        cleanup_carrier = names.cleanup_carrier,
        cause = format!("__sifr_python_async_context_cause_{}", names.manager),
        body_error = names.body_error,
        manager = names.manager,
        cleanup_error = names.cleanup_error,
    )
}

fn sifr_error_exit(
    names: &AsyncContextNames,
    error: &str,
    cause_kind: &str,
    error_type: &str,
    return_primary: bool,
) -> String {
    let primary_return = if return_primary {
        format!("return Err({error});")
    } else {
        String::new()
    };
    format!(
        r#"let {cleanup_carrier} = sifr_runtime::cancellation::CancellationCarrier::new();
let {cleanup_result} = __SIFR_TASK_CANCELLATION.scope(
    {cleanup_carrier}.clone(),
    sifr_runtime::python::submit_async_context_exit_with_callbacks(
        {manager}.__sifr_python_object,
        sifr_runtime::python::PythonAsyncExitCause::Sifr(sifr_runtime::python::SifrExitCause {{
            kind: sifr_runtime::python::SifrExitCauseKind::{cause_kind},
            sifr_type: "{error_type}".to_string(),
            message: format!("{{}}", {error}),
        }}),
        Some(&{cleanup_carrier}),
        {manager}.__sifr_python_callbacks,
    ),
).await;
match {cleanup_result} {{
    Ok(sifr_runtime::python::PythonExitDecision::Suppress) => {{
        sifr_runtime::python::record_context_ignored_suppression("{cause_label}:{error_type}");
    }},
    Ok(sifr_runtime::python::PythonExitDecision::Propagate) => {{}},
    Err({cleanup_error}) => sifr_runtime::python::record_context_cleanup_evidence(
        "{cause_label}:{error_type}",
        &{cleanup_error},
    ),
}}
{primary_return}"#,
        cleanup_carrier = names.cleanup_carrier,
        cleanup_result = names.cleanup_result,
        manager = names.manager,
        cleanup_error = names.cleanup_error,
        cause_label = cause_label(cause_kind),
    )
}

fn mapped_internal_error(active_error_type: &Type) -> String {
    let runtime = RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "sifr_runtime".to_string(),
            "python".to_string(),
            "PythonError".to_string(),
            "without_replay".to_string(),
        ])),
        args: vec![
            RustExpr::Ident("\"runtime\"".to_string()),
            RustExpr::Ident("\"SifrPythonAsyncContextError\"".to_string()),
            RustExpr::Ident("\"async context cancellation handoff failed\"".to_string()),
            RustExpr::Ident("String::new()".to_string()),
            RustExpr::Ident("\"async context\"".to_string()),
        ],
    };
    if matches!(
        active_error_type.resolve_alias(),
        Type::Class { name, .. } if name == "Error"
    ) {
        return crate::render_expr(&RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Error".to_string(), "new".to_string()])),
            args: vec![RustExpr::MethodCall {
                receiver: Box::new(runtime),
                method: "to_string".to_string(),
                args: vec![],
            }],
        });
    }
    crate::render_expr(&bridge_error_expr(runtime, active_error_type))
}

fn mapped_result_error(error_type: &Type) -> String {
    crate::render_expr(&bridge_error_expr(
        RustExpr::Ident("error".to_string()),
        error_type,
    ))
}

fn loop_control_arms(normal_exit: &str, can_loop: bool) -> String {
    if can_loop {
        format!(
            "Some(Ok(Err(false))) => {{ {normal_exit} break; }}, Some(Ok(Err(true))) => {{ {normal_exit} continue; }},"
        )
    } else {
        format!(
            "Some(Ok(Err(_))) => {{ {normal_exit} unreachable!(\"Python async context emitted loop control outside a loop\"); }},"
        )
    }
}
