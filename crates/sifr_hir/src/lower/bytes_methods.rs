use super::LowerCtx;
use crate::hir_nodes::HirExpr;
use sifr_type_system::Type;

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
        _ => {
            ctx.error(format!("bytes has no method '{method}'"));
            None
        }
    }
}
