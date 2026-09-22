use super::{RustEmitter, RustStmt};

impl RustEmitter {
    pub(crate) fn prepare_checked_place_witnesses_for_mutation(
        &mut self,
        stmt: &crate::HirStmt,
        following: Option<&[crate::HirStmt]>,
    ) -> Vec<RustStmt> {
        let mut affected = self.checked_place_witnesses_affected_by_stmt(stmt);
        for (key, witness) in &self.checked_place_read_witnesses {
            if witness.exclusive_owner.as_ref().is_some_and(|owner| {
                self.body_analysis
                    .references_outside_checked_read(stmt, owner, key)
            }) && !affected.iter().any(|(existing, _)| existing == key)
            {
                affected.push((key.clone(), witness.clone()));
            }
        }
        affected.sort_by_key(|(_, witness)| witness.order);
        let mut preparations = Vec::new();
        for (key, mut witness) in affected {
            let following_uses =
                following.is_some_and(|tail| self.body_analysis.checked_read_is_used(tail, &key));
            let exclusive_use = witness.exclusive_owner.as_ref().is_some_and(|owner| {
                self.body_analysis
                    .references_outside_checked_read(stmt, owner, &key)
            }) && (following_uses
                || self
                    .body_analysis
                    .checked_read_is_used(std::slice::from_ref(stmt), &key));
            if !witness.borrowed
                || (!exclusive_use
                    && (Self::checked_place_witness_is_invalidated_by_stmt(&witness, stmt)
                        || !following_uses))
            {
                continue;
            }
            preparations.push(RustStmt::Let {
                mutable: false,
                name: witness.binding.clone(),
                ty: None,
                value: crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(crate::RustExpr::Deref(
                        Box::new(crate::RustExpr::Ident(witness.binding.clone())),
                    )))),
                    method: "clone".to_string(),
                    args: Vec::new(),
                },
            });
            witness.borrowed = false;
            witness.exclusive_owner = None;
            witness.option = crate::RustExpr::MethodCall {
                receiver: Box::new(witness.option),
                method: "cloned".to_string(),
                args: Vec::new(),
            };
            self.checked_place_read_witnesses.insert(key, witness);
        }
        preparations
    }
}
