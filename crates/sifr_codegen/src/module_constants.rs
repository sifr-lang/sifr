use crate::{
    RustEmitter, RustItem, RustStmt, Visibility, try_lower_simple_module_constant_item_result,
};
use sifr_ir::{HirExpr, HirModule};
use sifr_type_system::Type;

impl RustEmitter {
    pub(crate) fn emit_module_constants(&mut self, module: &HirModule, module_public: bool) {
        for (name, ty, value) in &module.constants {
            if let Err(err) =
                self.try_emit_lowered_module_constant_result(name, ty, value, module_public)
            {
                self.lowering_stats.item_lowering_errors += 1;
                self.record_codegen_error(err.in_context(format!(
                    "structured module constant emission missing for production path ({name})"
                )));
                break;
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
        let (mut item, rust_name_call) =
            match try_lower_simple_module_constant_item_result(name, ty, value)? {
                Some(lowered) => lowered,
                None => self.try_emit_expression_module_constant(name, ty, value)?,
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

    fn try_emit_expression_module_constant(
        &mut self,
        name: &str,
        ty: &Type,
        value: &HirExpr,
    ) -> Result<(RustItem, String), crate::CodegenError> {
        let Some(lowered_value) = self.lower_rendered_expr_for_ir(value)? else {
            return Err(crate::CodegenError::new(format!(
                "unsupported module constant lowering shape: name={name}, ty={ty:?}, value={value:?}"
            )));
        };
        let rust_name = format!("__const_{name}");
        Ok((
            RustItem::Fn {
                name: rust_name.clone(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![],
                ret: Some(crate::sifr_type_to_rust_type(ty)),
                body: vec![RustStmt::Return(Some(lowered_value))],
                is_async: false,
            },
            format!("{rust_name}()"),
        ))
    }
}
