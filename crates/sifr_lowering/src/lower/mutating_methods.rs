use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::TextRange;
use sifr_type_system::{ReceiverConvention, Type};

/// Canonical receiver convention for successfully resolved non-class methods.
///
/// Class and protocol methods carry their convention in `FunctionType`.
pub(in crate::lower) fn receiver_convention_for_non_class_method(
    object_ty: &Type,
    method: &str,
) -> ReceiverConvention {
    match object_ty.resolve_alias() {
        Type::PythonBuffer(_) => match method {
            "write" => ReceiverConvention::MutableBorrow,
            "release" => ReceiverConvention::Owned,
            _ => ReceiverConvention::SharedBorrow,
        },
        Type::PythonArrow(_) | Type::PythonDlpackTensor(_) => {
            if method == "release" {
                ReceiverConvention::Owned
            } else {
                ReceiverConvention::SharedBorrow
            }
        }
        Type::List(_) | Type::Dict(_, _) | Type::Set(_)
            if is_collection_mutating_method(object_ty, method) =>
        {
            ReceiverConvention::MutableBorrow
        }
        Type::AsyncGenerator(_, _) if method == "aclose" => ReceiverConvention::MutableBorrow,
        Type::Task(_, _) | Type::BlockingTask(_, _) => match method {
            "join" | "cancel_and_join" | "__sifr_timeout" => ReceiverConvention::Owned,
            _ => ReceiverConvention::SharedBorrow,
        },
        Type::JoinSet(_, _) => match method {
            "__sifr_join_all" | "__sifr_cancel_all" => ReceiverConvention::Owned,
            "add" | "__sifr_spawn_blocking" | "__sifr_spawn_cpu" => {
                ReceiverConvention::MutableBorrow
            }
            _ => ReceiverConvention::SharedBorrow,
        },
        Type::Class { name, .. } if matches!(name.as_str(), "TaskScope" | "TaskGroup") => {
            if method.starts_with("__sifr_") {
                ReceiverConvention::MutableBorrow
            } else {
                ReceiverConvention::SharedBorrow
            }
        }
        _ => ReceiverConvention::SharedBorrow,
    }
}

pub(in crate::lower) fn reject_immutable_parameter_method_mutation(
    ctx: &mut LowerCtx,
    object: &HirExpr,
    object_ty: &Type,
    method: &str,
    object_range: TextRange,
) -> bool {
    if !is_collection_mutating_method(object_ty, method) {
        return false;
    }

    if let Some(name) = mutation_receiver_root_name(object) {
        if ctx
            .scope
            .lookup(name)
            .is_some_and(|info| info.is_parameter_binding() && !info.is_mutable_binding())
        {
            super::ownership_diagnostics::immutable_parameter_mutation(ctx, name, object_range);
            return true;
        }
    }

    false
}

fn mutation_receiver_root_name(expr: &HirExpr) -> Option<&str> {
    match expr {
        HirExpr::Name { name, .. } => Some(name),
        HirExpr::FieldAccess { object, .. } | HirExpr::Index { object, .. } => {
            mutation_receiver_root_name(object)
        }
        _ => None,
    }
}

pub(in crate::lower) fn reject_immutable_method_mut_borrow_arguments(
    ctx: &mut LowerCtx,
    object_ty: &Type,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
) -> bool {
    let methods = match object_ty.resolve_alias() {
        Type::Class { methods, .. } | Type::Protocol { methods, .. } => methods,
        _ => return false,
    };
    let Some((_, signature)) = methods.iter().find(|(name, _)| name == method) else {
        return false;
    };

    let mut rejected = false;
    for (index, (arg, (_, _, convention))) in args.iter().zip(&signature.params).enumerate() {
        if !convention.is_mut_borrow() {
            continue;
        }
        let Some(root_name) = mutation_receiver_root_name(arg) else {
            continue;
        };
        let range = arg_ranges.get(index).copied().unwrap_or_default();
        if ctx
            .scope
            .lookup(root_name)
            .is_some_and(|info| info.is_parameter_binding() && !info.is_mutable_binding())
        {
            super::ownership_diagnostics::immutable_parameter_mutation(ctx, root_name, range);
            rejected = true;
        }
    }
    rejected
}

pub(in crate::lower) fn is_collection_mutating_method(object_ty: &Type, method: &str) -> bool {
    if let Type::Alias { body, .. } = object_ty {
        return is_collection_mutating_method(body, method);
    }

    match object_ty {
        Type::PythonBuffer(_) => method == "write",
        Type::List(_) => matches!(
            method,
            "append"
                | "extend"
                | "insert"
                | "clear"
                | "reverse"
                | "sort"
                | "pop"
                | "popleft"
                | "appendleft"
                | "remove"
        ),
        Type::Dict(_, _) => matches!(method, "update" | "clear" | "pop" | "setdefault"),
        Type::Set(_) => matches!(
            method,
            "add"
                | "remove"
                | "discard"
                | "clear"
                | "update"
                | "intersection_update"
                | "difference_update"
                | "symmetric_difference_update"
        ),
        _ => false,
    }
}

pub(in crate::lower) fn is_potential_collection_mutating_method(method: &str) -> bool {
    matches!(
        method,
        "write"
            | "append"
            | "extend"
            | "insert"
            | "clear"
            | "reverse"
            | "sort"
            | "pop"
            | "popleft"
            | "appendleft"
            | "remove"
            | "update"
            | "setdefault"
            | "add"
            | "discard"
            | "intersection_update"
            | "difference_update"
            | "symmetric_difference_update"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_class_receiver_registry_covers_iterators_and_task_handles() {
        let async_generator = Type::AsyncGenerator(Box::new(Type::Int), Box::new(Type::Never));
        assert_eq!(
            receiver_convention_for_non_class_method(&async_generator, "aclose"),
            ReceiverConvention::MutableBorrow
        );

        let task = Type::Task(Box::new(Type::Int), Box::new(Type::Never));
        assert_eq!(
            receiver_convention_for_non_class_method(&task, "cancel"),
            ReceiverConvention::SharedBorrow
        );
        assert_eq!(
            receiver_convention_for_non_class_method(&task, "join"),
            ReceiverConvention::Owned
        );

        let join_set = Type::JoinSet(Box::new(Type::Int), Box::new(Type::Never));
        assert_eq!(
            receiver_convention_for_non_class_method(&join_set, "add"),
            ReceiverConvention::MutableBorrow
        );
        assert_eq!(
            receiver_convention_for_non_class_method(&join_set, "__sifr_join_all"),
            ReceiverConvention::Owned
        );
    }
}
