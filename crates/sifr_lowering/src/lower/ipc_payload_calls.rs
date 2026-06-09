use super::{
    ipc_schema_extraction, ownership_diagnostics, task_scope_calls, ExprCall, HirExpr, LowerCtx,
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
        Type::Union(members) => {
            if let Some(non_none) = option_payload_member(members) {
                non_ipc_serializable_reason_inner(non_none.resolve_alias(), visiting)
            } else {
                Some("IPC payload unions are limited to Option[T] in this milestone".to_string())
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
                return Some(format!(
                    "`{}` is a process-local {label}",
                    task_scope_calls::public_type_name(name)
                ));
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
        Type::BigInt | Type::Decimal | Type::BigDecimal => {
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
        Type::Function(_) | Type::AsyncFunction(_) | Type::Callable(..) => {
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
    }
}

fn option_payload_member(members: &[Type]) -> Option<&Type> {
    if members.len() != 2 || !members.iter().any(|member| matches!(member, Type::None)) {
        return None;
    }
    members.iter().find(|member| !matches!(member, Type::None))
}

fn class_has_non_send_marker(name: &str, parent_chain: Option<&str>) -> bool {
    name == "NonSend"
        || parent_chain.is_some_and(|parents| parents.split('|').any(|parent| parent == "NonSend"))
}

fn ipc_local_resource_type_label(name: &str) -> Option<&'static str> {
    match task_scope_calls::public_type_name(name) {
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
mod tests {
    use super::non_ipc_serializable_reason;
    use sifr_type_system::{FunctionType, Type};

    #[test]
    fn accepts_initial_payload_families() {
        let record = Type::Class {
            name: "EchoRequest".to_string(),
            fields: vec![
                ("message".to_string(), Type::Str),
                ("attempts".to_string(), Type::Int),
                ("tags".to_string(), Type::List(Box::new(Type::Str))),
            ],
            methods: vec![],
            parent_class: None,
        };
        let payloads = [
            Type::Bool,
            Type::Int,
            Type::Float,
            Type::Str,
            Type::Bytes,
            Type::None,
            Type::Union(vec![Type::None, Type::Int]),
            Type::List(Box::new(Type::Int)),
            Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
            Type::Tuple(vec![Type::Int, Type::Str, Type::Bytes]),
            Type::Enum {
                name: "Color".to_string(),
                variants: vec![("RED".to_string(), Some(1)), ("BLUE".to_string(), Some(2))],
            },
            record,
        ];

        for payload in payloads {
            assert_eq!(non_ipc_serializable_reason(&payload), None);
        }
    }

    #[test]
    fn rejects_process_local_and_callable_payloads() {
        let process_reader = Type::Class {
            name: "PipeReader".to_string(),
            fields: vec![("_handle".to_string(), Type::Int)],
            methods: vec![],
            parent_class: None,
        };
        let callable = Type::Callable(vec![Type::Int], vec![], Box::new(Type::Int));
        let function = Type::Function(FunctionType::new(
            vec![("value".to_string(), Type::Int)],
            Type::Int,
        ));

        assert!(non_ipc_serializable_reason(&process_reader)
            .is_some_and(|reason| reason.contains("process-local process pipe handle")));
        assert!(non_ipc_serializable_reason(&callable)
            .is_some_and(|reason| reason.contains("callables are process-local code")));
        assert!(non_ipc_serializable_reason(&function)
            .is_some_and(|reason| reason.contains("callables are process-local code")));
    }
}
