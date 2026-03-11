use crate::hir_nodes::HirExpr;
use sifr_python_ast::{Expr, ExprCall};
use sifr_type_system::Type;

use super::expressions::lower_expr;
use super::LowerCtx;

fn set_constructor_element_type(arg_ty: &Type) -> Option<Type> {
    match arg_ty.resolve_alias() {
        Type::List(elem) | Type::Set(elem) => Some(*elem.clone()),
        Type::Tuple(elems) => {
            if elems.is_empty() {
                Some(Type::Any)
            } else {
                let first = elems[0].clone();
                if elems.iter().all(|elem| elem == &first) {
                    Some(first)
                } else {
                    Some(Type::Any)
                }
            }
        }
        Type::Any | Type::Unknown => Some(Type::Any),
        _ => None,
    }
}

pub(super) fn lower_set_constructor_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.keywords.len() > 1 {
        ctx.error("set() accepts at most one keyword argument".to_string());
        return None;
    }
    if let Some(keyword) = call.arguments.keywords.first() {
        let Some(name) = keyword.arg.as_ref() else {
            ctx.error("set() does not support unpacked keyword arguments".to_string());
            return None;
        };
        if name.as_str() != "iterable" {
            ctx.error(format!("set() got an unexpected keyword argument '{name}'"));
            return None;
        }
    }

    let mut positional_args = Vec::with_capacity(call.arguments.args.len());
    for arg in &call.arguments.args {
        positional_args.push(lower_expr(arg, ctx)?);
    }

    let iterable_arg = if let Some(keyword) = call.arguments.keywords.first() {
        Some(lower_expr(&keyword.value, ctx)?)
    } else {
        None
    };

    let arg = match (positional_args.len(), iterable_arg) {
        (0, None) => None,
        (1, None) => positional_args.into_iter().next(),
        (0, Some(arg)) => Some(arg),
        _ => {
            ctx.error("set() takes at most 1 argument".to_string());
            return None;
        }
    };

    match arg {
        None => Some(HirExpr::Call {
            func: "set".to_string(),
            args: Vec::new(),
            ty: Type::Set(Box::new(Type::Any)),
        }),
        Some(iterable) => {
            let Some(elem_ty) = set_constructor_element_type(iterable.ty()) else {
                ctx.error(format!(
                    "set() argument must be a list, set, tuple, or compatible iterable, got '{}'",
                    iterable.ty().display_name()
                ));
                return None;
            };
            Some(HirExpr::Call {
                func: "set".to_string(),
                args: vec![iterable],
                ty: Type::Set(Box::new(elem_ty)),
            })
        }
    }
}

pub(super) fn lower_len_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() != 1 {
        ctx.error(format!(
            "len() takes exactly 1 argument, got {}",
            call.arguments.args.len()
        ));
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let arg_ty = arg.ty().clone();

    let effective_ty = if let Type::Union(members) = &arg_ty {
        let non_none: Vec<&Type> = members
            .iter()
            .filter(|m| !matches!(m, Type::None))
            .collect();
        if non_none.len() == 1 {
            non_none[0].clone()
        } else {
            arg_ty.clone()
        }
    } else {
        arg_ty.clone()
    };
    match &effective_ty {
        Type::Str | Type::List(_) | Type::Dict(_, _) | Type::Tuple(_) | Type::Set(_) => {
            Some(HirExpr::MethodCall {
                object: Box::new(arg),
                method: "len".to_string(),
                args: vec![],
                ty: Type::Int,
            })
        }
        _ => {
            ctx.error(format!(
                "len() argument must be a string, list, dict, or tuple, got '{}'",
                arg_ty.display_name()
            ));
            None
        }
    }
}

pub(super) fn lower_isinstance_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() != 2 {
        ctx.error(format!(
            "isinstance() takes exactly 2 arguments, got {}",
            call.arguments.args.len()
        ));
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let type_name = match &call.arguments.args[1] {
        Expr::Name(n) => n.id.clone(),
        _ => "unknown".to_string(),
    };
    Some(HirExpr::Call {
        func: "isinstance".to_string(),
        args: vec![arg, HirExpr::StringLiteral(type_name)],
        ty: Type::Bool,
    })
}

pub(super) fn lower_reveal_type_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if call.arguments.args.len() != 1 {
        ctx.error(format!(
            "reveal_type() takes exactly 1 argument, got {}",
            call.arguments.args.len()
        ));
        return None;
    }
    let arg = lower_expr(&call.arguments.args[0], ctx)?;
    let ty = arg.ty().clone();
    ctx.reveal_types
        .push(format!("reveal_type: {}", ty.display_name()));
    Some(arg)
}

pub(super) fn lower_range_call(call: &ExprCall, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let args: Vec<_> = call.arguments.args.iter().collect();

    match args.len() {
        1 => {
            let end = lower_expr(args[0], ctx)?;
            if end.ty() != &Type::Int {
                ctx.error(format!(
                    "range() argument must be 'int', got '{}'",
                    end.ty().display_name()
                ));
                return None;
            }
            Some(HirExpr::RangeLiteral {
                start: Box::new(HirExpr::IntLiteral(0)),
                end: Box::new(end),
                step: None,
                ty: Type::Range,
            })
        }
        2 => {
            let start = lower_expr(args[0], ctx)?;
            let end = lower_expr(args[1], ctx)?;
            if start.ty() != &Type::Int {
                ctx.error(format!(
                    "range() start argument must be 'int', got '{}'",
                    start.ty().display_name()
                ));
                return None;
            }
            if end.ty() != &Type::Int {
                ctx.error(format!(
                    "range() end argument must be 'int', got '{}'",
                    end.ty().display_name()
                ));
                return None;
            }
            Some(HirExpr::RangeLiteral {
                start: Box::new(start),
                end: Box::new(end),
                step: None,
                ty: Type::Range,
            })
        }
        3 => {
            let start = lower_expr(args[0], ctx)?;
            let end = lower_expr(args[1], ctx)?;
            let step = lower_expr(args[2], ctx)?;
            Some(HirExpr::RangeLiteral {
                start: Box::new(start),
                end: Box::new(end),
                step: Some(Box::new(step)),
                ty: Type::Range,
            })
        }
        _ => {
            ctx.error(format!(
                "range() takes 1, 2, or 3 arguments, got {}",
                args.len()
            ));
            None
        }
    }
}
