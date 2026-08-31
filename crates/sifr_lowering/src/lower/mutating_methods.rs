use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::TextRange;
use sifr_type_system::{ReceiverConvention, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::lower) enum ReceiverMutationEffect {
    None,
    Growth,
    Removal,
    Reorder,
    ValueMutation,
}

pub(in crate::lower) fn receiver_mutation_effect(
    object_ty: &Type,
    method: &str,
    convention: ReceiverConvention,
) -> ReceiverMutationEffect {
    if convention != ReceiverConvention::MutableBorrow {
        return ReceiverMutationEffect::None;
    }

    match object_ty.resolve_alias() {
        Type::List(_) => match method {
            "append" | "appendleft" | "extend" | "insert" => ReceiverMutationEffect::Growth,
            "clear" | "pop" | "popleft" | "remove" => ReceiverMutationEffect::Removal,
            "reverse" | "sort" => ReceiverMutationEffect::Reorder,
            _ => ReceiverMutationEffect::Removal,
        },
        Type::Dict(_, _) => match method {
            "update" | "setdefault" => ReceiverMutationEffect::ValueMutation,
            "clear" | "pop" => ReceiverMutationEffect::Removal,
            _ => ReceiverMutationEffect::Removal,
        },
        Type::Set(_) => match method {
            "add" | "update" => ReceiverMutationEffect::Growth,
            "remove"
            | "discard"
            | "clear"
            | "intersection_update"
            | "difference_update"
            | "symmetric_difference_update" => ReceiverMutationEffect::Removal,
            _ => ReceiverMutationEffect::Removal,
        },
        Type::PythonBuffer(_) => ReceiverMutationEffect::ValueMutation,
        Type::JoinSet(_, _) if method == "add" => ReceiverMutationEffect::Growth,
        Type::Class { .. } | Type::Protocol { .. } => ReceiverMutationEffect::Removal,
        _ => ReceiverMutationEffect::Removal,
    }
}

pub(in crate::lower) fn apply_receiver_mutation_effect(
    ctx: &mut LowerCtx,
    receiver: &HirExpr,
    object_ty: &Type,
    method: &str,
    convention: ReceiverConvention,
) {
    let effect = receiver_mutation_effect(object_ty, method, convention);
    if effect == ReceiverMutationEffect::None {
        return;
    }
    let Some(target) = super::sequence_guards::hir_sequence_guard_target_name(receiver) else {
        return;
    };

    ctx.record_flow_effect(sifr_ir::FlowEffect::Mutation {
        target: target.clone(),
        operation: format!("method {method}"),
    });
    ctx.record_flow_effect(sifr_ir::FlowEffect::ClearNarrowing {
        binding: target.clone(),
    });
    if effect == ReceiverMutationEffect::Removal {
        ctx.clear_sequence_guards_for_binding(&target);
        ctx.clear_sequence_guards_for_target(&target);
    }
}

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

    #[test]
    fn receiver_effect_registry_distinguishes_collection_mutations() {
        let list = Type::List(Box::new(Type::Int));
        let dict = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));
        let set = Type::Set(Box::new(Type::Int));

        for method in ["append", "appendleft", "extend", "insert"] {
            assert_eq!(
                receiver_mutation_effect(&list, method, ReceiverConvention::MutableBorrow),
                ReceiverMutationEffect::Growth
            );
        }
        for method in ["clear", "pop", "popleft", "remove"] {
            assert_eq!(
                receiver_mutation_effect(&list, method, ReceiverConvention::MutableBorrow),
                ReceiverMutationEffect::Removal
            );
        }
        for method in ["reverse", "sort"] {
            assert_eq!(
                receiver_mutation_effect(&list, method, ReceiverConvention::MutableBorrow),
                ReceiverMutationEffect::Reorder
            );
        }
        for method in ["update", "setdefault"] {
            assert_eq!(
                receiver_mutation_effect(&dict, method, ReceiverConvention::MutableBorrow),
                ReceiverMutationEffect::ValueMutation
            );
        }
        for method in ["clear", "pop"] {
            assert_eq!(
                receiver_mutation_effect(&dict, method, ReceiverConvention::MutableBorrow),
                ReceiverMutationEffect::Removal
            );
        }
        for method in ["add", "update"] {
            assert_eq!(
                receiver_mutation_effect(&set, method, ReceiverConvention::MutableBorrow),
                ReceiverMutationEffect::Growth
            );
        }
        for method in [
            "remove",
            "discard",
            "clear",
            "intersection_update",
            "difference_update",
            "symmetric_difference_update",
        ] {
            assert_eq!(
                receiver_mutation_effect(&set, method, ReceiverConvention::MutableBorrow),
                ReceiverMutationEffect::Removal
            );
        }
        assert_eq!(
            receiver_mutation_effect(&list, "append", ReceiverConvention::SharedBorrow),
            ReceiverMutationEffect::None
        );
        assert_eq!(
            receiver_mutation_effect(&list, "future_mutator", ReceiverConvention::MutableBorrow),
            ReceiverMutationEffect::Removal
        );
    }
}
