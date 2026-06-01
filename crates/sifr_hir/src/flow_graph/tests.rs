use super::*;
use crate::{lower_module, HirExpr, HirStmt};
use sifr_python_parser::parse_module;

fn lower_source(source: &str) -> crate::LoweringResult {
    let parsed = parse_module(source).expect("source should parse");
    lower_module(parsed.suite()).expect("source should lower")
}

#[test]
fn statement_graph_tracks_branches_loops_mutations_and_exits() {
    let stmts = vec![
        HirStmt::If {
            condition: HirExpr::Compare {
                left: Box::new(HirExpr::Name {
                    name: "x".to_string(),
                    ty: Type::Union(vec![Type::Int, Type::None]),
                }),
                ops: vec!["is".to_string()],
                comparators: vec![HirExpr::NoneLiteral],
                ty: Type::Bool,
            },
            then_body: vec![HirStmt::Return { value: None }],
            elif_clauses: vec![],
            else_body: None,
        },
        HirStmt::Expr {
            expr: HirExpr::MethodCall {
                object: Box::new(HirExpr::Name {
                    name: "items".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                }),
                method: "pop".to_string(),
                args: vec![],
                ty: Type::Int,
            },
        },
    ];

    let graph = build_statement_flow_graph(&stmts);
    let trace = graph.debug_trace();
    assert!(trace.contains("narrow x -> None"));
    assert!(trace.contains("join"));
    assert!(trace.contains("mutate items via method pop"));
    assert!(trace.contains("exit Return"));
}

#[test]
fn graph_fingerprint_includes_effect_payloads() {
    let first = build_statement_flow_graph(&[HirStmt::Expr {
        expr: HirExpr::Call {
            func: "first".to_string(),
            args: vec![],
            ty: Type::None,
        },
    }]);
    let second = build_statement_flow_graph(&[HirStmt::Expr {
        expr: HirExpr::Call {
            func: "second".to_string(),
            args: vec![],
            ty: Type::None,
        },
    }]);

    assert_ne!(first.shape_fingerprint(), second.shape_fingerprint());
}

#[test]
fn lowering_result_exposes_snapshot_scoped_flow_effects() {
    let result = lower_source(
        "def take(x: str) -> None:\n    pass\n\n\
             def main(mut v: str | None, mut items: list[int]) -> None:\n    \
             if v is None:\n        return\n    v = \"ok\"\n    items.pop()\n    take(\"ok\")\n",
    );

    let trace = result.flow_graph.debug_trace();
    assert!(trace.contains("function main"));
    assert!(trace.contains("narrow v -> None"));
    assert!(trace.contains("clear-narrowing v"));
    assert!(trace.contains("clear-narrowing items"));
    assert!(trace.contains("mutate items via method pop"));
    assert_eq!(
        result.flow_graph.shape_fingerprint(),
        result.flow_graph.shape_fingerprint()
    );
}

#[test]
fn await_task_handle_records_move_effect() {
    let result = lower_source(
        "@cpu_heavy\n\
         def compute_value() -> int:\n    return 42\n\n\
         async def main() -> Result[None, ScopeFailure]:\n    \
         handle = task.spawn_blocking(compute_value)\n    result = await handle\n    return None\n",
    );

    let trace = result.flow_graph.debug_trace();
    assert!(trace.contains("move handle"));
}
