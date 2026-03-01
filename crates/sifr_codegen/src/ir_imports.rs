use crate::{RustExpr, RustItem, RustParam, RustStmt, RustType};
use syn::visit::{self, Visit};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IrImportNeeds {
    pub(crate) collections: IrCollectionImportNeeds,
    pub(crate) runtime: IrRuntimeImportNeeds,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IrCollectionImportNeeds {
    pub(crate) needs_hashmap: bool,
    pub(crate) needs_hashset: bool,
    pub(crate) needs_vecdeque: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IrRuntimeImportNeeds {
    pub(crate) needs_mutex: bool,
    pub(crate) needs_bigint: bool,
}

pub(crate) fn collect_import_needs_from_items(items: &[RustItem]) -> IrImportNeeds {
    collect_import_needs_from_items_with_raw_mode(items, false)
}

#[cfg(test)]
pub(crate) fn collect_import_needs_from_items_allow_raw(items: &[RustItem]) -> IrImportNeeds {
    collect_import_needs_from_items_with_raw_mode(items, true)
}

fn collect_import_needs_from_items_with_raw_mode(
    items: &[RustItem],
    allow_raw: bool,
) -> IrImportNeeds {
    let mut needs = IrImportNeeds::default();
    for item in items {
        collect_item(item, &mut needs, allow_raw);
    }
    needs
}

fn collect_item(item: &RustItem, needs: &mut IrImportNeeds, allow_raw: bool) {
    match item {
        RustItem::Use(_) | RustItem::UseAlias { .. } | RustItem::Attr(_) => {}
        RustItem::SynItem(code) => collect_from_syn_item_code(code, needs),
        RustItem::RawCode(code) => {
            if allow_raw {
                collect_from_raw_item_code(code, needs);
            } else {
                raw_import_in_production_forbidden("item RawCode");
            }
        }
        RustItem::Struct { fields, .. } => {
            for (_, ty) in fields {
                collect_type(ty, needs, allow_raw);
            }
        }
        RustItem::TupleStruct { inner, .. } => collect_type(inner, needs, allow_raw),
        RustItem::Enum { variants, .. } => {
            for variant in variants {
                for (_, ty) in &variant.fields {
                    collect_type(ty, needs, allow_raw);
                }
                if let Some(expr) = &variant.value {
                    collect_expr(expr, needs, allow_raw);
                }
            }
        }
        RustItem::Trait { methods, .. } | RustItem::Impl { items: methods, .. } => {
            for method in methods {
                collect_item(method, needs, allow_raw);
            }
        }
        RustItem::Fn {
            params, ret, body, ..
        } => {
            for param in params {
                if let RustParam::Named { ty, .. } = param {
                    collect_type(ty, needs, allow_raw);
                }
            }
            if let Some(ret_ty) = ret {
                collect_type(ret_ty, needs, allow_raw);
            }
            for stmt in body {
                collect_stmt(stmt, needs, allow_raw);
            }
        }
        RustItem::TraitMethodSig { params, ret, .. } => {
            for param in params {
                if let RustParam::Named { ty, .. } = param {
                    collect_type(ty, needs, allow_raw);
                }
            }
            if let Some(ret_ty) = ret {
                collect_type(ret_ty, needs, allow_raw);
            }
        }
        RustItem::TypeAlias { ty, .. } => collect_type(ty, needs, allow_raw),
        RustItem::Const { ty, value, .. } | RustItem::Static { ty, value, .. } => {
            collect_type(ty, needs, allow_raw);
            collect_expr(value, needs, allow_raw);
        }
    }
}

fn collect_stmt(stmt: &RustStmt, needs: &mut IrImportNeeds, allow_raw: bool) {
    match stmt {
        RustStmt::Let { ty, value, .. } => {
            if let Some(ty) = ty {
                collect_type(ty, needs, allow_raw);
            }
            collect_expr(value, needs, allow_raw);
        }
        RustStmt::LetPattern { value, .. } => collect_expr(value, needs, allow_raw),
        RustStmt::Assign { target, value } | RustStmt::AugAssign { target, value, .. } => {
            collect_expr(target, needs, allow_raw);
            collect_expr(value, needs, allow_raw);
        }
        RustStmt::Expr(expr) | RustStmt::Return(Some(expr)) => collect_expr(expr, needs, allow_raw),
        RustStmt::Assert { cond, msg } => {
            collect_expr(cond, needs, allow_raw);
            if let Some(msg) = msg {
                collect_expr(msg, needs, allow_raw);
            }
        }
        RustStmt::RawCode(code) => {
            if allow_raw {
                collect_from_raw_stmt_code(code, needs);
            } else {
                raw_import_in_production_forbidden("statement RawCode");
            }
        }
        RustStmt::Return(None) | RustStmt::Break | RustStmt::Continue => {}
        RustStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            collect_expr(cond, needs, allow_raw);
            for stmt in then_body {
                collect_stmt(stmt, needs, allow_raw);
            }
            if let Some(else_body) = else_body {
                for stmt in else_body {
                    collect_stmt(stmt, needs, allow_raw);
                }
            }
        }
        RustStmt::IfLet {
            expr,
            then_body,
            else_body,
            ..
        } => {
            collect_expr(expr, needs, allow_raw);
            for stmt in then_body {
                collect_stmt(stmt, needs, allow_raw);
            }
            if let Some(else_body) = else_body {
                for stmt in else_body {
                    collect_stmt(stmt, needs, allow_raw);
                }
            }
        }
        RustStmt::Match { expr, arms } => {
            collect_expr(expr, needs, allow_raw);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_expr(guard, needs, allow_raw);
                }
                for stmt in &arm.body {
                    collect_stmt(stmt, needs, allow_raw);
                }
            }
        }
        RustStmt::For { iter, body, .. } => {
            collect_expr(iter, needs, allow_raw);
            for stmt in body {
                collect_stmt(stmt, needs, allow_raw);
            }
        }
        RustStmt::With { items, body } => {
            for item in items {
                collect_expr(&item.value, needs, allow_raw);
            }
            for stmt in body {
                collect_stmt(stmt, needs, allow_raw);
            }
        }
        RustStmt::While { cond, body } => {
            collect_expr(cond, needs, allow_raw);
            for stmt in body {
                collect_stmt(stmt, needs, allow_raw);
            }
        }
        RustStmt::Loop { body } | RustStmt::Block(body) => {
            for stmt in body {
                collect_stmt(stmt, needs, allow_raw);
            }
        }
        RustStmt::LocalFn {
            params, ret, body, ..
        } => {
            for param in params {
                match param {
                    RustParam::Named { ty, .. } => collect_type(ty, needs, allow_raw),
                    RustParam::SelfParam { .. } | RustParam::SelfValue => {}
                }
            }
            if let Some(ret) = ret {
                collect_type(ret, needs, allow_raw);
            }
            for stmt in body {
                collect_stmt(stmt, needs, allow_raw);
            }
        }
    }
}

fn collect_expr(expr: &RustExpr, needs: &mut IrImportNeeds, allow_raw: bool) {
    match expr {
        RustExpr::Literal(_) => {}
        RustExpr::RawCode(code) => {
            if allow_raw {
                collect_from_raw_expr_code(code, needs);
            } else {
                raw_import_in_production_forbidden("expression RawCode");
            }
        }
        RustExpr::Ident(name) => mark_symbol(name, needs),
        RustExpr::Path(segments) => {
            if let Some(first) = segments.first() {
                mark_symbol(first, needs);
            }
        }
        RustExpr::MethodCall { receiver, args, .. } => {
            collect_expr(receiver, needs, allow_raw);
            for arg in args {
                collect_expr(arg, needs, allow_raw);
            }
        }
        RustExpr::FnCall { func, args } => {
            collect_expr(func, needs, allow_raw);
            for arg in args {
                collect_expr(arg, needs, allow_raw);
            }
        }
        RustExpr::MacroCall { args, .. } | RustExpr::Vec(args) | RustExpr::Tuple(args) => {
            for arg in args {
                collect_expr(arg, needs, allow_raw);
            }
        }
        RustExpr::FormatMacro { args, .. } => {
            for arg in args {
                collect_expr(arg, needs, allow_raw);
            }
        }
        RustExpr::BinOp { left, right, .. } => {
            collect_expr(left, needs, allow_raw);
            collect_expr(right, needs, allow_raw);
        }
        RustExpr::UnaryOp { operand, .. }
        | RustExpr::Deref(operand)
        | RustExpr::Clone(operand)
        | RustExpr::Try(operand)
        | RustExpr::Paren(operand)
        | RustExpr::Await(operand) => collect_expr(operand, needs, allow_raw),
        RustExpr::Field { expr, .. } => collect_expr(expr, needs, allow_raw),
        RustExpr::Index { expr, index } => {
            collect_expr(expr, needs, allow_raw);
            collect_expr(index, needs, allow_raw);
        }
        RustExpr::Slice { expr, start, stop } => {
            collect_expr(expr, needs, allow_raw);
            if let Some(start) = start {
                collect_expr(start, needs, allow_raw);
            }
            if let Some(stop) = stop {
                collect_expr(stop, needs, allow_raw);
            }
        }
        RustExpr::Ref { expr, .. } => collect_expr(expr, needs, allow_raw),
        RustExpr::Cast { expr, ty } => {
            collect_expr(expr, needs, allow_raw);
            collect_type(ty, needs, allow_raw);
        }
        RustExpr::Block { stmts, expr } => {
            for stmt in stmts {
                collect_stmt(stmt, needs, allow_raw);
            }
            if let Some(expr) = expr {
                collect_expr(expr, needs, allow_raw);
            }
        }
        RustExpr::If {
            cond,
            then_expr,
            else_expr,
        } => {
            collect_expr(cond, needs, allow_raw);
            collect_expr(then_expr, needs, allow_raw);
            if let Some(else_expr) = else_expr {
                collect_expr(else_expr, needs, allow_raw);
            }
        }
        RustExpr::Match { expr, arms } => {
            collect_expr(expr, needs, allow_raw);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_expr(guard, needs, allow_raw);
                }
                for stmt in &arm.body {
                    collect_stmt(stmt, needs, allow_raw);
                }
            }
        }
        RustExpr::Closure { body, .. } => collect_expr(body, needs, allow_raw),
        RustExpr::ClosureBlock { body, .. } => {
            for stmt in body {
                collect_stmt(stmt, needs, allow_raw);
            }
        }
        RustExpr::StructInit { fields, .. } => {
            for (_, value) in fields {
                collect_expr(value, needs, allow_raw);
            }
        }
        RustExpr::Range { start, end } => {
            collect_expr(start, needs, allow_raw);
            collect_expr(end, needs, allow_raw);
        }
    }
}

fn collect_type(ty: &RustType, needs: &mut IrImportNeeds, allow_raw: bool) {
    match ty {
        RustType::I64 | RustType::F64 | RustType::Bool | RustType::String_ | RustType::Unit => {}
        RustType::RawCode(code) => {
            if allow_raw {
                collect_from_raw_type_code(code, needs);
            } else {
                raw_import_in_production_forbidden("type RawCode");
            }
        }
        RustType::Vec(inner)
        | RustType::HashSet(inner)
        | RustType::VecDeque(inner)
        | RustType::Option(inner) => {
            collect_type(inner, needs, allow_raw);
        }
        RustType::HashMap(k, v) | RustType::Result(k, v) => {
            collect_type(k, needs, allow_raw);
            collect_type(v, needs, allow_raw);
        }
        RustType::Tuple(items) => {
            for item in items {
                collect_type(item, needs, allow_raw);
            }
        }
        RustType::Ref { inner, .. } => collect_type(inner, needs, allow_raw),
        RustType::Named(name) => collect_from_type_text(name, needs),
        RustType::DynTrait(name) => collect_from_type_text(&format!("dyn {name}"), needs),
        RustType::Impl(name) => collect_from_type_text(&format!("impl {name}"), needs),
        RustType::Generic { base, params } => {
            mark_symbol(base, needs);
            for param in params {
                collect_type(param, needs, allow_raw);
            }
        }
        RustType::Fn { params, ret } => {
            for param in params {
                collect_type(param, needs, allow_raw);
            }
            collect_type(ret, needs, allow_raw);
        }
    }
}

fn collect_from_raw_item_code(code: &str, needs: &mut IrImportNeeds) {
    if let Ok(file) = syn::parse_file(code) {
        collect_from_syn_file(&file, needs);
        return;
    }
    scan_named_text(code, needs);
}

fn collect_from_syn_item_code(code: &str, needs: &mut IrImportNeeds) {
    let item = syn::parse_str::<syn::Item>(code).unwrap_or_else(|err| {
        panic!("invalid syn-backed item in structural import pass: {err}; code:\n{code}")
    });
    collect_from_syn_item(&item, needs);
}

fn collect_from_raw_stmt_code(code: &str, needs: &mut IrImportNeeds) {
    if let Ok(stmt) = syn::parse_str::<syn::Stmt>(code) {
        collect_from_syn_stmt(&stmt, needs);
        return;
    }
    scan_named_text(code, needs);
}

fn collect_from_raw_expr_code(code: &str, needs: &mut IrImportNeeds) {
    if let Ok(expr) = syn::parse_str::<syn::Expr>(code) {
        collect_from_syn_expr(&expr, needs);
        return;
    }
    scan_named_text(code, needs);
}

fn collect_from_raw_type_code(code: &str, needs: &mut IrImportNeeds) {
    if let Ok(ty) = syn::parse_str::<syn::Type>(code) {
        collect_from_syn_type(&ty, needs);
        return;
    }
    scan_named_text(code, needs);
}

fn collect_from_type_text(text: &str, needs: &mut IrImportNeeds) {
    if let Ok(ty) = syn::parse_str::<syn::Type>(text) {
        collect_from_syn_type(&ty, needs);
        return;
    }
    scan_named_text(text, needs);
}

fn collect_from_syn_file(file: &syn::File, needs: &mut IrImportNeeds) {
    let mut collector = SynImportNeedsCollector { needs };
    collector.visit_file(file);
}

fn collect_from_syn_item(item: &syn::Item, needs: &mut IrImportNeeds) {
    let mut collector = SynImportNeedsCollector { needs };
    collector.visit_item(item);
}

fn collect_from_syn_stmt(stmt: &syn::Stmt, needs: &mut IrImportNeeds) {
    let mut collector = SynImportNeedsCollector { needs };
    collector.visit_stmt(stmt);
}

fn collect_from_syn_expr(expr: &syn::Expr, needs: &mut IrImportNeeds) {
    let mut collector = SynImportNeedsCollector { needs };
    collector.visit_expr(expr);
}

fn collect_from_syn_type(ty: &syn::Type, needs: &mut IrImportNeeds) {
    let mut collector = SynImportNeedsCollector { needs };
    collector.visit_type(ty);
}

struct SynImportNeedsCollector<'a> {
    needs: &'a mut IrImportNeeds,
}

impl Visit<'_> for SynImportNeedsCollector<'_> {
    fn visit_path(&mut self, path: &syn::Path) {
        if path.leading_colon.is_none() {
            if let Some(first) = path.segments.first() {
                mark_symbol(&first.ident.to_string(), self.needs);
            }
        }
        visit::visit_path(self, path);
    }
}

fn scan_named_text(text: &str, needs: &mut IrImportNeeds) {
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch == '_' || ch.is_ascii_alphabetic() {
            let start = i;
            i += 1;
            while i < bytes.len() {
                let c = bytes[i] as char;
                if c == '_' || c.is_ascii_alphanumeric() {
                    i += 1;
                } else {
                    break;
                }
            }
            let ident = &text[start..i];
            let is_qualified = start >= 2 && &text[start - 2..start] == "::";
            if !is_qualified {
                mark_symbol(ident, needs);
            }
            continue;
        }
        i += 1;
    }
}

fn raw_import_in_production_forbidden(context: &str) -> ! {
    panic!("RawCode is forbidden in production structural import pass ({context})")
}

fn mark_symbol(symbol: &str, needs: &mut IrImportNeeds) {
    match symbol {
        "HashMap" => needs.collections.needs_hashmap = true,
        "HashSet" => needs.collections.needs_hashset = true,
        "VecDeque" => needs.collections.needs_vecdeque = true,
        "Mutex" => needs.runtime.needs_mutex = true,
        "BigInt" => needs.runtime.needs_bigint = true,
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RustLiteral, Visibility};

    #[test]
    fn collects_unqualified_import_symbols() {
        let items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![RustParam::Named {
                name: "m".to_string(),
                ty: RustType::Named("HashMap<String, i64>".to_string()),
            }],
            ret: Some(RustType::Named("BigInt".to_string())),
            body: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "s".to_string(),
                    ty: Some(RustType::Named("HashSet<String>".to_string())),
                    value: RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "HashSet".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![],
                    },
                },
                RustStmt::Expr(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "VecDeque".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                }),
                RustStmt::Expr(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Mutex".to_string(), "new".to_string()])),
                    args: vec![RustExpr::Literal(RustLiteral::Int(1))],
                }),
            ],
            is_async: false,
        }];
        let needs = collect_import_needs_from_items(&items);
        assert!(needs.collections.needs_hashmap);
        assert!(needs.collections.needs_hashset);
        assert!(needs.collections.needs_vecdeque);
        assert!(needs.runtime.needs_mutex);
        assert!(needs.runtime.needs_bigint);
    }

    #[test]
    fn ignores_fully_qualified_symbols() {
        let items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![RustParam::Named {
                name: "m".to_string(),
                ty: RustType::Named("std::collections::HashMap<String, i64>".to_string()),
            }],
            ret: Some(RustType::Named("num_bigint::BigInt".to_string())),
            body: vec![RustStmt::Expr(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "sync".to_string(),
                    "Mutex".to_string(),
                    "new".to_string(),
                ])),
                args: vec![RustExpr::Literal(RustLiteral::Int(1))],
            })],
            is_async: false,
        }];
        let needs = collect_import_needs_from_items(&items);
        assert!(!needs.collections.needs_hashmap);
        assert!(!needs.collections.needs_hashset);
        assert!(!needs.collections.needs_vecdeque);
        assert!(!needs.runtime.needs_mutex);
        assert!(!needs.runtime.needs_bigint);
    }

    #[test]
    fn collects_symbols_from_raw_code_items() {
        let items = vec![RustItem::RawCode(
            "fn demo(m: HashMap<String, i64>) -> BigInt { \
             let _s: HashSet<String> = HashSet::new(); \
             let _q: VecDeque<i64> = VecDeque::new(); \
             let _m = Mutex::new(1); \
             BigInt::from(1) \
            }"
            .to_string(),
        )];
        let needs = collect_import_needs_from_items_allow_raw(&items);
        assert!(needs.collections.needs_hashmap);
        assert!(needs.collections.needs_hashset);
        assert!(needs.collections.needs_vecdeque);
        assert!(needs.runtime.needs_mutex);
        assert!(needs.runtime.needs_bigint);
    }

    #[test]
    fn ignores_fully_qualified_symbols_in_raw_code() {
        let items = vec![RustItem::RawCode(
            "fn demo() { let _ = std::collections::HashMap::<String, i64>::new(); \
             let _ = num_bigint::BigInt::from(1); \
             let _ = std::sync::Mutex::new(1); }"
                .to_string(),
        )];
        let needs = collect_import_needs_from_items_allow_raw(&items);
        assert!(!needs.collections.needs_hashmap);
        assert!(!needs.collections.needs_hashset);
        assert!(!needs.collections.needs_vecdeque);
        assert!(!needs.runtime.needs_mutex);
        assert!(!needs.runtime.needs_bigint);
    }

    #[test]
    #[should_panic(expected = "RawCode is forbidden in production structural import pass")]
    fn production_mode_panics_on_raw_item() {
        let _ = collect_import_needs_from_items(&[RustItem::RawCode("fn demo() {}".to_string())]);
    }
}
