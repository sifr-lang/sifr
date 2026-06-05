use super::*;
use crate::{lower_module, HirExpr, HirStmt};
use sifr_python_parser::parse_module;

fn lower_source(source: &str) -> crate::LoweringResult {
    let parsed = parse_module(source).expect("source should parse");
    lower_module(parsed.suite()).expect("source should lower")
}

fn call_stmt(callee: &str) -> HirStmt {
    HirStmt::Expr {
        expr: HirExpr::Call {
            func: callee.to_string(),
            args: vec![],
            ty: Type::None,
        },
    }
}

fn loop_node_id(graph: &FlowGraph) -> FlowNodeId {
    graph
        .nodes()
        .iter()
        .find_map(|node| match &node.kind {
            FlowNodeKind::Loop { label } if label == "while" => Some(node.id),
            _ => None,
        })
        .expect("flow graph should contain a while loop node")
}

fn call_node_id(graph: &FlowGraph, callee: &str) -> FlowNodeId {
    graph
        .nodes()
        .iter()
        .find_map(|node| {
            node.effects
                .iter()
                .any(|effect| matches!(effect, FlowEffect::Call { callee: name } if name == callee))
                .then_some(node.id)
        })
        .expect("flow graph should contain the requested call node")
}

fn loop_join_ids(graph: &FlowGraph) -> Vec<FlowNodeId> {
    graph
        .nodes()
        .iter()
        .filter_map(|node| match &node.kind {
            FlowNodeKind::Join { label } if label == "loop" => Some(node.id),
            _ => None,
        })
        .collect()
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
fn loop_with_else_omits_synthetic_loop_join() {
    let graph = build_statement_flow_graph(&[
        HirStmt::While {
            condition: HirExpr::BoolLiteral(true),
            body: vec![call_stmt("loop_body")],
            else_body: Some(vec![call_stmt("loop_else")]),
        },
        call_stmt("after_loop"),
    ]);

    assert!(loop_join_ids(&graph).is_empty());

    let loop_node = loop_node_id(&graph);
    let else_node = call_node_id(&graph, "loop_else");
    let after_loop_node = call_node_id(&graph, "after_loop");
    assert!(graph.edges().iter().any(|edge| edge.from == loop_node
        && edge.to == else_node
        && edge.kind == FlowEdgeKind::False));
    assert!(graph.edges().iter().any(|edge| edge.from == else_node
        && edge.to == after_loop_node
        && edge.kind == FlowEdgeKind::Sequence));
}

#[test]
fn loop_without_else_emits_single_synthetic_loop_join() {
    let graph = build_statement_flow_graph(&[
        HirStmt::While {
            condition: HirExpr::BoolLiteral(true),
            body: vec![call_stmt("loop_body")],
            else_body: None,
        },
        call_stmt("after_loop"),
    ]);

    let joins = loop_join_ids(&graph);
    assert_eq!(joins.len(), 1);

    let loop_node = loop_node_id(&graph);
    let join_node = joins[0];
    let after_loop_node = call_node_id(&graph, "after_loop");
    assert!(graph.edges().iter().any(|edge| edge.from == loop_node
        && edge.to == join_node
        && edge.kind == FlowEdgeKind::False));
    assert!(graph.edges().iter().any(|edge| edge.from == join_node
        && edge.to == after_loop_node
        && edge.kind == FlowEdgeKind::Sequence));
}

#[test]
fn graph_fingerprint_includes_effect_payloads() {
    let first = build_statement_flow_graph(&[call_stmt("first")]);
    let second = build_statement_flow_graph(&[call_stmt("second")]);

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
