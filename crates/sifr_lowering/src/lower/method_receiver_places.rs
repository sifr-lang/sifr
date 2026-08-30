use super::{
    LowerCtx, classes::fixed_trait_receiver_convention, method_receiver_analysis::method_signature,
    ownership_diagnostics,
};
use crate::hir_nodes::{HirExpr, HirFunction};
use crate::scope::{BindingKind, BindingMutability};
use ruff_text_size::TextRange;
use sifr_ir::{
    BindingId, FieldIdentity, MutableArgumentTarget, MutableReceiverTarget, Place, PlaceProjection,
};
use sifr_type_system::{FunctionType, ParamConvention, ReceiverConvention, Type};

mod footprint;
mod indexed_storage;

use footprint::expression_overlaps;
#[cfg(test)]
use footprint::places_overlap;
use indexed_storage::{
    indexed_storage_borrow_follows_argument_evaluation, specialized_indexed_storage_base,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InvalidPlace {
    ImmutableParameter,
    Unsupported,
}

#[derive(Clone, Copy)]
struct ReceiverOverlap<'a> {
    target: Option<&'a MutableReceiverTarget>,
    object: Option<(&'a HirExpr, Option<ReceiverConvention>)>,
    borrow_follows_argument_evaluation: bool,
}

pub(super) fn validate_function_method_places(function: &mut HirFunction, ctx: &mut LowerCtx) {
    // Fixed Rust trait methods report attempted receiver mutation through
    // PROTO-0006 during receiver analysis. Avoid layering a secondary place
    // diagnostic onto the same invalid body.
    let allow_fixed_receiver = fixed_trait_receiver_convention(&function.name).is_some();
    sifr_ir::visit_hir_function_exprs_mut(function, &mut |expr| {
        let HirExpr::MethodCall {
            object,
            method,
            args,
            receiver_convention,
            receiver_target,
            mutable_arg_places,
            source,
            ..
        } = expr
        else {
            return;
        };

        let signature = method_signature(object.ty(), method, &ctx.class_types, &ctx.functions);
        let conventions = signature
            .as_ref()
            .map(|signature| {
                signature
                    .params
                    .iter()
                    .map(|(_, _, convention)| *convention)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let receiver_range = source
            .as_ref()
            .map_or_else(TextRange::default, |source| source.receiver_range);
        let arg_ranges = source
            .as_ref()
            .map_or_else(Vec::new, |source| source.arg_ranges.clone());

        *receiver_target = match receiver_convention {
            Some(ReceiverConvention::MutableBorrow) => {
                if let Some(storage) = specialized_indexed_storage_base(object, method) {
                    match prove_mutable_place(storage, ctx, allow_fixed_receiver) {
                        Ok(place) => Some(MutableReceiverTarget::SpecializedIndexedStorage(place)),
                        Err(InvalidPlace::ImmutableParameter) => {
                            report_immutable_root(storage, receiver_range, ctx);
                            None
                        }
                        Err(InvalidPlace::Unsupported) => {
                            ownership_diagnostics::unsupported_mutable_receiver_place(
                                ctx,
                                &place_display(object, ctx),
                                receiver_range,
                            );
                            None
                        }
                    }
                } else {
                    match prove_mutable_place(object, ctx, allow_fixed_receiver) {
                        Ok(place) => Some(MutableReceiverTarget::Place(place)),
                        Err(InvalidPlace::ImmutableParameter) => {
                            report_immutable_root(object, receiver_range, ctx);
                            None
                        }
                        Err(InvalidPlace::Unsupported) if is_owned_temporary(object) => {
                            Some(MutableReceiverTarget::OwnedTemporary)
                        }
                        Err(InvalidPlace::Unsupported) => {
                            ownership_diagnostics::unsupported_mutable_receiver_place(
                                ctx,
                                &place_display(object, ctx),
                                receiver_range,
                            );
                            None
                        }
                    }
                }
            }
            Some(
                ReceiverConvention::SharedBorrow
                | ReceiverConvention::Owned
                | ReceiverConvention::OwnedMutable,
            )
            | None => None,
        };

        *mutable_arg_places =
            prove_mutable_arguments(args, &conventions, &arg_ranges, allow_fixed_receiver, ctx);
        validate_call_overlaps(
            ReceiverOverlap {
                target: receiver_target.as_ref(),
                object: Some((object, *receiver_convention)),
                borrow_follows_argument_evaluation:
                    indexed_storage_borrow_follows_argument_evaluation(object, method),
            },
            args,
            mutable_arg_places,
            &conventions,
            &arg_ranges,
            None,
            ctx,
        );
    });
}

pub(super) fn validate_regular_call_arguments(
    args: &[HirExpr],
    signature: &FunctionType,
    arg_ranges: &[Option<TextRange>],
    fallback_range: TextRange,
    function_name: &str,
    ctx: &mut LowerCtx,
) -> Vec<Option<MutableArgumentTarget>> {
    let conventions = signature
        .params
        .iter()
        .map(|(_, _, convention)| *convention)
        .collect::<Vec<_>>();
    let ranges = arg_ranges
        .iter()
        .map(|range| range.unwrap_or(fallback_range))
        .collect::<Vec<_>>();
    let places = prove_mutable_arguments(args, &conventions, &ranges, true, ctx);
    validate_call_overlaps(
        ReceiverOverlap {
            target: None,
            object: None,
            borrow_follows_argument_evaluation: false,
        },
        args,
        &places,
        &conventions,
        &ranges,
        Some(function_name),
        ctx,
    );
    places
}

fn prove_mutable_arguments(
    args: &[HirExpr],
    conventions: &[ParamConvention],
    ranges: &[TextRange],
    allow_pending_receiver: bool,
    ctx: &mut LowerCtx,
) -> Vec<Option<MutableArgumentTarget>> {
    args.iter()
        .enumerate()
        .map(|(index, arg)| {
            if !conventions
                .get(index)
                .is_some_and(|convention| convention.is_mut_borrow())
            {
                return None;
            }
            let range = ranges.get(index).copied().unwrap_or_default();
            match prove_mutable_place(arg, ctx, allow_pending_receiver) {
                Ok(place) => Some(MutableArgumentTarget::Place(place)),
                Err(InvalidPlace::ImmutableParameter) => {
                    report_immutable_root(arg, range, ctx);
                    None
                }
                Err(InvalidPlace::Unsupported) if is_owned_temporary(arg) => {
                    Some(MutableArgumentTarget::OwnedTemporary)
                }
                Err(InvalidPlace::Unsupported) => {
                    ownership_diagnostics::unsupported_mutable_receiver_place(
                        ctx,
                        &place_display(arg, ctx),
                        range,
                    );
                    None
                }
            }
        })
        .collect()
}

fn validate_call_overlaps(
    receiver: ReceiverOverlap<'_>,
    args: &[HirExpr],
    mutable_arg_places: &[Option<MutableArgumentTarget>],
    conventions: &[ParamConvention],
    arg_ranges: &[TextRange],
    function_name: Option<&str>,
    ctx: &mut LowerCtx,
) {
    if let Some(MutableReceiverTarget::Place(receiver)) = receiver.target {
        for (index, arg) in args.iter().enumerate() {
            if expression_overlaps(arg, receiver, ctx) {
                report_overlap(receiver, index, arg_ranges, ctx);
            }
        }
    }
    if !receiver.borrow_follows_argument_evaluation {
        if let Some(MutableReceiverTarget::SpecializedIndexedStorage(base)) = receiver.target {
            for (index, arg) in args.iter().enumerate() {
                if expression_overlaps(arg, base, ctx) {
                    report_overlap(base, index, arg_ranges, ctx);
                }
            }
        }
    }
    if let Some((
        receiver,
        Some(
            ReceiverConvention::SharedBorrow
            | ReceiverConvention::Owned
            | ReceiverConvention::OwnedMutable,
        ),
    )) = receiver.object
    {
        for (index, mutable_place) in mutable_arg_places.iter().enumerate() {
            let Some(MutableArgumentTarget::Place(mutable_place)) = mutable_place else {
                continue;
            };
            if expression_overlaps(receiver, mutable_place, ctx) {
                report_overlap(mutable_place, index, arg_ranges, ctx);
            }
        }
    }

    let same_binding_pairs =
        report_same_binding_conflicts(args, conventions, arg_ranges, function_name, ctx);
    for (mutable_index, mutable_place) in mutable_arg_places.iter().enumerate() {
        let Some(MutableArgumentTarget::Place(mutable_place)) = mutable_place else {
            continue;
        };
        for (other_index, arg) in args.iter().enumerate() {
            if mutable_index == other_index {
                continue;
            }
            let pair = (
                mutable_index.min(other_index),
                mutable_index.max(other_index),
            );
            if same_binding_pairs.contains(&pair) {
                continue;
            }
            if other_index < mutable_index
                && mutable_arg_places
                    .get(other_index)
                    .is_some_and(Option::is_some)
            {
                continue;
            }
            if expression_overlaps(arg, mutable_place, ctx) {
                report_overlap(
                    mutable_place,
                    mutable_index.max(other_index),
                    arg_ranges,
                    ctx,
                );
            }
        }
    }
}

fn report_same_binding_conflicts(
    args: &[HirExpr],
    conventions: &[ParamConvention],
    arg_ranges: &[TextRange],
    function_name: Option<&str>,
    ctx: &mut LowerCtx,
) -> Vec<(usize, usize)> {
    let Some(function_name) = function_name else {
        return Vec::new();
    };
    let mut reported = Vec::new();
    for left_index in 0..args.len() {
        for right_index in (left_index + 1)..args.len() {
            let (
                HirExpr::Name {
                    name: left_name,
                    binding_id: Some(left_id),
                    ..
                },
                HirExpr::Name {
                    binding_id: Some(right_id),
                    ..
                },
            ) = (&args[left_index], &args[right_index])
            else {
                continue;
            };
            if left_id != right_id {
                continue;
            }
            let left = conventions
                .get(left_index)
                .copied()
                .unwrap_or_else(ParamConvention::borrow);
            let right = conventions
                .get(right_index)
                .copied()
                .unwrap_or_else(ParamConvention::borrow);
            let range = arg_ranges.get(right_index).copied().unwrap_or_default();
            if left.is_mut_borrow() && right.is_mut_borrow() {
                ownership_diagnostics::double_mutable_borrow(ctx, left_name, function_name, range);
            } else if left.is_shared_borrow() && right.is_mut_borrow() {
                ownership_diagnostics::mutable_borrow_after_immutable(
                    ctx,
                    left_name,
                    function_name,
                    range,
                );
            } else if left.is_mut_borrow() && right.is_shared_borrow() {
                ownership_diagnostics::immutable_borrow_after_mutable(
                    ctx,
                    left_name,
                    function_name,
                    range,
                );
            } else {
                continue;
            }
            reported.push((left_index, right_index));
        }
    }
    reported
}

fn report_overlap(place: &Place, later_index: usize, arg_ranges: &[TextRange], ctx: &mut LowerCtx) {
    ownership_diagnostics::same_call_place_conflict(
        ctx,
        &display_checked_place(place, ctx),
        arg_ranges.get(later_index).copied().unwrap_or_default(),
    );
}

fn prove_mutable_place(
    expr: &HirExpr,
    ctx: &LowerCtx,
    allow_pending_receiver: bool,
) -> Result<Place, InvalidPlace> {
    let place = extract_place(expr, ctx)?;
    let Some(fact) = ctx.scope.retained_binding(place.root) else {
        return Err(InvalidPlace::Unsupported);
    };
    if is_optional_or_recursive(&fact.ty)
        || matches!(
            fact.ty.resolve_alias(),
            Type::Callable(..) | Type::AsyncCallable(..)
        )
    {
        return Err(InvalidPlace::Unsupported);
    }
    match fact.binding_kind {
        BindingKind::Local => Ok(place),
        BindingKind::Function => Err(InvalidPlace::Unsupported),
        BindingKind::ModuleConstant => Err(InvalidPlace::Unsupported),
        BindingKind::EphemeralLocal(_) => Err(InvalidPlace::Unsupported),
        BindingKind::Parameter => {
            if fact.mutability == BindingMutability::Mutable
                || fact
                    .parameter_convention
                    .is_some_and(ParamConvention::is_mutable)
            {
                Ok(place)
            } else {
                Err(InvalidPlace::ImmutableParameter)
            }
        }
        BindingKind::Receiver => {
            if allow_pending_receiver || fact.mutability == BindingMutability::Mutable {
                Ok(place)
            } else {
                Err(InvalidPlace::Unsupported)
            }
        }
    }
}

fn extract_place(expr: &HirExpr, ctx: &LowerCtx) -> Result<Place, InvalidPlace> {
    match expr {
        HirExpr::Name {
            binding_id: Some(root),
            ..
        } => Ok(Place {
            root: *root,
            projections: Vec::new(),
        }),
        HirExpr::FieldAccess { object, field, ty } => {
            if is_optional_or_recursive(object.ty())
                || matches!(
                    ty.resolve_alias(),
                    Type::Callable(..) | Type::AsyncCallable(..)
                )
                || is_recursive_field(object.ty(), field, ty)
            {
                return Err(InvalidPlace::Unsupported);
            }
            let mut place = extract_place(object, ctx)?;
            place
                .projections
                .push(PlaceProjection::Field(resolve_field_identity(
                    object.ty(),
                    field,
                    ctx,
                )));
            Ok(place)
        }
        _ => Err(InvalidPlace::Unsupported),
    }
}

fn resolve_field_identity(object_ty: &Type, field: &str, ctx: &LowerCtx) -> FieldIdentity {
    let Some((mut declaring, chain)) = nominal_class_and_parent_chain(object_ty) else {
        return FieldIdentity {
            declaring_class: object_ty.display_name(),
            field: field.to_string(),
        };
    };
    for parent in chain {
        let Some(parent_ty) = find_class_type(&parent, ctx) else {
            continue;
        };
        if class_has_field(parent_ty, field) {
            declaring = nominal_identity(parent_ty).unwrap_or(parent);
        }
    }
    FieldIdentity {
        declaring_class: declaring,
        field: field.to_string(),
    }
}

fn nominal_class_and_parent_chain(ty: &Type) -> Option<(String, Vec<String>)> {
    let Type::Class {
        identity,
        name,
        parent_class,
        ..
    } = ty.resolve_alias()
    else {
        return None;
    };
    Some((
        identity.clone().unwrap_or_else(|| name.clone()),
        parent_class
            .as_deref()
            .unwrap_or_default()
            .split('|')
            .filter(|part| !part.is_empty())
            .map(str::to_string)
            .collect(),
    ))
}

fn find_class_type<'a>(name_or_identity: &str, ctx: &'a LowerCtx) -> Option<&'a Type> {
    ctx.class_types.get(name_or_identity).or_else(|| {
        ctx.class_types
            .values()
            .find(|ty| nominal_identity(ty).is_some_and(|identity| identity == name_or_identity))
    })
}

fn nominal_identity(ty: &Type) -> Option<String> {
    let Type::Class { identity, name, .. } = ty.resolve_alias() else {
        return None;
    };
    Some(identity.clone().unwrap_or_else(|| name.clone()))
}

fn class_has_field(ty: &Type, field: &str) -> bool {
    matches!(
        ty.resolve_alias(),
        Type::Class { fields, .. } if fields.iter().any(|(candidate, _)| candidate == field)
    )
}

fn is_recursive_field(object_ty: &Type, field: &str, field_ty: &Type) -> bool {
    let Some((identity, _)) = nominal_class_and_parent_chain(object_ty) else {
        return false;
    };
    let declared_ty = match object_ty.resolve_alias() {
        Type::Class { fields, .. } => fields
            .iter()
            .find_map(|(candidate, ty)| (candidate == field).then_some(ty))
            .unwrap_or(field_ty),
        _ => field_ty,
    };
    contains_nominal_or_recursive_alias(declared_ty, &identity)
}

fn contains_nominal_or_recursive_alias(ty: &Type, target: &str) -> bool {
    match ty {
        Type::Class { identity, name, .. } => identity.as_deref().unwrap_or(name) == target,
        Type::Alias { name, body, .. } => {
            name == target || contains_nominal_or_recursive_alias(body, target)
        }
        Type::Union(items) => items
            .iter()
            .any(|item| contains_nominal_or_recursive_alias(item, target)),
        _ => false,
    }
}

fn is_optional_or_recursive(ty: &Type) -> bool {
    match ty {
        Type::Union(items) => {
            items
                .iter()
                .any(|item| matches!(item.resolve_alias(), Type::None))
                || items.iter().any(is_optional_or_recursive)
        }
        Type::Alias { body, .. } => {
            matches!(body.as_ref(), Type::Unknown) || is_optional_or_recursive(body)
        }
        _ => false,
    }
}

pub(super) fn is_owned_temporary(expr: &HirExpr) -> bool {
    match expr {
        HirExpr::IfExpr {
            then_expr,
            else_expr,
            ..
        } => is_owned_temporary(then_expr) && is_owned_temporary(else_expr),
        HirExpr::Call { .. }
        | HirExpr::GenericCall { .. }
        | HirExpr::PythonCall { .. }
        | HirExpr::IntrinsicCall { .. }
        | HirExpr::IteratorCall { .. }
        | HirExpr::MethodCall { .. }
        | HirExpr::ConstructorCall { .. }
        | HirExpr::SuperCall { .. }
        | HirExpr::Await { .. }
        | HirExpr::BinOp { .. }
        | HirExpr::UnaryOp { .. }
        | HirExpr::Compare { .. }
        | HirExpr::ContainsOp { .. }
        | HirExpr::RangeLiteral { .. }
        | HirExpr::Slice { .. }
        | HirExpr::ListLiteral { .. }
        | HirExpr::SetLiteral { .. }
        | HirExpr::DictLiteral { .. }
        | HirExpr::TupleLiteral { .. }
        | HirExpr::FString { .. }
        | HirExpr::TemplateString(_)
        | HirExpr::QuestionMark { .. }
        | HirExpr::OkWrap { .. }
        | HirExpr::ErrWrap { .. }
        | HirExpr::Lambda { .. }
        | HirExpr::ListComp { .. }
        | HirExpr::DictComp { .. }
        | HirExpr::SetComp { .. }
        | HirExpr::GeneratorExpr { .. }
        | HirExpr::EnumVariant { .. }
        | HirExpr::IntLiteral(_)
        | HirExpr::LargeIntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral => true,
        HirExpr::Name { .. }
        | HirExpr::BoolOp { .. }
        | HirExpr::Index { .. }
        | HirExpr::WalrusExpr { .. }
        | HirExpr::StructuralRecordProject { .. }
        | HirExpr::FieldAccess { .. } => false,
    }
}

fn report_immutable_root(expr: &HirExpr, range: TextRange, ctx: &mut LowerCtx) {
    let name = root_binding_id(expr)
        .and_then(|id| ctx.scope.retained_binding(id))
        .map(|fact| fact.name.clone())
        .or_else(|| root_binding_name(expr).map(str::to_string));
    if let Some(name) = name {
        ownership_diagnostics::immutable_parameter_mutation(ctx, &name, range);
    } else {
        ownership_diagnostics::unsupported_mutable_receiver_place(
            ctx,
            &place_display(expr, ctx),
            range,
        );
    }
}

fn root_binding_name(expr: &HirExpr) -> Option<&str> {
    match expr {
        HirExpr::Name { name, .. } => Some(name),
        HirExpr::FieldAccess { object, .. }
        | HirExpr::Index { object, .. }
        | HirExpr::Slice { object, .. } => root_binding_name(object),
        _ => None,
    }
}

fn place_display(expr: &HirExpr, ctx: &LowerCtx) -> String {
    extract_place(expr, ctx).map_or_else(
        |_| expr.ty().display_name(),
        |place| display_checked_place(&place, ctx),
    )
}

fn display_checked_place(place: &Place, ctx: &LowerCtx) -> String {
    let mut display = ctx
        .scope
        .retained_binding(place.root)
        .map_or_else(|| format!("#{}", place.root.0), |fact| fact.name.clone());
    for projection in &place.projections {
        let PlaceProjection::Field(field) = projection;
        display.push('.');
        display.push_str(&field.field);
    }
    display
}

fn root_binding_id(expr: &HirExpr) -> Option<BindingId> {
    match expr {
        HirExpr::Name { binding_id, .. } => *binding_id,
        HirExpr::FieldAccess { object, .. }
        | HirExpr::Index { object, .. }
        | HirExpr::Slice { object, .. } => root_binding_id(object),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(root: u32, fields: &[&str]) -> Place {
        Place {
            root: BindingId(root),
            projections: fields
                .iter()
                .map(|field| {
                    PlaceProjection::Field(FieldIdentity {
                        declaring_class: "Owner".to_string(),
                        field: (*field).to_string(),
                    })
                })
                .collect(),
        }
    }

    #[test]
    fn overlap_uses_binding_identity_and_projection_prefixes() {
        assert!(places_overlap(
            &place(1, &["helper"]),
            &place(1, &["helper", "items"])
        ));
        assert!(places_overlap(&place(1, &[]), &place(1, &["helper"])));
        assert!(!places_overlap(
            &place(1, &["helper"]),
            &place(1, &["other"])
        ));
        assert!(!places_overlap(
            &place(1, &["helper"]),
            &place(2, &["helper"])
        ));
    }

    #[test]
    fn recursive_nominal_field_is_not_a_supported_projection() {
        let node_ref = Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "Node".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        };
        let node = Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "Node".to_string(),
            fields: vec![(
                "next".to_string(),
                Type::Union(vec![node_ref.clone(), Type::None]),
            )],
            methods: Vec::new(),
            parent_class: None,
        };

        assert!(is_recursive_field(&node, "next", &node_ref));
    }
}
