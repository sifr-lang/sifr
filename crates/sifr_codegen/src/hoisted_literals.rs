use sifr_hir::{HirExpr, HirFunction, HirStmt};

use crate::{
    hir_analysis::traversal::{walk_expr, walk_stmts, TraversalConfig},
    HashMap, HashSet, RustEmitter, RustExpr, RustItem, RustStmt, RustType, Type, Visibility,
};

impl RustEmitter {
    pub(crate) fn collect_hoistable_static_dict_locals(
        &self,
        func: &HirFunction,
    ) -> HashSet<String> {
        let candidates = func
            .body
            .iter()
            .filter_map(|stmt| match stmt {
                HirStmt::Let { name, value, .. }
                    if !self.mutated_vars.contains(name)
                        && Self::is_static_eligible_dict_literal(value) =>
                {
                    Some(name.clone())
                }
                _ => None,
            })
            .collect::<HashSet<_>>();
        if candidates.is_empty() {
            return HashSet::new();
        }

        let mut all_uses = HashMap::<String, usize>::new();
        let mut allowed_uses = HashMap::<String, usize>::new();
        walk_stmts(
            &func.body,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut |_| {},
            &mut |expr| {
                if let HirExpr::Name { name, .. } = expr {
                    if candidates.contains(name) {
                        *all_uses.entry(name.clone()).or_default() += 1;
                    }
                }
                Self::count_allowed_static_dict_uses(expr, &candidates, &mut allowed_uses);
            },
        );

        candidates
            .into_iter()
            .filter(|name| {
                all_uses.get(name).copied().unwrap_or_default()
                    == allowed_uses.get(name).copied().unwrap_or_default()
            })
            .collect()
    }

    pub(crate) fn try_hoist_static_readonly_dict_literal(
        &mut self,
        name: &str,
        effective_ty: &Type,
        value: &HirExpr,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        if !self.hoistable_static_dict_locals.contains(name)
            || !Self::is_static_eligible_dict_literal(value)
        {
            return Ok(None);
        }
        let Some(lowered_value) = self.lower_rendered_expr_for_ir(value)? else {
            return Ok(None);
        };

        let static_name = format!("__SIFR_HOISTED_DICT_{}", self.hoisted_literal_counter);
        self.hoisted_literal_counter += 1;
        let dict_ty = self.rust_ir_type_with_generics(effective_ty);
        self.body_items.push(RustItem::Static {
            name: static_name.clone(),
            visibility: Visibility::Private,
            ty: RustType::Named(format!(
                "std::sync::LazyLock<{}>",
                crate::render::Renderer::render_type_string(&dict_ty)
            )),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "sync".to_string(),
                    "LazyLock".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::Closure {
                    params: vec![],
                    body: Box::new(lowered_value),
                    is_move: false,
                }],
            },
        });

        Ok(Some(RustStmt::Let {
            mutable: false,
            name: name.to_string(),
            ty: None,
            value: RustExpr::Ref {
                mutable: false,
                expr: Box::new(RustExpr::Deref(Box::new(RustExpr::Ident(static_name)))),
            },
        }))
    }

    fn count_allowed_static_dict_uses(
        expr: &HirExpr,
        candidates: &HashSet<String>,
        allowed_uses: &mut HashMap<String, usize>,
    ) {
        match expr {
            HirExpr::Compare {
                ops, comparators, ..
            } => {
                for (op, comparator) in ops.iter().zip(comparators.iter()) {
                    if op == "in" || op == "not in" {
                        Self::count_direct_candidate_name(comparator, candidates, allowed_uses);
                    }
                }
            }
            HirExpr::ContainsOp { collection, .. } => {
                Self::count_direct_candidate_name(collection, candidates, allowed_uses);
            }
            HirExpr::Index { object, .. } => {
                Self::count_direct_candidate_name(object, candidates, allowed_uses);
            }
            _ => {}
        }
    }

    fn count_direct_candidate_name(
        expr: &HirExpr,
        candidates: &HashSet<String>,
        allowed_uses: &mut HashMap<String, usize>,
    ) {
        if let HirExpr::Name { name, .. } = expr {
            if candidates.contains(name) {
                *allowed_uses.entry(name.clone()).or_default() += 1;
            }
        }
    }

    fn is_static_eligible_dict_literal(expr: &HirExpr) -> bool {
        let HirExpr::DictLiteral { keys, values, .. } = expr else {
            return false;
        };
        keys.iter()
            .chain(values.iter())
            .all(Self::is_static_eligible_literal_expr)
    }

    fn is_static_eligible_literal_expr(expr: &HirExpr) -> bool {
        let mut eligible = true;
        walk_expr(expr, &mut |node| match node {
            HirExpr::StringLiteral(_)
            | HirExpr::IntLiteral(_)
            | HirExpr::LargeIntLiteral(_)
            | HirExpr::BoolLiteral(_)
            | HirExpr::NoneLiteral => {}
            other if std::ptr::eq(other, expr) => {}
            _ => eligible = false,
        });
        eligible
    }
}
