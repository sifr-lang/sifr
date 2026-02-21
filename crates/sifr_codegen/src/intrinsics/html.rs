//! HTML intrinsic lowerers for registry migration.

use crate::RustExpr;

fn borrowed_str(expr: &str) -> String {
    format!("&({expr})")
}

pub(super) fn lower_html_escape(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __s = {}; __s.replace('&', \"&amp;\").replace('<', \"&lt;\").replace('>', \"&gt;\").replace('\"', \"&quot;\").replace('\\'', \"&#x27;\") }}",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_html_unescape(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __s = {}; __s.replace(\"&amp;\", \"&\").replace(\"&lt;\", \"<\").replace(\"&gt;\", \">\").replace(\"&quot;\", \"\\\"\").replace(\"&#x27;\", \"'\").replace(\"&#39;\", \"'\") }}",
        borrowed_str(&args[0])
    )))
}
