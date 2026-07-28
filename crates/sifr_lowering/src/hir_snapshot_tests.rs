use crate::{
    lower_module, HirAsyncWithKind, HirExpr, HirFStringPart, HirIteratorOp, HirModule, HirPattern,
    HirStmt, HirTupleTargetBinding, HirWithItemKind,
};
use serde_json::{json, Value};
use sifr_python_parser::parse_module;
use sifr_type_system::Type;
use std::{fs, path::PathBuf};

#[test]
fn hir_lowering_snapshot_matrix_matches_lowered_module_shape() {
    let matrix = read_matrix();
    let cases = matrix
        .get("hir_snapshots")
        .and_then(Value::as_array)
        .expect("hir_snapshots must be an array");

    assert!(!cases.is_empty(), "hir snapshot matrix must not be empty");

    for case in cases {
        let id = case
            .get("id")
            .and_then(Value::as_str)
            .expect("case id must be a string");
        let source = case
            .get("source")
            .and_then(Value::as_str)
            .expect("case source must be a string");
        let expected = case
            .get("expected_hir_snapshot")
            .expect("case must declare expected_hir_snapshot");
        let actual = project_module(&lower_source(source));

        assert_eq!(&actual, expected, "HIR snapshot mismatch for {id}");
    }
}

fn read_matrix() -> Value {
    let path = matrix_path();
    serde_json::from_str(&fs::read_to_string(&path).expect("HIR snapshot matrix must be readable"))
        .expect("HIR snapshot matrix must be valid JSON")
}

fn matrix_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("verification/areas/core_language/data/hir_lowering_snapshot_matrix.json")
}

fn lower_source(source: &str) -> HirModule {
    let parsed = parse_module(source).expect("source must parse");
    lower_module(parsed.suite())
        .map(|result| result.module)
        .expect("source must lower")
}

fn project_module(module: &HirModule) -> Value {
    json!({
        "functions": module
            .functions
            .iter()
            .map(project_function)
            .collect::<Vec<_>>(),
    })
}

fn project_function(function: &crate::HirFunction) -> Value {
    json!({
        "name": function.name,
        "params": function
            .params
            .iter()
            .map(|param| {
                json!({
                    "name": param.name,
                    "ty": type_name(&param.ty),
                })
            })
            .collect::<Vec<_>>(),
        "return_type": type_name(&function.return_type),
        "body": project_stmts(&function.body),
    })
}

fn project_stmts(stmts: &[HirStmt]) -> Vec<Value> {
    stmts.iter().map(project_stmt).collect()
}

fn project_stmt(stmt: &HirStmt) -> Value {
    match stmt {
        HirStmt::Let {
            name, ty, value, ..
        } => json!({
            "kind": "Let",
            "name": name,
            "ty": type_name(ty),
            "value": project_expr(value),
        }),
        HirStmt::Assign { name, value } => json!({
            "kind": "Assign",
            "name": name,
            "value": project_expr(value),
        }),
        HirStmt::AugAssign { name, op, value } => json!({
            "kind": "AugAssign",
            "name": name,
            "op": op,
            "value": project_expr(value),
        }),
        HirStmt::Return { value } => json!({
            "kind": "Return",
            "value": value.as_ref().map(project_expr),
        }),
        HirStmt::Expr { expr } => json!({
            "kind": "Expr",
            "expr": project_expr(expr),
        }),
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } => json!({
            "kind": "If",
            "condition": project_expr(condition),
            "then_body": project_stmts(then_body),
            "elif_clauses": elif_clauses
                .iter()
                .map(|(condition, body)| {
                    json!({
                        "condition": project_expr(condition),
                        "body": project_stmts(body),
                    })
                })
                .collect::<Vec<_>>(),
            "else_body": else_body.as_deref().map(project_stmts),
        }),
        HirStmt::While {
            condition,
            body,
            else_body,
        } => json!({
            "kind": "While",
            "condition": project_expr(condition),
            "body": project_stmts(body),
            "else_body": else_body.as_deref().map(project_stmts),
        }),
        HirStmt::For {
            target,
            target_ty,
            iter,
            body,
            else_body,
        } => json!({
            "kind": "For",
            "target": target,
            "target_ty": type_name(target_ty),
            "iter": project_expr(iter),
            "body": project_stmts(body),
            "else_body": else_body.as_deref().map(project_stmts),
        }),
        HirStmt::AsyncFor {
            target,
            target_ty,
            iter,
            iter_error_ty,
            close_error_ty,
            body,
            else_body,
        } => json!({
            "kind": "AsyncFor",
            "target": target,
            "target_ty": type_name(target_ty),
            "iter": project_expr(iter),
            "iter_error_ty": type_name(iter_error_ty),
            "close_error_ty": close_error_ty.as_ref().map(type_name),
            "body": project_stmts(body),
            "else_body": else_body.as_deref().map(project_stmts),
        }),
        HirStmt::Break => json!({"kind": "Break"}),
        HirStmt::Continue => json!({"kind": "Continue"}),
        HirStmt::TupleUnpack { targets, value } => json!({
            "kind": "TupleUnpack",
            "targets": targets
                .iter()
                .map(|target| {
                    json!({
                        "binding": tuple_target_binding_name(&target.binding),
                        "ty": type_name(&target.ty),
                    })
                })
                .collect::<Vec<_>>(),
            "value": project_expr(value),
        }),
        HirStmt::StarUnpack {
            before,
            star,
            after,
            value,
        } => json!({
            "kind": "StarUnpack",
            "before": project_named_types(before),
            "star": project_named_type(star),
            "after": project_named_types(after),
            "value": project_expr(value),
        }),
        HirStmt::Pass => json!({"kind": "Pass"}),
        HirStmt::Assert { test, msg } => json!({
            "kind": "Assert",
            "test": project_expr(test),
            "msg": msg.as_ref().map(project_expr),
        }),
        HirStmt::Raise { value } => json!({
            "kind": "Raise",
            "value": project_expr(value),
        }),
        HirStmt::TryExcept {
            body,
            handlers,
            body_error_types,
        } => json!({
            "kind": "TryExcept",
            "body": project_stmts(body),
            "handlers": handlers
                .iter()
                .map(|handler| {
                    json!({
                        "error_type": handler.error_type,
                        "error_resolved_type": handler.error_resolved_type.as_ref().map(type_name),
                        "name": handler.name,
                        "body": project_stmts(&handler.body),
                    })
                })
                .collect::<Vec<_>>(),
            "body_error_types": body_error_types
                .iter()
                .map(Type::display_name)
                .collect::<Vec<_>>(),
        }),
        HirStmt::TryFinally { body, finalbody } => json!({
            "kind": "TryFinally",
            "body": project_stmts(body),
            "finalbody": project_stmts(finalbody),
        }),
        HirStmt::FieldAssign {
            object,
            field,
            field_ty,
            value,
        } => json!({
            "kind": "FieldAssign",
            "object": object,
            "field": field,
            "field_ty": type_name(field_ty),
            "value": project_expr(value),
        }),
        HirStmt::NestedFieldAssign {
            object,
            field,
            field_ty,
            nested_field,
            nested_field_ty,
            value,
        } => json!({
            "kind": "NestedFieldAssign",
            "object": object,
            "field": field,
            "field_ty": type_name(field_ty),
            "nested_field": nested_field,
            "nested_field_ty": type_name(nested_field_ty),
            "value": project_expr(value),
        }),
        HirStmt::SubscriptAssign {
            object,
            index,
            value,
            object_ty,
        } => json!({
            "kind": "SubscriptAssign",
            "object": object,
            "index": project_expr(index),
            "value": project_expr(value),
            "object_ty": type_name(object_ty),
        }),
        HirStmt::NestedSubscriptAssign {
            object,
            outer_index,
            inner_index,
            value,
            object_ty,
        } => json!({
            "kind": "NestedSubscriptAssign",
            "object": object,
            "outer_index": project_expr(outer_index),
            "inner_index": project_expr(inner_index),
            "value": project_expr(value),
            "object_ty": type_name(object_ty),
        }),
        HirStmt::AttributeNestedSubscriptAssign {
            object,
            field,
            outer_index,
            inner_index,
            value,
            field_ty,
        } => json!({
            "kind": "AttributeNestedSubscriptAssign",
            "object": object,
            "field": field,
            "outer_index": project_expr(outer_index),
            "inner_index": project_expr(inner_index),
            "value": project_expr(value),
            "field_ty": type_name(field_ty),
        }),
        HirStmt::SubscriptAugAssign {
            object,
            index,
            op,
            value,
            object_ty,
        } => json!({
            "kind": "SubscriptAugAssign",
            "object": object,
            "index": project_expr(index),
            "op": op,
            "value": project_expr(value),
            "object_ty": type_name(object_ty),
        }),
        HirStmt::AttributeAugAssign {
            object,
            field,
            op,
            value,
        } => json!({
            "kind": "AttributeAugAssign",
            "object": object,
            "field": field,
            "op": op,
            "value": project_expr(value),
        }),
        HirStmt::AttributeSubscriptAssign {
            object,
            field,
            index,
            value,
            field_ty,
        } => json!({
            "kind": "AttributeSubscriptAssign",
            "object": object,
            "field": field,
            "index": project_expr(index),
            "value": project_expr(value),
            "field_ty": type_name(field_ty),
        }),
        HirStmt::Delete { object, index } => json!({
            "kind": "Delete",
            "object": project_expr(object),
            "index": project_expr(index),
        }),
        HirStmt::Yield { value } => json!({
            "kind": "Yield",
            "value": project_expr(value),
        }),
        HirStmt::With { items, body } => json!({
            "kind": "With",
            "items": items
                .iter()
                .map(|item| {
                    let protocol = match &item.kind {
                        HirWithItemKind::Native {
                            has_context_manager_protocol,
                        } => json!({
                            "kind": "Native",
                            "has_context_manager_protocol": has_context_manager_protocol,
                        }),
                        HirWithItemKind::Python {
                            entered_type,
                            enter_error_type,
                            exit_error_type,
                            entered_is_opaque_borrow,
                        } => json!({
                            "kind": "Python",
                            "entered_type": type_name(entered_type),
                            "enter_error_type": type_name(enter_error_type),
                            "exit_error_type": type_name(exit_error_type),
                            "entered_is_opaque_borrow": entered_is_opaque_borrow,
                        }),
                    };
                    json!({
                        "name": item.target,
                        "context": project_expr(&item.context),
                        "protocol": protocol,
                    })
                })
                .collect::<Vec<_>>(),
            "body": project_stmts(body),
        }),
        HirStmt::AsyncWith { kind, target, body } => json!({
            "kind": "AsyncWith",
            "async_with_kind": async_with_kind_name(kind),
            "target": target,
            "body": project_stmts(body),
        }),
        HirStmt::NestedFunction { func, .. } => json!({
            "kind": "NestedFunction",
            "function": project_function(func),
        }),
        HirStmt::Match {
            subject,
            subject_ty,
            arms,
        } => json!({
            "kind": "Match",
            "subject": project_expr(subject),
            "subject_ty": type_name(subject_ty),
            "arms": arms
                .iter()
                .map(|arm| {
                    json!({
                        "pattern": pattern_kind_name(&arm.pattern),
                        "guard": arm.guard.as_ref().map(project_expr),
                        "body": project_stmts(&arm.body),
                    })
                })
                .collect::<Vec<_>>(),
        }),
    }
}

fn project_expr(expr: &HirExpr) -> Value {
    match expr {
        HirExpr::IntLiteral(_) => scalar_expr("IntLiteral", expr),
        HirExpr::LargeIntLiteral(_) => scalar_expr("LargeIntLiteral", expr),
        HirExpr::FloatLiteral(_) => scalar_expr("FloatLiteral", expr),
        HirExpr::StringLiteral(_) => scalar_expr("StringLiteral", expr),
        HirExpr::BoolLiteral(_) => scalar_expr("BoolLiteral", expr),
        HirExpr::NoneLiteral => scalar_expr("NoneLiteral", expr),
        HirExpr::Name { name, .. } => json!({
            "kind": "Name",
            "ty": expr_type_name(expr),
            "name": name,
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
            ..
        } => json!({
            "kind": "MethodCall",
            "ty": expr_type_name(expr),
            "object": project_expr(object),
            "method": method,
            "args": args.iter().map(project_expr).collect::<Vec<_>>(),
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

fn project_named_types(items: &[(String, Type)]) -> Vec<Value> {
    items.iter().map(project_named_type).collect()
}

fn project_named_type((name, ty): &(String, Type)) -> Value {
    json!({
        "name": name,
        "ty": type_name(ty),
    })
}

fn type_name(ty: &Type) -> String {
    ty.display_name()
}

fn expr_type_name(expr: &HirExpr) -> String {
    type_name(expr.ty())
}

fn tuple_target_binding_name(binding: &HirTupleTargetBinding) -> Value {
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

fn async_with_kind_name(kind: &HirAsyncWithKind) -> Value {
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

fn pattern_kind_name(pattern: &HirPattern) -> Value {
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
