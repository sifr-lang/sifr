use crate::{RustEmitter, RustExpr, RustParam, RustType};
use sifr_type_system::Type;

impl RustEmitter {
    pub(crate) fn try_lower_optional_class_field_access(
        &self,
        class_ty: &Type,
        field: &str,
        lowered_object: &RustExpr,
    ) -> Option<RustExpr> {
        let Type::Class { name, fields, .. } = crate::resolve_alias_type_for_plain_call(class_ty)
        else {
            return None;
        };
        let (_, field_ty) = fields.iter().find(|(candidate, _)| candidate == field)?;

        // Project through the present class value so None propagates without
        // attempting direct field access on the Option wrapper.
        let binding = "sifr_generated_optional_field_value";
        let storage = self.lower_field_storage_access_for_class(
            Some(name),
            field,
            RustExpr::Ident(binding.to_string()),
        );
        let field_value = if self
            .recursive_fields
            .contains(&(name.clone(), field.to_string()))
            && crate::helpers::is_option_type(field_ty)
        {
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(storage),
                    method: "as_deref".to_string(),
                    args: vec![],
                }),
                method: "cloned".to_string(),
                args: vec![],
            }
        } else {
            RustExpr::MethodCall {
                receiver: Box::new(storage),
                method: "clone".to_string(),
                args: vec![],
            }
        };
        Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(lowered_object.clone()),
                method: "as_ref".to_string(),
                args: vec![],
            }),
            method: if crate::helpers::is_option_type(field_ty) {
                "and_then"
            } else {
                "map"
            }
            .to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: binding.to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(field_value),
                is_move: false,
            }],
        })
    }
}
