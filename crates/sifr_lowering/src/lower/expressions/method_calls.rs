use super::{
    canonicalize_class_surface_type, consume_affine_collection_method_arguments,
    consume_owned_method_arguments, invalidate_collection_flow_facts_for_method,
    is_task_handle_type, lower_expr, lower_task_handle_method_call, method_function_type,
    refine_defaultdict_binding_expr, refine_empty_list_binding_expr, refine_empty_set_binding_expr,
    refine_generic_class_binding_expr, refine_nonempty_method_return_type,
    reject_immutable_method_mut_borrow_arguments, reject_immutable_parameter_method_mutation,
    resolve_bytes_method_type, resolve_class_method_on_type, resolve_decimal_method_type,
    resolve_dict_method_type, resolve_enum_method_type, resolve_fixed_width_method_type,
    resolve_list_method_type, resolve_newtype_method_type, resolve_protocol_method_type,
    resolve_python_arrow_method_type, resolve_python_buffer_method_type,
    resolve_python_dlpack_method_type, resolve_set_method_type, resolve_str_method_type,
    resolve_tuple_method_type, str, try_lower_class_method_call, try_lower_super_method_call, tsc,
    DiagnosticCode, ExprAttribute, ExprCall, HirExpr, LowerCtx, Ranged, TextRange, Type,
    DEFAULTDICT_INT_ALIAS, DEFAULTDICT_LIST_ALIAS, DEFAULTDICT_SET_ALIAS,
};
use super::{method_call_arguments, python_raw_object_methods};
use crate::lower::python_interop::callback_method_arg_ranges;
use crate::lower::{parallel_calls, task_join_set_calls, task_scope_offload_calls};
use sifr_ir::{CompilerIntrinsicId, MethodCallSource};
use sifr_type_system::ReceiverConvention;
pub(in crate::lower) fn lower_method_call(
    attr: &ExprAttribute,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if let Some(result) = try_lower_super_method_call(attr, call, ctx) {
        return result;
    }
    if let Some(result) = try_lower_class_method_call(attr, call, ctx) {
        return result;
    }

    let mut object = lower_expr(&attr.value, ctx)?;
    let method_name = attr.attr.to_string();
    if let Some(result) =
        super::attached_api_calls::try_lower_instance_call(object.clone(), attr, call, ctx)
    {
        return result;
    }
    if let Some(result) = super::blocking_executor_calls::lower_thread_pool_submit_call(
        &object,
        &method_name,
        call,
        ctx,
    ) {
        return result;
    }
    if let Some(result) =
        parallel_calls::lower_parallel_pool_method_call(&object, &method_name, call, ctx)
    {
        return result;
    }
    if let Some(result) =
        task_join_set_calls::lower_join_set_method_call(object.clone(), &method_name, call, ctx)
    {
        return result;
    }
    if let Some(result) = task_scope_offload_calls::lower_task_scope_offload_method_call(
        object.clone(),
        &method_name,
        call,
        ctx,
    ) {
        return result;
    }
    if tsc::is_task_scope_type(object.ty()) && method_name == "spawn" {
        return tsc::lower_task_scope_spawn_call(object, attr, call, ctx);
    }
    if is_task_handle_type(object.ty()) {
        if let Some(expr) = lower_task_handle_method_call(object.clone(), &method_name, call, ctx) {
            return Some(expr);
        }
    }
    let object_ty_for_args = canonicalize_class_surface_type(object.ty().resolve_alias());
    let raw_python_method = python_raw_object_methods::for_call(
        &object_ty_for_args,
        &method_name,
        attr.attr.range(),
        ctx,
    );
    super::workload_annotations::reject_async_direct_method_call(
        ctx,
        &object_ty_for_args,
        &method_name,
        attr.attr.range(),
    );
    let method_type = raw_python_method
        .clone()
        .or_else(|| method_function_type(&object_ty_for_args, &method_name));
    let args = method_call_arguments::lower(
        object.ty(),
        &object_ty_for_args,
        &method_name,
        call,
        raw_python_method.as_ref(),
        ctx,
    )?;

    if matches!(method_name.as_str(), "append" | "insert" | "extend") {
        object = refine_empty_list_binding_expr(object, &method_name, &args, ctx);
    }
    if matches!(
        method_name.as_str(),
        "add" | "remove" | "discard" | "contains"
    ) {
        if let Some(first_arg_ty) = args.first().map(|arg| arg.ty().clone()) {
            object = refine_empty_set_binding_expr(object, first_arg_ty, ctx);
        }
    }
    if let Some(refined_object) =
        refine_defaultdict_binding_expr(object.clone(), &method_name, &args, ctx)
    {
        object = refined_object;
    }
    object = refine_generic_class_binding_expr(object, &method_name, &args, ctx);
    let object_ty = object.ty().clone();
    if reject_immutable_parameter_method_mutation(
        ctx,
        &object,
        &object_ty,
        &method_name,
        attr.value.range(),
    ) {
        return None;
    }
    let method_arg_ranges = callback_method_arg_ranges(
        &object,
        &object_ty_for_args,
        &method_name,
        call,
        &args,
        method_type.as_ref(),
        ctx,
    );
    if reject_immutable_method_mut_borrow_arguments(
        ctx,
        &object_ty_for_args,
        &method_name,
        &args,
        &method_arg_ranges,
    ) {
        return None;
    }
    let resolved_method_type = resolve_method_type(
        &object_ty,
        &method_name,
        &args,
        &method_arg_ranges,
        attr.attr.range(),
        ctx,
    )?;
    if let Some(function_type) = &method_type {
        consume_owned_method_arguments(&args, call, function_type, ctx);
    }
    consume_affine_collection_method_arguments(
        &object_ty,
        &method_name,
        &args,
        &method_arg_ranges,
        ctx,
    );
    let return_ty = refine_nonempty_method_return_type(
        &object_ty,
        &object,
        &method_name,
        &args,
        &resolved_method_type,
        ctx,
    );
    tsc::validate_channel_send_element(
        &object_ty,
        &method_name,
        &args,
        &method_arg_ranges,
        call,
        ctx,
    );
    invalidate_collection_flow_facts_for_method(ctx, &object, &object_ty, &method_name);
    if matches!(object_ty.resolve_alias(), Type::Str) && method_name == "encode" {
        let mut intrinsic_args = vec![object];
        let intrinsic = if args.is_empty() {
            CompilerIntrinsicId::StringEncode
        } else {
            CompilerIntrinsicId::StringEncodeWithEncoding
        };
        intrinsic_args.extend(args.iter().cloned());
        let mut intrinsic_arg_ranges = vec![attr.value.range()];
        intrinsic_arg_ranges.extend(method_arg_ranges.iter().copied());
        return Some(HirExpr::IntrinsicCall {
            intrinsic,
            args: intrinsic_args,
            ty: return_ty,
            call_range: call.range(),
            arg_ranges: intrinsic_arg_ranges,
        });
    }
    if matches!(object_ty.resolve_alias(), Type::Bytes) && method_name == "decode" {
        let mut intrinsic_args = vec![object];
        let intrinsic = if args.is_empty() {
            CompilerIntrinsicId::BytesDecode
        } else {
            CompilerIntrinsicId::BytesDecodeWithEncoding
        };
        intrinsic_args.extend(args.iter().cloned());
        let mut intrinsic_arg_ranges = vec![attr.value.range()];
        intrinsic_arg_ranges.extend(method_arg_ranges.iter().copied());
        return Some(HirExpr::IntrinsicCall {
            intrinsic,
            args: intrinsic_args,
            ty: return_ty,
            call_range: call.range(),
            arg_ranges: intrinsic_arg_ranges,
        });
    }

    if let Type::Class {
        name: class_name, ..
    } = object_ty.resolve_alias()
    {
        let qualified = format!("{class_name}.{method_name}");
        if ctx.python_context_exit_methods.contains(&qualified) {
            ctx.error_with_code_at(
                sifr_diagnostics::DiagnosticCode::PYCTX_INVALID_DECLARATION,
                "invalid Python context declaration: context exit methods are compiler-invoked and cannot be called directly"
                    .to_string(),
                call.range(),
            );
            return None;
        }
        let consumes_rust_receiver = ctx.rust_consuming_methods.contains(&qualified);
        if consumes_rust_receiver {
            match &object {
                HirExpr::Name { name, .. } if ctx.borrowed_params.contains(name) => {
                    ctx.error_with_code_at(
                        sifr_diagnostics::DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES,
                        format!(
                            "cannot consume borrowed parameter '{name}' through Rust opaque cleanup; accept it with `own`"
                        ),
                        attr.value.range(),
                    );
                    return None;
                }
                HirExpr::Name { .. } => {}
                _ => {
                    ctx.error_with_code_at(
                        sifr_diagnostics::DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES,
                        "Rust opaque cleanup must consume an owned local binding; field and temporary receivers cannot prove exclusive ownership"
                            .to_string(),
                        attr.value.range(),
                    );
                    return None;
                }
            }
        }
        if ctx.python_consuming_methods.contains(&qualified) || consumes_rust_receiver {
            if let HirExpr::Name { name, .. } = &object {
                ctx.mark_moved_with_flow(name);
            }
        }
    }

    if matches!(object_ty.resolve_alias(), Type::PythonBuffer(_))
        && method_name == "release"
        && !super::consume_python_buffer_release_receiver(&object, attr.value.range(), ctx)
    {
        return None;
    }
    if matches!(object_ty.resolve_alias(), Type::PythonArrow(_))
        && method_name == "release"
        && !super::consume_python_arrow_release_receiver(&object, attr.value.range(), ctx)
    {
        return None;
    }
    if matches!(object_ty.resolve_alias(), Type::PythonDlpackTensor(_))
        && method_name == "release"
        && !super::consume_python_dlpack_release_receiver(&object, attr.value.range(), ctx)
    {
        return None;
    }

    let receiver_is_current_class = matches!(
        object_ty.resolve_alias(),
        Type::Class { name, .. } if ctx.current_class.as_deref() == Some(name)
    );
    super::super::generic_method_requirements::record_current_method_dependency(
        ctx,
        receiver_is_current_class,
        &method_name,
    );

    let receiver_convention = method_type
        .as_ref()
        .and_then(|signature| signature.receiver)
        .unwrap_or_else(|| {
            super::super::mutating_methods::receiver_convention_for_non_class_method(
                &object_ty,
                &method_name,
            )
        });
    let receiver_convention = if let Type::Class {
        name: class_name, ..
    } = object_ty.resolve_alias()
    {
        let qualified = format!("{class_name}.{method_name}");
        if ctx.python_consuming_methods.contains(&qualified)
            || ctx.rust_consuming_methods.contains(&qualified)
        {
            if receiver_convention.is_owned() {
                receiver_convention
            } else {
                ReceiverConvention::Owned
            }
        } else {
            receiver_convention
        }
    } else {
        receiver_convention
    };
    let interop_receiver_already_consumed = match object_ty.resolve_alias() {
        Type::Class { name, .. } => {
            let qualified = format!("{name}.{method_name}");
            ctx.python_consuming_methods.contains(&qualified)
                || ctx.rust_consuming_methods.contains(&qualified)
        }
        _ => false,
    };
    let declared_class_owned = matches!(object_ty.resolve_alias(), Type::Class { .. })
        && method_type
            .as_ref()
            .is_some_and(|signature| signature.receiver.is_some_and(ReceiverConvention::is_owned));
    if declared_class_owned && !interop_receiver_already_consumed {
        consume_declared_owned_receiver(&object, attr.value.range(), ctx);
    }
    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: method_name,
        args,
        receiver_convention: Some(receiver_convention),
        receiver_target: None,
        mutable_arg_places: Vec::new(),
        source: Some(MethodCallSource {
            call_range: call.range(),
            receiver_range: attr.value.range(),
            arg_ranges: method_arg_ranges,
        }),
        ty: return_ty,
    })
}

fn consume_declared_owned_receiver(object: &HirExpr, range: TextRange, ctx: &mut LowerCtx) {
    let HirExpr::Name { name, .. } = object else {
        ctx.error_with_code_at(
            DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES,
            "declared owned receiver must consume an owned local binding; field and temporary receivers cannot prove exclusive ownership".to_string(),
            range,
        );
        return;
    };
    if ctx.borrowed_params.contains(name) {
        ctx.error_with_code_at(
            DiagnosticCode::OWN_BORROWED_PARAMETER_ESCAPES,
            format!(
                "cannot consume borrowed parameter '{name}' through an owned receiver; accept it with `own`"
            ),
            range,
        );
    } else if ctx.scope.is_moved(name) {
        super::ownership_diagnostics::use_after_move(ctx, name, range);
    } else {
        ctx.mark_moved_with_flow(name);
    }
}

/// Resolve the return type of a method call on a given type.
pub(in crate::lower) fn resolve_method_type(
    object_ty: &Type,
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    let canonical_object_ty = canonicalize_class_surface_type(object_ty);
    let object_ty = &canonical_object_ty;
    if let Some(method_type) =
        python_raw_object_methods::method_type(object_ty, method, method_range, ctx)
    {
        return Some(method_type.return_type.as_ref().clone());
    }
    if let Type::Alias {
        name: alias_name,
        body,
        ..
    } = object_ty
    {
        if matches!(
            alias_name.as_str(),
            DEFAULTDICT_INT_ALIAS | DEFAULTDICT_LIST_ALIAS | DEFAULTDICT_SET_ALIAS
        ) {
            return resolve_method_type(body, method, args, arg_ranges, method_range, ctx);
        }
    }
    if matches!(object_ty, Type::AsyncGenerator(_, _)) {
        return super::async_generator_methods::resolve_async_generator_method_type(
            object_ty,
            method,
            args,
            arg_ranges,
            method_range,
            ctx,
        );
    }
    match object_ty {
        Type::List(elem_ty) => {
            resolve_list_method_type(elem_ty, method, args, arg_ranges, method_range, ctx)
        }
        Type::Dict(key_ty, val_ty) => {
            resolve_dict_method_type(key_ty, val_ty, method, args, arg_ranges, method_range, ctx)
        }
        Type::Set(elem_ty) => {
            resolve_set_method_type(elem_ty, method, args, arg_ranges, method_range, ctx)
        }
        Type::Str => resolve_str_method_type(method, args, arg_ranges, method_range, ctx),
        Type::Bytes => resolve_bytes_method_type(method, args, arg_ranges, method_range, ctx),
        Type::FixedInt(fixed) => {
            resolve_fixed_width_method_type(*fixed, method, args, arg_ranges, method_range, ctx)
        }
        Type::Tuple(_) => resolve_tuple_method_type(method, args, arg_ranges, method_range, ctx),
        Type::PythonBuffer(element) => {
            resolve_python_buffer_method_type(element, method, args, arg_ranges, method_range, ctx)
        }
        Type::PythonArrow(kind) => {
            resolve_python_arrow_method_type(*kind, method, args, arg_ranges, method_range, ctx)
        }
        Type::PythonDlpackTensor(element) => resolve_python_dlpack_method_type(
            Some(element),
            method,
            args,
            arg_ranges,
            method_range,
            ctx,
        ),
        Type::PythonDlpackStream => {
            resolve_python_dlpack_method_type(None, method, args, arg_ranges, method_range, ctx)
        }
        class @ Type::Class { .. } => {
            resolve_class_method_on_type(class, method, args, arg_ranges, method_range, ctx)
        }
        Type::Protocol { name, methods, .. } => {
            resolve_protocol_method_type(name, methods, method, args, arg_ranges, method_range, ctx)
        }
        Type::Newtype { name, inner, .. } => {
            resolve_newtype_method_type(name, inner, method, args, arg_ranges, method_range, ctx)
        }
        Type::Enum { name, .. } => {
            resolve_enum_method_type(name, method, args, arg_ranges, method_range, ctx)
        }
        Type::Decimal | Type::BigDecimal => {
            resolve_decimal_method_type(object_ty, method, args, arg_ranges, method_range, ctx)
        }
        _ => {
            ctx.error_with_code_at(
                DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE,
                format!(
                    "type '{}' has no method '{}'",
                    object_ty.display_name(),
                    method
                ),
                method_range,
            );
            None
        }
    }
}
