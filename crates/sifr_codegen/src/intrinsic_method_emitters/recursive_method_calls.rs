use super::{HirExpr, RustEmitter, Type};
use sifr_type_system::ReceiverConvention;

impl RustEmitter {
    pub(crate) fn lower_recursive_method_receiver_and_args(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
        receiver_convention: Option<ReceiverConvention>,
    ) -> Option<(crate::RustExpr, Type, Vec<crate::RustExpr>)> {
        let suppress_receiver =
            self.method_call_needs_field_clone_suppression(object, receiver_convention);
        let suppression_before = self.pending_self_field_clone_suppression;
        if suppress_receiver {
            self.pending_self_field_clone_suppression += 1;
        }
        let object_expr = self.try_lower_registry_expr_strict(object)?;
        if suppress_receiver && self.pending_self_field_clone_suppression > suppression_before {
            self.pending_self_field_clone_suppression -= 1;
        }

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
            let suppress_arg = self.method_mut_arg_needs_field_clone_suppression(arg, convention);
            let suppression_before = self.pending_self_field_clone_suppression;
            if suppress_arg {
                self.pending_self_field_clone_suppression += 1;
            }
            let lowered = self.try_lower_registry_expr_strict(arg);
            if suppress_arg && self.pending_self_field_clone_suppression > suppression_before {
                self.pending_self_field_clone_suppression -= 1;
            }
            arg_exprs.push(lowered?);
        }

        Some((object_expr, effective_object_ty, arg_exprs))
    }
}
