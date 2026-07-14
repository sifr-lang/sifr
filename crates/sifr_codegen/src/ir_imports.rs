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
    pub(crate) needs_sifr_int: bool,
    pub(crate) numeric: IrNumericImportNeeds,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IrNumericImportNeeds {
    pub(crate) needs_bigint: bool,
    pub(crate) needs_decimal: bool,
    pub(crate) needs_bigdecimal: bool,
}

pub(crate) fn collect_import_needs_from_items(items: &[RustItem]) -> IrImportNeeds {
    let mut needs = IrImportNeeds::default();
    for item in items {
        collect_item(item, &mut needs);
    }
    needs
}

pub(crate) fn collect_import_needs_from_source(source: &str) -> IrImportNeeds {
    let mut needs = IrImportNeeds::default();
    if source.trim().is_empty() {
        return needs;
    }
    if let Ok(file) = syn::parse_file(source) {
        let mut collector = SynImportNeedsCollector { needs: &mut needs };
        collector.visit_file(&file);
        return needs;
    }
    scan_named_text(source, &mut needs);
    needs
}

fn collect_item(item: &RustItem, needs: &mut IrImportNeeds) {
    match item {
        RustItem::Use(_) | RustItem::UseAlias { .. } | RustItem::Attr(_) => {}
        RustItem::Struct { fields, .. } => {
            for (_, ty) in fields {
                collect_type(ty, needs);
            }
        }
        RustItem::TupleStruct { inner, .. } => collect_type(inner, needs),
        RustItem::Enum { variants, .. } => {
            for variant in variants {
                for (_, ty) in &variant.fields {
                    collect_type(ty, needs);
                }
                if let Some(expr) = &variant.value {
                    collect_expr(expr, needs);
                }
            }
        }
        RustItem::Trait { methods, .. } | RustItem::Impl { items: methods, .. } => {
            for method in methods {
                collect_item(method, needs);
            }
        }
        RustItem::Fn {
            params, ret, body, ..
        } => {
            for param in params {
                if let RustParam::Named { ty, .. } | RustParam::NamedMut { ty, .. } = param {
                    collect_type(ty, needs);
                }
            }
            if let Some(ret_ty) = ret {
                collect_type(ret_ty, needs);
            }
            for stmt in body {
                collect_stmt(stmt, needs);
            }
        }
        RustItem::TraitMethodSig { params, ret, .. } => {
            for param in params {
                if let RustParam::Named { ty, .. } | RustParam::NamedMut { ty, .. } = param {
                    collect_type(ty, needs);
                }
            }
            if let Some(ret_ty) = ret {
                collect_type(ret_ty, needs);
            }
        }
        RustItem::TypeAlias { ty, .. } => collect_type(ty, needs),
        RustItem::Const { ty, value, .. } | RustItem::Static { ty, value, .. } => {
            collect_type(ty, needs);
            collect_expr(value, needs);
        }
    }
}

fn collect_stmt(stmt: &RustStmt, needs: &mut IrImportNeeds) {
    match stmt {
        RustStmt::Let { ty, value, .. } => {
            if let Some(ty) = ty {
                collect_type(ty, needs);
            }
            collect_expr(value, needs);
        }
        RustStmt::LetPattern { value, .. } => collect_expr(value, needs),
        RustStmt::LetElse {
            value, else_body, ..
        } => {
            collect_expr(value, needs);
            for stmt in else_body {
                collect_stmt(stmt, needs);
            }
        }
        RustStmt::Assign { target, value } | RustStmt::AugAssign { target, value, .. } => {
            collect_expr(target, needs);
            collect_expr(value, needs);
        }
        RustStmt::Expr(expr) | RustStmt::Return(Some(expr)) => collect_expr(expr, needs),
        RustStmt::Assert { cond, msg } => {
            collect_expr(cond, needs);
            if let Some(msg) = msg {
                collect_expr(msg, needs);
            }
        }
        RustStmt::Return(None) | RustStmt::Break | RustStmt::Continue => {}
        RustStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            collect_expr(cond, needs);
            for stmt in then_body {
                collect_stmt(stmt, needs);
            }
            if let Some(else_body) = else_body {
                for stmt in else_body {
                    collect_stmt(stmt, needs);
                }
            }
        }
        RustStmt::IfLet {
            expr,
            then_body,
            else_body,
            ..
        } => {
            collect_expr(expr, needs);
            for stmt in then_body {
                collect_stmt(stmt, needs);
            }
            if let Some(else_body) = else_body {
                for stmt in else_body {
                    collect_stmt(stmt, needs);
                }
            }
        }
        RustStmt::Match { expr, arms } => {
            collect_expr(expr, needs);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_expr(guard, needs);
                }
                for stmt in &arm.body {
                    collect_stmt(stmt, needs);
                }
            }
        }
        RustStmt::For { iter, body, .. } => {
            collect_expr(iter, needs);
            for stmt in body {
                collect_stmt(stmt, needs);
            }
        }
        RustStmt::With { items, body } => {
            for item in items {
                collect_expr(&item.value, needs);
            }
            for stmt in body {
                collect_stmt(stmt, needs);
            }
        }
        RustStmt::While { cond, body } => {
            collect_expr(cond, needs);
            for stmt in body {
                collect_stmt(stmt, needs);
            }
        }
        RustStmt::Loop { body } | RustStmt::Block(body) => {
            for stmt in body {
                collect_stmt(stmt, needs);
            }
        }
        RustStmt::LocalFn {
            params, ret, body, ..
        } => {
            for param in params {
                match param {
                    RustParam::Named { ty, .. } | RustParam::NamedMut { ty, .. } => {
                        collect_type(ty, needs);
                    }
                    RustParam::SelfParam { .. } | RustParam::SelfValue => {}
                }
            }
            if let Some(ret) = ret {
                collect_type(ret, needs);
            }
            for stmt in body {
                collect_stmt(stmt, needs);
            }
        }
    }
}

fn collect_expr(expr: &RustExpr, needs: &mut IrImportNeeds) {
    match expr {
        RustExpr::Literal(_) => {}
        RustExpr::Ident(name) => mark_symbol(name, needs),
        RustExpr::Path(segments) => {
            if let Some(first) = segments.first() {
                mark_symbol(first, needs);
            }
        }
        RustExpr::MethodCall { receiver, args, .. } => {
            collect_expr(receiver, needs);
            for arg in args {
                collect_expr(arg, needs);
            }
        }
        RustExpr::FnCall { func, args } => {
            collect_expr(func, needs);
            for arg in args {
                collect_expr(arg, needs);
            }
        }
        RustExpr::MacroCall { args, .. }
        | RustExpr::Vec(args)
        | RustExpr::Tuple(args)
        | RustExpr::Array(args) => {
            for arg in args {
                collect_expr(arg, needs);
            }
        }
        RustExpr::TimeoutAwait { duration, future } => {
            collect_expr(duration, needs);
            collect_expr(future, needs);
        }
        RustExpr::FormatMacro { args, .. } => {
            for arg in args {
                collect_expr(arg, needs);
            }
        }
        RustExpr::BinOp { left, right, .. } => {
            collect_expr(left, needs);
            collect_expr(right, needs);
        }
        RustExpr::UnaryOp { operand, .. }
        | RustExpr::Deref(operand)
        | RustExpr::Clone(operand)
        | RustExpr::Try(operand)
        | RustExpr::Paren(operand)
        | RustExpr::Await(operand) => collect_expr(operand, needs),
        RustExpr::Field { expr, .. } => collect_expr(expr, needs),
        RustExpr::Index { expr, index } => {
            collect_expr(expr, needs);
            collect_expr(index, needs);
        }
        RustExpr::Slice { expr, start, stop } => {
            collect_expr(expr, needs);
            if let Some(start) = start {
                collect_expr(start, needs);
            }
            if let Some(stop) = stop {
                collect_expr(stop, needs);
            }
        }
        RustExpr::Ref { expr, .. } => collect_expr(expr, needs),
        RustExpr::Cast { expr, ty } => {
            collect_expr(expr, needs);
            collect_type(ty, needs);
        }
        RustExpr::Block { stmts, expr } => {
            for stmt in stmts {
                collect_stmt(stmt, needs);
            }
            if let Some(expr) = expr {
                collect_expr(expr, needs);
            }
        }
        RustExpr::If {
            cond,
            then_expr,
            else_expr,
        } => {
            collect_expr(cond, needs);
            collect_expr(then_expr, needs);
            if let Some(else_expr) = else_expr {
                collect_expr(else_expr, needs);
            }
        }
        RustExpr::Match { expr, arms } => {
            collect_expr(expr, needs);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_expr(guard, needs);
                }
                for stmt in &arm.body {
                    collect_stmt(stmt, needs);
                }
            }
        }
        RustExpr::Closure { body, .. } => collect_expr(body, needs),
        RustExpr::ClosureBlock { body, .. } | RustExpr::AsyncBlock { body, .. } => {
            for stmt in body {
                collect_stmt(stmt, needs);
            }
        }
        RustExpr::StructInit { fields, .. } => {
            for (_, value) in fields {
                collect_expr(value, needs);
            }
        }
        RustExpr::Range { start, end } => {
            collect_expr(start, needs);
            collect_expr(end, needs);
        }
    }
}

fn collect_type(ty: &RustType, needs: &mut IrImportNeeds) {
    match ty {
        RustType::I64 | RustType::F64 | RustType::Bool | RustType::String_ | RustType::Unit => {}
        RustType::Vec(inner)
        | RustType::HashSet(inner)
        | RustType::VecDeque(inner)
        | RustType::Option(inner) => {
            collect_type(inner, needs);
        }
        RustType::HashMap(k, v) | RustType::Result(k, v) => {
            collect_type(k, needs);
            collect_type(v, needs);
        }
        RustType::Tuple(items) => {
            for item in items {
                collect_type(item, needs);
            }
        }
        RustType::Ref { inner, .. } => collect_type(inner, needs),
        RustType::Named(name) => collect_from_type_text(name, needs),
        RustType::DynTrait(name) => collect_from_type_text(&format!("dyn {name}"), needs),
        RustType::Impl(name) => collect_from_type_text(&format!("impl {name}"), needs),
        RustType::Generic { base, params } => {
            mark_symbol(base, needs);
            for param in params {
                collect_type(param, needs);
            }
        }
        RustType::Fn { params, ret } => {
            for param in params {
                collect_type(param, needs);
            }
            collect_type(ret, needs);
        }
    }
}

fn collect_from_type_text(text: &str, needs: &mut IrImportNeeds) {
    if let Ok(ty) = syn::parse_str::<syn::Type>(text) {
        collect_from_syn_type(&ty, needs);
        return;
    }
    scan_named_text(text, needs);
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

fn mark_symbol(symbol: &str, needs: &mut IrImportNeeds) {
    match symbol {
        "HashMap" => needs.collections.needs_hashmap = true,
        "HashSet" => needs.collections.needs_hashset = true,
        "VecDeque" => needs.collections.needs_vecdeque = true,
        "Mutex" => needs.runtime.needs_mutex = true,
        "SifrInt" => needs.runtime.needs_sifr_int = true,
        "sifr_runtime" => needs.runtime.needs_sifr_int = true,
        "BigInt" => needs.runtime.numeric.needs_bigint = true,
        "Decimal" => needs.runtime.numeric.needs_decimal = true,
        "BigDecimal" => needs.runtime.numeric.needs_bigdecimal = true,
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
        assert!(needs.runtime.numeric.needs_bigint);
        assert!(!needs.runtime.numeric.needs_decimal);
        assert!(!needs.runtime.numeric.needs_bigdecimal);
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
        assert!(!needs.runtime.numeric.needs_bigint);
        assert!(!needs.runtime.numeric.needs_decimal);
        assert!(!needs.runtime.numeric.needs_bigdecimal);
    }

    #[test]
    fn collects_sifr_runtime_integer_symbols() {
        let items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![RustParam::Named {
                name: "value".to_string(),
                ty: RustType::Named("SifrInt".to_string()),
            }],
            ret: Some(RustType::Named("SifrInt".to_string())),
            body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "sifr_runtime".to_string(),
                    "SifrInt".to_string(),
                    "from_i64".to_string(),
                ])),
                args: vec![RustExpr::Literal(RustLiteral::Int(1))],
            }))],
            is_async: false,
        }];

        let needs = collect_import_needs_from_items(&items);

        assert!(needs.runtime.needs_sifr_int);
    }
}
