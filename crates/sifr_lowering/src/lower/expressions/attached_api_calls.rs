use super::{
    DiagnosticCode, Expr, ExprAttribute, ExprCall, FunctionType, HashMap, HirExpr, LowerCtx,
    Ranged, Type, call_argument_ranges_by_param, collect_type_vars, consume_owned_value,
    decode_typevar_constraint, infer_type_var_bindings, lower_signature_call_args,
    protocol_diagnostics, substitute_type_vars, type_satisfies_bound, type_satisfies_constraint,
};
use crate::lower::typing_and_functions::resolve_annotation_expr;
use sifr_ir::AttachedApiReceiver;

pub(super) fn try_lower_type_call(
    attr: &ExprAttribute,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<Option<HirExpr>> {
    let (surface_name, owner_type) = match attr.value.as_ref() {
        Expr::Name(name) => {
            let class_name = name.id.to_string();
            let owner_type = ctx.class_types.get(&class_name).cloned()?;
            (class_name, owner_type)
        }
        Expr::Subscript(subscript) => {
            let Expr::Name(name) = subscript.value.as_ref() else {
                return None;
            };
            if !ctx.class_types.contains_key(name.id.as_str())
                && ctx
                    .scope
                    .lookup_generic_type_alias(name.id.as_str())
                    .is_none()
            {
                return None;
            }
            let owner_type = resolve_annotation_expr(&attr.value, ctx);
            if matches!(owner_type, Type::Any | Type::Unknown) {
                return Some(None);
            }
            (name.id.to_string(), owner_type)
        }
        _ => return None,
    };
    let binding = super::super::attached_api_surfaces::binding_for_owner(
        ctx,
        &surface_name,
        &owner_type,
        attr.attr.as_str(),
    )?
    .clone();
    if binding.declaration.receiver != AttachedApiReceiver::Type {
        return None;
    }
    Some(lower_call(
        binding,
        &owner_type,
        None,
        call,
        attr.value.range(),
        ctx,
    ))
}

pub(super) fn try_lower_instance_call(
    object: HirExpr,
    attr: &ExprAttribute,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<Option<HirExpr>> {
    let Type::Class { name, .. } = object.ty().resolve_alias() else {
        return None;
    };
    let binding = super::super::attached_api_surfaces::binding_for_owner(
        ctx,
        name,
        object.ty(),
        attr.attr.as_str(),
    )?
    .clone();
    if binding.declaration.receiver == AttachedApiReceiver::Type {
        return None;
    }
    let owner_type = object.ty().clone();
    Some(lower_call(
        binding,
        &owner_type,
        Some(object),
        call,
        attr.value.range(),
        ctx,
    ))
}

fn lower_call(
    binding: super::super::attached_api_surfaces::AttachedMethodBinding,
    owner_type: &Type,
    receiver: Option<HirExpr>,
    call: &ExprCall,
    receiver_range: ruff_text_size::TextRange,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    let full_type =
        super::super::attached_api_surfaces::specialize_owner(&binding.declaration, owner_type);
    let exposed_type = exposed_function_type(binding.declaration.receiver, &full_type);
    let exposed_defaults =
        exposed_defaults(binding.declaration.receiver, &binding.declaration.defaults);
    let mut args = lower_signature_call_args(
        call,
        &format!(
            "{}.{}",
            owner_type.display_name(),
            binding.declaration.public_name
        ),
        &exposed_type,
        Some(&exposed_defaults),
        ctx,
    )?;
    let mut ranges = call_argument_ranges_by_param(call, &exposed_type);
    if let Some(receiver) = receiver {
        args.insert(0, receiver);
        ranges.insert(0, Some(receiver_range));
    }

    let mut bindings = HashMap::from([(
        binding.declaration.owner_type_param.clone(),
        owner_type.clone(),
    )]);
    if let Some(expected_type) = ctx.contextual_expr_type(call.range()) {
        let mut expected_type_vars = Vec::new();
        collect_type_vars(expected_type, &mut expected_type_vars);
        if expected_type_vars.is_empty() {
            infer_type_var_bindings(&full_type.return_type, expected_type, &mut bindings);
        }
    }
    for (argument, (_, parameter_type, _)) in args.iter().zip(&full_type.params) {
        infer_type_var_bindings(parameter_type, argument.ty(), &mut bindings);
    }

    validate_bindings(&binding, &bindings, call, ctx);
    validate_arguments(&binding, &full_type, &bindings, &args, &ranges, call, ctx);
    let mutable_arg_places = super::super::method_receiver_places::validate_regular_call_arguments(
        &args,
        &substitute_function_type(&full_type, &bindings),
        &ranges,
        call.range(),
        &binding.emitted_function,
        ctx,
    );
    for (index, argument) in args.iter().enumerate() {
        if full_type
            .params
            .get(index)
            .is_some_and(|(_, _, convention)| convention.is_owned())
        {
            let range = ranges
                .get(index)
                .copied()
                .flatten()
                .unwrap_or_else(|| call.range());
            consume_owned_value(argument, range, ctx);
        }
    }

    let concrete_type_args = binding
        .declaration
        .type_params
        .iter()
        .filter_map(|type_param| bindings.get(type_param))
        .cloned()
        .collect::<Vec<_>>();
    if concrete_type_args.len() == binding.declaration.type_params.len()
        && !concrete_type_args.is_empty()
    {
        return Some(HirExpr::GenericCall {
            mutable_arg_places,
            func: binding.emitted_function,
            type_args: concrete_type_args,
            args,
            ty: substitute_type_vars(&full_type.return_type, &bindings),
        });
    }
    Some(HirExpr::Call {
        mutable_arg_places,
        func: binding.emitted_function,
        args,
        ty: substitute_type_vars(&full_type.return_type, &bindings),
    })
}

fn exposed_defaults(
    receiver: AttachedApiReceiver,
    defaults: &[(usize, HirExpr)],
) -> Vec<(usize, HirExpr)> {
    if receiver == AttachedApiReceiver::Type {
        return defaults.to_vec();
    }
    defaults
        .iter()
        .filter_map(|(index, value)| index.checked_sub(1).map(|index| (index, value.clone())))
        .collect()
}

fn validate_bindings(
    binding: &super::super::attached_api_surfaces::AttachedMethodBinding,
    bindings: &HashMap<String, Type>,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) {
    for type_param in &binding.declaration.type_params {
        let Some(concrete_type) = bindings.get(type_param) else {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "attached API '{}' cannot infer type parameter '{}'",
                    binding.declaration.public_name, type_param
                ),
                call.range(),
            );
            continue;
        };
        let Some(bounds) = binding.declaration.type_param_bounds.get(type_param) else {
            continue;
        };
        let mut constraints = Vec::new();
        for bound in bounds {
            if provisional_owner_bound_is_deferred(
                binding.provisional,
                type_param,
                &binding.declaration.owner_type_param,
                bound,
            ) {
                continue;
            }
            if let Some(constraint) = decode_typevar_constraint(bound) {
                constraints.push(constraint.to_string());
            } else if !type_satisfies_bound(concrete_type, bound, ctx) {
                protocol_diagnostics::bound_not_satisfied(
                    ctx,
                    &concrete_type.display_name(),
                    bound,
                    type_param,
                    call.range(),
                );
            }
        }
        if !constraints.is_empty()
            && !constraints
                .iter()
                .any(|constraint| type_satisfies_constraint(concrete_type, constraint, ctx))
        {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_TYPEVAR_CONSTRAINT_NOT_SATISFIED,
                format!(
                    "type '{}' does not satisfy constraints ({}) required by type parameter '{}'",
                    concrete_type.display_name(),
                    constraints.join(", "),
                    type_param
                ),
                call.range(),
            );
        }
    }
}

fn provisional_owner_bound_is_deferred(
    provisional: bool,
    type_param: &str,
    owner_type_param: &str,
    bound: &str,
) -> bool {
    provisional
        && type_param == owner_type_param
        && matches!(bound, "StaticProgram" | "MethodSlots")
}

fn validate_arguments(
    binding: &super::super::attached_api_surfaces::AttachedMethodBinding,
    function_type: &FunctionType,
    bindings: &HashMap<String, Type>,
    args: &[HirExpr],
    ranges: &[Option<ruff_text_size::TextRange>],
    call: &ExprCall,
    ctx: &mut LowerCtx,
) {
    for (index, (argument, (name, parameter_type, _))) in
        args.iter().zip(&function_type.params).enumerate()
    {
        let concrete = substitute_type_vars(parameter_type, bindings);
        let mut unresolved = Vec::new();
        collect_type_vars(&concrete, &mut unresolved);
        if !unresolved.is_empty() || argument.ty().is_assignable_to(&concrete) {
            continue;
        }
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            format!(
                "argument {} ('{}') of attached API '{}': expected '{}', got '{}'",
                index + 1,
                name,
                binding.declaration.public_name,
                concrete.display_name(),
                argument.ty().display_name()
            ),
            ranges
                .get(index)
                .copied()
                .flatten()
                .unwrap_or_else(|| call.range()),
        );
    }
}

fn exposed_function_type(
    receiver: AttachedApiReceiver,
    function_type: &FunctionType,
) -> FunctionType {
    FunctionType {
        receiver: None,
        params: if receiver == AttachedApiReceiver::Type {
            function_type.params.clone()
        } else {
            function_type.params.iter().skip(1).cloned().collect()
        },
        return_type: function_type.return_type.clone(),
    }
}

fn substitute_function_type(
    function_type: &FunctionType,
    bindings: &HashMap<String, Type>,
) -> FunctionType {
    FunctionType {
        receiver: function_type.receiver,
        params: function_type
            .params
            .iter()
            .map(|(name, ty, convention)| {
                (
                    name.clone(),
                    substitute_type_vars(ty, bindings),
                    *convention,
                )
            })
            .collect(),
        return_type: Box::new(substitute_type_vars(&function_type.return_type, bindings)),
    }
}

#[cfg(test)]
mod tests {
    use super::provisional_owner_bound_is_deferred;

    #[test]
    fn provisional_static_owner_bounds_have_exact_parity() {
        assert!(provisional_owner_bound_is_deferred(
            true,
            "T",
            "T",
            "StaticProgram"
        ));
        assert!(provisional_owner_bound_is_deferred(
            true,
            "T",
            "T",
            "MethodSlots"
        ));
        assert!(!provisional_owner_bound_is_deferred(
            true,
            "T",
            "T",
            "Structural"
        ));
        assert!(!provisional_owner_bound_is_deferred(
            true,
            "Context",
            "T",
            "MethodSlots"
        ));
        assert!(!provisional_owner_bound_is_deferred(
            false,
            "T",
            "T",
            "MethodSlots"
        ));
    }
}
