//! Flow-graph construction for HIR statement snapshots.

use crate::{HirClass, HirExpr, HirFunction, HirMatchArm, HirModule, HirStmt};
use sifr_ir::{FlowEdge, FlowEdgeKind, FlowEffect, FlowGraph, FlowNode, FlowNodeId, FlowNodeKind};
use sifr_type_system::{NarrowingCondition, Type};

mod effects;

use effects::{stmt_effects, stmt_label};

pub fn build_statement_flow_graph(stmts: &[HirStmt]) -> FlowGraph {
    let mut builder = FlowGraphBuilder::new("statement-snapshot");
    let frontier =
        builder.build_stmt_list(vec![builder.entry], FlowEdgeKind::Sequence, stmts, true);
    builder.finish(&frontier)
}

pub fn build_module_flow_graph(module: &HirModule, lowering_effects: &[FlowEffect]) -> FlowGraph {
    let mut builder = FlowGraphBuilder::new("module-snapshot");
    let mut frontier = vec![builder.entry];

    for function in &module.functions {
        frontier = builder.build_function(function, &frontier);
    }
    for class in &module.classes {
        frontier = builder.build_class(class, &frontier);
    }
    for effect in lowering_effects {
        let node = builder.new_node(
            FlowNodeKind::Statement {
                label: "lowering_effect".to_string(),
                top_level_stmt_index: None,
            },
            vec![effect.clone()],
        );
        builder.connect_frontier(&frontier, node, FlowEdgeKind::SnapshotEffect);
        frontier = vec![node];
    }

    builder.finish(&frontier)
}

pub fn narrowing_effects_for_condition(
    condition: &NarrowingCondition,
    is_true: bool,
    current_type: &Type,
) -> Vec<FlowEffect> {
    match condition {
        NarrowingCondition::And(conditions) if is_true => conditions
            .iter()
            .flat_map(|condition| narrowing_effects_for_condition(condition, true, current_type))
            .collect(),
        NarrowingCondition::Or(conditions) if !is_true => conditions
            .iter()
            .flat_map(|condition| narrowing_effects_for_condition(condition, false, current_type))
            .collect(),
        _ => condition.var_name().map_or_else(Vec::new, |binding| {
            vec![FlowEffect::Narrow {
                binding: binding.to_string(),
                narrowed_type: sifr_type_system::narrow_type(current_type, condition, is_true),
                condition: format!("{condition:?}"),
                is_true,
            }]
        }),
    }
}

struct FlowGraphBuilder {
    nodes: Vec<FlowNode>,
    edges: Vec<FlowEdge>,
    entry: FlowNodeId,
    exit: FlowNodeId,
}

impl FlowGraphBuilder {
    fn new(scope: &str) -> Self {
        let mut builder = Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            entry: 0,
            exit: 0,
        };
        builder.entry = builder.new_node(
            FlowNodeKind::Entry {
                scope: scope.to_string(),
            },
            Vec::new(),
        );
        builder.exit = builder.new_node(
            FlowNodeKind::Exit {
                scope: scope.to_string(),
            },
            Vec::new(),
        );
        builder
    }

    fn new_node(&mut self, kind: FlowNodeKind, effects: Vec<FlowEffect>) -> FlowNodeId {
        let id = self.nodes.len();
        self.nodes.push(FlowNode { id, kind, effects });
        id
    }

    fn add_edge(&mut self, from: FlowNodeId, to: FlowNodeId, kind: FlowEdgeKind) {
        self.edges.push(FlowEdge { from, to, kind });
    }

    fn connect_frontier(&mut self, frontier: &[FlowNodeId], to: FlowNodeId, kind: FlowEdgeKind) {
        for &from in frontier {
            self.add_edge(from, to, kind);
        }
    }

    fn finish(mut self, frontier: &[FlowNodeId]) -> FlowGraph {
        self.connect_frontier(frontier, self.exit, FlowEdgeKind::Exit);
        FlowGraph::new(self.nodes, self.edges, self.entry, self.exit)
    }

    fn build_class(&mut self, class: &HirClass, frontier: &[FlowNodeId]) -> Vec<FlowNodeId> {
        let node = self.new_node(
            FlowNodeKind::Statement {
                label: format!("class {}", class.name),
                top_level_stmt_index: None,
            },
            Vec::new(),
        );
        self.connect_frontier(frontier, node, FlowEdgeKind::Sequence);
        let mut next = vec![node];
        for method in &class.methods {
            next = self.build_function(method, &next);
        }
        next
    }

    fn build_function(
        &mut self,
        function: &HirFunction,
        frontier: &[FlowNodeId],
    ) -> Vec<FlowNodeId> {
        let node = self.new_node(
            FlowNodeKind::Statement {
                label: format!("function {}", function.name),
                top_level_stmt_index: None,
            },
            Vec::new(),
        );
        self.connect_frontier(frontier, node, FlowEdgeKind::Sequence);
        self.build_stmt_list(vec![node], FlowEdgeKind::Sequence, &function.body, true)
    }

    fn build_stmt_list(
        &mut self,
        mut frontier: Vec<FlowNodeId>,
        first_edge_kind: FlowEdgeKind,
        stmts: &[HirStmt],
        top_level: bool,
    ) -> Vec<FlowNodeId> {
        let mut edge_kind = first_edge_kind;
        for (index, stmt) in stmts.iter().enumerate() {
            let effects = stmt_effects(stmt);
            let kind = if frontier.is_empty() {
                FlowNodeKind::Unreachable {
                    label: stmt_label(stmt).to_string(),
                }
            } else if matches!(stmt, HirStmt::If { .. }) {
                FlowNodeKind::Condition {
                    label: "if".to_string(),
                }
            } else if matches!(
                stmt,
                HirStmt::While { .. } | HirStmt::For { .. } | HirStmt::AsyncFor { .. }
            ) {
                FlowNodeKind::Loop {
                    label: stmt_label(stmt).to_string(),
                }
            } else {
                FlowNodeKind::Statement {
                    label: stmt_label(stmt).to_string(),
                    top_level_stmt_index: top_level.then_some(index),
                }
            };
            let node = self.new_node(kind, effects);
            self.connect_frontier(&frontier, node, edge_kind);
            frontier = self.build_stmt(stmt, node);
            edge_kind = FlowEdgeKind::Sequence;
        }
        frontier
    }

    fn build_stmt(&mut self, stmt: &HirStmt, node: FlowNodeId) -> Vec<FlowNodeId> {
        match stmt {
            HirStmt::Return { .. } | HirStmt::Raise { .. } | HirStmt::Break | HirStmt::Continue => {
                Vec::new()
            }
            HirStmt::If {
                then_body,
                elif_clauses,
                else_body,
                ..
            } => self.build_if(node, then_body, elif_clauses, else_body.as_deref()),
            HirStmt::While {
                body, else_body, ..
            }
            | HirStmt::For {
                body, else_body, ..
            }
            | HirStmt::AsyncFor {
                body, else_body, ..
            } => self.build_loop(node, body, else_body.as_deref()),
            HirStmt::TryExcept { body, handlers, .. } => {
                let join = self.join_node("try_except");
                let body_frontier =
                    self.build_stmt_list(vec![node], FlowEdgeKind::Branch(0), body, false);
                self.connect_frontier(&body_frontier, join, FlowEdgeKind::Sequence);
                for (idx, handler) in handlers.iter().enumerate() {
                    let handler_frontier = self.build_stmt_list(
                        vec![node],
                        FlowEdgeKind::Branch(idx + 1),
                        &handler.body,
                        false,
                    );
                    self.connect_frontier(&handler_frontier, join, FlowEdgeKind::Sequence);
                }
                vec![join]
            }
            HirStmt::TryFinally { body, finalbody } => {
                let body_frontier =
                    self.build_stmt_list(vec![node], FlowEdgeKind::Sequence, body, false);
                self.build_stmt_list(body_frontier, FlowEdgeKind::Sequence, finalbody, false)
            }
            HirStmt::With { body, .. } | HirStmt::AsyncWith { body, .. } => {
                self.build_stmt_list(vec![node], FlowEdgeKind::Sequence, body, false)
            }
            HirStmt::Match { arms, .. } => self.build_match(node, arms),
            HirStmt::NestedFunction { func, .. } => self.build_function(func, &[node]),
            _ => vec![node],
        }
    }

    fn build_if(
        &mut self,
        node: FlowNodeId,
        then_body: &[HirStmt],
        elif_clauses: &[(HirExpr, Vec<HirStmt>)],
        else_body: Option<&[HirStmt]>,
    ) -> Vec<FlowNodeId> {
        let join = self.join_node("if");
        let then_frontier = self.build_stmt_list(vec![node], FlowEdgeKind::True, then_body, false);
        self.connect_frontier(&then_frontier, join, FlowEdgeKind::Sequence);

        let mut false_source = node;
        for (idx, (_, body)) in elif_clauses.iter().enumerate() {
            let elif_node = self.new_node(
                FlowNodeKind::Condition {
                    label: format!("elif {idx}"),
                },
                Vec::new(),
            );
            self.add_edge(false_source, elif_node, FlowEdgeKind::False);
            let body_frontier =
                self.build_stmt_list(vec![elif_node], FlowEdgeKind::True, body, false);
            self.connect_frontier(&body_frontier, join, FlowEdgeKind::Sequence);
            false_source = elif_node;
        }

        if let Some(else_body) = else_body {
            let else_frontier =
                self.build_stmt_list(vec![false_source], FlowEdgeKind::False, else_body, false);
            self.connect_frontier(&else_frontier, join, FlowEdgeKind::Sequence);
        } else {
            self.add_edge(false_source, join, FlowEdgeKind::False);
        }
        vec![join]
    }

    fn build_loop(
        &mut self,
        node: FlowNodeId,
        body: &[HirStmt],
        else_body: Option<&[HirStmt]>,
    ) -> Vec<FlowNodeId> {
        let body_frontier = self.build_stmt_list(vec![node], FlowEdgeKind::True, body, false);
        self.connect_frontier(&body_frontier, node, FlowEdgeKind::LoopBack);
        if let Some(else_body) = else_body {
            self.build_stmt_list(vec![node], FlowEdgeKind::False, else_body, false)
        } else {
            let join = self.join_node("loop");
            self.add_edge(node, join, FlowEdgeKind::False);
            vec![join]
        }
    }

    fn build_match(&mut self, node: FlowNodeId, arms: &[HirMatchArm]) -> Vec<FlowNodeId> {
        let join = self.join_node("match");
        if arms.is_empty() {
            self.add_edge(node, join, FlowEdgeKind::Sequence);
            return vec![join];
        }
        for (idx, arm) in arms.iter().enumerate() {
            let frontier =
                self.build_stmt_list(vec![node], FlowEdgeKind::Branch(idx), &arm.body, false);
            self.connect_frontier(&frontier, join, FlowEdgeKind::Sequence);
        }
        vec![join]
    }

    fn join_node(&mut self, label: &str) -> FlowNodeId {
        self.new_node(
            FlowNodeKind::Join {
                label: label.to_string(),
            },
            vec![FlowEffect::Join],
        )
    }
}

#[cfg(test)]
mod tests;
