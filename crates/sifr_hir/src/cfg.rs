//! Canonical control-flow graph and flow-truth queries for HIR statement blocks.

use crate::{HirExpr, HirStmt};
use sifr_type_system::Type;
use std::fmt::Write;

/// Identifier for a CFG block.
pub type CfgBlockId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CfgBlockLabel {
    Entry,
    Exit,
    Statement(&'static str),
    /// Compiler-internal dispatcher/join block (for example: elif chain nodes).
    Synthetic,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CfgTerminator {
    Goto(CfgBlockId),
    Branch(Vec<CfgBlockId>),
    Return { ty: Type, has_value: bool },
    Raise,
    Exit,
}

impl CfgTerminator {
    fn successors(&self) -> &[CfgBlockId] {
        match self {
            Self::Goto(target) => std::slice::from_ref(target),
            Self::Branch(targets) => targets.as_slice(),
            Self::Return { .. } | Self::Raise | Self::Exit => &[],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CfgBlock {
    pub id: CfgBlockId,
    pub label: CfgBlockLabel,
    pub top_level_stmt_index: Option<usize>,
    pub terminator: CfgTerminator,
}

/// Canonical control-flow graph for a lowered HIR statement block.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlFlowGraph {
    blocks: Vec<CfgBlock>,
    entry: CfgBlockId,
    exit: CfgBlockId,
    top_level_stmt_nodes: Vec<CfgBlockId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CfgInvariantError {
    message: String,
}

impl CfgInvariantError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for CfgInvariantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CfgInvariantError {}

impl ControlFlowGraph {
    pub fn blocks(&self) -> &[CfgBlock] {
        &self.blocks
    }

    pub fn entry(&self) -> CfgBlockId {
        self.entry
    }

    pub fn exit(&self) -> CfgBlockId {
        self.exit
    }

    pub fn top_level_stmt_nodes(&self) -> &[CfgBlockId] {
        &self.top_level_stmt_nodes
    }

    pub fn reachable_blocks(&self) -> Vec<bool> {
        let mut reachable = vec![false; self.blocks.len()];
        let mut stack = vec![self.entry];
        while let Some(block_id) = stack.pop() {
            if reachable[block_id] {
                continue;
            }
            reachable[block_id] = true;
            for &next in self.blocks[block_id].terminator.successors().iter().rev() {
                if !reachable[next] {
                    stack.push(next);
                }
            }
        }
        reachable
    }

    pub fn shape_fingerprint(&self) -> String {
        let mut fingerprint = String::new();
        let _ = write!(fingerprint, "entry:{};exit:{};", self.entry, self.exit);
        for block in &self.blocks {
            let _ = write!(
                fingerprint,
                "b{}:{:?}:{:?}:",
                block.id, block.label, block.top_level_stmt_index
            );
            match &block.terminator {
                CfgTerminator::Goto(target) => {
                    let _ = write!(fingerprint, "goto:{target};");
                }
                CfgTerminator::Branch(targets) => {
                    fingerprint.push_str("branch:");
                    for target in targets {
                        let _ = write!(fingerprint, "{target},");
                    }
                    fingerprint.push(';');
                }
                CfgTerminator::Return { ty, has_value } => {
                    let _ = write!(fingerprint, "return:{}:{};", ty.display_name(), has_value);
                }
                CfgTerminator::Raise => {
                    fingerprint.push_str("raise;");
                }
                CfgTerminator::Exit => {
                    fingerprint.push_str("exit;");
                }
            }
        }
        fingerprint
    }

    pub fn validate(&self) -> Result<(), CfgInvariantError> {
        if self.blocks.is_empty() {
            return Err(CfgInvariantError::new("cfg has no blocks"));
        }
        if self.entry >= self.blocks.len() {
            return Err(CfgInvariantError::new(format!(
                "entry block id {} is out of range for {} blocks",
                self.entry,
                self.blocks.len()
            )));
        }
        if self.exit >= self.blocks.len() {
            return Err(CfgInvariantError::new(format!(
                "exit block id {} is out of range for {} blocks",
                self.exit,
                self.blocks.len()
            )));
        }
        for (idx, block) in self.blocks.iter().enumerate() {
            if block.id != idx {
                return Err(CfgInvariantError::new(format!(
                    "block id mismatch at index {idx}: found id {}, expected {}",
                    block.id, idx
                )));
            }
            if let CfgTerminator::Branch(targets) = &block.terminator {
                if targets.len() < 2 {
                    return Err(CfgInvariantError::new(format!(
                        "branch terminator in block {} is incomplete ({} target(s))",
                        block.id,
                        targets.len()
                    )));
                }
            }
            for &target in block.terminator.successors() {
                if target >= self.blocks.len() {
                    return Err(CfgInvariantError::new(format!(
                        "block {} has invalid successor {} (max {})",
                        block.id,
                        target,
                        self.blocks.len().saturating_sub(1)
                    )));
                }
            }
        }
        let mut seen_top = vec![false; self.top_level_stmt_nodes.len()];
        for (idx, &block_id) in self.top_level_stmt_nodes.iter().enumerate() {
            if block_id >= self.blocks.len() {
                return Err(CfgInvariantError::new(format!(
                    "top-level stmt {idx} maps to invalid block id {block_id}",
                )));
            }
            let mapped = self.blocks[block_id].top_level_stmt_index;
            if mapped != Some(idx) {
                return Err(CfgInvariantError::new(format!(
                    "top-level stmt {idx} maps to block {block_id} with mismatched marker {mapped:?}",
                )));
            }
            if seen_top[idx] {
                return Err(CfgInvariantError::new(format!(
                    "duplicate mapping for top-level stmt {idx}",
                )));
            }
            seen_top[idx] = true;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowExitEffect {
    FallsThrough,
    AlwaysReturns,
    AlwaysRaises,
    AlwaysExits,
}

impl FlowExitEffect {
    pub fn always_exits(self) -> bool {
        !matches!(self, Self::FallsThrough)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowFacts {
    exit_effect: FlowExitEffect,
    flow_graph: crate::flow_graph::FlowGraph,
    reachable_top_level_stmt_indices: Vec<usize>,
    unreachable_top_level_stmt_indices: Vec<usize>,
    reachable_return_types: Vec<Type>,
    has_reachable_return: bool,
    has_reachable_value_return: bool,
}

impl FlowFacts {
    pub fn exit_effect(&self) -> FlowExitEffect {
        self.exit_effect
    }

    pub fn always_exits(&self) -> bool {
        self.exit_effect.always_exits()
    }

    pub fn has_reachable_return(&self) -> bool {
        self.has_reachable_return
    }

    pub fn has_reachable_value_return(&self) -> bool {
        self.has_reachable_value_return
    }

    pub fn reachable_return_types(&self) -> &[Type] {
        &self.reachable_return_types
    }

    pub fn reachable_top_level_stmt_indices(&self) -> &[usize] {
        &self.reachable_top_level_stmt_indices
    }

    pub fn unreachable_top_level_stmt_indices(&self) -> &[usize] {
        &self.unreachable_top_level_stmt_indices
    }

    pub fn flow_graph(&self) -> &crate::flow_graph::FlowGraph {
        &self.flow_graph
    }

    pub fn flow_graph_fingerprint(&self) -> String {
        self.flow_graph.shape_fingerprint()
    }

    pub fn flow_graph_debug_trace(&self) -> String {
        self.flow_graph.debug_trace()
    }
}

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
        ControlFlowGraph {
            blocks: self.blocks,
            entry: self.entry,
            exit: self.exit,
            top_level_stmt_nodes: self.top_level_stmt_nodes,
        }
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

    FlowFacts {
        exit_effect,
        flow_graph,
        reachable_top_level_stmt_indices,
        unreachable_top_level_stmt_indices,
        reachable_return_types,
        has_reachable_return,
        has_reachable_value_return,
    }
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
        let mut cfg = build_control_flow_graph(&[HirStmt::Expr {
            expr: HirExpr::IntLiteral(1),
        }]);
        cfg.blocks[0].terminator = CfgTerminator::Goto(usize::MAX);
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
