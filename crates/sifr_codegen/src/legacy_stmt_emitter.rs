use crate::helpers::{
    body_calls_function, codegen_body_always_exits, collect_locally_defined_vars,
    collect_mutated_vars, collect_referenced_vars_with_types, detect_and_not_none_vars,
    detect_is_none_union_var, detect_is_none_var, detect_is_not_none_var, detect_isinstance_union,
    detect_option_truthiness, find_union_variant, is_option_type, stmts_reference_var,
    try_body_has_value_return,
};
use crate::RustEmitter;
use sifr_hir::{HirExpr, HirStmt};
use sifr_type_system::Type;
use std::collections::HashSet;

impl RustEmitter {
    pub(super) fn emit_stmt_legacy(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Let {
                name,
                ty,
                value,
                is_mutable: _,
            } => {
                self.write_indent();
                // Only emit `mut` if the variable is actually mutated later
                if self.mutated_vars.contains(name) {
                    self.write("let mut ");
                } else {
                    self.write("let ");
                }
                self.write(name);
                // Skip explicit type annotation for generic class instances (let Rust infer)
                let is_generic_class = matches!(ty, Type::Class { name: ref cn, .. } if self.generic_classes.contains(cn));
                if !is_generic_class {
                    self.write(": ");
                    self.write(&ty.rust_type());
                }
                self.write(" = ");
                if matches!(ty, Type::None) && matches!(value, HirExpr::NoneLiteral) {
                    // `x: None = None` -> `let x: () = ()`
                    self.write("()");
                } else if matches!(ty, Type::BigInt) && matches!(value, HirExpr::IntLiteral(_)) {
                    // `x: bigint = 42` -> `BigInt::from(42_i64)`
                    if let HirExpr::IntLiteral(v) = value {
                        self.write(&format!("BigInt::from({v}_i64)"));
                    }
                } else if is_option_type(ty) && matches!(value, HirExpr::NoneLiteral) {
                    // `x: str | None = None` -> `let x: Option<String> = None`
                    self.write("None");
                } else if is_option_type(ty)
                    && !is_option_type(value.ty())
                    && !matches!(value.ty(), Type::None)
                {
                    // RHS is a plain value (not already Option) -> wrap in Some()
                    // But if RHS is a function call returning Option, don't double-wrap
                    self.write("Some(");
                    self.emit_expr(value);
                    self.write(")");
                } else {
                    // Check if RHS is a call to a generator function and target is list[T]
                    let needs_collect =
                        matches!(ty, Type::List(_)) && self.is_generator_call(value);
                    self.emit_expr(value);
                    if needs_collect {
                        self.write(".collect()");
                    }
                    // Clone borrowed TypeVar params assigned to owned TypeVar locals
                    let needs_clone_for_typevar = matches!(ty, Type::TypeVar(_))
                        && if let HirExpr::Name {
                            name: ref vname, ..
                        } = value
                        {
                            self.borrowed_params.contains(vname.as_str())
                        } else {
                            false
                        };
                    if needs_clone_for_typevar {
                        self.write(".clone()");
                    }
                }
                self.write(";\n");
            }
            HirStmt::Assign { name, value } => {
                self.write_indent();
                self.write(name);
                self.write(" = ");
                self.emit_expr(value);
                // Clone borrowed TypeVar params reassigned to owned TypeVar locals
                if matches!(value.ty(), Type::TypeVar(_)) {
                    if let HirExpr::Name {
                        name: ref vname, ..
                    } = value
                    {
                        if self.borrowed_params.contains(vname.as_str()) {
                            self.write(".clone()");
                        }
                    }
                }
                self.write(";\n");
            }
            HirStmt::AugAssign { name, op, value } => {
                self.write_indent();
                let var_ty = value.ty();
                match op.as_str() {
                    "+=" => {
                        // Special cases for string and list
                        match var_ty {
                            Type::Str => {
                                self.write(name);
                                self.write(".push_str(");
                                self.emit_str_ref_expr(value);
                                self.write(");\n");
                                return;
                            }
                            _ => {
                                // Check if target is a list (we need to look at the value context)
                                // For list += list, use extend
                                if let Type::List(_) = var_ty {
                                    self.write(name);
                                    self.write(".extend(");
                                    self.emit_expr(value);
                                    self.write(");\n");
                                    return;
                                }
                            }
                        }
                        self.write(name);
                        self.write(" += ");
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                    "-=" | "*=" | "%=" => {
                        self.write(name);
                        self.write(&format!(" {op} "));
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                    "/=" => {
                        self.write(name);
                        self.write(" /= ");
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                    "//=" => {
                        self.write(name);
                        self.write(" /= ");
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                    "**=" => {
                        // Power assignment: x **= y
                        // If the value (exponent) is int, use i64::pow for int targets
                        if matches!(var_ty, Type::Int) {
                            self.write(name);
                            self.write(" = ");
                            self.write(&format!("{name}.pow("));
                            self.emit_expr(value);
                            self.write(" as u32);\n");
                        } else {
                            self.write(name);
                            self.write(" = ");
                            self.write(&format!("({name} as f64).powf("));
                            self.emit_expr(value);
                            self.write(" as f64);\n");
                        }
                    }
                    _ => {
                        self.write(name);
                        self.write(&format!(" {op} "));
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                }
            }
            HirStmt::Return { value } => {
                // Inside Display::fmt (for __str__ methods), return statements become
                // write!(f, "{}", val); return Ok(())
                if self.emission_ctx.in_display_impl {
                    if let Some(val) = value {
                        self.write_indent();
                        self.write("write!(f, \"{}\", ");
                        self.emit_expr(val);
                        self.write(")?;\n");
                        self.write_indent();
                        self.write("return Ok(());\n");
                    } else {
                        self.write_indent();
                        self.write("return Ok(());\n");
                    }
                    return;
                }
                let ret_is_option = self
                    .current_return_type
                    .as_ref()
                    .is_some_and(is_option_type);
                let ret_is_non_option_union = self
                    .current_return_type
                    .as_ref()
                    .is_some_and(|t| matches!(t, Type::Union(_)) && !is_option_type(t));
                self.write_indent();
                if let Some(val) = value {
                    self.write("return ");
                    if ret_is_option && matches!(val, HirExpr::NoneLiteral) {
                        // `return None` in Python -> `return None` in Rust Option
                        self.write("None");
                    } else if ret_is_option && !is_option_type(val.ty()) {
                        // Returning a non-Option value from an Option function -> wrap in Some()
                        self.write("Some(");
                        self.emit_expr(val);
                        self.write(")");
                    } else if ret_is_non_option_union {
                        // Returning a value from a non-Option union function -> wrap in enum variant
                        if let Some(ret_ty) = &self.current_return_type.clone() {
                            if let Type::Union(members) = ret_ty {
                                let arg_ty = val.ty();
                                if let Some(variant) = find_union_variant(members, arg_ty) {
                                    let enum_name = ret_ty.union_enum_name();
                                    self.write(&format!("{enum_name}::{variant}("));
                                    self.emit_expr(val);
                                    self.write(")");
                                } else {
                                    self.emit_expr(val);
                                }
                            } else {
                                self.emit_expr(val);
                            }
                        } else {
                            self.emit_expr(val);
                        }
                    } else if !ret_is_option
                        && is_option_type(val.ty())
                        && !matches!(val.ty(), Type::None)
                    {
                        // Returning an Option value from a non-Option function -> unwrap
                        // This happens with generic functions where T is inferred as a concrete type
                        // but the body has safe-indexing that returns Option<T>
                        self.emit_expr(val);
                        self.write(".unwrap()");
                    } else if matches!(val.ty(), Type::TypeVar(_)) {
                        // Returning a TypeVar-typed value needs .clone() to avoid move from &self
                        self.emit_expr(val);
                        self.write(".clone()");
                    } else if self.current_class_name.is_some() {
                        // Inside a class method: if returning `self` (a Name expr),
                        // we need .clone() because methods take &self in Rust
                        if let HirExpr::Name { name, .. } = val {
                            if name == "self" {
                                self.emit_expr(val);
                                self.write(".clone()");
                            } else {
                                self.emit_expr(val);
                            }
                        } else {
                            self.emit_expr(val);
                        }
                    } else {
                        self.emit_expr(val);
                    }
                    self.write(";\n");
                } else {
                    if ret_is_option {
                        self.write("return None;\n");
                    } else {
                        self.write("return;\n");
                    }
                }
            }
            HirStmt::Expr { expr } => {
                self.write_indent();
                self.emit_expr(expr);
                self.write(";\n");
            }
            HirStmt::If {
                condition,
                then_body,
                elif_clauses,
                else_body,
            } => {
                // Detect isinstance narrowing for union enums:
                // `if isinstance(x, int):` -> `match x { IntOrStr::Int(x) => { ... }, IntOrStr::Str(x) => { ... } }`
                if let Some((var_name, variant_name, enum_name, other_variants)) =
                    detect_isinstance_union(condition)
                {
                    self.write_indent();
                    self.write(&format!("match {var_name} {{\n"));
                    self.indent += 1;

                    // Then branch: the matched variant
                    let then_mutated = collect_mutated_vars(then_body);
                    let var_mut = if then_mutated.contains(&var_name) {
                        "mut "
                    } else {
                        ""
                    };
                    self.write_indent();
                    self.write(&format!(
                        "{enum_name}::{variant_name}({var_mut}{var_name}) => {{\n"
                    ));
                    self.indent += 1;
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                    self.writeln("}");

                    // Emit elif isinstance branches as additional match arms
                    let mut remaining_variants = other_variants.clone();
                    for (elif_cond, elif_body) in elif_clauses {
                        if let Some((_, elif_variant, _, _)) = detect_isinstance_union(elif_cond) {
                            let elif_mutated = collect_mutated_vars(elif_body);
                            let elif_var_mut = if elif_mutated.contains(&var_name) {
                                "mut "
                            } else {
                                ""
                            };
                            self.write_indent();
                            self.write(&format!(
                                "{enum_name}::{elif_variant}({elif_var_mut}{var_name}) => {{\n"
                            ));
                            self.indent += 1;
                            for s in elif_body {
                                self.emit_stmt(s);
                            }
                            self.indent -= 1;
                            self.writeln("}");
                            // Remove this variant from remaining
                            remaining_variants.retain(|(v, _)| v != &elif_variant);
                        }
                    }

                    // Else branch: remaining variant(s)
                    if let Some(else_stmts) = else_body {
                        let else_mutated = collect_mutated_vars(else_stmts);
                        let else_var_mut = if else_mutated.contains(&var_name) {
                            "mut "
                        } else {
                            ""
                        };
                        if remaining_variants.len() == 1 {
                            let (other_variant, _) = &remaining_variants[0];
                            self.write_indent();
                            self.write(&format!(
                                "{enum_name}::{other_variant}({else_var_mut}{var_name}) => {{\n"
                            ));
                        } else {
                            self.write_indent();
                            self.write("_ => {\n");
                        }
                        self.indent += 1;
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.indent -= 1;
                        self.writeln("}");
                    } else {
                        // No else body: add wildcard arm so match is exhaustive
                        self.write_indent();
                        self.write("_ => {}\n");
                    }

                    self.indent -= 1;
                    self.writeln("}");
                }
                // Detect truthiness on Option: `if x:` where x is Option -> `if let Some(x) = x {`
                else if let Some(var_name) = detect_option_truthiness(condition) {
                    self.write_indent();
                    self.write(&format!("if let Some({var_name}) = {var_name} {{\n"));
                    self.indent += 1;
                    self.option_unwrapped_vars.insert(var_name.clone());
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.option_unwrapped_vars.remove(&var_name);
                    self.indent -= 1;

                    if let Some(else_stmts) = else_body {
                        self.write_indent();
                        self.write("} else {\n");
                        self.indent += 1;
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.indent -= 1;
                    }
                    self.writeln("}");
                }
                // Detect compound `a is not None and b is not None` -> nested if let Some
                else if let Some(vars) = detect_and_not_none_vars(condition) {
                    // Emit nested if-let-Some for each variable
                    for (i, var_name) in vars.iter().enumerate() {
                        self.write_indent();
                        self.write(&format!("if let Some({var_name}) = {var_name} {{\n"));
                        self.indent += 1;
                        self.option_unwrapped_vars.insert(var_name.clone());
                        if i < vars.len() - 1 {
                            // More variables to unwrap, continue nesting
                        }
                    }
                    // Emit the then-body inside the innermost block
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    // Close all nested blocks
                    for var_name in vars.iter().rev() {
                        self.option_unwrapped_vars.remove(var_name);
                        self.indent -= 1;
                        if let Some(else_stmts) = else_body {
                            if var_name == vars.first().unwrap() {
                                // Only emit else on the outermost block
                                self.write_indent();
                                self.write("} else {\n");
                                self.indent += 1;
                                for s in else_stmts {
                                    self.emit_stmt(s);
                                }
                                self.indent -= 1;
                            }
                        }
                        self.writeln("}");
                    }
                }
                // Detect Option narrowing: `if x is not None:` -> `if let Some(x) = x {`
                else if let Some(var_name) = detect_is_not_none_var(condition) {
                    self.write_indent();
                    // Use `if let Some(var) = var` to unwrap and shadow the variable
                    self.write(&format!("if let Some({var_name}) = {var_name} {{\n"));
                    self.indent += 1;
                    self.option_unwrapped_vars.insert(var_name.clone());
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.option_unwrapped_vars.remove(&var_name);
                    self.indent -= 1;

                    if let Some(else_stmts) = else_body {
                        self.write_indent();
                        self.write("} else {\n");
                        self.indent += 1;
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.indent -= 1;
                    }
                    self.writeln("}");
                } else if let Some((var_name, enum_name, _non_none_variants)) =
                    detect_is_none_union_var(condition)
                {
                    // 3+ member union `is None` check: use match with None variant
                    self.write_indent();
                    self.write(&format!("match {var_name} {{\n"));
                    self.indent += 1;

                    // None arm -> then_body
                    self.write_indent();
                    self.write(&format!("{enum_name}::None(()) => {{\n"));
                    self.indent += 1;
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                    self.writeln("}");

                    // Non-None arms -> else_body
                    if let Some(else_stmts) = else_body {
                        self.write_indent();
                        self.write("_ => {\n");
                        self.indent += 1;
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.indent -= 1;
                        self.writeln("}");
                    } else {
                        // Need a catch-all arm even without else
                        self.write_indent();
                        self.write("_ => {}\n");
                    }

                    self.indent -= 1;
                    self.writeln("}");
                } else if let Some(var_name) = detect_is_none_var(condition) {
                    self.write_indent();
                    self.write(&format!("if {var_name}.is_none() {{\n"));
                    self.indent += 1;
                    let then_exits = codegen_body_always_exits(then_body);
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;

                    if let Some(else_stmts) = else_body {
                        // In the else branch of `if x is None`, x is not None
                        self.write_indent();
                        self.write(&format!(
                            "}} else if let Some({var_name}) = {var_name} {{\n"
                        ));
                        self.indent += 1;
                        self.option_unwrapped_vars.insert(var_name.clone());
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.option_unwrapped_vars.remove(&var_name);
                        self.indent -= 1;
                    }
                    self.writeln("}");

                    // Early-return narrowing: if the then-body always exits (return/break),
                    // unwrap the variable after the if block so subsequent code can use it directly
                    if then_exits && else_body.is_none() {
                        self.write_indent();
                        self.write(&format!("let {var_name} = {var_name}.unwrap();\n"));
                        self.option_unwrapped_vars.insert(var_name.clone());
                    }
                } else {
                    // Normal if/elif/else
                    // Hoist any walrus expressions before the if
                    self.emit_walrus_hoists(condition);
                    self.write_indent();
                    self.write("if ");
                    self.emit_expr(condition);
                    self.write(" {\n");
                    self.indent += 1;
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;

                    for (cond, body) in elif_clauses {
                        self.write_indent();
                        self.write("} else if ");
                        self.emit_expr(cond);
                        self.write(" {\n");
                        self.indent += 1;
                        for s in body {
                            self.emit_stmt(s);
                        }
                        self.indent -= 1;
                    }

                    if let Some(else_stmts) = else_body {
                        self.write_indent();
                        self.write("} else {\n");
                        self.indent += 1;
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.indent -= 1;
                    }

                    self.writeln("}");
                }
            }
            HirStmt::While {
                condition,
                body,
                else_body,
            } => {
                let has_else = else_body.is_some();
                if has_else {
                    self.writeln("let mut _broke = false;");
                }
                self.loop_else_stack.push(has_else);
                // Hoist any walrus expressions
                self.emit_walrus_hoists(condition);
                self.write_indent();
                self.write("while ");
                self.emit_expr(condition);
                self.write(" {\n");
                self.indent += 1;
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.writeln("}");
                let popped = self.loop_else_stack.pop();
                debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
                if let Some(else_stmts) = else_body {
                    self.writeln("if !_broke {");
                    self.indent += 1;
                    for s in else_stmts {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                    self.writeln("}");
                }
            }
            HirStmt::For {
                target,
                iter,
                body,
                else_body,
                ..
            } => {
                let has_else = else_body.is_some();
                if has_else {
                    self.writeln("let mut _broke = false;");
                }
                self.loop_else_stack.push(has_else);
                self.write_indent();
                self.write("for ");
                // Handle tuple unpacking: "i,v" -> "(i, v)"
                if target.contains(',') {
                    let names: Vec<&str> = target.split(',').collect();
                    self.write("(");
                    for (i, name) in names.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.write(name);
                    }
                    self.write(")");
                } else {
                    self.write(target);
                }
                self.write(" in ");
                // For lists, iterate with .iter() to borrow and clone elements
                // But not for generator expressions which are already iterators
                let is_generator_expr = matches!(iter, HirExpr::GeneratorExpr { .. });
                let is_generator_fn_call = self.is_generator_call(iter);
                let is_list = matches!(iter.ty(), Type::List(_));
                let is_dict = matches!(iter.ty(), Type::Dict(_, _));
                let is_str = matches!(iter.ty(), Type::Str);
                self.emit_expr(iter);
                if is_generator_expr || is_generator_fn_call {
                    // Generator expressions and generator function calls are already iterators
                } else if is_list {
                    self.write(".iter().cloned()");
                } else if is_dict {
                    self.write(".keys().cloned()");
                } else if is_str {
                    self.write(".chars().map(|c| c.to_string())");
                }
                self.write(" {\n");
                self.indent += 1;
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.writeln("}");
                let popped = self.loop_else_stack.pop();
                debug_assert!(popped.is_some(), "loop_else_stack should not underflow");
                if let Some(else_stmts) = else_body {
                    self.writeln("if !_broke {");
                    self.indent += 1;
                    for s in else_stmts {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                    self.writeln("}");
                }
            }
            HirStmt::Break => {
                if self.current_loop_has_else() {
                    self.writeln("_broke = true;");
                }
                self.writeln("break;");
            }
            HirStmt::Continue => {
                self.writeln("continue;");
            }
            HirStmt::Pass => {
                // No-op in Rust
            }
            HirStmt::TupleUnpack { targets, value } => {
                self.write_indent();
                self.write("let (");
                for (i, (name, _ty)) in targets.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(name);
                }
                self.write(") = ");
                self.emit_expr(value);
                self.write(";\n");
            }
            HirStmt::StarUnpack {
                before,
                star,
                after,
                value,
            } => {
                // Emit: let _tmp = value.clone() to avoid moving;
                self.write_indent();
                self.write("let _star_tmp = ");
                self.emit_expr(value);
                self.write(".clone();\n");
                // Emit before vars
                for (i, (name, _ty)) in before.iter().enumerate() {
                    self.write_indent();
                    self.write(&format!("let {name} = _star_tmp[{i}].clone();\n"));
                }
                // Emit star var
                let (star_name, _star_ty) = star;
                if after.is_empty() {
                    self.write_indent();
                    self.write(&format!(
                        "let {} = _star_tmp[{}..].to_vec();\n",
                        star_name,
                        before.len()
                    ));
                } else {
                    self.write_indent();
                    self.write(&format!(
                        "let {} = _star_tmp[{}.._star_tmp.len() - {}].to_vec();\n",
                        star_name,
                        before.len(),
                        after.len()
                    ));
                }
                // Emit after vars
                for (i, (name, _ty)) in after.iter().enumerate() {
                    self.write_indent();
                    self.write(&format!(
                        "let {} = _star_tmp[_star_tmp.len() - {}].clone();\n",
                        name,
                        after.len() - i
                    ));
                }
            }
            HirStmt::TryExcept {
                body,
                handlers,
                body_error_types,
            } => {
                // Helper: map IOError subclass names to their Rust kind string
                fn io_subclass_kind(name: &str) -> Option<&'static str> {
                    match name {
                        "FileNotFoundError" => Some("FileNotFound"),
                        "PermissionError" => Some("PermissionDenied"),
                        "FileExistsError" => Some("FileExists"),
                        "IsADirectoryError" => Some("IsADirectory"),
                        "NotADirectoryError" => Some("NotADirectory"),
                        "DirectoryNotEmptyError" => Some("DirectoryNotEmpty"),
                        _ => None,
                    }
                }

                // Map IOError subclass names to their parent type for Rust codegen
                fn rust_error_type(name: &str) -> &str {
                    if io_subclass_kind(name).is_some() {
                        "IOError"
                    } else {
                        name
                    }
                }

                // Collect distinct Rust error types from handlers and body
                let mut error_type_names: Vec<String> = Vec::new();
                let mut has_catch_all = false;
                for handler in handlers {
                    if let Some(ref et) = handler.error_type {
                        if et == "Error" {
                            has_catch_all = true;
                        } else {
                            let rust_ty = rust_error_type(et).to_string();
                            if !error_type_names.contains(&rust_ty) {
                                error_type_names.push(rust_ty);
                            }
                        }
                    } else {
                        has_catch_all = true;
                    }
                }
                // If catch-all only (no specific handlers), use body error types
                if error_type_names.is_empty() && has_catch_all {
                    for et in body_error_types {
                        if et != "Error" {
                            let rust_ty = rust_error_type(et).to_string();
                            if !error_type_names.contains(&rust_ty) {
                                error_type_names.push(rust_ty);
                            }
                        }
                    }
                }

                // Check if any handler catches an IOError subclass specifically
                let has_io_subclass_handler = handlers.iter().any(|h| {
                    h.error_type
                        .as_ref()
                        .is_some_and(|et| io_subclass_kind(et).is_some())
                });

                let needs_enum = error_type_names.len() > 1;

                if needs_enum {
                    // Multi-error-type try block: generate a local error enum
                    self.try_enum_counter += 1;
                    let enum_name = format!("_TryErr{}", self.try_enum_counter);

                    // Emit enum definition
                    self.write_indent();
                    self.write("#[allow(non_camel_case_types)]\n");
                    self.write_indent();
                    self.write(&format!("enum {enum_name} {{\n"));
                    self.indent += 1;
                    for et in &error_type_names {
                        self.write_indent();
                        self.write(&format!("{et}({et}),\n"));
                    }
                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");

                    // Emit From impls for each error type
                    for et in &error_type_names {
                        self.write_indent();
                        self.write(&format!("impl From<{et}> for {enum_name} {{\n"));
                        self.indent += 1;
                        self.write_indent();
                        self.write(&format!(
                            "fn from(e: {et}) -> Self {{ {enum_name}::{et}(e) }}\n"
                        ));
                        self.indent -= 1;
                        self.write_indent();
                        self.write("}\n");
                    }

                    // Emit try body as a closure
                    // Check if the try body contains a return statement with a value.
                    let body_has_return_multi = try_body_has_value_return(body);
                    let (closure_ok_type_multi, ok_arm_multi) = if body_has_return_multi {
                        let inner_ty = self
                            .current_return_type
                            .as_ref()
                            .and_then(|t| {
                                if let Type::Result(ok, _) = t {
                                    Some(ok.rust_type())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| "_".to_string());
                        (
                            inner_ty,
                            "Ok(__try_ret) => { return Ok(__try_ret); }".to_string(),
                        )
                    } else {
                        ("()".to_string(), "Ok(()) => {}".to_string())
                    };

                    self.write_indent();
                    self.write(&format!(
                        "match (|| -> Result<{closure_ok_type_multi}, {enum_name}> {{\n"
                    ));
                    self.indent += 1;
                    for stmt in body {
                        self.emit_stmt(stmt);
                    }
                    self.write_indent();
                    if body_has_return_multi {
                        self.write("unreachable!()\n");
                    } else {
                        self.write("Ok(())\n");
                    }
                    self.indent -= 1;
                    self.write_indent();
                    self.write("})() {\n");
                    self.indent += 1;
                    self.write_indent();
                    self.write(&format!("{ok_arm_multi}\n"));

                    // Emit match arms
                    for handler in handlers {
                        if let Some(ref et) = handler.error_type {
                            if et == "Error" {
                                // Catch-all: match on any remaining variant
                                let var_name = handler.name.as_deref().unwrap_or("_e");
                                self.write_indent();
                                self.write(&format!("Err({var_name}) => {{\n"));
                                if handler.name.is_some() {
                                    self.indent += 1;
                                    self.write_indent();
                                    self.indent -= 1;
                                }
                            } else if let Some(kind) = io_subclass_kind(et) {
                                // IOError subclass: match on the parent enum variant with a guard
                                let var_name = handler.name.as_deref().unwrap_or("_e");
                                self.write_indent();
                                self.write(&format!(
                                    "Err({enum_name}::IOError(ref {var_name})) if {var_name}.kind == \"{kind}\" => {{\n"
                                ));
                                // Clone the variable so handler body can use it as owned
                                if handler.name.is_some() {
                                    self.indent += 1;
                                    self.write_indent();
                                    self.write(&format!("let {var_name} = {var_name}.clone();\n"));
                                    self.indent -= 1;
                                }
                            } else if et == "IOError" && has_io_subclass_handler {
                                // IOError parent catch-all (when subclass handlers exist)
                                let var_name = handler.name.as_deref().unwrap_or("_e");
                                self.write_indent();
                                self.write(&format!(
                                    "Err({enum_name}::IOError({var_name})) => {{\n"
                                ));
                            } else {
                                let var_name = handler.name.as_deref().unwrap_or("_e");
                                self.write_indent();
                                self.write(&format!("Err({enum_name}::{et}({var_name})) => {{\n"));
                            }
                        } else {
                            // Bare except — catch-all
                            let var_name = handler.name.as_deref().unwrap_or("_e");
                            self.write_indent();
                            self.write(&format!("Err({var_name}) => {{\n"));
                        }
                        self.indent += 1;
                        for stmt in &handler.body {
                            self.emit_stmt(stmt);
                        }
                        self.indent -= 1;
                        self.write_indent();
                        self.write("}\n");
                    }

                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");
                } else {
                    // Single error type: use simple codegen
                    let error_rust_type = if let Some(first_body_err) = error_type_names.first() {
                        first_body_err.clone()
                    } else {
                        handlers
                            .first()
                            .and_then(|h| h.error_resolved_type.as_ref())
                            .map(|t| {
                                let rt = t.rust_type();
                                // Map IOError subclass resolved types to IOError
                                if io_subclass_kind(&rt).is_some() {
                                    "IOError".to_string()
                                } else {
                                    rt
                                }
                            })
                            .unwrap_or_else(|| "String".to_string())
                    };

                    // Check if the try body contains a return statement with a value.
                    // If so, the closure must return Result<T, E> instead of Result<(), E>.
                    let body_has_return = try_body_has_value_return(body);
                    let (closure_ok_type, ok_arm) = if body_has_return {
                        // Use the function's return type's inner type for the closure
                        let inner_ty = self
                            .current_return_type
                            .as_ref()
                            .and_then(|t| {
                                if let Type::Result(ok, _) = t {
                                    Some(ok.rust_type())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| "_".to_string());
                        (
                            inner_ty.clone(),
                            "Ok(__try_ret) => { return Ok(__try_ret); }".to_string(),
                        )
                    } else {
                        ("()".to_string(), "Ok(()) => {}".to_string())
                    };

                    self.write_indent();
                    self.write(&format!(
                        "match (|| -> Result<{closure_ok_type}, {error_rust_type}> {{\n"
                    ));
                    self.indent += 1;
                    for stmt in body {
                        self.emit_stmt(stmt);
                    }
                    self.write_indent();
                    if body_has_return {
                        self.write("unreachable!()\n");
                    } else {
                        self.write("Ok(())\n");
                    }
                    self.indent -= 1;
                    self.write_indent();
                    self.write("})() {\n");
                    self.indent += 1;
                    self.write_indent();
                    self.write(&format!("{ok_arm}\n"));

                    if has_io_subclass_handler && error_rust_type == "IOError" {
                        // IOError with subclass dispatch: use guard-based matching
                        for handler in handlers {
                            if let Some(ref et) = handler.error_type {
                                if et == "Error" || et == "IOError" {
                                    // Parent catch-all
                                    let var_name = handler.name.as_deref().unwrap_or("_e");
                                    self.write_indent();
                                    self.write(&format!("Err({var_name}) => {{\n"));
                                } else if let Some(kind) = io_subclass_kind(et) {
                                    // Subclass match with guard
                                    let var_name = handler.name.as_deref().unwrap_or("_e");
                                    self.write_indent();
                                    self.write(&format!(
                                        "Err(ref {var_name}) if {var_name}.kind == \"{kind}\" => {{\n"
                                    ));
                                    // Clone the variable so handler body can use it as owned
                                    if handler.name.is_some() {
                                        self.indent += 1;
                                        self.write_indent();
                                        self.write(&format!(
                                            "let {var_name} = {var_name}.clone();\n"
                                        ));
                                        self.indent -= 1;
                                    }
                                } else {
                                    let var_name = handler.name.as_deref().unwrap_or("_e");
                                    self.write_indent();
                                    self.write(&format!("Err({var_name}) => {{\n"));
                                }
                            } else {
                                let var_name = handler.name.as_deref().unwrap_or("_e");
                                self.write_indent();
                                self.write(&format!("Err({var_name}) => {{\n"));
                            }
                            self.indent += 1;
                            for stmt in &handler.body {
                                self.emit_stmt(stmt);
                            }
                            self.indent -= 1;
                            self.write_indent();
                            self.write("}\n");
                        }
                    } else {
                        // No subclass dispatch needed — simple match
                        for handler in handlers {
                            self.write_indent();
                            if let Some(ref name) = handler.name {
                                self.write(&format!("Err({name}) => {{\n"));
                            } else {
                                self.write("Err(_e) => {\n");
                            }
                            self.indent += 1;
                            for stmt in &handler.body {
                                self.emit_stmt(stmt);
                            }
                            self.indent -= 1;
                            self.write_indent();
                            self.write("}\n");
                        }
                    }

                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");
                }
            }
            HirStmt::Raise { value } => {
                self.write_indent();
                self.write("return Err(");
                self.emit_expr(value);
                self.write(");\n");
            }
            HirStmt::Assert { test, msg } => {
                self.write_indent();
                if let Some(msg_expr) = msg {
                    self.write("assert!(");
                    self.emit_expr(test);
                    self.write(", \"{}\", ");
                    self.emit_display_expr(msg_expr);
                    self.write(");\n");
                } else {
                    self.write("assert!(");
                    self.emit_expr(test);
                    self.write(");\n");
                }
            }
            HirStmt::FieldAssign {
                object,
                field,
                value,
            } => {
                self.write_indent();
                // Check if this is assigning to a parent field via inheritance
                if let Some(ref class_name) = self.current_class_name.clone() {
                    if let Some((parent_name, parent_field_names)) =
                        self.parent_fields.get(class_name).cloned()
                    {
                        if parent_field_names.contains(field.as_str()) {
                            self.write(object);
                            self.write(".");
                            self.write(&parent_name.to_lowercase());
                            self.write(".");
                            self.write(field);
                            self.write(" = ");
                            self.emit_expr(value);
                            self.write(";\n");
                            return;
                        }
                    }
                }
                self.write(object);
                self.write(".");
                self.write(field);
                self.write(" = ");
                // deque._data = [] → VecDeque::new()
                if self.current_class_name.as_deref() == Some("deque") && field == "_data" {
                    if let HirExpr::ListLiteral { elements, .. } = value {
                        if elements.is_empty() {
                            self.write("VecDeque::new()");
                            self.write(";\n");
                            return;
                        }
                    }
                }
                self.emit_expr(value);
                self.write(";\n");
            }
            HirStmt::SubscriptAssign {
                object,
                index,
                value,
                object_ty,
            } => {
                self.write_indent();
                match object_ty {
                    Type::List(_) => {
                        // list[i] = val -> bounds-checked assignment (safe no-op if out of bounds)
                        self.write("{ let __idx = ");
                        self.emit_expr(index);
                        self.write(" as usize; if let Some(__elem) = ");
                        self.write(object);
                        self.write(".get_mut(__idx) { *__elem = ");
                        self.emit_expr(value);
                        self.write("; } }\n");
                    }
                    Type::Dict(_, _) => {
                        // dict[key] = val -> dict.insert(key, val)
                        self.write(object);
                        self.write(".insert(");
                        self.emit_expr(index);
                        self.write(", ");
                        self.emit_expr(value);
                        self.write(");\n");
                    }
                    _ => {
                        // Fallback: direct subscript
                        self.write(object);
                        self.write("[");
                        self.emit_expr(index);
                        self.write("] = ");
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                }
            }
            HirStmt::NestedSubscriptAssign {
                object,
                outer_index,
                inner_index,
                value,
                object_ty: _,
            } => {
                self.write_indent();
                // matrix[i][j] = val -> bounds-checked nested assignment (safe no-op if out of bounds)
                self.write("{ let __oi = ");
                self.emit_expr(outer_index);
                self.write(" as usize; let __ii = ");
                self.emit_expr(inner_index);
                self.write(" as usize; if let Some(__row) = ");
                self.write(object);
                self.write(
                    ".get_mut(__oi) { if let Some(__elem) = __row.get_mut(__ii) { *__elem = ",
                );
                self.emit_expr(value);
                self.write("; } } }\n");
            }
            HirStmt::SubscriptAugAssign {
                object,
                index,
                op,
                value,
                object_ty: _,
            } => {
                self.write_indent();
                // list[i] += val -> bounds-checked augmented assignment (safe no-op if out of bounds)
                self.write("{ let __idx = ");
                self.emit_expr(index);
                self.write(" as usize; if let Some(__elem) = ");
                self.write(object);
                self.write(".get_mut(__idx) { ");
                // Convert **= to .pow() pattern
                if op == "**=" {
                    self.write("*__elem = __elem.pow(");
                    self.emit_expr(value);
                    self.write(" as u32);");
                } else if op == "//=" {
                    self.write("*__elem = *__elem / ");
                    self.emit_expr(value);
                    self.write(";");
                } else {
                    self.write("*__elem ");
                    self.write(op);
                    self.write(" ");
                    self.emit_expr(value);
                    self.write(";");
                }
                self.write(" } }\n");
            }
            HirStmt::AttributeAugAssign {
                object,
                field,
                op,
                value,
            } => {
                self.write_indent();
                self.write(object);
                self.write(".");
                self.write(field);
                self.write(&format!(" {op} "));
                self.emit_expr(value);
                self.write(";\n");
            }
            HirStmt::AttributeSubscriptAssign {
                object,
                field,
                index,
                value,
                field_ty,
            } => {
                self.write_indent();
                let field_access = format!("{object}.{field}");
                match field_ty {
                    Type::List(_) => {
                        // self.field[i] = val -> bounds-checked assignment
                        self.write("{ let __idx = ");
                        self.emit_expr(index);
                        self.write(" as usize; if let Some(__elem) = ");
                        self.write(&field_access);
                        self.write(".get_mut(__idx) { *__elem = ");
                        self.emit_expr(value);
                        self.write("; } }\n");
                    }
                    Type::Dict(ref key_ty, _) => {
                        // self.field[key] = val -> self.field.insert(key_owned, val)
                        // For move-type keys: if key is a borrowed param (&T), clone for owned insert.
                        self.write(&field_access);
                        self.write(".insert(");
                        let key_needs_clone =
                            matches!(key_ty.as_ref(), Type::Str | Type::TypeVar(_));
                        if key_needs_clone {
                            if let HirExpr::Name { name, .. } = index {
                                if self.borrowed_params.contains(name.as_str())
                                    || self.mut_borrowed_params.contains(name.as_str())
                                {
                                    self.emit_expr(index);
                                    self.write(".clone()");
                                } else {
                                    self.emit_expr(index);
                                }
                            } else {
                                self.emit_expr(index);
                            }
                        } else {
                            self.emit_expr(index);
                        }
                        self.write(", ");
                        self.emit_expr(value);
                        self.write(");\n");
                    }
                    _ => {
                        // Fallback: direct subscript
                        self.write(&field_access);
                        self.write("[");
                        self.emit_expr(index);
                        self.write("] = ");
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                }
            }
            HirStmt::Delete { object, index } => {
                let obj_ty = object.ty();
                self.write_indent();
                match obj_ty {
                    Type::Dict(_, _) => {
                        // del d[key] -> let _ = d.remove(&key);
                        self.write("let _ = ");
                        self.emit_expr(object);
                        self.write(".remove(");
                        self.emit_key_ref_expr(index);
                        self.write(");\n");
                    }
                    Type::List(_) => {
                        // del a[i] -> let _ = a.remove(i as usize);
                        self.write("let _ = ");
                        self.emit_expr(object);
                        self.write(".remove(");
                        self.emit_expr(index);
                        self.write(" as usize);\n");
                    }
                    _ => {
                        self.write("/* unsupported del */\n");
                    }
                }
            }
            HirStmt::Yield { value } => {
                if self.emission_ctx.in_generator_closure {
                    // Inside a generator closure: yield becomes return Some(val)
                    self.write_indent();
                    self.write("return Some(");
                    self.emit_expr(value);
                    self.write(");\n");
                } else {
                    // Eager fallback: push to yields vec
                    self.write_indent();
                    self.write("_yields.push(");
                    self.emit_expr(value);
                    self.write(");\n");
                }
            }
            HirStmt::With { items, body } => {
                self.write_indent();
                self.write("{\n");
                self.indent += 1;
                // Emit each context manager item with Drop-based cleanup
                // This ensures __exit__() is called on ALL exit paths:
                // normal completion, early return, break, continue
                for (i, (var, value, has_cm)) in items.iter().enumerate() {
                    let ctx_name = format!("__ctx_{i}");
                    let guard_type = format!("__WithGuard{i}");
                    let guard_var = format!("__guard_{i}");
                    if *has_cm {
                        // Extract the class type name for the guard struct
                        let class_name = if let Type::Class { name, .. } = value.ty() {
                            name.clone()
                        } else {
                            "Unknown".to_string()
                        };
                        // Create context manager variable
                        self.write_indent();
                        self.write("let mut ");
                        self.write(&ctx_name);
                        self.write(" = ");
                        self.emit_expr(value);
                        self.write(";\n");
                        // Emit Drop guard struct that calls __exit__() on scope exit
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
                        // Create guard instance, moving ctx into it
                        self.write_indent();
                        self.write(&format!(
                            "let mut {guard_var} = {guard_type} {{ ctx: {ctx_name} }};\n"
                        ));
                        // Call __enter__() on guard's ctx and bind result to var
                        self.write_indent();
                        if stmts_reference_var(body, var)
                            || items.iter().any(|(v, _, _)| v != var && v.contains(var))
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
                        // Fallback: no context manager protocol, just bind directly
                        self.write_indent();
                        if stmts_reference_var(body, var) {
                            self.write("let ");
                            self.write(var);
                        } else {
                            self.write("let _");
                            self.write(var);
                        }
                        self.write(" = ");
                        self.emit_expr(value);
                        self.write(";\n");
                    }
                }
                // Emit body
                for s in body {
                    self.emit_stmt(s);
                }
                // No explicit __exit__() calls needed — Drop guards handle cleanup
                self.indent -= 1;
                self.write_indent();
                self.write("}\n");
            }
            HirStmt::NestedFunction { func } => {
                let saved_return_type = self.current_return_type.clone();
                let saved_mutated = self.mutated_vars.clone();

                self.current_return_type = Some(func.return_type.clone());
                self.mutated_vars = collect_mutated_vars(&func.body);

                // Collect the set of parameter names
                let param_names: HashSet<String> =
                    func.params.iter().map(|p| p.name.clone()).collect();

                // Detect captured variables: variables referenced in body that are
                // not parameters and not defined locally in the body
                let referenced_with_types = collect_referenced_vars_with_types(&func.body);
                let locally_defined = collect_locally_defined_vars(&func.body);
                let captures: Vec<(String, Type)> = referenced_with_types
                    .into_iter()
                    .filter(|(v, _)| !param_names.contains(v) && !locally_defined.contains(v))
                    .collect();

                // Check if the nested function calls itself (recursive)
                let is_recursive = body_calls_function(&func.body, &func.name);

                if captures.is_empty() {
                    // No captures: emit as a plain inner fn (works for both recursive and non-recursive)
                    self.write_indent();
                    self.write("fn ");
                    self.write(&func.name);
                    self.write("(");

                    for (i, param) in func.params.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        if self.mutated_vars.contains(&param.name) {
                            self.write("mut ");
                        }
                        self.write(&param.name);
                        self.write(": ");
                        self.write(&param.ty.rust_type());
                    }

                    self.write(")");

                    if func.return_type != Type::None {
                        self.write(" -> ");
                        self.write(&func.return_type.rust_type());
                    }

                    self.write(" {\n");
                    self.indent += 1;

                    for s in &func.body {
                        self.emit_stmt(s);
                    }

                    self.indent -= 1;
                    self.writeln("}");
                } else if !is_recursive {
                    // Has captures but not recursive: emit as a closure
                    self.write_indent();
                    self.write("let ");
                    self.write(&func.name);
                    self.write(" = |");

                    for (i, param) in func.params.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        if self.mutated_vars.contains(&param.name) {
                            self.write("mut ");
                        }
                        self.write(&param.name);
                        self.write(": ");
                        self.write(&param.ty.rust_type());
                    }

                    self.write("|");

                    if func.return_type != Type::None {
                        self.write(" -> ");
                        self.write(&func.return_type.rust_type());
                    }

                    self.write(" {\n");
                    self.indent += 1;

                    for s in &func.body {
                        self.emit_stmt(s);
                    }

                    self.indent -= 1;
                    self.writeln("};");
                } else {
                    // Recursive AND captures: emit as inner fn with captured vars as extra cloned params
                    // Store the capture info so call sites can pass the extra args
                    self.nested_fn_captures
                        .insert(func.name.clone(), captures.clone());

                    self.write_indent();
                    self.write("fn ");
                    self.write(&func.name);
                    self.write("(");

                    for (i, param) in func.params.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        if self.mutated_vars.contains(&param.name) {
                            self.write("mut ");
                        }
                        self.write(&param.name);
                        self.write(": ");
                        self.write(&param.ty.rust_type());
                    }

                    // Add captured variables as extra parameters with types
                    for (cap_name, cap_ty) in &captures {
                        self.write(", ");
                        self.write(cap_name);
                        self.write(": ");
                        self.write(&cap_ty.rust_type());
                    }

                    self.write(")");

                    if func.return_type != Type::None {
                        self.write(" -> ");
                        self.write(&func.return_type.rust_type());
                    }

                    self.write(" {\n");
                    self.indent += 1;

                    for s in &func.body {
                        self.emit_stmt(s);
                    }

                    self.indent -= 1;
                    self.writeln("}");
                }

                self.current_return_type = saved_return_type;
                self.mutated_vars = saved_mutated;
            }
            HirStmt::Match {
                subject,
                subject_ty,
                arms,
            } => {
                self.emit_match(subject, subject_ty, arms);
            }
        }
    }
}
