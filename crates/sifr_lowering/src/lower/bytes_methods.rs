use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_type_system::Type;

fn parse_error_type(ctx: &LowerCtx) -> Type {
    ctx.class_types
        .get("ParseError")
        .cloned()
        .unwrap_or(Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "ParseError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: vec![],
            parent_class: Some("Error".to_string()),
        })
}

fn arity_range(arg_ranges: &[TextRange], method_range: TextRange) -> TextRange {
    arg_ranges.last().copied().unwrap_or(method_range)
}

fn arg_range(arg_ranges: &[TextRange], index: usize, method_range: TextRange) -> TextRange {
    arg_ranges.get(index).copied().unwrap_or(method_range)
}

fn reject_wrong_count(ctx: &mut LowerCtx, message: String, range: TextRange) {
    ctx.error_with_code_at(DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT, message, range);
}

fn reject_type_mismatch(ctx: &mut LowerCtx, message: String, range: TextRange) {
    ctx.error_with_code_at(DiagnosticCode::TYPE_MISMATCH, message, range);
}

fn reject_unsupported_surface(ctx: &mut LowerCtx, message: String, range: TextRange) {
    ctx.error_with_code_at(DiagnosticCode::STDLIB_UNSUPPORTED_SURFACE, message, range);
}

fn reject_dynamic_error_handler(ctx: &mut LowerCtx, range: TextRange) {
    ctx.error_with_code_at(
        DiagnosticCode::ENCODING_HANDLER_REQUIRES_STATIC_VALUE,
        "encoding error handlers must be statically known typed values".to_string(),
        range,
    );
}

fn validate_encode_error_handler(arg: &HirExpr, range: TextRange, ctx: &mut LowerCtx) -> bool {
    if arg.ty() != &Type::Str {
        reject_type_mismatch(
            ctx,
            format!(
                "str.encode() errors must be 'str', got '{}'",
                arg.ty().display_name()
            ),
            range,
        );
        return false;
    }
    let HirExpr::StringLiteral(label) = arg else {
        reject_dynamic_error_handler(ctx, range);
        return false;
    };
    if matches!(
        label.as_str(),
        "strict"
            | "replace"
            | "ignore"
            | "backslashreplace"
            | "backslash-replace"
            | "xmlcharrefreplace"
            | "xml-char-ref-replace"
            | "namereplace"
            | "name-replace"
    ) {
        return true;
    }
    reject_dynamic_error_handler(ctx, range);
    false
}

fn validate_decode_error_handler(arg: &HirExpr, range: TextRange, ctx: &mut LowerCtx) -> bool {
    if arg.ty() != &Type::Str {
        reject_type_mismatch(
            ctx,
            format!(
                "bytes.decode() errors must be 'str', got '{}'",
                arg.ty().display_name()
            ),
            range,
        );
        return false;
    }
    let HirExpr::StringLiteral(label) = arg else {
        reject_dynamic_error_handler(ctx, range);
        return false;
    };
    if matches!(
        label.as_str(),
        "strict" | "replace" | "ignore" | "backslashreplace" | "backslash-replace"
    ) {
        return true;
    }
    reject_dynamic_error_handler(ctx, range);
    false
}

pub(in crate::lower) fn resolve_str_encode_method_type(
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    if args.len() > 2 {
        reject_wrong_count(
            ctx,
            format!("str.encode() takes 0 to 2 arguments, got {}", args.len()),
            arity_range(arg_ranges, method_range),
        );
        return None;
    }
    if let Some(encoding) = args.first() {
        if encoding.ty() != &Type::Str {
            reject_type_mismatch(
                ctx,
                format!(
                    "str.encode() encoding must be 'str', got '{}'",
                    encoding.ty().display_name()
                ),
                arg_range(arg_ranges, 0, method_range),
            );
            return None;
        }
    }
    if let Some(errors) = args.get(1) {
        if !validate_encode_error_handler(errors, arg_range(arg_ranges, 1, method_range), ctx) {
            return None;
        }
    }
    Some(Type::Result(
        Box::new(Type::Bytes),
        Box::new(parse_error_type(ctx)),
    ))
}

pub(in crate::lower) fn resolve_bytes_method_type(
    method: &str,
    args: &[HirExpr],
    arg_ranges: &[TextRange],
    method_range: TextRange,
    ctx: &mut LowerCtx,
) -> Option<Type> {
    match method {
        "len" => {
            if !args.is_empty() {
                reject_wrong_count(
                    ctx,
                    "bytes.len() takes no arguments".to_string(),
                    arity_range(arg_ranges, method_range),
                );
                return None;
            }
            Some(Type::Int)
        }
        "count" => {
            if args.len() != 1 {
                reject_wrong_count(
                    ctx,
                    format!("bytes.count() takes exactly 1 argument, got {}", args.len()),
                    arity_range(arg_ranges, method_range),
                );
                return None;
            }
            if args[0].ty() != &Type::Int {
                reject_type_mismatch(
                    ctx,
                    format!(
                        "bytes.count() argument must be 'int', got '{}'",
                        args[0].ty().display_name()
                    ),
                    arg_range(arg_ranges, 0, method_range),
                );
            }
            Some(Type::Int)
        }
        "contains" => {
            if args.len() != 1 {
                reject_wrong_count(
                    ctx,
                    format!(
                        "bytes.contains() takes exactly 1 argument, got {}",
                        args.len()
                    ),
                    arity_range(arg_ranges, method_range),
                );
                return None;
            }
            if args[0].ty() != &Type::Int {
                reject_type_mismatch(
                    ctx,
                    format!(
                        "bytes.contains() argument must be 'int', got '{}'",
                        args[0].ty().display_name()
                    ),
                    arg_range(arg_ranges, 0, method_range),
                );
            }
            Some(Type::Bool)
        }
        "find" => {
            if args.is_empty() || args.len() > 3 {
                reject_wrong_count(
                    ctx,
                    format!("bytes.find() takes 1 to 3 arguments, got {}", args.len()),
                    arity_range(arg_ranges, method_range),
                );
                return None;
            }
            if args[0].ty() != &Type::Int {
                reject_type_mismatch(
                    ctx,
                    format!(
                        "bytes.find() first argument must be 'int', got '{}'",
                        args[0].ty().display_name()
                    ),
                    arg_range(arg_ranges, 0, method_range),
                );
            }
            for (index, bound) in args.iter().enumerate().skip(1) {
                if bound.ty() != &Type::Int {
                    reject_type_mismatch(
                        ctx,
                        format!(
                            "bytes.find() bounds must be 'int', got '{}'",
                            bound.ty().display_name()
                        ),
                        arg_range(arg_ranges, index, method_range),
                    );
                }
            }
            Some(Type::Union(vec![Type::Int, Type::None]))
        }
        "startswith" | "endswith" => {
            if args.len() != 1 {
                reject_wrong_count(
                    ctx,
                    format!(
                        "bytes.{method}() takes exactly 1 argument, got {}",
                        args.len()
                    ),
                    arity_range(arg_ranges, method_range),
                );
                return None;
            }
            if args[0].ty() != &Type::Bytes {
                reject_type_mismatch(
                    ctx,
                    format!(
                        "bytes.{method}() argument must be 'bytes', got '{}'",
                        args[0].ty().display_name()
                    ),
                    arg_range(arg_ranges, 0, method_range),
                );
            }
            Some(Type::Bool)
        }
        "hex" => {
            if !args.is_empty() {
                reject_wrong_count(
                    ctx,
                    "bytes.hex() takes no arguments".to_string(),
                    arity_range(arg_ranges, method_range),
                );
                return None;
            }
            Some(Type::Str)
        }
        "to_ints" => {
            if !args.is_empty() {
                reject_wrong_count(
                    ctx,
                    "bytes.to_ints() takes no arguments".to_string(),
                    arity_range(arg_ranges, method_range),
                );
                return None;
            }
            Some(Type::List(Box::new(Type::Int)))
        }
        "decode" => {
            if args.len() > 2 {
                reject_wrong_count(
                    ctx,
                    format!("bytes.decode() takes 0 to 2 arguments, got {}", args.len()),
                    arity_range(arg_ranges, method_range),
                );
                return None;
            }
            if let Some(encoding) = args.first() {
                if encoding.ty() != &Type::Str {
                    reject_type_mismatch(
                        ctx,
                        format!(
                            "bytes.decode() encoding must be 'str', got '{}'",
                            encoding.ty().display_name()
                        ),
                        arg_range(arg_ranges, 0, method_range),
                    );
                    return None;
                }
            }
            if let Some(errors) = args.get(1) {
                if !validate_decode_error_handler(
                    errors,
                    arg_range(arg_ranges, 1, method_range),
                    ctx,
                ) {
                    return None;
                }
            }
            Some(Type::Result(
                Box::new(Type::Str),
                Box::new(parse_error_type(ctx)),
            ))
        }
        _ => {
            reject_unsupported_surface(
                ctx,
                format!(
                    "bytes has no method '{method}' (supported: len, count, contains, find, startswith, endswith, hex, to_ints, decode)"
                ),
                method_range,
            );
            None
        }
    }
}
