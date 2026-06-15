use crate::{lower_module, HirExpr, HirFunction, HirModule, HirStmt};
use serde_json::{json, Value};
use sifr_python_parser::parse_module;
use sifr_type_system::Type;
use std::{fs, path::PathBuf};

// Scope: these snapshots intentionally project stable, matrix-owned facts for
// module constants, top-level functions, nested functions, simple local
// bindings, loop targets, name references, and call forms. The projection is
// not a complete definition-ID graph; HIR does not expose one here. Binder
// shapes such as class/import scopes, pattern captures, with/as targets,
// tuple/star unpack targets, lambda parameters, and comprehension targets must
// be added explicitly before fixtures rely on them.
#[test]
fn name_resolution_snapshot_matrix_matches_lowered_name_facts() {
    let matrix = read_matrix();
    let cases = matrix
        .get("name_snapshots")
        .and_then(Value::as_array)
        .expect("name_snapshots must be an array");

    assert!(
        !cases.is_empty(),
        "name-resolution snapshot matrix must not be empty"
    );

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
            .get("expected_name_resolution_snapshot")
            .expect("case must declare expected_name_resolution_snapshot");
        let actual = project_name_resolution(&lower_source(source));

        assert_eq!(
            &actual, expected,
            "name-resolution snapshot mismatch for {id}"
        );
    }
}

fn read_matrix() -> Value {
    let path = matrix_path();
    serde_json::from_str(
        &fs::read_to_string(&path).expect("name-resolution snapshot matrix must be readable"),
    )
    .expect("name-resolution snapshot matrix must be valid JSON")
}

fn matrix_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("verification/areas/core_language/data/name_resolution_snapshot_matrix.json")
}

fn lower_source(source: &str) -> HirModule {
    let parsed = parse_module(source).expect("source must parse");
    lower_module(parsed.suite())
        .map(|result| result.module)
        .expect("source must lower")
}

fn project_name_resolution(module: &HirModule) -> Value {
    json!({
        "constants": module
            .constants
            .iter()
            .map(project_constant)
            .collect::<Vec<_>>(),
        "functions": module
            .functions
            .iter()
            .map(|function| project_function(function, &function.name))
            .collect::<Vec<_>>(),
    })
}

fn project_constant((name, ty, value): &(String, Type, HirExpr)) -> Value {
    json!({
        "name": name,
        "ty": type_name(ty),
        "value_kind": expr_kind(value),
    })
}

fn project_function(function: &HirFunction, path: &str) -> Value {
    let mut facts = NameFacts::default();
    collect_stmts(&function.body, path, &mut facts);

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
        "local_bindings": facts.local_bindings,
        "loop_targets": facts.loop_targets,
        "name_references": facts.name_references,
        "calls": facts.calls,
        "nested_functions": facts.nested_functions,
    })
}

#[derive(Default)]
struct NameFacts {
    local_bindings: Vec<Value>,
    loop_targets: Vec<Value>,
    name_references: Vec<Value>,
    calls: Vec<Value>,
    nested_functions: Vec<Value>,
}

fn collect_stmts(stmts: &[HirStmt], path: &str, facts: &mut NameFacts) {
    for (index, stmt) in stmts.iter().enumerate() {
        let stmt_path = format!("{path}/body[{index}]");
        collect_stmt(stmt, &stmt_path, facts);
    }
}

fn collect_stmt(stmt: &HirStmt, path: &str, facts: &mut NameFacts) {
    match stmt {
        HirStmt::Let {
            name, ty, value, ..
        } => {
            let binding_path = format!("{path}/let:{name}");
            facts.local_bindings.push(json!({
                "path": binding_path,
                "name": name,
                "ty": type_name(ty),
                "value_kind": expr_kind(value),
            }));
            collect_expr(value, &format!("{path}/let:{name}"), facts);
        }
        HirStmt::Assign { name, value } => {
            collect_expr(value, &format!("{path}/assign:{name}"), facts);
        }
        HirStmt::AugAssign { name, value, .. } => {
            collect_expr(value, &format!("{path}/augassign:{name}"), facts);
        }
        HirStmt::Return { value } => {
            if let Some(value) = value {
                collect_expr(value, &format!("{path}/return"), facts);
            }
        }
        HirStmt::Expr { expr } => collect_expr(expr, &format!("{path}/expr"), facts),
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } => {
            collect_expr(condition, &format!("{path}/condition"), facts);
            collect_nested_block(then_body, &format!("{path}/then"), facts);
            for (index, (condition, body)) in elif_clauses.iter().enumerate() {
                let elif_path = format!("{path}/elif[{index}]");
                collect_expr(condition, &format!("{elif_path}/condition"), facts);
                collect_nested_block(body, &elif_path, facts);
            }
            if let Some(else_body) = else_body {
                collect_nested_block(else_body, &format!("{path}/else"), facts);
            }
        }
        HirStmt::While {
            condition,
            body,
            else_body,
        } => {
            collect_expr(condition, &format!("{path}/condition"), facts);
            collect_nested_block(body, path, facts);
            if let Some(else_body) = else_body {
                collect_nested_block(else_body, &format!("{path}/else"), facts);
            }
        }
        HirStmt::For {
            target,
            target_ty,
            iter,
            body,
            else_body,
        } => {
            let for_path = format!("{path}/for:{target}");
            facts.loop_targets.push(json!({
                "path": for_path,
                "name": target,
                "ty": type_name(target_ty),
            }));
            collect_expr(iter, &format!("{path}/for:{target}/iter"), facts);
            collect_nested_block(body, &format!("{path}/for:{target}"), facts);
            if let Some(else_body) = else_body {
                collect_nested_block(else_body, &format!("{path}/for:{target}/else"), facts);
            }
        }
        HirStmt::NestedFunction { func } => {
            facts.nested_functions.push(project_function(
                func,
                &format!("{path}/nested:{}", func.name),
            ));
        }
        HirStmt::TryFinally { body, finalbody } => {
            collect_nested_block(body, path, facts);
            collect_nested_block(finalbody, &format!("{path}/finally"), facts);
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            collect_nested_block(body, path, facts);
            for (index, handler) in handlers.iter().enumerate() {
                collect_nested_block(&handler.body, &format!("{path}/handler[{index}]"), facts);
            }
        }
        HirStmt::Match { subject, arms, .. } => {
            collect_expr(subject, &format!("{path}/subject"), facts);
            for (index, arm) in arms.iter().enumerate() {
                if let Some(guard) = &arm.guard {
                    collect_expr(guard, &format!("{path}/arm[{index}]/guard"), facts);
                }
                collect_nested_block(&arm.body, &format!("{path}/arm[{index}]"), facts);
            }
        }
        HirStmt::AsyncFor {
            target,
            target_ty,
            iter,
            body,
            else_body,
            ..
        } => {
            facts.loop_targets.push(json!({
                "path": format!("{path}/async_for:{target}"),
                "name": target,
                "ty": type_name(target_ty),
            }));
            collect_expr(iter, &format!("{path}/async_for:{target}/iter"), facts);
            collect_nested_block(body, &format!("{path}/async_for:{target}"), facts);
            if let Some(else_body) = else_body {
                collect_nested_block(else_body, &format!("{path}/async_for:{target}/else"), facts);
            }
        }
        HirStmt::TupleUnpack { value, .. }
        | HirStmt::StarUnpack { value, .. }
        | HirStmt::Assert { test: value, .. }
        | HirStmt::Raise { value }
        | HirStmt::FieldAssign { value, .. }
        | HirStmt::NestedFieldAssign { value, .. }
        | HirStmt::SubscriptAugAssign { value, .. }
        | HirStmt::AttributeAugAssign { value, .. }
        | HirStmt::Yield { value } => collect_expr(value, path, facts),
        HirStmt::SubscriptAssign { index, value, .. }
        | HirStmt::AttributeSubscriptAssign { index, value, .. } => {
            collect_expr(index, &format!("{path}/index"), facts);
            collect_expr(value, path, facts);
        }
        HirStmt::NestedSubscriptAssign {
            outer_index,
            inner_index,
            value,
            ..
        }
        | HirStmt::AttributeNestedSubscriptAssign {
            outer_index,
            inner_index,
            value,
            ..
        } => {
            collect_expr(outer_index, &format!("{path}/outer_index"), facts);
            collect_expr(inner_index, &format!("{path}/inner_index"), facts);
            collect_expr(value, path, facts);
        }
        HirStmt::Delete { object, index } => {
            collect_expr(object, &format!("{path}/object"), facts);
            collect_expr(index, &format!("{path}/index"), facts);
        }
        HirStmt::With { items, body } => {
            for (index, (_, context, _)) in items.iter().enumerate() {
                collect_expr(context, &format!("{path}/with_item[{index}]"), facts);
            }
            collect_nested_block(body, path, facts);
        }
        HirStmt::AsyncWith { body, .. } => collect_nested_block(body, path, facts),
        HirStmt::Break | HirStmt::Continue | HirStmt::Pass => {}
    }
}

fn collect_nested_block(stmts: &[HirStmt], path: &str, facts: &mut NameFacts) {
    for (index, stmt) in stmts.iter().enumerate() {
        collect_stmt(stmt, &format!("{path}/body[{index}]"), facts);
    }
}

fn collect_expr(expr: &HirExpr, path: &str, facts: &mut NameFacts) {
    match expr {
        HirExpr::Name { name, ty } => facts.name_references.push(json!({
            "path": path,
            "name": name,
            "ty": type_name(ty),
        })),
        HirExpr::Call { func, args, ty } => {
            facts.calls.push(json!({
                "kind": "Call",
                "path": path,
                "func": func,
                "ty": type_name(ty),
                "args": args.iter().map(expr_summary).collect::<Vec<_>>(),
            }));
            for (index, arg) in args.iter().enumerate() {
                collect_expr(arg, &format!("{path}/arg[{index}]"), facts);
            }
        }
        HirExpr::BinOp { left, right, .. } => {
            collect_expr(left, &format!("{path}/left"), facts);
            collect_expr(right, &format!("{path}/right"), facts);
        }
        HirExpr::UnaryOp { operand, .. }
        | HirExpr::Await { value: operand, .. }
        | HirExpr::QuestionMark { expr: operand, .. }
        | HirExpr::OkWrap { value: operand, .. }
        | HirExpr::ErrWrap { value: operand, .. } => collect_expr(operand, path, facts),
        HirExpr::Compare {
            left, comparators, ..
        } => {
            collect_expr(left, &format!("{path}/left"), facts);
            for (index, comparator) in comparators.iter().enumerate() {
                collect_expr(comparator, &format!("{path}/comparator[{index}]"), facts);
            }
        }
        HirExpr::BoolOp { values, .. }
        | HirExpr::ListLiteral {
            elements: values, ..
        }
        | HirExpr::SetLiteral {
            elements: values, ..
        }
        | HirExpr::TupleLiteral {
            elements: values, ..
        } => {
            for (index, value) in values.iter().enumerate() {
                collect_expr(value, &format!("{path}/value[{index}]"), facts);
            }
        }
        HirExpr::IteratorCall { args, .. } | HirExpr::SuperCall { args, .. } => {
            for (index, arg) in args.iter().enumerate() {
                collect_expr(arg, &format!("{path}/arg[{index}]"), facts);
            }
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            collect_expr(condition, &format!("{path}/condition"), facts);
            collect_expr(then_expr, &format!("{path}/then"), facts);
            collect_expr(else_expr, &format!("{path}/else"), facts);
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            collect_expr(start, &format!("{path}/start"), facts);
            collect_expr(end, &format!("{path}/end"), facts);
            if let Some(step) = step {
                collect_expr(step, &format!("{path}/step"), facts);
            }
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            for (index, key) in keys.iter().enumerate() {
                collect_expr(key, &format!("{path}/key[{index}]"), facts);
            }
            for (index, value) in values.iter().enumerate() {
                collect_expr(value, &format!("{path}/value[{index}]"), facts);
            }
        }
        HirExpr::Index { object, index, .. } => {
            collect_expr(object, &format!("{path}/object"), facts);
            collect_expr(index, &format!("{path}/index"), facts);
        }
        HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } => {
            facts.calls.push(json!({
                "kind": "MethodCall",
                "path": path,
                "method": method,
                "receiver": expr_summary(object),
                "ty": type_name(expr.ty()),
                "args": args.iter().map(expr_summary).collect::<Vec<_>>(),
            }));
            collect_expr(object, &format!("{path}/object"), facts);
            for (index, arg) in args.iter().enumerate() {
                collect_expr(arg, &format!("{path}/arg[{index}]"), facts);
            }
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => {
            collect_expr(element, &format!("{path}/element"), facts);
            collect_expr(collection, &format!("{path}/collection"), facts);
        }
        HirExpr::FString { parts, .. } => {
            for (index, part) in parts.iter().enumerate() {
                if let crate::HirFStringPart::Expr(expr) = part {
                    collect_expr(expr, &format!("{path}/part[{index}]"), facts);
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
            collect_expr(object, &format!("{path}/object"), facts);
            if let Some(start) = start {
                collect_expr(start, &format!("{path}/start"), facts);
            }
            if let Some(stop) = stop {
                collect_expr(stop, &format!("{path}/stop"), facts);
            }
            if let Some(step) = step {
                collect_expr(step, &format!("{path}/step"), facts);
            }
        }
        HirExpr::WalrusExpr { value, .. } => collect_expr(value, path, facts),
        HirExpr::FieldAccess { object, .. } => {
            collect_expr(object, &format!("{path}/object"), facts)
        }
        HirExpr::ConstructorCall { args, .. } => {
            for (index, arg) in args.iter().enumerate() {
                collect_expr(arg, &format!("{path}/arg[{index}]"), facts);
            }
        }
        HirExpr::Lambda { body, .. } => collect_expr(body, &format!("{path}/body"), facts),
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            collect_expr(expr, &format!("{path}/expr"), facts);
            collect_generators(generators, path, facts);
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            collect_expr(key_expr, &format!("{path}/key"), facts);
            collect_expr(val_expr, &format!("{path}/value"), facts);
            collect_generators(generators, path, facts);
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            collect_expr(expr, &format!("{path}/expr"), facts);
            collect_expr(iter, &format!("{path}/iter"), facts);
            if let Some(filter) = filter {
                collect_expr(filter, &format!("{path}/filter"), facts);
            }
        }
        HirExpr::IntLiteral(_)
        | HirExpr::LargeIntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::EnumVariant { .. } => {}
    }
}

fn collect_generators(
    generators: &[(String, HirExpr, Option<HirExpr>)],
    path: &str,
    facts: &mut NameFacts,
) {
    for (index, (_, iter, filter)) in generators.iter().enumerate() {
        collect_expr(iter, &format!("{path}/generator[{index}]/iter"), facts);
        if let Some(filter) = filter {
            collect_expr(filter, &format!("{path}/generator[{index}]/filter"), facts);
        }
    }
}

fn expr_summary(expr: &HirExpr) -> Value {
    match expr {
        HirExpr::Name { name, .. } => json!({
            "kind": "Name",
            "ty": type_name(expr.ty()),
            "name": name,
        }),
        _ => json!({
            "kind": expr_kind(expr),
            "ty": type_name(expr.ty()),
        }),
    }
}

fn expr_kind(expr: &HirExpr) -> &'static str {
    match expr {
        HirExpr::IntLiteral(_) => "IntLiteral",
        HirExpr::LargeIntLiteral(_) => "LargeIntLiteral",
        HirExpr::FloatLiteral(_) => "FloatLiteral",
        HirExpr::StringLiteral(_) => "StringLiteral",
        HirExpr::BoolLiteral(_) => "BoolLiteral",
        HirExpr::NoneLiteral => "NoneLiteral",
        HirExpr::Name { .. } => "Name",
        HirExpr::BinOp { .. } => "BinOp",
        HirExpr::UnaryOp { .. } => "UnaryOp",
        HirExpr::Compare { .. } => "Compare",
        HirExpr::BoolOp { .. } => "BoolOp",
        HirExpr::Call { .. } => "Call",
        HirExpr::Await { .. } => "Await",
        HirExpr::IteratorCall { .. } => "IteratorCall",
        HirExpr::IfExpr { .. } => "IfExpr",
        HirExpr::RangeLiteral { .. } => "RangeLiteral",
        HirExpr::ListLiteral { .. } => "ListLiteral",
        HirExpr::SetLiteral { .. } => "SetLiteral",
        HirExpr::DictLiteral { .. } => "DictLiteral",
        HirExpr::TupleLiteral { .. } => "TupleLiteral",
        HirExpr::Index { .. } => "Index",
        HirExpr::MethodCall { .. } => "MethodCall",
        HirExpr::ContainsOp { .. } => "ContainsOp",
        HirExpr::FString { .. } => "FString",
        HirExpr::Slice { .. } => "Slice",
        HirExpr::WalrusExpr { .. } => "WalrusExpr",
        HirExpr::FieldAccess { .. } => "FieldAccess",
        HirExpr::ConstructorCall { .. } => "ConstructorCall",
        HirExpr::QuestionMark { .. } => "QuestionMark",
        HirExpr::OkWrap { .. } => "OkWrap",
        HirExpr::ErrWrap { .. } => "ErrWrap",
        HirExpr::SuperCall { .. } => "SuperCall",
        HirExpr::Lambda { .. } => "Lambda",
        HirExpr::ListComp { .. } => "ListComp",
        HirExpr::DictComp { .. } => "DictComp",
        HirExpr::SetComp { .. } => "SetComp",
        HirExpr::GeneratorExpr { .. } => "GeneratorExpr",
        HirExpr::EnumVariant { .. } => "EnumVariant",
    }
}

fn type_name(ty: &Type) -> String {
    ty.display_name()
}
