//! Control-flow graph construction and flow-truth queries for HIR statement blocks.

use crate::{HirAsyncWithKind, HirExpr, HirStmt, HirWithItemKind};
use sifr_ir::{
    CfgBlock, CfgBlockId, CfgBlockLabel, CfgInvariantError, CfgTerminator, ControlFlowGraph,
    FlowExitEffect, FlowFacts,
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
                let arm_entries: Vec<CfgBlockId> = arms
                    .iter()
                    .map(|arm| self.build_stmt_list(&arm.body, next, loop_targets, false))
                    .collect();
                match arm_entries.as_slice() {
                    [] => self.set_terminator(block, CfgTerminator::Goto(next)),
                    [only_arm] => {
                        self.set_terminator(block, CfgTerminator::Goto(*only_arm));
                    }
                    _ => self.set_terminator(block, CfgTerminator::Branch(arm_entries)),
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
            HirStmt::With { items, body } => {
                let body_entry = self.build_stmt_list(body, next, loop_targets, false);
                let block = self.new_block(CfgBlockLabel::Statement("with"), top_level_stmt_index);
                if items.iter().any(|item| {
                    matches!(
                        item.kind,
                        HirWithItemKind::Python {
                            body_may_raise: true,
                            ..
                        }
                    )
                }) {
                    self.set_terminator(block, CfgTerminator::Branch(vec![body_entry, next]));
                } else {
                    self.set_terminator(block, CfgTerminator::Goto(body_entry));
                }
                block
            }
            HirStmt::AsyncWith { kind, body, .. } => {
                let body_entry = self.build_stmt_list(body, next, loop_targets, false);
                let block =
                    self.new_block(CfgBlockLabel::Statement("async_with"), top_level_stmt_index);
                if matches!(
                    kind,
                    HirAsyncWithKind::Python {
                        body_may_raise: true,
                        ..
                    }
                ) {
                    self.set_terminator(block, CfgTerminator::Branch(vec![body_entry, next]));
                } else {
                    self.set_terminator(block, CfgTerminator::Goto(body_entry));
                }
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

    fn finish(mut self, root_entry: CfgBlockId) -> Result<ControlFlowGraph, CfgInvariantError> {
        self.set_terminator(self.entry, CfgTerminator::Goto(root_entry));
        self.set_terminator(self.exit, CfgTerminator::Exit);
        let cfg = ControlFlowGraph::new(
            self.blocks,
            self.entry,
            self.exit,
            self.top_level_stmt_nodes,
        );
        cfg.validate()?;
        Ok(cfg)
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

pub fn build_control_flow_graph(stmts: &[HirStmt]) -> Result<ControlFlowGraph, CfgInvariantError> {
    let mut builder = CfgBuilder::new(stmts.len());
    let root_entry = builder.build_stmt_list(stmts, builder.exit, None, true);
    builder.finish(root_entry)
}

/// Reachability-only query using the same validated CFG as full flow facts.
/// Callers that do not consume returns or statement-flow edges need not build
/// the separate rich statement graph or clone its return-type collection.
pub(crate) fn block_always_exits(stmts: &[HirStmt]) -> Result<bool, CfgInvariantError> {
    let cfg = build_control_flow_graph(stmts)?;
    Ok(!cfg.reachable_blocks()[cfg.exit()])
}

pub fn flow_facts(stmts: &[HirStmt]) -> Result<FlowFacts, CfgInvariantError> {
    let cfg = build_control_flow_graph(stmts)?;
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

    Ok(FlowFacts::new(
        exit_effect,
        flow_graph,
        reachable_top_level_stmt_indices,
        unreachable_top_level_stmt_indices,
        reachable_return_types,
        has_reachable_return,
        has_reachable_value_return,
    ))
}

#[cfg(test)]
#[path = "../tests/unit/cfg/tests.rs"]
mod tests;
