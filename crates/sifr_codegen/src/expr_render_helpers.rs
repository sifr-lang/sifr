use crate::helpers::{needs_clone_for_type, MUTATING_METHODS};
use crate::RustEmitter;
use sifr_hir::{HirExpr, HirFStringPart};
use sifr_type_system::{ParamConvention, Type};

impl RustEmitter {
    pub(super) fn render_expr_via_direct_emit(&mut self, expr: &HirExpr) -> String {
        let saved_output = std::mem::take(&mut self.output);
        let saved_indent = self.indent;
        self.indent = 0;
        self.emit_expr(expr);
        let result = std::mem::take(&mut self.output);
        self.output = saved_output;
        self.indent = saved_indent;
        result.trim().to_string()
    }

    pub(super) fn try_lower_registry_expr_result(
        &self,
        expr: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if self.should_force_render_guard(expr) {
            return Ok(None);
        }
        Ok(crate::try_lower_leaf_expr_result(expr)?
            .map(|lowered| self.rewrite_stdlib_constant_idents_in_expr(lowered)))
    }

    pub(super) fn render_expr_with_lowered_path(&mut self, expr: &HirExpr) -> String {
        match self.try_lower_registry_expr_result(expr) {
            Ok(Some(lowered_expr)) => crate::render_expr(&lowered_expr),
            Ok(None) => self.render_expr_via_direct_emit(expr),
            Err(_) => {
                self.lowering_stats.expr_lowering_errors += 1;
                self.render_expr_via_direct_emit(expr)
            }
        }
    }

    pub(super) fn should_force_render_guard(&self, expr: &HirExpr) -> bool {
        if render_expr_contains_force_guard_name(self, expr) {
            return true;
        }
        matches!(expr, HirExpr::Compare { .. } | HirExpr::BoolOp { .. })
            && render_expr_uses_borrowed_param(
                expr,
                &self.borrowed_params,
                &self.mut_borrowed_params,
            )
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
                receiver: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*receiver)),
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
            crate::RustExpr::Field { expr, field } => crate::RustExpr::Field {
                expr: Box::new(self.rewrite_stdlib_constant_idents_in_expr(*expr)),
                field,
            },
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
            crate::RustExpr::RawCode(_) => {
                panic!("RawCode expression reached core production rewrite path")
            }
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
            crate::RustStmt::RawCode(_) => {
                panic!("RawCode statement reached core production rewrite path")
            }
        }
    }

    pub(crate) fn try_emit_structured_method_call(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
    ) -> Result<bool, crate::CodegenError> {
        if self.try_emit_method_via_registry(object.ty(), object, method, args) {
            return Ok(true);
        }

        let suppress_self_field_clone = crate::is_self_field_access_expr(object)
            && MUTATING_METHODS.contains(&method)
            && self.pending_self_field_clone_suppression == 0;
        if suppress_self_field_clone {
            self.pending_self_field_clone_suppression += 1;
        }

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

        let Some(lowered_object) = lowered_object else {
            if suppress_self_field_clone && self.pending_self_field_clone_suppression > 0 {
                self.pending_self_field_clone_suppression -= 1;
            }
            return Ok(false);
        };

        let class_ty = crate::resolve_alias_type_for_plain_call(object.ty());
        let method_params = if let Type::Class { methods, .. } = class_ty {
            methods
                .iter()
                .find(|(name, _)| name == method)
                .map(|(_, fty)| &fty.params)
        } else {
            None
        };

        let mut rendered_args = Vec::with_capacity(args.len());
        for (idx, arg) in args.iter().enumerate() {
            let mut rendered_arg = if let Some(lowered_arg) = crate::try_lower_leaf_or_name_expr_result(arg)? {
                let rewritten = self.rewrite_stdlib_constant_idents_in_expr(lowered_arg);
                crate::render_expr(&rewritten)
            } else {
                let saved_stats = self.lowering_stats;
                let Some(rendered) = self.try_render_structured_expr(arg)? else {
                    if suppress_self_field_clone && self.pending_self_field_clone_suppression > 0 {
                        self.pending_self_field_clone_suppression -= 1;
                    }
                    return Ok(false);
                };
                self.lowering_stats = saved_stats;
                rendered
            };

            if let Some(params) = method_params {
                if let Some((_, _param_ty, convention)) = params.get(idx) {
                    rendered_arg = match convention {
                        ParamConvention::Borrow => format!("&({rendered_arg})"),
                        ParamConvention::MutBorrow => format!("&mut ({rendered_arg})"),
                        ParamConvention::Own => rendered_arg,
                    };
                }
            }

            rendered_args.push(rendered_arg);
        }

        let lowered_object = crate::render_expr(&lowered_object);
        let is_callable_field = if let Type::Class { fields, methods, .. } = class_ty {
            !methods.iter().any(|(name, _)| name == method)
                && fields
                    .iter()
                    .any(|(name, ty)| name == method && matches!(ty, Type::Callable(..)))
        } else {
            false
        };
        if is_callable_field {
            self.write("(");
            self.write(&lowered_object);
            self.write(".");
            self.write(method);
            self.write(")(");
        } else {
            self.write(&lowered_object);
            self.write(".");
            self.write(method);
            self.write("(");
        }
        for (idx, arg) in rendered_args.iter().enumerate() {
            if idx > 0 {
                self.write(", ");
            }
            self.write(arg);
        }
        self.write(")");
        Ok(true)
    }

    pub(crate) fn try_emit_structured_print_call(
        &mut self,
        args: &[HirExpr],
    ) -> Result<bool, crate::CodegenError> {
        if args.is_empty() {
            self.write("println!()");
            return Ok(true);
        }
        if args.len() != 1 {
            return Ok(false);
        }
        let arg = &args[0];
        if let HirExpr::StringLiteral(value) = arg {
            let escaped = value
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('{', "{{")
                .replace('}', "}}");
            self.write(&format!("println!(\"{escaped}\")"));
            return Ok(true);
        }
        if let HirExpr::FString { parts, .. } = arg {
            self.emit_fstring_macro("println!", parts);
            return Ok(true);
        }

        let Some(arg_rendered) = self.try_render_structured_expr(arg)? else {
            return Ok(false);
        };
        self.write("println!(\"{}\", ");
        self.write(&arg_rendered);
        self.write(")");
        Ok(true)
    }

    pub(crate) fn try_render_structured_expr(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<String>, crate::CodegenError> {
        let saved_output = std::mem::take(&mut self.output);
        let rendered = match self.try_emit_structured_expr(expr) {
            Ok(true) => Some(std::mem::take(&mut self.output)),
            Ok(false) => None,
            Err(err) => {
                self.output = saved_output;
                return Err(err);
            }
        };
        self.output = saved_output;
        Ok(rendered)
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
        if func.contains("::")
            || crate::is_reserved_plain_builtin_call(func)
            || self.nested_fn_captures.contains_key(func)
        {
            return Ok(false);
        }

        let param_types = self
            .func_signatures
            .get(func)
            .map(|(params, _)| params.clone());
        if param_types
            .as_ref()
            .is_some_and(|params| params.len() != args.len())
        {
            return Ok(false);
        }

        let output_len = self.output.len();
        self.write(func);
        self.write("(");
        for (idx, arg) in args.iter().enumerate() {
            if idx > 0 {
                self.write(", ");
            }
            if let Some(params) = param_types.as_ref() {
                let (param_ty, convention) = &params[idx];
                if !crate::type_has_typevar(param_ty)
                    && !crate::type_has_typevar(arg.ty())
                    && crate::resolve_alias_type_for_plain_call(param_ty)
                        != crate::resolve_alias_type_for_plain_call(arg.ty())
                {
                    self.output.truncate(output_len);
                    return Ok(false);
                }
                self.emit_borrow_prefix(*convention, arg.ty(), Some(param_ty));
            }
            if !self.try_emit_structured_expr(arg)? {
                self.output.truncate(output_len);
                return Ok(false);
            }
        }
        self.write(")");
        Ok(true)
    }

    pub(crate) fn try_emit_structured_plain_call(
        &mut self,
        func: &str,
        args: &[HirExpr],
    ) -> Result<bool, crate::CodegenError> {
        if crate::is_reserved_plain_builtin_call(func) || func == "print" {
            return Ok(false);
        }

        let mut rendered_args = Vec::with_capacity(args.len());
        for arg in args {
            let saved_stats = self.lowering_stats;
            let Some(rendered) = self.try_render_structured_expr(arg)? else {
                return Ok(false);
            };
            self.lowering_stats = saved_stats;
            rendered_args.push(rendered);
        }

        self.write(func);
        self.write("(");
        for (idx, rendered_arg) in rendered_args.iter().enumerate() {
            if idx > 0 {
                self.write(", ");
            }
            self.write(rendered_arg);
        }
        if let Some(captures) = self.nested_fn_captures.get(func).cloned() {
            for (idx, (capture_name, _)) in captures.iter().enumerate() {
                if !rendered_args.is_empty() || idx > 0 {
                    self.write(", ");
                }
                self.write(capture_name);
            }
        }
        self.write(")");
        Ok(true)
    }

    pub(crate) fn try_emit_structured_special_call(
        &mut self,
        func: &str,
        args: &[HirExpr],
    ) -> Result<bool, crate::CodegenError> {
        match func {
            "str" => {
                if args.is_empty() {
                    self.write("String::new()");
                    return Ok(true);
                }
                let [arg] = args else {
                    return Ok(false);
                };
                let saved_stats = self.lowering_stats;
                let Some(arg_rendered) = self.try_render_structured_expr(arg)? else {
                    return Ok(false);
                };
                self.lowering_stats = saved_stats;
                if matches!(
                    crate::resolve_alias_type_for_plain_call(arg.ty()),
                    Type::List(_) | Type::Dict(_, _) | Type::Set(_) | Type::Tuple(_)
                ) {
                    self.write("format!(\"{:?}\", ");
                } else {
                    self.write("format!(\"{}\", ");
                }
                self.write(&arg_rendered);
                self.write(")");
                Ok(true)
            }
            "isinstance" => {
                self.write("true");
                Ok(true)
            }
            "bool" => {
                let [arg] = args else {
                    return Ok(false);
                };
                let saved_stats = self.lowering_stats;
                let Some(arg_rendered) = self.try_render_structured_expr(arg)? else {
                    return Ok(false);
                };
                self.lowering_stats = saved_stats;
                match crate::resolve_alias_type_for_plain_call(arg.ty()) {
                    Type::Int | Type::LiteralInt(_) => {
                        self.write(&arg_rendered);
                        self.write(" != 0");
                    }
                    Type::Float => {
                        self.write(&arg_rendered);
                        self.write(" != 0.0");
                    }
                    Type::Str | Type::List(_) | Type::Dict(_, _) => {
                        self.write("!(");
                        self.write(&arg_rendered);
                        self.write(").is_empty()");
                    }
                    Type::Tuple(elems) => self.write(if elems.is_empty() { "false" } else { "true" }),
                    Type::Bool => self.write(&arg_rendered),
                    Type::None => self.write("false"),
                    _ => self.write(&arg_rendered),
                }
                Ok(true)
            }
            "int" => {
                let [arg] = args else {
                    return Ok(false);
                };
                let saved_stats = self.lowering_stats;
                let Some(arg_rendered) = self.try_render_structured_expr(arg)? else {
                    return Ok(false);
                };
                self.lowering_stats = saved_stats;
                match crate::resolve_alias_type_for_plain_call(arg.ty()) {
                    Type::Float => {
                        self.write("(");
                        self.write(&arg_rendered);
                        self.write(") as i64");
                    }
                    Type::Str => {
                        self.write("(");
                        self.write(&arg_rendered);
                        self.write(").parse::<i64>().map_err(|e| ParseError { message: e.to_string() })");
                    }
                    Type::Bool => {
                        self.write("if ");
                        self.write(&arg_rendered);
                        self.write(" { 1_i64 } else { 0_i64 }");
                    }
                    Type::BigInt => {
                        self.write("i64::try_from(&(");
                        self.write(&arg_rendered);
                        self.write(")).map_err(|_| OverflowError { message: \"bigint value out of range for int\".to_string() })");
                    }
                    _ => self.write(&arg_rendered),
                }
                Ok(true)
            }
            "pow" => {
                let [base, exp] = args else {
                    return Ok(false);
                };
                let saved_stats = self.lowering_stats;
                let Some(base_rendered) = self.try_render_structured_expr(base)? else {
                    return Ok(false);
                };
                self.lowering_stats = saved_stats;
                let Some(exp_rendered) = self.try_render_structured_expr(exp)? else {
                    return Ok(false);
                };
                self.lowering_stats = saved_stats;
                if matches!(crate::resolve_alias_type_for_plain_call(base.ty()), Type::Int)
                    && matches!(crate::resolve_alias_type_for_plain_call(exp.ty()), Type::Int)
                {
                    self.write("(");
                    self.write(&base_rendered);
                    self.write(").pow((");
                    self.write(&exp_rendered);
                    self.write(") as u32)");
                } else {
                    self.write("((");
                    self.write(&base_rendered);
                    self.write(") as f64).powf((");
                    self.write(&exp_rendered);
                    self.write(") as f64)");
                }
                Ok(true)
            }
            "bigint" => {
                let [arg] = args else {
                    return Ok(false);
                };
                let saved_stats = self.lowering_stats;
                let Some(arg_rendered) = self.try_render_structured_expr(arg)? else {
                    return Ok(false);
                };
                self.lowering_stats = saved_stats;
                self.write("BigInt::from(");
                self.write(&arg_rendered);
                self.write(")");
                Ok(true)
            }
            "float" => {
                let [arg] = args else {
                    return Ok(false);
                };
                let saved_stats = self.lowering_stats;
                let Some(arg_rendered) = self.try_render_structured_expr(arg)? else {
                    return Ok(false);
                };
                self.lowering_stats = saved_stats;
                match crate::resolve_alias_type_for_plain_call(arg.ty()) {
                    Type::Int | Type::LiteralInt(_) => {
                        self.write("(");
                        self.write(&arg_rendered);
                        self.write(") as f64");
                    }
                    Type::Str => {
                        self.write("(");
                        self.write(&arg_rendered);
                        self.write(").parse::<f64>().map_err(|e| ParseError { message: e.to_string() })");
                    }
                    Type::Bool => {
                        self.write("if ");
                        self.write(&arg_rendered);
                        self.write(" { 1.0_f64 } else { 0.0_f64 }");
                    }
                    _ => self.write(&arg_rendered),
                }
                Ok(true)
            }
            "round" => {
                let [arg] = args else {
                    return Ok(false);
                };
                let Some(arg_rendered) = self.try_render_structured_expr(arg)? else {
                    return Ok(false);
                };
                self.write("(");
                self.write(&arg_rendered);
                self.write(").round() as i64");
                Ok(true)
            }
            "min" | "max" => {
                if args.len() == 1 {
                    let arg = &args[0];
                    let saved_stats = self.lowering_stats;
                    let Some(arg_rendered) = self.try_render_structured_expr(arg)? else {
                        return Ok(false);
                    };
                    self.lowering_stats = saved_stats;
                    self.write("(");
                    self.write(&arg_rendered);
                    self.write(").iter().cloned().");
                    if matches!(
                        crate::resolve_alias_type_for_plain_call(arg.ty()),
                        Type::List(inner)
                            if matches!(crate::resolve_alias_type_for_plain_call(inner), Type::Float)
                    ) {
                        self.write(func);
                        self.write("_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))");
                    } else {
                        self.write(func);
                        self.write("()");
                    }
                } else {
                    let [left, right] = args else {
                        return Ok(false);
                    };
                    let Some(left_rendered) = self.try_render_structured_expr(left)? else {
                        return Ok(false);
                    };
                    let Some(right_rendered) = self.try_render_structured_expr(right)? else {
                        return Ok(false);
                    };
                    if matches!(crate::resolve_alias_type_for_plain_call(left.ty()), Type::Float)
                        && matches!(crate::resolve_alias_type_for_plain_call(right.ty()), Type::Float)
                    {
                        self.write("(");
                        self.write(&left_rendered);
                        self.write(").");
                        self.write(func);
                        self.write("(");
                        self.write(&right_rendered);
                        self.write(")");
                    } else {
                        self.write("std::cmp::");
                        self.write(func);
                        self.write("(");
                        self.write(&left_rendered);
                        self.write(", ");
                        self.write(&right_rendered);
                        self.write(")");
                    }
                }
                Ok(true)
            }
            "sorted" => {
                let [arg] = args else {
                    return Ok(false);
                };
                let saved_stats = self.lowering_stats;
                let Some(arg_rendered) = self.try_render_structured_expr(arg)? else {
                    return Ok(false);
                };
                self.lowering_stats = saved_stats;
                let Type::List(elem_ty) = crate::resolve_alias_type_for_plain_call(arg.ty()) else {
                    return Ok(false);
                };
                self.write("{ let mut _v = (");
                self.write(&arg_rendered);
                self.write(").iter().cloned().collect::<Vec<_>>(); ");
                if matches!(
                    crate::resolve_alias_type_for_plain_call(elem_ty),
                    Type::Float
                ) {
                    self.write("_v.sort_by(f64::total_cmp); ");
                } else {
                    self.write("_v.sort(); ");
                }
                self.write("_v }");
                Ok(true)
            }
            "any" => {
                let [arg] = args else {
                    return Ok(false);
                };
                let saved_stats = self.lowering_stats;
                let Some(arg_rendered) = self.try_render_structured_expr(arg)? else {
                    return Ok(false);
                };
                self.lowering_stats = saved_stats;
                self.write("(");
                self.write(&arg_rendered);
                self.write(").iter().any(|x| *x)");
                Ok(true)
            }
            "all" => {
                let [arg] = args else {
                    return Ok(false);
                };
                let saved_stats = self.lowering_stats;
                let Some(arg_rendered) = self.try_render_structured_expr(arg)? else {
                    return Ok(false);
                };
                self.lowering_stats = saved_stats;
                self.write("(");
                self.write(&arg_rendered);
                self.write(").iter().all(|x| *x)");
                Ok(true)
            }
            "sum" => {
                let [arg] = args else {
                    return Ok(false);
                };
                let saved_stats = self.lowering_stats;
                let Some(arg_rendered) = self.try_render_structured_expr(arg)? else {
                    return Ok(false);
                };
                self.lowering_stats = saved_stats;
                self.write("(");
                self.write(&arg_rendered);
                self.write(").iter().cloned().sum()");
                Ok(true)
            }
            "reversed" => {
                let [arg] = args else {
                    return Ok(false);
                };
                let saved_stats = self.lowering_stats;
                let Some(arg_rendered) = self.try_render_structured_expr(arg)? else {
                    return Ok(false);
                };
                self.lowering_stats = saved_stats;
                self.write("(");
                self.write(&arg_rendered);
                self.write(").iter().cloned().rev().collect::<Vec<_>>()");
                Ok(true)
            }
            "enumerate" => {
                let [arg] = args else {
                    return Ok(false);
                };
                let saved_stats = self.lowering_stats;
                let Some(arg_rendered) = self.try_render_structured_expr(arg)? else {
                    return Ok(false);
                };
                self.lowering_stats = saved_stats;
                self.write("(");
                self.write(&arg_rendered);
                self.write(").iter().cloned().enumerate().map(|(i, v)| (i as i64, v)).collect::<Vec<_>>()");
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

        let mut lhs_expr = left;
        let mut rendered_chain = String::new();
        for (idx, op) in ops.iter().enumerate() {
            let Some(rhs_expr) = comparators.get(idx) else {
                return Ok(false);
            };
            let lhs_saved_stats = self.lowering_stats;
            let Some(lhs_rendered) = self.try_render_structured_expr(lhs_expr)? else {
                return Ok(false);
            };
            self.lowering_stats = lhs_saved_stats;
            let rhs_saved_stats = self.lowering_stats;
            let Some(rhs_rendered) = self.try_render_structured_expr(rhs_expr)? else {
                return Ok(false);
            };
            self.lowering_stats = rhs_saved_stats;
            let lowered_op = match op.as_str() {
                "==" | "!=" | "<" | "<=" | ">" | ">=" => op.as_str(),
                "is" => "==",
                "is not" => "!=",
                _ => return Ok(false),
            };
            if idx > 0 {
                rendered_chain.push_str(" && ");
            }
            let lhs_wrapped = if lhs_rendered.contains(" as ") {
                format!("({lhs_rendered})")
            } else {
                lhs_rendered
            };
            let rhs_wrapped = if rhs_rendered.contains(" as ") {
                format!("({rhs_rendered})")
            } else {
                rhs_rendered
            };
            rendered_chain.push('(');
            rendered_chain.push_str(&lhs_wrapped);
            rendered_chain.push(' ');
            rendered_chain.push_str(lowered_op);
            rendered_chain.push(' ');
            rendered_chain.push_str(&rhs_wrapped);
            rendered_chain.push(')');
            lhs_expr = rhs_expr;
        }

        self.write(&rendered_chain);
        Ok(true)
    }

    pub(crate) fn try_emit_structured_question_mark_expr(
        &mut self,
        inner: &HirExpr,
    ) -> Result<bool, crate::CodegenError> {
        let saved_stats = self.lowering_stats;
        let Some(inner_rendered) = self.try_render_structured_expr(inner)? else {
            return Ok(false);
        };
        self.lowering_stats = saved_stats;
        self.write(&inner_rendered);
        self.write("?");
        Ok(true)
    }

    pub(crate) fn try_emit_structured_result_wrap_expr(
        &mut self,
        wrapper: &str,
        value: &HirExpr,
    ) -> Result<bool, crate::CodegenError> {
        let saved_stats = self.lowering_stats;
        let Some(value_rendered) = self.try_render_structured_expr(value)? else {
            return Ok(false);
        };
        self.lowering_stats = saved_stats;
        self.write(wrapper);
        self.write("(");
        self.write(&value_rendered);
        self.write(")");
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

        let mut rendered_values = Vec::with_capacity(values.len());
        for value in values {
            let saved_stats = self.lowering_stats;
            let Some(rendered) = self.try_render_structured_expr(value)? else {
                return Ok(false);
            };
            self.lowering_stats = saved_stats;
            rendered_values.push(rendered);
        }

        self.write("(");
        for (idx, rendered) in rendered_values.iter().enumerate() {
            if idx > 0 {
                self.write(" ");
                self.write(lowered_op);
                self.write(" ");
            }
            self.write("(");
            self.write(rendered);
            self.write(")");
        }
        self.write(")");
        Ok(true)
    }

    pub(crate) fn try_emit_structured_constructor_call_expr(
        &mut self,
        class_name: &str,
        args: &[HirExpr],
    ) -> Result<bool, crate::CodegenError> {
        let mut rendered_args = Vec::with_capacity(args.len());
        for arg in args {
            let saved_stats = self.lowering_stats;
            let Some(rendered) = self.try_render_structured_expr(arg)? else {
                return Ok(false);
            };
            self.lowering_stats = saved_stats;
            rendered_args.push(rendered);
        }

        self.write(class_name);
        self.write("::new(");
        for (idx, rendered_arg) in rendered_args.iter().enumerate() {
            if idx > 0 {
                self.write(", ");
            }
            self.write(rendered_arg);
        }
        self.write(")");
        Ok(true)
    }

    pub(crate) fn try_emit_structured_list_literal_expr(
        &mut self,
        elements: &[HirExpr],
    ) -> Result<bool, crate::CodegenError> {
        let mut rendered_elements = Vec::with_capacity(elements.len());
        for element in elements {
            let saved_stats = self.lowering_stats;
            let Some(rendered) = self.try_render_structured_expr(element)? else {
                return Ok(false);
            };
            self.lowering_stats = saved_stats;
            rendered_elements.push(rendered);
        }

        self.write("vec![");
        for (idx, rendered_element) in rendered_elements.iter().enumerate() {
            if idx > 0 {
                self.write(", ");
            }
            self.write(rendered_element);
        }
        self.write("]");
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
            let saved_stats = self.lowering_stats;
            let Some(key_rendered) = self.try_render_structured_expr(key)? else {
                return Ok(false);
            };
            self.lowering_stats = saved_stats;
            let Some(value_rendered) = self.try_render_structured_expr(value)? else {
                return Ok(false);
            };
            self.lowering_stats = saved_stats;
            entries.push((key_rendered, value_rendered));
        }
        self.write("HashMap::from([");
        for (idx, (key_rendered, value_rendered)) in entries.iter().enumerate() {
            if idx > 0 {
                self.write(", ");
            }
            self.write("(");
            self.write(key_rendered);
            self.write(", ");
            self.write(value_rendered);
            self.write(")");
        }
        self.write("])");
        Ok(true)
    }

    pub(crate) fn try_emit_structured_set_literal_expr(
        &mut self,
        elements: &[HirExpr],
    ) -> Result<bool, crate::CodegenError> {
        let mut rendered_elements = Vec::with_capacity(elements.len());
        for element in elements {
            let saved_stats = self.lowering_stats;
            let Some(rendered) = self.try_render_structured_expr(element)? else {
                return Ok(false);
            };
            self.lowering_stats = saved_stats;
            rendered_elements.push(rendered);
        }

        self.write("HashSet::from([");
        for (idx, rendered_element) in rendered_elements.iter().enumerate() {
            if idx > 0 {
                self.write(", ");
            }
            self.write(rendered_element);
        }
        self.write("])");
        Ok(true)
    }

    pub(crate) fn try_emit_structured_pre_call_expr(
        &mut self,
        expr: &HirExpr,
    ) -> Result<bool, crate::CodegenError> {
        match expr {
            HirExpr::Name { name, .. } => {
                self.write(name);
                Ok(true)
            }
            HirExpr::UnaryOp { op, operand, .. } => {
                self.try_emit_structured_unary_expr(op, operand)
            }
            HirExpr::BoolOp { op, values, .. } => self.try_emit_structured_bool_op_expr(op, values),
            HirExpr::QuestionMark { expr: inner, .. } => {
                self.try_emit_structured_question_mark_expr(inner)
            }
            HirExpr::OkWrap { value, .. } => {
                self.try_emit_structured_result_wrap_expr("Ok", value)
            }
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
        let saved_stats = self.lowering_stats;
        let Some(rendered_value) = self.try_render_structured_expr(value)? else {
            return Ok(false);
        };
        self.lowering_stats = saved_stats;
        self.write("{ let ");
        self.write(name);
        self.write(" = ");
        self.write(&rendered_value);
        self.write("; ");
        self.write(name);
        self.write(" }");
        Ok(true)
    }

    pub(crate) fn try_emit_structured_contains_expr(
        &mut self,
        element: &HirExpr,
        collection: &HirExpr,
    ) -> Result<bool, crate::CodegenError> {
        let saved_stats = self.lowering_stats;
        let Some(element_rendered) = self.try_render_structured_expr(element)? else {
            return Ok(false);
        };
        self.lowering_stats = saved_stats;

        let Some(collection_rendered) = self.try_render_structured_expr(collection)? else {
            return Ok(false);
        };
        self.lowering_stats = saved_stats;

        match crate::resolve_alias_type_for_plain_call(collection.ty()) {
            Type::Dict(_, _) => {
                self.write("(");
                self.write(&collection_rendered);
                self.write(").contains_key(&(");
                self.write(&element_rendered);
                self.write("))");
                Ok(true)
            }
            Type::List(_) | Type::Set(_) | Type::Str => {
                self.write("(");
                self.write(&collection_rendered);
                self.write(").contains(&(");
                self.write(&element_rendered);
                self.write("))");
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn try_emit_structured_unary_expr(
        &mut self,
        op: &str,
        operand: &HirExpr,
    ) -> Result<bool, crate::CodegenError> {
        let saved_stats = self.lowering_stats;
        let Some(operand_rendered) = self.try_render_structured_expr(operand)? else {
            return Ok(false);
        };
        self.lowering_stats = saved_stats;

        match op {
            "not" => {
                self.write("!(");
                self.write(&operand_rendered);
                self.write(")");
                Ok(true)
            }
            "-" => {
                self.write("-(");
                self.write(&operand_rendered);
                self.write(")");
                Ok(true)
            }
            "+" => {
                self.write("(");
                self.write(&operand_rendered);
                self.write(")");
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

        let left_saved_stats = self.lowering_stats;
        let Some(left_rendered) = self.try_render_structured_expr(left)? else {
            return Ok(false);
        };
        self.lowering_stats = left_saved_stats;
        let right_saved_stats = self.lowering_stats;
        let Some(right_rendered) = self.try_render_structured_expr(right)? else {
            return Ok(false);
        };
        self.lowering_stats = right_saved_stats;

        if op == "**" {
            match (
                crate::resolve_alias_type_for_plain_call(left.ty()),
                crate::resolve_alias_type_for_plain_call(right.ty()),
            ) {
                (Type::BigInt, _) => {
                    self.write("(");
                    self.write(&left_rendered);
                    self.write(").pow(u32::try_from(");
                    self.write(&right_rendered);
                    self.write(").unwrap_or(0))");
                }
                (Type::Int | Type::LiteralInt(_), Type::Int | Type::LiteralInt(_)) => {
                    self.write("(");
                    self.write(&left_rendered);
                    self.write(").pow((");
                    self.write(&right_rendered);
                    self.write(") as u32)");
                }
                (Type::Float, Type::Int | Type::LiteralInt(_)) => {
                    self.write("(");
                    self.write(&left_rendered);
                    self.write(").powi((");
                    self.write(&right_rendered);
                    self.write(") as i32)");
                }
                _ => {
                    self.write("((");
                    self.write(&left_rendered);
                    self.write(") as f64).powf((");
                    self.write(&right_rendered);
                    self.write(") as f64)");
                }
            }
            return Ok(true);
        }

        let rendered_op = if op == "//" { "/" } else { op };
        if matches!(crate::resolve_alias_type_for_plain_call(ty), Type::BigInt) {
            let left_rendered = if matches!(left, HirExpr::Name { .. } | HirExpr::FieldAccess { .. })
            {
                format!("({left_rendered}).clone()")
            } else {
                left_rendered
            };
            let right_rendered =
                if matches!(right, HirExpr::Name { .. } | HirExpr::FieldAccess { .. }) {
                    format!("({right_rendered}).clone()")
                } else {
                    right_rendered
                };
            self.write("(");
            self.write(&left_rendered);
            self.write(" ");
            self.write(rendered_op);
            self.write(" ");
            self.write(&right_rendered);
            self.write(")");
            return Ok(true);
        }
        self.write("(");
        self.write(&left_rendered);
        self.write(" ");
        self.write(rendered_op);
        self.write(" ");
        self.write(&right_rendered);
        self.write(")");
        Ok(true)
    }

    pub(crate) fn try_emit_structured_if_expr(
        &mut self,
        condition: &HirExpr,
        then_expr: &HirExpr,
        else_expr: &HirExpr,
    ) -> Result<bool, crate::CodegenError> {
        let saved_stats = self.lowering_stats;
        let Some(rendered_condition) = self.try_render_structured_expr(condition)? else {
            return Ok(false);
        };
        self.lowering_stats = saved_stats;
        let Some(rendered_then) = self.try_render_structured_expr(then_expr)? else {
            return Ok(false);
        };
        self.lowering_stats = saved_stats;
        let Some(rendered_else) = self.try_render_structured_expr(else_expr)? else {
            return Ok(false);
        };
        self.lowering_stats = saved_stats;

        self.write("if ");
        self.write(&rendered_condition);
        self.write(" { ");
        self.write(&rendered_then);
        self.write(" } else { ");
        self.write(&rendered_else);
        self.write(" }");
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
        let Type::Class { methods, .. } = crate::resolve_alias_type_for_plain_call(left.ty()) else {
            return Ok(None);
        };
        let Some((_, method_sig)) = methods.iter().find(|(name, _)| name == method_name) else {
            return Ok(None);
        };

        let lowered_left = if let HirExpr::FieldAccess {
            object,
            field,
            ty,
        } = left
        {
            self.try_lower_structured_field_access_expr(object, field, ty)?
        } else {
            crate::try_lower_leaf_or_name_expr_result(left)?
        };
        let Some(lowered_left) = lowered_left else {
            return Ok(None);
        };

        let lowered_right = if let HirExpr::FieldAccess {
            object,
            field,
            ty,
        } = right
        {
            self.try_lower_structured_field_access_expr(object, field, ty)?
        } else {
            crate::try_lower_leaf_or_name_expr_result(right)?
        };
        let Some(lowered_right) = lowered_right else {
            return Ok(None);
        };

        let arg_expr = if method_sig
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

        Ok(Some(crate::RustExpr::MethodCall {
            receiver: Box::new(lowered_left),
            method: method_name.to_string(),
            args: vec![arg_expr],
        }))
    }

    pub(crate) fn try_lower_structured_index_expr(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let object_ty = crate::resolve_alias_type_for_plain_call(object.ty());
        if !matches!(object_ty, Type::Dict(_, _) | Type::List(_) | Type::Str) {
            return Ok(None);
        }

        let suppress_self_field_clone = matches!(object, HirExpr::FieldAccess { object: inner, .. }
            if matches!(inner.as_ref(), HirExpr::Name { name, .. } if name == "self"))
            && self.pending_self_field_clone_suppression == 0;
        if suppress_self_field_clone {
            self.pending_self_field_clone_suppression += 1;
        }

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

        let Some(lowered_object) = lowered_object else {
            if suppress_self_field_clone && self.pending_self_field_clone_suppression > 0 {
                self.pending_self_field_clone_suppression -= 1;
            }
            return Ok(None);
        };

        let Some(lowered_index) = crate::try_lower_leaf_or_name_expr_result(index)? else {
            if suppress_self_field_clone && self.pending_self_field_clone_suppression > 0 {
                self.pending_self_field_clone_suppression -= 1;
            }
            return Ok(None);
        };

        let lowered_expr = match object_ty {
            Type::Dict(_, _) => {
                let key_arg = if let HirExpr::StringLiteral(value) = index {
                    crate::RustExpr::Ident(format!("{value:?}"))
                } else {
                    crate::RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(lowered_index),
                    }
                };
                crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(lowered_object),
                        method: "get".to_string(),
                        args: vec![key_arg],
                    }),
                    method: "cloned".to_string(),
                    args: vec![],
                }
            }
            Type::List(_) => crate::RustExpr::MethodCall {
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
            },
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
            _ => unreachable!(),
        };

        Ok(Some(lowered_expr))
    }

    pub(crate) fn try_emit_structured_index_expr(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
    ) -> Result<bool, crate::CodegenError> {
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
        if !matches!(object_ty, Type::Dict(_, _) | Type::List(_) | Type::Str | Type::Tuple(_))
            && !matches!(option_inner_ty, Some(Type::Dict(_, _) | Type::List(_) | Type::Str))
        {
            return Ok(false);
        }

        let suppress_self_field_clone = matches!(object, HirExpr::FieldAccess { object: inner, .. }
            if matches!(inner.as_ref(), HirExpr::Name { name, .. } if name == "self"))
            && self.pending_self_field_clone_suppression == 0;
        if suppress_self_field_clone {
            self.pending_self_field_clone_suppression += 1;
        }

        let saved_stats = self.lowering_stats;
        let Some(object_rendered) = self.try_render_structured_expr(object)? else {
            if suppress_self_field_clone && self.pending_self_field_clone_suppression > 0 {
                self.pending_self_field_clone_suppression -= 1;
            }
            return Ok(false);
        };
        self.lowering_stats = saved_stats;
        if suppress_self_field_clone && self.pending_self_field_clone_suppression > 0 {
            self.pending_self_field_clone_suppression -= 1;
        }

        let saved_stats = self.lowering_stats;
        let Some(index_rendered) = self.try_render_structured_expr(index)? else {
            return Ok(false);
        };
        self.lowering_stats = saved_stats;

        if let Some(inner_ty) = option_inner_ty {
            self.write("(");
            self.write(&object_rendered);
            self.write(").as_ref().and_then(|__v| ");
            match inner_ty {
                Type::Dict(_, _) => {
                    self.write("__v.get(");
                    if let HirExpr::StringLiteral(value) = index {
                        self.write(&format!("{value:?}"));
                    } else {
                        self.write("&(");
                        self.write(&index_rendered);
                        self.write(")");
                    }
                    self.write(").cloned()");
                }
                Type::List(_) => {
                    self.write("__v.get((");
                    self.write(&index_rendered);
                    self.write(") as usize).cloned()");
                }
                Type::Str => {
                    self.write("__v.chars().nth((");
                    self.write(&index_rendered);
                    self.write(") as usize).map(|c| c.to_string())");
                }
                _ => return Ok(false),
            }
            self.write(")");
            return Ok(true);
        }

        match object_ty {
            Type::Tuple(elements) => {
                let HirExpr::IntLiteral(idx) = index else {
                    return Ok(false);
                };
                let Ok(idx) = usize::try_from(*idx) else {
                    return Ok(false);
                };
                if idx >= elements.len() {
                    return Ok(false);
                }
                self.write("(");
                self.write(&object_rendered);
                self.write(").");
                self.write(&idx.to_string());
            }
            Type::Dict(_, _) => {
                self.write(&object_rendered);
                self.write(".get(");
                if let HirExpr::StringLiteral(value) = index {
                    self.write(&format!("{value:?}"));
                } else {
                    self.write("&(");
                    self.write(&index_rendered);
                    self.write(")");
                }
                self.write(").cloned()");
            }
            Type::List(_) => {
                self.write(&object_rendered);
                self.write(".get((");
                self.write(&index_rendered);
                self.write(") as usize).cloned()");
            }
            Type::Str => {
                self.write(&object_rendered);
                self.write(".chars().nth((");
                self.write(&index_rendered);
                self.write(") as usize).map(|c| c.to_string())");
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    pub(crate) fn try_emit_structured_slice_expr(
        &mut self,
        object: &HirExpr,
        start: Option<&HirExpr>,
        stop: Option<&HirExpr>,
        step: Option<&HirExpr>,
    ) -> Result<bool, crate::CodegenError> {
        let object_ty = crate::resolve_alias_type_for_plain_call(object.ty());
        if !matches!(object_ty, Type::Str | Type::List(_)) {
            return Ok(false);
        }

        let saved_stats = self.lowering_stats;
        let Some(object_rendered) = self.try_render_structured_expr(object)? else {
            return Ok(false);
        };
        self.lowering_stats = saved_stats;

        let start_rendered = if let Some(start) = start {
            let saved_stats = self.lowering_stats;
            let Some(rendered) = self.try_render_structured_expr(start)? else {
                return Ok(false);
            };
            self.lowering_stats = saved_stats;
            Some(rendered)
        } else {
            None
        };

        let stop_rendered = if let Some(stop) = stop {
            let saved_stats = self.lowering_stats;
            let Some(rendered) = self.try_render_structured_expr(stop)? else {
                return Ok(false);
            };
            self.lowering_stats = saved_stats;
            Some(rendered)
        } else {
            None
        };

        let step_rendered = if let Some(step) = step {
            let saved_stats = self.lowering_stats;
            let Some(rendered) = self.try_render_structured_expr(step)? else {
                return Ok(false);
            };
            self.lowering_stats = saved_stats;
            Some(rendered)
        } else {
            None
        };

        match object_ty {
            Type::Str => {
                self.write("{ let _s = &(");
                self.write(&object_rendered);
                self.write("); let _len = _s.chars().count() as i64; ");

                self.write("let _start = ");
                if let Some(start_rendered) = &start_rendered {
                    self.write("{ let _sv = ");
                    self.write(start_rendered);
                    self.write("; if _sv < 0 { ((_len + _sv).max(0)) as usize } else { (_sv.min(_len)) as usize } }");
                } else {
                    self.write("0_usize");
                }
                self.write("; ");

                self.write("let _stop = ");
                if let Some(stop_rendered) = &stop_rendered {
                    self.write("{ let _ev = ");
                    self.write(stop_rendered);
                    self.write("; if _ev < 0 { ((_len + _ev).max(0)) as usize } else { (_ev.min(_len)) as usize } }");
                } else {
                    self.write("_len as usize");
                }
                self.write("; ");

                if let Some(step_rendered) = &step_rendered {
                    self.write("let _step = { let _st = ");
                    self.write(step_rendered);
                    self.write("; if _st <= 0 { 1_usize } else { _st as usize } }; ");
                    self.write("_s.chars().skip(_start).take(_stop.saturating_sub(_start)).step_by(_step).collect::<String>() }");
                } else {
                    self.write("_s.chars().skip(_start).take(_stop - _start).collect::<String>() }");
                }
                Ok(true)
            }
            Type::List(_) => {
                self.write("{ let _v = &(");
                self.write(&object_rendered);
                self.write("); let _len = _v.len() as i64; ");

                self.write("let _start = ");
                if let Some(start_rendered) = &start_rendered {
                    self.write("{ let _s = ");
                    self.write(start_rendered);
                    self.write("; if _s < 0 { ((_len + _s).max(0)) as usize } else { (_s.min(_len)) as usize } }");
                } else {
                    self.write("0_usize");
                }
                self.write("; ");

                self.write("let _stop = ");
                if let Some(stop_rendered) = &stop_rendered {
                    self.write("{ let _e = ");
                    self.write(stop_rendered);
                    self.write("; if _e < 0 { ((_len + _e).max(0)) as usize } else { (_e.min(_len)) as usize } }");
                } else {
                    self.write("_len as usize");
                }
                self.write("; ");

                if let Some(step_rendered) = &step_rendered {
                    self.write("let _step = { let _st = ");
                    self.write(step_rendered);
                    self.write("; if _st <= 0 { 1_usize } else { _st as usize } }; ");
                    self.write("_v.iter().skip(_start).take(_stop.saturating_sub(_start)).step_by(_step).cloned().collect::<Vec<_>>() }");
                } else {
                    self.write("_v[_start.._stop].to_vec() }");
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub(crate) fn try_emit_structured_string_concat_expr(
        &mut self,
        left: &HirExpr,
        op: &str,
        right: &HirExpr,
        ty: &Type,
    ) -> Result<bool, crate::CodegenError> {
        if op != "+"
            || !matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Str)
        {
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
            self.write(&format!("{combined:?}.to_string()"));
            return Ok(true);
        }

        self.write("format!(\"");
        for part in &parts {
            if let HirExpr::StringLiteral(value) = part {
                for ch in value.chars() {
                    match ch {
                        '{' => self.write("{{"),
                        '}' => self.write("}}"),
                        '"' => self.write("\\\""),
                        '\\' => self.write("\\\\"),
                        _ => self.write(&ch.to_string()),
                    }
                }
            } else {
                self.write("{}");
            }
        }
        self.write("\"");
        for part in &parts {
            if !matches!(part, HirExpr::StringLiteral(_)) {
                let Some(rendered) = self.try_render_structured_expr(part)? else {
                    return Ok(false);
                };
                self.write(", ");
                self.write(&rendered);
            }
        }
        self.write(")");
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

        let (string_expr, count_expr) =
            match (
                crate::resolve_alias_type_for_plain_call(left.ty()),
                crate::resolve_alias_type_for_plain_call(right.ty()),
            ) {
                (Type::Str, Type::Int | Type::LiteralInt(_)) => (left, right),
                (Type::Int | Type::LiteralInt(_), Type::Str) => (right, left),
                _ => return Ok(false),
            };

        let saved_stats = self.lowering_stats;
        let Some(rendered_string) = self.try_render_structured_expr(string_expr)? else {
            return Ok(false);
        };
        self.lowering_stats = saved_stats;
        let Some(rendered_count) = self.try_render_structured_expr(count_expr)? else {
            return Ok(false);
        };
        self.lowering_stats = saved_stats;

        self.write("{ let __n = ");
        self.write(&rendered_count);
        self.write("; if __n <= 0 { String::new() } else { (");
        self.write(&rendered_string);
        self.write(").repeat(__n as usize) } }");
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

        let saved_stats = self.lowering_stats;
        let Some(rendered_left) = self.try_render_structured_expr(left)? else {
            return Ok(false);
        };
        self.lowering_stats = saved_stats;
        let Some(rendered_right) = self.try_render_structured_expr(right)? else {
            return Ok(false);
        };
        self.lowering_stats = saved_stats;

        self.write("{ let mut __v = (");
        self.write(&rendered_left);
        self.write(").clone(); __v.extend((");
        self.write(&rendered_right);
        self.write(").iter().cloned()); __v }");
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

fn render_expr_contains_force_guard_name(emitter: &RustEmitter, expr: &HirExpr) -> bool {
    match expr {
        HirExpr::Name { name, .. } => {
            emitter.intrinsic_functions.contains(name.as_str()) && !is_stdlib_math_constant(name)
        }
        HirExpr::BinOp { left, right, .. } => {
            render_expr_contains_force_guard_name(emitter, left)
                || render_expr_contains_force_guard_name(emitter, right)
        }
        HirExpr::UnaryOp { operand, .. } => {
            render_expr_contains_force_guard_name(emitter, operand)
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            render_expr_contains_force_guard_name(emitter, left)
                || comparators
                    .iter()
                    .any(|expr| render_expr_contains_force_guard_name(emitter, expr))
        }
        HirExpr::BoolOp { values, .. } => values
            .iter()
            .any(|expr| render_expr_contains_force_guard_name(emitter, expr)),
        HirExpr::Call { args, .. } => args
            .iter()
            .any(|expr| render_expr_contains_force_guard_name(emitter, expr)),
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            render_expr_contains_force_guard_name(emitter, condition)
                || render_expr_contains_force_guard_name(emitter, then_expr)
                || render_expr_contains_force_guard_name(emitter, else_expr)
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            render_expr_contains_force_guard_name(emitter, start)
                || render_expr_contains_force_guard_name(emitter, end)
                || step
                    .as_ref()
                    .is_some_and(|expr| render_expr_contains_force_guard_name(emitter, expr))
        }
        HirExpr::ListLiteral { elements, .. }
        | HirExpr::SetLiteral { elements, .. }
        | HirExpr::TupleLiteral { elements, .. } => elements
            .iter()
            .any(|expr| render_expr_contains_force_guard_name(emitter, expr)),
        HirExpr::DictLiteral { keys, values, .. } => {
            keys.iter()
                .any(|expr| render_expr_contains_force_guard_name(emitter, expr))
                || values
                    .iter()
                    .any(|expr| render_expr_contains_force_guard_name(emitter, expr))
        }
        HirExpr::Index { object, index, .. } => {
            render_expr_contains_force_guard_name(emitter, object)
                || render_expr_contains_force_guard_name(emitter, index)
        }
        HirExpr::MethodCall { object, args, .. } => {
            render_expr_contains_force_guard_name(emitter, object)
                || args
                    .iter()
                    .any(|expr| render_expr_contains_force_guard_name(emitter, expr))
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => {
            render_expr_contains_force_guard_name(emitter, element)
                || render_expr_contains_force_guard_name(emitter, collection)
        }
        HirExpr::FString { parts, .. } => parts.iter().any(|part| {
            matches!(
                part,
                HirFStringPart::Expr(expr)
                    if render_expr_contains_force_guard_name(emitter, expr)
            )
        }),
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            render_expr_contains_force_guard_name(emitter, object)
                || start
                    .as_ref()
                    .is_some_and(|expr| render_expr_contains_force_guard_name(emitter, expr))
                || stop
                    .as_ref()
                    .is_some_and(|expr| render_expr_contains_force_guard_name(emitter, expr))
                || step
                    .as_ref()
                    .is_some_and(|expr| render_expr_contains_force_guard_name(emitter, expr))
        }
        HirExpr::WalrusExpr { value, .. } => {
            render_expr_contains_force_guard_name(emitter, value)
        }
        HirExpr::FieldAccess { object, .. } => {
            render_expr_contains_force_guard_name(emitter, object)
        }
        HirExpr::ConstructorCall { args, .. } => args
            .iter()
            .any(|expr| render_expr_contains_force_guard_name(emitter, expr)),
        HirExpr::QuestionMark { expr, .. } => {
            render_expr_contains_force_guard_name(emitter, expr)
        }
        HirExpr::OkWrap { value, .. } | HirExpr::ErrWrap { value, .. } => {
            render_expr_contains_force_guard_name(emitter, value)
        }
        HirExpr::SuperCall { args, .. } => args
            .iter()
            .any(|expr| render_expr_contains_force_guard_name(emitter, expr)),
        HirExpr::Lambda { body, .. } => render_expr_contains_force_guard_name(emitter, body),
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            render_expr_contains_force_guard_name(emitter, expr)
                || generators.iter().any(|(_, iter_expr, maybe_filter)| {
                    render_expr_contains_force_guard_name(emitter, iter_expr)
                        || maybe_filter.as_ref().is_some_and(|filter| {
                            render_expr_contains_force_guard_name(emitter, filter)
                        })
                })
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            render_expr_contains_force_guard_name(emitter, key_expr)
                || render_expr_contains_force_guard_name(emitter, val_expr)
                || generators.iter().any(|(_, iter_expr, maybe_filter)| {
                    render_expr_contains_force_guard_name(emitter, iter_expr)
                        || maybe_filter.as_ref().is_some_and(|filter| {
                            render_expr_contains_force_guard_name(emitter, filter)
                        })
                })
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            render_expr_contains_force_guard_name(emitter, expr)
                || render_expr_contains_force_guard_name(emitter, iter)
                || filter
                    .as_ref()
                    .is_some_and(|cond| render_expr_contains_force_guard_name(emitter, cond))
        }
        HirExpr::IntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::EnumVariant { .. } => false,
    }
}

fn is_stdlib_math_constant(name: &str) -> bool {
    matches!(name, "pi" | "e" | "tau" | "inf" | "nan")
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
    pub(super) fn emit_lambda_untyped(&mut self, expr: &HirExpr) {
        if let HirExpr::Lambda { params, body, .. } = expr {
            self.write("|");
            for (i, param) in params.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                self.write(&param.name);
            }
            self.write("| ");
            self.emit_expr(body);
        } else {
            // Not a lambda, emit as-is
            self.emit_expr(expr);
        }
    }

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
        self.write(macro_name);
        self.write("(\"");
        self.write(&format_str);
        self.write("\"");
        for expr in &exprs {
            self.write(", ");
            self.emit_display_expr(expr);
        }
        self.write(")");
    }
}

fn render_expr_uses_borrowed_param(
    expr: &HirExpr,
    borrowed_params: &std::collections::HashSet<String>,
    mut_borrowed_params: &std::collections::HashSet<String>,
) -> bool {
    match expr {
        HirExpr::Name { name, .. } => {
            borrowed_params.contains(name) || mut_borrowed_params.contains(name)
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            render_expr_uses_borrowed_param(left, borrowed_params, mut_borrowed_params)
                || comparators.iter().any(|c| {
                    render_expr_uses_borrowed_param(c, borrowed_params, mut_borrowed_params)
                })
        }
        HirExpr::BoolOp { values, .. } => values
            .iter()
            .any(|v| render_expr_uses_borrowed_param(v, borrowed_params, mut_borrowed_params)),
        HirExpr::UnaryOp { operand, .. } => {
            render_expr_uses_borrowed_param(operand, borrowed_params, mut_borrowed_params)
        }
        HirExpr::BinOp { left, right, .. } => {
            render_expr_uses_borrowed_param(left, borrowed_params, mut_borrowed_params)
                || render_expr_uses_borrowed_param(right, borrowed_params, mut_borrowed_params)
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            render_expr_uses_borrowed_param(condition, borrowed_params, mut_borrowed_params)
                || render_expr_uses_borrowed_param(then_expr, borrowed_params, mut_borrowed_params)
                || render_expr_uses_borrowed_param(else_expr, borrowed_params, mut_borrowed_params)
        }
        _ => false,
    }
}
