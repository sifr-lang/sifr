use super::{
    ExprCall, HirExpr, LowerCtx, ipc_schema_extraction, ownership_diagnostics, task_scope_calls,
};
use ruff_text_size::{Ranged, TextRange};
use sifr_type_system::Type;
use std::collections::HashSet;

pub(in crate::lower) fn validate_require_serializable_call(
    func_name: &str,
    args: &[HirExpr],
    arg_ranges: &[Option<TextRange>],
    call: &ExprCall,
    ctx: &mut LowerCtx,
) {
    if func_name != "require_serializable" {
        return;
    }
    let Some(arg) = args.first() else {
        return;
    };
    let Some(reason) = non_ipc_serializable_reason(arg.ty()) else {
        let _schema = ipc_schema_extraction::extract_ipc_schema_type(arg.ty());
        return;
    };
    ownership_diagnostics::non_ipc_serializable_payload(
        ctx,
        &payload_arg_label(arg),
        &arg.ty().display_name(),
        &reason,
        arg_ranges
            .first()
            .copied()
            .flatten()
            .unwrap_or_else(|| call.range()),
    );
}

fn payload_arg_label(expr: &HirExpr) -> String {
    match expr {
        HirExpr::Name { name, .. } => name.clone(),
        HirExpr::FieldAccess { field, .. } => format!("field `{field}`"),
        _ => "value".to_string(),
    }
}

pub(in crate::lower) fn non_ipc_serializable_reason(ty: &Type) -> Option<String> {
    non_ipc_serializable_reason_inner(ty.resolve_alias(), &mut HashSet::new())
}

fn non_ipc_serializable_reason_inner(ty: &Type, visiting: &mut HashSet<String>) -> Option<String> {
    match ty {
        Type::Bool
        | Type::Int
        | Type::FixedInt(_)
        | Type::Float
        | Type::Str
        | Type::Bytes
        | Type::None
        | Type::Never
        | Type::LiteralInt(_)
        | Type::LiteralStr(_)
        | Type::LiteralBool(_) => None,
        Type::List(elem) => non_ipc_serializable_reason_inner(elem.resolve_alias(), visiting),
        Type::Dict(key, value) => {
            if !matches!(key.resolve_alias(), Type::Str) {
                return Some("dict IPC payload keys must be str".to_string());
            }
            non_ipc_serializable_reason_inner(value.resolve_alias(), visiting)
        }
        Type::Result(ok, err) => non_ipc_serializable_reason_inner(ok.resolve_alias(), visiting)
            .or_else(|| non_ipc_serializable_reason_inner(err.resolve_alias(), visiting)),
        Type::Tuple(elems) => elems
            .iter()
            .find_map(|elem| non_ipc_serializable_reason_inner(elem.resolve_alias(), visiting)),
        Type::StructuralRecord(record) => record.fields().iter().find_map(|field| {
            non_ipc_serializable_reason_inner(field.ty().resolve_alias(), visiting)
                .map(|reason| format!("field `{}` is not IPC-serializable: {reason}", field.name()))
        }),
        Type::Union(_) => {
            if let Some(non_none) = ty.optional_member_type() {
                non_ipc_serializable_reason_inner(non_none.resolve_alias(), visiting)
            } else {
                Some(
                    "IPC payload unions are limited to Option[T] in the current IPC payload rules"
                        .to_string(),
                )
            }
        }
        Type::Alias { body, .. } => {
            non_ipc_serializable_reason_inner(body.resolve_alias(), visiting)
        }
        Type::Newtype { inner, .. } => {
            non_ipc_serializable_reason_inner(inner.resolve_alias(), visiting)
        }
        Type::Class {
            name,
            fields,
            parent_class,
            ..
        } => {
            if let Some(label) = ipc_local_resource_type_label(name) {
                return Some(format!("`{name}` is a process-local {label}"));
            }
            if let Some(reason) = task_scope_calls::non_send_reason(ty) {
                return Some(reason);
            }
            if class_has_non_send_marker(name, parent_class.as_deref()) {
                return Some(format!("`{name}` inherits the `NonSend` marker"));
            }
            if !visiting.insert(name.clone()) {
                return None;
            }
            let found = fields.iter().find_map(|(field, field_ty)| {
                non_ipc_serializable_reason_inner(field_ty.resolve_alias(), visiting)
                    .map(|reason| format!("field `{field}` is not IPC-serializable: {reason}"))
            });
            visiting.remove(name);
            found
        }
        // Sifr enums currently carry integer-backed variants, not typed payload fields.
        Type::Enum { .. } => None,
        Type::Set(_) => Some("set payloads do not have stable IPC schema ordering".to_string()),
        Type::PythonBuffer(_) => {
            Some("Python buffer resources are affine and process-local".to_string())
        }
        Type::PythonArrow(_) => {
            Some("Python Arrow resources are affine and process-local".to_string())
        }
        Type::PythonDlpackTensor(_) | Type::PythonDlpackStream => {
            Some("Python DLPack resources are affine and process-local".to_string())
        }
        Type::Decimal | Type::BigDecimal => {
            Some("this numeric family is not part of the initial IPC schema set".to_string())
        }
        Type::Task(_, _)
        | Type::TaskResult(_, _)
        | Type::BlockingTask(_, _)
        | Type::JoinSet(_, _)
        | Type::Coroutine(_, _)
        | Type::Awaitable(_)
        | Type::AsyncIterator(_, _)
        | Type::AsyncGenerator(_, _)
        | Type::Failure(_)
        | Type::TimeoutResult(_)
        | Type::Select2(_, _) => {
            Some("task and async runtime handles are process-local".to_string())
        }
        Type::Function(_)
        | Type::AsyncFunction(_)
        | Type::Callable(..)
        | Type::AsyncCallable(..) => {
            Some("callables are process-local code, not IPC payload data".to_string())
        }
        Type::Iterator(_) | Type::Iterable(_) | Type::Range => {
            Some("iterators and ranges are not stable IPC payload schemas".to_string())
        }
        Type::Any | Type::Unknown | Type::TypeVar(_) => {
            Some("IPC payload type must be statically known".to_string())
        }
        Type::Protocol { .. } | Type::Intersection(_) => {
            Some("structural protocol payloads need generated concrete schemas".to_string())
        }
        Type::Template(_) => {
            Some("template values are compiler-owned processor inputs".to_string())
        }
    }
}

fn class_has_non_send_marker(name: &str, parent_chain: Option<&str>) -> bool {
    name == "NonSend"
        || parent_chain.is_some_and(|parents| parents.split('|').any(|parent| parent == "NonSend"))
}

fn ipc_local_resource_type_label(name: &str) -> Option<&'static str> {
    match name {
        "Child" | "AsyncChild" | "ProcessHandle" => Some("process handle"),
        "PipeReader" | "PipeWriter" | "AsyncPipeReader" | "AsyncPipeWriter" => {
            Some("process pipe handle")
        }
        "Lock" | "RwLock" | "Semaphore" | "Notify" | "Shared" => Some("synchronization wrapper"),
        "LockGuard" | "RwLockReadGuard" | "RwLockWriteGuard" | "SemaphorePermit" => {
            Some("synchronization guard")
        }
        "Channel" | "ChannelSender" | "ChannelReceiver" => Some("channel endpoint"),
        "Context" | "ContextKey" => Some("task context value"),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../../tests/unit/lower/ipc_payload_calls/tests.rs"]
mod tests;
