use crate::RustEmitter;
use sifr_ir::{HirClass, HirParam};
use sifr_type_system::Type;

impl RustEmitter {
    pub(crate) fn operator_output_type(&self, class: &HirClass, ty: &Type) -> String {
        if class.is_self_type(ty) {
            Self::class_impl_target(class)
        } else {
            self.render_rust_type_with_generics(ty)
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
            self.render_rust_type_with_generics(&param.ty)
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
#[path = "../tests/unit/operator_type_rendering/tests.rs"]
mod tests;
