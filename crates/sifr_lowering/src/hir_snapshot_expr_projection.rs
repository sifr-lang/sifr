use crate::{
    HirAsyncWithKind, HirExpr, HirFStringPart, HirIteratorOp, HirPattern, HirTupleTargetBinding,
};
use serde_json::{json, Value};
use sifr_type_system::Type;

pub(super) fn project_expr(expr: &HirExpr) -> Value {
    match expr {
        HirExpr::IntLiteral(_) => scalar_expr("IntLiteral", expr),
        HirExpr::LargeIntLiteral(_) => scalar_expr("LargeIntLiteral", expr),
        HirExpr::FloatLiteral(_) => scalar_expr("FloatLiteral", expr),
        HirExpr::StringLiteral(_) => scalar_expr("StringLiteral", expr),
        HirExpr::BoolLiteral(_) => scalar_expr("BoolLiteral", expr),
        HirExpr::NoneLiteral => scalar_expr("NoneLiteral", expr),
        HirExpr::Name {
            name, binding_id, ..
        } => json!({
            "kind": "Name",
            "ty": expr_type_name(expr),
            "name": name,
            "binding_id": binding_id.map(|id| id.0),
        }),
        HirExpr::BinOp {
            left, op, right, ..
        } => json!({
            "kind": "BinOp",
            "ty": expr_type_name(expr),
            "op": op,
            "left": project_expr(left),
            "right": project_expr(right),
        }),
        HirExpr::UnaryOp { op, operand, .. } => json!({
            "kind": "UnaryOp",
            "ty": expr_type_name(expr),
            "op": op,
            "operand": project_expr(operand),
        }),
        HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } => json!({
            "kind": "Compare",
            "ty": expr_type_name(expr),
            "ops": ops,
            "left": project_expr(left),
            "comparators": comparators.iter().map(project_expr).collect::<Vec<_>>(),
        }),
        HirExpr::BoolOp { op, values, .. } => json!({
            "kind": "BoolOp",
            "ty": expr_type_name(expr),
            "op": op,
            "values": values.iter().map(project_expr).collect::<Vec<_>>(),
        }),
        HirExpr::Call { func, args, .. } => json!({
            "kind": "Call",
            "ty": expr_type_name(expr),
            "func": func,
            "args": args.iter().map(project_expr).collect::<Vec<_>>(),
        }),
        HirExpr::GenericCall {
            func,
            type_args,
            args,
            ..
        } => json!({
            "kind": "GenericCall",
            "ty": expr_type_name(expr),
            "func": func,
            "type_args": type_args.iter().map(Type::display_name).collect::<Vec<_>>(),
            "args": args.iter().map(project_expr).collect::<Vec<_>>(),
        }),
        HirExpr::PythonCall {
            func,
            args,
            provided_arguments,
            record_expansions,
            ..
        } => json!({
            "kind": "PythonCall",
            "ty": expr_type_name(expr),
            "func": func,
            "args": args.iter().map(project_expr).collect::<Vec<_>>(),
            "provided_arguments": provided_arguments,
            "record_fields": record_expansions.iter().map(|expansion| expansion.fields.clone()).collect::<Vec<_>>(),
        }),
        HirExpr::IntrinsicCall {
            intrinsic, args, ..
        } => json!({
            "kind": "IntrinsicCall",
            "ty": expr_type_name(expr),
            "intrinsic": intrinsic.declaration_name(),
            "args": args.iter().map(project_expr).collect::<Vec<_>>(),
        }),
        HirExpr::Await { value, .. } => json!({
            "kind": "Await",
            "ty": expr_type_name(expr),
            "value": project_expr(value),
        }),
        HirExpr::IteratorCall { op, args, .. } => json!({
            "kind": "IteratorCall",
            "ty": expr_type_name(expr),
            "op": iterator_op_name(op),
            "args": args.iter().map(project_expr).collect::<Vec<_>>(),
        }),
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => json!({
            "kind": "IfExpr",
            "ty": expr_type_name(expr),
            "condition": project_expr(condition),
            "then_expr": project_expr(then_expr),
            "else_expr": project_expr(else_expr),
        }),
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => json!({
            "kind": "RangeLiteral",
            "ty": expr_type_name(expr),
            "start": project_expr(start),
            "end": project_expr(end),
            "step": step.as_deref().map(project_expr),
        }),
        HirExpr::ListLiteral { elements, .. } => collection_expr("ListLiteral", expr, elements),
        HirExpr::SetLiteral { elements, .. } => collection_expr("SetLiteral", expr, elements),
        HirExpr::DictLiteral { keys, values, .. } => json!({
            "kind": "DictLiteral",
            "ty": expr_type_name(expr),
            "keys": keys.iter().map(project_expr).collect::<Vec<_>>(),
            "values": values.iter().map(project_expr).collect::<Vec<_>>(),
        }),
        HirExpr::TupleLiteral { elements, .. } => collection_expr("TupleLiteral", expr, elements),
        HirExpr::Index { object, index, .. } => json!({
            "kind": "Index",
            "ty": expr_type_name(expr),
            "object": project_expr(object),
            "index": project_expr(index),
        }),
        HirExpr::MethodCall {
            object,
            method,
            args,
            receiver_convention,
            source,
            ..
        } => json!({
            "kind": "MethodCall",
            "ty": expr_type_name(expr),
            "object": project_expr(object),
            "method": method,
            "args": args.iter().map(project_expr).collect::<Vec<_>>(),
            "receiver_convention": receiver_convention.map(|receiver| format!("{receiver:?}")),
            "source": source.as_ref().map(|source| json!({
                "call": [u32::from(source.call_range.start()), u32::from(source.call_range.end())],
                "receiver": [u32::from(source.receiver_range.start()), u32::from(source.receiver_range.end())],
                "args": source.arg_ranges.iter().map(|range| {
                    [u32::from(range.start()), u32::from(range.end())]
                }).collect::<Vec<_>>(),
            })),
        }),
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => json!({
            "kind": "ContainsOp",
            "ty": expr_type_name(expr),
            "element": project_expr(element),
            "collection": project_expr(collection),
        }),
        HirExpr::FString { parts, .. } => json!({
            "kind": "FString",
            "ty": expr_type_name(expr),
            "parts": parts.iter().map(project_f_string_part).collect::<Vec<_>>(),
        }),
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => json!({
            "kind": "Slice",
            "ty": expr_type_name(expr),
            "object": project_expr(object),
            "start": start.as_deref().map(project_expr),
            "stop": stop.as_deref().map(project_expr),
            "step": step.as_deref().map(project_expr),
        }),
        HirExpr::WalrusExpr { name, value, .. } => json!({
            "kind": "WalrusExpr",
            "ty": expr_type_name(expr),
            "name": name,
            "value": project_expr(value),
        }),
        HirExpr::FieldAccess { object, field, .. } => json!({
            "kind": "FieldAccess",
            "ty": expr_type_name(expr),
            "object": project_expr(object),
            "field": field,
        }),
        HirExpr::ConstructorCall {
            class_name, args, ..
        } => json!({
            "kind": "ConstructorCall",
            "ty": expr_type_name(expr),
            "class_name": class_name,
            "args": args.iter().map(project_expr).collect::<Vec<_>>(),
        }),
        HirExpr::QuestionMark { expr: inner, .. } => json!({
            "kind": "QuestionMark",
            "ty": expr_type_name(expr),
            "expr": project_expr(inner),
        }),
        HirExpr::OkWrap { value, .. } => json!({
            "kind": "OkWrap",
            "ty": expr_type_name(expr),
            "value": project_expr(value),
        }),
        HirExpr::ErrWrap { value, .. } => json!({
            "kind": "ErrWrap",
            "ty": expr_type_name(expr),
            "value": project_expr(value),
        }),
        HirExpr::SuperCall {
            parent_class,
            method,
            args,
            ..
        } => json!({
            "kind": "SuperCall",
            "ty": expr_type_name(expr),
            "parent_class": parent_class,
            "method": method,
            "args": args.iter().map(project_expr).collect::<Vec<_>>(),
        }),
        HirExpr::Lambda { params, body, .. } => json!({
            "kind": "Lambda",
            "ty": expr_type_name(expr),
            "params": params
                .iter()
                .map(|param| {
                    json!({
                        "name": param.name,
                        "ty": type_name(&param.ty),
                    })
                })
                .collect::<Vec<_>>(),
            "body": project_expr(body),
        }),
        HirExpr::ListComp {
            expr: element_expr,
            generators,
            ..
        } => comprehension_expr("ListComp", expr, element_expr, generators),
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => json!({
            "kind": "DictComp",
            "ty": expr_type_name(expr),
            "key_expr": project_expr(key_expr),
            "val_expr": project_expr(val_expr),
            "generators": project_generators(generators),
        }),
        HirExpr::SetComp {
            expr: element_expr,
            generators,
            ..
        } => comprehension_expr("SetComp", expr, element_expr, generators),
        HirExpr::GeneratorExpr {
            expr: element_expr,
            var,
            iter,
            filter,
            ..
        } => json!({
            "kind": "GeneratorExpr",
            "ty": expr_type_name(expr),
            "expr": project_expr(element_expr),
            "var": var,
            "iter": project_expr(iter),
            "filter": filter.as_deref().map(project_expr),
        }),
        HirExpr::EnumVariant {
            enum_name, variant, ..
        } => json!({
            "kind": "EnumVariant",
            "ty": expr_type_name(expr),
            "enum_name": enum_name,
            "variant": variant,
        }),
    }
}

fn scalar_expr(kind: &str, expr: &HirExpr) -> Value {
    json!({
        "kind": kind,
        "ty": expr_type_name(expr),
    })
}

fn collection_expr(kind: &str, expr: &HirExpr, elements: &[HirExpr]) -> Value {
    json!({
        "kind": kind,
        "ty": expr_type_name(expr),
        "elements": elements.iter().map(project_expr).collect::<Vec<_>>(),
    })
}

fn comprehension_expr(
    kind: &str,
    expr: &HirExpr,
    element_expr: &HirExpr,
    generators: &[(String, HirExpr, Option<HirExpr>)],
) -> Value {
    json!({
        "kind": kind,
        "ty": expr_type_name(expr),
        "expr": project_expr(element_expr),
        "generators": project_generators(generators),
    })
}

fn project_generators(generators: &[(String, HirExpr, Option<HirExpr>)]) -> Vec<Value> {
    generators
        .iter()
        .map(|(var, iter, filter)| {
            json!({
                "var": var,
                "iter": project_expr(iter),
                "filter": filter.as_ref().map(project_expr),
            })
        })
        .collect()
}

fn project_f_string_part(part: &HirFStringPart) -> Value {
    match part {
        HirFStringPart::Literal(_) => json!({"kind": "Literal"}),
        HirFStringPart::Expr(expr) => json!({
            "kind": "Expr",
            "expr": project_expr(expr),
        }),
    }
}

pub(super) fn project_named_types(items: &[(String, Type)]) -> Vec<Value> {
    items.iter().map(project_named_type).collect()
}

pub(super) fn project_named_type((name, ty): &(String, Type)) -> Value {
    json!({
        "name": name,
        "ty": type_name(ty),
    })
}

pub(super) fn type_name(ty: &Type) -> String {
    ty.display_name()
}

fn expr_type_name(expr: &HirExpr) -> String {
    type_name(expr.ty())
}

pub(super) fn tuple_target_binding_name(binding: &HirTupleTargetBinding) -> Value {
    match binding {
        HirTupleTargetBinding::Name(name) => json!({
            "kind": "Name",
            "name": name,
        }),
        HirTupleTargetBinding::Field { object, field } => json!({
            "kind": "Field",
            "object": object,
            "field": field,
        }),
    }
}

pub(super) fn async_with_kind_name(kind: &HirAsyncWithKind) -> Value {
    match kind {
        HirAsyncWithKind::TaskScope => json!({"kind": "TaskScope"}),
        HirAsyncWithKind::TaskGroup { context } => json!({
            "kind": "TaskGroup",
            "context": context.as_ref().map(project_expr),
        }),
        HirAsyncWithKind::TaskTimeout { duration } => json!({
            "kind": "TaskTimeout",
            "duration": project_expr(duration),
        }),
        HirAsyncWithKind::UserDefined {
            context,
            enter_value_ty,
            enter_error_ty,
            exit_error_ty,
        } => json!({
            "kind": "UserDefined",
            "context": project_expr(context),
            "enter_value_ty": type_name(enter_value_ty),
            "enter_error_ty": type_name(enter_error_ty),
            "exit_error_ty": type_name(exit_error_ty),
        }),
        HirAsyncWithKind::Python {
            context,
            manager_class,
            entered_type,
            enter_error_type,
            exit_error_type,
            entered_is_opaque_borrow,
            active_error_type,
        } => json!({
            "kind": "Python",
            "context": project_expr(context),
            "manager_class": manager_class,
            "entered_type": type_name(entered_type),
            "enter_error_type": type_name(enter_error_type),
            "exit_error_type": type_name(exit_error_type),
            "entered_is_opaque_borrow": entered_is_opaque_borrow,
            "active_error_type": type_name(active_error_type),
        }),
    }
}

pub(super) fn pattern_kind_name(pattern: &HirPattern) -> Value {
    match pattern {
        HirPattern::Wildcard => json!({"kind": "Wildcard"}),
        HirPattern::Capture { name, ty } => json!({
            "kind": "Capture",
            "name": name,
            "ty": type_name(ty),
        }),
        HirPattern::Literal { value } => json!({
            "kind": "Literal",
            "value": project_expr(value),
        }),
        HirPattern::None => json!({"kind": "None"}),
        HirPattern::Or { patterns } => json!({
            "kind": "Or",
            "patterns": patterns.iter().map(pattern_kind_name).collect::<Vec<_>>(),
        }),
        HirPattern::Class {
            class_name, fields, ..
        } => json!({
            "kind": "Class",
            "class_name": class_name,
            "fields": fields
                .iter()
                .map(|(name, pattern)| {
                    json!({
                        "name": name,
                        "pattern": pattern_kind_name(pattern),
                    })
                })
                .collect::<Vec<_>>(),
        }),
        HirPattern::Value { path } => json!({
            "kind": "Value",
            "path": path,
        }),
        HirPattern::Tuple { elements } => json!({
            "kind": "Tuple",
            "elements": elements.iter().map(pattern_kind_name).collect::<Vec<_>>(),
        }),
    }
}

fn iterator_op_name(op: &HirIteratorOp) -> &'static str {
    match op {
        HirIteratorOp::Iter => "Iter",
        HirIteratorOp::Next => "Next",
        HirIteratorOp::Reversed => "Reversed",
        HirIteratorOp::Map => "Map",
        HirIteratorOp::Filter => "Filter",
        HirIteratorOp::Zip => "Zip",
        HirIteratorOp::Enumerate => "Enumerate",
    }
}
