use crate::{try_lower_simple_module_constant_item_result, RustEmitter, RustItem, Visibility};
use sifr_ir::{HirExpr, HirModule};
use sifr_type_system::Type;

impl RustEmitter {
    pub(crate) fn emit_module_constants(&mut self, module: &HirModule, module_public: bool) {
        for (name, ty, value) in &module.constants {
            if let Err(err) =
                self.try_emit_lowered_module_constant_result(name, ty, value, module_public)
            {
                self.lowering_stats.item_lowering_errors += 1;
                panic!(
                    "structured module constant emission missing for production path ({name}): {err}"
                );
            }
        }
    }

    pub(crate) fn try_emit_lowered_module_constant_result(
        &mut self,
        name: &str,
        ty: &Type,
        value: &HirExpr,
        module_public: bool,
    ) -> Result<(), crate::CodegenError> {
        let Some((mut item, rust_name_call)) =
            try_lower_simple_module_constant_item_result(name, ty, value)?
        else {
            return Err(crate::CodegenError::new(format!(
                "unsupported module constant lowering shape: name={name}, ty={ty:?}, value={value:?}"
            )));
        };
        if module_public {
            match &mut item {
                RustItem::Const { visibility, .. }
                | RustItem::Fn { visibility, .. }
                | RustItem::Static { visibility, .. } => {
                    *visibility = Visibility::Pub;
                }
                _ => {}
            }
        }
        self.body_items.push(item);
        self.module_constants
            .insert(name.to_string(), (ty.clone(), rust_name_call));
        Ok(())
    }
}
