use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use sifr_type_system::{union_contains_none, Type};

pub(super) fn validate_control_flow_condition(
    condition: &HirExpr,
    keyword: &str,
    ctx: &mut LowerCtx,
) {
    let is_supported = matches!(
        condition.ty().resolve_alias(),
        Type::Bool
            | Type::LiteralBool(_)
            | Type::List(_)
            | Type::Set(_)
            | Type::Dict(_, _)
            | Type::Tuple(_)
            | Type::Str
            | Type::Bytes
            | Type::Class { .. }
            | Type::Protocol { .. }
            | Type::Any
            | Type::Unknown
    ) || union_contains_none(condition.ty());
    if !is_supported {
        ctx.error(format!(
            "{keyword} condition must be bool or collection/string truthiness, got '{}'",
            condition.ty().display_name()
        ));
    }
}
