use super::{
    BodyAnalysis, CallParamConventions, HashMap, HashSet, HirExpr, HirStmt, direct_stmt_summary,
    expr_key, stmt_key,
};

fn invalidate_aliases(
    aliases: &mut HashMap<String, String>,
    mutations: &HashSet<String>,
    shadow: Option<&str>,
) {
    aliases.retain(|alias, collection| {
        shadow != Some(alias.as_str())
            && shadow != collection.strip_prefix("name:")
            && !mutations.contains(alias)
            && !collection
                .strip_prefix("name:")
                .is_some_and(|name| mutations.contains(name))
    });
}

impl BodyAnalysis {
    pub(crate) fn stable_length_aliases(&self, condition: &HirExpr) -> &HashMap<String, String> {
        self.stable_length_aliases
            .get(&expr_key(condition))
            .unwrap_or(&EMPTY_ALIASES)
    }

    pub(super) fn collect_stable_length_aliases(
        &mut self,
        stmts: &[HirStmt],
        inherited: &HashMap<String, String>,
        call_param_conventions: &CallParamConventions,
    ) {
        let mut aliases = inherited.clone();
        for stmt in stmts {
            let shadow = match stmt {
                HirStmt::Let { name, .. }
                | HirStmt::For { target: name, .. }
                | HirStmt::AsyncFor { target: name, .. } => Some(name.as_str()),
                HirStmt::NestedFunction { func, .. } => Some(func.name.as_str()),
                HirStmt::AsyncWith { target, .. } => target.as_deref(),
                _ => None,
            };
            // A condition can itself mutate a collection. Child bodies have not
            // run yet, so their effects must only invalidate the outgoing state.
            let direct = direct_stmt_summary(stmt, call_param_conventions);
            invalidate_aliases(&mut aliases, &direct.mutated, shadow);
            match stmt {
                HirStmt::If {
                    condition,
                    then_body,
                    elif_clauses,
                    else_body,
                } => {
                    self.stable_length_aliases
                        .insert(expr_key(condition), aliases.clone());
                    self.collect_stable_length_aliases(then_body, &aliases, call_param_conventions);
                    for (condition, body) in elif_clauses {
                        self.stable_length_aliases
                            .insert(expr_key(condition), aliases.clone());
                        self.collect_stable_length_aliases(body, &aliases, call_param_conventions);
                    }
                    if let Some(body) = else_body {
                        self.collect_stable_length_aliases(body, &aliases, call_param_conventions);
                    }
                }
                HirStmt::While {
                    condition,
                    body,
                    else_body,
                } => {
                    // A while condition is evaluated again after the body. Do
                    // not reuse an entry proof if any iteration can change it.
                    let mut loop_aliases = aliases.clone();
                    if let Some(summary) = self.statements.get(&stmt_key(stmt)) {
                        invalidate_aliases(&mut loop_aliases, &summary.mutated, None);
                    }
                    self.stable_length_aliases
                        .insert(expr_key(condition), loop_aliases.clone());
                    self.collect_stable_length_aliases(body, &loop_aliases, call_param_conventions);
                    if let Some(body) = else_body {
                        self.collect_stable_length_aliases(
                            body,
                            &loop_aliases,
                            call_param_conventions,
                        );
                    }
                }
                _ => {}
            }
            // The statement summary includes every child path. At a join, an
            // alias is retained only if no path can rebind or mutate its inputs.
            if let Some(summary) = self.statements.get(&stmt_key(stmt)) {
                invalidate_aliases(&mut aliases, &summary.mutated, shadow);
            }
            // Context-manager targets, except-handler names, and match captures
            // may introduce bindings that the mutation summary does not record.
            // No proof crosses those scope boundaries.
            if matches!(
                stmt,
                HirStmt::With { .. } | HirStmt::TryExcept { .. } | HirStmt::Match { .. }
            ) {
                aliases.clear();
            }
            if let HirStmt::Let { name, value, .. } | HirStmt::Assign { name, value } = stmt {
                if let HirExpr::MethodCall {
                    object,
                    method,
                    args,
                    ..
                } = value
                    && method == "len"
                    && args.is_empty()
                    && let HirExpr::Name {
                        name: collection, ..
                    } = object.as_ref()
                    && name != collection
                {
                    aliases.insert(name.clone(), format!("name:{collection}"));
                }
            }
        }
    }
}

static EMPTY_ALIASES: std::sync::LazyLock<HashMap<String, String>> =
    std::sync::LazyLock::new(HashMap::new);
