use crate::helpers::needs_clone_for_type;
use crate::RustEmitter;
use sifr_hir::{HirExpr, HirFStringPart};
use sifr_type_system::{ParamConvention, Type};

fn uses_debug_display_format(ty: &Type) -> bool {
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
        Type::Alias(_, inner) => uses_debug_display_format(inner),
    }
}

fn option_inner_type(ty: &Type) -> Option<&Type> {
    let resolved = crate::resolve_alias_type_for_plain_call(ty);
    let Type::Union(members) = resolved else {
        return None;
    };
    if members.len() != 2 || !members.iter().any(|member| matches!(member, Type::None)) {
        return None;
    }
    members.iter().find(|member| !matches!(member, Type::None))
}

fn option_inner_from_rust_type(ty: &Type) -> Option<Type> {
    let rust_ty = ty.rust_type();
    if !rust_ty.starts_with("Option<") {
        return None;
    }
    if rust_ty.contains("String") {
        return Some(Type::Str);
    }
    if rust_ty.contains("i64") {
        return Some(Type::Int);
    }
    if rust_ty.contains("f64") {
        return Some(Type::Float);
    }
    if rust_ty.contains("bool") {
        return Some(Type::Bool);
    }
    Some(Type::Unknown)
}

fn display_option_inner_type(expr: &HirExpr) -> Option<Type> {
    if let Some(inner) = option_inner_type(expr.ty()) {
        return Some(inner.clone());
    }
    if let HirExpr::Index { object, .. } = expr {
        match crate::resolve_alias_type_for_plain_call(object.ty()) {
            Type::List(elem) => return Some((**elem).clone()),
            Type::Dict(_, value) => return Some((**value).clone()),
            Type::Str => return Some(Type::Str),
            _ => {}
        }
    }
    option_inner_from_rust_type(expr.ty())
}

fn can_construct_error_from_message(ty_name: &str) -> bool {
    matches!(
        ty_name,
        "Error"
            | "IOError"
            | "ParseError"
            | "ValueError"
            | "DivisionError"
            | "KeyError"
            | "JSONDecodeError"
            | "TOMLDecodeError"
            | "RegexError"
            | "OverflowError"
            | "IndexError"
            | "AttributeError"
            | "TypeError"
            | "ZeroDivisionError"
            | "RuntimeError"
            | "NotImplementedError"
            | "HashlibError"
    )
}

fn is_string_like_type(ty: &Type) -> bool {
    matches!(
        crate::resolve_alias_type_for_plain_call(ty),
        Type::Str | Type::LiteralStr(_)
    )
}

fn is_borrowed_string_name_expr(
    expr: &HirExpr,
    borrowed_params: &std::collections::HashSet<String>,
    mut_borrowed_params: &std::collections::HashSet<String>,
) -> bool {
    matches!(
        expr,
        HirExpr::Name { name, ty }
            if is_string_like_type(ty)
                && (borrowed_params.contains(name) || mut_borrowed_params.contains(name))
    )
}

impl RustEmitter {
    fn write_format_macro_call(
        &mut self,
        macro_name: &str,
        format_str: &str,
        args: &[crate::RustExpr],
    ) {
        let name = macro_name.trim_end_matches('!').to_string();
        self.emit_rust_expr(&crate::RustExpr::FormatMacro {
            name,
            format_str: format_str.to_string(),
            args: args.to_vec(),
        });
    }

    pub(super) fn try_lower_registry_expr_result(
        &self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        Ok(crate::try_lower_leaf_expr_result(expr)?
            .map(|lowered| self.rewrite_stdlib_constant_idents_in_expr(lowered)))
    }

    pub(super) fn rewrite_stdlib_constant_idents_in_expr(
        &self,
        expr: crate::RustExpr,
    ) -> crate::RustExpr {
        match expr {
            crate::RustExpr::Ident(name) => self.rewrite_special_ident(name),
            crate::RustExpr::MethodCall {
                receiver,
                method,
                args,
            } => crate::RustExpr::MethodCall {
                receiver: Box::new(match receiver.as_ref() {
                    crate::RustExpr::Ident(name) if self.is_stdlib_constant(name) => {
                        crate::RustExpr::Ident(name.clone())
                    }
                    _ => self.rewrite_stdlib_constant_idents_in_expr(*receiver),
                }),
                method,
                args: args
                    .into_iter()
                    .map(|arg| self.rewrite_stdlib_constant_idents_in_expr(arg))
                    .collect(),
            },
            crate::RustExpr::FnCall { func, args } => crate::RustExpr::FnCall {
                func: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*func)),
                args: args
                    .into_iter()
                    .map(|arg| self.rewrite_stdlib_constant_idents_in_expr(arg))
                    .collect(),
            },
            crate::RustExpr::MacroCall { name, args } => crate::RustExpr::MacroCall {
                name,
                args: args
                    .into_iter()
                    .map(|arg| self.rewrite_stdlib_constant_idents_in_expr(arg))
                    .collect(),
            },
            crate::RustExpr::FormatMacro {
                name,
                format_str,
                args,
            } => crate::RustExpr::FormatMacro {
                name,
                format_str,
                args: args
                    .into_iter()
                    .map(|arg| self.rewrite_stdlib_constant_idents_in_expr(arg))
                    .collect(),
            },
            crate::RustExpr::BinOp { left, op, right } => crate::RustExpr::BinOp {
                left: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*left)),
                op,
                right: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*right)),
            },
            crate::RustExpr::UnaryOp { op, operand } => crate::RustExpr::UnaryOp {
                op,
                operand: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*operand)),
            },
            crate::RustExpr::Field { expr, field } => {
                let rewritten_expr = match expr.as_ref() {
                    crate::RustExpr::Ident(name) if self.is_stdlib_constant(name) => {
                        crate::RustExpr::Ident(name.clone())
                    }
                    _ => self.rewrite_stdlib_constant_idents_in_expr(*expr),
                };
                crate::RustExpr::Field {
                    expr: Box::new(rewritten_expr),
                    field,
                }
            }
            crate::RustExpr::Index { expr, index } => crate::RustExpr::Index {
                expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)),
                index: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*index)),
            },
            crate::RustExpr::Slice { expr, start, stop } => crate::RustExpr::Slice {
                expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)),
                start: start
                    .map(|part| Box::new(self.rewrite_stdlib_constant_idents_in_expr(*part))),
                stop: stop.map(|part| Box::new(self.rewrite_stdlib_constant_idents_in_expr(*part))),
            },
            crate::RustExpr::Ref { mutable, expr } => crate::RustExpr::Ref {
                mutable,
                expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)),
            },
            crate::RustExpr::Deref(expr) => {
                crate::RustExpr::Deref(Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)))
            }
            crate::RustExpr::Clone(expr) => {
                crate::RustExpr::Clone(Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)))
            }
            crate::RustExpr::Cast { expr, ty } => crate::RustExpr::Cast {
                expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)),
                ty,
            },
            crate::RustExpr::Block { stmts, expr } => crate::RustExpr::Block {
                stmts: stmts
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
                expr: expr
                    .map(|inner| Box::new(self.rewrite_stdlib_constant_idents_in_expr(*inner))),
            },
            crate::RustExpr::If {
                cond,
                then_expr,
                else_expr,
            } => crate::RustExpr::If {
                cond: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*cond)),
                then_expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*then_expr)),
                else_expr: else_expr
                    .map(|inner| Box::new(self.rewrite_stdlib_constant_idents_in_expr(*inner))),
            },
            crate::RustExpr::Match { expr, arms } => crate::RustExpr::Match {
                expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)),
                arms: arms
                    .into_iter()
                    .map(|arm| crate::RustMatchArm {
                        pattern: arm.pattern,
                        bindings: arm.bindings,
                        guard: arm
                            .guard
                            .map(|guard| self.rewrite_stdlib_constant_idents_in_expr(guard)),
                        body: arm
                            .body
                            .into_iter()
                            .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                            .collect(),
                    })
                    .collect(),
            },
            crate::RustExpr::Closure {
                params,
                body,
                is_move,
            } => crate::RustExpr::Closure {
                params,
                body: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*body)),
                is_move,
            },
            crate::RustExpr::ClosureBlock {
                params,
                body,
                is_move,
            } => crate::RustExpr::ClosureBlock {
                params,
                body: body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
                is_move,
            },
            crate::RustExpr::StructInit { name, fields } => crate::RustExpr::StructInit {
                name,
                fields: fields
                    .into_iter()
                    .map(|(field, value)| {
                        (field, self.rewrite_stdlib_constant_idents_in_expr(value))
                    })
                    .collect(),
            },
            crate::RustExpr::Tuple(items) => crate::RustExpr::Tuple(
                items
                    .into_iter()
                    .map(|item| self.rewrite_stdlib_constant_idents_in_expr(item))
                    .collect(),
            ),
            crate::RustExpr::Array(items) => crate::RustExpr::Array(
                items
                    .into_iter()
                    .map(|item| self.rewrite_stdlib_constant_idents_in_expr(item))
                    .collect(),
            ),
            crate::RustExpr::Vec(items) => crate::RustExpr::Vec(
                items
                    .into_iter()
                    .map(|item| self.rewrite_stdlib_constant_idents_in_expr(item))
                    .collect(),
            ),
            crate::RustExpr::Try(expr) => {
                crate::RustExpr::Try(Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)))
            }
            crate::RustExpr::Await(expr) => {
                crate::RustExpr::Await(Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)))
            }
            crate::RustExpr::Paren(expr) => {
                crate::RustExpr::Paren(Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)))
            }
            crate::RustExpr::Range { start, end } => crate::RustExpr::Range {
                start: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*start)),
                end: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*end)),
            },
            crate::RustExpr::Literal(lit) => crate::RustExpr::Literal(lit),
            crate::RustExpr::Path(path) => crate::RustExpr::Path(path),
        }
    }

    pub(super) fn rewrite_stdlib_constant_idents_in_stmt(
        &self,
        stmt: crate::RustStmt,
    ) -> crate::RustStmt {
        match stmt {
            crate::RustStmt::Let {
                mutable,
                name,
                ty,
                value,
            } => crate::RustStmt::Let {
                mutable,
                name,
                ty,
                value: self.rewrite_stdlib_constant_idents_in_expr(value),
            },
            crate::RustStmt::LetPattern { pattern, value } => crate::RustStmt::LetPattern {
                pattern,
                value: self.rewrite_stdlib_constant_idents_in_expr(value),
            },
            crate::RustStmt::Assign { target, value } => crate::RustStmt::Assign {
                target: self.rewrite_stdlib_constant_idents_in_expr(target),
                value: self.rewrite_stdlib_constant_idents_in_expr(value),
            },
            crate::RustStmt::AugAssign { target, op, value } => crate::RustStmt::AugAssign {
                target: self.rewrite_stdlib_constant_idents_in_expr(target),
                op,
                value: self.rewrite_stdlib_constant_idents_in_expr(value),
            },
            crate::RustStmt::Expr(expr) => {
                crate::RustStmt::Expr(self.rewrite_stdlib_constant_idents_in_expr(expr))
            }
            crate::RustStmt::Assert { cond, msg } => crate::RustStmt::Assert {
                cond: self.rewrite_stdlib_constant_idents_in_expr(cond),
                msg: msg.map(|msg| self.rewrite_stdlib_constant_idents_in_expr(msg)),
            },
            crate::RustStmt::Return(expr) => crate::RustStmt::Return(
                expr.map(|ret| self.rewrite_stdlib_constant_idents_in_expr(ret)),
            ),
            crate::RustStmt::If {
                cond,
                then_body,
                else_body,
            } => crate::RustStmt::If {
                cond: self.rewrite_stdlib_constant_idents_in_expr(cond),
                then_body: then_body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
                else_body: else_body.map(|body| {
                    body.into_iter()
                        .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                        .collect()
                }),
            },
            crate::RustStmt::IfLet {
                pattern,
                expr,
                then_body,
                else_body,
            } => crate::RustStmt::IfLet {
                pattern,
                expr: self.rewrite_stdlib_constant_idents_in_expr(expr),
                then_body: then_body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
                else_body: else_body.map(|body| {
                    body.into_iter()
                        .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                        .collect()
                }),
            },
            crate::RustStmt::Match { expr, arms } => crate::RustStmt::Match {
                expr: self.rewrite_stdlib_constant_idents_in_expr(expr),
                arms: arms
                    .into_iter()
                    .map(|arm| crate::RustMatchArm {
                        pattern: arm.pattern,
                        bindings: arm.bindings,
                        guard: arm
                            .guard
                            .map(|guard| self.rewrite_stdlib_constant_idents_in_expr(guard)),
                        body: arm
                            .body
                            .into_iter()
                            .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                            .collect(),
                    })
                    .collect(),
            },
            crate::RustStmt::For { var, iter, body } => crate::RustStmt::For {
                var,
                iter: self.rewrite_stdlib_constant_idents_in_expr(iter),
                body: body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
            },
            crate::RustStmt::With { items, body } => crate::RustStmt::With {
                items: items
                    .into_iter()
                    .map(|item| crate::RustWithItem {
                        binding: item.binding,
                        value: self.rewrite_stdlib_constant_idents_in_expr(item.value),
                        has_cm: item.has_cm,
                        class_name: item.class_name,
                    })
                    .collect(),
                body: body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
            },
            crate::RustStmt::While { cond, body } => crate::RustStmt::While {
                cond: self.rewrite_stdlib_constant_idents_in_expr(cond),
                body: body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
            },
            crate::RustStmt::Loop { body } => crate::RustStmt::Loop {
                body: body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
            },
            crate::RustStmt::LocalFn {
                name,
                params,
                ret,
                body,
            } => crate::RustStmt::LocalFn {
                name,
                params,
                ret,
                body: body
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
            },
            crate::RustStmt::Block(stmts) => crate::RustStmt::Block(
                stmts
                    .into_iter()
                    .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt))
                    .collect(),
            ),
            crate::RustStmt::Break | crate::RustStmt::Continue => stmt,
        }
    }

    pub(crate) fn try_emit_structured_print_call(
        &mut self,
        args: &[HirExpr],
    ) -> Result<bool, crate::CodegenError> {
        if args.is_empty() {
            self.emit_rust_expr(&crate::RustExpr::MacroCall {
                name: "println".to_string(),
                args: vec![],
            });
            return Ok(true);
        }
        if args.len() > 1 {
            if let HirExpr::StringLiteral(fmt) = &args[0] {
                let lowered_args = args
                    .iter()
                    .skip(1)
                    .map(|arg| self.lower_display_expr(arg))
                    .collect::<Vec<_>>();
                self.write_format_macro_call("println!", fmt, &lowered_args);
                return Ok(true);
            }
            let format_str = (0..args.len()).map(|_| "{}").collect::<Vec<_>>().join(" ");
            let lowered_args = args
                .iter()
                .map(|arg| self.lower_display_expr(arg))
                .collect::<Vec<_>>();
            self.write_format_macro_call("println!", &format_str, &lowered_args);
            return Ok(true);
        }
        let arg = &args[0];
        if let HirExpr::StringLiteral(value) = arg {
            let escaped = value
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('{', "{{")
                .replace('}', "}}");
            self.emit_rust_expr(&crate::RustExpr::FormatMacro {
                name: "println".to_string(),
                format_str: escaped,
                args: vec![],
            });
            return Ok(true);
        }
        if let HirExpr::FString { parts, .. } = arg {
            self.emit_fstring_macro("println!", parts);
            return Ok(true);
        }

        let Some(lowered_arg) = self.try_lower_registry_expr_strict(arg) else {
            return Ok(false);
        };
        if let Some(inner) = display_option_inner_type(arg) {
            let option_format_str = if uses_debug_display_format(&inner) {
                "{:?}".to_string()
            } else {
                "{}".to_string()
            };
            self.emit_rust_expr(&crate::RustExpr::FormatMacro {
                name: "println".to_string(),
                format_str: "{}".to_string(),
                args: vec![crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                    method: "map_or".to_string(),
                    args: vec![
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Str(
                                "None".to_string(),
                            ))),
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
                                format_str: option_format_str,
                                args: vec![crate::RustExpr::Ident("__v".to_string())],
                            }),
                            is_move: false,
                        },
                    ],
                }],
            });
        } else if uses_debug_display_format(arg.ty()) {
            self.emit_rust_expr(&crate::RustExpr::FormatMacro {
                name: "println".to_string(),
                format_str: "{:?}".to_string(),
                args: vec![lowered_arg],
            });
        } else {
            self.emit_rust_expr(&crate::RustExpr::FormatMacro {
                name: "println".to_string(),
                format_str: "{}".to_string(),
                args: vec![lowered_arg],
            });
        }
        Ok(true)
    }

    pub(crate) fn try_lower_structured_field_access_expr(
        &mut self,
        object: &HirExpr,
        field: &str,
        ty: &Type,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let Some(lowered_object) = crate::try_lower_leaf_or_name_expr_result(object)? else {
            return Ok(None);
        };

        if matches!(object.ty(), Type::Enum { .. }) {
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Field {
                    expr: Box::new(lowered_object),
                    field: field.to_string(),
                }),
                args: vec![],
            }));
        }

        let is_self_access = matches!(object, HirExpr::Name { name, .. } if name == "self");
        let suppress_self_clone = if is_self_access && self.pending_self_field_clone_suppression > 0
        {
            self.pending_self_field_clone_suppression -= 1;
            true
        } else {
            false
        };
        let needs_clone = is_self_access && needs_clone_for_type(ty) && !suppress_self_clone;

        let class_name_for_parent = if let Some(ref current_class_name) = self.current_class_name {
            if is_self_access {
                Some(current_class_name.clone())
            } else {
                None
            }
        } else {
            None
        }
        .or_else(|| {
            if let Type::Class { name, .. } = object.ty() {
                Some(name.clone())
            } else {
                None
            }
        });

        let lowered_base = if let Some(ref class_name) = class_name_for_parent {
            if let Some((parent_name, parent_field_names)) = self.parent_fields.get(class_name) {
                if parent_field_names.contains(field) {
                    crate::RustExpr::Field {
                        expr: Box::new(lowered_object),
                        field: parent_name.to_lowercase(),
                    }
                } else {
                    lowered_object
                }
            } else {
                lowered_object
            }
        } else {
            lowered_object
        };

        let lowered_field = crate::RustExpr::Field {
            expr: Box::new(lowered_base),
            field: field.to_string(),
        };

        if needs_clone {
            return Ok(Some(crate::RustExpr::MethodCall {
                receiver: Box::new(lowered_field),
                method: "clone".to_string(),
                args: vec![],
            }));
        }

        Ok(Some(lowered_field))
    }

    pub(crate) fn try_emit_structured_plain_call_with_signature(
        &mut self,
        func: &str,
        args: &[HirExpr],
    ) -> Result<bool, crate::CodegenError> {
        if crate::is_reserved_plain_builtin_call(func) || self.nested_fn_captures.contains_key(func)
        {
            return Ok(false);
        }

        let param_types = self
            .func_signatures
            .get(func)
            .map(|(params, _)| params.clone())
            .or_else(|| self.callable_var_conventions.get(func).cloned());
        if param_types
            .as_ref()
            .is_some_and(|params| params.len() != args.len())
        {
            return Ok(false);
        }

        let mut lowered_args = Vec::with_capacity(args.len());
        for (idx, arg) in args.iter().enumerate() {
            let mut param_ty_for_arg: Option<Type> = None;
            let mut convention_for_arg: Option<ParamConvention> = None;
            if let Some(params) = param_types.as_ref() {
                let (param_ty, convention) = &params[idx];
                param_ty_for_arg = Some(param_ty.clone());
                convention_for_arg = Some(*convention);
            }
            let Some(mut arg_expr) = self.try_lower_expr_for_structured_emit(arg)? else {
                return Ok(false);
            };
            let mut should_wrap_option = false;
            let mut should_wrap_option_box = false;
            let mut should_box_protocol = false;
            let mut result_error_coerce_target: Option<String> = None;
            if let Some(param_ty) = param_ty_for_arg.as_ref() {
                if let Some(adapted) =
                    self.try_build_callable_adapter_closure(arg, param_ty, arg_expr.clone())
                {
                    arg_expr = adapted;
                }
                let resolved_param = crate::resolve_alias_type_for_plain_call(param_ty);
                should_wrap_option = crate::helpers::is_option_type(resolved_param)
                    && !crate::helpers::is_option_type(arg.ty())
                    && !matches!(arg, HirExpr::NoneLiteral);
                should_wrap_option_box =
                    should_wrap_option && param_ty.rust_type().starts_with("Option<Box<");
                should_box_protocol = matches!(resolved_param, Type::Protocol { .. });
                if let (Type::Result(param_ok, param_err), Type::Result(arg_ok, arg_err)) = (
                    resolved_param,
                    crate::resolve_alias_type_for_plain_call(arg.ty()),
                ) {
                    let param_ok_ty = crate::render_type(&crate::sifr_type_to_rust_type(param_ok));
                    let arg_ok_ty = crate::render_type(&crate::sifr_type_to_rust_type(arg_ok));
                    let param_err_ty =
                        crate::render_type(&crate::sifr_type_to_rust_type(param_err));
                    let arg_err_ty = crate::render_type(&crate::sifr_type_to_rust_type(arg_err));
                    let ok_compatible =
                        matches!(param_ok.as_ref(), Type::TypeVar(_)) || param_ok_ty == arg_ok_ty;
                    if ok_compatible
                        && param_err_ty != arg_err_ty
                        && can_construct_error_from_message(&param_err_ty)
                    {
                        result_error_coerce_target = Some(param_err_ty);
                    }
                }
            }
            if let Some(target_err_ty) = result_error_coerce_target {
                let mut target_path: Vec<String> =
                    target_err_ty.split("::").map(str::to_string).collect();
                target_path.push("new".to_string());
                arg_expr = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(arg_expr))),
                    method: "map_err".to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "__e".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(target_path)),
                            args: vec![crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident("__e".to_string())),
                                method: "to_string".to_string(),
                                args: vec![],
                            }],
                        }),
                        is_move: false,
                    }],
                };
            }
            if should_wrap_option {
                if should_wrap_option_box {
                    arg_expr = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                        args: vec![crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "Box".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![arg_expr],
                        }],
                    };
                } else {
                    arg_expr = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                        args: vec![arg_expr],
                    };
                }
            }
            if should_box_protocol {
                arg_expr = crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "Box".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![arg_expr],
                };
            }
            let borrowed_name_arg = matches!(arg, HirExpr::Name { name, ty }
                if self.borrowed_params.contains(name)
                    || self.mut_borrowed_params.contains(name)
                    || ty.rust_type().starts_with('&'));
            let should_clone_for_own = (matches!(convention_for_arg, Some(ParamConvention::Own))
                && borrowed_name_arg)
                || (convention_for_arg.is_none()
                    && func.ends_with("::new")
                    && matches!(arg, HirExpr::Name { .. }));

            if let Some(param_ty) = param_ty_for_arg.as_ref() {
                let resolved_param = crate::resolve_alias_type_for_plain_call(param_ty);
                if let Type::Union(members) = resolved_param {
                    if !crate::helpers::is_option_type(resolved_param)
                        && !matches!(
                            crate::resolve_alias_type_for_plain_call(arg.ty()),
                            Type::Union(_)
                        )
                    {
                        if let Some(variant) = crate::helpers::find_union_variant(members, arg.ty())
                        {
                            let enum_name = resolved_param.union_enum_name();
                            if should_clone_for_own {
                                arg_expr = crate::RustExpr::Clone(Box::new(
                                    crate::RustExpr::Paren(Box::new(arg_expr)),
                                ));
                            } else if matches!(
                                convention_for_arg,
                                Some(ParamConvention::Borrow | ParamConvention::MutBorrow)
                            ) && !matches!(arg, HirExpr::Name { .. })
                            {
                                arg_expr = crate::RustExpr::Paren(Box::new(arg_expr));
                            }
                            arg_expr = crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    enum_name,
                                    variant.to_string(),
                                ])),
                                args: vec![arg_expr],
                            };
                            if let Some(convention) = convention_for_arg {
                                arg_expr = self.apply_borrow_prefix_expr(
                                    convention,
                                    arg.ty(),
                                    param_ty_for_arg.as_ref(),
                                    match arg {
                                        HirExpr::Name { name, .. } => Some(name.as_str()),
                                        _ => None,
                                    },
                                    arg_expr,
                                );
                            }
                            lowered_args.push(arg_expr);
                            continue;
                        }
                    }
                }
            }
            if should_clone_for_own {
                arg_expr =
                    crate::RustExpr::Clone(Box::new(crate::RustExpr::Paren(Box::new(arg_expr))));
            } else if matches!(
                convention_for_arg,
                Some(ParamConvention::Borrow | ParamConvention::MutBorrow)
            ) && !matches!(arg, HirExpr::Name { .. })
            {
                arg_expr = crate::RustExpr::Paren(Box::new(arg_expr));
            }
            if let Some(convention) = convention_for_arg {
                arg_expr = self.apply_borrow_prefix_expr(
                    convention,
                    arg.ty(),
                    param_ty_for_arg.as_ref(),
                    match arg {
                        HirExpr::Name { name, .. } => Some(name.as_str()),
                        _ => None,
                    },
                    arg_expr,
                );
            }
            lowered_args.push(arg_expr);
        }
        self.emit_rust_expr(&crate::RustExpr::FnCall {
            func: Box::new(Self::lower_callable_name_expr(func)),
            args: lowered_args,
        });
        Ok(true)
    }

    fn try_build_callable_adapter_closure(
        &self,
        arg: &HirExpr,
        param_ty: &Type,
        callee_expr: crate::RustExpr,
    ) -> Option<crate::RustExpr> {
        let Type::Callable(_, expected_conventions, _) =
            crate::resolve_alias_type_for_plain_call(param_ty)
        else {
            return None;
        };
        let HirExpr::Name { name: callee, .. } = arg else {
            return None;
        };
        let provided_params = self
            .func_signatures
            .get(callee)
            .map(|(params, _)| params.clone())
            .or_else(|| self.callable_var_conventions.get(callee).cloned())?;
        if provided_params.len() != expected_conventions.len() {
            return None;
        }
        if !provided_params
            .iter()
            .zip(expected_conventions.iter())
            .any(|((_, provided), expected)| *provided != *expected)
        {
            return None;
        }

        let mut params = Vec::with_capacity(provided_params.len());
        let mut call_args = Vec::with_capacity(provided_params.len());
        for idx in 0..provided_params.len() {
            params.push(crate::RustParam::Named {
                name: format!("__arg{idx}"),
                ty: crate::RustType::Named("_".to_string()),
            });
        }

        for (idx, ((_, provided), expected)) in provided_params
            .iter()
            .zip(expected_conventions.iter())
            .enumerate()
        {
            let arg_name = format!("__arg{idx}");
            let adapted = match (expected, provided) {
                (ParamConvention::Borrow | ParamConvention::MutBorrow, ParamConvention::Own) => {
                    crate::RustExpr::Clone(Box::new(crate::RustExpr::Paren(Box::new(
                        crate::RustExpr::Ident(arg_name),
                    ))))
                }
                (ParamConvention::Own, ParamConvention::Borrow) => crate::RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(crate::RustExpr::Ident(arg_name)),
                },
                (ParamConvention::Own, ParamConvention::MutBorrow) => crate::RustExpr::Ref {
                    mutable: true,
                    expr: Box::new(crate::RustExpr::Ident(arg_name)),
                },
                (ParamConvention::Borrow, ParamConvention::MutBorrow) => crate::RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident(
                        arg_name,
                    )))),
                },
                _ => crate::RustExpr::Ident(arg_name),
            };
            call_args.push(adapted);
        }
        Some(crate::RustExpr::Closure {
            params,
            body: Box::new(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Paren(Box::new(callee_expr))),
                args: call_args,
            }),
            is_move: false,
        })
    }

    pub(crate) fn try_emit_structured_plain_call(
        &mut self,
        func: &str,
        args: &[HirExpr],
    ) -> Result<bool, crate::CodegenError> {
        if crate::is_reserved_plain_builtin_call(func) || func == "print" {
            return Ok(false);
        }

        let Some(mut lowered_args) = self.try_lower_registry_exprs_strict(args) else {
            return Ok(false);
        };
        if let Some(captures) = self.nested_fn_captures.get(func).cloned() {
            for (capture_name, _) in captures {
                lowered_args.push(crate::RustExpr::Ident(capture_name));
            }
        }
        let lowered = crate::RustExpr::FnCall {
            func: Box::new(crate::RustExpr::Ident(func.to_string())),
            args: lowered_args,
        };
        self.write_registry_expr(&lowered);
        Ok(true)
    }

    pub(crate) fn try_emit_structured_special_call(
        &mut self,
        func: &str,
        args: &[HirExpr],
    ) -> Result<bool, crate::CodegenError> {
        if matches!(
            func,
            "bool"
                | "pow"
                | "bigint"
                | "int"
                | "float"
                | "round"
                | "abs"
                | "sum"
                | "any"
                | "all"
                | "reversed"
                | "min"
                | "max"
                | "sorted"
                | "enumerate"
        ) || (func == "zip" && args.len() == 2)
        {
            if let Some(lowered) = self.try_lower_registry_builtin_call_expr(func, args) {
                self.write_registry_expr(&lowered);
                return Ok(true);
            }
        }

        match func {
            "str" => {
                if args.is_empty() {
                    self.write_registry_expr(&crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "String".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![],
                    });
                    return Ok(true);
                }
                let [arg] = args else {
                    return Ok(false);
                };
                let Some(lowered_arg) = self.try_lower_registry_expr_strict(arg) else {
                    return Ok(false);
                };
                let call_return_ty = if let HirExpr::Call { func, .. } = arg {
                    self.func_signatures.get(func).map(|(_, ret)| ret.clone())
                } else {
                    None
                };
                let str_arg_ty = call_return_ty.as_ref().unwrap_or_else(|| arg.ty());
                if let Some(inner) = option_inner_type(str_arg_ty) {
                    let format_str = if uses_debug_display_format(inner) {
                        "{:?}".to_string()
                    } else {
                        "{}".to_string()
                    };
                    self.write_registry_expr(&crate::RustExpr::MethodCall {
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
                    });
                    return Ok(true);
                }
                self.write_registry_expr(&crate::RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str: if uses_debug_display_format(str_arg_ty) {
                        "{:?}".to_string()
                    } else {
                        "{}".to_string()
                    },
                    args: vec![lowered_arg],
                });
                Ok(true)
            }
            "repr" => {
                let [arg] = args else {
                    return Ok(false);
                };
                let Some(lowered_arg) = self.try_lower_registry_expr_strict(arg) else {
                    return Ok(false);
                };
                if option_inner_type(arg.ty()).is_some() {
                    self.write_registry_expr(&crate::RustExpr::MethodCall {
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
                                    format_str: "{:?}".to_string(),
                                    args: vec![crate::RustExpr::Ident("__v".to_string())],
                                }),
                                is_move: false,
                            },
                        ],
                    });
                } else {
                    self.write_registry_expr(&crate::RustExpr::FormatMacro {
                        name: "format".to_string(),
                        format_str: "{:?}".to_string(),
                        args: vec![lowered_arg],
                    });
                }
                Ok(true)
            }
            "isinstance" => {
                let [object, type_arg] = args else {
                    return Ok(false);
                };
                let type_name = match type_arg {
                    HirExpr::StringLiteral(name) => name.as_str(),
                    HirExpr::Name { name, .. } => name.as_str(),
                    _ => return Ok(false),
                };
                let resolved_object_ty = crate::resolve_alias_type_for_plain_call(object.ty());
                if let Type::Union(members) = resolved_object_ty {
                    let target_ty = match type_name {
                        "int" => Some(Type::Int),
                        "str" => Some(Type::Str),
                        "float" => Some(Type::Float),
                        "bool" => Some(Type::Bool),
                        other => members
                            .iter()
                            .find(|m| matches!(m, Type::Class { name, .. } if name == other))
                            .cloned(),
                    };
                    let Some(target_ty) = target_ty else {
                        self.emit_rust_expr(&crate::RustExpr::Literal(crate::RustLiteral::Bool(
                            false,
                        )));
                        return Ok(true);
                    };
                    if !members.contains(&target_ty) {
                        self.emit_rust_expr(&crate::RustExpr::Literal(crate::RustLiteral::Bool(
                            false,
                        )));
                        return Ok(true);
                    }
                    let Some(object_lowered) = self.try_lower_expr_for_structured_emit(object)?
                    else {
                        return Ok(false);
                    };
                    let preferred_enum_name = resolved_object_ty.union_enum_name();
                    let variant_name = target_ty.union_variant_name();
                    let enum_name = if self.union_enums.contains_key(&preferred_enum_name) {
                        preferred_enum_name
                    } else {
                        self.union_enums
                            .iter()
                            .find_map(|(candidate, candidate_members)| {
                                let has_variant = candidate_members
                                    .iter()
                                    .any(|member| member.union_variant_name() == variant_name);
                                let has_all_members = members.iter().all(|needed_member| {
                                    let needed_variant = needed_member.union_variant_name();
                                    candidate_members
                                        .iter()
                                        .any(|member| member.union_variant_name() == needed_variant)
                                });
                                if has_variant && has_all_members {
                                    Some(candidate.clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(preferred_enum_name)
                    };
                    self.emit_rust_expr(&crate::RustExpr::Block {
                        stmts: vec![
                            crate::RustStmt::Let {
                                mutable: true,
                                name: "__sifr_isinstance_match".to_string(),
                                ty: Some(crate::RustType::Bool),
                                value: crate::RustExpr::Literal(crate::RustLiteral::Bool(false)),
                            },
                            crate::RustStmt::IfLet {
                                pattern: format!("{enum_name}::{variant_name}(..)"),
                                expr: object_lowered,
                                then_body: vec![crate::RustStmt::Assign {
                                    target: crate::RustExpr::Ident(
                                        "__sifr_isinstance_match".to_string(),
                                    ),
                                    value: crate::RustExpr::Literal(crate::RustLiteral::Bool(
                                        true,
                                    )),
                                }],
                                else_body: None,
                            },
                        ],
                        expr: Some(Box::new(crate::RustExpr::Ident(
                            "__sifr_isinstance_match".to_string(),
                        ))),
                    });
                    return Ok(true);
                }
                let matches = match type_name {
                    "int" => matches!(resolved_object_ty, Type::Int | Type::LiteralInt(_)),
                    "str" => matches!(resolved_object_ty, Type::Str | Type::LiteralStr(_)),
                    "float" => matches!(resolved_object_ty, Type::Float),
                    "bool" => matches!(resolved_object_ty, Type::Bool | Type::LiteralBool(_)),
                    other => {
                        matches!(resolved_object_ty, Type::Class { name, .. } if name == other)
                    }
                };
                self.emit_rust_expr(&crate::RustExpr::Literal(crate::RustLiteral::Bool(matches)));
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn try_emit_structured_compare_expr(
        &mut self,
        left: &HirExpr,
        ops: &[String],
        comparators: &[HirExpr],
    ) -> Result<bool, crate::CodegenError> {
        if ops.is_empty() || ops.len() != comparators.len() {
            return Ok(false);
        }

        let Some(mut lhs_expr_ir) = self.try_lower_expr_for_structured_emit(left)? else {
            return Ok(false);
        };
        let mut lhs_expr = left;
        let mut chain_expr: Option<crate::RustExpr> = None;
        for (idx, op) in ops.iter().enumerate() {
            let Some(rhs_expr) = comparators.get(idx) else {
                return Ok(false);
            };
            let Some(rhs_expr_ir) = self.try_lower_expr_for_structured_emit(rhs_expr)? else {
                return Ok(false);
            };
            let lowered_op = match op.as_str() {
                "==" | "!=" | "<" | "<=" | ">" | ">=" => op.as_str(),
                "is" => "==",
                "is not" => "!=",
                _ => return Ok(false),
            };

            let lhs_none_like = matches!(lhs_expr, HirExpr::NoneLiteral)
                || matches!(
                    crate::resolve_alias_type_for_plain_call(lhs_expr.ty()),
                    Type::None
                );
            let rhs_none_like = matches!(rhs_expr, HirExpr::NoneLiteral)
                || matches!(
                    crate::resolve_alias_type_for_plain_call(rhs_expr.ty()),
                    Type::None
                );
            if (op == "is" || op == "is not") && lhs_none_like && rhs_none_like {
                let pair_expr = crate::RustExpr::Literal(crate::RustLiteral::Bool(op == "is"));
                chain_expr = Some(if let Some(prev) = chain_expr {
                    crate::RustExpr::BinOp {
                        left: Box::new(crate::RustExpr::Paren(Box::new(prev))),
                        op: "&&".to_string(),
                        right: Box::new(crate::RustExpr::Paren(Box::new(pair_expr))),
                    }
                } else {
                    pair_expr
                });
                lhs_expr_ir = rhs_expr_ir;
                lhs_expr = rhs_expr;
                continue;
            }
            let mut lhs_cmp = lhs_expr_ir.clone();
            let mut rhs_cmp = rhs_expr_ir.clone();
            let mut string_as_str_applied = false;

            let is_comparison_op = matches!(lowered_op, "==" | "!=" | "<" | "<=" | ">" | ">=");
            if is_comparison_op
                && option_inner_type(lhs_expr.ty()).is_some()
                && option_inner_type(rhs_expr.ty()).is_none()
                && !rhs_none_like
            {
                rhs_cmp = crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                    args: vec![rhs_cmp],
                };
            } else if is_comparison_op
                && option_inner_type(lhs_expr.ty()).is_none()
                && option_inner_type(rhs_expr.ty()).is_some()
                && !lhs_none_like
            {
                lhs_cmp = crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                    args: vec![lhs_cmp],
                };
            } else if is_string_like_type(lhs_expr.ty()) && is_string_like_type(rhs_expr.ty()) {
                lhs_cmp = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lhs_cmp))),
                    method: "as_str".to_string(),
                    args: vec![],
                };
                rhs_cmp = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(rhs_cmp))),
                    method: "as_str".to_string(),
                    args: vec![],
                };
                string_as_str_applied = true;
            }

            if !string_as_str_applied
                && ((is_borrowed_string_name_expr(
                    lhs_expr,
                    &self.borrowed_params,
                    &self.mut_borrowed_params,
                ) && is_string_like_type(rhs_expr.ty()))
                    || (is_borrowed_string_name_expr(
                        rhs_expr,
                        &self.borrowed_params,
                        &self.mut_borrowed_params,
                    ) && is_string_like_type(lhs_expr.ty())))
            {
                lhs_cmp = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lhs_cmp))),
                    method: "as_str".to_string(),
                    args: vec![],
                };
                rhs_cmp = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(rhs_cmp))),
                    method: "as_str".to_string(),
                    args: vec![],
                };
            }

            let pair_expr = crate::RustExpr::BinOp {
                left: Box::new(lhs_cmp),
                op: lowered_op.to_string(),
                right: Box::new(rhs_cmp),
            };
            chain_expr = Some(if let Some(prev) = chain_expr {
                crate::RustExpr::BinOp {
                    left: Box::new(crate::RustExpr::Paren(Box::new(prev))),
                    op: "&&".to_string(),
                    right: Box::new(crate::RustExpr::Paren(Box::new(pair_expr))),
                }
            } else {
                pair_expr
            });
            lhs_expr_ir = rhs_expr_ir;
            lhs_expr = rhs_expr;
        }

        let Some(chain_expr) = chain_expr else {
            return Ok(false);
        };
        self.emit_rust_expr(&chain_expr);
        Ok(true)
    }

    pub(crate) fn try_emit_structured_question_mark_expr(
        &mut self,
        inner: &HirExpr,
    ) -> Result<bool, crate::CodegenError> {
        let Some(inner_expr) = self.try_lower_expr_for_structured_emit(inner)? else {
            return Ok(false);
        };
        if let Some(target_err_ty) = self.try_closure_error_type.last().cloned() {
            let resolved_inner_ty = crate::resolve_alias_type_for_plain_call(inner.ty());
            if let Type::Result(_, inner_err_ty) = resolved_inner_ty {
                let inner_err_ty_name =
                    crate::render_type(&crate::sifr_type_to_rust_type(inner_err_ty));
                if inner_err_ty_name == target_err_ty {
                    self.emit_rust_expr(&crate::RustExpr::Try(Box::new(inner_expr)));
                    return Ok(true);
                }
                if can_construct_error_from_message(&target_err_ty) {
                    let mut path: Vec<String> =
                        target_err_ty.split("::").map(str::to_string).collect();
                    path.push("new".to_string());
                    let ctor_func = crate::RustExpr::Path(path);
                    self.emit_rust_expr(&crate::RustExpr::Try(Box::new(
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(inner_expr))),
                            method: "map_err".to_string(),
                            args: vec![crate::RustExpr::Closure {
                                params: vec![crate::RustParam::Named {
                                    name: "__e".to_string(),
                                    ty: crate::RustType::Named("_".to_string()),
                                }],
                                body: Box::new(crate::RustExpr::FnCall {
                                    func: Box::new(ctor_func),
                                    args: vec![crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident(
                                            "__e".to_string(),
                                        )),
                                        method: "to_string".to_string(),
                                        args: vec![],
                                    }],
                                }),
                                is_move: false,
                            }],
                        },
                    )));
                    return Ok(true);
                }
            }
        }
        self.emit_rust_expr(&crate::RustExpr::Try(Box::new(inner_expr)));
        Ok(true)
    }

    pub(crate) fn try_emit_structured_result_wrap_expr(
        &mut self,
        wrapper: &str,
        value: &HirExpr,
    ) -> Result<bool, crate::CodegenError> {
        let Some(lowered_value) = self.try_lower_expr_for_structured_emit(value)? else {
            return Ok(false);
        };
        let lowered_wrapper = if wrapper.contains("::") {
            crate::RustExpr::Path(wrapper.split("::").map(str::to_string).collect())
        } else {
            crate::RustExpr::Ident(wrapper.to_string())
        };
        self.emit_rust_expr(&crate::RustExpr::FnCall {
            func: Box::new(lowered_wrapper),
            args: vec![lowered_value],
        });
        Ok(true)
    }

    pub(crate) fn try_emit_structured_bool_op_expr(
        &mut self,
        op: &str,
        values: &[HirExpr],
    ) -> Result<bool, crate::CodegenError> {
        if values.len() < 2 {
            return Ok(false);
        }
        let lowered_op = match op {
            "and" => "&&",
            "or" => "||",
            _ => return Ok(false),
        };

        let mut lowered_values = Vec::with_capacity(values.len());
        for value in values {
            let Some(lowered) = self.try_lower_registry_expr_strict(value) else {
                return Ok(false);
            };
            lowered_values.push(lowered);
        }
        let mut iter = lowered_values.into_iter();
        let Some(first) = iter.next() else {
            return Ok(false);
        };
        let combined = iter.fold(first, |acc, value| crate::RustExpr::BinOp {
            left: Box::new(crate::RustExpr::Paren(Box::new(acc))),
            op: lowered_op.to_string(),
            right: Box::new(crate::RustExpr::Paren(Box::new(value))),
        });
        self.emit_rust_expr(&crate::RustExpr::Paren(Box::new(combined)));
        Ok(true)
    }

    pub(crate) fn try_emit_structured_constructor_call_expr(
        &mut self,
        class_name: &str,
        args: &[HirExpr],
    ) -> Result<bool, crate::CodegenError> {
        let ctor_params = self
            .func_signatures
            .get(&format!("{class_name}::new"))
            .map(|(params, _)| params.clone());
        if ctor_params
            .as_ref()
            .is_some_and(|params| params.len() != args.len())
        {
            return Ok(false);
        }

        let mut lowered_args = Vec::with_capacity(args.len());
        for (idx, arg) in args.iter().enumerate() {
            let Some(lowered) = self.try_lower_expr_for_structured_emit(arg)? else {
                return Ok(false);
            };
            let mut lowered_arg = lowered;
            let mut should_wrap_option = false;
            let mut should_wrap_option_box = false;
            let borrowed_name_arg = matches!(arg, HirExpr::Name { name, ty }
                if self.borrowed_params.contains(name)
                    || self.mut_borrowed_params.contains(name)
                    || ty.rust_type().starts_with('&'));
            if let Some(params) = ctor_params.as_ref() {
                let (param_ty, convention) = &params[idx];
                let is_recursive_ctor_field = self
                    .class_field_order
                    .get(class_name)
                    .and_then(|fields| fields.get(idx))
                    .is_some_and(|field_name| {
                        self.recursive_fields
                            .contains(&(class_name.to_string(), field_name.clone()))
                    });
                let resolved_param = crate::resolve_alias_type_for_plain_call(param_ty);
                should_wrap_option = crate::helpers::is_option_type(resolved_param)
                    && !crate::helpers::is_option_type(arg.ty())
                    && !matches!(arg, HirExpr::NoneLiteral);
                should_wrap_option_box = should_wrap_option
                    && (param_ty.rust_type().starts_with("Option<Box<") || is_recursive_ctor_field);
                lowered_arg = self.apply_borrow_prefix_expr(
                    *convention,
                    arg.ty(),
                    Some(param_ty),
                    match arg {
                        HirExpr::Name { name, .. } => Some(name.as_str()),
                        _ => None,
                    },
                    lowered_arg,
                );
                if *convention == ParamConvention::Own && borrowed_name_arg {
                    lowered_arg = crate::RustExpr::Clone(Box::new(crate::RustExpr::Paren(
                        Box::new(lowered_arg),
                    )));
                }
            } else if matches!(arg, HirExpr::Name { .. }) {
                lowered_arg = crate::RustExpr::Clone(Box::new(crate::RustExpr::Paren(Box::new(
                    lowered_arg,
                ))));
            }
            if should_wrap_option {
                if should_wrap_option_box {
                    lowered_arg = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "Some".to_string(),
                        ])),
                        args: vec![crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "Box".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![lowered_arg],
                        }],
                    };
                } else {
                    lowered_arg = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                        args: vec![lowered_arg],
                    };
                }
            }
            lowered_args.push(lowered_arg);
        }

        self.emit_rust_expr(&crate::RustExpr::FnCall {
            func: Box::new(crate::RustExpr::Path(vec![
                class_name.to_string(),
                "new".to_string(),
            ])),
            args: lowered_args,
        });
        Ok(true)
    }

    pub(crate) fn try_emit_structured_list_literal_expr(
        &mut self,
        elements: &[HirExpr],
    ) -> Result<bool, crate::CodegenError> {
        let mut lowered_elements = Vec::with_capacity(elements.len());
        for element in elements {
            let Some(lowered) = self.lower_stmt_expr_for_ir(element)? else {
                return Ok(false);
            };
            lowered_elements.push(lowered);
        }

        self.emit_rust_expr(&crate::RustExpr::Vec(lowered_elements));
        Ok(true)
    }

    pub(crate) fn try_emit_structured_dict_literal_expr(
        &mut self,
        keys: &[HirExpr],
        values: &[HirExpr],
    ) -> Result<bool, crate::CodegenError> {
        if keys.len() != values.len() {
            return Ok(false);
        }
        let mut entries = Vec::with_capacity(keys.len());
        for (key, value) in keys.iter().zip(values.iter()) {
            let Some(key_lowered) = self.lower_stmt_expr_for_ir(key)? else {
                return Ok(false);
            };
            let Some(value_lowered) = self.lower_stmt_expr_for_ir(value)? else {
                return Ok(false);
            };
            entries.push((key_lowered, value_lowered));
        }
        let map_ident = "__sifr_dict_lit".to_string();
        let mut stmts = vec![crate::RustStmt::Let {
            mutable: true,
            name: map_ident.clone(),
            ty: None,
            value: crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    "std".to_string(),
                    "collections".to_string(),
                    "HashMap".to_string(),
                    "new".to_string(),
                ])),
                args: vec![],
            },
        }];
        for (key_expr, value_expr) in entries {
            stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Ident(map_ident.clone())),
                method: "insert".to_string(),
                args: vec![key_expr, value_expr],
            }));
        }
        self.emit_rust_expr(&crate::RustExpr::Block {
            stmts,
            expr: Some(Box::new(crate::RustExpr::Ident(map_ident))),
        });
        Ok(true)
    }

    pub(crate) fn try_emit_structured_set_literal_expr(
        &mut self,
        elements: &[HirExpr],
    ) -> Result<bool, crate::CodegenError> {
        let mut lowered_elements = Vec::with_capacity(elements.len());
        for element in elements {
            let Some(lowered) = self.lower_stmt_expr_for_ir(element)? else {
                return Ok(false);
            };
            lowered_elements.push(lowered);
        }

        let set_ident = "__sifr_set_lit".to_string();
        let mut stmts = vec![crate::RustStmt::Let {
            mutable: true,
            name: set_ident.clone(),
            ty: None,
            value: crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    "std".to_string(),
                    "collections".to_string(),
                    "HashSet".to_string(),
                    "new".to_string(),
                ])),
                args: vec![],
            },
        }];
        for lowered in lowered_elements {
            stmts.push(crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Ident(set_ident.clone())),
                method: "insert".to_string(),
                args: vec![lowered],
            }));
        }
        self.emit_rust_expr(&crate::RustExpr::Block {
            stmts,
            expr: Some(Box::new(crate::RustExpr::Ident(set_ident))),
        });
        Ok(true)
    }

    pub(crate) fn try_emit_structured_pre_call_expr(
        &mut self,
        expr: &HirExpr,
    ) -> Result<bool, crate::CodegenError> {
        match expr {
            HirExpr::Name { name, .. } => {
                let rewritten = self.rewrite_special_ident(name.clone());
                self.emit_rust_expr(&rewritten);
                Ok(true)
            }
            HirExpr::UnaryOp { op, operand, .. } => {
                self.try_emit_structured_unary_expr(op, operand)
            }
            HirExpr::BoolOp { op, values, .. } => self.try_emit_structured_bool_op_expr(op, values),
            HirExpr::QuestionMark { expr: inner, .. } => {
                self.try_emit_structured_question_mark_expr(inner)
            }
            HirExpr::OkWrap { value, .. } => self.try_emit_structured_result_wrap_expr("Ok", value),
            HirExpr::ErrWrap { value, .. } => {
                self.try_emit_structured_result_wrap_expr("Err", value)
            }
            HirExpr::WalrusExpr { name, value, .. } => {
                self.try_emit_structured_walrus_expr(name, value)
            }
            HirExpr::Compare {
                left,
                ops,
                comparators,
                ..
            } => self.try_emit_structured_compare_expr(left, ops, comparators),
            HirExpr::FString { parts, .. } => {
                self.emit_fstring_macro("format!", parts);
                Ok(true)
            }
            HirExpr::ContainsOp {
                element,
                collection,
                ..
            } => self.try_emit_structured_contains_expr(element, collection),
            _ => Ok(false),
        }
    }

    pub(crate) fn try_emit_structured_walrus_expr(
        &mut self,
        name: &str,
        value: &HirExpr,
    ) -> Result<bool, crate::CodegenError> {
        let Some(lowered_value) = self.try_lower_expr_for_structured_emit(value)? else {
            return Ok(false);
        };
        self.emit_rust_expr(&crate::RustExpr::Block {
            stmts: vec![crate::RustStmt::Let {
                mutable: false,
                name: name.to_string(),
                ty: None,
                value: lowered_value,
            }],
            expr: Some(Box::new(crate::RustExpr::Ident(name.to_string()))),
        });
        Ok(true)
    }

    pub(crate) fn try_emit_structured_contains_expr(
        &mut self,
        element: &HirExpr,
        collection: &HirExpr,
    ) -> Result<bool, crate::CodegenError> {
        let Some(element_expr) = self.try_lower_expr_for_structured_emit(element)? else {
            return Ok(false);
        };
        let Some(collection_expr) = self.try_lower_expr_for_structured_emit(collection)? else {
            return Ok(false);
        };

        match crate::resolve_alias_type_for_plain_call(collection.ty()) {
            Type::Dict(_, _) => {
                let key_arg = self.lower_dict_key_lookup_arg_expr(element, &element_expr);
                self.emit_rust_expr(&crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(collection_expr))),
                    method: "contains_key".to_string(),
                    args: vec![key_arg],
                });
                Ok(true)
            }
            Type::List(_) | Type::Set(_) | Type::Str => {
                self.emit_rust_expr(&crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(collection_expr))),
                    method: "contains".to_string(),
                    args: vec![crate::RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(crate::RustExpr::Paren(Box::new(element_expr))),
                    }],
                });
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn render_dict_key_lookup_arg(&mut self, key_expr: &HirExpr, key_rendered: &str) -> String {
        if let HirExpr::StringLiteral(value) = key_expr {
            return format!("{value:?}");
        }
        if let HirExpr::Name { name, ty } = key_expr {
            if self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name) {
                if matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Str) {
                    return format!("({key_rendered}).as_str()");
                } else {
                    return key_rendered.to_string();
                }
            }
        }
        format!("&({key_rendered})")
    }

    fn lower_dict_key_lookup_arg_expr(
        &self,
        key_expr: &HirExpr,
        key_lowered: &crate::RustExpr,
    ) -> crate::RustExpr {
        if let HirExpr::StringLiteral(value) = key_expr {
            return crate::RustExpr::Literal(crate::RustLiteral::Str(value.clone()));
        }
        if let HirExpr::Name { name, ty } = key_expr {
            if self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name) {
                if matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Str) {
                    return crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(key_lowered.clone()))),
                        method: "as_str".to_string(),
                        args: vec![],
                    };
                }
                return key_lowered.clone();
            }
        }
        crate::RustExpr::Ref {
            mutable: false,
            expr: Box::new(crate::RustExpr::Paren(Box::new(key_lowered.clone()))),
        }
    }

    fn try_lower_expr_for_structured_emit(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let saved_stats = self.lowering_stats;
        if let Some(lowered) = crate::try_lower_leaf_or_name_expr_result(expr)? {
            self.lowering_stats = saved_stats;
            return Ok(Some(self.rewrite_stdlib_constant_idents_in_expr(lowered)));
        }
        let Some(lowered) = self.try_lower_registry_expr_strict(expr) else {
            return Ok(None);
        };
        self.lowering_stats = saved_stats;
        Ok(Some(self.rewrite_stdlib_constant_idents_in_expr(lowered)))
    }

    fn lower_callable_name_expr(name: &str) -> crate::RustExpr {
        if name.contains("::") {
            crate::RustExpr::Path(name.split("::").map(str::to_string).collect())
        } else {
            crate::RustExpr::Ident(name.to_string())
        }
    }

    fn apply_borrow_prefix_expr(
        &mut self,
        convention: ParamConvention,
        arg_ty: &Type,
        param_ty: Option<&Type>,
        arg_name: Option<&str>,
        lowered_expr: crate::RustExpr,
    ) -> crate::RustExpr {
        match self.borrow_prefix_for_name(convention, arg_ty, param_ty, arg_name) {
            Some("&") => crate::RustExpr::Ref {
                mutable: false,
                expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_expr))),
            },
            Some("&mut ") => crate::RustExpr::Ref {
                mutable: true,
                expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_expr))),
            },
            _ => lowered_expr,
        }
    }

    pub(crate) fn try_emit_structured_unary_expr(
        &mut self,
        op: &str,
        operand: &HirExpr,
    ) -> Result<bool, crate::CodegenError> {
        let Some(operand_expr) = self.try_lower_expr_for_structured_emit(operand)? else {
            return Ok(false);
        };

        match op {
            "not" => {
                match crate::resolve_alias_type_for_plain_call(operand.ty()) {
                    Type::Int | Type::LiteralInt(_) => {
                        self.emit_rust_expr(&crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Paren(Box::new(operand_expr))),
                            op: "==".to_string(),
                            right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                        });
                    }
                    Type::Float => {
                        self.emit_rust_expr(&crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Paren(Box::new(operand_expr))),
                            op: "==".to_string(),
                            right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Float(
                                0.0,
                            ))),
                        });
                    }
                    Type::Str | Type::List(_) | Type::Dict(_, _) | Type::Set(_) => {
                        self.emit_rust_expr(&crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(operand_expr))),
                            method: "is_empty".to_string(),
                            args: vec![],
                        });
                    }
                    Type::Tuple(elems) => {
                        self.emit_rust_expr(&crate::RustExpr::Literal(crate::RustLiteral::Bool(
                            elems.is_empty(),
                        )));
                    }
                    Type::Bool => {
                        self.emit_rust_expr(&crate::RustExpr::UnaryOp {
                            op: "!".to_string(),
                            operand: Box::new(crate::RustExpr::Paren(Box::new(operand_expr))),
                        });
                    }
                    Type::None => {
                        self.emit_rust_expr(&crate::RustExpr::Literal(crate::RustLiteral::Bool(
                            true,
                        )));
                    }
                    _ if option_inner_type(operand.ty()).is_some() => {
                        self.emit_rust_expr(&crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(operand_expr))),
                            method: "is_none".to_string(),
                            args: vec![],
                        });
                    }
                    _ => {
                        self.emit_rust_expr(&crate::RustExpr::UnaryOp {
                            op: "!".to_string(),
                            operand: Box::new(crate::RustExpr::Paren(Box::new(operand_expr))),
                        });
                    }
                }
                Ok(true)
            }
            "-" => {
                self.emit_rust_expr(&crate::RustExpr::UnaryOp {
                    op: "-".to_string(),
                    operand: Box::new(crate::RustExpr::Paren(Box::new(operand_expr))),
                });
                Ok(true)
            }
            "+" => {
                self.emit_rust_expr(&crate::RustExpr::Paren(Box::new(operand_expr)));
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn try_emit_structured_numeric_binop_expr(
        &mut self,
        left: &HirExpr,
        op: &str,
        right: &HirExpr,
        ty: &Type,
    ) -> Result<bool, crate::CodegenError> {
        if !matches!(op, "+" | "-" | "*" | "/" | "//" | "%" | "**")
            || !matches!(
                crate::resolve_alias_type_for_plain_call(ty),
                Type::Int | Type::Float | Type::LiteralInt(_) | Type::TypeVar(_) | Type::BigInt
            )
        {
            return Ok(false);
        }

        let Some(mut left_expr) = self.try_lower_expr_for_structured_emit(left)? else {
            return Ok(false);
        };
        let Some(mut right_expr) = self.try_lower_expr_for_structured_emit(right)? else {
            return Ok(false);
        };
        if option_inner_type(ty).is_none() {
            if option_inner_type(left.ty()).is_some() {
                left_expr = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(left_expr))),
                    method: "unwrap".to_string(),
                    args: vec![],
                };
            }
            if option_inner_type(right.ty()).is_some() {
                right_expr = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(right_expr))),
                    method: "unwrap".to_string(),
                    args: vec![],
                };
            }
        }

        if matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Float) {
            if matches!(
                crate::resolve_alias_type_for_plain_call(left.ty()),
                Type::Int | Type::LiteralInt(_)
            ) {
                left_expr = crate::RustExpr::Cast {
                    expr: Box::new(crate::RustExpr::Paren(Box::new(left_expr))),
                    ty: crate::RustType::F64,
                };
            }
            if matches!(
                crate::resolve_alias_type_for_plain_call(right.ty()),
                Type::Int | Type::LiteralInt(_)
            ) {
                right_expr = crate::RustExpr::Cast {
                    expr: Box::new(crate::RustExpr::Paren(Box::new(right_expr))),
                    ty: crate::RustType::F64,
                };
            }
        }

        if op == "**" {
            let pow_expr = match (
                crate::resolve_alias_type_for_plain_call(left.ty()),
                crate::resolve_alias_type_for_plain_call(right.ty()),
            ) {
                (Type::BigInt, _) => crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(left_expr))),
                    method: "pow".to_string(),
                    args: vec![crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec![
                                "u32".to_string(),
                                "try_from".to_string(),
                            ])),
                            args: vec![right_expr],
                        }),
                        method: "unwrap_or".to_string(),
                        args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                    }],
                },
                (Type::Int | Type::LiteralInt(_), Type::Int | Type::LiteralInt(_)) => {
                    crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(left_expr))),
                        method: "pow".to_string(),
                        args: vec![crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::Paren(Box::new(right_expr))),
                            ty: crate::RustType::Named("u32".to_string()),
                        }],
                    }
                }
                (Type::Float, Type::Int | Type::LiteralInt(_)) => crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(left_expr))),
                    method: "powi".to_string(),
                    args: vec![crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::Paren(Box::new(right_expr))),
                        ty: crate::RustType::Named("i32".to_string()),
                    }],
                },
                _ => crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::Paren(Box::new(left_expr))),
                        ty: crate::RustType::F64,
                    }),
                    method: "powf".to_string(),
                    args: vec![crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::Paren(Box::new(right_expr))),
                        ty: crate::RustType::F64,
                    }],
                },
            };
            self.emit_rust_expr(&pow_expr);
            return Ok(true);
        }

        let rendered_op = if op == "//" { "/" } else { op };
        if matches!(crate::resolve_alias_type_for_plain_call(ty), Type::BigInt) {
            if matches!(left, HirExpr::Name { .. } | HirExpr::FieldAccess { .. }) {
                left_expr =
                    crate::RustExpr::Clone(Box::new(crate::RustExpr::Paren(Box::new(left_expr))));
            }
            if matches!(right, HirExpr::Name { .. } | HirExpr::FieldAccess { .. }) {
                right_expr =
                    crate::RustExpr::Clone(Box::new(crate::RustExpr::Paren(Box::new(right_expr))));
            }
            self.emit_rust_expr(&crate::RustExpr::Paren(Box::new(crate::RustExpr::BinOp {
                left: Box::new(left_expr),
                op: rendered_op.to_string(),
                right: Box::new(right_expr),
            })));
            return Ok(true);
        }
        self.emit_rust_expr(&crate::RustExpr::Paren(Box::new(crate::RustExpr::BinOp {
            left: Box::new(left_expr),
            op: rendered_op.to_string(),
            right: Box::new(right_expr),
        })));
        Ok(true)
    }

    pub(crate) fn try_emit_structured_if_expr(
        &mut self,
        condition: &HirExpr,
        then_expr: &HirExpr,
        else_expr: &HirExpr,
    ) -> Result<bool, crate::CodegenError> {
        let Some(condition_expr) = self.try_lower_expr_for_structured_emit(condition)? else {
            return Ok(false);
        };
        let Some(then_expr) = self.try_lower_expr_for_structured_emit(then_expr)? else {
            return Ok(false);
        };
        let Some(else_expr) = self.try_lower_expr_for_structured_emit(else_expr)? else {
            return Ok(false);
        };
        self.emit_rust_expr(&crate::RustExpr::If {
            cond: Box::new(condition_expr),
            then_expr: Box::new(then_expr),
            else_expr: Some(Box::new(else_expr)),
        });
        Ok(true)
    }

    pub(crate) fn try_lower_structured_class_binop_expr(
        &mut self,
        left: &HirExpr,
        op: &str,
        right: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let method_name = match op {
            "+" => "__add__",
            "-" => "__sub__",
            _ => return Ok(None),
        };
        let Type::Class { methods, .. } = crate::resolve_alias_type_for_plain_call(left.ty())
        else {
            return Ok(None);
        };
        let Some((_, method_sig)) = methods.iter().find(|(name, _)| name == method_name) else {
            return Ok(None);
        };

        let lowered_left = if let HirExpr::FieldAccess { object, field, ty } = left {
            self.try_lower_structured_field_access_expr(object, field, ty)?
        } else {
            crate::try_lower_leaf_or_name_expr_result(left)?
        };
        let Some(lowered_left) = lowered_left else {
            return Ok(None);
        };

        let lowered_right = if let HirExpr::FieldAccess { object, field, ty } = right {
            self.try_lower_structured_field_access_expr(object, field, ty)?
        } else {
            crate::try_lower_leaf_or_name_expr_result(right)?
        };
        let Some(lowered_right) = lowered_right else {
            return Ok(None);
        };

        let left_expr = crate::RustExpr::Ref {
            mutable: false,
            expr: Box::new(lowered_left),
        };

        let right_expr = if method_sig
            .params
            .first()
            .is_some_and(|(_, _, conv)| *conv == ParamConvention::Borrow)
        {
            crate::RustExpr::Ref {
                mutable: false,
                expr: Box::new(lowered_right),
            }
        } else {
            lowered_right
        };

        Ok(Some(crate::RustExpr::BinOp {
            left: Box::new(left_expr),
            op: op.to_string(),
            right: Box::new(right_expr),
        }))
    }

    pub(crate) fn try_lower_structured_index_expr(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
        result_ty: &Type,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let object_ty = crate::resolve_alias_type_for_plain_call(object.ty());
        let option_inner_ty = if let Type::Union(members) = object_ty {
            let mut inner = None;
            for member in members {
                let member = crate::resolve_alias_type_for_plain_call(member);
                if matches!(member, Type::None) {
                    continue;
                }
                if inner.is_some() {
                    inner = None;
                    break;
                }
                inner = Some(member);
            }
            inner
        } else {
            None
        };
        let index_base_ty = if matches!(
            object_ty,
            Type::Dict(_, _) | Type::List(_) | Type::Str | Type::Tuple(_)
        ) {
            Some(object_ty)
        } else if matches!(
            option_inner_ty,
            Some(Type::Dict(_, _) | Type::List(_) | Type::Str)
        ) {
            option_inner_ty
        } else {
            None
        };
        let Some(index_base_ty) = index_base_ty else {
            return Ok(None);
        };

        let suppress_self_field_clone = matches!(object, HirExpr::FieldAccess { object: inner, .. }
            if matches!(inner.as_ref(), HirExpr::Name { name, .. } if name == "self"))
            && self.pending_self_field_clone_suppression == 0;
        if suppress_self_field_clone {
            self.pending_self_field_clone_suppression += 1;
        }

        let lowered = (|| -> Result<Option<crate::RustExpr>, crate::CodegenError> {
            let lowered_object = if let HirExpr::FieldAccess {
                object: inner,
                field,
                ty,
            } = object
            {
                self.try_lower_structured_field_access_expr(inner, field, ty)?
            } else {
                crate::try_lower_leaf_or_name_expr_result(object)?
            };
            let lowered_object = match lowered_object {
                Some(expr) => expr,
                None => match self.try_lower_registry_expr_strict(object) {
                    Some(expr) => expr,
                    None => return Ok(None),
                },
            };

            let lowered_index = match crate::try_lower_leaf_or_name_expr_result(index)? {
                Some(expr) => expr,
                None => match self.try_lower_registry_expr_strict(index) {
                    Some(expr) => expr,
                    None => return Ok(None),
                },
            };

            let build_inner_index = |container_expr: crate::RustExpr| -> Option<crate::RustExpr> {
                let lowered_expr = match index_base_ty {
                    Type::Dict(key_ty, _) => {
                        let key_is_string_like = matches!(
                            crate::resolve_alias_type_for_plain_call(key_ty.as_ref()),
                            Type::Str | Type::LiteralStr(_)
                        );
                        let key_arg = if let HirExpr::StringLiteral(value) = index {
                            crate::RustExpr::Ident(format!("{value:?}"))
                        } else if key_is_string_like {
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                    lowered_index.clone(),
                                ))),
                                method: "as_str".to_string(),
                                args: vec![],
                            }
                        } else {
                            crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(lowered_index.clone()),
                            }
                        };
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(container_expr),
                                method: "get".to_string(),
                                args: vec![key_arg],
                            }),
                            method: "cloned".to_string(),
                            args: vec![],
                        }
                    }
                    Type::List(_) => {
                        let object_name = "__sifr_index_list".to_string();
                        let index_name = "__sifr_index_i".to_string();
                        let normalized_name = "__sifr_index_norm".to_string();
                        crate::RustExpr::Block {
                            stmts: vec![
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: object_name.clone(),
                                    ty: None,
                                    value: crate::RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(container_expr),
                                    },
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: index_name.clone(),
                                    ty: None,
                                    value: lowered_index.clone(),
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: normalized_name.clone(),
                                    ty: None,
                                    value: crate::RustExpr::If {
                                        cond: Box::new(crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                index_name.clone(),
                                            )),
                                            op: "<".to_string(),
                                            right: Box::new(crate::RustExpr::Literal(
                                                crate::RustLiteral::Int(0),
                                            )),
                                        }),
                                        then_expr: Box::new(crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::BinOp {
                                                left: Box::new(crate::RustExpr::Cast {
                                                    expr: Box::new(crate::RustExpr::MethodCall {
                                                        receiver: Box::new(crate::RustExpr::Ident(
                                                            object_name.clone(),
                                                        )),
                                                        method: "len".to_string(),
                                                        args: vec![],
                                                    }),
                                                    ty: crate::RustType::I64,
                                                }),
                                                op: "+".to_string(),
                                                right: Box::new(crate::RustExpr::Ident(
                                                    index_name.clone(),
                                                )),
                                            }),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        }),
                                        else_expr: Some(Box::new(crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::Ident(index_name)),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        })),
                                    },
                                },
                            ],
                            expr: Some(Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident(object_name)),
                                    method: "get".to_string(),
                                    args: vec![crate::RustExpr::Ident(normalized_name)],
                                }),
                                method: "cloned".to_string(),
                                args: vec![],
                            })),
                        }
                    }
                    Type::Str => {
                        let object_name = "__sifr_index_str".to_string();
                        let index_name = "__sifr_index_i".to_string();
                        let normalized_name = "__sifr_index_norm".to_string();
                        crate::RustExpr::Block {
                            stmts: vec![
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: object_name.clone(),
                                    ty: None,
                                    value: crate::RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(container_expr),
                                    },
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: index_name.clone(),
                                    ty: None,
                                    value: lowered_index.clone(),
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: normalized_name.clone(),
                                    ty: None,
                                    value: crate::RustExpr::If {
                                        cond: Box::new(crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                index_name.clone(),
                                            )),
                                            op: "<".to_string(),
                                            right: Box::new(crate::RustExpr::Literal(
                                                crate::RustLiteral::Int(0),
                                            )),
                                        }),
                                        then_expr: Box::new(crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::BinOp {
                                                left: Box::new(crate::RustExpr::Cast {
                                                    expr: Box::new(crate::RustExpr::MethodCall {
                                                        receiver: Box::new(
                                                            crate::RustExpr::MethodCall {
                                                                receiver: Box::new(
                                                                    crate::RustExpr::Ident(
                                                                        object_name.clone(),
                                                                    ),
                                                                ),
                                                                method: "chars".to_string(),
                                                                args: vec![],
                                                            },
                                                        ),
                                                        method: "count".to_string(),
                                                        args: vec![],
                                                    }),
                                                    ty: crate::RustType::I64,
                                                }),
                                                op: "+".to_string(),
                                                right: Box::new(crate::RustExpr::Ident(
                                                    index_name.clone(),
                                                )),
                                            }),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        }),
                                        else_expr: Some(Box::new(crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::Ident(index_name)),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        })),
                                    },
                                },
                            ],
                            expr: Some(Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident(object_name)),
                                        method: "chars".to_string(),
                                        args: vec![],
                                    }),
                                    method: "nth".to_string(),
                                    args: vec![crate::RustExpr::Ident(normalized_name)],
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
                            })),
                        }
                    }
                    Type::Tuple(elements) => {
                        let HirExpr::IntLiteral(idx) = index else {
                            return None;
                        };
                        let Ok(idx) = usize::try_from(*idx) else {
                            return None;
                        };
                        if idx >= elements.len() {
                            return None;
                        }
                        crate::RustExpr::Field {
                            expr: Box::new(container_expr),
                            field: idx.to_string(),
                        }
                    }
                    _ => return None,
                };
                Some(lowered_expr)
            };

            if let Some(inner_ty) = option_inner_ty {
                if !matches!(inner_ty, Type::Dict(_, _) | Type::List(_) | Type::Str) {
                    return Ok(None);
                }
                let Some(inner_expr) = build_inner_index(crate::RustExpr::Ident("__v".to_string()))
                else {
                    return Ok(None);
                };
                let option_expr = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_object))),
                        method: "as_ref".to_string(),
                        args: vec![],
                    }),
                    method: "and_then".to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "__v".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(inner_expr),
                        is_move: false,
                    }],
                };
                if crate::helpers::is_option_type(result_ty) {
                    return Ok(Some(option_expr));
                }
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(option_expr))),
                    method: "unwrap".to_string(),
                    args: vec![],
                }));
            }

            let Some(lowered_expr) = build_inner_index(lowered_object) else {
                return Ok(None);
            };
            if crate::helpers::is_option_type(result_ty) || matches!(index_base_ty, Type::Tuple(_))
            {
                return Ok(Some(lowered_expr));
            }
            Ok(Some(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_expr))),
                method: "unwrap".to_string(),
                args: vec![],
            }))
        })();

        if suppress_self_field_clone && self.pending_self_field_clone_suppression > 0 {
            self.pending_self_field_clone_suppression -= 1;
        }
        lowered
    }

    pub(crate) fn try_emit_structured_index_expr(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
        result_ty: &Type,
    ) -> Result<bool, crate::CodegenError> {
        let Some(lowered_expr) = self.try_lower_structured_index_expr(object, index, result_ty)?
        else {
            return Ok(false);
        };
        self.emit_rust_expr(&lowered_expr);
        Ok(true)
    }

    pub(crate) fn try_emit_structured_slice_expr(
        &mut self,
        object: &HirExpr,
        start: Option<&HirExpr>,
        stop: Option<&HirExpr>,
        step: Option<&HirExpr>,
        result_ty: &Type,
    ) -> Result<bool, crate::CodegenError> {
        let object_ty = crate::resolve_alias_type_for_plain_call(object.ty());
        if !matches!(object_ty, Type::Str | Type::List(_)) {
            return Ok(false);
        }
        let slice_expr = HirExpr::Slice {
            object: Box::new(object.clone()),
            start: start.cloned().map(Box::new),
            stop: stop.cloned().map(Box::new),
            step: step.cloned().map(Box::new),
            ty: result_ty.clone(),
        };
        let Some(lowered_slice) = self.try_lower_registry_expr_strict(&slice_expr) else {
            return Ok(false);
        };
        self.emit_rust_expr(&lowered_slice);
        Ok(true)
    }

    pub(crate) fn try_emit_structured_string_concat_expr(
        &mut self,
        left: &HirExpr,
        op: &str,
        right: &HirExpr,
        ty: &Type,
    ) -> Result<bool, crate::CodegenError> {
        if op != "+" || !matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Str) {
            return Ok(false);
        }

        let mut parts = Vec::new();
        collect_string_concat_parts_hir(left, &mut parts);
        collect_string_concat_parts_hir(right, &mut parts);

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
            self.emit_rust_expr(&crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Str(combined))),
                method: "to_string".to_string(),
                args: vec![],
            });
            return Ok(true);
        }

        let mut format_str = String::new();
        let mut format_args = Vec::new();
        for part in &parts {
            if let HirExpr::StringLiteral(value) = part {
                for ch in value.chars() {
                    match ch {
                        '{' => format_str.push_str("{{"),
                        '}' => format_str.push_str("}}"),
                        _ => format_str.push(ch),
                    }
                }
            } else {
                format_str.push_str("{}");
            }
        }
        for part in &parts {
            if !matches!(part, HirExpr::StringLiteral(_)) {
                let Some(lowered) = self.try_lower_expr_for_structured_emit(part)? else {
                    return Ok(false);
                };
                format_args.push(lowered);
            }
        }
        self.emit_rust_expr(&crate::RustExpr::FormatMacro {
            name: "format".to_string(),
            format_str,
            args: format_args,
        });
        Ok(true)
    }

    pub(crate) fn try_emit_structured_string_repeat_expr(
        &mut self,
        left: &HirExpr,
        op: &str,
        right: &HirExpr,
        ty: &Type,
    ) -> Result<bool, crate::CodegenError> {
        if op != "*" || !matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Str) {
            return Ok(false);
        }

        let (string_expr, count_expr) = match (
            crate::resolve_alias_type_for_plain_call(left.ty()),
            crate::resolve_alias_type_for_plain_call(right.ty()),
        ) {
            (Type::Str, Type::Int | Type::LiteralInt(_)) => (left, right),
            (Type::Int | Type::LiteralInt(_), Type::Str) => (right, left),
            _ => return Ok(false),
        };

        let Some(string_lowered) = self.try_lower_expr_for_structured_emit(string_expr)? else {
            return Ok(false);
        };
        let Some(count_lowered) = self.try_lower_expr_for_structured_emit(count_expr)? else {
            return Ok(false);
        };

        self.emit_rust_expr(&crate::RustExpr::Block {
            stmts: vec![crate::RustStmt::Let {
                mutable: false,
                name: "__n".to_string(),
                ty: None,
                value: count_lowered,
            }],
            expr: Some(Box::new(crate::RustExpr::If {
                cond: Box::new(crate::RustExpr::BinOp {
                    left: Box::new(crate::RustExpr::Ident("__n".to_string())),
                    op: "<=".to_string(),
                    right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                }),
                then_expr: Box::new(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "String".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                }),
                else_expr: Some(Box::new(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(string_lowered))),
                    method: "repeat".to_string(),
                    args: vec![crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::Ident("__n".to_string())),
                        ty: crate::RustType::Named("usize".to_string()),
                    }],
                })),
            })),
        });
        Ok(true)
    }

    pub(crate) fn try_emit_structured_list_concat_expr(
        &mut self,
        left: &HirExpr,
        op: &str,
        right: &HirExpr,
        ty: &Type,
    ) -> Result<bool, crate::CodegenError> {
        if op != "+" || !matches!(crate::resolve_alias_type_for_plain_call(ty), Type::List(_)) {
            return Ok(false);
        }
        if !matches!(
            crate::resolve_alias_type_for_plain_call(left.ty()),
            Type::List(_)
        ) || !matches!(
            crate::resolve_alias_type_for_plain_call(right.ty()),
            Type::List(_)
        ) {
            return Ok(false);
        }

        let Some(left_lowered) = self.try_lower_expr_for_structured_emit(left)? else {
            return Ok(false);
        };
        let Some(right_lowered) = self.try_lower_expr_for_structured_emit(right)? else {
            return Ok(false);
        };

        self.emit_rust_expr(&crate::RustExpr::Block {
            stmts: vec![
                crate::RustStmt::Let {
                    mutable: true,
                    name: "__v".to_string(),
                    ty: None,
                    value: crate::RustExpr::Clone(Box::new(crate::RustExpr::Paren(Box::new(
                        left_lowered,
                    )))),
                },
                crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Ident("__v".to_string())),
                    method: "extend".to_string(),
                    args: vec![crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(right_lowered))),
                            method: "iter".to_string(),
                            args: vec![],
                        }),
                        method: "cloned".to_string(),
                        args: vec![],
                    }],
                }),
            ],
            expr: Some(Box::new(crate::RustExpr::Ident("__v".to_string()))),
        });
        Ok(true)
    }

    fn rewrite_special_ident(&self, name: String) -> crate::RustExpr {
        if self.is_stdlib_constant(&name) {
            return match name.as_str() {
                "pi" => crate::RustExpr::Path(vec![
                    "std".to_string(),
                    "f64".to_string(),
                    "consts".to_string(),
                    "PI".to_string(),
                ]),
                "e" => crate::RustExpr::Path(vec![
                    "std".to_string(),
                    "f64".to_string(),
                    "consts".to_string(),
                    "E".to_string(),
                ]),
                "tau" => crate::RustExpr::Path(vec![
                    "std".to_string(),
                    "f64".to_string(),
                    "consts".to_string(),
                    "TAU".to_string(),
                ]),
                "inf" => crate::RustExpr::Path(vec!["f64".to_string(), "INFINITY".to_string()]),
                "nan" => crate::RustExpr::Path(vec!["f64".to_string(), "NAN".to_string()]),
                _ => crate::RustExpr::Ident(name),
            };
        }

        if let Some((_ty, rust_name)) = self.module_constants.get(&name) {
            if let Some(mapped) = parse_module_constant_expr(rust_name) {
                return mapped;
            }
        }

        crate::RustExpr::Ident(name)
    }
}

fn parse_module_constant_expr(rust_name: &str) -> Option<crate::RustExpr> {
    if let Some(func) = rust_name.strip_suffix("()") {
        let func_expr = parse_identifier_path_expr(func)?;
        return Some(crate::RustExpr::FnCall {
            func: Box::new(func_expr),
            args: vec![],
        });
    }
    parse_identifier_path_expr(rust_name)
}

fn parse_identifier_path_expr(name: &str) -> Option<crate::RustExpr> {
    let segments = name
        .split("::")
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.is_empty() || !segments.iter().all(|segment| is_ident(segment)) {
        return None;
    }
    if segments.len() == 1 {
        return Some(crate::RustExpr::Ident(segments[0].to_string()));
    }
    Some(crate::RustExpr::Path(
        segments.into_iter().map(ToString::to_string).collect(),
    ))
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(ch) if ch == '_' || ch.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn collect_string_concat_parts_hir<'a>(expr: &'a HirExpr, parts: &mut Vec<&'a HirExpr>) {
    if let HirExpr::BinOp {
        left,
        op,
        right,
        ty,
    } = expr
    {
        if op == "+" && matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Str) {
            collect_string_concat_parts_hir(left, parts);
            collect_string_concat_parts_hir(right, parts);
            return;
        }
    }
    parts.push(expr);
}

impl RustEmitter {
    pub(super) fn emit_fstring_macro(&mut self, macro_name: &str, parts: &[HirFStringPart]) {
        let mut format_str = String::new();
        let mut exprs: Vec<&HirExpr> = Vec::new();
        for part in parts {
            match part {
                HirFStringPart::Literal(s) => {
                    // Escape braces in the literal for Rust's format!
                    for ch in s.chars() {
                        match ch {
                            '{' => format_str.push_str("{{"),
                            '}' => format_str.push_str("}}"),
                            _ => format_str.push(ch),
                        }
                    }
                }
                HirFStringPart::Expr(expr) => {
                    format_str.push_str("{}");
                    exprs.push(expr);
                }
            }
        }
        let lowered_args = exprs
            .iter()
            .map(|expr| self.lower_display_expr(expr))
            .collect::<Vec<_>>();
        self.write_format_macro_call(macro_name, &format_str, &lowered_args);
    }
}
