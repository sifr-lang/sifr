use crate::{render_items, try_lower_simple_module_constant_item, RustEmitter};
use sifr_hir::{HirExpr, HirModule};
use sifr_type_system::Type;

impl RustEmitter {
    pub(super) fn emit_module_constants(&mut self, module: &HirModule) {
        for (name, ty, value) in &module.constants {
            if self.try_emit_lowered_module_constant(name, ty, value) {
                continue;
            }
            self.emit_module_constant_fallback(name, ty, value);
        }
        if !module.constants.is_empty() {
            self.output.push('\n');
        }
    }

    fn try_emit_lowered_module_constant(&mut self, name: &str, ty: &Type, value: &HirExpr) -> bool {
        let Some((item, rust_name_call)) = try_lower_simple_module_constant_item(name, ty, value) else {
            return false;
        };
        self.output.push_str(&render_items(&[item]));
        self.module_constants
            .insert(name.to_string(), (ty.clone(), rust_name_call));
        true
    }

    fn emit_module_constant_fallback(&mut self, name: &str, ty: &Type, value: &HirExpr) {
        let rust_name = format!("__const_{name}");
        self.write_indent();
        self.write(&format!(
            "fn {rust_name}() -> {} {{ ",
            fallback_module_constant_return_type(ty)
        ));
        self.emit_expr(value);
        if matches!(ty, Type::Str) {
            self.write(".to_string()");
        }
        self.write(" }\n");
        self.module_constants
            .insert(name.to_string(), (ty.clone(), format!("{rust_name}()")));
    }
}

fn fallback_module_constant_return_type(ty: &Type) -> String {
    if matches!(ty, Type::Str) {
        "String".to_string()
    } else {
        ty.rust_type()
    }
}

#[cfg(test)]
mod tests {
    use super::fallback_module_constant_return_type;
    use sifr_type_system::Type;

    #[test]
    fn fallback_string_return_type_uses_owned_string() {
        assert_eq!(fallback_module_constant_return_type(&Type::Str), "String");
    }

    #[test]
    fn fallback_non_string_return_type_uses_rust_type() {
        let ty = Type::Int;
        assert_eq!(fallback_module_constant_return_type(&ty), ty.rust_type());
    }
}
