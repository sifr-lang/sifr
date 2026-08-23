use super::{ShapeFieldDefault, const_value_from_hir, validated_adapter_value};
use sifr_lowering::{AdapterFieldDefault, AdapterFieldPlan, HirExpr};

pub(super) fn shape_field_default(
    index: usize,
    defaults: Option<&Vec<(usize, HirExpr)>>,
    field_plans: Option<&[AdapterFieldPlan]>,
) -> ShapeFieldDefault {
    if let Some(plan) = field_plans.and_then(|plans| plans.get(index)) {
        return match &plan.default {
            AdapterFieldDefault::Required => ShapeFieldDefault::Required,
            AdapterFieldDefault::Const(value) => {
                ShapeFieldDefault::Const(validated_adapter_value(value))
            }
            AdapterFieldDefault::Factory(factory) => ShapeFieldDefault::Factory(factory.clone()),
        };
    }
    let Some(value) = defaults
        .and_then(|values| values.iter().find(|(field, _)| *field == index))
        .map(|(_, value)| value)
    else {
        return ShapeFieldDefault::Required;
    };
    const_value_from_hir(value)
        .map(ShapeFieldDefault::Const)
        .unwrap_or(ShapeFieldDefault::Runtime)
}
