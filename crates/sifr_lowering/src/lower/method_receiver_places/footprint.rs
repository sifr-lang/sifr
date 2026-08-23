use super::{
    HirExpr, LowerCtx, Place, PlaceProjection, extract_place, resolve_field_identity,
    root_binding_id,
};
use crate::hir_nodes::HirFStringPart;
use sifr_ir::{BindingId, FieldIdentity};
use sifr_type_system::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Footprint {
    Place(Place),
    Dynamic(BindingId),
}

pub(super) fn expression_overlaps(expr: &HirExpr, mutable_place: &Place, ctx: &LowerCtx) -> bool {
    let mut footprint = Vec::new();
    collect_footprint(expr, ctx, &mut footprint);
    footprint.into_iter().any(|access| match access {
        Footprint::Place(place) => places_overlap(mutable_place, &place),
        Footprint::Dynamic(root) => root == mutable_place.root,
    })
}

pub(super) fn places_overlap(left: &Place, right: &Place) -> bool {
    left.root == right.root
        && (left.projections.starts_with(&right.projections)
            || right.projections.starts_with(&left.projections))
}

fn extract_footprint_place(expr: &HirExpr, ctx: &LowerCtx) -> Option<Place> {
    match expr {
        HirExpr::Name {
            binding_id: Some(root),
            ..
        } => Some(Place {
            root: *root,
            projections: Vec::new(),
        }),
        HirExpr::FieldAccess { object, field, .. } => {
            let mut place = extract_footprint_place(object, ctx)?;
            place
                .projections
                .push(PlaceProjection::Field(resolve_field_identity(
                    object.ty(),
                    field,
                    ctx,
                )));
            Some(place)
        }
        _ => None,
    }
}

fn callable_field_identity(
    object: &HirExpr,
    method: &str,
    ctx: &LowerCtx,
) -> Option<FieldIdentity> {
    let Type::Class {
        fields, methods, ..
    } = object.ty().resolve_alias()
    else {
        return None;
    };
    if methods.iter().any(|(name, _)| name == method) {
        return None;
    }
    fields
        .iter()
        .find(|(name, ty)| {
            name == method
                && matches!(
                    ty.resolve_alias(),
                    Type::Callable(..) | Type::AsyncCallable(..)
                )
        })
        .map(|_| resolve_field_identity(object.ty(), method, ctx))
}

fn collect_callable_field(
    object: &HirExpr,
    method: &str,
    ctx: &LowerCtx,
    footprint: &mut Vec<Footprint>,
) -> bool {
    let Some(field) = callable_field_identity(object, method, ctx) else {
        return false;
    };
    if let Some(mut place) = extract_footprint_place(object, ctx) {
        place.projections.push(PlaceProjection::Field(field));
        footprint.push(Footprint::Place(place));
    } else {
        collect_footprint(object, ctx, footprint);
    }
    true
}

fn collect_footprint(expr: &HirExpr, ctx: &LowerCtx, footprint: &mut Vec<Footprint>) {
    if let Ok(place) = extract_place(expr, ctx) {
        footprint.push(Footprint::Place(place));
        return;
    }
    match expr {
        HirExpr::Index { object, index, .. } => {
            if let Some(root) = root_binding_id(object) {
                footprint.push(Footprint::Dynamic(root));
            }
            collect_footprint(object, ctx, footprint);
            collect_footprint(index, ctx, footprint);
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            if let Some(root) = root_binding_id(object) {
                footprint.push(Footprint::Dynamic(root));
            }
            collect_footprint(object, ctx, footprint);
            for bound in [start, stop, step].into_iter().flatten() {
                collect_footprint(bound, ctx, footprint);
            }
        }
        HirExpr::BinOp { left, right, .. } => {
            collect_footprint(left, ctx, footprint);
            collect_footprint(right, ctx, footprint);
        }
        HirExpr::UnaryOp { operand, .. }
        | HirExpr::Await { value: operand, .. }
        | HirExpr::WalrusExpr { value: operand, .. }
        | HirExpr::QuestionMark { expr: operand, .. }
        | HirExpr::OkWrap { value: operand, .. }
        | HirExpr::ErrWrap { value: operand, .. } => {
            collect_footprint(operand, ctx, footprint);
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            collect_footprint(left, ctx, footprint);
            collect_many(comparators, ctx, footprint);
        }
        HirExpr::BoolOp { values, .. }
        | HirExpr::Call { args: values, .. }
        | HirExpr::GenericCall { args: values, .. }
        | HirExpr::PythonCall { args: values, .. }
        | HirExpr::IntrinsicCall { args: values, .. }
        | HirExpr::IteratorCall { args: values, .. }
        | HirExpr::ListLiteral {
            elements: values, ..
        }
        | HirExpr::SetLiteral {
            elements: values, ..
        }
        | HirExpr::TupleLiteral {
            elements: values, ..
        }
        | HirExpr::ConstructorCall { args: values, .. }
        | HirExpr::SuperCall { args: values, .. } => {
            collect_many(values, ctx, footprint);
        }
        HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } => {
            if !collect_callable_field(object, method, ctx, footprint) {
                collect_footprint(object, ctx, footprint);
            }
            collect_many(args, ctx, footprint);
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            collect_footprint(condition, ctx, footprint);
            collect_footprint(then_expr, ctx, footprint);
            collect_footprint(else_expr, ctx, footprint);
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            collect_footprint(start, ctx, footprint);
            collect_footprint(end, ctx, footprint);
            if let Some(step) = step {
                collect_footprint(step, ctx, footprint);
            }
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            collect_many(keys, ctx, footprint);
            collect_many(values, ctx, footprint);
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => {
            collect_footprint(element, ctx, footprint);
            collect_footprint(collection, ctx, footprint);
        }
        HirExpr::FString { parts, .. } => {
            for part in parts {
                if let HirFStringPart::Expr(expr) = part {
                    collect_footprint(expr, ctx, footprint);
                }
            }
        }
        HirExpr::Lambda { body, .. } => collect_footprint(body, ctx, footprint),
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            collect_footprint(expr, ctx, footprint);
            collect_generators(generators, ctx, footprint);
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            collect_footprint(key_expr, ctx, footprint);
            collect_footprint(val_expr, ctx, footprint);
            collect_generators(generators, ctx, footprint);
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            collect_footprint(expr, ctx, footprint);
            collect_footprint(iter, ctx, footprint);
            if let Some(filter) = filter {
                collect_footprint(filter, ctx, footprint);
            }
        }
        HirExpr::FieldAccess { object, .. } => {
            if let Some(place) = extract_footprint_place(expr, ctx) {
                footprint.push(Footprint::Place(place));
            } else {
                if let Some(root) = root_binding_id(object) {
                    footprint.push(Footprint::Dynamic(root));
                }
                collect_footprint(object, ctx, footprint);
            }
        }
        HirExpr::IntLiteral(_)
        | HirExpr::LargeIntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::Name { .. }
        | HirExpr::EnumVariant { .. } => {}
    }
}

fn collect_many(expressions: &[HirExpr], ctx: &LowerCtx, footprint: &mut Vec<Footprint>) {
    for expression in expressions {
        collect_footprint(expression, ctx, footprint);
    }
}

fn collect_generators(
    generators: &[(String, HirExpr, Option<HirExpr>)],
    ctx: &LowerCtx,
    footprint: &mut Vec<Footprint>,
) {
    for (_, iter, filter) in generators {
        collect_footprint(iter, ctx, footprint);
        if let Some(filter) = filter {
            collect_footprint(filter, ctx, footprint);
        }
    }
}
