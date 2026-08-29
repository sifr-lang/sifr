use crate::{CodegenError, RustExpr, RustParam, RustType, try_lower_leaf_expr_result};
use sifr_ir::HirExpr;
use sifr_type_system::Type;

pub(crate) fn resolve_alias_type_for_plain_call(ty: &Type) -> &Type {
    match ty {
        Type::Alias { body, .. } => resolve_alias_type_for_plain_call(body),
        _ => ty,
    }
}

pub(crate) fn homogeneous_large_tuple_backing_array(ty: &Type) -> Option<(&Type, usize)> {
    let Type::Tuple(items) = resolve_alias_type_for_plain_call(ty) else {
        return None;
    };
    let first = items.first()?;
    if items.len() <= 12 || !items.iter().all(|item| item == first) {
        return None;
    }
    Some((first, items.len()))
}

pub(crate) fn try_lower_leaf_or_name_expr_result(
    expr: &HirExpr,
) -> Result<Option<RustExpr>, CodegenError> {
    if let HirExpr::Lambda { params, body, .. } = expr {
        let lowered_body = try_lower_leaf_or_name_expr_result(body)?
            .ok_or_else(|| CodegenError::new("lambda body could not be lowered"))?;
        let lowered_params = params
            .iter()
            .map(|param| RustParam::Named {
                name: param.name.clone(),
                ty: RustType::Named("_".to_string()),
            })
            .collect::<Vec<_>>();
        return Ok(Some(RustExpr::Closure {
            params: lowered_params,
            body: Box::new(lowered_body),
            is_move: false,
        }));
    }
    if let Some(lowered) = try_lower_leaf_expr_result(expr)? {
        return Ok(Some(lowered));
    }
    if let HirExpr::Name { name, .. } = expr {
        return Ok(Some(RustExpr::Ident(name.clone())));
    }
    Ok(None)
}
