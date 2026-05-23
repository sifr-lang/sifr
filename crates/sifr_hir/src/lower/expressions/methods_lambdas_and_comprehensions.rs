pub(super) fn lower_method_call(
    attr: &ExprAttribute,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    // Handle super().__init__() and super().method() calls
    if let Expr::Call(super_call) = attr.value.as_ref() {
        if let Expr::Name(name) = super_call.func.as_ref() {
            if name.id.as_str() == "super" {
                let method_name = attr.attr.to_string();
                if let Some(parent_name) = ctx.current_parent_class.clone() {
                    // Lower arguments
                    let mut args = Vec::new();
                    for arg in &call.arguments.args {
                        let expr = lower_expr(arg, ctx)?;
                        args.push(expr);
                    }

                    return Some(HirExpr::SuperCall {
                        parent_class: parent_name,
                        method: if method_name == "__init__" {
                            "new".to_string()
                        } else {
                            method_name
                        },
                        args,
                        ty: Type::None,
                    });
                }
                ctx.error_with_code_at(
                    DiagnosticCode::CLASS_INVALID_BASE,
                    "super() used outside of a class with a parent".to_string(),
                    attr.value.range(),
                );
                return None;
            }
        }
    }

    // Handle ClassName.method() calls (classmethod/staticmethod)
    if let Expr::Name(name) = attr.value.as_ref() {
        let class_name = name.id.to_string();
        if ctx.class_types.contains_key(&class_name) {
            let method_name = attr.attr.to_string();
            // Lower arguments
            let mut args = Vec::new();
            for arg in &call.arguments.args {
                let expr = lower_expr(arg, ctx)?;
                args.push(expr);
            }
            // Look up the method's return type from the class type
            if let Some(Type::Class { methods, .. }) = ctx.class_types.get(&class_name) {
                if let Some((_, ft)) = methods.iter().find(|(n, _)| n == &method_name) {
                    let return_ty = *ft.return_type.clone();
                    return Some(HirExpr::Call {
                        func: format!("{class_name}::{method_name}"),
                        args,
                        ty: return_ty,
                    });
                }
            }
            ctx.error_with_code_at(
                DiagnosticCode::CLASS_MISSING_MEMBER,
                format!("type '{class_name}' has no class/static method '{method_name}'"),
                attr.attr.range(),
            );
            return None;
        }
    }

    let mut object = lower_expr(&attr.value, ctx)?;
    let method_name = attr.attr.to_string();
    if let Some(result) = super::blocking_executor_calls::lower_thread_pool_submit_call(
        &object,
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
    let args = match &object_ty_for_args {
        Type::Class { name, methods, .. } => {
            if let Some((_, ft)) = methods
                .iter()
                .find(|(candidate, _)| candidate == &method_name)
            {
                let ft = ft.clone();
                let defaults_key = format!("{name}.{method_name}");
                let method_defaults = ctx.function_defaults.get(&defaults_key).cloned();
                lower_signature_call_args(
                    call,
                    &format!("{name}.{method_name}"),
                    &ft,
                    method_defaults.as_deref(),
                    ctx,
                )?
            } else {
                lower_method_call_args(object.ty(), &method_name, call, ctx)?
            }
        }
        Type::Protocol { name, methods, .. } => {
            if let Some((_, ft)) = methods
                .iter()
                .find(|(candidate, _)| candidate == &method_name)
            {
                let ft = ft.clone();
                lower_signature_call_args(call, &format!("{name}.{method_name}"), &ft, None, ctx)?
            } else {
                lower_method_call_args(object.ty(), &method_name, call, ctx)?
            }
        }
        _ => lower_method_call_args(object.ty(), &method_name, call, ctx)?,
    };

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
    let method_arg_ranges = resolved_method_arg_ranges(&object_ty_for_args, &method_name, call);
    let resolved_method_type = resolve_method_type(
        &object_ty,
        &method_name,
        &args,
        &method_arg_ranges,
        attr.attr.range(),
        ctx,
    )?;
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
        let intrinsic_name = if args.is_empty() {
            "str_encode_utf8_result"
        } else {
            "str_encode_utf8_result_with_encoding"
        };
        if let Some(encoding) = args.first().cloned() {
            intrinsic_args.push(encoding);
        }
        return Some(HirExpr::Call {
            func: intrinsic_name.to_string(),
            args: intrinsic_args,
            ty: return_ty,
        });
    }
    if matches!(object_ty.resolve_alias(), Type::Bytes) && method_name == "decode" {
        let mut intrinsic_args = vec![object];
        let intrinsic_name = if args.is_empty() {
            "decode_utf8"
        } else {
            "decode_utf8_with_encoding"
        };
        if let Some(encoding) = args.first().cloned() {
            intrinsic_args.push(encoding);
        }
        return Some(HirExpr::Call {
            func: intrinsic_name.to_string(),
            args: intrinsic_args,
            ty: return_ty,
        });
    }

    Some(HirExpr::MethodCall {
        object: Box::new(object),
        method: method_name,
        args,
        ty: return_ty,
    })
}

/// Resolve the return type of a method call on a given type.
#[rustfmt::skip]
pub(super) fn resolve_method_type( object_ty: &Type, method: &str, args: &[HirExpr], arg_ranges: &[TextRange], method_range: TextRange, ctx: &mut LowerCtx, ) -> Option<Type> { let canonical_object_ty = canonicalize_class_surface_type(object_ty); let object_ty = &canonical_object_ty; if let Type::Alias { name: alias_name, body, .. } = object_ty { if matches!( alias_name.as_str(), DEFAULTDICT_INT_ALIAS | DEFAULTDICT_LIST_ALIAS | DEFAULTDICT_SET_ALIAS ) { return resolve_method_type(body, method, args, arg_ranges, method_range, ctx); } } if matches!(object_ty, Type::AsyncGenerator(_, _)) { return super::async_generator_methods::resolve_async_generator_method_type( object_ty, method, args, arg_ranges, method_range, ctx, ); } match object_ty { Type::List(elem_ty) => match method { "append" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, "list.append", 1, args.len(), arg_ranges, method_range, ); return None; } if !args[0].ty().is_assignable_to(elem_ty) { list_append_argument_type_mismatch(ctx, args[0].ty(), elem_ty, arg_ranges[0]); } Some(Type::None) } "extend" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, "list.extend", 1, args.len(), arg_ranges, method_range, ); return None; } validate_list_extend_arg(elem_ty, args[0].ty(), arg_ranges[0], ctx); Some(Type::None) } "insert" => { if args.len() != 2 { reject_exact_method_arg_count( ctx, "list.insert", 2, args.len(), arg_ranges, method_range, ); return None; } Some(Type::None) } "clear" => { if !args.is_empty() { reject_no_method_args(ctx, "list.clear", arg_ranges, method_range); return None; } Some(Type::None) } "copy" => { if !args.is_empty() { reject_no_method_args(ctx, "list.copy", arg_ranges, method_range); return None; } Some(Type::List(elem_ty.clone())) } "reverse" => { if !args.is_empty() { reject_no_method_args(ctx, "list.reverse", arg_ranges, method_range); return None; } Some(Type::None) } "sort" => { if args.len() > 1 { reject_max_method_arg_count( ctx, "list.sort", 1, args.len(), arg_ranges, method_range, ); return None; } if let Some(reverse_arg) = args.first() { if reverse_arg.ty() != &Type::Bool { expression_diagnostics::type_mismatch( ctx, format!( "list.sort() argument 'reverse' must be 'bool', got '{}'", reverse_arg.ty().display_name() ), arg_ranges[0], ); return None; } } Some(Type::None) } "count" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, "list.count", 1, args.len(), arg_ranges, method_range, ); return None; } Some(Type::Int) } "contains" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, "list.contains", 1, args.len(), arg_ranges, method_range, ); return None; } Some(Type::Bool) } "len" => { if !args.is_empty() { reject_no_method_args(ctx, "list.len", arg_ranges, method_range); return None; } Some(Type::Int) } "pop" => { if args.len() > 1 { reject_max_method_arg_count( ctx, "list.pop", 1, args.len(), arg_ranges, method_range, ); return None; } if let Some(index_arg) = args.first() { if index_arg.ty() != &Type::Int { expression_diagnostics::type_mismatch( ctx, format!( "list.pop() index must be 'int', got '{}'", index_arg.ty().display_name() ), arg_ranges[0], ); } } Some(Type::Union(vec![*elem_ty.clone(), Type::None])) } "popleft" => { if !args.is_empty() { reject_no_method_args(ctx, "list.popleft", arg_ranges, method_range); return None; } Some(Type::Union(vec![*elem_ty.clone(), Type::None])) } "appendleft" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, "list.appendleft", 1, args.len(), arg_ranges, method_range, ); return None; } Some(Type::None) } "remove" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, "list.remove", 1, args.len(), arg_ranges, method_range, ); return None; } Some(Type::None) } "index" => { if args.is_empty() || args.len() > 3 { reject_method_arg_count( ctx, format!("list.index() takes 1 to 3 arguments, got {}", args.len()), method_count_range(args.len(), 3, arg_ranges, method_range), ); return None; } for (bound_index, bound) in args.iter().enumerate().skip(1) { if bound.ty() != &Type::Int { expression_diagnostics::type_mismatch( ctx, format!( "list.index() bounds must be 'int', got '{}'", bound.ty().display_name() ), arg_ranges.get(bound_index).copied().unwrap_or(method_range), ); } } Some(Type::Union(vec![Type::Int, Type::None])) } _ => { ctx.error_with_code_at( DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE, format!("list has no method '{method}'"), method_range, ); None } }, Type::Dict(key_ty, val_ty) => match method { "len" => { if !args.is_empty() { reject_no_method_args(ctx, "dict.len", arg_ranges, method_range); return None; } Some(Type::Int) } "keys" => { if !args.is_empty() { reject_no_method_args(ctx, "dict.keys", arg_ranges, method_range); return None; } Some(Type::List(key_ty.clone())) } "values" => { if !args.is_empty() { reject_no_method_args(ctx, "dict.values", arg_ranges, method_range); return None; } Some(Type::List(val_ty.clone())) } "items" => { if !args.is_empty() { reject_no_method_args(ctx, "dict.items", arg_ranges, method_range); return None; } Some(Type::List(Box::new(Type::Tuple(vec![ *key_ty.clone(), *val_ty.clone(), ])))) } "update" => { if args.len() > 2 { reject_max_method_arg_count( ctx, "dict.update", 2, args.len(), arg_ranges, method_range, ); return None; } if let Some(arg) = args.first() { validate_dict_update_arg(key_ty, val_ty, arg.ty(), arg_ranges[0], ctx); } if let Some(keyword_dict) = args.get(1) { validate_dict_update_arg(key_ty, val_ty, keyword_dict.ty(), arg_ranges[1], ctx); } Some(Type::None) } "clear" => { if !args.is_empty() { reject_no_method_args(ctx, "dict.clear", arg_ranges, method_range); return None; } Some(Type::None) } "copy" => { if !args.is_empty() { reject_no_method_args(ctx, "dict.copy", arg_ranges, method_range); return None; } Some(Type::Dict(key_ty.clone(), val_ty.clone())) } "contains" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, "dict.contains", 1, args.len(), arg_ranges, method_range, ); return None; } if !args[0].ty().is_assignable_to(key_ty) { expression_diagnostics::type_mismatch( ctx, format!( "dict.contains() key type '{}' is not compatible with dict key type '{}'", args[0].ty().display_name(), key_ty.display_name() ), arg_ranges[0], ); } Some(Type::Bool) } "get" => { if args.is_empty() || args.len() > 2 { reject_method_arg_count( ctx, format!("dict.get() takes 1 or 2 arguments, got {}", args.len()), method_count_range(args.len(), 2, arg_ranges, method_range), ); return None; } if !args[0].ty().is_assignable_to(key_ty) { expression_diagnostics::type_mismatch( ctx, format!( "dict.get() key type '{}' is not compatible with dict key type '{}'", args[0].ty().display_name(), key_ty.display_name() ), arg_ranges[0], ); } if args.len() == 2 { if !args[1].ty().is_assignable_to(val_ty) { expression_diagnostics::type_mismatch( ctx, format!( "dict.get() default type '{}' is not compatible with dict value type '{}'", args[1].ty().display_name(), val_ty.display_name() ), arg_ranges[1], ); } if matches!(val_ty.as_ref(), Type::Any | Type::Unknown) { Some(args[1].ty().clone()) } else { Some(*val_ty.clone()) } } else { Some(Type::Union(vec![*val_ty.clone(), Type::None])) } } "pop" => { if args.is_empty() || args.len() > 2 { reject_method_arg_count( ctx, format!("dict.pop() takes 1 or 2 arguments, got {}", args.len()), method_count_range(args.len(), 2, arg_ranges, method_range), ); return None; } if !args[0].ty().is_assignable_to(key_ty) { expression_diagnostics::type_mismatch( ctx, format!( "dict.pop() key type '{}' is not compatible with dict key type '{}'", args[0].ty().display_name(), key_ty.display_name() ), arg_ranges[0], ); } if args.len() == 2 { if !args[1].ty().is_assignable_to(val_ty) { expression_diagnostics::type_mismatch( ctx, format!( "dict.pop() default type '{}' is not compatible with dict value type '{}'", args[1].ty().display_name(), val_ty.display_name() ), arg_ranges[1], ); } Some(*val_ty.clone()) } else { Some(Type::Union(vec![*val_ty.clone(), Type::None])) } } "setdefault" => { if args.len() != 2 { reject_exact_method_arg_count( ctx, "dict.setdefault", 2, args.len(), arg_ranges, method_range, ); return None; } if !args[0].ty().is_assignable_to(key_ty) { expression_diagnostics::type_mismatch( ctx, format!( "dict.setdefault() key type '{}' is not compatible with dict key type '{}'", args[0].ty().display_name(), key_ty.display_name() ), arg_ranges[0], ); } if !args[1].ty().is_assignable_to(val_ty) { expression_diagnostics::type_mismatch( ctx, format!( "dict.setdefault() default type '{}' is not compatible with dict value type '{}'", args[1].ty().display_name(), val_ty.display_name() ), arg_ranges[1], ); } Some(*val_ty.clone()) } _ => { ctx.error_with_code_at( DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE, format!("dict has no method '{method}'"), method_range, ); None } }, Type::Set(elem_ty) => match method { "len" => { if !args.is_empty() { reject_no_method_args(ctx, "set.len", arg_ranges, method_range); return None; } Some(Type::Int) } "add" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, "set.add", 1, args.len(), arg_ranges, method_range, ); return None; } Some(Type::None) } "remove" | "discard" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, &format!("set.{method}"), 1, args.len(), arg_ranges, method_range, ); return None; } Some(Type::None) } "contains" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, "set.contains", 1, args.len(), arg_ranges, method_range, ); return None; } Some(Type::Bool) } "clear" => { if !args.is_empty() { reject_no_method_args(ctx, "set.clear", arg_ranges, method_range); return None; } Some(Type::None) } "copy" => { if !args.is_empty() { reject_no_method_args(ctx, "set.copy", arg_ranges, method_range); return None; } Some(Type::Set(elem_ty.clone())) } "union" | "intersection" | "difference" => { for (index, arg) in args.iter().enumerate() { validate_set_iterable_arg(elem_ty, arg.ty(), method, arg_ranges[index], ctx); } Some(Type::Set(elem_ty.clone())) } "symmetric_difference" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, &format!("set.{method}"), 1, args.len(), arg_ranges, method_range, ); return None; } validate_set_iterable_arg(elem_ty, args[0].ty(), method, arg_ranges[0], ctx); Some(Type::Set(elem_ty.clone())) } "update" | "intersection_update" | "difference_update" => { for (index, arg) in args.iter().enumerate() { validate_set_iterable_arg(elem_ty, arg.ty(), method, arg_ranges[index], ctx); } Some(Type::None) } "symmetric_difference_update" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, &format!("set.{method}"), 1, args.len(), arg_ranges, method_range, ); return None; } validate_set_iterable_arg(elem_ty, args[0].ty(), method, arg_ranges[0], ctx); Some(Type::None) } "issubset" | "issuperset" | "isdisjoint" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, &format!("set.{method}"), 1, args.len(), arg_ranges, method_range, ); return None; } validate_set_iterable_arg(elem_ty, args[0].ty(), method, arg_ranges[0], ctx); Some(Type::Bool) } "pop" => { if !args.is_empty() { reject_no_method_args(ctx, "set.pop", arg_ranges, method_range); return None; } Some(Type::Union(vec![*elem_ty.clone(), Type::None])) } _ => { ctx.error_with_code_at( DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE, format!("set has no method '{method}'"), method_range, ); None } }, Type::Str => match method { "len" => Some(Type::Int), "upper" | "lower" | "strip" | "lstrip" | "rstrip" | "title" | "capitalize" | "swapcase" => Some(Type::Str), "startswith" | "endswith" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, &format!("str.{method}"), 1, args.len(), arg_ranges, method_range, ); return None; } Some(Type::Bool) } "isdigit" | "isalpha" | "isalnum" | "isspace" | "isupper" | "islower" => { if !args.is_empty() { reject_no_method_args(ctx, &format!("str.{method}"), arg_ranges, method_range); return None; } Some(Type::Bool) } "split" => { if args.len() > 2 { reject_method_arg_count( ctx, format!("str.split() takes 0 to 2 arguments, got {}", args.len()), method_count_range(args.len(), 2, arg_ranges, method_range), ); return None; } if let Some(maxsplit) = args.get(1) { if maxsplit.ty() != &Type::Int { expression_diagnostics::type_mismatch( ctx, format!( "str.split() maxsplit must be 'int', got '{}'", maxsplit.ty().display_name() ), arg_ranges[1], ); } } Some(Type::List(Box::new(Type::Str))) } "replace" => { if args.len() < 2 || args.len() > 3 { reject_method_arg_count( ctx, format!("str.replace() takes 2 or 3 arguments, got {}", args.len()), method_count_range(args.len(), 3, arg_ranges, method_range), ); return None; } if let Some(count) = args.get(2) { if count.ty() != &Type::Int { expression_diagnostics::type_mismatch( ctx, format!( "str.replace() count must be 'int', got '{}'", count.ty().display_name() ), arg_ranges[2], ); } } Some(Type::Str) } "join" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, "str.join", 1, args.len(), arg_ranges, method_range, ); return None; } Some(Type::Str) } "count" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, "str.count", 1, args.len(), arg_ranges, method_range, ); return None; } Some(Type::Int) } "center" | "ljust" | "rjust" | "zfill" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, &format!("str.{method}"), 1, args.len(), arg_ranges, method_range, ); return None; } Some(Type::Str) } "find" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, "str.find", 1, args.len(), arg_ranges, method_range, ); return None; } Some(Type::Union(vec![Type::Int, Type::None])) } "encode" => resolve_str_encode_method_type(args, arg_ranges, method_range, ctx), _ => { ctx.error_with_code_at( DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE, format!("str has no method '{method}'"), method_range, ); None } }, Type::Bytes => resolve_bytes_method_type(method, args, arg_ranges, method_range, ctx), Type::FixedInt(fixed) => { resolve_fixed_width_method_type(*fixed, method, args, arg_ranges, method_range, ctx) } Type::Tuple(_) => match method { "len" => Some(Type::Int), "count" => { if args.len() != 1 { reject_exact_method_arg_count( ctx, "tuple.count", 1, args.len(), arg_ranges, method_range, ); return None; } Some(Type::Int) } "index" => { if args.is_empty() || args.len() > 3 { reject_method_arg_count( ctx, format!("tuple.index() takes 1 to 3 arguments, got {}", args.len()), method_count_range(args.len(), 3, arg_ranges, method_range), ); return None; } for (bound_index, bound) in args.iter().enumerate().skip(1) { if bound.ty() != &Type::Int { expression_diagnostics::type_mismatch( ctx, format!( "tuple.index() bounds must be 'int', got '{}'", bound.ty().display_name() ), arg_ranges.get(bound_index).copied().unwrap_or(method_range), ); } } Some(Type::Union(vec![Type::Int, Type::None])) } _ => { ctx.error_with_code_at( DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE, format!("tuple has no method '{method}'"), method_range, ); None } }, Type::Class { name, fields, methods, .. } => { if let Some((_, ft)) = methods.iter().find(|(n, _)| n == method) { if args.len() != ft.params.len() { reject_method_arg_count( ctx, format!( "{}.{}() takes {} argument(s), got {}", name, method, ft.params.len(), args.len() ), method_count_range(args.len(), ft.params.len(), arg_ranges, method_range), ); return None; } for (i, (arg, (param_name, param_ty, _))) in args.iter().zip(ft.params.iter()).enumerate() { if !arg.ty().is_assignable_to(param_ty) { expression_diagnostics::type_mismatch( ctx, format!( "argument {} ('{}') of {}.{}(): expected '{}', got '{}'", i + 1, param_name, name, method, param_ty.display_name(), arg.ty().display_name() ), arg_ranges.get(i).copied().unwrap_or(method_range), ); } } Some(canonicalize_class_surface_type(&ft.return_type)) } else if let Some((_, field_ty)) = fields.iter().find(|(n, _)| n == method) { if let Type::Callable(param_types, _, ret_type) = field_ty { if args.len() != param_types.len() { expression_diagnostics::call_not_callable_or_arity( ctx, format!( "{}.{}() (callable field) takes {} argument(s), got {}", name, method, param_types.len(), args.len() ), method_count_range( args.len(), param_types.len(), arg_ranges, method_range, ), ); return None; } for (i, (arg, param_ty)) in args.iter().zip(param_types.iter()).enumerate() { if !arg.ty().is_assignable_to(param_ty) { expression_diagnostics::type_mismatch( ctx, format!( "argument {} of {}.{}(): expected '{}', got '{}'", i + 1, name, method, param_ty.display_name(), arg.ty().display_name() ), arg_ranges.get(i).copied().unwrap_or(method_range), ); } } Some(canonicalize_class_surface_type(ret_type)) } else { expression_diagnostics::call_not_callable_or_arity( ctx, format!( "field '{}' of class '{}' is not callable (type: '{}')", method, name, field_ty.display_name() ), method_range, ); None } } else { ctx.error_with_code_at( DiagnosticCode::CLASS_MISSING_MEMBER, format!("class '{name}' has no method '{method}'"), method_range, ); None } } Type::Protocol { name, methods, .. } => { if let Some((_, ft)) = methods.iter().find(|(n, _)| n == method) { if args.len() != ft.params.len() { reject_method_arg_count( ctx, format!( "{}.{}() takes {} argument(s), got {}", name, method, ft.params.len(), args.len() ), method_count_range(args.len(), ft.params.len(), arg_ranges, method_range), ); return None; } Some(canonicalize_class_surface_type(&ft.return_type)) } else { ctx.error_with_code_at( DiagnosticCode::PROTO_BOUND_NOT_SATISFIED, format!("protocol '{name}' has no method '{method}'"), method_range, ); None } } Type::Newtype { name, inner } => { if method == "value" { if !args.is_empty() { reject_no_method_args(ctx, &format!("{name}.value"), arg_ranges, method_range); return None; } Some(*inner.clone()) } else { resolve_method_type(inner, method, args, arg_ranges, method_range, ctx) } } Type::Enum { name, .. } => { match method { "name" => { if !args.is_empty() { reject_no_method_args( ctx, &format!("{name}.name"), arg_ranges, method_range, ); return None; } Some(Type::Str) } "value" => { if !args.is_empty() { reject_no_method_args( ctx, &format!("{name}.value"), arg_ranges, method_range, ); return None; } Some(Type::Int) } _ => { let method_key = format!("{name}.{method}"); if let Some(ft) = ctx.functions.get(&method_key).cloned() { return Some(*ft.return_type.clone()); } ctx.error_with_code_at( DiagnosticCode::CLASS_MISSING_MEMBER, format!("enum '{name}' has no method '{method}'"), method_range, ); None } } } Type::BigInt => { if method == "clone" { if !args.is_empty() { reject_no_method_args(ctx, "bigint.clone", arg_ranges, method_range); return None; } Some(Type::BigInt) } else { ctx.error_with_code_at( DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE, format!("type 'bigint' has no method '{method}'"), method_range, ); None } } Type::Decimal | Type::BigDecimal => { resolve_decimal_method_type(object_ty, method, args, arg_ranges, method_range, ctx) } _ => { ctx.error_with_code_at( DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE, format!( "type '{}' has no method '{}'", object_ty.display_name(), method ), method_range, ); None } } } 
pub(super) fn lower_lambda_with_context(
    expr: &Expr,
    context_types: &[Type],
    ctx: &mut LowerCtx,
) -> Option<HirExpr> {
    if let Expr::Lambda(lambda) = expr {
        let (params, body, body_ty) = ctx.with_pushed_scope(|ctx| {
            let mut params = Vec::new();
            if let Some(ref parameters) = lambda.parameters {
                for (i, param) in parameters.args.iter().enumerate() {
                    let param_name = param.parameter.name.to_string();
                    let param_ty = if let Some(ref ann) = param.parameter.annotation {
                        resolve_annotation_expr(ann, ctx)
                    } else if i < context_types.len() {
                        // Use contextual type
                        context_types[i].clone()
                    } else {
                        Type::Any
                    };
                    ctx.scope.define(param_name.clone(), param_ty.clone());
                    params.push(HirParam {
                        name: param_name,
                        ty: param_ty,
                        default: None,
                        keyword_only: false,
                        convention: ParamConvention::default(),
                    });
                }
            }

            let body = lower_expr(&lambda.body, ctx)?;
            let body_ty = body.ty().clone();
            Some((params, body, body_ty))
        })?;

        let param_types: Vec<(String, Type)> = params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        let fn_ty = Type::Function(FunctionType::new(param_types, body_ty));

        Some(HirExpr::Lambda {
            params,
            body: Box::new(body),
            ty: fn_ty,
        })
    } else {
        // Not a lambda, lower normally
        lower_expr(expr, ctx)
    }
}

pub(super) fn lower_lambda(lambda: &ExprLambda, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let (params, body, body_ty) = ctx.with_pushed_scope(|ctx| {
        let mut params = Vec::new();
        if let Some(ref parameters) = lambda.parameters {
            for param in &parameters.args {
                let param_name = param.parameter.name.to_string();
                let param_ty = if let Some(ref ann) = param.parameter.annotation {
                    resolve_annotation_expr(ann, ctx)
                } else {
                    // Unannotated lambda params start as Any and may be refined
                    // by contextual typing at call sites.
                    Type::Any
                };
                ctx.scope.define(param_name.clone(), param_ty.clone());
                params.push(HirParam {
                    name: param_name,
                    ty: param_ty,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::default(),
                });
            }
        }

        let body = lower_expr(&lambda.body, ctx)?;
        let body_ty = body.ty().clone();
        Some((params, body, body_ty))
    })?;

    // Build the function type for the lambda
    let param_types: Vec<(String, Type)> = params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    let fn_ty = Type::Function(FunctionType::new(param_types, body_ty));

    Some(HirExpr::Lambda {
        params,
        body: Box::new(body),
        ty: fn_ty,
    })
}

fn reject_invalid_expression_target(ctx: &mut LowerCtx, message: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::FLOW_INVALID_ASSIGNMENT_TARGET,
        message.to_string(),
        range,
    );
}

fn reject_invalid_expression_iteration(ctx: &mut LowerCtx, iter_ty: &Type, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::FLOW_INVALID_ITERATION,
        format!("cannot iterate over type '{}'", iter_ty.display_name()),
        range,
    );
}

fn reject_unsupported_expression_form(ctx: &mut LowerCtx, message: &str, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM,
        message.to_string(),
        range,
    );
}

pub(super) fn lower_list_comp(comp: &ExprListComp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if comp.generators.is_empty() {
        reject_unsupported_expression_form(
            ctx,
            "list comprehension must have at least one generator",
            comp.range(),
        );
        return None;
    }

    if let Some(result) = super::async_comprehensions::lower_list_comp(comp, ctx) {
        return result;
    }

    if super::async_comprehension_diagnostics::reject_deferred_async_comprehension_shape(
        ctx,
        "list",
        &comp.generators,
        comp.range(),
    ) {
        return None;
    }

    let mut generators = Vec::new();
    let mut pushed_scopes = 0;
    let result = (|| {
        // Process each generator: push scope, define var, lower iter
        for gen in &comp.generators {
            let var_name = match &gen.target {
                Expr::Name(n) => n.id.to_string(),
                Expr::Tuple(tup) => {
                    let names: Vec<String> = tup
                        .elts
                        .iter()
                        .filter_map(|e| {
                            if let Expr::Name(n) = e {
                                Some(n.id.to_string())
                            } else {
                                None
                            }
                        })
                        .collect();
                    if names.len() != tup.elts.len() {
                        reject_invalid_expression_target(
                            ctx,
                            "comprehension tuple target must contain only simple names",
                            gen.target.range(),
                        );
                        return None;
                    }
                    names.join(",")
                }
                _ => {
                    reject_invalid_expression_target(
                        ctx,
                        "comprehension target must be a simple name or tuple",
                        gen.target.range(),
                    );
                    return None;
                }
            };

            let iter_source_expr = lower_expr(&gen.iter, ctx)?;
            let iter_ty = iter_source_expr.ty().clone();
            let Some(elem_ty) = callable_builtin_element_type(&iter_ty) else {
                reject_invalid_expression_iteration(ctx, &iter_ty, gen.iter.range());
                return None;
            };

            ctx.scope.push();
            pushed_scopes += 1;
            if var_name.contains(',') {
                let names: Vec<&str> = var_name.split(',').collect();
                if let Type::Tuple(elem_types) = &elem_ty {
                    for (i, name) in names.iter().enumerate() {
                        let ty = elem_types.get(i).cloned().unwrap_or(Type::Any);
                        ctx.scope.define((*name).to_string(), ty);
                    }
                } else {
                    for name in &names {
                        ctx.scope.define((*name).to_string(), Type::Any);
                    }
                }
            } else {
                ctx.scope.define(var_name.clone(), elem_ty.clone());
            }

            let filter = if gen.ifs.is_empty() {
                None
            } else {
                let first = lower_expr(&gen.ifs[0], ctx)?;
                if gen.ifs.len() == 1 {
                    Some(first)
                } else {
                    let mut combined = first;
                    for cond in &gen.ifs[1..] {
                        let next = lower_expr(cond, ctx)?;
                        combined = HirExpr::BoolOp {
                            op: "and".to_string(),
                            values: vec![combined, next],
                            ty: Type::Bool,
                        };
                    }
                    Some(combined)
                }
            };

            let iter_expr = lower_iterator_protocol_entry(iter_source_expr, elem_ty);
            generators.push((var_name, iter_expr, filter));
        }

        // Lower the expression (all generator vars are in scope)
        let expr = lower_expr(&comp.elt, ctx)?;
        let expr_ty = expr.ty().clone();
        let result_ty = Type::List(Box::new(expr_ty));

        Some(HirExpr::ListComp {
            expr: Box::new(expr),
            generators,
            ty: result_ty,
        })
    })();
    ctx.pop_scopes(pushed_scopes);
    result
}

pub(super) fn lower_set_comp(comp: &ExprSetComp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if super::async_comprehension_diagnostics::reject_deferred_async_comprehension_shape(
        ctx,
        "set",
        &comp.generators,
        comp.range(),
    ) {
        return None;
    }

    let mut generators = Vec::new();
    let mut pushed_scopes = 0;
    let result = (|| {
        for gen in &comp.generators {
            let var_name = if let Expr::Name(n) = &gen.target {
                n.id.to_string()
            } else {
                reject_invalid_expression_target(
                    ctx,
                    "set comprehension target must be a simple name",
                    gen.target.range(),
                );
                return None;
            };
            let iter_source_expr = lower_expr(&gen.iter, ctx)?;
            let iter_ty = iter_source_expr.ty().clone();
            let Some(elem_ty) = callable_builtin_element_type(&iter_ty) else {
                reject_invalid_expression_iteration(ctx, &iter_ty, gen.iter.range());
                return None;
            };
            ctx.scope.push();
            pushed_scopes += 1;
            ctx.scope.define(var_name.clone(), elem_ty.clone());
            let filter = if gen.ifs.is_empty() {
                None
            } else {
                Some(lower_expr(&gen.ifs[0], ctx)?)
            };
            let iter_expr = lower_iterator_protocol_entry(iter_source_expr, elem_ty);
            generators.push((var_name, iter_expr, filter));
        }
        let expr = lower_expr(&comp.elt, ctx)?;
        let expr_ty = expr.ty().clone();
        let result_ty = Type::Set(Box::new(expr_ty));
        Some(HirExpr::SetComp {
            expr: Box::new(expr),
            generators,
            ty: result_ty,
        })
    })();
    ctx.pop_scopes(pushed_scopes);
    result
}

pub(super) fn lower_dict_comp(comp: &ExprDictComp, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if super::async_comprehension_diagnostics::reject_deferred_async_comprehension_shape(
        ctx,
        "dict",
        &comp.generators,
        comp.range(),
    ) {
        return None;
    }

    let mut generators = Vec::new();
    let mut pushed_scopes = 0;
    let result = (|| {
        for gen in &comp.generators {
            let var_name = match &gen.target {
                Expr::Name(n) => n.id.to_string(),
                Expr::Tuple(tup) => {
                    let names: Vec<String> = tup
                        .elts
                        .iter()
                        .filter_map(|e| {
                            if let Expr::Name(n) = e {
                                Some(n.id.to_string())
                            } else {
                                None
                            }
                        })
                        .collect();
                    if names.len() != tup.elts.len() {
                        reject_invalid_expression_target(
                            ctx,
                            "dict comprehension tuple target must contain only simple names",
                            gen.target.range(),
                        );
                        return None;
                    }
                    names.join(",")
                }
                _ => {
                    reject_invalid_expression_target(
                        ctx,
                        "dict comprehension target must be a simple name or tuple",
                        gen.target.range(),
                    );
                    return None;
                }
            };
            let iter_source_expr = lower_expr(&gen.iter, ctx)?;
            let iter_ty = iter_source_expr.ty().clone();
            let Some(elem_ty) = callable_builtin_element_type(&iter_ty) else {
                reject_invalid_expression_iteration(ctx, &iter_ty, gen.iter.range());
                return None;
            };
            ctx.scope.push();
            pushed_scopes += 1;
            if var_name.contains(',') {
                let names: Vec<&str> = var_name.split(',').collect();
                if let Type::Tuple(elem_types) = &elem_ty {
                    for (i, name) in names.iter().enumerate() {
                        let ty = elem_types.get(i).cloned().unwrap_or(Type::Any);
                        ctx.scope.define((*name).to_string(), ty);
                    }
                } else {
                    for name in &names {
                        ctx.scope.define((*name).to_string(), Type::Any);
                    }
                }
            } else {
                ctx.scope.define(var_name.clone(), elem_ty.clone());
            }
            let filter = if gen.ifs.is_empty() {
                None
            } else {
                Some(lower_expr(&gen.ifs[0], ctx)?)
            };
            let iter_expr = lower_iterator_protocol_entry(iter_source_expr, elem_ty);
            generators.push((var_name, iter_expr, filter));
        }
        let key_expr = lower_expr(&comp.key, ctx)?;
        let val_expr = lower_expr(&comp.value, ctx)?;
        let key_ty = key_expr.ty().clone();
        let val_ty = val_expr.ty().clone();
        let result_ty = Type::Dict(Box::new(key_ty), Box::new(val_ty));
        Some(HirExpr::DictComp {
            key_expr: Box::new(key_expr),
            val_expr: Box::new(val_expr),
            generators,
            ty: result_ty,
        })
    })();
    ctx.pop_scopes(pushed_scopes);
    result
}

pub(super) fn lower_generator_expr(gen: &ExprGenerator, ctx: &mut LowerCtx) -> Option<HirExpr> {
    if gen.generators.iter().any(|generator| generator.is_async) {
        super::async_comprehension_diagnostics::reject_async_generator_expression(ctx, gen.range());
        return None;
    }

    // Only support single generator: (expr for var in iter) or (expr for var in iter if cond)
    if gen.generators.len() != 1 {
        reject_unsupported_expression_form(
            ctx,
            "only single-generator generator expressions are supported",
            gen.range(),
        );
        return None;
    }

    let comp = &gen.generators[0];

    let var_name = if let Expr::Name(n) = &comp.target {
        n.id.to_string()
    } else {
        reject_invalid_expression_target(
            ctx,
            "generator target must be a simple name",
            comp.target.range(),
        );
        return None;
    };
    let iter_source_expr = lower_expr(&comp.iter, ctx)?;
    let iter_ty = iter_source_expr.ty().clone();
    let Some(elem_ty) = callable_builtin_element_type(&iter_ty) else {
        reject_invalid_expression_iteration(ctx, &iter_ty, comp.iter.range());
        return None;
    };

    let (expr, expr_ty, filter) = ctx.with_pushed_scope(|ctx| {
        ctx.scope.define(var_name.clone(), elem_ty.clone());
        let expr = lower_expr(&gen.elt, ctx)?;
        let expr_ty = expr.ty().clone();
        let filter = if comp.ifs.is_empty() {
            None
        } else {
            let first = lower_expr(&comp.ifs[0], ctx)?;
            if comp.ifs.len() == 1 {
                Some(Box::new(first))
            } else {
                let mut combined = first;
                for cond in &comp.ifs[1..] {
                    let next = lower_expr(cond, ctx)?;
                    combined = HirExpr::BoolOp {
                        op: "and".to_string(),
                        values: vec![combined, next],
                        ty: Type::Bool,
                    };
                }
                Some(Box::new(combined))
            }
        };
        Some((expr, expr_ty, filter))
    })?;
    let result_ty = Type::Iterator(Box::new(expr_ty));
    let iter_expr = lower_iterator_protocol_entry(iter_source_expr, elem_ty);
    Some(HirExpr::GeneratorExpr {
        expr: Box::new(expr),
        var: var_name,
        iter: Box::new(iter_expr),
        filter,
        ty: result_ty,
    })
}

fn lower_iterator_protocol_entry(iter_source_expr: HirExpr, elem_ty: Type) -> HirExpr {
    HirExpr::IteratorCall {
        op: HirIteratorOp::Iter,
        args: vec![iter_source_expr],
        ty: Type::Iterator(Box::new(elem_ty)),
    }
}

pub(super) fn lower_named_expr(named: &ExprNamed, ctx: &mut LowerCtx) -> Option<HirExpr> {
    let name = if let Expr::Name(n) = named.target.as_ref() {
        n.id.to_string()
    } else {
        reject_invalid_expression_target(
            ctx,
            "walrus operator target must be a simple name",
            named.target.range(),
        );
        return None;
    };

    let value = lower_expr(&named.value, ctx)?;
    let ty = value.ty().clone();

    // Define the variable in the current scope
    ctx.scope.define(name.clone(), ty.clone());

    Some(HirExpr::WalrusExpr {
        name,
        value: Box::new(value),
        ty,
    })
}
