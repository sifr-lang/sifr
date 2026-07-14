use super::{callbacks, decorator_path, parameter_metadata, LowerCtx};
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{HirExpr, PythonCallbackConcurrency, PythonCallbackDispatch, PythonCallbackLifetime};
use sifr_python_ast::{Decorator, Expr, ExprCall, Parameters};

#[derive(Clone, Copy, Debug)]
pub(in crate::lower) struct CallbackCallPolicy {
    pub(in crate::lower) parameter_index: usize,
    pub(in crate::lower) lifetime: PythonCallbackLifetime,
    pub(in crate::lower) dispatch: PythonCallbackDispatch,
    pub(in crate::lower) concurrency: Option<PythonCallbackConcurrency>,
}

pub(in crate::lower) fn callback_call_policies(
    decorators: &[Decorator],
    parameters: &Parameters,
    has_receiver: bool,
) -> Vec<CallbackCallPolicy> {
    let metadata = parameter_metadata(parameters);
    decorators
        .iter()
        .filter_map(|decorator| {
            let Expr::Call(call) = &decorator.expression else {
                return None;
            };
            if decorator_path(&call.func)
                .is_none_or(|path| path.as_slice() != ["python", "callback"])
            {
                return None;
            }
            if call.arguments.args.len() != 1 {
                return None;
            }
            let parameter_path = decorator_path(&call.arguments.args[0])?;
            let [parameter_name] = parameter_path.as_slice() else {
                return None;
            };
            let raw_index = metadata
                .iter()
                .position(|candidate| candidate.name == *parameter_name)?;
            let parameter_index = raw_index.checked_sub(usize::from(has_receiver))?;
            let mut lifetime = None;
            let mut dispatch = None;
            let mut concurrency = None;
            for keyword in &call.arguments.keywords {
                let name = keyword.arg.as_ref()?.as_str();
                let value = callbacks::policy_atom(&keyword.value)?;
                match (name, value.as_str()) {
                    ("lifetime", "call") => lifetime = Some(PythonCallbackLifetime::Call),
                    ("lifetime", "result") => lifetime = Some(PythonCallbackLifetime::Result),
                    ("lifetime", "Self") => lifetime = Some(PythonCallbackLifetime::Receiver),
                    ("dispatch", "current") => dispatch = Some(PythonCallbackDispatch::Current),
                    ("dispatch", "foreign") => dispatch = Some(PythonCallbackDispatch::Foreign),
                    ("dispatch", "asyncio") => dispatch = Some(PythonCallbackDispatch::Asyncio),
                    ("concurrency", "serial") => {
                        concurrency = Some(PythonCallbackConcurrency::Serial);
                    }
                    ("concurrency", "parallel") => {
                        concurrency = Some(PythonCallbackConcurrency::Parallel);
                    }
                    _ => {}
                }
            }
            Some(CallbackCallPolicy {
                parameter_index,
                lifetime: lifetime?,
                dispatch: dispatch?,
                concurrency,
            })
        })
        .collect()
}

pub(in crate::lower) fn validate_callback_call_captures(
    callable: &str,
    args: &[HirExpr],
    argument_ranges: &[Option<TextRange>],
    receiver_name: Option<&str>,
    fallback_range: TextRange,
    ctx: &mut LowerCtx,
) {
    let Some(policies) = ctx.python_callback_call_policies.get(callable).cloned() else {
        return;
    };
    for policy in policies {
        if policy.dispatch == PythonCallbackDispatch::Current {
            continue;
        }
        let Some(HirExpr::Name { name, .. }) = args.get(policy.parameter_index) else {
            reject_unverifiable_handler(
                policy.parameter_index,
                argument_ranges,
                fallback_range,
                ctx,
            );
            continue;
        };
        let range = argument_range(policy.parameter_index, argument_ranges, fallback_range);
        let Some(captures) = ctx.nested_function_captures.get(name).cloned() else {
            if !ctx.scope.resolves_to_module_binding(name) {
                reject_unproven_named_handler(name, range, ctx);
            }
            continue;
        };
        for (capture_name, capture_ty) in captures {
            let reason =
                capture_rejection_reason(policy, &capture_name, &capture_ty, receiver_name, ctx);
            if let Some(reason) = reason {
                ctx.error_with_code_at(
                    DiagnosticCode::PYCB_INVALID_DECLARATION,
                    format!(
                        "invalid Python callback attachment: handler `{name}` capture `{capture_name}` of type `{}` {reason}",
                        capture_ty.display_name()
                    ),
                    range,
                );
            }
        }
    }
}

fn reject_unproven_named_handler(name: &str, range: TextRange, ctx: &mut LowerCtx) {
    ctx.error_with_code_at(
        DiagnosticCode::PYCB_INVALID_DECLARATION,
        format!(
            "invalid Python callback attachment: handler `{name}` is a callable value whose captures cannot be proven safe; use a top-level function or a directly declared nested function"
        ),
        range,
    );
}

pub(in crate::lower) fn callback_method_arg_ranges(
    object: &HirExpr,
    object_ty: &sifr_type_system::Type,
    method_name: &str,
    call: &ExprCall,
    args: &[HirExpr],
    ctx: &mut LowerCtx,
) -> Vec<TextRange> {
    let ranges =
        crate::lower::method_call_args::resolved_method_arg_ranges(object_ty, method_name, call);
    if let sifr_type_system::Type::Class { name, .. } = object_ty.resolve_alias() {
        let receiver_name = match object {
            HirExpr::Name { name, .. } => Some(name.as_str()),
            _ => None,
        };
        validate_callback_call_captures(
            &format!("{name}.{method_name}"),
            args,
            &ranges.iter().copied().map(Some).collect::<Vec<_>>(),
            receiver_name,
            call.range,
            ctx,
        );
    }
    ranges
}

fn capture_rejection_reason(
    policy: CallbackCallPolicy,
    capture_name: &str,
    capture_ty: &sifr_type_system::Type,
    receiver_name: Option<&str>,
    ctx: &LowerCtx,
) -> Option<String> {
    (policy.lifetime == PythonCallbackLifetime::Receiver && receiver_name == Some(capture_name))
        .then_some(
            "captures its retained owner; same-owner close from a callback cannot be proven absent"
                .to_string(),
        )
        .or_else(|| {
            (policy.dispatch == PythonCallbackDispatch::Foreign
                && callbacks::contains_python_identity(capture_ty, ctx))
            .then_some("contains Python identity".to_string())
        })
        .or_else(|| {
            crate::lower::task_scope_calls::non_send_reason(capture_ty)
                .map(|reason| format!("is not sendable: {reason}"))
        })
        .or_else(|| {
            (policy.concurrency == Some(PythonCallbackConcurrency::Parallel))
                .then(|| crate::lower::task_scope_calls::non_share_safe_reason(capture_ty))
                .flatten()
                .map(|reason| format!("is not share-safe: {reason}"))
        })
}

fn reject_unverifiable_handler(
    parameter_index: usize,
    argument_ranges: &[Option<TextRange>],
    fallback_range: TextRange,
    ctx: &mut LowerCtx,
) {
    ctx.error_with_code_at(
        DiagnosticCode::PYCB_INVALID_DECLARATION,
        "invalid Python callback attachment: foreign and asyncio handlers must be a named function so captures can be proven safe"
            .to_string(),
        argument_range(parameter_index, argument_ranges, fallback_range),
    );
}

fn argument_range(
    parameter_index: usize,
    argument_ranges: &[Option<TextRange>],
    fallback_range: TextRange,
) -> TextRange {
    argument_ranges
        .get(parameter_index)
        .copied()
        .flatten()
        .unwrap_or(fallback_range)
}
