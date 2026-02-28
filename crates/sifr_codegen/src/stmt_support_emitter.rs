use crate::{RustEmitter, RustStmt};
use sifr_hir::{HirExpr, HirStmt};
use sifr_type_system::Type;

impl RustEmitter {
    fn resolve_alias_type_for_loop_iter(ty: &Type) -> &Type {
        match ty {
            Type::Alias(_, inner) => Self::resolve_alias_type_for_loop_iter(inner),
            _ => ty,
        }
    }

    /// Emit a generator initialization statement (always mutable for closure capture)
    pub(super) fn emit_generator_init_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Let {
                name, ty, value, ..
            } => {
                self.write_indent();
                self.write("let mut ");
                self.write(name);
                self.write(": ");
                self.write(&ty.rust_type());
                self.write(" = ");
                match self.try_emit_structured_expr(value) {
                    Ok(true) => {}
                    Ok(false) | Err(_) => {
                        panic!(
                            "structured generator-init expression emission missing for production path: {value:?}"
                        );
                    }
                }
                self.write(";\n");
            }
            _ => {
                match self.try_emit_structured_stmt(stmt) {
                    Ok(true) => {}
                    Ok(false) | Err(_) => {
                        panic!(
                            "structured generator-init statement emission missing for production path: {stmt:?}"
                        );
                    }
                }
            }
        }
    }

    pub(super) fn emit_lowered_stmts(&mut self, lowered_stmts: &[RustStmt]) {
        for lowered_stmt in lowered_stmts {
            match lowered_stmt {
                RustStmt::Expr(lowered_expr) => {
                    self.write_indent();
                    self.write(&crate::render_expr(lowered_expr));
                    self.write(";\n");
                }
                RustStmt::RawCode(_) => {
                    panic!("RawCode statement reached core production emission path");
                }
                RustStmt::Break => {
                    self.writeln("break;");
                }
                RustStmt::Continue => {
                    self.writeln("continue;");
                }
                _ => {
                    self.write_indent();
                    let rendered = crate::render_stmts(std::slice::from_ref(lowered_stmt));
                    self.write(rendered.trim_end());
                    self.write("\n");
                }
            }
        }
    }

    pub(super) fn current_loop_has_else(&self) -> bool {
        self.loop_else_stack.last().copied().unwrap_or(false)
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
                self.try_emit_structured_expr(iter)?
            }
        } else {
            self.try_emit_structured_expr(iter)?
        };

        if !emitted_iter {
            let popped = self.loop_else_stack.pop();
            debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
            self.output.truncate(output_len);
            return Ok(false);
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
                let Type::Class { name: class_name, .. } = value.ty() else {
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

    pub(crate) fn try_emit_structured_try_except_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> bool {
        let HirStmt::TryExcept { body, handlers, .. } = stmt else {
            return false;
        };
        if handlers.is_empty() {
            return false;
        }
        let handler = &handlers[0];
        let err_ty = handler
            .error_resolved_type
            .as_ref()
            .map(|ty| crate::render_type(&crate::sifr_type_to_rust_type(ty)))
            .unwrap_or_else(|| "()".to_string());

        let output_len = self.output.len();
        self.write_indent();
        self.write("let __sifr_try_res: Result<(), ");
        self.write(&err_ty);
        self.write("> = (|| {\n");
        self.indent += 1;
        for try_stmt in body {
            self.emit_stmt(try_stmt);
        }
        self.write_indent();
        self.write("return Ok(());\n");
        self.indent -= 1;
        self.write_indent();
        self.write("})();\n");

        self.write_indent();
        self.write("if let Err(");
        self.write(handler.name.as_deref().unwrap_or("_e"));
        self.write(") = __sifr_try_res {\n");
        self.indent += 1;
        for handler_stmt in &handler.body {
            self.emit_stmt(handler_stmt);
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
            return false;
        }
        true
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

        let output_len = self.output.len();
        self.write_indent();
        self.write(object);
        self.write(".");
        self.write(field);
        self.write(" = ");
        if self.try_emit_structured_expr(value)? {
            self.write(";\n");
            return Ok(true);
        }
        self.output.truncate(output_len);
        Ok(false)
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

        let output_len = self.output.len();
        self.write_indent();
        self.write(object);
        self.write(".");
        self.write(field);
        self.write(".insert(");

        let key_needs_clone = matches!(key_ty.as_ref(), Type::Str | Type::TypeVar(_))
            && matches!(index, HirExpr::Name { name, .. }
                if self.borrowed_params.contains(name.as_str()) || self.mut_borrowed_params.contains(name.as_str()));
        if !self.try_emit_structured_expr(index)? {
            self.output.truncate(output_len);
            return Ok(false);
        }
        if key_needs_clone {
            self.write(".clone()");
        }

        self.write(", ");
        if !self.try_emit_structured_expr(value)? {
            self.output.truncate(output_len);
            return Ok(false);
        }
        self.write(");\n");
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
        if op == "**=" {
            let output_len = self.output.len();
            self.write_indent();
            self.write(name);
            self.write(" = (");
            self.write(name);
            self.write(").pow((");
            if !self.try_emit_structured_expr(value)? {
                self.output.truncate(output_len);
                return Ok(false);
            }
            self.write(") as u32);\n");
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

        let output_len = self.output.len();
        self.write_indent();
        self.write(name);
        self.write(" ");
        self.write(rust_op);
        self.write(" ");
        if !self.try_emit_structured_expr(value)? {
            self.output.truncate(output_len);
            return Ok(false);
        }
        self.write(";\n");
        Ok(true)
    }
}
