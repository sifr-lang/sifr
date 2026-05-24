use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::TextRange;
use sifr_type_system::{union_contains_none, Type};

pub(in crate::lower) fn validate_control_flow_condition(
    condition: &HirExpr,
    keyword: &str,
    primary_range: TextRange,
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
        let actual = condition.ty().display_name();
        super::flow_diagnostics::invalid_condition_type(
            ctx,
            keyword,
            actual.as_str(),
            primary_range,
        );
    }
}
