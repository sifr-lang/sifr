use crate::hir_snapshot_expr_projection::{
    async_with_kind_name, pattern_kind_name, project_expr, tuple_target_binding_name, type_name,
};
use crate::{
    ExternalDefs, HirModule, HirStmt, HirTupleTarget, HirWithItemKind, LoweringOptions,
    lower_module_with_externals_name_and_options,
};
use serde_json::{Value, json};
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
    lower_module_with_externals_name_and_options(
        "snapshot",
        parsed.suite(),
        &ExternalDefs::default(),
        LoweringOptions {
            source_text: Some(source.to_string()),
            ..LoweringOptions::default()
        },
    )
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
        "receiver": function.receiver.map(|receiver| format!("{receiver:?}")),
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
            ..
        } => json!({
            "kind": "StarUnpack",
            "before": before.iter().map(project_unpack_target).collect::<Vec<_>>(),
            "star": project_unpack_target(star),
            "after": after.iter().map(project_unpack_target).collect::<Vec<_>>(),
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
            ..
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
            ..
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
            ..
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
            failure,
        } => json!({
            "kind": "SubscriptAugAssign",
            "object": object,
            "index": project_expr(index),
            "op": op,
            "value": project_expr(value),
            "object_ty": type_name(object_ty),
            "failure": failure.as_ref().map(type_name),
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
            ..
        } => json!({
            "kind": "AttributeSubscriptAssign",
            "object": object,
            "field": field,
            "index": project_expr(index),
            "value": project_expr(value),
            "field_ty": type_name(field_ty),
        }),
        HirStmt::Delete { object, index, .. } => json!({
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
                            body_may_raise,
                        } => json!({
                            "kind": "Python",
                            "entered_type": type_name(entered_type),
                            "enter_error_type": type_name(enter_error_type),
                            "exit_error_type": type_name(exit_error_type),
                            "entered_is_opaque_borrow": entered_is_opaque_borrow,
                            "body_may_raise": body_may_raise,
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

fn project_unpack_target(target: &HirTupleTarget) -> Value {
    json!({
        "binding": tuple_target_binding_name(&target.binding),
        "ty": type_name(&target.ty),
        "rebind_existing": target.rebind_existing,
    })
}
