use crate::{RustEmitter, RustExpr, RustStmt};
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
    Expr(RustExpr),
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

    fn collect_stmt_string_concat_parts_for_ir<'a>(
        expr: &'a HirExpr,
        parts: &mut Vec<&'a HirExpr>,
    ) {
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
            return Ok(Some(crate::RustExpr::Literal(crate::RustLiteral::Str(
                combined,
            ))));
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
        if matches!(expr, HirExpr::Slice { .. }) {
            let saved_stats = self.lowering_stats;
            let Some(rendered_slice) = self.try_render_structured_expr(expr)? else {
                return Ok(None);
            };
            self.lowering_stats = saved_stats;
            return Ok(Some(crate::RustExpr::RawCode(rendered_slice)));
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
        if let HirExpr::ContainsOp {
            element,
            collection,
            ..
        } = expr
        {
            let Some(lowered_element) = self.lower_stmt_expr_for_ir(element)? else {
                return Ok(None);
            };
            let Some(lowered_collection) = self.lower_stmt_expr_for_ir(collection)? else {
                return Ok(None);
            };
            let lowered = match crate::resolve_alias_type_for_plain_call(collection.ty()) {
                Type::Dict(_, _) => {
                    let key_arg = if let HirExpr::StringLiteral(value) = element.as_ref() {
                        crate::RustExpr::Literal(crate::RustLiteral::Str(value.clone()))
                    } else if let HirExpr::Name { name, ty } = element.as_ref() {
                        if self.borrowed_params.contains(name)
                            || self.mut_borrowed_params.contains(name)
                        {
                            if matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Str) {
                                crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                        lowered_element,
                                    ))),
                                    method: "as_str".to_string(),
                                    args: vec![],
                                }
                            } else {
                                lowered_element
                            }
                        } else {
                            crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_element))),
                            }
                        }
                    } else {
                        crate::RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_element))),
                        }
                    };
                    crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_collection))),
                        method: "contains_key".to_string(),
                        args: vec![key_arg],
                    }
                }
                Type::List(_) | Type::Set(_) | Type::Str => crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_collection))),
                    method: "contains".to_string(),
                    args: vec![crate::RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_element))),
                    }],
                },
                _ => return Ok(None),
            };
            return Ok(Some(lowered));
        }
        if let HirExpr::UnaryOp { op, operand, .. } = expr {
            let Some(lowered_operand) = self.lower_stmt_expr_for_ir(operand)? else {
                return Ok(None);
            };
            let lowered = match op.as_str() {
                "not" => crate::RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(crate::RustExpr::Paren(Box::new(lowered_operand))),
                },
                "-" => crate::RustExpr::UnaryOp {
                    op: "-".to_string(),
                    operand: Box::new(crate::RustExpr::Paren(Box::new(lowered_operand))),
                },
                "+" => crate::RustExpr::Paren(Box::new(lowered_operand)),
                _ => return Ok(None),
            };
            return Ok(Some(lowered));
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

    fn borrowed_return_name_clone_expr_for_ir(&self, value: &HirExpr) -> Option<crate::RustExpr> {
        let HirExpr::Name { name, .. } = value else {
            return None;
        };
        if !(self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name)) {
            return None;
        }
        Some(crate::RustExpr::Clone(Box::new(crate::RustExpr::Ident(
            name.clone(),
        ))))
    }

    fn lower_non_option_index_return_expr_for_ir(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let object_ty = crate::resolve_alias_type_for_plain_call(object.ty());
        if !matches!(
            object_ty,
            Type::Tuple(_) | Type::Dict(_, _) | Type::List(_) | Type::Str
        ) {
            return Ok(None);
        }

        let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
            return Ok(None);
        };
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };

        let lowered = match object_ty {
            Type::Tuple(elements) => {
                let HirExpr::IntLiteral(raw_idx) = index else {
                    return Ok(None);
                };
                let Ok(idx) = usize::try_from(*raw_idx) else {
                    return Ok(None);
                };
                if idx >= elements.len() {
                    return Ok(None);
                }
                crate::RustExpr::Field {
                    expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_object))),
                    field: idx.to_string(),
                }
            }
            Type::Dict(_, _) => {
                let lowered_key = if matches!(index, HirExpr::StringLiteral(_)) {
                    lowered_index
                } else {
                    crate::RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_index))),
                    }
                };
                crate::RustExpr::Clone(Box::new(crate::RustExpr::Index {
                    expr: Box::new(lowered_object),
                    index: Box::new(lowered_key),
                }))
            }
            Type::List(_) => crate::RustExpr::Clone(Box::new(crate::RustExpr::Index {
                expr: Box::new(lowered_object),
                index: Box::new(crate::RustExpr::Cast {
                    expr: Box::new(lowered_index),
                    ty: crate::RustType::Named("usize".to_string()),
                }),
            })),
            Type::Str => crate::RustExpr::MethodCall {
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
                method: "unwrap".to_string(),
                args: vec![],
            },
            _ => return Ok(None),
        };

        if matches!(object_ty, Type::Str) {
            return Ok(Some(crate::RustExpr::MethodCall {
                receiver: Box::new(lowered),
                method: "to_string".to_string(),
                args: vec![],
            }));
        }
        Ok(Some(lowered))
    }

    fn lower_return_value_expr_for_ir(
        &mut self,
        value: &HirExpr,
        return_ty: Option<&Type>,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if self.current_class_name.is_some()
            && matches!(value, HirExpr::Name { name, .. } if name == "self")
        {
            return Ok(Some(crate::RustExpr::Clone(Box::new(
                crate::RustExpr::Ident("self".to_string()),
            ))));
        }

        if let Some(clone_expr) = self.borrowed_return_name_clone_expr_for_ir(value) {
            return Ok(Some(clone_expr));
        }

        if return_ty.is_some_and(|ty| !crate::helpers::is_option_type(ty))
            && matches!(value, HirExpr::Index { .. })
        {
            let HirExpr::Index { object, index, .. } = value else {
                unreachable!();
            };
            if let Some(lowered) = self.lower_non_option_index_return_expr_for_ir(object, index)? {
                return Ok(Some(lowered));
            }
        }

        if let Some(lowered_leaf) = crate::try_lower_leaf_or_name_expr_result(value)? {
            return Ok(Some(lowered_leaf));
        }

        let saved_stats = self.lowering_stats;
        let Some(rendered_value) = self.try_render_structured_expr(value)? else {
            return Ok(None);
        };
        self.lowering_stats = saved_stats;
        Ok(Some(crate::RustExpr::RawCode(rendered_value)))
    }

    pub(super) fn lower_rendered_expr_for_ir(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let Some(lowered_leaf) = crate::try_lower_leaf_or_name_expr_result(expr)? {
            return Ok(Some(lowered_leaf));
        }
        let saved_stats = self.lowering_stats;
        let Some(rendered_expr) = self.try_render_structured_expr(expr)? else {
            return Ok(None);
        };
        self.lowering_stats = saved_stats;
        Ok(Some(crate::RustExpr::RawCode(rendered_expr)))
    }

    fn try_lower_stmt_block_for_ir(
        &mut self,
        stmts: &[HirStmt],
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let scope_ctx = crate::ScopeContext {
            function_return_type: self.current_return_type.clone(),
            in_generator_closure: self.emission_ctx.in_generator_closure,
            in_display_impl: self.emission_ctx.in_display_impl,
            in_loop_with_else: self.current_loop_has_else(),
            class_scope: if self.current_class_name.is_some() {
                crate::ClassScope::Inside
            } else {
                crate::ClassScope::Outside
            },
        };

        let mut lowered_block = Vec::new();
        for stmt in stmts {
            let (lowered_stmts, skip_rewrite) = if let Some(lowered_stmts) =
                crate::try_lower_simple_stmt_with_scope_result(
                    stmt,
                    &self.mutated_vars,
                    &self.borrowed_params,
                    &scope_ctx,
                )? {
                (lowered_stmts, false)
            } else if let HirStmt::Let {
                name, ty, value, ..
            } = stmt
            {
                let is_generic_class = matches!(ty, Type::Class { name: class_name, .. } if self.generic_classes.contains(class_name));
                let lowered_value = if ty.ownership() == sifr_type_system::OwnershipKind::Move {
                    if let HirExpr::Name {
                        name: value_name, ..
                    } = value
                    {
                        if self.borrowed_params.contains(value_name)
                            || self.mut_borrowed_params.contains(value_name)
                        {
                            Some(crate::RustExpr::Clone(Box::new(crate::RustExpr::Ident(
                                value_name.clone(),
                            ))))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                let lowered_value = if let Some(clone_expr) = lowered_value {
                    clone_expr
                } else {
                    let Some(lowered) = self.lower_rendered_expr_for_ir(value)? else {
                        return Ok(None);
                    };
                    lowered
                };
                (
                    vec![RustStmt::Let {
                        mutable: self.mutated_vars.contains(name),
                        name: name.clone(),
                        ty: if is_generic_class {
                            None
                        } else {
                            Some(crate::sifr_type_to_rust_type(ty))
                        },
                        value: lowered_value,
                    }],
                    true,
                )
            } else if let HirStmt::Assign { name, value } = stmt {
                let Some(lowered_value) = self.lower_rendered_expr_for_ir(value)? else {
                    return Ok(None);
                };
                (
                    vec![RustStmt::Assign {
                        target: crate::RustExpr::Ident(name.clone()),
                        value: lowered_value,
                    }],
                    true,
                )
            } else if let HirStmt::AttributeSubscriptAssign {
                object,
                field,
                index,
                value,
                field_ty,
            } = stmt
            {
                let Type::Dict(key_ty, _) = field_ty else {
                    return Ok(None);
                };

                let key_needs_clone = matches!(key_ty.as_ref(), Type::Str | Type::TypeVar(_))
                    && matches!(index, HirExpr::Name { name, .. }
                            if self.borrowed_params.contains(name.as_str()) || self.mut_borrowed_params.contains(name.as_str()));

                let Some(mut index_expr) = self.lower_rendered_expr_for_ir(index)? else {
                    return Ok(None);
                };
                if key_needs_clone {
                    index_expr = crate::RustExpr::Clone(Box::new(index_expr));
                }
                let Some(value_expr) = self.lower_rendered_expr_for_ir(value)? else {
                    return Ok(None);
                };

                let receiver = crate::RustExpr::Field {
                    expr: Box::new(Self::object_name_expr_for_ir(object)),
                    field: field.clone(),
                };
                (
                    vec![crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(receiver),
                        method: "insert".to_string(),
                        args: vec![index_expr, value_expr],
                    })],
                    true,
                )
            } else if let HirStmt::FieldAssign {
                object,
                field,
                value,
            } = stmt
            {
                let target = crate::RustExpr::Field {
                    expr: Box::new(Self::object_name_expr_for_ir(object)),
                    field: field.clone(),
                };
                let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                    return Ok(None);
                };
                (
                    vec![RustStmt::Assign {
                        target,
                        value: value_expr,
                    }],
                    true,
                )
            } else if let HirStmt::Assert { test, msg } = stmt {
                let Some(lowered_test) = self.lower_rendered_expr_for_ir(test)? else {
                    return Ok(None);
                };
                let lowered_msg = if let Some(msg_expr) = msg {
                    let Some(lowered) = self.lower_rendered_expr_for_ir(msg_expr)? else {
                        return Ok(None);
                    };
                    Some(lowered)
                } else {
                    None
                };
                (
                    vec![RustStmt::Assert {
                        cond: lowered_test,
                        msg: lowered_msg,
                    }],
                    true,
                )
            } else if let HirStmt::Expr { expr } = stmt {
                let Some(lowered_expr) = self.lower_rendered_expr_for_ir(expr)? else {
                    return Ok(None);
                };
                (vec![RustStmt::Expr(lowered_expr)], true)
            } else if let HirStmt::Return { value } = stmt {
                let return_ty_snapshot = self.current_return_type.clone();
                let lowered_return_stmt = if let Some(value) = value {
                    if self.emission_ctx.in_display_impl && self.try_closure_depth == 0 {
                        let Some(display_expr) = self
                            .lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
                        else {
                            return Ok(None);
                        };
                        RustStmt::Return(Some(crate::RustExpr::MacroCall {
                            name: "write".to_string(),
                            args: vec![
                                crate::RustExpr::Ident("f".to_string()),
                                crate::RustExpr::Literal(crate::RustLiteral::Str("{}".to_string())),
                                display_expr,
                            ],
                        }))
                    } else if self.try_closure_depth > 0 {
                        let wrap_option = self
                            .try_closure_option_wrap
                            .last()
                            .copied()
                            .unwrap_or(false);
                        let Some(mut lowered_return_value) = self
                            .lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
                        else {
                            return Ok(None);
                        };

                        if !wrap_option {
                            if let Some(return_ty) = return_ty_snapshot.as_ref() {
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
                                            crate::resolve_alias_type_for_plain_call(
                                                ok_ty.as_ref()
                                            ),
                                            Type::None
                                        )
                                    {
                                        lowered_return_value = crate::RustExpr::FnCall {
                                            func: Box::new(crate::RustExpr::Path(vec![
                                                "Ok".to_string()
                                            ])),
                                            args: vec![crate::RustExpr::Literal(
                                                crate::RustLiteral::Unit,
                                            )],
                                        };
                                    }
                                }
                            }
                        }

                        let try_payload = if wrap_option {
                            crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                                args: vec![lowered_return_value],
                            }
                        } else {
                            lowered_return_value
                        };
                        RustStmt::Return(Some(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                            args: vec![try_payload],
                        }))
                    } else {
                        let Some(lowered_return_value) = self
                            .lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
                        else {
                            return Ok(None);
                        };
                        RustStmt::Return(Some(lowered_return_value))
                    }
                } else if self.try_closure_depth > 0 {
                    let wrap_option = self
                        .try_closure_option_wrap
                        .last()
                        .copied()
                        .unwrap_or(false);
                    if wrap_option {
                        RustStmt::Return(Some(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                            args: vec![crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                                args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                            }],
                        }))
                    } else {
                        let direct_result_none =
                            return_ty_snapshot.as_ref().is_some_and(|ret_ty| {
                                match crate::resolve_alias_type_for_plain_call(ret_ty) {
                                    Type::Result(ok_ty, _) => matches!(
                                        crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
                                        Type::None
                                    ),
                                    _ => false,
                                }
                            });
                        if direct_result_none {
                            RustStmt::Return(Some(crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                                args: vec![crate::RustExpr::FnCall {
                                    func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                                }],
                            }))
                        } else {
                            RustStmt::Return(Some(crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                                args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                            }))
                        }
                    }
                } else if self.emission_ctx.in_display_impl {
                    RustStmt::Return(Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                        args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                    }))
                } else {
                    RustStmt::Return(None)
                };
                (vec![lowered_return_stmt], true)
            } else if let HirStmt::Raise { value } = stmt {
                let Some(lowered) = self.lower_stmt_expr_for_ir(value)? else {
                    return Ok(None);
                };
                (
                    vec![RustStmt::Return(Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Err".to_string()])),
                        args: vec![lowered],
                    }))],
                    true,
                )
            } else if let HirStmt::If {
                condition,
                then_body,
                elif_clauses,
                else_body,
            } = stmt
            {
                let Some(lowered_if_stmt) = self.try_lower_if_stmt_for_ir(
                    condition,
                    then_body,
                    elif_clauses,
                    else_body.as_deref(),
                )?
                else {
                    return Ok(None);
                };
                (vec![lowered_if_stmt], true)
            } else if let HirStmt::While {
                condition,
                body,
                else_body,
            } = stmt
            {
                if else_body.is_some() {
                    return Ok(None);
                }
                let Some(lowered_cond) = self.lower_condition_expr_for_ir(condition)? else {
                    return Ok(None);
                };
                let Some(lowered_body) = self.try_lower_stmt_block_for_ir(body)? else {
                    return Ok(None);
                };
                (
                    vec![RustStmt::While {
                        cond: lowered_cond,
                        body: lowered_body,
                    }],
                    true,
                )
            } else if let HirStmt::For {
                target,
                iter,
                body,
                else_body,
                ..
            } = stmt
            {
                if else_body.is_some() {
                    return Ok(None);
                }
                let Some(lowered_iter) = self.try_lower_for_iter_expr_for_ir(iter)? else {
                    return Ok(None);
                };
                let Some(lowered_body) = self.try_lower_stmt_block_for_ir(body)? else {
                    return Ok(None);
                };
                let var = if target.contains(',') {
                    let names = target
                        .split(',')
                        .map(str::trim)
                        .filter(|name| !name.is_empty())
                        .collect::<Vec<_>>();
                    if names.is_empty() {
                        return Ok(None);
                    }
                    format!("({})", names.join(", "))
                } else {
                    target.clone()
                };
                (
                    vec![RustStmt::For {
                        var,
                        iter: lowered_iter,
                        body: lowered_body,
                    }],
                    true,
                )
            } else if let HirStmt::With { items, body } = stmt {
                let Some(lowered_with) = self.try_lower_with_stmt_for_ir(items, body)? else {
                    return Ok(None);
                };
                (vec![lowered_with], true)
            } else if let HirStmt::TryExcept { body, handlers, .. } = stmt {
                let Some(lowered_try_except) =
                    self.try_lower_try_except_stmt_for_ir(body, handlers)?
                else {
                    return Ok(None);
                };
                (lowered_try_except, true)
            } else {
                return Ok(None);
            };
            if skip_rewrite {
                lowered_block.extend(lowered_stmts);
            } else {
                lowered_block.extend(
                    lowered_stmts
                        .into_iter()
                        .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt)),
                );
            }
        }
        Ok(Some(lowered_block))
    }

    fn lower_condition_expr_for_ir(
        &mut self,
        condition: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let Some(lowered) = self.try_lower_borrowed_name_compare_condition_for_ir(condition) {
            return Ok(Some(lowered));
        }
        if self.condition_uses_borrowed_name_for_ir(condition) {
            let saved_stats = self.lowering_stats;
            let Some(rendered_expr) = self.try_render_structured_expr(condition)? else {
                return Ok(None);
            };
            self.lowering_stats = saved_stats;
            return Ok(Some(crate::RustExpr::RawCode(rendered_expr)));
        }
        self.lower_rendered_expr_for_ir(condition)
    }

    fn try_lower_for_iter_expr_for_ir(
        &mut self,
        iter: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let HirExpr::Call { func, args, .. } = iter {
            if func == "enumerate" && args.len() == 1 {
                let Some(lowered_arg) = self.lower_rendered_expr_for_ir(&args[0])? else {
                    return Ok(None);
                };
                let iter_source = match Self::resolve_alias_type_for_loop_iter(args[0].ty()) {
                    Type::List(_) => crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_arg),
                            method: "iter".to_string(),
                            args: vec![],
                        }),
                        method: "cloned".to_string(),
                        args: vec![],
                    },
                    Type::Dict(_, _) => crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_arg),
                            method: "keys".to_string(),
                            args: vec![],
                        }),
                        method: "cloned".to_string(),
                        args: vec![],
                    },
                    Type::Str => crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_arg),
                            method: "chars".to_string(),
                            args: vec![],
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
                    _ => lowered_arg,
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(iter_source),
                        method: "enumerate".to_string(),
                        args: vec![],
                    }),
                    method: "map".to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "(i, v)".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(crate::RustExpr::Tuple(vec![
                            crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::Ident("i".to_string())),
                                ty: crate::RustType::I64,
                            },
                            crate::RustExpr::Ident("v".to_string()),
                        ])),
                        is_move: false,
                    }],
                }));
            }
        }

        let Some(lowered_iter) = self.lower_rendered_expr_for_ir(iter)? else {
            return Ok(None);
        };
        let is_generator_expr = matches!(iter, HirExpr::GeneratorExpr { .. });
        let is_generator_fn_call = self.is_generator_call(iter);
        if is_generator_expr
            || is_generator_fn_call
            || Self::is_iterator_like_expr_for_ir(&lowered_iter)
        {
            return Ok(Some(lowered_iter));
        }
        let lowered_iter = match Self::resolve_alias_type_for_loop_iter(iter.ty()) {
            Type::List(_) => crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_iter),
                    method: "iter".to_string(),
                    args: vec![],
                }),
                method: "cloned".to_string(),
                args: vec![],
            },
            Type::Dict(_, _) => crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_iter),
                    method: "keys".to_string(),
                    args: vec![],
                }),
                method: "cloned".to_string(),
                args: vec![],
            },
            Type::Str => crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_iter),
                    method: "chars".to_string(),
                    args: vec![],
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
            _ => lowered_iter,
        };
        Ok(Some(lowered_iter))
    }

    fn is_iterator_like_expr_for_ir(expr: &crate::RustExpr) -> bool {
        match expr {
            crate::RustExpr::RawCode(code) => {
                code.contains(".into_iter()")
                    || code.contains(".map(")
                    || code.contains(".filter(")
                    || code.contains(".zip(")
                    || code.contains(".chain(")
                    || code.contains(".enumerate(")
            }
            crate::RustExpr::MethodCall {
                receiver, method, ..
            } => {
                matches!(
                    method.as_str(),
                    "into_iter" | "map" | "filter" | "zip" | "chain" | "enumerate"
                ) || Self::is_iterator_like_expr_for_ir(receiver)
            }
            crate::RustExpr::FnCall { func, args } => {
                Self::is_iterator_like_expr_for_ir(func)
                    || args.iter().any(Self::is_iterator_like_expr_for_ir)
            }
            crate::RustExpr::Paren(inner)
            | crate::RustExpr::Try(inner)
            | crate::RustExpr::Await(inner)
            | crate::RustExpr::Deref(inner)
            | crate::RustExpr::Clone(inner) => Self::is_iterator_like_expr_for_ir(inner),
            _ => false,
        }
    }

    fn try_lower_with_stmt_for_ir(
        &mut self,
        items: &[(String, HirExpr, bool)],
        body: &[HirStmt],
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let mut lowered_items = Vec::with_capacity(items.len());
        for (var, value, has_cm) in items {
            let Some(lowered_value) = self.lower_rendered_expr_for_ir(value)? else {
                return Ok(None);
            };
            let binding = if crate::helpers::stmts_reference_var(body, var)
                || items
                    .iter()
                    .any(|(other_var, _, _)| other_var != var && other_var.contains(var))
            {
                var.clone()
            } else {
                format!("_{var}")
            };
            let class_name = if *has_cm {
                let Type::Class { name, .. } = value.ty() else {
                    return Ok(None);
                };
                Some(name.clone())
            } else {
                None
            };
            lowered_items.push(crate::RustWithItem {
                binding,
                value: lowered_value,
                has_cm: *has_cm,
                class_name,
            });
        }
        let Some(lowered_body) = self.try_lower_stmt_block_for_ir(body)? else {
            return Ok(None);
        };
        Ok(Some(RustStmt::With {
            items: lowered_items,
            body: lowered_body,
        }))
    }

    fn try_lower_borrowed_name_compare_condition_for_ir(
        &self,
        expr: &HirExpr,
    ) -> Option<crate::RustExpr> {
        let HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } = expr
        else {
            return None;
        };
        if ops.len() != 1 || comparators.len() != 1 {
            return None;
        }
        let rhs = comparators.first()?;
        let lowered_op = match ops[0].as_str() {
            "==" | "!=" | "<" | "<=" | ">" | ">=" => ops[0].as_str(),
            "is" => "==",
            "is not" => "!=",
            _ => return None,
        };

        let lower_operand =
            |operand: &HirExpr, emitter: &Self| -> Option<(crate::RustExpr, bool)> {
                let HirExpr::Name { name, .. } = operand else {
                    return None;
                };
                let borrowed = emitter.borrowed_params.contains(name)
                    || emitter.mut_borrowed_params.contains(name);
                let ident = crate::RustExpr::Ident(name.clone());
                let lowered = if borrowed {
                    crate::RustExpr::Deref(Box::new(ident))
                } else {
                    ident
                };
                Some((lowered, borrowed))
            };

        let (lowered_left, left_borrowed) = lower_operand(left, self)?;
        let (lowered_right, right_borrowed) = lower_operand(rhs, self)?;
        if !left_borrowed && !right_borrowed {
            return None;
        }

        Some(crate::RustExpr::BinOp {
            left: Box::new(lowered_left),
            op: lowered_op.to_string(),
            right: Box::new(lowered_right),
        })
    }

    fn condition_uses_borrowed_name_for_ir(&self, expr: &HirExpr) -> bool {
        match expr {
            HirExpr::Name { name, .. } => {
                self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name)
            }
            HirExpr::Compare {
                left, comparators, ..
            } => {
                self.condition_uses_borrowed_name_for_ir(left)
                    || comparators
                        .iter()
                        .any(|expr| self.condition_uses_borrowed_name_for_ir(expr))
            }
            HirExpr::BoolOp { values, .. } => values
                .iter()
                .any(|expr| self.condition_uses_borrowed_name_for_ir(expr)),
            HirExpr::BinOp { left, right, .. } => {
                self.condition_uses_borrowed_name_for_ir(left)
                    || self.condition_uses_borrowed_name_for_ir(right)
            }
            HirExpr::UnaryOp { operand, .. } => self.condition_uses_borrowed_name_for_ir(operand),
            HirExpr::Index { object, index, .. } => {
                self.condition_uses_borrowed_name_for_ir(object)
                    || self.condition_uses_borrowed_name_for_ir(index)
            }
            HirExpr::FieldAccess { object, .. } => self.condition_uses_borrowed_name_for_ir(object),
            HirExpr::MethodCall { object, args, .. } => {
                self.condition_uses_borrowed_name_for_ir(object)
                    || args
                        .iter()
                        .any(|expr| self.condition_uses_borrowed_name_for_ir(expr))
            }
            HirExpr::Call { args, .. } => args
                .iter()
                .any(|expr| self.condition_uses_borrowed_name_for_ir(expr)),
            HirExpr::TupleLiteral { elements, .. } | HirExpr::ListLiteral { elements, .. } => {
                elements
                    .iter()
                    .any(|expr| self.condition_uses_borrowed_name_for_ir(expr))
            }
            HirExpr::DictLiteral { keys, values, .. } => {
                keys.iter()
                    .any(|expr| self.condition_uses_borrowed_name_for_ir(expr))
                    || values
                        .iter()
                        .any(|expr| self.condition_uses_borrowed_name_for_ir(expr))
            }
            HirExpr::SetLiteral { elements, .. } => elements
                .iter()
                .any(|expr| self.condition_uses_borrowed_name_for_ir(expr)),
            HirExpr::IfExpr {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                self.condition_uses_borrowed_name_for_ir(condition)
                    || self.condition_uses_borrowed_name_for_ir(then_expr)
                    || self.condition_uses_borrowed_name_for_ir(else_expr)
            }
            HirExpr::WalrusExpr { value, .. } => self.condition_uses_borrowed_name_for_ir(value),
            HirExpr::GeneratorExpr {
                expr, iter, filter, ..
            } => {
                self.condition_uses_borrowed_name_for_ir(expr)
                    || self.condition_uses_borrowed_name_for_ir(iter)
                    || filter
                        .as_ref()
                        .is_some_and(|cond| self.condition_uses_borrowed_name_for_ir(cond))
            }
            HirExpr::ListComp {
                expr, generators, ..
            } => {
                self.condition_uses_borrowed_name_for_ir(expr)
                    || generators.iter().any(|(_, iter, cond)| {
                        self.condition_uses_borrowed_name_for_ir(iter)
                            || cond
                                .as_ref()
                                .is_some_and(|cond| self.condition_uses_borrowed_name_for_ir(cond))
                    })
            }
            HirExpr::DictComp {
                key_expr,
                val_expr,
                generators,
                ..
            } => {
                self.condition_uses_borrowed_name_for_ir(key_expr)
                    || self.condition_uses_borrowed_name_for_ir(val_expr)
                    || generators.iter().any(|(_, iter, cond)| {
                        self.condition_uses_borrowed_name_for_ir(iter)
                            || cond
                                .as_ref()
                                .is_some_and(|cond| self.condition_uses_borrowed_name_for_ir(cond))
                    })
            }
            HirExpr::SetComp {
                expr, generators, ..
            } => {
                self.condition_uses_borrowed_name_for_ir(expr)
                    || generators.iter().any(|(_, iter, cond)| {
                        self.condition_uses_borrowed_name_for_ir(iter)
                            || cond
                                .as_ref()
                                .is_some_and(|cond| self.condition_uses_borrowed_name_for_ir(cond))
                    })
            }
            HirExpr::RangeLiteral {
                start, end, step, ..
            } => {
                self.condition_uses_borrowed_name_for_ir(start)
                    || self.condition_uses_borrowed_name_for_ir(end)
                    || step
                        .as_ref()
                        .is_some_and(|step| self.condition_uses_borrowed_name_for_ir(step))
            }
            HirExpr::ContainsOp {
                element,
                collection,
                ..
            } => {
                self.condition_uses_borrowed_name_for_ir(element)
                    || self.condition_uses_borrowed_name_for_ir(collection)
            }
            HirExpr::Slice {
                object,
                start,
                stop,
                step,
                ..
            } => {
                self.condition_uses_borrowed_name_for_ir(object)
                    || start
                        .as_ref()
                        .is_some_and(|start| self.condition_uses_borrowed_name_for_ir(start))
                    || stop
                        .as_ref()
                        .is_some_and(|stop| self.condition_uses_borrowed_name_for_ir(stop))
                    || step
                        .as_ref()
                        .is_some_and(|step| self.condition_uses_borrowed_name_for_ir(step))
            }
            HirExpr::Lambda { body, .. } => self.condition_uses_borrowed_name_for_ir(body),
            HirExpr::QuestionMark { expr, .. } => self.condition_uses_borrowed_name_for_ir(expr),
            HirExpr::OkWrap { value, .. } | HirExpr::ErrWrap { value, .. } => {
                self.condition_uses_borrowed_name_for_ir(value)
            }
            HirExpr::SuperCall { args, .. } => args
                .iter()
                .any(|expr| self.condition_uses_borrowed_name_for_ir(expr)),
            _ => false,
        }
    }

    fn try_lower_if_stmt_for_ir(
        &mut self,
        condition: &HirExpr,
        then_body: &[HirStmt],
        elif_clauses: &[(HirExpr, Vec<HirStmt>)],
        else_body: Option<&[HirStmt]>,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let mut nested_else = if let Some(else_body) = else_body {
            let Some(lowered_else) = self.try_lower_stmt_block_for_ir(else_body)? else {
                return Ok(None);
            };
            Some(lowered_else)
        } else {
            None
        };

        for (elif_cond, elif_body) in elif_clauses.iter().rev() {
            let Some(lowered_elif) =
                self.try_lower_if_clause_for_ir(elif_cond, elif_body, nested_else)?
            else {
                return Ok(None);
            };
            nested_else = Some(vec![lowered_elif]);
        }

        self.try_lower_if_clause_for_ir(condition, then_body, nested_else)
    }

    fn try_lower_if_clause_for_ir(
        &mut self,
        condition: &HirExpr,
        then_body: &[HirStmt],
        nested_else: Option<Vec<RustStmt>>,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let Some(lowered_then_body) = self.try_lower_stmt_block_for_ir(then_body)? else {
            return Ok(None);
        };

        if let Some(option_var) = crate::helpers::detect_is_not_none_var(condition) {
            return Ok(Some(RustStmt::IfLet {
                pattern: format!("Some({option_var})"),
                expr: crate::RustExpr::Ident(option_var),
                then_body: lowered_then_body,
                else_body: nested_else,
            }));
        }

        if let Some(option_vars) = crate::helpers::detect_and_not_none_vars(condition) {
            let mut chain_then = lowered_then_body;
            for option_var in option_vars.iter().rev() {
                chain_then = vec![RustStmt::IfLet {
                    pattern: format!("Some({option_var})"),
                    expr: crate::RustExpr::Ident(option_var.clone()),
                    then_body: chain_then,
                    else_body: None,
                }];
            }
            let Some(mut chain_root) = chain_then.into_iter().next() else {
                return Ok(None);
            };
            if let RustStmt::IfLet { else_body, .. } = &mut chain_root {
                *else_body = nested_else;
            }
            return Ok(Some(chain_root));
        }

        if let Some(option_var) = crate::helpers::detect_is_none_var(condition) {
            let Some(lowered_cond) = self.lower_condition_expr_for_ir(condition)? else {
                return Ok(None);
            };
            let lowered_else = nested_else.map(|else_body| {
                vec![RustStmt::IfLet {
                    pattern: format!("Some({option_var})"),
                    expr: crate::RustExpr::Ident(option_var.clone()),
                    then_body: else_body,
                    else_body: None,
                }]
            });
            return Ok(Some(RustStmt::If {
                cond: lowered_cond,
                then_body: lowered_then_body,
                else_body: lowered_else,
            }));
        }

        let Some(lowered_cond) = self.lower_condition_expr_for_ir(condition)? else {
            return Ok(None);
        };
        Ok(Some(RustStmt::If {
            cond: lowered_cond,
            then_body: lowered_then_body,
            else_body: nested_else,
        }))
    }

    pub(super) fn emit_borrowed_return_name_clone_expr(&mut self, value: &HirExpr) -> bool {
        let Some(clone_expr) = self.borrowed_return_name_clone_expr_for_ir(value) else {
            return false;
        };
        self.emit_rust_expr(&clone_expr);
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
                RustStmt::Expr(lowered_expr) => self.emit_rust_stmt_with_current_indent(
                    &crate::RustStmt::Expr(lowered_expr.clone()),
                ),
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
        let return_ty_snapshot = self.current_return_type.clone();

        if let Some(value) = value {
            if self.emission_ctx.in_display_impl && self.try_closure_depth == 0 {
                let Some(display_expr) =
                    self.lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
                else {
                    return Ok(false);
                };
                self.emit_rust_stmt_with_current_indent(&RustStmt::Return(Some(
                    crate::RustExpr::MacroCall {
                        name: "write".to_string(),
                        args: vec![
                            crate::RustExpr::Ident("f".to_string()),
                            crate::RustExpr::Literal(crate::RustLiteral::Str("{}".to_string())),
                            display_expr,
                        ],
                    },
                )));
                return Ok(true);
            }
            if self.try_closure_depth > 0 {
                let wrap_option = self
                    .try_closure_option_wrap
                    .last()
                    .copied()
                    .unwrap_or(false);

                let Some(mut lowered_return_value) =
                    self.lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
                else {
                    return Ok(false);
                };

                if !wrap_option {
                    if let Some(return_ty) = return_ty_snapshot.as_ref() {
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
                                lowered_return_value = crate::RustExpr::FnCall {
                                    func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                                };
                            }
                        }
                    }
                }

                let try_payload = if wrap_option {
                    crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                        args: vec![lowered_return_value],
                    }
                } else {
                    lowered_return_value
                };
                self.emit_rust_stmt_with_current_indent(&RustStmt::Return(Some(
                    crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                        args: vec![try_payload],
                    },
                )));
                return Ok(true);
            }

            let Some(lowered_return_value) =
                self.lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
            else {
                return Ok(false);
            };
            self.emit_rust_stmt_with_current_indent(&RustStmt::Return(Some(lowered_return_value)));
            return Ok(true);
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
                let direct_result_none = return_ty_snapshot.as_ref().is_some_and(|ret_ty| {
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
            self.emit_rust_stmt_with_current_indent(&RustStmt::Return(Some(
                crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                },
            )));
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
                    let Some(lowered_value) = self.lower_rendered_expr_for_ir(value)? else {
                        return Ok(false);
                    };
                    let walrus_compare_expr = HirExpr::Compare {
                        left: Box::new(HirExpr::Name {
                            name: name.clone(),
                            ty: ty.clone(),
                        }),
                        ops: ops.clone(),
                        comparators: comparators.clone(),
                        ty: condition.ty().clone(),
                    };
                    let Some(lowered_cond) =
                        self.lower_rendered_expr_for_ir(&walrus_compare_expr)?
                    else {
                        return Ok(false);
                    };
                    let Some(lowered_then_body) = self.try_lower_stmt_block_for_ir(then_body)?
                    else {
                        return Ok(false);
                    };

                    self.emit_rust_stmt_with_current_indent(&RustStmt::Let {
                        mutable: false,
                        name: name.clone(),
                        ty: None,
                        value: lowered_value,
                    });
                    self.emit_rust_stmt_with_current_indent(&RustStmt::If {
                        cond: lowered_cond,
                        then_body: lowered_then_body,
                        else_body: None,
                    });
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
                let mut nested_else = if let Some(else_body) = else_body {
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
                    let Some(lowered_else_body) = self.try_lower_stmt_block_for_ir(else_body)?
                    else {
                        return Ok(false);
                    };
                    if remaining_variants.len() == 1 {
                        let else_mutated = crate::helpers::collect_mutated_vars(else_body);
                        let else_binding = if else_mutated.contains(&var_name) {
                            format!("mut {var_name}")
                        } else {
                            var_name.clone()
                        };
                        Some(vec![RustStmt::IfLet {
                            pattern: format!(
                                "{enum_name}::{}({else_binding})",
                                remaining_variants[0]
                            ),
                            expr: RustExpr::Ident(var_name.clone()),
                            then_body: lowered_else_body,
                            else_body: Some(vec![RustStmt::Expr(RustExpr::FormatMacro {
                                name: "unreachable".to_string(),
                                format_str:
                                    "sifr union narrowing fell through exhaustive branch chain"
                                        .to_string(),
                                args: vec![],
                            })]),
                        }])
                    } else {
                        Some(lowered_else_body)
                    }
                } else {
                    None
                };

                for (variant_name, body) in branch_specs.iter().rev() {
                    let mutated = crate::helpers::collect_mutated_vars(body);
                    let binding = if mutated.contains(&var_name) {
                        format!("mut {var_name}")
                    } else {
                        var_name.clone()
                    };
                    let Some(lowered_body) = self.try_lower_stmt_block_for_ir(body)? else {
                        return Ok(false);
                    };
                    nested_else = Some(vec![RustStmt::IfLet {
                        pattern: format!("{enum_name}::{variant_name}({binding})"),
                        expr: RustExpr::Ident(var_name.clone()),
                        then_body: lowered_body,
                        else_body: nested_else,
                    }]);
                }

                let Some(root) = nested_else.and_then(|stmts| stmts.into_iter().next()) else {
                    return Ok(false);
                };
                self.emit_rust_stmt_with_current_indent(&root);
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

                let then_mutated = crate::helpers::collect_mutated_vars(then_body);
                let then_binding = if then_mutated.contains(&var_name) {
                    format!("mut {var_name}")
                } else {
                    var_name.clone()
                };
                let Some(lowered_then_body) = self.try_lower_stmt_block_for_ir(then_body)? else {
                    return Ok(false);
                };

                let mut arms = vec![crate::RustMatchArm {
                    pattern: format!("{enum_name}::{variant_name}({then_binding})"),
                    bindings: vec![],
                    guard: None,
                    body: lowered_then_body,
                }];

                if let Some(else_body) = else_body {
                    let else_mutated = crate::helpers::collect_mutated_vars(else_body);
                    let else_binding = if else_mutated.contains(&var_name) {
                        format!("mut {var_name}")
                    } else {
                        var_name.clone()
                    };
                    let Some(lowered_else_body) = self.try_lower_stmt_block_for_ir(else_body)?
                    else {
                        return Ok(false);
                    };
                    if other_variants.len() == 1 {
                        let (other_variant, _) = &other_variants[0];
                        arms.push(crate::RustMatchArm {
                            pattern: format!("{enum_name}::{other_variant}({else_binding})"),
                            bindings: vec![],
                            guard: None,
                            body: lowered_else_body,
                        });
                    } else {
                        arms.push(crate::RustMatchArm {
                            pattern: "_".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: lowered_else_body,
                        });
                    }
                } else {
                    arms.push(crate::RustMatchArm {
                        pattern: "_".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![],
                    });
                }

                self.emit_rust_stmt_with_current_indent(&RustStmt::Match {
                    expr: RustExpr::Ident(var_name),
                    arms,
                });
                return Ok(true);
            }
        }

        let Some(lowered_if_stmt) = self.try_lower_if_stmt_for_ir(
            condition,
            then_body,
            elif_clauses,
            else_body.as_deref(),
        )?
        else {
            return Ok(false);
        };
        self.emit_rust_stmt_with_current_indent(&lowered_if_stmt);
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

        let Some(lowered_cond) = self.lower_condition_expr_for_ir(condition)? else {
            return Ok(false);
        };
        self.loop_else_stack.push(false);
        let lowered_body = self.try_lower_stmt_block_for_ir(body)?;
        let popped = self.loop_else_stack.pop();
        debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
        let Some(lowered_body) = lowered_body else {
            return Ok(false);
        };
        self.emit_rust_stmt_with_current_indent(&RustStmt::While {
            cond: lowered_cond,
            body: lowered_body,
        });
        Ok(true)
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
        let has_else = else_body.is_some();
        let var = if target.contains(',') {
            let names = target
                .split(',')
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .collect::<Vec<_>>();
            if names.is_empty() {
                return Ok(false);
            }
            format!("({})", names.join(", "))
        } else {
            target.clone()
        };

        self.loop_else_stack.push(has_else);
        let lowered_iter = self.try_lower_for_iter_expr_for_ir(iter)?;
        let lowered_body = self.try_lower_stmt_block_for_ir(body)?;
        let popped = self.loop_else_stack.pop();
        debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
        let Some(lowered_iter) = lowered_iter else {
            return Ok(false);
        };
        let Some(lowered_body) = lowered_body else {
            return Ok(false);
        };

        if let Some(else_body) = else_body {
            let Some(lowered_else_body) = self.try_lower_stmt_block_for_ir(else_body)? else {
                return Ok(false);
            };
            self.emit_rust_stmt_with_current_indent(&RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: true,
                    name: "_broke".to_string(),
                    ty: Some(crate::RustType::Bool),
                    value: crate::RustExpr::Literal(crate::RustLiteral::Bool(false)),
                },
                RustStmt::For {
                    var,
                    iter: lowered_iter,
                    body: lowered_body,
                },
                RustStmt::If {
                    cond: crate::RustExpr::UnaryOp {
                        op: "!".to_string(),
                        operand: Box::new(crate::RustExpr::Paren(Box::new(
                            crate::RustExpr::Ident("_broke".to_string()),
                        ))),
                    },
                    then_body: lowered_else_body,
                    else_body: None,
                },
            ]));
            return Ok(true);
        }

        self.emit_rust_stmt_with_current_indent(&RustStmt::For {
            var,
            iter: lowered_iter,
            body: lowered_body,
        });
        Ok(true)
    }

    pub(crate) fn try_emit_structured_with_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::With { items, body } = stmt else {
            return Ok(false);
        };
        let Some(lowered_with) = self.try_lower_with_stmt_for_ir(items, body)? else {
            return Ok(false);
        };
        self.emit_rust_stmt_with_current_indent(&lowered_with);
        Ok(true)
    }

    pub(crate) fn try_emit_structured_try_except_stmt(&mut self, stmt: &HirStmt) -> bool {
        let HirStmt::TryExcept { body, handlers, .. } = stmt else {
            return false;
        };
        let lowered = match self.try_lower_try_except_stmt_for_ir(body, handlers) {
            Ok(Some(lowered)) => lowered,
            Ok(None) => return false,
            Err(_) => return false,
        };
        for lowered_stmt in lowered {
            self.emit_rust_stmt_with_current_indent(&lowered_stmt);
        }
        true
    }

    fn try_lower_try_except_stmt_for_ir(
        &mut self,
        body: &[HirStmt],
        handlers: &[HirExceptHandler],
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        if handlers.is_empty() {
            return Ok(None);
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

        let mut closure_body = {
            if capture_returns {
                self.try_closure_depth += 1;
                self.try_closure_option_wrap.push(!direct_return_capture);
            }
            self.try_closure_error_type.push(err_ty.clone());
            let lowered = self.try_lower_stmt_block_for_ir(body)?;
            if capture_returns {
                self.try_closure_depth -= 1;
                self.try_closure_option_wrap.pop();
            }
            self.try_closure_error_type.pop();
            let Some(lowered) = lowered else {
                return Ok(None);
            };
            lowered
        };

        if !capture_returns {
            closure_body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("Ok".to_string())),
                args: vec![RustExpr::Literal(crate::RustLiteral::Unit)],
            })));
        } else if direct_return_capture {
            closure_body.push(RustStmt::Expr(RustExpr::FormatMacro {
                name: "unreachable".to_string(),
                format_str: "sifr try/except return capture fell through".to_string(),
                args: vec![],
            }));
        } else {
            closure_body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("Ok".to_string())),
                args: vec![RustExpr::Literal(crate::RustLiteral::None)],
            })));
        }

        let mut lowered = vec![RustStmt::Let {
            mutable: false,
            name: "__sifr_try_res".to_string(),
            ty: Some(crate::RustType::Result(
                Box::new(crate::RustType::Named(ok_ty.clone())),
                Box::new(crate::RustType::Named(err_ty.clone())),
            )),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Paren(Box::new(RustExpr::ClosureBlock {
                    params: vec![],
                    body: closure_body,
                    is_move: false,
                }))),
                args: vec![],
            },
        }];

        if capture_returns {
            let mut arms = vec![crate::RustMatchArm {
                pattern: if direct_return_capture {
                    "Ok(__sifr_ret_val)".to_string()
                } else {
                    "Ok(Some(__sifr_ret_val))".to_string()
                },
                bindings: vec!["__sifr_ret_val".to_string()],
                guard: None,
                body: vec![RustStmt::Return(Some(RustExpr::Ident(
                    "__sifr_ret_val".to_string(),
                )))],
            }];
            if !direct_return_capture {
                arms.push(crate::RustMatchArm {
                    pattern: "Ok(None)".to_string(),
                    bindings: vec![],
                    guard: None,
                    body: vec![],
                });
            }
            let Some(handler_chain) =
                self.lower_try_except_handler_chain_for_ir(handlers, "__sifr_try_err", &err_ty)?
            else {
                return Ok(None);
            };
            arms.push(crate::RustMatchArm {
                pattern: "Err(__sifr_try_err)".to_string(),
                bindings: vec!["__sifr_try_err".to_string()],
                guard: None,
                body: handler_chain,
            });
            lowered.push(RustStmt::Match {
                expr: RustExpr::Ident("__sifr_try_res".to_string()),
                arms,
            });
        } else {
            let Some(handler_chain) =
                self.lower_try_except_handler_chain_for_ir(handlers, "__sifr_try_err", &err_ty)?
            else {
                return Ok(None);
            };
            lowered.push(RustStmt::IfLet {
                pattern: "Err(__sifr_try_err)".to_string(),
                expr: RustExpr::Ident("__sifr_try_res".to_string()),
                then_body: handler_chain,
                else_body: None,
            });
        }
        Ok(Some(lowered))
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
                return HandlerMatchCondition::Expr(RustExpr::BinOp {
                    left: Box::new(RustExpr::Field {
                        expr: Box::new(RustExpr::Ident(err_ident.to_string())),
                        field: "kind".to_string(),
                    }),
                    op: "==".to_string(),
                    right: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Literal(crate::RustLiteral::Str(
                            kind.to_string(),
                        ))),
                        method: "to_string".to_string(),
                        args: vec![],
                    }),
                });
            }
            return HandlerMatchCondition::Unsupported;
        }
        if error_type == err_ty {
            return HandlerMatchCondition::Always;
        }
        HandlerMatchCondition::Unsupported
    }

    fn lower_try_except_handler_chain_for_ir(
        &mut self,
        handlers: &[HirExceptHandler],
        err_ident: &str,
        err_ty: &str,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let mut branches: Vec<(Option<RustExpr>, Vec<RustStmt>)> = Vec::new();
        for handler in handlers {
            let condition = Self::try_except_handler_condition_expr(handler, err_ident, err_ty);
            if matches!(condition, HandlerMatchCondition::Unsupported) {
                continue;
            }

            let mut handler_body = Vec::new();
            let handler_name = handler.name.as_deref().unwrap_or("_e");
            if handler_name != "_" {
                handler_body.push(RustStmt::Let {
                    mutable: false,
                    name: handler_name.to_string(),
                    ty: None,
                    value: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(err_ident.to_string())),
                        method: "clone".to_string(),
                        args: vec![],
                    },
                });
            }
            match self.try_lower_stmt_block_for_ir(&handler.body) {
                Ok(Some(lowered_handler_body)) => handler_body.extend(lowered_handler_body),
                Ok(None) => return Ok(None),
                Err(err) => return Err(err),
            }

            let cond_expr = match condition {
                HandlerMatchCondition::Always => None,
                HandlerMatchCondition::Expr(cond) => Some(cond),
                HandlerMatchCondition::Unsupported => continue,
            };
            branches.push((cond_expr, handler_body));
        }

        if branches.is_empty() {
            return Ok(Some(vec![RustStmt::Let {
                mutable: false,
                name: "_".to_string(),
                ty: None,
                value: RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Ident(err_ident.to_string())),
                },
            }]));
        }

        let mut current_else: Option<Vec<RustStmt>> = None;
        for (cond, body) in branches.into_iter().rev() {
            if let Some(cond) = cond {
                current_else = Some(vec![RustStmt::If {
                    cond,
                    then_body: body,
                    else_body: current_else,
                }]);
            } else {
                current_else = Some(body);
            }
        }
        Ok(Some(current_else.unwrap_or_default()))
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

        let Some(lowered_test) = self.lower_rendered_expr_for_ir(test)? else {
            return Ok(false);
        };
        let lowered_msg = if let Some(msg_expr) = msg {
            let Some(lowered) = self.lower_rendered_expr_for_ir(msg_expr)? else {
                return Ok(false);
            };
            Some(lowered)
        } else {
            None
        };
        self.emit_rust_stmt_with_current_indent(&RustStmt::Assert {
            cond: lowered_test,
            msg: lowered_msg,
        });
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
