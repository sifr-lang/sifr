use super::builtin_calls::{DEFAULTDICT_INT_ALIAS, DEFAULTDICT_LIST_ALIAS, DEFAULTDICT_SET_ALIAS};
use super::guarded_index::guarded_sequence_index_result_type;
use super::type_bounds::reject_unavailable_dict_hash_key;
use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::ExprSubscript;
use sifr_type_system::{make_union, Type};

fn tuple_members_for_subscript(object_ty: &Type) -> Option<Vec<Type>> {
    match object_ty.resolve_alias() {
        Type::Tuple(elems) => Some(elems.clone()),
        Type::Union(members) => {
            let non_none: Vec<&Type> = members
                .iter()
                .filter(|member| !matches!(member.resolve_alias(), Type::None))
                .collect();
            if non_none.len() != 1 {
                return None;
            }
            let Type::Tuple(elems) = non_none[0].resolve_alias() else {
                return None;
            };
            Some(elems.clone())
        }
        _ => None,
    }
}

pub(in crate::lower) fn resolve_subscript_result_type(
    sub: &ExprSubscript,
    object_ty: &Type,
    index: &HirExpr,
    index_ty: &Type,
    ctx: &mut LowerCtx,
) -> Type {
    reject_unavailable_dict_hash_key(object_ty, index_ty, "dict indexing", sub.range(), ctx);

    if let Type::Alias { name, body, .. } = object_ty {
        if matches!(
            name.as_str(),
            DEFAULTDICT_INT_ALIAS | DEFAULTDICT_LIST_ALIAS | DEFAULTDICT_SET_ALIAS
        ) {
            if let Type::Dict(_, value_ty) = body.resolve_alias() {
                return *value_ty.clone();
            }
        }
    }

    if let Some(elems) = tuple_members_for_subscript(object_ty) {
        if let HirExpr::IntLiteral(raw_index) = index {
            let Ok(len_i64) = i64::try_from(elems.len()) else {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISMATCH,
                    "tuple too large for indexing".to_string(),
                    sub.slice.range(),
                );
                return Type::Any;
            };
            let normalized = if *raw_index < 0 {
                len_i64 + *raw_index
            } else {
                *raw_index
            };
            if let Ok(idx) = usize::try_from(normalized) {
                if idx < elems.len() {
                    return elems[idx].clone();
                }
            }
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                "tuple index out of range".to_string(),
                sub.slice.range(),
            );
            return Type::Any;
        }
        if index_ty == &Type::Int && !elems.is_empty() {
            return make_union(elems.clone());
        }
    }

    if let Some(guarded_ty) = guarded_sequence_index_result_type(sub, object_ty, ctx) {
        return guarded_ty;
    }

    object_ty.index_result_type(index_ty).unwrap_or_else(|| {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "cannot index type '{}' with '{}'",
                object_ty.display_name(),
                index_ty.display_name()
            ),
            sub.range(),
        );
        Type::Any
    })
}
