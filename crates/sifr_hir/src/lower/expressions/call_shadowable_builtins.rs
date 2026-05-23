fn lower_shadowable_builtin_call(
    func_name: &str,
    call: &ExprCall,
    ctx: &mut LowerCtx,
) -> Option<CallLowering> {
    if func_name == "any" {
        return CallLowering::from_option(lower_any_all_call(call, "any", ctx));
    }

    // all(iterable) -> bool
    if func_name == "all" {
        return CallLowering::from_option(lower_any_all_call(call, "all", ctx));
    }

    // map(func, iterable) -> iterator
    if func_name == "map" {
        return CallLowering::from_option(lower_map_call(call, ctx));
    }
    if func_name == "filter" {
        return CallLowering::from_option(lower_filter_call(call, ctx));
    }
    if func_name == "open" {
        let n_args = call.arguments.args.len();
        let _n_kwargs = call.arguments.keywords.len();
        let path_arg = if n_args >= 1 {
            lower_expr(&call.arguments.args[0], ctx)?
        } else {
            expression_diagnostics::call_missing_required_argument(
                ctx,
                "open() requires at least 1 argument: open(path) or open(path, mode)".to_string(),
                call.func.range(),
            );
            return None;
        };
        let mode_arg = if n_args >= 2 {
            lower_expr(&call.arguments.args[1], ctx)?
        } else if let Some(kw) = call
            .arguments
            .keywords
            .iter()
            .find(|k| k.arg.as_deref() == Some("mode"))
        {
            lower_expr(&kw.value, ctx)?
        } else {
            HirExpr::StringLiteral("r".to_string())
        };
        // Return type: FileHandle (raises IOError on failure — used in try/except blocks)
        // FileHandle methods are defined in io.sifr; register them here for type checking.
        let io_err_ty = Type::Class {
            name: "IOError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: vec![],
            parent_class: None,
        };
        let file_handle_ty = Type::Class {
            name: "FileHandle".to_string(),
            fields: vec![
                ("_handle".to_string(), Type::Int),
                ("_mode".to_string(), Type::Str),
            ],
            methods: vec![
                (
                    "read".to_string(),
                    FunctionType::all_borrow(
                        vec![],
                        Type::Result(Box::new(Type::Str), Box::new(io_err_ty.clone())),
                    ),
                ),
                (
                    "write".to_string(),
                    FunctionType::all_borrow(
                        vec![("data".to_string(), Type::Str)],
                        Type::Result(Box::new(Type::None), Box::new(io_err_ty.clone())),
                    ),
                ),
                (
                    "readline".to_string(),
                    FunctionType::all_borrow(
                        vec![],
                        Type::Result(
                            Box::new(Type::Union(vec![Type::Str, Type::None])),
                            Box::new(io_err_ty.clone()),
                        ),
                    ),
                ),
                (
                    "readlines".to_string(),
                    FunctionType::all_borrow(
                        vec![],
                        Type::Result(
                            Box::new(Type::List(Box::new(Type::Str))),
                            Box::new(io_err_ty.clone()),
                        ),
                    ),
                ),
                (
                    "close".to_string(),
                    FunctionType::all_borrow(vec![], Type::None),
                ),
                (
                    "read_bytes".to_string(),
                    FunctionType::all_borrow(
                        vec![],
                        Type::Result(Box::new(Type::Bytes), Box::new(io_err_ty.clone())),
                    ),
                ),
                (
                    "write_bytes".to_string(),
                    FunctionType::all_borrow(
                        vec![("data".to_string(), Type::Bytes)],
                        Type::Result(Box::new(Type::None), Box::new(io_err_ty.clone())),
                    ),
                ),
                (
                    "__enter__".to_string(),
                    FunctionType::all_borrow(
                        vec![],
                        Type::Class {
                            name: "FileHandle".to_string(),
                            fields: vec![
                                ("_handle".to_string(), Type::Int),
                                ("_mode".to_string(), Type::Str),
                            ],
                            methods: vec![],
                            parent_class: None,
                        },
                    ),
                ),
                (
                    "__exit__".to_string(),
                    FunctionType::all_borrow(vec![], Type::None),
                ),
            ],
            parent_class: None,
        };
        // Register FileHandle in the class types so method calls work
        ctx.class_types
            .insert("FileHandle".to_string(), file_handle_ty.clone());
        // Register IOError as a possible exception from this call
        ctx.try_block_error_types.insert("IOError".to_string());
        return Some(CallLowering::Lowered(HirExpr::Call {
            func: "builtin_open".to_string(),
            args: vec![path_arg, mode_arg],
            ty: file_handle_ty,
        }));
    }

    Some(CallLowering::NoMatch)
}
