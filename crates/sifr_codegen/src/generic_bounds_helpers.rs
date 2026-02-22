use crate::RustEmitter;
use sifr_hir::{HirClass, HirExpr, HirFunction, HirStmt};
use sifr_type_system::Type;
use std::fmt::Write as _;

impl RustEmitter {
    /// Check if a generic class needs Hash + Eq bounds on its type parameters.
    /// This is true when a type parameter is used as a `HashMap` key (dict field with `TypeVar` key).
    pub(super) fn class_needs_hash_eq(class: &HirClass) -> bool {
        fn type_has_typevar_dict_key(ty: &Type) -> bool {
            match ty {
                Type::Dict(key, _) => matches!(key.as_ref(), Type::TypeVar(_)),
                Type::List(inner) => type_has_typevar_dict_key(inner),
                Type::Union(members) => members.iter().any(type_has_typevar_dict_key),
                _ => false,
            }
        }
        class.fields.iter().any(|(_, ty)| type_has_typevar_dict_key(ty))
    }

    /// Check if a generic function needs Hash + Eq bounds (uses `TypeVar` as dict key
    /// or returns a generic class that needs Hash + Eq).
    pub(super) fn func_needs_hash_eq(func: &HirFunction) -> bool {
        fn type_has_typevar_dict_key(ty: &Type) -> bool {
            match ty {
                Type::Dict(key, _) => matches!(key.as_ref(), Type::TypeVar(_)),
                Type::List(inner) => type_has_typevar_dict_key(inner),
                Type::Union(members) => members.iter().any(type_has_typevar_dict_key),
                Type::Class { fields, .. } => fields.iter().any(|(_, t)| type_has_typevar_dict_key(t)),
                _ => false,
            }
        }
        // Check params
        if func.params.iter().any(|p| type_has_typevar_dict_key(&p.ty)) {
            return true;
        }
        // Check return type
        if type_has_typevar_dict_key(&func.return_type) {
            return true;
        }
        false
    }

    pub(super) fn generic_bounds_for_class(class: &HirClass) -> String {
        if Self::class_needs_hash_eq(class) {
            "Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq".to_string()
        } else {
            "Clone + std::fmt::Display + PartialOrd".to_string()
        }
    }

    /// Convert a Type to its Rust representation, appending generic type params
    /// for classes that are known to be generic (e.g., Counter -> Counter<T>).
    pub(super) fn rust_type_with_generics(&self, ty: &Type) -> String {
        if let Type::Class { name, .. } = ty {
            if let Some(params) = self.generic_class_params.get(name) {
                return format!("{}<{}>", name, params.join(", "));
            }
        }
        ty.rust_type()
    }

    pub(super) fn extra_bounds_for_type_param(tp: &str, body: &[HirStmt]) -> String {
        let mut needs_add = false;
        let mut needs_sub = false;
        Self::scan_body_for_typevar_ops(tp, body, &mut needs_add, &mut needs_sub);
        let mut extra = String::new();
        if needs_add {
            let _ = write!(extra, " + std::ops::Add<Output = {tp}>");
        }
        if needs_sub {
            let _ = write!(extra, " + std::ops::Sub<Output = {tp}>");
        }
        extra
    }

    fn scan_body_for_typevar_ops(tp: &str, stmts: &[HirStmt], needs_add: &mut bool, needs_sub: &mut bool) {
        for stmt in stmts {
            Self::scan_stmt_for_typevar_ops(tp, stmt, needs_add, needs_sub);
        }
    }

    fn scan_stmt_for_typevar_ops(tp: &str, stmt: &HirStmt, needs_add: &mut bool, needs_sub: &mut bool) {
        match stmt {
            HirStmt::Let { value, .. } => {
                Self::scan_expr_for_typevar_ops(tp, value, needs_add, needs_sub);
            }
            HirStmt::Assign { value, .. } => {
                Self::scan_expr_for_typevar_ops(tp, value, needs_add, needs_sub);
            }
            HirStmt::Expr { expr } => {
                Self::scan_expr_for_typevar_ops(tp, expr, needs_add, needs_sub);
            }
            HirStmt::Return { value: Some(expr) } => {
                Self::scan_expr_for_typevar_ops(tp, expr, needs_add, needs_sub);
            }
            HirStmt::If {
                condition,
                then_body,
                elif_clauses,
                else_body,
                ..
            } => {
                Self::scan_expr_for_typevar_ops(tp, condition, needs_add, needs_sub);
                Self::scan_body_for_typevar_ops(tp, then_body, needs_add, needs_sub);
                for (cond, body) in elif_clauses {
                    Self::scan_expr_for_typevar_ops(tp, cond, needs_add, needs_sub);
                    Self::scan_body_for_typevar_ops(tp, body, needs_add, needs_sub);
                }
                if let Some(eb) = else_body {
                    Self::scan_body_for_typevar_ops(tp, eb, needs_add, needs_sub);
                }
            }
            HirStmt::While {
                condition, body, ..
            } => {
                Self::scan_expr_for_typevar_ops(tp, condition, needs_add, needs_sub);
                Self::scan_body_for_typevar_ops(tp, body, needs_add, needs_sub);
            }
            HirStmt::For { iter, body, .. } => {
                Self::scan_expr_for_typevar_ops(tp, iter, needs_add, needs_sub);
                Self::scan_body_for_typevar_ops(tp, body, needs_add, needs_sub);
            }
            _ => {}
        }
    }

    fn scan_expr_for_typevar_ops(tp: &str, expr: &HirExpr, needs_add: &mut bool, needs_sub: &mut bool) {
        if let HirExpr::BinOp {
            left,
            op,
            right,
            ty,
        } = expr
        {
            let left_is_tp = matches!(left.ty(), Type::TypeVar(ref n) if n == tp);
            let right_is_tp = matches!(right.ty(), Type::TypeVar(ref n) if n == tp);
            let result_is_tp = matches!(ty, Type::TypeVar(ref n) if n == tp);
            if left_is_tp || right_is_tp || result_is_tp {
                match op.as_str() {
                    "+" => *needs_add = true,
                    "-" => *needs_sub = true,
                    _ => {}
                }
            }
            Self::scan_expr_for_typevar_ops(tp, left, needs_add, needs_sub);
            Self::scan_expr_for_typevar_ops(tp, right, needs_add, needs_sub);
        }
    }
}
