use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use sifr_ir::{AdapterFieldDefault, TypedDeclarationMetadata, canonical_callable_identity};

pub(super) fn field_defaults(ctx: &LowerCtx, class_name: &str) -> Vec<(usize, HirExpr)> {
    ctx.class_field_defaults
        .get(class_name)
        .cloned()
        .unwrap_or_default()
}

pub(super) fn field_default_identities(ctx: &LowerCtx, class_name: &str) -> Vec<(usize, String)> {
    ctx.adapter_field_plans
        .get(class_name)
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(index, field)| match &field.default {
            AdapterFieldDefault::Factory(factory) => {
                Some((index, canonical_callable_identity(factory)))
            }
            AdapterFieldDefault::Required | AdapterFieldDefault::Const(_) => None,
        })
        .collect()
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
