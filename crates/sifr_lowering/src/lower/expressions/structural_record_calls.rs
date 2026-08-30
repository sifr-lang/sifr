use super::{
    DiagnosticCode, Expr, ExprCall, HashMap, HirExpr, LowerCtx, ParamConvention, Ranged, Type,
    consume_owned_value, lower_expr, lower_name,
};
use crate::lower::typing_and_functions::resolve_annotation_expr;
use ruff_text_size::TextRange;
use sifr_python_ast::{ExprAttribute, ExprSubscript};

pub(super) fn try_lower_structural_record_subscript_call(
    subscript: &ExprSubscript,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<Option<HirExpr>> {
    try_lower_generic_structural_record_constructor(subscript, call, ctx)
        .or_else(|| lower_structural_record_projection(subscript, call, ctx))
}

fn try_lower_generic_structural_record_constructor(
    subscript: &ExprSubscript,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<Option<HirExpr>> {
    let Expr::Name(alias) = subscript.value.as_ref() else {
        return None;
    };
    ctx.scope.lookup_generic_type_alias(alias.id.as_str())?;
    let alias_ty = resolve_annotation_expr(call.func.as_ref(), ctx);
    if !matches!(alias_ty.resolve_alias(), Type::StructuralRecord(_)) {
        return None;
    }
    let alias_name = alias_ty.display_name();
    Some(lower_structural_record_constructor(
        &alias_name,
        alias_ty,
        call,
        ctx,
    ))
}

pub(super) fn lower_structural_record_field_access(
    object: HirExpr,
    object_ty: &Type,
    resolved_object_ty: &Type,
    field_name: &str,
    attr: &ExprAttribute,
    ctx: &mut LowerCtx,
) -> Option<Option<HirExpr>> {
    let Type::StructuralRecord(record) = resolved_object_ty else {
        return None;
    };
    if let Some(field) = record.field(field_name) {
        if field.ty().contains_affine_resource() {
            ctx.error_with_code_at(
                DiagnosticCode::PYZC_INVALID_DECLARATION,
                "cannot project a field containing an affine Python resource; move the record as a whole"
                    .to_string(),
                attr.range(),
            );
            return Some(None);
        }
        return Some(Some(HirExpr::FieldAccess {
            object: Box::new(object),
            field: field_name.to_string(),
            ty: field.ty().clone(),
        }));
    }
    ctx.error_with_code_at(
        DiagnosticCode::CLASS_MISSING_MEMBER,
        format!(
            "record type '{}' has no field '{}'",
            object_ty.display_name(),
            field_name
        ),
        attr.attr.range(),
    );
    Some(None)
}

fn lower_structural_record_projection(
    subscript: &ExprSubscript,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<Option<HirExpr>> {
    let Expr::Attribute(attribute) = subscript.value.as_ref() else {
        return None;
    };
    if attribute.attr.as_str() != "project" {
        return None;
    }
    if !call.arguments.args.is_empty() || !call.arguments.keywords.is_empty() {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "record project() takes no runtime arguments".to_string(),
            call.range(),
        );
        return Some(None);
    }
    let Expr::Name(source_name) = attribute.value.as_ref() else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "record project() requires an owned local variable receiver".to_string(),
            attribute.value.range(),
        );
        return Some(None);
    };
    if ctx.borrowed_params.contains(source_name.id.as_str()) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!("cannot consume borrowed record '{}'", source_name.id),
            source_name.range(),
        );
        return Some(None);
    }
    let Some(source) = lower_name(source_name, ctx) else {
        return Some(None);
    };
    let Type::StructuralRecord(source_record) = source.ty().resolve_alias() else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "type '{}' does not support project()",
                source.ty().display_name()
            ),
            attribute.range(),
        );
        return Some(None);
    };
    let source_record = source_record.clone();
    let target_ty = resolve_annotation_expr(&subscript.slice, ctx);
    let Type::StructuralRecord(target_record) = target_ty.resolve_alias() else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "record project target must be a structural record type".to_string(),
            subscript.slice.range(),
        );
        return Some(None);
    };
    let target_record = target_record.clone();
    if source_record == target_record || !source_record.is_width_subtype_of(&target_record) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "project target '{}' must be a strict field subset of '{}'",
                target_ty.display_name(),
                source.ty().display_name()
            ),
            subscript.slice.range(),
        );
        return Some(None);
    }
    let fields = target_record
        .fields()
        .iter()
        .map(|field| field.name().to_string())
        .collect();
    consume_owned_value(&source, source_name.range(), ctx);
    Some(Some(HirExpr::StructuralRecordProject {
        source: Box::new(source),
        fields,
        ty: target_ty,
    }))
}

pub(super) fn lower_structural_record_constructor(
    alias_name: &str,
    alias_ty: Type,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    let Type::StructuralRecord(record) = alias_ty.resolve_alias() else {
        unreachable!("record constructor dispatch requires a structural record alias");
    };
    let record = record.clone();
    if !call.arguments.args.is_empty() {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!("record constructor '{alias_name}' accepts named fields only"),
            call.arguments.args[0].range(),
        );
        return None;
    }

    let mut provided = HashMap::new();
    for keyword in &call.arguments.keywords {
        let Some(name) = keyword.arg.as_ref() else {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!("record constructor '{alias_name}' does not accept ** expansion"),
                keyword.range(),
            );
            return None;
        };
        let name = name.to_string();
        let Some(field) = record.field(&name) else {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!("record constructor '{alias_name}' has no field '{name}'"),
                keyword.range(),
            );
            return None;
        };
        if provided.contains_key(&name) {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!("record constructor '{alias_name}' got field '{name}' more than once"),
                keyword.range(),
            );
            return None;
        }
        let value = lower_expr(&keyword.value, ctx)?;
        if !value.ty().is_assignable_to(field.ty()) {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "field '{name}' of record '{alias_name}': expected '{}', got '{}'",
                    field.ty().display_name(),
                    value.ty().display_name()
                ),
                keyword.value.range(),
            );
            return None;
        }
        consume_owned_value(&value, keyword.value.range(), ctx);
        provided.insert(name, value);
    }

    let missing = record
        .source_fields()
        .into_iter()
        .filter(|field| !provided.contains_key(field.name()))
        .map(|field| field.name().to_string())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "record constructor '{alias_name}' is missing field{} {}",
                if missing.len() == 1 { "" } else { "s" },
                missing.join(", ")
            ),
            call.range(),
        );
        return None;
    }

    let args = record
        .fields()
        .iter()
        .filter_map(|field| provided.remove(field.name()))
        .collect();
    Some(HirExpr::ConstructorCall {
        class_name: alias_name.to_string(),
        args,
        ty: alias_ty,
    })
}

pub(super) fn validate_borrowed_structural_coercion(
    source_ty: &Type,
    target_ty: &Type,
    convention: ParamConvention,
    range: TextRange,
    ctx: &mut LowerCtx,
) {
    let source = source_ty.resolve_alias();
    let target = target_ty.resolve_alias();
    if let (Type::StructuralRecord(source_record), Type::StructuralRecord(target_record)) =
        (source, target)
        && source_record != target_record
        && source_record.is_width_subtype_of(target_record)
    {
        if !convention.is_shared_borrow() {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                "record width subtyping is available only for shared-borrow parameters".to_string(),
                range,
            );
        }
        return;
    }
    if source == target
        || !source_ty.is_assignable_to(target_ty)
        || !matches!(target, Type::Union(_) | Type::Result(_, _))
    {
        return;
    }
    if convention.is_mut_borrow() {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "mutable borrow cannot change the generated representation from '{}' to '{}'",
                source_ty.display_name(),
                target_ty.display_name()
            ),
            range,
        );
    } else if convention.is_shared_borrow() && !source_ty.supports_derived_clone() {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "borrowed conversion from '{}' to '{}' requires a cloneable source representation",
                source_ty.display_name(),
                target_ty.display_name()
            ),
            range,
        );
    }
}
