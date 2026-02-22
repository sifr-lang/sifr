//! TOML intrinsic lowerers for registry migration.

use crate::RustExpr;

fn borrowed_str(expr: &str) -> String {
    format!("&({expr})")
}

pub(super) fn lower_toml_parse(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "{{ let __toml_str = {}; __toml_str.parse::<toml::Value>().map(|v| format!(\"{{}}\", v)).map_err(|e| TOMLDecodeError {{ message: e.to_string(), line: 0, column: 0 }}) }}",
        borrowed_str(&args[0])
    )))
}
