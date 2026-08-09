use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use sifr_ir::TypedDeclarationMetadata;

pub(super) fn field_defaults(ctx: &LowerCtx, class_name: &str) -> Vec<(usize, HirExpr)> {
    ctx.class_field_defaults
        .get(class_name)
        .cloned()
        .unwrap_or_default()
}

pub(super) fn declaration_metadata(
    ctx: &LowerCtx,
    class_name: &str,
) -> Vec<TypedDeclarationMetadata> {
    ctx.declaration_metadata
        .iter()
        .filter(|metadata| metadata.owner == class_name)
        .cloned()
        .collect()
}
