use crate::{HirExpr, RustEmitter, RustExpr, RustStmt, RustType, Type};

impl RustEmitter {
    pub(crate) fn begin_loop_target_string_cache(
        &mut self,
        target: &str,
        ty: &Type,
        enabled: bool,
    ) -> (std::collections::HashMap<String, String>, Option<RustStmt>) {
        let previous = self.string_char_cache_vars.clone();
        let init = if enabled && !target.contains(',') {
            self.string_char_cache_vars.remove(target);
            self.string_char_cache_init_stmt_for_loop_target(target, ty)
        } else {
            None
        };
        (previous, init)
    }

    pub(crate) fn string_char_cache_for_expr(&self, expr: &HirExpr) -> Option<String> {
        let HirExpr::Name { name, .. } = expr else {
            return None;
        };
        self.string_char_cache_vars.get(name).cloned()
    }

    pub(crate) fn string_char_cache_init_stmt_for_local(
        &mut self,
        name: &str,
        ty: &Type,
    ) -> Option<RustStmt> {
        let should_cache = self.string_char_cache_required_names.contains(name);
        if !should_cache
            || self.string_char_cache_vars.contains_key(name)
            || !matches!(ty.resolve_alias(), Type::Str | Type::LiteralStr(_))
        {
            return None;
        }
        Some(self.build_string_char_cache_init_stmt(name))
    }

    pub(crate) fn force_string_char_cache_init_stmt_for_local(
        &mut self,
        name: &str,
        ty: &Type,
    ) -> Option<RustStmt> {
        if self.string_char_cache_vars.contains_key(name)
            || !matches!(ty.resolve_alias(), Type::Str | Type::LiteralStr(_))
        {
            return None;
        }
        Some(self.build_string_char_cache_init_stmt(name))
    }

    pub(crate) fn string_char_cache_init_stmt_for_loop_target(
        &mut self,
        name: &str,
        ty: &Type,
    ) -> Option<RustStmt> {
        if !self.string_char_cache_required_names.contains(name)
            || self.string_char_cache_vars.contains_key(name)
            || !matches!(ty.resolve_alias(), Type::Str | Type::LiteralStr(_))
        {
            return None;
        }
        Some(self.build_string_char_cache_init_stmt(name))
    }

    pub(super) fn build_string_char_cache_init_stmt(&mut self, name: &str) -> RustStmt {
        let cache_name = format!("__sifr_chars_{name}");
        self.string_char_cache_vars
            .insert(name.to_string(), cache_name.clone());
        RustStmt::Let {
            mutable: true,
            name: cache_name,
            ty: Some(RustType::Vec(Box::new(RustType::Named("char".to_string())))),
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(name.to_string())),
                    method: "chars".to_string(),
                    args: vec![],
                }),
                method: "collect::<Vec<char>>".to_string(),
                args: vec![],
            },
        }
    }

    pub(crate) fn string_char_cache_rebuild_stmt_for_local(&self, name: &str) -> Option<RustStmt> {
        let cache_name = self.string_char_cache_vars.get(name)?;
        Some(RustStmt::Assign {
            target: RustExpr::Ident(cache_name.clone()),
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(name.to_string())),
                    method: "chars".to_string(),
                    args: vec![],
                }),
                method: "collect::<Vec<char>>".to_string(),
                args: vec![],
            },
        })
    }
}
