//! Sifr intermediate representation data schemas.
//!
//! This crate owns immutable HIR, CFG, flow-graph, and lowering result data.
//! Lowering construction algorithms remain in the producer crate.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

pub mod cfg;
mod class_ancestry;
pub mod diagnostic_types;
pub mod flow_graph;
mod hir_expr;
mod hir_flow;
pub mod hir_nodes;
pub mod lowering_outcome;
pub mod lowering_result;
pub mod python_interop;
pub mod rust_interop;
mod specialization_metadata;
mod sql_migrations;
mod sql_queries;
mod template_strings;
mod type_visit;

pub use cfg::*;
pub use diagnostic_types::*;
pub use flow_graph::*;
pub use hir_flow::*;
pub use hir_nodes::*;
pub use lowering_outcome::LoweringOutcome;
pub use lowering_result::LoweringResult;
pub use python_interop::*;
pub use rust_interop::*;
pub use specialization_metadata::*;
pub use sql_migrations::*;
pub use sql_queries::*;
pub use template_strings::*;
pub use type_visit::{
    transform_hir_function_types, visit_hir_function_exprs_mut, visit_hir_stmts_exprs_mut,
    visit_hir_stmts_storage_roots_mut,
};

#[cfg(test)]
mod tests {
    use super::{
        CfgBlock, CfgBlockLabel, CfgTerminator, ControlFlowGraph, FlowEdge, FlowEdgeKind,
        FlowEffect, FlowExitEffect, FlowExitKind, FlowFacts, FlowGraph, FlowNode, FlowNodeKind,
        HirExpr, MethodCallSource,
    };
    use ruff_text_size::{TextRange, TextSize};
    use sifr_type_system::{ReceiverConvention, Type};

    #[test]
    fn cfg_reachability_and_fingerprint_are_stable() {
        let cfg = ControlFlowGraph::new(
            vec![
                CfgBlock {
                    id: 0,
                    label: CfgBlockLabel::Entry,
                    top_level_stmt_index: None,
                    terminator: CfgTerminator::Goto(1),
                },
                CfgBlock {
                    id: 1,
                    label: CfgBlockLabel::Statement("return"),
                    top_level_stmt_index: Some(0),
                    terminator: CfgTerminator::Return {
                        ty: Type::Int,
                        has_value: true,
                    },
                },
                CfgBlock {
                    id: 2,
                    label: CfgBlockLabel::Exit,
                    top_level_stmt_index: None,
                    terminator: CfgTerminator::Exit,
                },
            ],
            0,
            2,
            vec![1],
        );

        cfg.validate().expect("valid cfg should pass invariants");
        assert_eq!(cfg.reachable_blocks(), vec![true, true, false]);
        assert_eq!(
            cfg.shape_fingerprint(),
            "entry:0;exit:2;b0:Entry:None:goto:1;b1:Statement(\"return\"):Some(0):return:int:true;b2:Exit:None:exit;"
        );
    }

    #[test]
    fn cfg_validation_rejects_invalid_successors() {
        let cfg = ControlFlowGraph::new(
            vec![CfgBlock {
                id: 0,
                label: CfgBlockLabel::Entry,
                top_level_stmt_index: None,
                terminator: CfgTerminator::Goto(3),
            }],
            0,
            0,
            vec![],
        );

        let error = cfg.validate().expect_err("invalid successor should fail");
        assert!(
            error.to_string().contains("invalid successor 3"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn method_call_schema_retains_receiver_and_source_metadata() {
        let range = TextRange::new(TextSize::new(2), TextSize::new(6));
        let call = HirExpr::MethodCall {
            object: Box::new(HirExpr::Name {
                name: "items".to_string(),
                binding_id: Some(crate::BindingId(7)),
                ty: Type::List(Box::new(Type::Int)),
            }),
            method: "append".to_string(),
            args: vec![HirExpr::IntLiteral(1)],
            receiver_convention: Some(ReceiverConvention::MutableBorrow),
            receiver_target: None,
            mutable_arg_places: Vec::new(),
            source: Some(MethodCallSource {
                call_range: range,
                receiver_range: range,
                arg_ranges: vec![range],
            }),
            ty: Type::None,
        };

        let HirExpr::MethodCall {
            receiver_convention,
            source: Some(source),
            ..
        } = call
        else {
            panic!("method call should retain its schema");
        };
        assert_eq!(receiver_convention, Some(ReceiverConvention::MutableBorrow));
        assert_eq!(source.arg_ranges, vec![range]);
    }

    #[test]
    fn flow_graph_trace_and_facts_preserve_effects() {
        let graph = FlowGraph::new(
            vec![
                FlowNode {
                    id: 0,
                    kind: FlowNodeKind::Entry {
                        scope: "main".to_string(),
                    },
                    effects: vec![FlowEffect::Define {
                        binding: "value".to_string(),
                        ty: Type::Int,
                    }],
                },
                FlowNode {
                    id: 1,
                    kind: FlowNodeKind::Statement {
                        label: "return".to_string(),
                        top_level_stmt_index: Some(0),
                    },
                    effects: vec![FlowEffect::Exit {
                        kind: FlowExitKind::Return,
                    }],
                },
            ],
            vec![FlowEdge {
                from: 0,
                to: 1,
                kind: FlowEdgeKind::Sequence,
            }],
            0,
            1,
        );
        let facts = FlowFacts::new(
            FlowExitEffect::AlwaysReturns,
            graph,
            vec![0],
            vec![1],
            vec![Type::Int],
            true,
            true,
        );

        assert!(facts.always_exits());
        assert!(facts.has_reachable_return());
        assert!(facts.has_reachable_value_return());
        assert_eq!(facts.reachable_top_level_stmt_indices(), &[0]);
        assert_eq!(facts.unreachable_top_level_stmt_indices(), &[1]);
        assert_eq!(facts.reachable_return_types(), &[Type::Int]);
        assert!(facts.flow_graph_fingerprint().contains("define value: int"));
        assert!(
            facts
                .flow_graph_debug_trace()
                .contains("effect exit Return")
        );
    }
}
