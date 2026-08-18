use super::{HirExpr, LowerCtx, Type};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{AdapterFieldDefault, CallableIdentity, StaticProgramValue};

pub(in crate::lower) fn defaults_for_class(
    class_name: &str,
    fields: &[(String, Type)],
    source_defaults: Vec<(usize, HirExpr)>,
    ctx: &mut LowerCtx,
) -> Vec<(usize, HirExpr)> {
    let Some(plans) = ctx.adapter_field_plans.get(class_name).cloned() else {
        return source_defaults;
    };
    if plans.len() != fields.len() {
        invalid_plan(
            ctx,
            class_name,
            &format!(
                "normalized field count changed before finalization (planned {}, finalized {}: {})",
                plans.len(),
                fields.len(),
                fields
                    .iter()
                    .map(|(name, _)| name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
        return Vec::new();
    }
    let mut defaults = Vec::new();
    for (index, (plan, (name, ty))) in plans.iter().zip(fields).enumerate() {
        if plan.name != *name || plan.declared_type != *ty {
            invalid_plan(
                ctx,
                class_name,
                &format!(
                    "normalized field contract changed before finalization at index {index} (planned '{}: {}', finalized '{}: {}')",
                    plan.name,
                    plan.declared_type.display_name(),
                    name,
                    ty.display_name()
                ),
            );
            return Vec::new();
        }
        let value = match &plan.default {
            AdapterFieldDefault::Required => continue,
            AdapterFieldDefault::Const(value) => static_value_expr(value, ty),
            AdapterFieldDefault::Factory(factory) => factory_expr(factory, ty, ctx),
        };
        let Some(value) = value else {
            invalid_plan(
                ctx,
                class_name,
                &format!("default for field '{name}' cannot be lowered"),
            );
            return Vec::new();
        };
        defaults.push((index, value));
    }
    defaults
}

fn static_value_expr(value: &StaticProgramValue, expected: &Type) -> Option<HirExpr> {
    match value {
        StaticProgramValue::None => Some(HirExpr::NoneLiteral),
        StaticProgramValue::Bool(value) => Some(HirExpr::BoolLiteral(*value)),
        StaticProgramValue::Integer(value) => value
            .parse::<i64>()
            .map(HirExpr::IntLiteral)
            .ok()
            .or_else(|| Some(HirExpr::LargeIntLiteral(value.clone()))),
        StaticProgramValue::FloatBits(value) => Some(HirExpr::FloatLiteral(f64::from_bits(*value))),
        StaticProgramValue::String(value) => Some(HirExpr::StringLiteral(value.clone())),
        StaticProgramValue::Bytes(values) => Some(HirExpr::ListLiteral {
            elements: values
                .iter()
                .map(|value| HirExpr::IntLiteral(i64::from(*value)))
                .collect(),
            ty: Type::Bytes,
        }),
        StaticProgramValue::Tuple(values) => {
            let Type::Tuple(types) = expected.resolve_alias() else {
                return None;
            };
            if values.len() != types.len() {
                return None;
            }
            Some(HirExpr::TupleLiteral {
                elements: values
                    .iter()
                    .zip(types)
                    .map(|(value, ty)| static_value_expr(value, ty))
                    .collect::<Option<Vec<_>>>()?,
                ty: expected.clone(),
            })
        }
        StaticProgramValue::List(values) => {
            let Type::List(item) = expected.resolve_alias() else {
                return None;
            };
            Some(HirExpr::ListLiteral {
                elements: values
                    .iter()
                    .map(|value| static_value_expr(value, item))
                    .collect::<Option<Vec<_>>>()?,
                ty: expected.clone(),
            })
        }
        StaticProgramValue::Record(values) => record_expr(values, expected),
        StaticProgramValue::CallableIdentity(_) => None,
    }
}

fn record_expr(values: &[(String, StaticProgramValue)], expected: &Type) -> Option<HirExpr> {
    match expected.resolve_alias() {
        Type::Dict(key, item) if matches!(key.resolve_alias(), Type::Str) => {
            Some(HirExpr::DictLiteral {
                keys: values
                    .iter()
                    .map(|(name, _)| HirExpr::StringLiteral(name.clone()))
                    .collect(),
                values: values
                    .iter()
                    .map(|(_, value)| static_value_expr(value, item))
                    .collect::<Option<Vec<_>>>()?,
                ty: expected.clone(),
            })
        }
        Type::Class { name, fields, .. } if fields.len() == values.len() => {
            let args = fields
                .iter()
                .map(|(field_name, field_type)| {
                    values
                        .iter()
                        .find(|(name, _)| name == field_name)
                        .and_then(|(_, value)| static_value_expr(value, field_type))
                })
                .collect::<Option<Vec<_>>>()?;
            Some(HirExpr::Call {
                func: name.clone(),
                args,
                mutable_arg_places: Vec::new(),
                ty: expected.clone(),
            })
        }
        _ => None,
    }
}

fn factory_expr(factory: &CallableIdentity, expected: &Type, ctx: &LowerCtx) -> Option<HirExpr> {
    if factory.module == "sifr.builtins" {
        return Some(HirExpr::Call {
            func: factory.symbol.clone(),
            args: Vec::new(),
            mutable_arg_places: Vec::new(),
            ty: expected.clone(),
        });
    }
    if let Some(owner) = &factory.owner {
        let owner_name = owner
            .rsplit_once('.')
            .map_or(owner.as_str(), |(_, name)| name);
        let local_owner = ctx
            .imported_symbol_bindings
            .get(&(factory.module.clone(), owner_name.to_string()))
            .map_or(owner_name, String::as_str);
        if factory.symbol == "__init__" {
            return Some(HirExpr::ConstructorCall {
                class_name: local_owner.to_string(),
                args: Vec::new(),
                ty: expected.clone(),
            });
        }
        ctx.class_types.get(local_owner)?;
        return Some(HirExpr::Call {
            func: format!("{local_owner}::{}", factory.symbol),
            args: Vec::new(),
            mutable_arg_places: Vec::new(),
            ty: expected.clone(),
        });
    }
    let callable = ctx
        .imported_symbol_bindings
        .get(&(factory.module.clone(), factory.symbol.clone()))
        .cloned()
        .unwrap_or_else(|| factory.symbol.clone());
    Some(HirExpr::Call {
        func: callable,
        args: Vec::new(),
        mutable_arg_places: Vec::new(),
        ty: expected.clone(),
    })
}

fn invalid_plan(ctx: &mut LowerCtx, class_name: &str, detail: &str) {
    ctx.error_with_code_at(
        DiagnosticCode::META_MALFORMED_DECLARATION,
        format!("adapted class '{class_name}' has an invalid finalized field plan: {detail}"),
        ruff_text_size::TextRange::default(),
    );
}
