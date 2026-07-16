use super::{RustEmitter, Type};

impl RustEmitter {
    pub(crate) fn consuming_class_upcast_for_ir(
        &self,
        target_ty: &Type,
        source_ty: &Type,
        mut lowered: crate::RustExpr,
    ) -> crate::RustExpr {
        let target_ty = Self::option_inner_type_for_ir(target_ty).unwrap_or(target_ty);
        let (
            Type::Class {
                identity: source_identity,
                name: source_name,
                parent_class: Some(parent_chain),
                ..
            },
            Type::Class {
                identity: target_identity,
                name: target_name,
                ..
            },
        ) = (
            crate::resolve_alias_type_for_plain_call(source_ty),
            crate::resolve_alias_type_for_plain_call(target_ty),
        )
        else {
            return lowered;
        };
        let source_identity = source_identity.as_ref().unwrap_or(source_name);
        let target_identity = target_identity.as_ref().unwrap_or(target_name);
        if source_identity == target_identity {
            return lowered;
        }
        let target_tail = target_identity
            .rsplit_once('.')
            .map_or(target_identity.as_str(), |(_, tail)| tail);
        let Some(target_index) = parent_chain.split('|').position(|ancestor| {
            ancestor == target_identity
                || ancestor.rsplit_once('.').map_or(ancestor, |(_, tail)| tail) == target_tail
        }) else {
            return lowered;
        };
        let ancestors = parent_chain.split('|').collect::<Vec<_>>();
        for (index, ancestor) in ancestors.iter().take(target_index + 1).enumerate() {
            let rendered_target = if index == target_index {
                self.rust_type_with_generics(target_ty)
            } else {
                ancestor.replace('.', "::")
            };
            lowered = crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Ident(format!(
                    "std::convert::Into::<{rendered_target}>::into"
                ))),
                args: vec![lowered],
            };
        }
        lowered
    }
}
