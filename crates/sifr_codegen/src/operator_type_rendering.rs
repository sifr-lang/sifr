use crate::RustEmitter;
use sifr_ir::{HirClass, HirParam};
use sifr_type_system::Type;

impl RustEmitter {
    pub(crate) fn operator_output_type(&self, class: &HirClass, ty: &Type) -> String {
        if class.is_self_type(ty) {
            Self::class_impl_target(class)
        } else {
            self.rust_type_with_generics(ty)
        }
    }

    pub(crate) fn operator_rhs_type(
        &self,
        class: &HirClass,
        param: Option<&HirParam>,
        class_target: &str,
    ) -> String {
        let Some(param) = param else {
            return format!("&{class_target}");
        };
        let rendered = if class.is_self_type(&param.ty) {
            class_target.to_string()
        } else {
            self.rust_type_with_generics(&param.ty)
        };
        if param.convention.is_borrowed() {
            format!("&{rendered}")
        } else {
            rendered
        }
    }

    pub(crate) fn operator_rust_bound(type_param: &str, requirement: &str) -> String {
        match requirement {
            "Add" | "Sub" | "Mul" | "Div" | "Rem" | "Neg" => {
                format!("std::ops::{requirement}<Output = {type_param}>")
            }
            _ => requirement.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_ir::HirClassKind;

    fn class(name: &str, type_params: Vec<String>) -> HirClass {
        HirClass {
            name: name.to_string(),
            identity: None,
            fields: Vec::new(),
            methods: Vec::new(),
            is_hashable: false,
            is_error_type: false,
            kind: HirClassKind::Regular,
            operator_impls: Vec::new(),
            newtype_inner: None,
            implements_protocols: Vec::new(),
            parent_class: None,
            parent_type: None,
            type_params,
            enum_variants: Vec::new(),
            rust_interop: Vec::new(),
        }
    }

    fn nominal(identity: Option<&str>, name: &str, type_args: Vec<Type>) -> Type {
        Type::Class {
            identity: identity.map(str::to_string),
            type_args,
            name: name.to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        }
    }

    #[test]
    fn self_type_requires_exact_identity_and_specialization() {
        let local = class("JsonValue", vec!["T".to_string()]);
        assert!(local.is_self_type(&nominal(
            None,
            "JsonValue",
            vec![Type::TypeVar("T".to_string())],
        )));
        assert!(!local.is_self_type(&nominal(
            Some("sifr.json.JsonValue"),
            "JsonValue",
            vec![Type::TypeVar("T".to_string())],
        )));
        assert!(!local.is_self_type(&nominal(None, "JsonValue", vec![Type::Int])));

        let compiler_prefixed = class("__SifrBox", vec!["T".to_string()]);
        assert!(compiler_prefixed.is_self_type(&nominal(
            None,
            "__SifrBox",
            vec![Type::TypeVar("T".to_string())],
        )));
    }
}
