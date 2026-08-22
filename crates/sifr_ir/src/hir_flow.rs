use crate::{HirExpr, HirStmt, HirWithItemKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirControlFlowEffect {
    FallsThrough,
    AlwaysReturns,
    AlwaysRaises,
    AlwaysExits,
}

impl HirControlFlowEffect {
    pub fn always_exits(self) -> bool {
        !matches!(self, Self::FallsThrough)
    }
}

pub fn block_control_flow_effect(stmts: &[HirStmt]) -> HirControlFlowEffect {
    flow_summary(stmts).control_flow_effect()
}

pub fn reachable_top_level_stmt_indices(stmts: &[HirStmt]) -> Vec<usize> {
    let mut reachable = Vec::new();
    let mut falls_through = true;
    for (index, stmt) in stmts.iter().enumerate() {
        if !falls_through {
            break;
        }
        reachable.push(index);
        falls_through = stmt_summary(stmt).falls_through;
    }
    reachable
}

pub fn unreachable_top_level_stmt_indices(stmts: &[HirStmt]) -> Vec<usize> {
    let reachable_len = reachable_top_level_stmt_indices(stmts).len();
    (reachable_len..stmts.len()).collect()
}

pub fn body_contains_return(stmts: &[HirStmt]) -> bool {
    flow_summary(stmts).has_return
}

pub fn try_body_has_value_return(stmts: &[HirStmt]) -> bool {
    flow_summary(stmts).has_value_return
}

#[derive(Debug, Clone, Copy)]
struct FlowSummary {
    falls_through: bool,
    has_return: bool,
    has_value_return: bool,
    has_raise: bool,
}

impl FlowSummary {
    const fn fallthrough() -> Self {
        Self {
            falls_through: true,
            has_return: false,
            has_value_return: false,
            has_raise: false,
        }
    }

    const fn return_stmt(has_value: bool) -> Self {
        Self {
            falls_through: false,
            has_return: true,
            has_value_return: has_value,
            has_raise: false,
        }
    }

    const fn raise_stmt() -> Self {
        Self {
            falls_through: false,
            has_return: false,
            has_value_return: false,
            has_raise: true,
        }
    }

    fn union(self, other: Self) -> Self {
        Self {
            falls_through: self.falls_through || other.falls_through,
            has_return: self.has_return || other.has_return,
            has_value_return: self.has_value_return || other.has_value_return,
            has_raise: self.has_raise || other.has_raise,
        }
    }

    fn sequence(self, next: Self) -> Self {
        if self.falls_through {
            Self {
                falls_through: next.falls_through,
                has_return: self.has_return || next.has_return,
                has_value_return: self.has_value_return || next.has_value_return,
                has_raise: self.has_raise || next.has_raise,
            }
        } else {
            self
        }
    }

    fn control_flow_effect(self) -> HirControlFlowEffect {
        if self.falls_through {
            HirControlFlowEffect::FallsThrough
        } else if self.has_return && !self.has_raise {
            HirControlFlowEffect::AlwaysReturns
        } else if self.has_raise && !self.has_return {
            HirControlFlowEffect::AlwaysRaises
        } else {
            HirControlFlowEffect::AlwaysExits
        }
    }
}

fn flow_summary(stmts: &[HirStmt]) -> FlowSummary {
    stmts
        .iter()
        .fold(FlowSummary::fallthrough(), |summary, stmt| {
            summary.sequence(stmt_summary(stmt))
        })
}

fn branch_summary<'a>(branches: impl IntoIterator<Item = &'a [HirStmt]>) -> FlowSummary {
    branches
        .into_iter()
        .map(flow_summary)
        .reduce(FlowSummary::union)
        .unwrap_or_else(FlowSummary::fallthrough)
}

fn stmt_summary(stmt: &HirStmt) -> FlowSummary {
    match stmt {
        HirStmt::Return { value } => FlowSummary::return_stmt(
            value
                .as_ref()
                .is_some_and(|expr| !matches!(expr, HirExpr::NoneLiteral)),
        ),
        HirStmt::Raise { .. } => FlowSummary::raise_stmt(),
        HirStmt::If {
            then_body,
            elif_clauses,
            else_body,
            ..
        } => {
            let elif_bodies = elif_clauses.iter().map(|(_, body)| body.as_slice());
            let else_branch = else_body
                .as_deref()
                .map_or_else(FlowSummary::fallthrough, flow_summary);
            branch_summary(std::iter::once(then_body.as_slice()).chain(elif_bodies))
                .union(else_branch)
        }
        HirStmt::While {
            body, else_body, ..
        }
        | HirStmt::For {
            body, else_body, ..
        }
        | HirStmt::AsyncFor {
            body, else_body, ..
        } => {
            let body_summary = flow_summary(body);
            let else_summary = else_body
                .as_deref()
                .map_or_else(FlowSummary::fallthrough, flow_summary);
            body_summary.union(else_summary)
        }
        HirStmt::Match { arms, .. } => branch_summary(arms.iter().map(|arm| arm.body.as_slice())),
        HirStmt::TryExcept { body, handlers, .. } => branch_summary(
            std::iter::once(body.as_slice())
                .chain(handlers.iter().map(|handler| handler.body.as_slice())),
        ),
        HirStmt::TryFinally { body, finalbody } => {
            flow_summary(body).sequence(flow_summary(finalbody))
        }
        HirStmt::With { items, body }
            if items
                .iter()
                .any(|item| matches!(item.kind, HirWithItemKind::Python { .. })) =>
        {
            python_context_flow_summary(body)
        }
        HirStmt::AsyncWith {
            kind: crate::HirAsyncWithKind::Python { .. },
            body,
            ..
        } => python_context_flow_summary(body),
        HirStmt::With { body, .. } | HirStmt::AsyncWith { body, .. } => flow_summary(body),
        _ => FlowSummary::fallthrough(),
    }
}

fn python_context_flow_summary(body: &[HirStmt]) -> FlowSummary {
    let mut summary = flow_summary(body);
    if summary.has_raise {
        summary.falls_through = true;
    }
    summary
}
