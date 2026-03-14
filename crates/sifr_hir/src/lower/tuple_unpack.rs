use crate::hir_nodes::{HirStmt, HirTupleTarget};
use sifr_python_ast::{Expr, ExprTuple};

use super::expressions::lower_expr;
use super::sequence_pointers::record_tuple_unpack_pointer_facts;
use super::LowerCtx;

pub(super) fn lower_tuple_unpack_assign(
    tuple: &ExprTuple,
    value: &Expr,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    let mut target_names = Vec::new();
    for elt in &tuple.elts {
        if let Expr::Name(n) = elt {
            target_names.push(n.id.clone());
        } else {
            ctx.error("tuple unpacking target must be a simple name".to_string());
            return None;
        }
    }

    let value_expr = lower_expr(value, ctx)?;
    let value_ty = value_expr.ty().clone();

    let elem_types = if let sifr_type_system::Type::Tuple(elems) = &value_ty {
        if elems.len() != target_names.len() {
            ctx.error(format!(
                "tuple unpacking: expected {} values, got {}",
                target_names.len(),
                elems.len()
            ));
            return None;
        }
        elems.clone()
    } else {
        ctx.error(format!(
            "cannot unpack non-tuple type '{}'",
            value_ty.display_name()
        ));
        return None;
    };

    let mut targets = Vec::new();
    record_tuple_unpack_pointer_facts(ctx, &target_names, value);
    for (name, ty) in target_names.into_iter().zip(elem_types.into_iter()) {
        if ctx.is_declared_nonlocal(&name) {
            ctx.error(
                "tuple unpacking cannot rebind captured state with `nonlocal` yet".to_string(),
            );
            return None;
        }
        let rebind_existing = if ctx.current_function_frame_start().is_some() {
            super::nonlocal_support::should_rebind_simple_name(ctx, &name)
        } else {
            ctx.scope.lookup(&name).is_some()
        };

        if rebind_existing {
            let Some(info) = ctx.scope.lookup(&name) else {
                ctx.error(format!("undefined variable: '{name}'"));
                return None;
            };
            if info.is_parameter_binding() && !info.is_mutable_binding {
                ctx.error(format!(
                    "cannot reassign immutable parameter `{name}`: add `mut` to the parameter declaration"
                ));
                return None;
            }
            if !ty.is_assignable_to(&info.ty) {
                ctx.error(format!(
                    "type mismatch: cannot assign '{}' to variable '{}' of type '{}'",
                    ty.display_name(),
                    name,
                    info.ty.display_name()
                ));
            }
            ctx.scope.reset_moved(&name);
        } else {
            ctx.scope.define(name.clone(), ty.clone());
        }

        targets.push(HirTupleTarget {
            name,
            ty,
            rebind_existing,
        });
    }

    Some(HirStmt::TupleUnpack {
        targets,
        value: value_expr,
    })
}

pub(super) fn lower_star_unpack_assign(
    tuple: &ExprTuple,
    value: &Expr,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    let value_expr = lower_expr(value, ctx)?;
    let value_ty = value_expr.ty().clone();

    let elem_ty = if let sifr_type_system::Type::List(elem) = &value_ty {
        *elem.clone()
    } else {
        ctx.error("star unpacking requires a list type".to_string());
        return None;
    };

    let mut before = Vec::new();
    let mut star: Option<(String, sifr_type_system::Type)> = None;
    let mut after = Vec::new();

    for elt in &tuple.elts {
        match elt {
            Expr::Starred(starred) => {
                if star.is_some() {
                    ctx.error("multiple starred expressions in assignment".to_string());
                    return None;
                }
                if let Expr::Name(n) = starred.value.as_ref() {
                    let name = n.id.clone();
                    let star_ty = sifr_type_system::Type::List(Box::new(elem_ty.clone()));
                    ctx.scope.define(name.clone(), star_ty.clone());
                    star = Some((name, star_ty));
                } else {
                    ctx.error("starred target must be a simple name".to_string());
                    return None;
                }
            }
            Expr::Name(n) => {
                let name = n.id.clone();
                ctx.scope.define(name.clone(), elem_ty.clone());
                if star.is_none() {
                    before.push((name, elem_ty.clone()));
                } else {
                    after.push((name, elem_ty.clone()));
                }
            }
            _ => {
                ctx.error("star unpacking target must be a simple name".to_string());
                return None;
            }
        }
    }

    let star = star.unwrap_or_else(|| {
        ctx.error("star unpacking requires a starred expression".to_string());
        (
            "_".to_string(),
            sifr_type_system::Type::List(Box::new(elem_ty.clone())),
        )
    });

    Some(HirStmt::StarUnpack {
        before,
        star,
        after,
        value: value_expr,
    })
}
