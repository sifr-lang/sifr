use crate::hir_nodes::{HirStmt, HirTupleTarget, HirTupleTargetBinding};
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, ExprAttribute, ExprTuple};

use super::assignment_widening::reconcile_optional_reassignment;
use super::binding_mutability::ensure_mutable_parameter_binding;
use super::expressions::{consume_owned_value, lower_expr};
use super::len_aliases::record_tuple_unpack_len_alias_facts;
use super::name_diagnostics;
use super::sequence_pointers::record_tuple_unpack_pointer_facts;
use super::LowerCtx;

#[derive(Debug, Clone)]
enum TupleAssignTarget {
    Name { name: String, range: TextRange },
    Field { object: String, field: String },
}

fn reject_python_context_borrow_unpack(
    value: &crate::hir_nodes::HirExpr,
    range: TextRange,
    ctx: &mut LowerCtx,
) {
    if let Some(borrowed) = super::python_interop::python_context_borrow_in_owned_expr(value, ctx) {
        ctx.error_with_code_at(
            DiagnosticCode::PYCTX_INVALID_DECLARATION,
            format!(
                "invalid Python context declaration: entered binding '{borrowed}' is a context-scoped borrow and cannot escape through unpacking"
            ),
            range,
        );
    }
}

fn lower_tuple_target(elt: &Expr, ctx: &mut LowerCtx) -> Option<TupleAssignTarget> {
    match elt {
        Expr::Name(n) => Some(TupleAssignTarget::Name {
            name: n.id.to_string(),
            range: n.range(),
        }),
        Expr::Attribute(ExprAttribute { value, attr, .. }) => {
            let Expr::Name(object_name) = value.as_ref() else {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH,
                    "tuple unpacking attribute target must be rooted at a simple name".to_string(),
                    value.range(),
                );
                return None;
            };
            let object = object_name.id.to_string();
            if !ensure_mutable_parameter_binding(ctx, &object, object_name.range()) {
                return None;
            }
            Some(TupleAssignTarget::Field {
                object,
                field: attr.to_string(),
            })
        }
        _ => {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH,
                "tuple unpacking target must be a simple name or attribute".to_string(),
                elt.range(),
            );
            None
        }
    }
}

pub(in crate::lower) fn lower_tuple_unpack_assign(
    tuple: &ExprTuple,
    value: &Expr,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    let mut targets = Vec::new();
    let mut target_names = Vec::new();
    for elt in &tuple.elts {
        let target = lower_tuple_target(elt, ctx)?;
        if let TupleAssignTarget::Name { name, .. } = &target {
            target_names.push(name.clone());
        }
        targets.push(target);
    }

    let value_expr = lower_expr(value, ctx)?;
    reject_python_context_borrow_unpack(&value_expr, value.range(), ctx);
    let value_ty = value_expr.ty().clone();

    let elem_types = if let sifr_type_system::Type::Tuple(elems) = &value_ty {
        if elems.len() != targets.len() {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH,
                format!(
                    "tuple unpacking: expected {} values, got {}",
                    targets.len(),
                    elems.len()
                ),
                tuple.range(),
            );
            return None;
        }
        elems.clone()
    } else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH,
            format!("cannot unpack non-tuple type '{}'", value_ty.display_name()),
            value.range(),
        );
        return None;
    };
    consume_owned_value(&value_expr, value.range(), ctx);

    record_tuple_unpack_pointer_facts(ctx, &target_names, value);
    record_tuple_unpack_len_alias_facts(ctx, &target_names, value);
    let mut lowered_targets = Vec::new();
    for (target, ty) in targets.into_iter().zip(elem_types.into_iter()) {
        match target {
            TupleAssignTarget::Name { name, range } => {
                if ctx.is_declared_nonlocal(&name) {
                    super::flow_diagnostics::tuple_unpack_nonlocal_rebind(ctx, tuple.range());
                    return None;
                }
                let rebind_existing = if ctx.current_function_frame_start().is_some() {
                    super::nonlocal_support::should_rebind_simple_name(ctx, &name)
                } else {
                    ctx.scope.lookup(&name).is_some()
                };

                if rebind_existing {
                    let Some(info) = ctx.scope.lookup(&name) else {
                        name_diagnostics::undefined_variable(ctx, &name, range);
                        return None;
                    };
                    if info.is_parameter_binding() && !info.is_mutable_binding() {
                        super::ownership_diagnostics::immutable_parameter_reassignment(
                            ctx, &name, range,
                        );
                        return None;
                    }
                    let info_ty = info.ty.clone();
                    let can_widen = info.is_inferred_local_binding();
                    if !reconcile_optional_reassignment(ctx, &name, &info_ty, &ty, can_widen) {
                        ctx.error_with_code_at(
                            DiagnosticCode::TYPE_MISMATCH,
                            format!(
                                "type mismatch: cannot assign '{}' to variable '{}' of type '{}'",
                                ty.display_name(),
                                name,
                                info_ty.display_name()
                            ),
                            range,
                        );
                    }
                    ctx.reset_moved_with_flow(&name);
                    ctx.clear_narrowing_with_flow(&name);
                    ctx.clear_sequence_guards_for_binding(&name);
                } else {
                    ctx.scope.define(name.clone(), ty.clone());
                }
                ctx.clear_sequence_shape_fact(&name);
                ctx.empty_dict_specializations.remove(&name);
                ctx.pending_container_specialization_patches.remove(&name);

                lowered_targets.push(HirTupleTarget {
                    binding: HirTupleTargetBinding::Name(name),
                    ty,
                    rebind_existing,
                });
            }
            TupleAssignTarget::Field { object, field } => {
                lowered_targets.push(HirTupleTarget {
                    binding: HirTupleTargetBinding::Field { object, field },
                    ty,
                    rebind_existing: false,
                });
            }
        }
    }

    Some(HirStmt::TupleUnpack {
        targets: lowered_targets,
        value: value_expr,
    })
}

pub(in crate::lower) fn lower_star_unpack_assign(
    tuple: &ExprTuple,
    value: &Expr,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    let value_expr = lower_expr(value, ctx)?;
    reject_python_context_borrow_unpack(&value_expr, value.range(), ctx);
    let value_ty = value_expr.ty().clone();

    let elem_ty = if let sifr_type_system::Type::List(elem) = &value_ty {
        *elem.clone()
    } else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH,
            "star unpacking requires a list type".to_string(),
            value.range(),
        );
        return None;
    };
    if elem_ty.contains_affine_resource() {
        ctx.error_with_code_at(
            DiagnosticCode::PYZC_INVALID_DECLARATION,
            "star unpacking is unavailable for elements containing affine Python buffers because it clones projected values"
                .to_string(),
            value.range(),
        );
        return None;
    }

    let mut before = Vec::new();
    let mut star: Option<(String, sifr_type_system::Type)> = None;
    let mut after = Vec::new();

    for elt in &tuple.elts {
        match elt {
            Expr::Starred(starred) => {
                if star.is_some() {
                    ctx.error_with_code_at(
                        DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH,
                        "multiple starred expressions in assignment".to_string(),
                        starred.range(),
                    );
                    return None;
                }
                if let Expr::Name(n) = starred.value.as_ref() {
                    let name = n.id.to_string();
                    let star_ty = sifr_type_system::Type::List(Box::new(elem_ty.clone()));
                    ctx.scope.define(name.clone(), star_ty.clone());
                    star = Some((name, star_ty));
                } else {
                    ctx.error_with_code_at(
                        DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH,
                        "starred target must be a simple name".to_string(),
                        starred.value.range(),
                    );
                    return None;
                }
            }
            Expr::Name(n) => {
                let name = n.id.to_string();
                ctx.scope.define(name.clone(), elem_ty.clone());
                if star.is_none() {
                    before.push((name, elem_ty.clone()));
                } else {
                    after.push((name, elem_ty.clone()));
                }
            }
            _ => {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH,
                    "star unpacking target must be a simple name".to_string(),
                    elt.range(),
                );
                return None;
            }
        }
    }

    let Some(star) = star else {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_UNPACK_SHAPE_MISMATCH,
            "star unpacking requires a starred expression".to_string(),
            tuple.range(),
        );
        return None;
    };

    Some(HirStmt::StarUnpack {
        before,
        star,
        after,
        value: value_expr,
    })
}
