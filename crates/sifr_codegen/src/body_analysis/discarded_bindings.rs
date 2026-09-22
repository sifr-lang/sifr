use super::{BodyAnalysis, BodySummary, HirStmt, stmt_key};
use std::borrow::Cow;

#[derive(Clone)]
pub(super) enum UnusedProjection {
    Omit,
    Receiver(crate::HirExpr),
}

impl BodyAnalysis {
    pub(crate) fn statement_for_lowering<'a>(
        &self,
        statement: &'a HirStmt,
    ) -> Option<Cow<'a, HirStmt>> {
        match self.unused_projection_statements.get(&stmt_key(statement)) {
            None => Some(Cow::Borrowed(statement)),
            Some(UnusedProjection::Omit) => None,
            Some(UnusedProjection::Receiver(receiver)) => Some(Cow::Owned(HirStmt::Expr {
                expr: receiver.clone(),
            })),
        }
    }

    pub(super) fn remove_unused_projection_summaries(
        &mut self,
        statements: &[HirStmt],
        mut block: BodySummary,
    ) -> BodySummary {
        // Recompute dependency summaries when a dead source read disappears.
        // This also prevents an otherwise unused handler carrier from being kept.
        loop {
            let mut changed = false;
            for statement in statements {
                let key = stmt_key(statement);
                if self.unused_projection_statements.contains_key(&key) {
                    continue;
                }
                let value = match statement {
                    HirStmt::Let { name, value, .. } if !block.uses_binding(name) => value,
                    HirStmt::Expr { expr } => expr,
                    _ => continue,
                };
                if let Some(receiver) =
                    crate::discardability::hir_unused_string_projection_receiver(value)
                {
                    if matches!(receiver, crate::HirExpr::Name { .. }) {
                        self.unused_projection_statements
                            .insert(key, UnusedProjection::Omit);
                        self.statements.insert(key, BodySummary::default());
                    } else {
                        self.unused_projection_statements
                            .insert(key, UnusedProjection::Receiver(receiver.clone()));
                    }
                    changed = true;
                }
            }
            if !changed {
                return block;
            }
            block = BodySummary::default();
            for statement in statements {
                if let Some(summary) = self.statements.get(&stmt_key(statement)) {
                    block.merge(summary);
                }
            }
        }
    }
}
