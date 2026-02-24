//! Control Flow Graph for type narrowing.
//!
//! Inspired by TypeScript's binder, this module builds a control flow graph
//! during HIR lowering. Each statement/expression gets a `FlowNode` that
//! tracks how variable types change through branches.

use sifr_type_system::Type;

/// Unique identifier for a flow node.
pub type FlowNodeId = usize;

/// A node in the control flow graph.
#[derive(Debug, Clone)]
pub enum FlowNode {
    /// Entry point of a function.
    Start,
    /// A variable assignment that establishes or changes a type.
    Assignment {
        var: String,
        ty: Type,
        antecedent: FlowNodeId,
    },
    /// A conditional branch point (if/elif/while condition).
    Condition {
        /// The antecedent before the condition is evaluated.
        antecedent: FlowNodeId,
        /// Flow node for the true branch.
        true_branch: FlowNodeId,
        /// Flow node for the false branch.
        false_branch: FlowNodeId,
    },
    /// A join point where multiple branches merge (after if/else, loop exit).
    Label { antecedents: Vec<FlowNodeId> },
    /// Unreachable code (after return, break, continue, or exhaustive narrowing).
    Unreachable,
}

/// The control flow graph, built during HIR lowering.
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// Arena of flow nodes.
    nodes: Vec<FlowNode>,
    /// The current flow node (where we're "at" during lowering).
    current: FlowNodeId,
}

impl ControlFlowGraph {
    /// Create a new CFG with a Start node.
    pub fn new() -> Self {
        let mut cfg = Self {
            nodes: Vec::new(),
            current: 0,
        };
        let start = cfg.add_node(FlowNode::Start);
        cfg.current = start;
        cfg
    }

    /// Add a node to the graph and return its ID.
    pub fn add_node(&mut self, node: FlowNode) -> FlowNodeId {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }

    /// Get the current flow node ID.
    pub fn current(&self) -> FlowNodeId {
        self.current
    }

    /// Set the current flow node.
    pub fn set_current(&mut self, id: FlowNodeId) {
        self.current = id;
    }

    /// Record a variable assignment at the current point.
    pub fn record_assignment(&mut self, var: String, ty: Type) -> FlowNodeId {
        let antecedent = self.current;
        let id = self.add_node(FlowNode::Assignment {
            var,
            ty,
            antecedent,
        });
        self.current = id;
        id
    }

    /// Create a condition branch point. Returns (`true_branch_start`, `false_branch_start`).
    pub fn branch(&mut self) -> (FlowNodeId, FlowNodeId) {
        let antecedent = self.current;
        let true_start = self.add_node(FlowNode::Label {
            antecedents: vec![antecedent],
        });
        let false_start = self.add_node(FlowNode::Label {
            antecedents: vec![antecedent],
        });
        let _cond = self.add_node(FlowNode::Condition {
            antecedent,
            true_branch: true_start,
            false_branch: false_start,
        });
        (true_start, false_start)
    }

    /// Create a join point merging multiple branches.
    pub fn join(&mut self, branches: Vec<FlowNodeId>) -> FlowNodeId {
        let id = self.add_node(FlowNode::Label {
            antecedents: branches,
        });
        self.current = id;
        id
    }

    /// Mark the current point as unreachable.
    pub fn mark_unreachable(&mut self) -> FlowNodeId {
        let id = self.add_node(FlowNode::Unreachable);
        self.current = id;
        id
    }

    /// Get a reference to a flow node by ID.
    pub fn get_node(&self, id: FlowNodeId) -> Option<&FlowNode> {
        self.nodes.get(id)
    }

    /// Get the number of nodes in the graph.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the graph is empty (shouldn't happen, always has Start).
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for ControlFlowGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfg_new_has_start() {
        let cfg = ControlFlowGraph::new();
        assert_eq!(cfg.len(), 1);
        assert_eq!(cfg.current(), 0);
        assert!(matches!(cfg.get_node(0), Some(FlowNode::Start)));
    }

    #[test]
    fn test_cfg_assignment() {
        let mut cfg = ControlFlowGraph::new();
        let id = cfg.record_assignment("x".to_string(), Type::Int);
        assert_eq!(id, 1);
        assert_eq!(cfg.current(), 1);
        if let Some(FlowNode::Assignment {
            var,
            ty,
            antecedent,
        }) = cfg.get_node(1)
        {
            assert_eq!(var, "x");
            assert_eq!(*ty, Type::Int);
            assert_eq!(*antecedent, 0); // points to Start
        } else {
            panic!("Expected Assignment node");
        }
    }

    #[test]
    fn test_cfg_branch_and_join() {
        let mut cfg = ControlFlowGraph::new();
        let (true_start, false_start) = cfg.branch();
        assert!(true_start > 0);
        assert!(false_start > 0);

        // Simulate work in true branch
        cfg.set_current(true_start);
        let true_end = cfg.record_assignment("x".to_string(), Type::Int);

        // Simulate work in false branch
        cfg.set_current(false_start);
        let false_end = cfg.record_assignment("x".to_string(), Type::Str);

        // Join
        let join = cfg.join(vec![true_end, false_end]);
        assert_eq!(cfg.current(), join);
    }

    #[test]
    fn test_cfg_unreachable() {
        let mut cfg = ControlFlowGraph::new();
        let id = cfg.mark_unreachable();
        assert!(matches!(cfg.get_node(id), Some(FlowNode::Unreachable)));
    }
}
