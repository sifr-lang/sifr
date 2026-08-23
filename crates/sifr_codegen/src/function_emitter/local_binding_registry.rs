use super::{
    HashMap, HashSet, HirExpr, HirStmt, RustEmitter, TraversalConfig, Type, make_union, traversal,
};

impl RustEmitter {
    pub(crate) fn register_local_body_binding_types(&mut self, body: &[HirStmt]) {
        let mut bindings = HashMap::new();
        let mut ambiguous_let_names = HashSet::new();
        let mut widened_bindings = HashSet::new();
        let mut on_stmt = |stmt: &HirStmt| match stmt {
            HirStmt::Let { name, ty, .. } => {
                if !ambiguous_let_names.contains(name) {
                    if bindings.get(name).is_some_and(|existing| existing != ty) {
                        bindings.remove(name);
                        widened_bindings.remove(name);
                        ambiguous_let_names.insert(name.clone());
                    } else {
                        bindings.entry(name.clone()).or_insert_with(|| ty.clone());
                    }
                }
            }
            HirStmt::Assign { name, value }
                if matches!(value, HirExpr::NoneLiteral) || matches!(value.ty(), Type::None) =>
            {
                if let Some(existing) = bindings.get(name).cloned() {
                    if !crate::helpers::is_option_type(&existing) {
                        bindings.insert(name.clone(), make_union(vec![existing, Type::None]));
                        widened_bindings.insert(name.clone());
                    }
                }
            }
            _ => {}
        };
        let mut on_expr = |_expr: &HirExpr| {};
        traversal::walk_stmts(
            body,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr,
        );
        for (name, ty) in bindings {
            self.local_binding_types.entry(name).or_insert(ty);
        }
        self.none_widened_local_bindings.extend(widened_bindings);
        self.register_sifr_int_forced_local_bindings(body);
    }
}
