use crate::hir_analysis::traversal::{self, TraversalConfig};
use crate::{HirExpr, HirFunction, Type};
use std::collections::HashSet;

pub(crate) fn function_calls_itself(func: &HirFunction) -> bool {
    let mut recursive = false;
    traversal::walk_stmts(
        &func.body,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut |_| {},
        &mut |expr| {
            if let HirExpr::Call { func: callee, .. } = expr {
                recursive |= callee == &func.name;
            }
        },
    );
    recursive
}

pub(crate) fn collect_string_cache_uses(expr: &HirExpr, used: &mut HashSet<String>) {
    match expr {
        HirExpr::Index { object, .. } if matches!(object.ty().resolve_alias(), Type::Str) => {
            if let HirExpr::Name { name, .. } = object.as_ref() {
                used.insert(name.clone());
            }
        }
        HirExpr::Slice { object, .. } if matches!(object.ty().resolve_alias(), Type::Str) => {
            if let HirExpr::Name { name, .. } = object.as_ref() {
                used.insert(name.clone());
            }
        }
        HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } if method == "len"
            && args.is_empty()
            && matches!(object.ty().resolve_alias(), Type::Str | Type::LiteralStr(_)) =>
        {
            if let HirExpr::Name { name, .. } = object.as_ref() {
                used.insert(name.clone());
            }
        }
        _ => {}
    }

    match expr {
        HirExpr::BinOp { left, right, .. } => {
            collect_string_cache_uses(left, used);
            collect_string_cache_uses(right, used);
        }
        HirExpr::UnaryOp { operand, .. }
        | HirExpr::Await { value: operand, .. }
        | HirExpr::QuestionMark { expr: operand, .. }
        | HirExpr::OkWrap { value: operand, .. }
        | HirExpr::ErrWrap { value: operand, .. } => collect_string_cache_uses(operand, used),
        HirExpr::Compare {
            left, comparators, ..
        } => {
            collect_string_cache_uses(left, used);
            for comparator in comparators {
                collect_string_cache_uses(comparator, used);
            }
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
            for value in values {
                collect_string_cache_uses(value, used);
            }
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            collect_string_cache_uses(condition, used);
            collect_string_cache_uses(then_expr, used);
            collect_string_cache_uses(else_expr, used);
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            collect_string_cache_uses(start, used);
            collect_string_cache_uses(end, used);
            if let Some(step) = step {
                collect_string_cache_uses(step, used);
            }
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            for value in keys.iter().chain(values.iter()) {
                collect_string_cache_uses(value, used);
            }
        }
        HirExpr::Index { object, index, .. } => {
            collect_string_cache_uses(object, used);
            collect_string_cache_uses(index, used);
        }
        HirExpr::MethodCall { object, args, .. } => {
            collect_string_cache_uses(object, used);
            for arg in args {
                collect_string_cache_uses(arg, used);
            }
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => {
            collect_string_cache_uses(element, used);
            collect_string_cache_uses(collection, used);
        }
        HirExpr::FString { parts, .. } => {
            for part in parts {
                if let sifr_ir::HirFStringPart::Expr(expr) = part {
                    collect_string_cache_uses(expr, used);
                }
            }
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            collect_string_cache_uses(object, used);
            if let Some(start) = start {
                collect_string_cache_uses(start, used);
            }
            if let Some(stop) = stop {
                collect_string_cache_uses(stop, used);
            }
            if let Some(step) = step {
                collect_string_cache_uses(step, used);
            }
        }
        HirExpr::WalrusExpr { value, .. }
        | HirExpr::FieldAccess { object: value, .. }
        | HirExpr::Lambda { body: value, .. } => collect_string_cache_uses(value, used),
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            collect_string_cache_uses(expr, used);
            for (_, iter, filter) in generators {
                collect_string_cache_uses(iter, used);
                if let Some(filter) = filter {
                    collect_string_cache_uses(filter, used);
                }
            }
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            collect_string_cache_uses(key_expr, used);
            collect_string_cache_uses(val_expr, used);
            for (_, iter, filter) in generators {
                collect_string_cache_uses(iter, used);
                if let Some(filter) = filter {
                    collect_string_cache_uses(filter, used);
                }
            }
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            collect_string_cache_uses(expr, used);
            collect_string_cache_uses(iter, used);
            if let Some(filter) = filter {
                collect_string_cache_uses(filter, used);
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
