use super::{HirExpr, RustEmitter, Type};
use sifr_ir::{MutableArgumentTarget, MutableReceiverTarget};
use sifr_type_system::ReceiverConvention;

impl RustEmitter {
    pub(crate) fn try_lower_recursive_indexed_list_append(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
        receiver_target: Option<&MutableReceiverTarget>,
    ) -> Option<crate::RustExpr> {
        if method != "append" || args.len() != 1 {
            return None;
        }
        let MutableReceiverTarget::SpecializedIndexedStorage(base_place) = receiver_target? else {
            return None;
        };
        let HirExpr::Index {
            object: index_object,
            index,
            ..
        } = object
        else {
            return None;
        };
        if !matches!(
            crate::resolve_alias_type_for_plain_call(object.ty()),
            Type::List(_)
        ) || !matches!(
            crate::resolve_alias_type_for_plain_call(index_object.ty()),
            Type::Dict(_, _)
        ) {
            return None;
        }

        let lowered_object = self.emit_checked_place(index_object, base_place)?;
        let lowered_index = self.try_lower_registry_expr_strict(index)?;
        let lowered_arg = self.try_lower_registry_expr_strict(&args[0])?;
        let key_arg = Self::build_dict_lookup_key_arg_for_ir(
            Self::clone_non_copy_name_expr_for_ir(index, lowered_index),
        );
        let pushed_arg = self.materialize_reusable_value_for_ir(&args[0], lowered_arg);
        Some(crate::RustExpr::Block {
            stmts: vec![crate::RustStmt::IfLet {
                pattern: "Some(__elem)".to_string(),
                expr: crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_object),
                    method: "get_mut".to_string(),
                    args: vec![key_arg],
                },
                then_body: vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Ident("__elem".to_string())),
                    method: "push".to_string(),
                    args: vec![pushed_arg],
                })],
                else_body: None,
            }],
            expr: None,
        })
    }

    pub(crate) fn lower_recursive_method_receiver_and_args(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
        receiver_convention: Option<ReceiverConvention>,
        receiver_target: Option<&MutableReceiverTarget>,
        mutable_arg_places: &[Option<MutableArgumentTarget>],
    ) -> Option<(crate::RustExpr, Type, Vec<crate::RustExpr>)> {
        let object_expr = self.lower_method_receiver_place_for_registry(
            object,
            receiver_convention,
            receiver_target,
        )?;

        let effective_object_ty = self.effective_method_object_ty(object);
        let method_params = self.resolve_registry_method_params(&effective_object_ty, method);
        let mut arg_exprs = Vec::with_capacity(args.len());
        for (index, arg) in args.iter().enumerate() {
            let convention = method_params
                .as_ref()
                .and_then(|params| params.get(index))
                .map_or(
                    sifr_type_system::ParamConvention::default(),
                    |(_, convention)| *convention,
                );
            arg_exprs.push(self.lower_method_argument_place_for_registry(
                arg,
                convention,
                mutable_arg_places.get(index).and_then(Option::as_ref),
            )?);
        }

        Some((object_expr, effective_object_ty, arg_exprs))
    }
}
