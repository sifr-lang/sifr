use crate::{
    helpers::body_contains_field_assign_codegen,
    hir_analysis::traversal::{self, TraversalConfig},
    RustEmitter,
};
use sifr_ir::{HirClass, HirExpr, HirFunction, HirStmt, MethodKind};
use sifr_type_system::Type;
use std::collections::HashSet;

impl RustEmitter {
    pub(crate) fn class_method_requires_mutable_self(
        &self,
        class: &HirClass,
        method: &HirFunction,
    ) -> bool {
        if method.method_kind != MethodKind::Regular || method.name == "new" {
            return false;
        }
        let mut visiting = HashSet::new();
        self.class_method_requires_mutable_self_recursive(class, &method.name, &mut visiting)
    }

    pub(crate) fn class_method_requires_mutable_self_recursive(
        &self,
        class: &HirClass,
        method_name: &str,
        visiting: &mut HashSet<String>,
    ) -> bool {
        let Some(method) = class
            .methods
            .iter()
            .find(|candidate| candidate.name == method_name)
        else {
            return false;
        };

        if body_contains_field_assign_codegen(&method.body)
            || self.body_contains_mut_borrowed_self_field_call(&method.body)
        {
            return true;
        }
        if !visiting.insert(method_name.to_string()) {
            return false;
        }

        for delegated_method in Self::collect_direct_self_method_calls(&method.body) {
            if self.class_method_requires_mutable_self_recursive(class, &delegated_method, visiting)
            {
                visiting.remove(method_name);
                return true;
            }
        }

        visiting.remove(method_name);
        false
    }

    fn body_contains_mut_borrowed_self_field_call(&self, stmts: &[HirStmt]) -> bool {
        let found = std::cell::Cell::new(false);
        let mut on_stmt = |_stmt: &HirStmt| {};
        let mut on_expr = |expr: &HirExpr| {
            if found.get() {
                return;
            }
            let has_mut_self_field_arg = match expr {
                HirExpr::Call { func, args, .. } => self
                    .resolve_plain_call_param_info(func, args.len())
                    .is_some_and(|params| Self::has_mut_self_field_arg(args, &params)),
                HirExpr::MethodCall {
                    object,
                    method,
                    args,
                    ..
                } => self
                    .resolve_registry_method_params(object.ty(), method)
                    .is_some_and(|params| Self::has_mut_self_field_arg(args, &params)),
                _ => false,
            };
            if has_mut_self_field_arg {
                found.set(true);
            }
        };
        traversal::walk_stmts(
            stmts,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr,
        );
        found.get()
    }

    fn has_mut_self_field_arg(
        args: &[HirExpr],
        params: &[(Type, sifr_type_system::ParamConvention)],
    ) -> bool {
        args.iter().zip(params).any(|(arg, (_, convention))| {
            convention.is_mut_borrow() && Self::expr_is_self_field_path(arg)
        })
    }

    fn expr_is_self_field_path(expr: &HirExpr) -> bool {
        match expr {
            HirExpr::FieldAccess { object, .. } => {
                matches!(object.as_ref(), HirExpr::Name { name, .. } if name == "self")
                    || Self::expr_is_self_field_path(object)
            }
            HirExpr::Index { object, .. } | HirExpr::Slice { object, .. } => {
                Self::expr_is_self_field_path(object)
            }
            _ => false,
        }
    }

    pub(crate) fn collect_direct_self_method_calls(stmts: &[HirStmt]) -> HashSet<String> {
        let mut calls = HashSet::new();
        let mut on_stmt = |_stmt: &HirStmt| {};
        let mut on_expr = |expr: &HirExpr| {
            if let HirExpr::MethodCall { object, method, .. } = expr {
                if matches!(object.as_ref(), HirExpr::Name { name, .. } if name == "self") {
                    calls.insert(method.clone());
                }
            }
        };
        traversal::walk_stmts(
            stmts,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr,
        );
        calls
    }

    pub(crate) fn collect_direct_class_method_calls(
        stmts: &[HirStmt],
        class_name: &str,
    ) -> HashSet<String> {
        let mut calls = HashSet::new();
        let mut on_stmt = |_stmt: &HirStmt| {};
        let mut on_expr = |expr: &HirExpr| {
            if let HirExpr::MethodCall { object, method, .. } = expr {
                if matches!(object.ty().resolve_alias(), Type::Class { name, .. } if name == class_name)
                {
                    calls.insert(method.clone());
                }
            }
        };
        traversal::walk_stmts(
            stmts,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr,
        );
        calls
    }
}
