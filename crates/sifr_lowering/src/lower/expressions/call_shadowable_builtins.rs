use super::{
    expression_diagnostics, lower_any_all_call, lower_expr, lower_filter_call, lower_map_call, str,
    CallLowering, ExprCall, FunctionType, HirExpr, LowerCtx, Ranged, Type,
};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::CompilerIntrinsicId;
use sifr_python_ast::Expr;

const TEXT_FILE_HANDLE_IDENTITY: &str = "sifr.io.TextFileHandle";
const FILE_HANDLE_IDENTITY: &str = "sifr.io.FileHandle";

fn class_type_with_identity(ctx: &LowerCtx, identity: &str) -> Option<Type> {
    ctx.class_types
        .values()
        .find(|candidate| {
            matches!(
                candidate.resolve_alias(),
                Type::Class {
                    identity: Some(candidate_identity),
                    ..
                } if candidate_identity == identity
            )
        })
        .cloned()
}

fn string_literal_value(expr: &Expr) -> Option<String> {
    match expr {
        Expr::StringLiteral(literal) => Some(literal.value.to_str().to_string()),
        _ => None,
    }
}

fn open_mode_expr(call: &ExprCall, n_args: usize) -> Option<&Expr> {
    if n_args >= 2 {
        return call.arguments.args.get(1);
    }
    call.arguments
        .keywords
        .iter()
        .find(|k| k.arg.as_deref() == Some("mode"))
        .map(|kw| &kw.value)
}

fn is_binary_open_mode(mode: &str) -> bool {
    mode.contains('b')
}

fn report_open_text_requires_encoding(ctx: &mut LowerCtx, range: ruff_text_size::TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::IO_TEXT_OPEN_REQUIRES_ENCODING,
        "text-mode open requires an explicit encoding; Sifr does not use locale-derived default encodings"
            .to_string(),
        range,
    );
}

fn report_open_mode_requires_literal(ctx: &mut LowerCtx, range: ruff_text_size::TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::IO_OPEN_MODE_REQUIRES_LITERAL,
        "open mode must be a string literal so Sifr can choose a binary or text handle type"
            .to_string(),
        range,
    );
}

fn report_static_encoding_handler_required(ctx: &mut LowerCtx, range: ruff_text_size::TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::ENCODING_HANDLER_REQUIRES_STATIC_VALUE,
        "encoding error handlers must be statically known typed values".to_string(),
        range,
    );
}

fn is_decode_handler_label(label: &str) -> bool {
    matches!(
        label,
        "strict" | "replace" | "ignore" | "backslashreplace" | "backslash-replace"
    )
}

fn is_encode_handler_label(label: &str) -> bool {
    matches!(
        label,
        "strict"
            | "replace"
            | "ignore"
            | "backslashreplace"
            | "backslash-replace"
            | "xmlcharrefreplace"
            | "xml-char-ref-replace"
            | "namereplace"
            | "name-replace"
    )
}

fn is_open_write_mode(mode: &str) -> bool {
    mode == "w" || mode == "wt" || mode == "a" || mode == "at"
}

pub(super) fn lower_shadowable_builtin_call(
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
        let mode_expr = open_mode_expr(call, n_args);
        let mode_literal = match mode_expr {
            Some(expr) => {
                if let Some(value) = string_literal_value(expr) {
                    value
                } else {
                    report_open_mode_requires_literal(ctx, expr.range());
                    return None;
                }
            }
            None => "r".to_string(),
        };
        let encoding_kw = call
            .arguments
            .keywords
            .iter()
            .find(|k| k.arg.as_deref() == Some("encoding"));
        let errors_kw = call
            .arguments
            .keywords
            .iter()
            .find(|k| k.arg.as_deref() == Some("errors"));
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
        if let Some(encoding_keyword) = encoding_kw {
            let encoding_arg = lower_expr(&encoding_keyword.value, ctx)?;
            if encoding_arg.ty() != &Type::Str {
                expression_diagnostics::type_mismatch(
                    ctx,
                    format!(
                        "open() encoding must be 'str', got '{}'",
                        encoding_arg.ty().display_name()
                    ),
                    encoding_keyword.value.range(),
                );
                return None;
            }
            let errors_arg = if let Some(keyword) = errors_kw {
                let Some(handler_label) = string_literal_value(&keyword.value) else {
                    report_static_encoding_handler_required(ctx, keyword.value.range());
                    return None;
                };
                let handler_allowed = if is_open_write_mode(mode_literal.as_str()) {
                    is_encode_handler_label(handler_label.as_str())
                } else {
                    is_decode_handler_label(handler_label.as_str())
                };
                if !handler_allowed {
                    report_static_encoding_handler_required(ctx, keyword.value.range());
                    return None;
                }
                let lowered = lower_expr(&keyword.value, ctx)?;
                if lowered.ty() != &Type::Str {
                    expression_diagnostics::type_mismatch(
                        ctx,
                        format!(
                            "open() errors must be 'str', got '{}'",
                            lowered.ty().display_name()
                        ),
                        keyword.value.range(),
                    );
                    return None;
                }
                lowered
            } else {
                HirExpr::StringLiteral("strict".to_string())
            };
            let io_err_ty = Type::Class {
                identity: None,
                type_args: Vec::new(),
                name: "IOError".to_string(),
                fields: vec![("message".to_string(), Type::Str)],
                methods: vec![],
                parent_class: None,
            };
            let text_handle_ty = Type::Class {
                identity: Some(TEXT_FILE_HANDLE_IDENTITY.to_string()),
                type_args: Vec::new(),
                name: "TextFileHandle".to_string(),
                fields: vec![],
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
                            vec![("text".to_string(), Type::Str)],
                            Type::Result(Box::new(Type::None), Box::new(io_err_ty.clone())),
                        ),
                    ),
                    (
                        "close".to_string(),
                        FunctionType::all_borrow(vec![], Type::None),
                    ),
                    (
                        "__enter__".to_string(),
                        FunctionType::all_borrow(
                            vec![],
                            Type::Class {
                                identity: Some(TEXT_FILE_HANDLE_IDENTITY.to_string()),
                                type_args: Vec::new(),
                                name: "TextFileHandle".to_string(),
                                fields: vec![],
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
            // Preserve an explicitly imported stdlib handle even when it is aliased.
            // A local same-basename class is a distinct declaration and must never be
            // selected for the compiler-special `open()` result.
            let text_handle_ty =
                class_type_with_identity(ctx, TEXT_FILE_HANDLE_IDENTITY).unwrap_or(text_handle_ty);
            ctx.try_block_error_types.insert("IOError".to_string());
            return Some(CallLowering::Lowered(HirExpr::IntrinsicCall {
                intrinsic: CompilerIntrinsicId::OpenText,
                args: vec![path_arg, mode_arg, encoding_arg, errors_arg],
                ty: text_handle_ty,
                call_range: call.range(),
                arg_ranges: call
                    .arguments
                    .args
                    .iter()
                    .map(Ranged::range)
                    .chain(std::iter::repeat(call.range()))
                    .take(4)
                    .collect(),
            }));
        }
        if !is_binary_open_mode(mode_literal.as_str()) {
            report_open_text_requires_encoding(ctx, call.func.range());
            return None;
        }
        // Return type: FileHandle (raises IOError on failure — used in try/except blocks)
        // FileHandle methods are defined in io.sifr; register them here for type checking.
        let io_err_ty = Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "IOError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: vec![],
            parent_class: None,
        };
        let file_handle_ty = Type::Class {
            identity: Some(FILE_HANDLE_IDENTITY.to_string()),
            type_args: Vec::new(),
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
                            identity: Some(FILE_HANDLE_IDENTITY.to_string()),
                            type_args: Vec::new(),
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
        let file_handle_ty =
            class_type_with_identity(ctx, FILE_HANDLE_IDENTITY).unwrap_or(file_handle_ty);
        // Register IOError as a possible exception from this call
        ctx.try_block_error_types.insert("IOError".to_string());
        return Some(CallLowering::Lowered(HirExpr::IntrinsicCall {
            intrinsic: CompilerIntrinsicId::OpenBinary,
            args: vec![path_arg, mode_arg],
            ty: file_handle_ty,
            call_range: call.range(),
            arg_ranges: call
                .arguments
                .args
                .iter()
                .map(Ranged::range)
                .chain(std::iter::repeat(call.range()))
                .take(2)
                .collect(),
        }));
    }

    Some(CallLowering::NoMatch)
}
