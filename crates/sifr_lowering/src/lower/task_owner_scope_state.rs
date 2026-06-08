use super::LowerCtx;
use crate::hir_nodes::HirAsyncWithKind;
use sifr_type_system::Type;
use std::collections::HashMap;

pub(in crate::lower) struct ActiveTaskOwnerSnapshot {
    previous_task_owner_depth: usize,
    previous_task_owner_bindings_len: usize,
    target_name: Option<String>,
    previous_group_error_type: Option<Type>,
    previous_group_not_proven_open: bool,
    previous_handle_group_owners: HashMap<String, String>,
}

pub(in crate::lower) fn task_scope_type() -> Type {
    Type::Class {
        name: "TaskScope".to_string(),
        fields: vec![],
        methods: vec![],
        parent_class: None,
    }
}

pub(in crate::lower) fn task_group_type() -> Type {
    Type::Class {
        name: "TaskGroup".to_string(),
        fields: vec![],
        methods: vec![],
        parent_class: None,
    }
}

pub(in crate::lower) fn enter_task_owner_scope(
    ctx: &mut LowerCtx,
    kind: &HirAsyncWithKind,
    target: Option<&String>,
) -> ActiveTaskOwnerSnapshot {
    let target_name = matches!(
        kind,
        HirAsyncWithKind::TaskScope | HirAsyncWithKind::TaskGroup { .. }
    )
    .then(|| target.cloned())
    .flatten();
    let previous_group_error_type = target_name
        .as_ref()
        .and_then(|name| ctx.task_group_error_types.get(name).cloned());
    let previous_group_not_proven_open = target_name
        .as_ref()
        .is_some_and(|name| ctx.task_groups_not_proven_open.contains(name));
    let snapshot = ActiveTaskOwnerSnapshot {
        previous_task_owner_depth: ctx.active_task_owner_depth,
        previous_task_owner_bindings_len: ctx.active_task_owner_bindings.len(),
        target_name,
        previous_group_error_type,
        previous_group_not_proven_open,
        previous_handle_group_owners: ctx.task_handle_group_owners.clone(),
    };
    if matches!(
        kind,
        HirAsyncWithKind::TaskScope | HirAsyncWithKind::TaskGroup { .. }
    ) {
        ctx.active_task_owner_depth = ctx.active_task_owner_depth.saturating_add(1);
        if let Some(name) = target {
            let owner_ty = match kind {
                HirAsyncWithKind::TaskScope => task_scope_type(),
                HirAsyncWithKind::TaskGroup { .. } => task_group_type(),
                _ => unreachable!("task owner kind checked above"),
            };
            ctx.active_task_owner_bindings
                .push((name.clone(), owner_ty));
        }
    }
    snapshot
}

pub(in crate::lower) fn exit_task_owner_scope(
    ctx: &mut LowerCtx,
    snapshot: ActiveTaskOwnerSnapshot,
) {
    ctx.active_task_owner_depth = snapshot.previous_task_owner_depth;
    ctx.active_task_owner_bindings
        .truncate(snapshot.previous_task_owner_bindings_len);

    let Some(target_name) = snapshot.target_name else {
        return;
    };

    restore_target_group_error_type(ctx, &target_name, snapshot.previous_group_error_type);
    restore_target_group_open_state(ctx, &target_name, snapshot.previous_group_not_proven_open);
    restore_target_handle_group_owners(ctx, &target_name, snapshot.previous_handle_group_owners);
}

fn restore_target_group_error_type(
    ctx: &mut LowerCtx,
    target_name: &str,
    previous_group_error_type: Option<Type>,
) {
    if let Some(previous) = previous_group_error_type {
        ctx.task_group_error_types
            .insert(target_name.to_string(), previous);
    } else {
        ctx.task_group_error_types.remove(target_name);
    }
}

fn restore_target_group_open_state(
    ctx: &mut LowerCtx,
    target_name: &str,
    previous_group_not_proven_open: bool,
) {
    if previous_group_not_proven_open {
        ctx.task_groups_not_proven_open
            .insert(target_name.to_string());
    } else {
        ctx.task_groups_not_proven_open.remove(target_name);
    }
}

fn restore_target_handle_group_owners(
    ctx: &mut LowerCtx,
    target_name: &str,
    previous_handle_group_owners: HashMap<String, String>,
) {
    ctx.task_handle_group_owners.retain(|handle, owner| {
        owner != target_name || previous_handle_group_owners.get(handle) == Some(owner)
    });
    for (handle, owner) in previous_handle_group_owners {
        if owner == target_name {
            ctx.task_handle_group_owners.insert(handle, owner);
        }
    }
}
