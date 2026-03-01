use crate::{RustEmitter, RustStmt};
use sifr_hir::{HirExceptHandler, HirExpr, HirFStringPart, HirStmt};
use sifr_type_system::Type;

fn io_error_kind_for_handler(error_type: &str) -> Option<&'static str> {
    match error_type {
        "FileNotFoundError" => Some("FileNotFound"),
        "PermissionError" => Some("PermissionDenied"),
        "FileExistsError" => Some("FileExists"),
        "IsADirectoryError" => Some("IsADirectory"),
        "NotADirectoryError" => Some("NotADirectory"),
        "DirectoryNotEmptyError" => Some("DirectoryNotEmpty"),
        _ => None,
    }
}

fn select_try_error_type(handlers: &[HirExceptHandler]) -> String {
    if handlers.iter().any(|handler| {
        let Some(error_type) = handler.error_type.as_deref() else {
            return false;
        };
        error_type == "IOError" || io_error_kind_for_handler(error_type).is_some()
    }) {
        return "IOError".to_string();
    }

    handlers
        .first()
        .and_then(|handler| handler.error_resolved_type.as_ref())
        .map(|ty| crate::render_type(&crate::sifr_type_to_rust_type(ty)))
        .unwrap_or_else(|| "Error".to_string())
}

enum HandlerMatchCondition {
    Unsupported,
    Always,
    Expr(String),
}

fn body_contains_return_stmt(stmts: &[HirStmt]) -> bool {
    stmts.iter().any(stmt_contains_return_stmt)
}

fn stmt_contains_return_stmt(stmt: &HirStmt) -> bool {
    match stmt {
        HirStmt::Return { .. } => true,
        HirStmt::If {
            then_body,
            elif_clauses,
            else_body,
            ..
        } => {
            body_contains_return_stmt(then_body)
                || elif_clauses
                    .iter()
                    .any(|(_, body)| body_contains_return_stmt(body))
                || else_body
                    .as_ref()
                    .is_some_and(|body| body_contains_return_stmt(body))
        }
        HirStmt::While {
            body, else_body, ..
        }
        | HirStmt::For {
            body, else_body, ..
        } => {
            body_contains_return_stmt(body)
                || else_body
                    .as_ref()
                    .is_some_and(|body| body_contains_return_stmt(body))
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            body_contains_return_stmt(body)
                || handlers
                    .iter()
                    .any(|handler| body_contains_return_stmt(&handler.body))
        }
        HirStmt::With { body, .. } => body_contains_return_stmt(body),
        HirStmt::Match { arms, .. } => arms.iter().any(|arm| body_contains_return_stmt(&arm.body)),
        HirStmt::NestedFunction { .. }
        | HirStmt::Let { .. }
        | HirStmt::Assign { .. }
        | HirStmt::AugAssign { .. }
        | HirStmt::Expr { .. }
        | HirStmt::Break
        | HirStmt::Continue
        | HirStmt::TupleUnpack { .. }
        | HirStmt::StarUnpack { .. }
        | HirStmt::Pass
        | HirStmt::Assert { .. }
        | HirStmt::Raise { .. }
        | HirStmt::FieldAssign { .. }
        | HirStmt::SubscriptAssign { .. }
        | HirStmt::NestedSubscriptAssign { .. }
        | HirStmt::SubscriptAugAssign { .. }
        | HirStmt::AttributeAugAssign { .. }
        | HirStmt::AttributeSubscriptAssign { .. }
        | HirStmt::Delete { .. }
        | HirStmt::Yield { .. } => false,
    }
}

fn body_always_exits_stmt(stmts: &[HirStmt]) -> bool {
    let Some(last) = stmts.last() else {
        return false;
    };
    match last {
        HirStmt::Return { .. } | HirStmt::Raise { .. } => true,
        HirStmt::If {
            then_body,
            elif_clauses,
            else_body,
            ..
        } => {
            body_always_exits_stmt(then_body)
                && elif_clauses
                    .iter()
                    .all(|(_, body)| body_always_exits_stmt(body))
                && else_body
                    .as_ref()
                    .is_some_and(|body| body_always_exits_stmt(body))
        }
        HirStmt::Match { arms, .. } => {
            !arms.is_empty() && arms.iter().all(|arm| body_always_exits_stmt(&arm.body))
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            body_always_exits_stmt(body)
                && !handlers.is_empty()
                && handlers
                    .iter()
                    .all(|handler| body_always_exits_stmt(&handler.body))
        }
        _ => false,
    }
}

impl RustEmitter {
    fn uses_debug_display_format_for_ir(ty: &Type) -> bool {
        match crate::resolve_alias_type_for_plain_call(ty) {
            Type::Int
            | Type::Float
            | Type::Bool
            | Type::Str
            | Type::None
            | Type::Range
            | Type::Union(_)
            | Type::LiteralInt(_)
            | Type::LiteralStr(_)
            | Type::LiteralBool(_)
            | Type::Class { .. }
            | Type::Newtype { .. }
            | Type::TypeVar(_)
            | Type::Enum { .. }
            | Type::BigInt => false,
            Type::List(_)
            | Type::Dict(_, _)
            | Type::Set(_)
            | Type::Tuple(_)
            | Type::Function(_)
            | Type::Callable(..)
            | Type::Result(_, _)
            | Type::Protocol { .. }
            | Type::Any
            | Type::Unknown
            | Type::Intersection(_)
            | Type::Never => true,
            Type::Alias(_, inner) => Self::uses_debug_display_format_for_ir(inner),
        }
    }

    fn option_inner_type_for_ir(ty: &Type) -> Option<&Type> {
        let resolved = crate::resolve_alias_type_for_plain_call(ty);
        let Type::Union(members) = resolved else {
            return None;
        };
        if members.len() != 2 || !members.iter().any(|member| matches!(member, Type::None)) {
            return None;
        }
        members.iter().find(|member| !matches!(member, Type::None))
    }

    fn collect_stmt_string_concat_parts_for_ir<'a>(expr: &'a HirExpr, parts: &mut Vec<&'a HirExpr>) {
        if let HirExpr::BinOp {
            left,
            op,
            right,
            ty,
        } = expr
        {
            if op == "+" && matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Str) {
                Self::collect_stmt_string_concat_parts_for_ir(left, parts);
                Self::collect_stmt_string_concat_parts_for_ir(right, parts);
                return;
            }
        }
        parts.push(expr);
    }

    fn try_lower_stmt_string_concat_expr_for_ir(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let HirExpr::BinOp {
            left,
            op,
            right,
            ty,
        } = expr
        else {
            return Ok(None);
        };
        if op != "+" || !matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Str) {
            return Ok(None);
        }

        let mut parts = Vec::new();
        Self::collect_stmt_string_concat_parts_for_ir(left, &mut parts);
        Self::collect_stmt_string_concat_parts_for_ir(right, &mut parts);

        if parts
            .iter()
            .all(|part| matches!(part, HirExpr::StringLiteral(_)))
        {
            let mut combined = String::new();
            for part in parts {
                if let HirExpr::StringLiteral(value) = part {
                    combined.push_str(value);
                }
            }
            return Ok(Some(crate::RustExpr::Literal(crate::RustLiteral::Str(combined))));
        }

        let mut lowered_parts = Vec::with_capacity(parts.len());
        for part in parts {
            let Some(lowered_part) = self.lower_stmt_expr_for_ir(part)? else {
                return Ok(None);
            };
            lowered_parts.push(lowered_part);
        }
        Ok(Some(crate::RustExpr::FormatMacro {
            name: "format".to_string(),
            format_str: "{}".repeat(lowered_parts.len()),
            args: lowered_parts,
        }))
    }

    fn resolve_alias_type_for_loop_iter(ty: &Type) -> &Type {
        match ty {
            Type::Alias(_, inner) => Self::resolve_alias_type_for_loop_iter(inner),
            _ => ty,
        }
    }

    pub(crate) fn lower_stmt_expr_for_ir(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let Some(lowered) = self.try_lower_registry_expr_result(expr)? {
            return Ok(Some(lowered));
        }
        if let HirExpr::FieldAccess { object, field, ty } = expr {
            if let Some(lowered) = self.try_lower_structured_field_access_expr(object, field, ty)? {
                return Ok(Some(lowered));
            }
        }
        if let HirExpr::ConstructorCall {
            class_name, args, ..
        } = expr
        {
            let mut lowered_args = Vec::with_capacity(args.len());
            for arg in args {
                let Some(lowered_arg) = self.lower_stmt_expr_for_ir(arg)? else {
                    return Ok(None);
                };
                lowered_args.push(lowered_arg);
            }
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    class_name.clone(),
                    "new".to_string(),
                ])),
                args: lowered_args,
            }));
        }
        if let HirExpr::FString { parts, .. } = expr {
            let mut format_str = String::new();
            let mut args = Vec::new();
            for part in parts {
                match part {
                    HirFStringPart::Literal(text) => {
                        format_str.push_str(&text.replace('{', "{{").replace('}', "}}"));
                    }
                    HirFStringPart::Expr(inner) => {
                        let Some(lowered_inner) = self.lower_stmt_expr_for_ir(inner)? else {
                            return Ok(None);
                        };
                        format_str.push_str("{}");
                        args.push(lowered_inner);
                    }
                }
            }
            return Ok(Some(crate::RustExpr::FormatMacro {
                name: "format".to_string(),
                format_str,
                args,
            }));
        }
        if let HirExpr::ListLiteral { elements, .. } = expr {
            let mut lowered_elements = Vec::with_capacity(elements.len());
            for element in elements {
                let Some(lowered_element) = self.lower_stmt_expr_for_ir(element)? else {
                    return Ok(None);
                };
                lowered_elements.push(lowered_element);
            }
            return Ok(Some(crate::RustExpr::Vec(lowered_elements)));
        }
        if let HirExpr::Call { func, args, .. } = expr {
            if let Some(lowered_intrinsic) = self.try_lower_registry_intrinsic_call_expr(func, args)
            {
                return Ok(Some(lowered_intrinsic));
            }
            if let Some(lowered_builtin) = self.try_lower_registry_builtin_call_expr(func, args) {
                return Ok(Some(lowered_builtin));
            }
            if func == "str" && args.is_empty() {
                return Ok(Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "String".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                }));
            }
            if func == "str" && args.len() == 1 {
                let arg = &args[0];
                let Some(lowered_arg) = self.lower_stmt_expr_for_ir(arg)? else {
                    return Ok(None);
                };
                if let Some(inner) = Self::option_inner_type_for_ir(arg.ty()) {
                    let format_str = if Self::uses_debug_display_format_for_ir(inner) {
                        "{:?}".to_string()
                    } else {
                        "{}".to_string()
                    };
                    return Ok(Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                        method: "map_or".to_string(),
                        args: vec![
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Literal(
                                    crate::RustLiteral::Str("None".to_string()),
                                )),
                                method: "to_string".to_string(),
                                args: vec![],
                            },
                            crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__v".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::FormatMacro {
                                    name: "format".to_string(),
                                    format_str,
                                    args: vec![crate::RustExpr::Ident("__v".to_string())],
                                }),
                                is_move: false,
                            },
                        ],
                    }));
                }
                return Ok(Some(crate::RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str: if Self::uses_debug_display_format_for_ir(arg.ty()) {
                        "{:?}".to_string()
                    } else {
                        "{}".to_string()
                    },
                    args: vec![lowered_arg],
                }));
            }
            let mut lowered_args = Vec::with_capacity(args.len());
            for arg in args {
                let Some(lowered_arg) = self.lower_stmt_expr_for_ir(arg)? else {
                    return Ok(None);
                };
                lowered_args.push(lowered_arg);
            }
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(
                    func.split("::").map(|s| s.to_string()).collect(),
                )),
                args: lowered_args,
            }));
        }
        if let HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } = expr
        {
            let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
                return Ok(None);
            };
            let mut lowered_args = Vec::with_capacity(args.len());
            for arg in args {
                let Some(lowered_arg) = self.lower_stmt_expr_for_ir(arg)? else {
                    return Ok(None);
                };
                lowered_args.push(lowered_arg);
            }
            return Ok(Some(crate::RustExpr::MethodCall {
                receiver: Box::new(lowered_object),
                method: method.clone(),
                args: lowered_args,
            }));
        }
        if let HirExpr::QuestionMark { expr: inner, .. } = expr {
            let Some(lowered_inner) = self.lower_stmt_expr_for_ir(inner)? else {
                return Ok(None);
            };
            return Ok(Some(crate::RustExpr::Try(Box::new(lowered_inner))));
        }
        if let HirExpr::OkWrap { value, .. } = expr {
            let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
                return Ok(None);
            };
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                args: vec![lowered_value],
            }));
        }
        if let HirExpr::ErrWrap { value, .. } = expr {
            let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
                return Ok(None);
            };
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Err".to_string()])),
                args: vec![lowered_value],
            }));
        }
        if let HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } = expr
        {
            let Some(lowered_condition) = self.lower_stmt_expr_for_ir(condition)? else {
                return Ok(None);
            };
            let Some(lowered_then) = self.lower_stmt_expr_for_ir(then_expr)? else {
                return Ok(None);
            };
            let Some(lowered_else) = self.lower_stmt_expr_for_ir(else_expr)? else {
                return Ok(None);
            };
            return Ok(Some(crate::RustExpr::If {
                cond: Box::new(lowered_condition),
                then_expr: Box::new(lowered_then),
                else_expr: Some(Box::new(lowered_else)),
            }));
        }
        if let HirExpr::Index { object, index, .. } = expr {
            if let Some(lowered) = self.try_lower_structured_index_expr(object, index)? {
                return Ok(Some(lowered));
            }
            let object_ty = crate::resolve_alias_type_for_plain_call(object.ty());
            let option_inner_ty = if let Type::Union(members) = object_ty {
                let mut non_none = members.iter().filter(|m| !matches!(m, Type::None));
                let first = non_none.next();
                if non_none.next().is_none() && members.iter().any(|m| matches!(m, Type::None)) {
                    first
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(inner_ty) = option_inner_ty {
                let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
                    return Ok(None);
                };
                let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
                    return Ok(None);
                };
                let option_index_expr = match inner_ty {
                    Type::Dict(_, _) => {
                        let key_arg = if matches!(index.as_ref(), HirExpr::StringLiteral(_)) {
                            lowered_index
                        } else {
                            crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(lowered_index),
                            }
                        };
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                method: "get".to_string(),
                                args: vec![key_arg],
                            }),
                            method: "cloned".to_string(),
                            args: vec![],
                        }
                    }
                    Type::List(_) => crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident("__v".to_string())),
                            method: "get".to_string(),
                            args: vec![crate::RustExpr::Cast {
                                expr: Box::new(lowered_index),
                                ty: crate::RustType::Named("usize".to_string()),
                            }],
                        }),
                        method: "cloned".to_string(),
                        args: vec![],
                    },
                    Type::Str => crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                method: "chars".to_string(),
                                args: vec![],
                            }),
                            method: "nth".to_string(),
                            args: vec![crate::RustExpr::Cast {
                                expr: Box::new(lowered_index),
                                ty: crate::RustType::Named("usize".to_string()),
                            }],
                        }),
                        method: "map".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "c".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident("c".to_string())),
                                method: "to_string".to_string(),
                                args: vec![],
                            }),
                            is_move: false,
                        }],
                    },
                    _ => return Ok(None),
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(lowered_object),
                        method: "as_ref".to_string(),
                        args: vec![],
                    }),
                    method: "and_then".to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "__v".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(option_index_expr),
                        is_move: false,
                    }],
                }));
            }

            let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
                return Ok(None);
            };
            let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
                return Ok(None);
            };
            match object_ty {
                Type::Dict(_, _) => {
                    let key_arg = if matches!(index.as_ref(), HirExpr::StringLiteral(_)) {
                        lowered_index
                    } else {
                        crate::RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(lowered_index),
                        }
                    };
                    return Ok(Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_object),
                            method: "get".to_string(),
                            args: vec![key_arg],
                        }),
                        method: "cloned".to_string(),
                        args: vec![],
                    }));
                }
                Type::List(_) => {
                    return Ok(Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_object),
                            method: "get".to_string(),
                            args: vec![crate::RustExpr::Cast {
                                expr: Box::new(lowered_index),
                                ty: crate::RustType::Named("usize".to_string()),
                            }],
                        }),
                        method: "cloned".to_string(),
                        args: vec![],
                    }));
                }
                Type::Str => {
                    return Ok(Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered_object),
                                method: "chars".to_string(),
                                args: vec![],
                            }),
                            method: "nth".to_string(),
                            args: vec![crate::RustExpr::Cast {
                                expr: Box::new(lowered_index),
                                ty: crate::RustType::Named("usize".to_string()),
                            }],
                        }),
                        method: "map".to_string(),
                        args: vec![crate::RustExpr::Closure {
                            params: vec![crate::RustParam::Named {
                                name: "c".to_string(),
                                ty: crate::RustType::Named("_".to_string()),
                            }],
                            body: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident("c".to_string())),
                                method: "to_string".to_string(),
                                args: vec![],
                            }),
                            is_move: false,
                        }],
                    }));
                }
                Type::Tuple(_) => {
                    let HirExpr::IntLiteral(idx) = index.as_ref() else {
                        return Ok(None);
                    };
                    return Ok(Some(crate::RustExpr::Field {
                        expr: Box::new(lowered_object),
                        field: idx.to_string(),
                    }));
                }
                _ => {}
            }
        }
        if let HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } = expr
        {
            if ops.len() == 1 && comparators.len() == 1 {
                let lowered_op = match ops[0].as_str() {
                    "==" | "!=" | "<" | "<=" | ">" | ">=" => ops[0].clone(),
                    "is" => "==".to_string(),
                    "is not" => "!=".to_string(),
                    _ => return Ok(None),
                };
                let Some(lowered_left) = self.lower_stmt_expr_for_ir(left)? else {
                    return Ok(None);
                };
                let Some(lowered_right) = self.lower_stmt_expr_for_ir(&comparators[0])? else {
                    return Ok(None);
                };
                return Ok(Some(crate::RustExpr::BinOp {
                    left: Box::new(lowered_left),
                    op: lowered_op,
                    right: Box::new(lowered_right),
                }));
            }
        }
        if let HirExpr::BoolOp { op, values, .. } = expr {
            let lowered_op = match op.as_str() {
                "and" => "&&",
                "or" => "||",
                _ => return Ok(None),
            };
            if values.is_empty() {
                return Ok(None);
            }
            let mut iter = values.iter();
            let Some(first) = iter.next() else {
                return Ok(None);
            };
            let Some(mut acc) = self.lower_stmt_expr_for_ir(first)? else {
                return Ok(None);
            };
            for value in iter {
                let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
                    return Ok(None);
                };
                acc = crate::RustExpr::BinOp {
                    left: Box::new(crate::RustExpr::Paren(Box::new(acc))),
                    op: lowered_op.to_string(),
                    right: Box::new(crate::RustExpr::Paren(Box::new(lowered_value))),
                };
            }
            return Ok(Some(crate::RustExpr::Paren(Box::new(acc))));
        }
        if let HirExpr::BinOp {
            left, op, right, ..
        } = expr
        {
            if let Some(lowered) = self.try_lower_structured_class_binop_expr(left, op, right)? {
                return Ok(Some(lowered));
            }
            if let Some(lowered) = self.try_lower_stmt_string_concat_expr_for_ir(expr)? {
                return Ok(Some(lowered));
            }
            let Some(lowered_left) = self.lower_stmt_expr_for_ir(left)? else {
                return Ok(None);
            };
            let Some(lowered_right) = self.lower_stmt_expr_for_ir(right)? else {
                return Ok(None);
            };
            if op == "**" {
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_left))),
                    method: "pow".to_string(),
                    args: vec![crate::RustExpr::Cast {
                        expr: Box::new(lowered_right),
                        ty: crate::RustType::Named("u32".to_string()),
                    }],
                }));
            }
            return Ok(Some(crate::RustExpr::BinOp {
                left: Box::new(lowered_left),
                op: if op == "//" {
                    "/".to_string()
                } else {
                    op.clone()
                },
                right: Box::new(lowered_right),
            }));
        }
        crate::try_lower_leaf_or_name_expr_result(expr)
    }

    fn object_name_expr_for_ir(object: &str) -> crate::RustExpr {
        if object.contains("::") {
            return crate::RustExpr::Path(object.split("::").map(|s| s.to_string()).collect());
        }
        crate::RustExpr::Ident(object.to_string())
    }

    pub(super) fn write_stmt_terminator(&mut self) {
        self.write(";\n");
    }

    fn write_wrapped_try_return_prefix(&mut self, wrap_option: bool) {
        if wrap_option {
            self.write("return Ok(Some(");
        } else {
            self.write("return Ok(");
        }
    }

    fn write_wrapped_try_return_suffix(&mut self, wrap_option: bool) {
        if wrap_option {
            self.write("));\n");
        } else {
            self.write(");\n");
        }
    }

    pub(super) fn emit_borrowed_return_name_clone_expr(&mut self, value: &HirExpr) -> bool {
        let HirExpr::Name { name, .. } = value else {
            return false;
        };
        if !(self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name)) {
            return false;
        }
        self.emit_rust_expr(&crate::RustExpr::Clone(Box::new(crate::RustExpr::Ident(
            name.clone(),
        ))));
        true
    }

    /// Emit a generator initialization statement (always mutable for closure capture)
    pub(super) fn emit_generator_init_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Let {
                name, ty, value, ..
            } => {
                let lowered_value = match self.lower_stmt_expr_for_ir(value) {
                    Ok(Some(lowered)) => lowered,
                    Ok(None) | Err(_) => {
                        panic!(
                            "structured generator-init expression emission missing for production path: {value:?}"
                        );
                    }
                };
                self.emit_rust_stmt_with_current_indent(&crate::RustStmt::Let {
                    mutable: true,
                    name: name.clone(),
                    ty: Some(crate::sifr_type_to_rust_type(ty)),
                    value: lowered_value,
                });
            }
            _ => match self.try_emit_structured_stmt(stmt) {
                Ok(true) => {}
                Ok(false) | Err(_) => {
                    panic!(
                            "structured generator-init statement emission missing for production path: {stmt:?}"
                        );
                }
            },
        }
    }

    pub(super) fn emit_lowered_stmts(&mut self, lowered_stmts: &[RustStmt]) {
        for lowered_stmt in lowered_stmts {
            match lowered_stmt {
                RustStmt::Let {
                    mutable,
                    name,
                    ty,
                    value,
                } => self.emit_rust_stmt_with_current_indent(&crate::RustStmt::Let {
                    mutable: *mutable,
                    name: name.clone(),
                    ty: ty.clone(),
                    value: if let crate::RustExpr::Ident(value_name) = value {
                        if self.borrowed_params.contains(value_name)
                            || self.mut_borrowed_params.contains(value_name)
                        {
                            crate::RustExpr::Clone(Box::new(crate::RustExpr::Paren(Box::new(
                                crate::RustExpr::Ident(value_name.clone()),
                            ))))
                        } else {
                            value.clone()
                        }
                    } else {
                        value.clone()
                    },
                }),
                RustStmt::Expr(lowered_expr) => self
                    .emit_rust_stmt_with_current_indent(&crate::RustStmt::Expr(lowered_expr.clone())),
                RustStmt::RawCode(_) => {
                    panic!("RawCode statement reached core production emission path");
                }
                _ => self.emit_rust_stmt_with_current_indent(lowered_stmt),
            }
        }
    }

    pub(super) fn current_loop_has_else(&self) -> bool {
        self.loop_else_stack.last().copied().unwrap_or(false)
    }

    pub(crate) fn try_emit_structured_return_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::Return { value } = stmt else {
            return Ok(false);
        };

        if let Some(value) = value {
            if self.emission_ctx.in_display_impl && self.try_closure_depth == 0 {
                let output_len = self.output.len();
                self.write_indent();
                self.write("return write!(f, \"{}\", ");
                if self.try_emit_structured_expr(value)? {
                    self.write(");\n");
                    return Ok(true);
                }
                self.output.truncate(output_len);
                return Ok(false);
            }
            let output_len = self.output.len();
            if self.try_closure_depth > 0 {
                let wrap_option = self
                    .try_closure_option_wrap
                    .last()
                    .copied()
                    .unwrap_or(false);
                self.write_wrapped_try_return_prefix(wrap_option);
                if self.current_class_name.is_some()
                    && matches!(value, HirExpr::Name { name, .. } if name == "self")
                {
                    self.write("self.clone()");
                    self.write_wrapped_try_return_suffix(wrap_option);
                    return Ok(true);
                }
                if !wrap_option {
                    if let Some(return_ty) = self.current_return_type.as_ref() {
                        if let Type::Result(ok_ty, _) =
                            crate::resolve_alias_type_for_plain_call(return_ty)
                        {
                            let value_is_none_like = matches!(value, HirExpr::NoneLiteral)
                                || matches!(
                                    crate::resolve_alias_type_for_plain_call(value.ty()),
                                    Type::None
                                );
                            if value_is_none_like
                                && matches!(
                                    crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
                                    Type::None
                                )
                            {
                                self.write("Ok(())");
                                self.write_wrapped_try_return_suffix(wrap_option);
                                return Ok(true);
                            }
                        }
                    }
                }
                if let Some(return_ty) = self.current_return_type.clone() {
                    if let HirExpr::Index { object, index, .. } = value {
                        if !crate::helpers::is_option_type(&return_ty)
                            && self.try_emit_structured_index_expr(object, index, &return_ty)?
                        {
                            self.write_wrapped_try_return_suffix(wrap_option);
                            return Ok(true);
                        }
                    }
                }
                if self.emit_borrowed_return_name_clone_expr(value) {
                    self.write_wrapped_try_return_suffix(wrap_option);
                    return Ok(true);
                }
                if self.try_emit_structured_expr(value)? {
                    self.write_wrapped_try_return_suffix(wrap_option);
                    return Ok(true);
                }
                self.output.truncate(output_len);
                return Ok(false);
            }

            if self.current_class_name.is_some()
                && matches!(value, HirExpr::Name { name, .. } if name == "self")
            {
                self.emit_rust_stmt_with_current_indent(&RustStmt::Return(Some(
                    crate::RustExpr::Clone(Box::new(crate::RustExpr::Ident("self".to_string()))),
                )));
                return Ok(true);
            }
            self.write("return ");
            if let Some(return_ty) = self.current_return_type.clone() {
                if let HirExpr::Index { object, index, .. } = value {
                    if !crate::helpers::is_option_type(&return_ty)
                        && self.try_emit_structured_index_expr(object, index, &return_ty)?
                    {
                        self.write_stmt_terminator();
                        return Ok(true);
                    }
                }
            }
            if self.emit_borrowed_return_name_clone_expr(value) {
                self.write_stmt_terminator();
                return Ok(true);
            }
            if self.try_emit_structured_expr(value)? {
                self.write_stmt_terminator();
                return Ok(true);
            }
            self.output.truncate(output_len);
            return Ok(false);
        }

        if self.try_closure_depth > 0 {
            let wrap_option = self
                .try_closure_option_wrap
                .last()
                .copied()
                .unwrap_or(false);
            if wrap_option {
                self.emit_rust_stmt_with_current_indent(&RustStmt::Return(Some(
                    crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                        args: vec![crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                            args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                        }],
                    },
                )));
            } else {
                let direct_result_none = self.current_return_type.as_ref().is_some_and(|ret_ty| {
                    match crate::resolve_alias_type_for_plain_call(ret_ty) {
                        Type::Result(ok_ty, _) => matches!(
                            crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
                            Type::None
                        ),
                        _ => false,
                    }
                });
                if direct_result_none {
                    self.emit_rust_stmt_with_current_indent(&RustStmt::Return(Some(
                        crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                            args: vec![crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                                args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                            }],
                        },
                    )));
                } else {
                    self.emit_rust_stmt_with_current_indent(&RustStmt::Return(Some(
                        crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                            args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                        },
                    )));
                }
            }
        } else if self.emission_ctx.in_display_impl {
            self.emit_rust_stmt_with_current_indent(&RustStmt::Return(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
            })));
        } else {
            self.emit_rust_stmt_with_current_indent(&RustStmt::Return(None));
        }
        Ok(true)
    }

    pub(crate) fn try_emit_structured_raise_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::Raise { value } = stmt else {
            return Ok(false);
        };
        let Some(lowered) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(false);
        };
        self.emit_rust_stmt_with_current_indent(&RustStmt::Return(Some(crate::RustExpr::FnCall {
            func: Box::new(crate::RustExpr::Path(vec!["Err".to_string()])),
            args: vec![lowered],
        })));
        Ok(true)
    }

    pub(crate) fn try_emit_structured_if_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } = stmt
        else {
            return Ok(false);
        };

        if elif_clauses.is_empty() && else_body.is_none() {
            if let HirExpr::Compare {
                left,
                ops,
                comparators,
                ..
            } = condition
            {
                if let HirExpr::WalrusExpr { name, value, ty } = left.as_ref() {
                    let output_len = self.output.len();
                    self.write_indent();
                    self.write("let ");
                    self.write(name);
                    self.write(" = ");
                    if !self.try_emit_structured_expr(value)? {
                        self.output.truncate(output_len);
                        return Ok(false);
                    }
                    self.write(";\n");
                    self.write_indent();
                    self.write("if ");
                    let walrus_name_expr = HirExpr::Name {
                        name: name.clone(),
                        ty: ty.clone(),
                    };
                    if !self.try_emit_structured_compare_expr(&walrus_name_expr, ops, comparators)? {
                        self.output.truncate(output_len);
                        return Ok(false);
                    }
                    self.write(" {\n");
                    self.indent += 1;
                    for then_stmt in then_body {
                        self.emit_stmt(then_stmt);
                    }
                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");
                    return Ok(true);
                }
            }
        }

        if let Some((var_name, first_variant, first_enum_name, _)) =
            crate::helpers::detect_isinstance_union(condition)
        {
            let mut branch_specs: Vec<(String, &[HirStmt])> = vec![(first_variant, then_body)];
            let mut needed_variants = vec![branch_specs[0].0.clone()];
            let mut all_isinstance = true;
            for (elif_cond, elif_body) in elif_clauses {
                let Some((elif_var, elif_variant, _, _)) =
                    crate::helpers::detect_isinstance_union(elif_cond)
                else {
                    all_isinstance = false;
                    break;
                };
                if elif_var != var_name {
                    all_isinstance = false;
                    break;
                }
                needed_variants.push(elif_variant.clone());
                branch_specs.push((elif_variant, elif_body.as_slice()));
            }
            if all_isinstance {
                let enum_name = self.resolve_union_enum_name(&first_enum_name, &needed_variants);
                let output_len = self.output.len();
                for (idx, (variant_name, body)) in branch_specs.iter().enumerate() {
                    self.write_indent();
                    if idx == 0 {
                        self.write("if let ");
                    } else {
                        self.write("else if let ");
                    }
                    let mutated = crate::helpers::collect_mutated_vars(body);
                    let binding = if mutated.contains(&var_name) {
                        format!("mut {var_name}")
                    } else {
                        var_name.clone()
                    };
                    self.write(&format!(
                        "{enum_name}::{variant_name}({binding}) = {var_name} {{\n"
                    ));
                    self.indent += 1;
                    for branch_stmt in *body {
                        self.emit_stmt(branch_stmt);
                    }
                    self.indent -= 1;
                    self.write_indent();
                    self.write("}");
                    if idx + 1 == branch_specs.len() {
                        if let Some(else_body) = else_body {
                            let remaining_variants = self
                                .union_enums
                                .get(&enum_name)
                                .map(|members| {
                                    members
                                        .iter()
                                        .map(Type::union_variant_name)
                                        .filter(|variant| !needed_variants.contains(variant))
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default();
                            if remaining_variants.len() == 1 {
                                let else_mutated = crate::helpers::collect_mutated_vars(else_body);
                                let else_binding = if else_mutated.contains(&var_name) {
                                    format!("mut {var_name}")
                                } else {
                                    var_name.clone()
                                };
                                self.write(" else if let ");
                                self.write(&format!(
                                    "{enum_name}::{}({else_binding}) = {var_name} {{\n",
                                    remaining_variants[0]
                                ));
                                self.indent += 1;
                                for else_stmt in else_body {
                                    self.emit_stmt(else_stmt);
                                }
                                self.indent -= 1;
                                self.write_indent();
                                self.write("} else {\n");
                                self.indent += 1;
                                self.write_indent();
                                self.write(
                                    "unreachable!(\"sifr union narrowing fell through exhaustive branch chain\");\n",
                                );
                                self.indent -= 1;
                                self.write_indent();
                                self.write("}");
                            } else {
                                self.write(" else {\n");
                                self.indent += 1;
                                for else_stmt in else_body {
                                    self.emit_stmt(else_stmt);
                                }
                                self.indent -= 1;
                                self.write_indent();
                                self.write("}");
                            }
                        }
                        self.write("\n");
                    } else {
                        self.write(" ");
                    }
                }
                if !self
                    .output
                    .get(output_len..)
                    .is_some_and(|segment| !segment.is_empty())
                {
                    self.output.truncate(output_len);
                    return Ok(false);
                }
                return Ok(true);
            }
        }

        if elif_clauses.is_empty() {
            if let Some((var_name, variant_name, enum_name, other_variants)) =
                crate::helpers::detect_isinstance_union(condition)
            {
                let mut needed_variants = vec![variant_name.clone()];
                needed_variants.extend(other_variants.iter().map(|(variant, _)| variant.clone()));
                let enum_name = self.resolve_union_enum_name(&enum_name, &needed_variants);
                let output_len = self.output.len();
                self.write_indent();
                self.write("match ");
                self.write(&var_name);
                self.write(" {\n");
                self.indent += 1;

                let then_mutated = crate::helpers::collect_mutated_vars(then_body);
                let then_binding = if then_mutated.contains(&var_name) {
                    format!("mut {var_name}")
                } else {
                    var_name.clone()
                };
                self.write_indent();
                self.write(&format!(
                    "{enum_name}::{variant_name}({then_binding}) => {{\n"
                ));
                self.indent += 1;
                for then_stmt in then_body {
                    self.emit_stmt(then_stmt);
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");

                if let Some(else_body) = else_body {
                    let else_mutated = crate::helpers::collect_mutated_vars(else_body);
                    let else_binding = if else_mutated.contains(&var_name) {
                        format!("mut {var_name}")
                    } else {
                        var_name.clone()
                    };
                    if other_variants.len() == 1 {
                        let (other_variant, _) = &other_variants[0];
                        self.write_indent();
                        self.write(&format!(
                            "{enum_name}::{other_variant}({else_binding}) => {{\n"
                        ));
                    } else {
                        self.write_indent();
                        self.write("_ => {\n");
                    }
                    self.indent += 1;
                    for else_stmt in else_body {
                        self.emit_stmt(else_stmt);
                    }
                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");
                } else {
                    self.write_indent();
                    self.write("_ => {}\n");
                }

                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
                if !self
                    .output
                    .get(output_len..)
                    .is_some_and(|segment| !segment.is_empty())
                {
                    self.output.truncate(output_len);
                    return Ok(false);
                }
                return Ok(true);
            }

            if let Some(var_name) = crate::helpers::detect_is_not_none_var(condition) {
                let output_len = self.output.len();
                self.write_indent();
                self.write("if let Some(");
                self.write(&var_name);
                self.write(") = ");
                self.write(&var_name);
                self.write(" {\n");
                self.indent += 1;
                self.option_unwrapped_vars.insert(var_name.clone());
                for then_stmt in then_body {
                    self.emit_stmt(then_stmt);
                }
                self.option_unwrapped_vars.remove(&var_name);
                self.indent -= 1;
                self.write_indent();
                self.write("}");
                if let Some(else_body) = else_body {
                    self.write(" else {\n");
                    self.indent += 1;
                    for else_stmt in else_body {
                        self.emit_stmt(else_stmt);
                    }
                    self.indent -= 1;
                    self.write_indent();
                    self.write("}");
                }
                self.write("\n");
                if !self
                    .output
                    .get(output_len..)
                    .is_some_and(|segment| !segment.is_empty())
                {
                    self.output.truncate(output_len);
                    return Ok(false);
                }
                return Ok(true);
            }

            if let Some(var_name) = crate::helpers::detect_is_none_var(condition) {
                let output_len = self.output.len();
                self.write_indent();
                self.write("if ");
                if !self.try_emit_structured_expr(condition)? {
                    self.output.truncate(output_len);
                    return Ok(false);
                }
                self.write(" {\n");
                self.indent += 1;
                for then_stmt in then_body {
                    self.emit_stmt(then_stmt);
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}");
                if let Some(else_body) = else_body {
                    self.write(" else if let Some(");
                    self.write(&var_name);
                    self.write(") = ");
                    self.write(&var_name);
                    self.write(" {\n");
                    self.indent += 1;
                    self.option_unwrapped_vars.insert(var_name.clone());
                    for else_stmt in else_body {
                        self.emit_stmt(else_stmt);
                    }
                    self.option_unwrapped_vars.remove(&var_name);
                    self.indent -= 1;
                    self.write_indent();
                    self.write("}");
                }
                self.write("\n");
                if !self
                    .output
                    .get(output_len..)
                    .is_some_and(|segment| !segment.is_empty())
                {
                    self.output.truncate(output_len);
                    return Ok(false);
                }
                return Ok(true);
            }

            if let Some(var_names) = crate::helpers::detect_and_not_none_vars(condition) {
                let output_len = self.output.len();
                self.write_indent();
                self.write("if let (");
                for (idx, var_name) in var_names.iter().enumerate() {
                    if idx > 0 {
                        self.write(", ");
                    }
                    self.write("Some(");
                    self.write(var_name);
                    self.write(")");
                }
                self.write(") = (");
                for (idx, var_name) in var_names.iter().enumerate() {
                    if idx > 0 {
                        self.write(", ");
                    }
                    self.write("&");
                    self.write(var_name);
                }
                self.write(") {\n");
                self.indent += 1;
                for var_name in &var_names {
                    self.option_unwrapped_vars.insert(var_name.clone());
                }
                for then_stmt in then_body {
                    self.emit_stmt(then_stmt);
                }
                for var_name in &var_names {
                    self.option_unwrapped_vars.remove(var_name);
                }
                self.indent -= 1;
                self.write_indent();
                self.write("}");
                if let Some(else_body) = else_body {
                    self.write(" else {\n");
                    self.indent += 1;
                    for else_stmt in else_body {
                        self.emit_stmt(else_stmt);
                    }
                    self.indent -= 1;
                    self.write_indent();
                    self.write("}");
                }
                self.write("\n");
                if !self
                    .output
                    .get(output_len..)
                    .is_some_and(|segment| !segment.is_empty())
                {
                    self.output.truncate(output_len);
                    return Ok(false);
                }
                return Ok(true);
            }
        }

        let output_len = self.output.len();
        self.write_indent();
        self.write("if ");
        if !self.try_emit_structured_expr(condition)? {
            self.output.truncate(output_len);
            return Ok(false);
        }
        self.write(" {\n");
        self.indent += 1;
        for then_stmt in then_body {
            self.emit_stmt(then_stmt);
        }
        self.indent -= 1;
        self.write_indent();
        self.write("}");

        for (elif_cond, elif_body) in elif_clauses {
            self.write(" else if ");
            if !self.try_emit_structured_expr(elif_cond)? {
                self.output.truncate(output_len);
                return Ok(false);
            }
            self.write(" {\n");
            self.indent += 1;
            for elif_stmt in elif_body {
                self.emit_stmt(elif_stmt);
            }
            self.indent -= 1;
            self.write_indent();
            self.write("}");
        }

        if let Some(else_body) = else_body {
            self.write(" else {\n");
            self.indent += 1;
            for else_stmt in else_body {
                self.emit_stmt(else_stmt);
            }
            self.indent -= 1;
            self.write_indent();
            self.write("}");
        }
        self.write("\n");
        Ok(true)
    }

    fn resolve_union_enum_name(&self, preferred: &str, needed_variants: &[String]) -> String {
        if self.union_enums.contains_key(preferred) {
            return preferred.to_string();
        }
        for (candidate, members) in &self.union_enums {
            if needed_variants.iter().all(|needed| {
                members
                    .iter()
                    .any(|member| member.union_variant_name() == *needed)
            }) {
                return candidate.clone();
            }
        }
        preferred.to_string()
    }

    pub(crate) fn try_emit_structured_while_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::While {
            condition,
            body,
            else_body,
        } = stmt
        else {
            return Ok(false);
        };
        if else_body.is_some() {
            return Ok(false);
        }

        let output_len = self.output.len();
        self.write_indent();
        self.write("while ");
        if self.try_emit_structured_expr(condition)? {
            self.loop_else_stack.push(false);
            self.write(" {\n");
            self.indent += 1;
            for body_stmt in body {
                self.emit_stmt(body_stmt);
            }
            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
            let popped = self.loop_else_stack.pop();
            debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
            return Ok(true);
        }
        self.output.truncate(output_len);
        Ok(false)
    }

    pub(crate) fn try_emit_structured_for_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::For {
            target,
            iter,
            body,
            else_body,
            ..
        } = stmt
        else {
            return Ok(false);
        };

        let output_len = self.output.len();
        let has_else = else_body.is_some();
        if has_else {
            self.write_indent();
            self.write("let mut _broke = false;\n");
        }

        self.loop_else_stack.push(has_else);
        self.write_indent();
        self.write("for ");
        if target.contains(',') {
            let names = target
                .split(',')
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .collect::<Vec<_>>();
            if names.is_empty() {
                let popped = self.loop_else_stack.pop();
                debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
                self.output.truncate(output_len);
                return Ok(false);
            }
            self.write("(");
            self.write(&names.join(", "));
            self.write(")");
        } else {
            self.write(target);
        }
        self.write(" in ");
        let iter_output_start = self.output.len();
        let mut iter_is_iterator = false;
        let emitted_iter = if let HirExpr::Call { func, args, .. } = iter {
            if func == "enumerate" && args.len() == 1 {
                let saved_stats = self.lowering_stats;
                let arg_rendered = self.try_render_structured_expr(&args[0])?;
                self.lowering_stats = saved_stats;
                if let Some(arg_rendered) = arg_rendered {
                    self.write("(");
                    self.write(&arg_rendered);
                    self.write(").iter().cloned().enumerate().map(|(i, v)| (i as i64, v))");
                    iter_is_iterator = true;
                    true
                } else {
                    false
                }
            } else {
                let saved_stats = self.lowering_stats;
                let rendered = self.try_render_structured_expr(iter)?;
                self.lowering_stats = saved_stats;
                if let Some(rendered) = rendered {
                    if rendered.contains(".into_iter()")
                        || rendered.contains(".map(")
                        || rendered.contains(".filter(")
                        || rendered.contains(".zip(")
                        || rendered.contains(".chain(")
                    {
                        iter_is_iterator = true;
                    }
                    self.write(&rendered);
                    true
                } else {
                    false
                }
            }
        } else {
            let saved_stats = self.lowering_stats;
            let rendered = self.try_render_structured_expr(iter)?;
            self.lowering_stats = saved_stats;
            if let Some(rendered) = rendered {
                if rendered.contains(".into_iter()")
                    || rendered.contains(".map(")
                    || rendered.contains(".filter(")
                    || rendered.contains(".zip(")
                    || rendered.contains(".chain(")
                {
                    iter_is_iterator = true;
                }
                self.write(&rendered);
                true
            } else {
                false
            }
        };

        if !emitted_iter {
            let popped = self.loop_else_stack.pop();
            debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
            self.output.truncate(output_len);
            return Ok(false);
        }
        if !iter_is_iterator {
            if let Some(iter_segment) = self.output.get(iter_output_start..) {
                if iter_segment.contains(".into_iter()")
                    || iter_segment.contains(".map(")
                    || iter_segment.contains(".filter(")
                    || iter_segment.contains(".zip(")
                    || iter_segment.contains(".chain(")
                {
                    iter_is_iterator = true;
                }
            }
        }

        let is_generator_expr = matches!(iter, HirExpr::GeneratorExpr { .. });
        let is_generator_fn_call = self.is_generator_call(iter);
        if !is_generator_expr && !is_generator_fn_call && !iter_is_iterator {
            match Self::resolve_alias_type_for_loop_iter(iter.ty()) {
                Type::List(_) => self.write(".iter().cloned()"),
                Type::Dict(_, _) => self.write(".keys().cloned()"),
                Type::Str => self.write(".chars().map(|c| c.to_string())"),
                _ => {}
            }
        }

        self.write(" {\n");
        self.indent += 1;
        for body_stmt in body {
            self.emit_stmt(body_stmt);
        }
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
        let popped = self.loop_else_stack.pop();
        debug_assert!(popped.is_some(), "loop_else_stack should not underflow");

        if let Some(else_body) = else_body {
            self.write_indent();
            self.write("if !_broke {\n");
            self.indent += 1;
            for else_stmt in else_body {
                self.emit_stmt(else_stmt);
            }
            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
        }

        Ok(true)
    }

    pub(crate) fn try_emit_structured_with_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::With { items, body } = stmt else {
            return Ok(false);
        };

        let output_len = self.output.len();
        self.write_indent();
        self.write("{\n");
        self.indent += 1;

        for (idx, (var, value, has_cm)) in items.iter().enumerate() {
            let ctx_name = format!("__ctx_{idx}");
            let guard_type = format!("__WithGuard{idx}");
            let guard_var = format!("__guard_{idx}");
            if *has_cm {
                let Type::Class {
                    name: class_name, ..
                } = value.ty()
                else {
                    self.indent -= 1;
                    self.output.truncate(output_len);
                    return Ok(false);
                };

                self.write_indent();
                self.write("let mut ");
                self.write(&ctx_name);
                self.write(" = ");
                if !self.try_emit_structured_expr(value)? {
                    self.indent -= 1;
                    self.output.truncate(output_len);
                    return Ok(false);
                }
                self.write(";\n");

                self.write_indent();
                self.write(&format!("struct {guard_type} {{ ctx: {class_name} }}\n"));
                self.write_indent();
                self.write(&format!("impl Drop for {guard_type} {{\n"));
                self.indent += 1;
                self.write_indent();
                self.write("fn drop(&mut self) { self.ctx.__exit__(); }\n");
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");

                self.write_indent();
                self.write(&format!(
                    "let mut {guard_var} = {guard_type} {{ ctx: {ctx_name} }};\n"
                ));

                self.write_indent();
                if crate::helpers::stmts_reference_var(body, var)
                    || items
                        .iter()
                        .any(|(other_var, _, _)| other_var != var && other_var.contains(var))
                {
                    self.write("let ");
                    self.write(var);
                } else {
                    self.write("let _");
                    self.write(var);
                }
                self.write(" = ");
                self.write(&guard_var);
                self.write(".ctx.__enter__();\n");
            } else {
                self.write_indent();
                if crate::helpers::stmts_reference_var(body, var) {
                    self.write("let ");
                    self.write(var);
                } else {
                    self.write("let _");
                    self.write(var);
                }
                self.write(" = ");
                if !self.try_emit_structured_expr(value)? {
                    self.indent -= 1;
                    self.output.truncate(output_len);
                    return Ok(false);
                }
                self.write(";\n");
            }
        }

        for body_stmt in body {
            self.emit_stmt(body_stmt);
        }
        self.indent -= 1;
        self.write_indent();
        self.write("}\n");
        Ok(true)
    }

    pub(crate) fn try_emit_structured_try_except_stmt(&mut self, stmt: &HirStmt) -> bool {
        let HirStmt::TryExcept { body, handlers, .. } = stmt else {
            return false;
        };
        if handlers.is_empty() {
            return false;
        }
        let err_ty = select_try_error_type(handlers);
        let capture_returns = body_contains_return_stmt(body) && self.current_return_type.is_some();
        let direct_return_capture = capture_returns
            && body_always_exits_stmt(body)
            && handlers
                .iter()
                .all(|handler| body_always_exits_stmt(&handler.body));
        let ok_ty = if capture_returns {
            if let Some(return_ty) = self.current_return_type.as_ref() {
                if direct_return_capture {
                    crate::render_type(&crate::sifr_type_to_rust_type(return_ty))
                } else {
                    format!(
                        "Option<{}>",
                        crate::render_type(&crate::sifr_type_to_rust_type(return_ty))
                    )
                }
            } else {
                "()".to_string()
            }
        } else {
            "()".to_string()
        };

        let output_len = self.output.len();
        self.write_indent();
        self.write("let __sifr_try_res: Result<");
        self.write(&ok_ty);
        self.write(", ");
        self.write(&err_ty);
        self.write("> = (|| {\n");
        self.indent += 1;
        if capture_returns {
            self.try_closure_depth += 1;
            self.try_closure_option_wrap.push(!direct_return_capture);
        }
        self.try_closure_error_type.push(err_ty.clone());
        for try_stmt in body {
            self.emit_stmt(try_stmt);
        }
        self.write_indent();
        if !capture_returns {
            self.write("return Ok(());\n");
        } else if direct_return_capture {
            self.write("unreachable!(\"sifr try/except return capture fell through\");\n");
        } else {
            self.write("return Ok(None);\n");
        }
        if capture_returns {
            self.try_closure_depth -= 1;
            self.try_closure_option_wrap.pop();
        }
        self.try_closure_error_type.pop();
        self.indent -= 1;
        self.write_indent();
        self.write("})();\n");

        if capture_returns {
            self.write_indent();
            self.write("match __sifr_try_res {\n");
            self.indent += 1;
            self.write_indent();
            if direct_return_capture {
                self.write("Ok(__sifr_ret_val) => {\n");
            } else {
                self.write("Ok(Some(__sifr_ret_val)) => {\n");
            }
            self.indent += 1;
            self.write_indent();
            self.write("return __sifr_ret_val;\n");
            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
            if !direct_return_capture {
                self.write_indent();
                self.write("Ok(None) => {}\n");
            }
            self.write_indent();
            self.write("Err(__sifr_try_err) => {\n");
            self.indent += 1;
            self.emit_try_except_handler_chain(handlers, "__sifr_try_err", &err_ty);
            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
        } else {
            self.write_indent();
            self.write("if let Err(__sifr_try_err) = __sifr_try_res {\n");
            self.indent += 1;
            self.emit_try_except_handler_chain(handlers, "__sifr_try_err", &err_ty);
            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
        }

        if !self
            .output
            .get(output_len..)
            .is_some_and(|segment| !segment.is_empty())
        {
            self.output.truncate(output_len);
            return false;
        }
        true
    }

    fn try_except_handler_condition_expr(
        handler: &HirExceptHandler,
        err_ident: &str,
        err_ty: &str,
    ) -> HandlerMatchCondition {
        let Some(error_type) = handler.error_type.as_deref() else {
            return HandlerMatchCondition::Always;
        };
        if error_type == "Error" {
            return HandlerMatchCondition::Always;
        }
        if err_ty == "IOError" {
            if error_type == "IOError" {
                return HandlerMatchCondition::Always;
            }
            if let Some(kind) = io_error_kind_for_handler(error_type) {
                return HandlerMatchCondition::Expr(format!(
                    "{err_ident}.kind == {kind:?}.to_string()"
                ));
            }
            return HandlerMatchCondition::Unsupported;
        }
        if error_type == err_ty {
            return HandlerMatchCondition::Always;
        }
        HandlerMatchCondition::Unsupported
    }

    fn emit_try_except_handler_chain(
        &mut self,
        handlers: &[HirExceptHandler],
        err_ident: &str,
        err_ty: &str,
    ) {
        let mut emitted_any = false;
        for handler in handlers {
            let condition = Self::try_except_handler_condition_expr(handler, err_ident, err_ty);
            if matches!(condition, HandlerMatchCondition::Unsupported) {
                continue;
            }
            self.write_indent();
            if !emitted_any {
                if let HandlerMatchCondition::Expr(condition) = &condition {
                    self.write("if ");
                    self.write(condition);
                    self.write(" {\n");
                } else {
                    self.write("{\n");
                }
            } else if let HandlerMatchCondition::Expr(condition) = &condition {
                self.write("else if ");
                self.write(condition);
                self.write(" {\n");
            } else {
                self.write("else {\n");
            }
            emitted_any = true;
            self.indent += 1;

            let handler_name = handler.name.as_deref().unwrap_or("_e");
            if handler_name != "_" {
                self.write_indent();
                self.write("let ");
                self.write(handler_name);
                self.write(" = ");
                self.write(err_ident);
                self.write(".clone();\n");
            }
            for handler_stmt in &handler.body {
                self.emit_stmt(handler_stmt);
            }
            self.indent -= 1;
            self.write_indent();
            self.write("}\n");
        }
        if !emitted_any {
            self.write_indent();
            self.write("let _ = &");
            self.write(err_ident);
            self.write(";\n");
        }
    }

    pub(crate) fn try_emit_structured_field_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::FieldAssign {
            object,
            field,
            value,
        } = stmt
        else {
            return Ok(false);
        };

        let target = crate::RustExpr::Field {
            expr: Box::new(Self::object_name_expr_for_ir(object)),
            field: field.clone(),
        };

        if self.current_class_name.as_deref() == Some("deque") && field == "_data" {
            if let HirExpr::ListLiteral { elements, .. } = value {
                let value_expr = if elements.is_empty() {
                    crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "VecDeque".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![],
                    }
                } else {
                    let Some(list_expr) = self.lower_stmt_expr_for_ir(value)? else {
                        return Ok(false);
                    };
                    crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "VecDeque".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![list_expr],
                    }
                };
                let lowered = crate::RustStmt::Assign {
                    target,
                    value: value_expr,
                };
                self.emit_lowered_stmts(std::slice::from_ref(&lowered));
                return Ok(true);
            }
        }

        let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(false);
        };
        let lowered = crate::RustStmt::Assign {
            target,
            value: value_expr,
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn try_emit_structured_attribute_subscript_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::AttributeSubscriptAssign {
            object,
            field,
            index,
            value,
            field_ty,
        } = stmt
        else {
            return Ok(false);
        };
        let Type::Dict(key_ty, _) = field_ty else {
            return Ok(false);
        };

        let key_needs_clone = matches!(key_ty.as_ref(), Type::Str | Type::TypeVar(_))
            && matches!(index, HirExpr::Name { name, .. }
                if self.borrowed_params.contains(name.as_str()) || self.mut_borrowed_params.contains(name.as_str()));

        let Some(mut index_expr) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(false);
        };
        if key_needs_clone {
            index_expr = crate::RustExpr::Clone(Box::new(index_expr));
        }
        let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(false);
        };

        let receiver = crate::RustExpr::Field {
            expr: Box::new(Self::object_name_expr_for_ir(object)),
            field: field.clone(),
        };
        let lowered = crate::RustStmt::Expr(crate::RustExpr::MethodCall {
            receiver: Box::new(receiver),
            method: "insert".to_string(),
            args: vec![index_expr, value_expr],
        });
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn try_emit_structured_assert_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::Assert { test, msg } = stmt else {
            return Ok(false);
        };

        let output_len = self.output.len();
        self.write_indent();
        self.write("assert!(");
        if !self.try_emit_structured_expr(test)? {
            self.output.truncate(output_len);
            return Ok(false);
        }
        if let Some(msg_expr) = msg {
            self.write(", ");
            if !self.try_emit_structured_expr(msg_expr)? {
                self.output.truncate(output_len);
                return Ok(false);
            }
        }
        self.write(");\n");
        Ok(true)
    }

    pub(crate) fn try_emit_structured_aug_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::AugAssign { name, op, value } = stmt else {
            return Ok(false);
        };
        let value_ty = Self::resolve_alias_type_for_loop_iter(value.ty());

        if op == "+=" {
            match value_ty {
                Type::Str => {
                    let arg_expr = if let HirExpr::StringLiteral(val) = value {
                        crate::RustExpr::Ident(format!("{val:?}"))
                    } else {
                        let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                            return Ok(false);
                        };
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(value_expr))),
                            method: "as_str".to_string(),
                            args: vec![],
                        }
                    };
                    let lowered = crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                        method: "push_str".to_string(),
                        args: vec![arg_expr],
                    });
                    self.emit_lowered_stmts(std::slice::from_ref(&lowered));
                    return Ok(true);
                }
                Type::List(_) => {
                    let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                        return Ok(false);
                    };
                    let lowered = crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                        method: "extend".to_string(),
                        args: vec![value_expr],
                    });
                    self.emit_lowered_stmts(std::slice::from_ref(&lowered));
                    return Ok(true);
                }
                _ => {}
            }
        }

        if op == "**=" {
            let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                return Ok(false);
            };
            let pow_value = crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Paren(Box::new(crate::RustExpr::Ident(
                    name.clone(),
                )))),
                method: "pow".to_string(),
                args: vec![crate::RustExpr::Cast {
                    expr: Box::new(value_expr),
                    ty: crate::RustType::Named("u32".to_string()),
                }],
            };
            let lowered = crate::RustStmt::Assign {
                target: crate::RustExpr::Ident(name.clone()),
                value: pow_value,
            };
            self.emit_lowered_stmts(std::slice::from_ref(&lowered));
            return Ok(true);
        }
        let rust_op = match op.as_str() {
            "+=" => "+=",
            "-=" => "-=",
            "*=" => "*=",
            "/=" | "//=" => "/=",
            "%=" => "%=",
            _ => return Ok(false),
        };

        let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(false);
        };
        let lowered = crate::RustStmt::AugAssign {
            target: crate::RustExpr::Ident(name.clone()),
            op: rust_op.to_string(),
            value: value_expr,
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }
}
