//! First-class data-flow graph for HIR statement snapshots.
//!
//! The CFG in `cfg.rs` answers structural reachability questions. This graph
//! keeps data-flow effects explicit so narrowing, ownership, mutation, joins,
//! and exits have a stable debug surface tied to the exact HIR snapshot.

use sifr_type_system::Type;
use std::fmt::Write;

pub type FlowNodeId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowNodeKind {
    Entry {
        scope: String,
    },
    Exit {
        scope: String,
    },
    Statement {
        label: String,
        top_level_stmt_index: Option<usize>,
    },
    Condition {
        label: String,
    },
    Join {
        label: String,
    },
    Loop {
        label: String,
    },
    Unreachable {
        label: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlowEffect {
    Define {
        binding: String,
        ty: Type,
    },
    Assign {
        binding: String,
    },
    ClearNarrowing {
        binding: String,
    },
    Narrow {
        binding: String,
        narrowed_type: Type,
        condition: String,
        is_true: bool,
    },
    Move {
        binding: String,
    },
    ResetMove {
        binding: String,
    },
    Borrow {
        binding: String,
        mutable: bool,
    },
    Mutation {
        target: String,
        operation: String,
    },
    Call {
        callee: String,
    },
    Exit {
        kind: FlowExitKind,
    },
    Join,
    Unreachable,
}

impl FlowEffect {
    fn trace_label(&self) -> String {
        match self {
            Self::Define { binding, ty } => format!("define {binding}: {}", ty.display_name()),
            Self::Assign { binding } => format!("assign {binding}"),
            Self::ClearNarrowing { binding } => format!("clear-narrowing {binding}"),
            Self::Narrow {
                binding,
                narrowed_type,
                condition,
                is_true,
            } => format!(
                "narrow {binding} -> {} when {condition} is {is_true}",
                narrowed_type.display_name()
            ),
            Self::Move { binding } => format!("move {binding}"),
            Self::ResetMove { binding } => format!("reset-move {binding}"),
            Self::Borrow { binding, mutable } => {
                format!("{}borrow {binding}", if *mutable { "mut-" } else { "" })
            }
            Self::Mutation { target, operation } => format!("mutate {target} via {operation}"),
            Self::Call { callee } => format!("call {callee}"),
            Self::Exit { kind } => format!("exit {kind:?}"),
            Self::Join => "join".to_string(),
            Self::Unreachable => "unreachable".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowExitKind {
    Return,
    Raise,
    Break,
    Continue,
    Fallthrough,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowEdgeKind {
    Sequence,
    True,
    False,
    Branch(usize),
    LoopBack,
    Exit,
    SnapshotEffect,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowNode {
    pub id: FlowNodeId,
    pub kind: FlowNodeKind,
    pub effects: Vec<FlowEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowEdge {
    pub from: FlowNodeId,
    pub to: FlowNodeId,
    pub kind: FlowEdgeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowGraph {
    nodes: Vec<FlowNode>,
    edges: Vec<FlowEdge>,
    entry: FlowNodeId,
    exit: FlowNodeId,
}

impl FlowGraph {
    pub fn nodes(&self) -> &[FlowNode] {
        &self.nodes
    }

    pub fn edges(&self) -> &[FlowEdge] {
        &self.edges
    }

    pub fn entry(&self) -> FlowNodeId {
        self.entry
    }

    pub fn exit(&self) -> FlowNodeId {
        self.exit
    }

    pub fn effects(&self) -> impl Iterator<Item = &FlowEffect> {
        self.nodes.iter().flat_map(|node| node.effects.iter())
    }

    pub fn shape_fingerprint(&self) -> String {
        let mut fingerprint = String::new();
        let _ = write!(fingerprint, "entry:{};exit:{};", self.entry, self.exit);
        for node in &self.nodes {
            let _ = write!(fingerprint, "n{}:{:?}:", node.id, node.kind);
            for effect in &node.effects {
                let _ = write!(fingerprint, "{}|", effect.trace_label());
            }
            fingerprint.push(';');
        }
        for edge in &self.edges {
            let _ = write!(fingerprint, "e{}>{}:{:?};", edge.from, edge.to, edge.kind);
        }
        fingerprint
    }

    pub fn debug_trace(&self) -> String {
        let mut trace = String::new();
        for node in &self.nodes {
            let _ = writeln!(trace, "node {} {:?}", node.id, node.kind);
            for effect in &node.effects {
                let _ = writeln!(trace, "  effect {}", effect.trace_label());
            }
        }
        for edge in &self.edges {
            let _ = writeln!(trace, "edge {} -> {} {:?}", edge.from, edge.to, edge.kind);
        }
        trace
    }
}

impl FlowGraph {
    #[must_use]
    pub fn new(
        nodes: Vec<FlowNode>,
        edges: Vec<FlowEdge>,
        entry: FlowNodeId,
        exit: FlowNodeId,
    ) -> Self {
        Self {
            nodes,
            edges,
            entry,
            exit,
        }
    }
}
