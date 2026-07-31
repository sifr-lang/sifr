use super::{LowerCtx, Type};
use std::collections::HashMap;

pub(super) type NestedCaptureSnapshot = Vec<(String, Option<Vec<(String, Type)>>)>;
pub(super) type NestedMutationSnapshot = Vec<(String, Option<Vec<String>>)>;
pub(super) type ContainerPatchSnapshot = HashMap<String, Type>;

pub(super) fn suspend_container_specialization_patches(
    ctx: &mut LowerCtx,
) -> ContainerPatchSnapshot {
    std::mem::take(&mut ctx.pending_container_specialization_patches)
}

pub(super) fn restore_container_specialization_patches(
    enclosing: ContainerPatchSnapshot,
    function_name: &str,
    ctx: &mut LowerCtx,
) {
    let nested = std::mem::take(&mut ctx.pending_container_specialization_patches);
    ctx.pending_container_specialization_patches = enclosing;

    let captured_names: Vec<_> = ctx
        .nested_function_captures
        .get(function_name)
        .into_iter()
        .flatten()
        .map(|(name, _)| name.clone())
        .collect();
    for name in captured_names {
        if let Some(ty) = nested.get(&name) {
            ctx.pending_container_specialization_patches
                .insert(name, ty.clone());
        }
    }
}

pub(super) fn push_nested_function_captures(
    captures: &std::collections::HashMap<String, Vec<(String, Type)>>,
    ctx: &mut LowerCtx,
) -> NestedCaptureSnapshot {
    captures
        .iter()
        .map(|(name, function_captures)| {
            (
                name.clone(),
                ctx.nested_function_captures
                    .insert(name.clone(), function_captures.clone()),
            )
        })
        .collect()
}

pub(super) fn restore_nested_function_captures(
    previous: NestedCaptureSnapshot,
    ctx: &mut LowerCtx,
) {
    for (name, captures) in previous {
        if let Some(captures) = captures {
            ctx.nested_function_captures.insert(name, captures);
        } else {
            ctx.nested_function_captures.remove(&name);
        }
    }
}

pub(super) fn push_nested_function_mutations(
    mutations: &std::collections::HashMap<String, Vec<String>>,
    ctx: &mut LowerCtx,
) -> NestedMutationSnapshot {
    mutations
        .iter()
        .map(|(name, function_mutations)| {
            (
                name.clone(),
                ctx.nested_function_mutated_captures
                    .insert(name.clone(), function_mutations.clone()),
            )
        })
        .collect()
}

pub(super) fn restore_nested_function_mutations(
    previous: NestedMutationSnapshot,
    ctx: &mut LowerCtx,
) {
    for (name, mutations) in previous {
        if let Some(mutations) = mutations {
            ctx.nested_function_mutated_captures.insert(name, mutations);
        } else {
            ctx.nested_function_mutated_captures.remove(&name);
        }
    }
}
