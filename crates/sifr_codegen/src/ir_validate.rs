use crate::{RustExpr, RustItem, RustParam, RustStmt, RustType};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IrValidationKind {
    DuplicateStructField,
    EmptyFunctionBody,
    ReturnOutsideFunction,
    InvalidVerbatimStatement,
    InvalidVerbatimExpression,
    InvalidIdentifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IrValidationIssue {
    pub(crate) kind: IrValidationKind,
    pub(crate) message: String,
}

pub(crate) fn validate_items(items: &[RustItem]) -> Vec<IrValidationIssue> {
    let mut issues = Vec::new();
    for item in items {
        validate_item(item, &mut issues);
    }
    issues
}

fn validate_item(item: &RustItem, issues: &mut Vec<IrValidationIssue>) {
    match item {
        RustItem::Use(_) | RustItem::UseAlias { .. } | RustItem::Attr(_) => {}
        RustItem::Struct { name, fields, .. } => {
            let mut seen = HashSet::new();
            for (field_name, field_ty) in fields {
                if !seen.insert(field_name.as_str()) {
                    issues.push(IrValidationIssue {
                        kind: IrValidationKind::DuplicateStructField,
                        message: format!("struct `{name}` has duplicate field `{field_name}`"),
                    });
                }
                validate_type(field_ty, issues);
            }
        }
        RustItem::TupleStruct { inner, .. } => validate_type(inner, issues),
        RustItem::Enum { variants, .. } => {
            for variant in variants {
                for (_, field_ty) in &variant.fields {
                    validate_type(field_ty, issues);
                }
                if let Some(value) = &variant.value {
                    validate_expr(value, issues, false);
                }
            }
        }
        RustItem::Trait { methods, .. } | RustItem::Impl { items: methods, .. } => {
            for method in methods {
                validate_item(method, issues);
            }
        }
        RustItem::Fn {
            name,
            params,
            ret,
            body,
            ..
        } => {
            if body.is_empty() {
                issues.push(IrValidationIssue {
                    kind: IrValidationKind::EmptyFunctionBody,
                    message: format!("function `{name}` has an empty body"),
                });
            }
            for param in params {
                if let crate::RustParam::Named { ty, .. } | crate::RustParam::NamedMut { ty, .. } =
                    param
                {
                    validate_type(ty, issues);
                }
            }
            if let Some(ret_ty) = ret {
                validate_type(ret_ty, issues);
            }
            for stmt in body {
                validate_stmt(stmt, issues, true);
            }
        }
        RustItem::TraitMethodSig { params, ret, .. } => {
            for param in params {
                if let crate::RustParam::Named { ty, .. } | crate::RustParam::NamedMut { ty, .. } =
                    param
                {
                    validate_type(ty, issues);
                }
            }
            if let Some(ret_ty) = ret {
                validate_type(ret_ty, issues);
            }
        }
        RustItem::TypeAlias { ty, .. } => validate_type(ty, issues),
        RustItem::Const { ty, value, .. } | RustItem::Static { ty, value, .. } => {
            validate_type(ty, issues);
            validate_expr(value, issues, false);
        }
    }
}

fn validate_stmt(stmt: &RustStmt, issues: &mut Vec<IrValidationIssue>, in_function: bool) {
    match stmt {
        RustStmt::Verbatim(source) => {
            let wrapper = format!("async fn __sifr_validate_verbatim() {{ {source} }}");
            if let Err(error) = syn::parse_file(&wrapper) {
                issues.push(IrValidationIssue {
                    kind: IrValidationKind::InvalidVerbatimStatement,
                    message: format!(
                        "compiler-owned verbatim Rust statement is invalid ({error}): {source}"
                    ),
                });
            }
        }
        RustStmt::Let { ty, value, .. } => {
            if let Some(ty) = ty {
                validate_type(ty, issues);
            }
            validate_expr(value, issues, in_function);
        }
        RustStmt::LetDecl { ty, .. } => validate_type(ty, issues),
        RustStmt::LetPattern { value, .. } => {
            validate_expr(value, issues, in_function);
        }
        RustStmt::LetElse {
            value, else_body, ..
        } => {
            validate_expr(value, issues, in_function);
            for stmt in else_body {
                validate_stmt(stmt, issues, in_function);
            }
        }
        RustStmt::Assign { target, value } | RustStmt::AugAssign { target, value, .. } => {
            validate_expr(target, issues, in_function);
            validate_expr(value, issues, in_function);
        }
        RustStmt::Expr(expr) | RustStmt::TailExpr(expr) => {
            validate_expr(expr, issues, in_function);
        }
        RustStmt::Assert { cond, msg } => {
            validate_expr(cond, issues, in_function);
            if let Some(msg) = msg {
                validate_expr(msg, issues, in_function);
            }
        }
        RustStmt::Return(expr) => {
            if !in_function {
                issues.push(IrValidationIssue {
                    kind: IrValidationKind::ReturnOutsideFunction,
                    message: "`return` appears outside a function body".to_string(),
                });
            }
            if let Some(expr) = expr {
                validate_expr(expr, issues, in_function);
            }
        }
        RustStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            validate_expr(cond, issues, in_function);
            for stmt in then_body {
                validate_stmt(stmt, issues, in_function);
            }
            if let Some(else_body) = else_body {
                for stmt in else_body {
                    validate_stmt(stmt, issues, in_function);
                }
            }
        }
        RustStmt::IfLet {
            expr,
            then_body,
            else_body,
            ..
        } => {
            validate_expr(expr, issues, in_function);
            for stmt in then_body {
                validate_stmt(stmt, issues, in_function);
            }
            if let Some(else_body) = else_body {
                for stmt in else_body {
                    validate_stmt(stmt, issues, in_function);
                }
            }
        }
        RustStmt::Match { expr, arms } => {
            validate_expr(expr, issues, in_function);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    validate_expr(guard, issues, in_function);
                }
                for stmt in &arm.body {
                    validate_stmt(stmt, issues, in_function);
                }
            }
        }
        RustStmt::For { iter, body, .. } => {
            validate_expr(iter, issues, in_function);
            for stmt in body {
                validate_stmt(stmt, issues, in_function);
            }
        }
        RustStmt::With { items, body } => {
            for item in items {
                validate_expr(&item.value, issues, in_function);
            }
            for stmt in body {
                validate_stmt(stmt, issues, in_function);
            }
        }
        RustStmt::While { cond, body } => {
            validate_expr(cond, issues, in_function);
            for stmt in body {
                validate_stmt(stmt, issues, in_function);
            }
        }
        RustStmt::Loop { body } | RustStmt::Block(body) => {
            for stmt in body {
                validate_stmt(stmt, issues, in_function);
            }
        }
        RustStmt::LocalFn {
            params, ret, body, ..
        } => {
            for param in params {
                if let RustParam::Named { ty, .. } | RustParam::NamedMut { ty, .. } = param {
                    validate_type(ty, issues);
                }
            }
            if let Some(ret) = ret {
                validate_type(ret, issues);
            }
            for stmt in body {
                validate_stmt(stmt, issues, true);
            }
        }
        RustStmt::Break | RustStmt::Continue => {}
    }
}

fn validate_expr(expr: &RustExpr, issues: &mut Vec<IrValidationIssue>, in_function: bool) {
    match expr {
        RustExpr::Literal(_) | RustExpr::Path(_) => {}
        RustExpr::Verbatim(source) => {
            if let Err(error) = syn::parse_str::<syn::Expr>(source) {
                issues.push(IrValidationIssue {
                    kind: IrValidationKind::InvalidVerbatimExpression,
                    message: format!("compiler-owned verbatim Rust expression is invalid: {error}"),
                });
            }
        }
        RustExpr::Ident(name) => {
            let mut characters = name.chars();
            let valid = characters
                .next()
                .is_some_and(|first| first == '_' || first.is_alphabetic())
                && characters.all(|character| character == '_' || character.is_alphanumeric());
            if !valid {
                issues.push(IrValidationIssue {
                    kind: IrValidationKind::InvalidIdentifier,
                    message: format!("Rust identifier IR contains raw syntax: {name}"),
                });
            }
        }
        RustExpr::MethodCall { receiver, args, .. } => {
            validate_expr(receiver, issues, in_function);
            for arg in args {
                validate_expr(arg, issues, in_function);
            }
        }
        RustExpr::FnCall { func, args } => {
            validate_expr(func, issues, in_function);
            for arg in args {
                validate_expr(arg, issues, in_function);
            }
        }
        RustExpr::MacroCall { args, .. }
        | RustExpr::Tuple(args)
        | RustExpr::Array(args)
        | RustExpr::Vec(args) => {
            for arg in args {
                validate_expr(arg, issues, in_function);
            }
        }
        RustExpr::TimeoutAwait {
            duration,
            future,
            error,
        } => {
            validate_expr(duration, issues, in_function);
            validate_expr(future, issues, in_function);
            validate_expr(error, issues, in_function);
        }
        RustExpr::FormatMacro { args, .. } => {
            for arg in args {
                validate_expr(arg, issues, in_function);
            }
        }
        RustExpr::BinOp { left, right, .. } => {
            validate_expr(left, issues, in_function);
            validate_expr(right, issues, in_function);
        }
        RustExpr::UnaryOp { operand, .. }
        | RustExpr::Deref(operand)
        | RustExpr::Clone(operand)
        | RustExpr::Try(operand)
        | RustExpr::Paren(operand)
        | RustExpr::Await(operand) => validate_expr(operand, issues, in_function),
        RustExpr::Field { expr, .. } => validate_expr(expr, issues, in_function),
        RustExpr::Index { expr, index } => {
            validate_expr(expr, issues, in_function);
            validate_expr(index, issues, in_function);
        }
        RustExpr::Slice { expr, start, stop } => {
            validate_expr(expr, issues, in_function);
            if let Some(start) = start {
                validate_expr(start, issues, in_function);
            }
            if let Some(stop) = stop {
                validate_expr(stop, issues, in_function);
            }
        }
        RustExpr::Ref { expr, .. } => validate_expr(expr, issues, in_function),
        RustExpr::Cast { expr, ty } => {
            validate_expr(expr, issues, in_function);
            validate_type(ty, issues);
        }
        RustExpr::Block { stmts, expr } => {
            for stmt in stmts {
                validate_stmt(stmt, issues, in_function);
            }
            if let Some(expr) = expr {
                validate_expr(expr, issues, in_function);
            }
        }
        RustExpr::If {
            cond,
            then_expr,
            else_expr,
        } => {
            validate_expr(cond, issues, in_function);
            validate_expr(then_expr, issues, in_function);
            if let Some(else_expr) = else_expr {
                validate_expr(else_expr, issues, in_function);
            }
        }
        RustExpr::Match { expr, arms } => {
            validate_expr(expr, issues, in_function);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    validate_expr(guard, issues, in_function);
                }
                for stmt in &arm.body {
                    validate_stmt(stmt, issues, in_function);
                }
            }
        }
        RustExpr::Closure { body, .. } => validate_expr(body, issues, in_function),
        RustExpr::ClosureBlock { body, .. } | RustExpr::AsyncBlock { body, .. } => {
            for stmt in body {
                validate_stmt(stmt, issues, in_function);
            }
        }
        RustExpr::StructInit { fields, .. } => {
            for (_, value) in fields {
                validate_expr(value, issues, in_function);
            }
        }
        RustExpr::Range { start, end } => {
            validate_expr(start, issues, in_function);
            validate_expr(end, issues, in_function);
        }
    }
}

fn validate_type(ty: &RustType, issues: &mut Vec<IrValidationIssue>) {
    let _ = issues.len();
    match ty {
        RustType::I64
        | RustType::F64
        | RustType::Bool
        | RustType::String_
        | RustType::Unit
        | RustType::Never
        | RustType::Named(_) => {}
        RustType::Vec(inner)
        | RustType::HashSet(inner)
        | RustType::VecDeque(inner)
        | RustType::Option(inner)
        | RustType::Boxed(inner)
        | RustType::Ref { inner, .. } => validate_type(inner, issues),
        RustType::Array { element, .. } => validate_type(element, issues),
        RustType::HashMap(key, value) | RustType::Result(key, value) => {
            validate_type(key, issues);
            validate_type(value, issues);
        }
        RustType::Tuple(types) => {
            for ty in types {
                validate_type(ty, issues);
            }
        }
        RustType::Generic { params, .. } => {
            for param in params {
                validate_type(param, issues);
            }
        }
        RustType::Fn { params, ret } => {
            for param in params {
                validate_type(param, issues);
            }
            validate_type(ret, issues);
        }
        RustType::DynTrait { trait_, .. } | RustType::ImplTrait { trait_, .. } => {
            validate_trait(trait_, issues);
        }
    }
}

fn validate_trait(trait_: &crate::RustTrait, issues: &mut Vec<IrValidationIssue>) {
    match trait_ {
        crate::RustTrait::Named {
            params,
            associated_types,
            ..
        } => {
            for ty in params {
                validate_type(ty, issues);
            }
            for (_, ty) in associated_types {
                validate_type(ty, issues);
            }
        }
        crate::RustTrait::Callable { params, ret, .. } => {
            for ty in params {
                validate_type(ty, issues);
            }
            if let Some(ret) = ret {
                validate_type(ret, issues);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RustLiteral, RustParam, Visibility};

    #[test]
    fn catches_duplicate_struct_fields() {
        let items = vec![RustItem::Struct {
            name: "User".to_string(),
            visibility: Visibility::Private,
            derives: vec![],
            fields: vec![
                ("id".to_string(), RustType::I64),
                ("id".to_string(), RustType::I64),
            ],
        }];

        let issues = validate_items(&items);
        assert!(issues.iter().any(|issue| {
            issue.kind == IrValidationKind::DuplicateStructField
                && issue.message.contains("duplicate field")
        }));
    }

    #[test]
    fn catches_empty_function_body() {
        let items = vec![RustItem::Fn {
            name: "empty".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: None,
            body: vec![],
            is_async: false,
        }];

        let issues = validate_items(&items);
        assert!(issues
            .iter()
            .any(|issue| issue.kind == IrValidationKind::EmptyFunctionBody));
    }

    #[test]
    fn catches_return_outside_function() {
        let items = vec![RustItem::Const {
            name: "X".to_string(),
            visibility: Visibility::Private,
            ty: RustType::I64,
            value: RustExpr::Block {
                stmts: vec![RustStmt::Return(Some(RustExpr::Literal(RustLiteral::Int(
                    1,
                ))))],
                expr: Some(Box::new(RustExpr::Literal(RustLiteral::Int(0)))),
            },
        }];

        let issues = validate_items(&items);
        assert!(issues
            .iter()
            .any(|issue| issue.kind == IrValidationKind::ReturnOutsideFunction));
    }

    #[test]
    fn accepts_balanced_ir() {
        let items = vec![RustItem::Fn {
            name: "ok".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![RustParam::Named {
                name: "n".to_string(),
                ty: RustType::I64,
            }],
            ret: Some(RustType::I64),
            body: vec![RustStmt::Return(Some(RustExpr::Ident("n".to_string())))],
            is_async: false,
        }];

        let issues = validate_items(&items);
        assert!(issues.is_empty());
    }

    #[test]
    fn rejects_raw_syntax_in_identifier_nodes() {
        let items = vec![RustItem::Const {
            name: "BROKEN".to_string(),
            visibility: Visibility::Private,
            ty: RustType::I64,
            value: RustExpr::Ident("std::cmp::max(1, 2)".to_string()),
        }];

        let issues = validate_items(&items);
        assert!(issues.iter().any(|issue| {
            issue.kind == IrValidationKind::InvalidIdentifier
                && issue.message.contains("std::cmp::max(1, 2)")
        }));
    }

    #[test]
    fn validates_explicit_verbatim_expression_nodes() {
        let valid = vec![RustItem::Const {
            name: "VALUE".to_string(),
            visibility: Visibility::Private,
            ty: RustType::I64,
            value: RustExpr::Verbatim("std::cmp::max(1, 2)".to_string()),
        }];
        assert!(validate_items(&valid).is_empty());

        let invalid = vec![RustItem::Const {
            name: "BROKEN".to_string(),
            visibility: Visibility::Private,
            ty: RustType::I64,
            value: RustExpr::Verbatim("std::cmp::max(".to_string()),
        }];
        assert!(validate_items(&invalid)
            .iter()
            .any(|issue| issue.kind == IrValidationKind::InvalidVerbatimExpression));
    }
}
