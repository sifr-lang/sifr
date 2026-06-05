//! Control-flow graph construction and flow-truth queries for HIR statement blocks.

use crate::{HirExpr, HirStmt};
use sifr_ir::{
    CfgBlock, CfgBlockId, CfgBlockLabel, CfgTerminator, ControlFlowGraph, FlowExitEffect, FlowFacts,
};
use sifr_type_system::Type;

#[derive(Clone, Copy)]
struct LoopTargets {
    break_target: CfgBlockId,
    continue_target: CfgBlockId,
}

struct CfgBuilder {
    blocks: Vec<CfgBlock>,
    entry: CfgBlockId,
    exit: CfgBlockId,
    top_level_stmt_nodes: Vec<CfgBlockId>,
}

impl CfgBuilder {
    fn new(top_level_stmt_count: usize) -> Self {
        let mut builder = Self {
            blocks: Vec::new(),
            entry: 0,
            exit: 0,
            top_level_stmt_nodes: vec![0; top_level_stmt_count],
        };
        builder.exit = builder.new_block(CfgBlockLabel::Exit, None);
        builder.entry = builder.new_block(CfgBlockLabel::Entry, None);
        builder
    }

    fn new_block(
        &mut self,
        label: CfgBlockLabel,
        top_level_stmt_index: Option<usize>,
    ) -> CfgBlockId {
        let id = self.blocks.len();
        self.blocks.push(CfgBlock {
            id,
            label,
            top_level_stmt_index,
            terminator: CfgTerminator::Exit,
        });
        id
    }

    fn set_terminator(&mut self, block_id: CfgBlockId, terminator: CfgTerminator) {
        self.blocks[block_id].terminator = terminator;
    }

    fn build_stmt_list(
        &mut self,
        stmts: &[HirStmt],
        fallthrough: CfgBlockId,
        loop_targets: Option<LoopTargets>,
        top_level: bool,
    ) -> CfgBlockId {
        let mut next = fallthrough;
        for (idx, stmt) in stmts.iter().enumerate().rev() {
            let top_level_stmt_index = if top_level { Some(idx) } else { None };
            let entry = self.build_stmt(stmt, next, loop_targets, top_level_stmt_index);
            if top_level {
                self.top_level_stmt_nodes[idx] = entry;
            }
            next = entry;
        }
        next
    }

    fn build_stmt(
        &mut self,
        stmt: &HirStmt,
        next: CfgBlockId,
        loop_targets: Option<LoopTargets>,
        top_level_stmt_index: Option<usize>,
    ) -> CfgBlockId {
        match stmt {
            HirStmt::Return { value } => {
                let block =
                    self.new_block(CfgBlockLabel::Statement("return"), top_level_stmt_index);
                let (ty, has_value) = match value {
                    Some(expr) => (expr.ty().clone(), !matches!(expr, HirExpr::NoneLiteral)),
                    None => (Type::None, false),
                };
                self.set_terminator(block, CfgTerminator::Return { ty, has_value });
                block
            }
            HirStmt::Raise { .. } => {
                let block = self.new_block(CfgBlockLabel::Statement("raise"), top_level_stmt_index);
                self.set_terminator(block, CfgTerminator::Raise);
                block
            }
            HirStmt::Break => {
                let target = loop_targets.map_or(next, |targets| targets.break_target);
                let block = self.new_block(CfgBlockLabel::Statement("break"), top_level_stmt_index);
                self.set_terminator(block, CfgTerminator::Goto(target));
                block
            }
            HirStmt::Continue => {
                let target = loop_targets.map_or(next, |targets| targets.continue_target);
                let block =
                    self.new_block(CfgBlockLabel::Statement("continue"), top_level_stmt_index);
                self.set_terminator(block, CfgTerminator::Goto(target));
                block
            }
            HirStmt::If {
                then_body,
                elif_clauses,
                else_body,
                ..
            } => {
                let mut else_entry = if let Some(else_body) = else_body {
                    self.build_stmt_list(else_body, next, loop_targets, false)
                } else {
                    next
                };

                for (_, elif_body) in elif_clauses.iter().rev() {
                    let elif_then = self.build_stmt_list(elif_body, next, loop_targets, false);
                    let elif_cond = self.new_block(CfgBlockLabel::Synthetic, None);
                    self.set_terminator(
                        elif_cond,
                        CfgTerminator::Branch(vec![elif_then, else_entry]),
                    );
                    else_entry = elif_cond;
                }

                let then_entry = self.build_stmt_list(then_body, next, loop_targets, false);
                let if_block = self.new_block(CfgBlockLabel::Statement("if"), top_level_stmt_index);
                self.set_terminator(
                    if_block,
                    CfgTerminator::Branch(vec![then_entry, else_entry]),
                );
                if_block
            }
            HirStmt::While {
                body, else_body, ..
            } => {
                let while_block =
                    self.new_block(CfgBlockLabel::Statement("while"), top_level_stmt_index);
                let false_target = if let Some(else_body) = else_body {
                    self.build_stmt_list(else_body, next, loop_targets, false)
                } else {
                    next
                };
                let loop_targets = LoopTargets {
                    break_target: next,
                    continue_target: while_block,
                };
                let body_entry = self.build_stmt_list(body, while_block, Some(loop_targets), false);
                self.set_terminator(
                    while_block,
                    CfgTerminator::Branch(vec![body_entry, false_target]),
                );
                while_block
            }
            HirStmt::For {
                body, else_body, ..
            }
            | HirStmt::AsyncFor {
                body, else_body, ..
            } => {
                let for_block = self.new_block(
                    CfgBlockLabel::Statement(stmt_label(stmt)),
                    top_level_stmt_index,
                );
                let false_target = if let Some(else_body) = else_body {
                    self.build_stmt_list(else_body, next, loop_targets, false)
                } else {
                    next
                };
                let loop_targets = LoopTargets {
                    break_target: next,
                    continue_target: for_block,
                };
                let body_entry = self.build_stmt_list(body, for_block, Some(loop_targets), false);
                self.set_terminator(
                    for_block,
                    CfgTerminator::Branch(vec![body_entry, false_target]),
                );
                for_block
            }
            HirStmt::Match { arms, .. } => {
                let block = self.new_block(CfgBlockLabel::Statement("match"), top_level_stmt_index);
                if arms.is_empty() {
                    self.set_terminator(block, CfgTerminator::Goto(next));
                } else {
                    let arm_entries: Vec<CfgBlockId> = arms
                        .iter()
                        .map(|arm| self.build_stmt_list(&arm.body, next, loop_targets, false))
                        .collect();
                    self.set_terminator(block, CfgTerminator::Branch(arm_entries));
                }
                block
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                let block =
                    self.new_block(CfgBlockLabel::Statement("try_except"), top_level_stmt_index);
                if handlers.is_empty() {
                    self.set_terminator(block, CfgTerminator::Goto(next));
                } else {
                    let mut targets = Vec::with_capacity(1 + handlers.len());
                    targets.push(self.build_stmt_list(body, next, loop_targets, false));
                    for handler in handlers {
                        targets.push(self.build_stmt_list(
                            &handler.body,
                            next,
                            loop_targets,
                            false,
                        ));
                    }
                    self.set_terminator(block, CfgTerminator::Branch(targets));
                }
                block
            }
            HirStmt::TryFinally { body, finalbody } => {
                let block = self.new_block(
                    CfgBlockLabel::Statement("try_finally"),
                    top_level_stmt_index,
                );
                let final_entry = self.build_stmt_list(finalbody, next, loop_targets, false);
                let body_entry = self.build_stmt_list(body, final_entry, loop_targets, false);
                self.set_terminator(block, CfgTerminator::Goto(body_entry));
                block
            }
            HirStmt::With { body, .. } => {
                let body_entry = self.build_stmt_list(body, next, loop_targets, false);
                let block = self.new_block(CfgBlockLabel::Statement("with"), top_level_stmt_index);
                self.set_terminator(block, CfgTerminator::Goto(body_entry));
                block
            }
            _ => {
                let block = self.new_block(
                    CfgBlockLabel::Statement(stmt_label(stmt)),
                    top_level_stmt_index,
                );
                self.set_terminator(block, CfgTerminator::Goto(next));
                block
            }
        }
    }

    fn finish(mut self, root_entry: CfgBlockId) -> ControlFlowGraph {
        self.set_terminator(self.entry, CfgTerminator::Goto(root_entry));
        self.set_terminator(self.exit, CfgTerminator::Exit);
        ControlFlowGraph::new(
            self.blocks,
            self.entry,
            self.exit,
            self.top_level_stmt_nodes,
        )
    }
}

fn stmt_label(stmt: &HirStmt) -> &'static str {
    match stmt {
        HirStmt::Let { .. } => "let",
        HirStmt::Assign { .. } => "assign",
        HirStmt::AugAssign { .. } => "aug_assign",
        HirStmt::Return { .. } => "return",
        HirStmt::Expr { .. } => "expr",
        HirStmt::If { .. } => "if",
        HirStmt::While { .. } => "while",
        HirStmt::For { .. } => "for",
        HirStmt::AsyncFor { .. } => "async_for",
        HirStmt::Break => "break",
        HirStmt::Continue => "continue",
        HirStmt::TupleUnpack { .. } => "tuple_unpack",
        HirStmt::StarUnpack { .. } => "star_unpack",
        HirStmt::Pass => "pass",
        HirStmt::Assert { .. } => "assert",
        HirStmt::Raise { .. } => "raise",
        HirStmt::TryExcept { .. } => "try_except",
        HirStmt::TryFinally { .. } => "try_finally",
        HirStmt::FieldAssign { .. } => "field_assign",
        HirStmt::NestedFieldAssign { .. } => "nested_field_assign",
        HirStmt::SubscriptAssign { .. } => "subscript_assign",
        HirStmt::NestedSubscriptAssign { .. } => "nested_subscript_assign",
        HirStmt::AttributeNestedSubscriptAssign { .. } => "attribute_nested_subscript_assign",
        HirStmt::SubscriptAugAssign { .. } => "subscript_aug_assign",
        HirStmt::AttributeAugAssign { .. } => "attribute_aug_assign",
        HirStmt::AttributeSubscriptAssign { .. } => "attribute_subscript_assign",
        HirStmt::Delete { .. } => "delete",
        HirStmt::Yield { .. } => "yield",
        HirStmt::With { .. } => "with",
        HirStmt::AsyncWith { .. } => "async_with",
        HirStmt::NestedFunction { .. } => "nested_function",
        HirStmt::Match { .. } => "match",
    }
}

pub fn build_control_flow_graph(stmts: &[HirStmt]) -> ControlFlowGraph {
    let mut builder = CfgBuilder::new(stmts.len());
    let root_entry = builder.build_stmt_list(stmts, builder.exit, None, true);
    let cfg = builder.finish(root_entry);
    if let Err(err) = cfg.validate() {
        panic!("internal compiler error: invalid control-flow graph: {err}");
    }
    cfg
}

pub fn flow_facts(stmts: &[HirStmt]) -> FlowFacts {
    let cfg = build_control_flow_graph(stmts);
    let flow_graph = crate::flow_graph::build_statement_flow_graph(stmts);
    let reachable = cfg.reachable_blocks();

    let mut reachable_top_level_stmt_indices = Vec::new();
    let mut unreachable_top_level_stmt_indices = Vec::new();
    for (idx, block_id) in cfg.top_level_stmt_nodes().iter().enumerate() {
        if reachable[*block_id] {
            reachable_top_level_stmt_indices.push(idx);
        } else {
            unreachable_top_level_stmt_indices.push(idx);
        }
    }

    let mut reachable_return_types = Vec::new();
    let mut has_reachable_return = false;
    let mut has_reachable_value_return = false;
    let mut has_reachable_raise = false;
    for (id, block) in cfg.blocks().iter().enumerate() {
        if !reachable[id] {
            continue;
        }
        match &block.terminator {
            CfgTerminator::Return { ty, has_value } => {
                has_reachable_return = true;
                has_reachable_value_return |= *has_value;
                reachable_return_types.push(ty.clone());
            }
            CfgTerminator::Raise => {
                has_reachable_raise = true;
            }
            CfgTerminator::Goto(_) | CfgTerminator::Branch(_) | CfgTerminator::Exit => {}
        }
    }

    let falls_through = reachable[cfg.exit()];
    let exit_effect = if falls_through {
        FlowExitEffect::FallsThrough
    } else if has_reachable_return && !has_reachable_raise {
        FlowExitEffect::AlwaysReturns
    } else if has_reachable_raise && !has_reachable_return {
        FlowExitEffect::AlwaysRaises
    } else {
        FlowExitEffect::AlwaysExits
    };

    FlowFacts::new(
        exit_effect,
        flow_graph,
        reachable_top_level_stmt_indices,
        unreachable_top_level_stmt_indices,
        reachable_return_types,
        has_reachable_return,
        has_reachable_value_return,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flow_facts_reports_always_raises_for_raise_only_branch() {
        let stmts = vec![HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::Raise {
                value: HirExpr::Call {
                    func: "ValueError".to_string(),
                    args: vec![HirExpr::StringLiteral("bad".to_string())],
                    ty: Type::Unknown,
                },
            }],
            elif_clauses: vec![],
            else_body: Some(vec![HirStmt::Raise {
                value: HirExpr::Call {
                    func: "ValueError".to_string(),
                    args: vec![HirExpr::StringLiteral("also bad".to_string())],
                    ty: Type::Unknown,
                },
            }]),
        }];

        let facts = flow_facts(&stmts);
        assert_eq!(facts.exit_effect(), FlowExitEffect::AlwaysRaises);
        assert!(facts.always_exits());
        assert!(!facts.has_reachable_return());
    }

    #[test]
    fn flow_facts_marks_trailing_stmt_unreachable_after_return() {
        let stmts = vec![
            HirStmt::Return {
                value: Some(HirExpr::IntLiteral(1)),
            },
            HirStmt::Expr {
                expr: HirExpr::IntLiteral(2),
            },
        ];

        let facts = flow_facts(&stmts);
        assert_eq!(facts.reachable_top_level_stmt_indices(), &[0]);
        assert_eq!(facts.unreachable_top_level_stmt_indices(), &[1]);
    }

    #[test]
    fn flow_facts_collects_reachable_return_types_only() {
        let stmts = vec![
            HirStmt::Return {
                value: Some(HirExpr::IntLiteral(1)),
            },
            HirStmt::Return {
                value: Some(HirExpr::StringLiteral("never".to_string())),
            },
        ];

        let facts = flow_facts(&stmts);
        assert_eq!(facts.reachable_return_types(), &[Type::Int]);
    }

    #[test]
    fn control_flow_graph_validate_accepts_valid_graph() {
        let stmts = vec![HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(1)),
            }],
            elif_clauses: vec![],
            else_body: Some(vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(2)),
            }]),
        }];
        let cfg = build_control_flow_graph(&stmts);
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn control_flow_graph_validate_rejects_invalid_edge() {
        let cfg = ControlFlowGraph::new(
            vec![CfgBlock {
                id: 0,
                label: CfgBlockLabel::Entry,
                top_level_stmt_index: None,
                terminator: CfgTerminator::Goto(usize::MAX),
            }],
            0,
            0,
            vec![],
        );
        let err = cfg
            .validate()
            .expect_err("invalid edge should fail validation");
        assert!(err.to_string().contains("invalid successor"));
    }

    #[test]
    fn control_flow_graph_shape_is_deterministic_across_rebuilds() {
        let stmts = vec![
            HirStmt::While {
                condition: HirExpr::BoolLiteral(true),
                body: vec![HirStmt::If {
                    condition: HirExpr::BoolLiteral(true),
                    then_body: vec![HirStmt::Continue],
                    elif_clauses: vec![],
                    else_body: Some(vec![HirStmt::Break]),
                }],
                else_body: Some(vec![HirStmt::Return {
                    value: Some(HirExpr::IntLiteral(7)),
                }]),
            },
            HirStmt::TryExcept {
                body: vec![HirStmt::Return {
                    value: Some(HirExpr::IntLiteral(9)),
                }],
                handlers: vec![crate::HirExceptHandler {
                    error_type: Some("Error".to_string()),
                    error_resolved_type: None,
                    name: Some("e".to_string()),
                    body: vec![HirStmt::Raise {
                        value: HirExpr::Call {
                            func: "ValueError".to_string(),
                            args: vec![HirExpr::StringLiteral("bad".to_string())],
                            ty: Type::Unknown,
                        },
                    }],
                }],
                body_error_types: vec!["Error".to_string()],
            },
        ];

        let cfg_one = build_control_flow_graph(&stmts);
        let cfg_two = build_control_flow_graph(&stmts);
        let facts_one = flow_facts(&stmts);
        let facts_two = flow_facts(&stmts);
        assert_eq!(cfg_one.shape_fingerprint(), cfg_two.shape_fingerprint());
        assert_eq!(facts_one, facts_two);
    }

    #[test]
    fn cfg_repeat_run_matrix_is_deterministic() {
        let corpus: Vec<Vec<HirStmt>> = vec![
            vec![HirStmt::If {
                condition: HirExpr::BoolLiteral(true),
                then_body: vec![HirStmt::Return {
                    value: Some(HirExpr::IntLiteral(1)),
                }],
                elif_clauses: vec![(
                    HirExpr::BoolLiteral(false),
                    vec![HirStmt::Raise {
                        value: HirExpr::Call {
                            func: "ValueError".to_string(),
                            args: vec![HirExpr::StringLiteral("bad".to_string())],
                            ty: Type::Unknown,
                        },
                    }],
                )],
                else_body: Some(vec![HirStmt::Return {
                    value: Some(HirExpr::IntLiteral(2)),
                }]),
            }],
            vec![HirStmt::For {
                target: "n".to_string(),
                target_ty: Type::Int,
                iter: HirExpr::RangeLiteral {
                    start: Box::new(HirExpr::IntLiteral(0)),
                    end: Box::new(HirExpr::IntLiteral(5)),
                    step: None,
                    ty: Type::List(Box::new(Type::Int)),
                },
                body: vec![
                    HirStmt::If {
                        condition: HirExpr::BoolLiteral(true),
                        then_body: vec![HirStmt::Continue],
                        elif_clauses: vec![],
                        else_body: Some(vec![HirStmt::Break]),
                    },
                    HirStmt::Expr {
                        expr: HirExpr::IntLiteral(9),
                    },
                ],
                else_body: Some(vec![HirStmt::Return {
                    value: Some(HirExpr::IntLiteral(7)),
                }]),
            }],
            vec![
                HirStmt::Raise {
                    value: HirExpr::Call {
                        func: "ValueError".to_string(),
                        args: vec![HirExpr::StringLiteral("x".to_string())],
                        ty: Type::Unknown,
                    },
                },
                HirStmt::Return {
                    value: Some(HirExpr::IntLiteral(99)),
                },
            ],
        ];

        for stmts in corpus {
            let cfg_first = build_control_flow_graph(&stmts);
            let cfg_second = build_control_flow_graph(&stmts);
            let facts_first = flow_facts(&stmts);
            let facts_second = flow_facts(&stmts);
            assert_eq!(
                cfg_first.shape_fingerprint(),
                cfg_second.shape_fingerprint()
            );
            assert_eq!(facts_first, facts_second);
        }
    }
}
