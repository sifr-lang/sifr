use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use sifr_type_system::Type;

fn parse_error_type(ctx: &LowerCtx) -> Type {
    ctx.class_types
        .get("ParseError")
        .cloned()
        .unwrap_or(Type::Class {
            name: "ParseError".to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: vec![],
            parent_class: Some("Error".to_string()),
        })
}

pub(super) fn resolve_str_encode_method_type(args: &[HirExpr], ctx: &mut LowerCtx) -> Option<Type> {
    if args.len() > 1 {
        ctx.error(format!(
            "str.encode() takes 0 or 1 argument, got {}",
            args.len()
        ));
        return None;
    }
    if let Some(encoding) = args.first() {
        if encoding.ty() != &Type::Str {
            ctx.error(format!(
                "str.encode() encoding must be 'str', got '{}'",
                encoding.ty().display_name()
            ));
            return None;
        }
        if let HirExpr::StringLiteral(value) = encoding {
            if !matches!(value.to_ascii_lowercase().as_str(), "utf-8" | "utf8") {
                ctx.error("str.encode() currently supports only UTF-8".to_string());
                return None;
            }
        }
    }
    Some(Type::Result(
        Box::new(Type::Bytes),
        Box::new(parse_error_type(ctx)),
    ))
}

pub(super) fn resolve_bytes_method_type(
    method: &str,
    args: &[HirExpr],
    ctx: &mut LowerCtx,
) -> Option<Type> {
    match method {
        "len" => {
            if !args.is_empty() {
                ctx.error("bytes.len() takes no arguments".to_string());
                return None;
            }
            Some(Type::Int)
        }
        "count" => {
            if args.len() != 1 {
                ctx.error(format!(
                    "bytes.count() takes exactly 1 argument, got {}",
                    args.len()
                ));
                return None;
            }
            if args[0].ty() != &Type::Int {
                ctx.error(format!(
                    "bytes.count() argument must be 'int', got '{}'",
                    args[0].ty().display_name()
                ));
            }
            Some(Type::Int)
        }
        "contains" => {
            if args.len() != 1 {
                ctx.error(format!(
                    "bytes.contains() takes exactly 1 argument, got {}",
                    args.len()
                ));
                return None;
            }
            if args[0].ty() != &Type::Int {
                ctx.error(format!(
                    "bytes.contains() argument must be 'int', got '{}'",
                    args[0].ty().display_name()
                ));
            }
            Some(Type::Bool)
        }
        "index" => {
            if args.is_empty() || args.len() > 3 {
                ctx.error(format!(
                    "bytes.index() takes 1 to 3 arguments, got {}",
                    args.len()
                ));
                return None;
            }
            if args[0].ty() != &Type::Int {
                ctx.error(format!(
                    "bytes.index() first argument must be 'int', got '{}'",
                    args[0].ty().display_name()
                ));
            }
            for bound in args.iter().skip(1) {
                if bound.ty() != &Type::Int {
                    ctx.error(format!(
                        "bytes.index() bounds must be 'int', got '{}'",
                        bound.ty().display_name()
                    ));
                }
            }
            Some(Type::Union(vec![Type::Int, Type::None]))
        }
        "to_ints" => {
            if !args.is_empty() {
                ctx.error("bytes.to_ints() takes no arguments".to_string());
                return None;
            }
            Some(Type::List(Box::new(Type::Int)))
        }
        "decode" => {
            if args.len() > 1 {
                ctx.error(format!(
                    "bytes.decode() takes 0 or 1 argument, got {}",
                    args.len()
                ));
                return None;
            }
            if let Some(encoding) = args.first() {
                if encoding.ty() != &Type::Str {
                    ctx.error(format!(
                        "bytes.decode() encoding must be 'str', got '{}'",
                        encoding.ty().display_name()
                    ));
                    return None;
                }
                if let HirExpr::StringLiteral(value) = encoding {
                    if !matches!(value.to_ascii_lowercase().as_str(), "utf-8" | "utf8") {
                        ctx.error("bytes.decode() currently supports only UTF-8".to_string());
                        return None;
                    }
                }
            }
            Some(Type::Result(
                Box::new(Type::Str),
                Box::new(parse_error_type(ctx)),
            ))
        }
        _ => {
            ctx.error(format!(
                "bytes has no method '{method}' (supported: len, count, contains, index, to_ints, decode)"
            ));
            None
        }
    }
}
