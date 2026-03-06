use crate::RustEmitter;
use sifr_hir::{HirClass, HirFunction, HirStmt};
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
        class
            .fields
            .iter()
            .any(|(_, ty)| type_has_typevar_dict_key(ty))
    }

    /// Check if a generic function needs Hash + Eq bounds (uses `TypeVar` as dict key
    /// or returns a generic class that needs Hash + Eq).
    pub(super) fn func_needs_hash_eq(func: &HirFunction) -> bool {
        fn type_has_typevar_dict_key(ty: &Type) -> bool {
            match ty {
                Type::Dict(key, _) => matches!(key.as_ref(), Type::TypeVar(_)),
                Type::List(inner) => type_has_typevar_dict_key(inner),
                Type::Union(members) => members.iter().any(type_has_typevar_dict_key),
                Type::Class { fields, .. } => {
                    fields.iter().any(|(_, t)| type_has_typevar_dict_key(t))
                }
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
        let requirements =
            crate::hir_analysis::queries::collect_typevar_operator_requirements(body, tp);
        let mut extra = String::new();
        if requirements.needs_add {
            let _ = write!(extra, " + std::ops::Add<Output = {tp}>");
        }
        if requirements.needs_sub {
            let _ = write!(extra, " + std::ops::Sub<Output = {tp}>");
        }
        extra
    }
}
