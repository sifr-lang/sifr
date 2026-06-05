//! Canonical control-flow graph and flow-truth queries for HIR statement blocks.

use crate::FlowGraph;
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
    flow_graph: FlowGraph,
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

    pub fn flow_graph(&self) -> &FlowGraph {
        &self.flow_graph
    }

    pub fn flow_graph_fingerprint(&self) -> String {
        self.flow_graph.shape_fingerprint()
    }

    pub fn flow_graph_debug_trace(&self) -> String {
        self.flow_graph.debug_trace()
    }
}

impl ControlFlowGraph {
    #[must_use]
    pub fn new(
        blocks: Vec<CfgBlock>,
        entry: CfgBlockId,
        exit: CfgBlockId,
        top_level_stmt_nodes: Vec<CfgBlockId>,
    ) -> Self {
        Self {
            blocks,
            entry,
            exit,
            top_level_stmt_nodes,
        }
    }
}

impl FlowFacts {
    #[must_use]
    pub fn new(
        exit_effect: FlowExitEffect,
        flow_graph: FlowGraph,
        reachable_top_level_stmt_indices: Vec<usize>,
        unreachable_top_level_stmt_indices: Vec<usize>,
        reachable_return_types: Vec<Type>,
        has_reachable_return: bool,
        has_reachable_value_return: bool,
    ) -> Self {
        Self {
            exit_effect,
            flow_graph,
            reachable_top_level_stmt_indices,
            unreachable_top_level_stmt_indices,
            reachable_return_types,
            has_reachable_return,
            has_reachable_value_return,
        }
    }
}
