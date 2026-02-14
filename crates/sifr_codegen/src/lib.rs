//! Sifr Code Generation
//!
//! Translates the typed HIR into Rust source code.

use sifr_hir::*;
use sifr_type_system::Type;
use std::collections::{HashMap, HashSet};

/// Generate Rust source code from a HIR module.
pub fn generate_rust(module: &HirModule) -> String {
    let mut emitter = RustEmitter::new();

    // First pass: collect all union types used in the module
    emitter.collect_union_types(module);

    // Generate enum definitions for non-Option union types
    emitter.generate_enum_definitions();

    // Second pass: emit the actual code
    emitter.emit_module(module);

    let mut result = String::new();
    if emitter.needs_hashmap {
        result.push_str("use std::collections::HashMap;\n\n");
    }
    if !emitter.enum_defs.is_empty() {
        result.push_str(&emitter.enum_defs);
        result.push('\n');
    }
    result.push_str(&emitter.output);
    result
}

/// Generate a complete Rust project (Cargo.toml + main.rs content).
pub fn generate_project(module: &HirModule, project_name: &str) -> (String, String) {
    let cargo_toml = format!(
        r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2021"
"#
    );

    let main_rs = generate_rust(module);
    (cargo_toml, main_rs)
}

struct RustEmitter {
    output: String,
    indent: usize,
    needs_hashmap: bool,
    /// Track union enum types that need to be defined (name -> member types)
    union_enums: HashMap<String, Vec<Type>>,
    /// Accumulated enum definitions to prepend
    enum_defs: String,
    /// The return type of the function currently being emitted
    current_return_type: Option<Type>,
    /// Set of variable names currently narrowed via `if let Some(...)` unwrap
    option_unwrapped_vars: HashSet<String>,
    /// Function signatures: name -> (param_types, return_type)
    func_signatures: HashMap<String, (Vec<Type>, Type)>,
}

impl RustEmitter {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            needs_hashmap: false,
            union_enums: HashMap::new(),
            enum_defs: String::new(),
            current_return_type: None,
            option_unwrapped_vars: HashSet::new(),
            func_signatures: HashMap::new(),
        }
    }

    /// Collect all union types from the module that need enum definitions,
    /// and build a map of function signatures for call-site wrapping.
    fn collect_union_types(&mut self, module: &HirModule) {
        for func in &module.functions {
            // Record function signature
            let param_types: Vec<Type> = func.params.iter().map(|p| p.ty.clone()).collect();
            self.func_signatures.insert(func.name.clone(), (param_types, func.return_type.clone()));

            // Check params
            for param in &func.params {
                self.register_union_type(&param.ty);
            }
            // Check return type
            self.register_union_type(&func.return_type);
            // Check body statements
            self.collect_union_types_in_stmts(&func.body);
        }
    }

    fn collect_union_types_in_stmts(&mut self, stmts: &[HirStmt]) {
        for stmt in stmts {
            match stmt {
                HirStmt::Let { ty, .. } => self.register_union_type(ty),
                HirStmt::If { then_body, elif_clauses, else_body, .. } => {
                    self.collect_union_types_in_stmts(then_body);
                    for (_, body) in elif_clauses {
                        self.collect_union_types_in_stmts(body);
                    }
                    if let Some(body) = else_body {
                        self.collect_union_types_in_stmts(body);
                    }
                }
                HirStmt::While { body, .. } | HirStmt::For { body, .. } => {
                    self.collect_union_types_in_stmts(body);
                }
                _ => {}
            }
        }
    }

    fn register_union_type(&mut self, ty: &Type) {
        if let Type::Union(members) = ty {
            // Skip Option<T> pattern (T | None with exactly 2 members)
            let non_none: Vec<&Type> = members.iter().filter(|m| !matches!(m, Type::None)).collect();
            let has_none = members.iter().any(|m| matches!(m, Type::None));
            if has_none && non_none.len() == 1 {
                return; // This maps to Option<T>, no enum needed
            }
            // Register the enum name and its member types
            let enum_name = ty.union_enum_name();
            self.union_enums.entry(enum_name).or_insert_with(|| members.clone());
        }
    }

    /// Generate Rust enum definitions for all collected union types.
    fn generate_enum_definitions(&mut self) {
        // Sort enum names for deterministic output
        let mut enums: Vec<(String, Vec<Type>)> = self.union_enums.clone().into_iter().collect();
        enums.sort_by(|a, b| a.0.cmp(&b.0));

        for (enum_name, members) in &enums {
            // Generate the enum definition
            self.enum_defs.push_str(&format!("#[derive(Debug, Clone)]\n"));
            self.enum_defs.push_str(&format!("enum {} {{\n", enum_name));
            for member in members {
                let variant = member.union_variant_name();
                let rust_ty = member.rust_type();
                self.enum_defs.push_str(&format!("    {}({}),\n", variant, rust_ty));
            }
            self.enum_defs.push_str("}\n\n");

            // Generate Display impl so println!("{}", x) works
            self.enum_defs.push_str(&format!("impl std::fmt::Display for {} {{\n", enum_name));
            self.enum_defs.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n");
            self.enum_defs.push_str("        match self {\n");
            for member in members {
                let variant = member.union_variant_name();
                self.enum_defs.push_str(&format!(
                    "            {}::{}(v) => write!(f, \"{{}}\", v),\n",
                    enum_name, variant
                ));
            }
            self.enum_defs.push_str("        }\n");
            self.enum_defs.push_str("    }\n");
            self.enum_defs.push_str("}\n\n");
        }
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn writeln(&mut self, s: &str) {
        self.write_indent();
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }

    fn emit_module(&mut self, module: &HirModule) {
        for (i, func) in module.functions.iter().enumerate() {
            if i > 0 {
                self.output.push('\n');
            }
            self.emit_function(func);
        }
    }

    fn emit_function(&mut self, func: &HirFunction) {
        // Track the current function's return type for Option wrapping
        self.current_return_type = Some(func.return_type.clone());

        // Function signature
        self.write_indent();
        self.write("fn ");
        self.write(&func.name);
        self.write("(");

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.write(&param.name);
            self.write(": ");
            self.write(&param.ty.rust_type());
        }

        self.write(")");

        // Return type (omit for main and for None return)
        if func.return_type != Type::None || func.name != "main" {
            if func.return_type != Type::None {
                self.write(" -> ");
                self.write(&func.return_type.rust_type());
            }
        }

        self.write(" {\n");
        self.indent += 1;

        // Body
        for stmt in &func.body {
            self.emit_stmt(stmt);
        }

        self.indent -= 1;
        self.writeln("}");

        self.current_return_type = None;
    }

    fn emit_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Let { name, ty, value, is_mutable } => {
                self.write_indent();
                if *is_mutable {
                    self.write("let mut ");
                } else {
                    self.write("let ");
                }
                self.write(name);
                self.write(": ");
                self.write(&ty.rust_type());
                self.write(" = ");
                if is_option_type(ty) && matches!(value, HirExpr::NoneLiteral) {
                    // `x: str | None = None` -> `let x: Option<String> = None`
                    self.write("None");
                } else if is_option_type(ty) && !is_option_type(value.ty()) && !matches!(value.ty(), Type::None) {
                    // RHS is a plain value (not already Option) -> wrap in Some()
                    // But if RHS is a function call returning Option, don't double-wrap
                    self.write("Some(");
                    self.emit_expr(value);
                    self.write(")");
                } else {
                    // RHS already returns the right type (e.g., function returning Option<T>)
                    self.emit_expr(value);
                }
                self.write(";\n");
            }
            HirStmt::Assign { name, value } => {
                self.write_indent();
                self.write(name);
                self.write(" = ");
                self.emit_expr(value);
                self.write(";\n");
            }
            HirStmt::Return { value } => {
                let ret_is_option = self.current_return_type.as_ref().map_or(false, |t| is_option_type(t));
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
                if let Some((var_name, variant_name, enum_name, other_variants)) = detect_isinstance_union(condition) {
                    self.write_indent();
                    self.write(&format!("match {} {{\n", var_name));
                    self.indent += 1;

                    // Then branch: the matched variant
                    self.write_indent();
                    self.write(&format!("{}::{}({}) => {{\n", enum_name, variant_name, var_name));
                    self.indent += 1;
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                    self.writeln("}");

                    // Else branch: the other variant(s)
                    if let Some(else_stmts) = else_body {
                        if other_variants.len() == 1 {
                            let (other_variant, _) = &other_variants[0];
                            self.write_indent();
                            self.write(&format!("{}::{}({}) => {{\n", enum_name, other_variant, var_name));
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
                    }

                    self.indent -= 1;
                    self.writeln("}");
                }
                // Detect truthiness on Option: `if x:` where x is Option -> `if let Some(x) = x {`
                else if let Some(var_name) = detect_option_truthiness(condition) {
                    self.write_indent();
                    self.write(&format!("if let Some({}) = {} {{\n", var_name, var_name));
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
                // Detect Option narrowing: `if x is not None:` -> `if let Some(x) = x {`
                else if let Some(var_name) = detect_is_not_none_var(condition) {
                    self.write_indent();
                    // Use `if let Some(var) = var` to unwrap and shadow the variable
                    self.write(&format!("if let Some({}) = {} {{\n", var_name, var_name));
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
                } else if let Some(var_name) = detect_is_none_var(condition) {
                    self.write_indent();
                    self.write(&format!("if {}.is_none() {{\n", var_name));
                    self.indent += 1;
                    for s in then_body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;

                    if let Some(else_stmts) = else_body {
                        // In the else branch of `if x is None`, x is not None
                        self.write_indent();
                        self.write(&format!("}} else if let Some({}) = {} {{\n", var_name, var_name));
                        self.indent += 1;
                        self.option_unwrapped_vars.insert(var_name.clone());
                        for s in else_stmts {
                            self.emit_stmt(s);
                        }
                        self.option_unwrapped_vars.remove(&var_name);
                        self.indent -= 1;
                    }
                    self.writeln("}");
                } else {
                    // Normal if/elif/else
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
            HirStmt::While { condition, body } => {
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
            }
            HirStmt::For { target, iter, body, .. } => {
                self.write_indent();
                self.write("for ");
                self.write(target);
                self.write(" in ");
                // For lists, iterate with .iter() to borrow and clone elements
                let is_list = matches!(iter.ty(), Type::List(_));
                self.emit_expr(iter);
                if is_list {
                    self.write(".iter().cloned()");
                }
                self.write(" {\n");
                self.indent += 1;
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.writeln("}");
            }
            HirStmt::Break => {
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
        }
    }

    fn emit_method_call(&mut self, object: &HirExpr, method: &str, args: &[HirExpr]) {
        let obj_ty = object.ty();
        match (obj_ty, method) {
            // String methods
            (Type::Str, "upper") => {
                self.emit_expr(object);
                self.write(".to_uppercase()");
            }
            (Type::Str, "lower") => {
                self.emit_expr(object);
                self.write(".to_lowercase()");
            }
            (Type::Str, "strip") => {
                self.emit_expr(object);
                self.write(".trim().to_string()");
            }
            (Type::Str, "lstrip") => {
                self.emit_expr(object);
                self.write(".trim_start().to_string()");
            }
            (Type::Str, "rstrip") => {
                self.emit_expr(object);
                self.write(".trim_end().to_string()");
            }
            (Type::Str, "startswith") => {
                self.emit_expr(object);
                self.write(".starts_with(");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                    self.write(".as_str()");
                }
                self.write(")");
            }
            (Type::Str, "endswith") => {
                self.emit_expr(object);
                self.write(".ends_with(");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                    self.write(".as_str()");
                }
                self.write(")");
            }
            (Type::Str, "split") => {
                self.emit_expr(object);
                if args.is_empty() {
                    self.write(".split_whitespace().map(|s| s.to_string()).collect::<Vec<String>>()");
                } else {
                    self.write(".split(");
                    self.emit_expr(&args[0]);
                    self.write(".as_str()).map(|s| s.to_string()).collect::<Vec<String>>()");
                }
            }
            (Type::Str, "replace") => {
                self.emit_expr(object);
                self.write(".replace(");
                if args.len() >= 2 {
                    self.emit_expr(&args[0]);
                    self.write(".as_str(), ");
                    self.emit_expr(&args[1]);
                    self.write(".as_str()");
                }
                self.write(")");
            }
            (Type::Str, "find") => {
                self.emit_expr(object);
                self.write(".find(");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                    self.write(".as_str()");
                }
                self.write(").map_or(-1_i64, |i| i as i64)");
            }
            // List methods
            (Type::List(_), "append") => {
                self.emit_expr(object);
                self.write(".push(");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(")");
            }
            (Type::List(_), "pop") => {
                self.emit_expr(object);
                self.write(".pop().unwrap()");
            }
            // Dict methods
            (Type::Dict(_, _), "keys") => {
                self.emit_expr(object);
                self.write(".keys().cloned().collect::<Vec<_>>()");
            }
            (Type::Dict(_, _), "values") => {
                self.emit_expr(object);
                self.write(".values().cloned().collect::<Vec<_>>()");
            }
            (Type::Dict(_, _), "get") => {
                self.emit_expr(object);
                self.write(".get(&");
                if !args.is_empty() {
                    self.emit_expr(&args[0]);
                }
                self.write(").cloned().unwrap()");
            }
            // Tuple len() - compile-time constant
            (Type::Tuple(elems), "len") => {
                self.write(&format!("{}_i64", elems.len()));
            }
            // Generic len() for all types
            (_, "len") => {
                self.emit_expr(object);
                self.write(".len() as i64");
            }
            _ => {
                // Fallback: emit as-is
                self.emit_expr(object);
                self.write(&format!(".{}(", method));
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(arg);
                }
                self.write(")");
            }
        }
    }

    fn emit_expr(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::IntLiteral(val) => {
                self.write(&val.to_string());
                self.write("_i64");
            }
            HirExpr::FloatLiteral(val) => {
                let s = val.to_string();
                self.write(&s);
                if !s.contains('.') {
                    self.write(".0");
                }
                self.write("_f64");
            }
            HirExpr::StringLiteral(val) => {
                self.write(&format!("{:?}.to_string()", val));
            }
            HirExpr::BoolLiteral(val) => {
                self.write(if *val { "true" } else { "false" });
            }
            HirExpr::NoneLiteral => {
                // None in sifr maps to Rust's None (for Option contexts)
                // The parent (Let/Return) handles the wrapping context
                self.write("None");
            }
            HirExpr::Name { name, .. } => {
                self.write(name);
            }
            HirExpr::BinOp { left, op, right, ty } => {
                // Special handling for string concatenation
                if op == "+" && *ty == Type::Str {
                    self.write("format!(\"{}{}\", ");
                    self.emit_expr(left);
                    self.write(", ");
                    self.emit_expr(right);
                    self.write(")");
                } else if op == "//" {
                    // Floor division
                    self.emit_expr(left);
                    self.write(" / ");
                    self.emit_expr(right);
                } else if op == "**" {
                    // Power
                    self.write("(");
                    self.emit_expr(left);
                    self.write(" as f64).powf(");
                    self.emit_expr(right);
                    self.write(" as f64)");
                } else {
                    self.emit_expr(left);
                    self.write(&format!(" {} ", op));
                    self.emit_expr(right);
                }
            }
            HirExpr::UnaryOp { op, operand, .. } => {
                if op == "not" {
                    self.write("!");
                } else {
                    self.write(op);
                }
                self.emit_expr(operand);
            }
            HirExpr::Compare { left, ops, comparators, .. } => {
                // For single comparison
                if ops.len() == 1 {
                    let op = &ops[0];
                    // Handle `is None` / `is not None` for Option types
                    if (op == "is" || op == "is not") && matches!(comparators[0], HirExpr::NoneLiteral) {
                        self.emit_expr(left);
                        if op == "is" {
                            self.write(".is_none()");
                        } else {
                            self.write(".is_some()");
                        }
                    } else if op == "is" {
                        self.emit_expr(left);
                        self.write(" == ");
                        self.emit_expr(&comparators[0]);
                    } else if op == "is not" {
                        self.emit_expr(left);
                        self.write(" != ");
                        self.emit_expr(&comparators[0]);
                    } else {
                        self.emit_expr(left);
                        self.write(&format!(" {} ", op));
                        self.emit_expr(&comparators[0]);
                    }
                } else {
                    // Chained comparisons: a < b < c -> a < b && b < c
                    self.write("(");
                    self.emit_expr(left);
                    self.write(&format!(" {} ", ops[0]));
                    self.emit_expr(&comparators[0]);
                    for i in 1..ops.len() {
                        self.write(" && ");
                        self.emit_expr(&comparators[i - 1]);
                        self.write(&format!(" {} ", ops[i]));
                        self.emit_expr(&comparators[i]);
                    }
                    self.write(")");
                }
            }
            HirExpr::BoolOp { op, values, .. } => {
                let rust_op = if op == "and" { "&&" } else { "||" };
                for (i, val) in values.iter().enumerate() {
                    if i > 0 {
                        self.write(&format!(" {} ", rust_op));
                    }
                    self.emit_expr(val);
                }
            }
            HirExpr::Call { func, args, .. } => {
                if func == "print" {
                    // Map print() to println!
                    self.write("println!(\"{}\", ");
                    if args.is_empty() {
                        self.write("\"\"");
                    } else {
                        self.emit_expr(&args[0]);
                    }
                    self.write(")");
                } else if func == "isinstance" {
                    // isinstance() is handled by narrowing at the HIR level.
                    // At codegen time, we emit `true` since the narrowing has
                    // already validated the types. In practice, isinstance checks
                    // appear in if-conditions and the narrowing determines which
                    // branch to take.
                    self.write("true");
                } else if func == "str" {
                    // str() conversion -> format!("{}", arg)
                    if !args.is_empty() {
                        self.write("format!(\"{}\", ");
                        self.emit_expr(&args[0]);
                        self.write(")");
                    } else {
                        self.write("String::new()");
                    }
                } else {
                    self.write(func);
                    self.write("(");
                    // Look up param types to wrap union enum arguments
                    let param_types: Option<Vec<Type>> = self.func_signatures.get(func).map(|(pts, _)| pts.clone());
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        // Wrap arguments to match parameter types
                        if let Some(ref pts) = param_types {
                            if i < pts.len() {
                                // Option param with non-Option arg -> wrap in Some()
                                if is_option_type(&pts[i]) && !is_option_type(arg.ty()) && !matches!(arg, HirExpr::NoneLiteral) {
                                    self.write("Some(");
                                    self.emit_expr(arg);
                                    self.write(")");
                                    continue;
                                }
                                // Non-Option union param -> wrap in enum variant
                                if let Type::Union(members) = &pts[i] {
                                    if !is_option_type(&pts[i]) {
                                        let arg_ty = arg.ty();
                                        if let Some(variant) = find_union_variant(members, arg_ty) {
                                            let enum_name = pts[i].union_enum_name();
                                            self.write(&format!("{}::{}(", enum_name, variant));
                                            self.emit_expr(arg);
                                            self.write(")");
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                        self.emit_expr(arg);
                    }
                    self.write(")");
                }
            }
            HirExpr::IfExpr {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                self.write("if ");
                self.emit_expr(condition);
                self.write(" { ");
                self.emit_expr(then_expr);
                self.write(" } else { ");
                self.emit_expr(else_expr);
                self.write(" }");
            }
            HirExpr::RangeLiteral { start, end, .. } => {
                self.emit_expr(start);
                self.write("..");
                self.emit_expr(end);
            }
            HirExpr::ListLiteral { elements, .. } => {
                self.write("vec![");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(elem);
                }
                self.write("]");
            }
            HirExpr::DictLiteral { keys, values, .. } => {
                self.needs_hashmap = true;
                self.write("std::collections::HashMap::from([");
                for (i, (key, val)) in keys.iter().zip(values.iter()).enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write("(");
                    self.emit_expr(key);
                    self.write(", ");
                    self.emit_expr(val);
                    self.write(")");
                }
                self.write("])");
            }
            HirExpr::TupleLiteral { elements, .. } => {
                self.write("(");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(elem);
                }
                if elements.len() == 1 {
                    self.write(","); // Single-element tuple needs trailing comma in Rust
                }
                self.write(")");
            }
            HirExpr::Index { object, index, .. } => {
                let obj_ty = object.ty();
                match obj_ty {
                    Type::Dict(_, _) => {
                        // Dict indexing: d[key] -> d[&key] with clone
                        self.emit_expr(object);
                        self.write("[&");
                        self.emit_expr(index);
                        self.write("]");
                    }
                    Type::Tuple(_) => {
                        // Tuple indexing: t.0, t.1, etc.
                        self.emit_expr(object);
                        self.write(".");
                        self.emit_expr(index);
                    }
                    _ => {
                        // List/string indexing: x[i] -> x[i as usize]
                        self.emit_expr(object);
                        self.write("[");
                        self.emit_expr(index);
                        self.write(" as usize]");
                    }
                }
            }
            HirExpr::MethodCall { object, method, args, .. } => {
                self.emit_method_call(object, method, args);
            }
            HirExpr::ContainsOp { element, collection, .. } => {
                let coll_ty = collection.ty();
                match coll_ty {
                    Type::Dict(_, _) => {
                        self.emit_expr(collection);
                        self.write(".contains_key(&");
                        self.emit_expr(element);
                        self.write(")");
                    }
                    Type::Str => {
                        self.emit_expr(collection);
                        self.write(".contains(&");
                        self.emit_expr(element);
                        self.write(" as &str)");
                    }
                    _ => {
                        // List: collection.contains(&element)
                        self.emit_expr(collection);
                        self.write(".contains(&");
                        self.emit_expr(element);
                        self.write(")");
                    }
                }
            }
            HirExpr::FString { parts, .. } => {
                // Build the format string and collect expressions
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
                self.write("format!(\"");
                self.write(&format_str);
                self.write("\"");
                for expr in &exprs {
                    self.write(", ");
                    self.emit_expr(expr);
                }
                self.write(")");
            }
        }
    }
}

/// Check if a type is an Option type (T | None with exactly 2 members).
fn is_option_type(ty: &Type) -> bool {
    if let Type::Union(members) = ty {
        let non_none: Vec<&Type> = members.iter().filter(|m| !matches!(m, Type::None)).collect();
        let has_none = members.iter().any(|m| matches!(m, Type::None));
        has_none && non_none.len() == 1
    } else {
        false
    }
}

/// Detect truthiness check on an Option variable: `if x:` where x has type T | None.
fn detect_option_truthiness(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Name { name, ty } = expr {
        if is_option_type(ty) {
            return Some(name.clone());
        }
    }
    None
}

/// Detect `x is not None` pattern in a Compare expression. Returns the variable name.
fn detect_is_not_none_var(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Compare { left, ops, comparators, .. } = expr {
        if ops.len() == 1 && ops[0] == "is not" && matches!(comparators[0], HirExpr::NoneLiteral) {
            if let HirExpr::Name { name, .. } = left.as_ref() {
                return Some(name.clone());
            }
        }
    }
    None
}

/// Detect `isinstance(x, type)` where x is a non-Option union type.
/// Returns (var_name, variant_name, enum_name, other_variants: Vec<(variant_name, type)>).
fn detect_isinstance_union(expr: &HirExpr) -> Option<(String, String, String, Vec<(String, Type)>)> {
    if let HirExpr::Call { func, args, .. } = expr {
        if func == "isinstance" && args.len() == 2 {
            if let HirExpr::Name { name, ty } = &args[0] {
                if let Type::Union(members) = ty {
                    if !is_option_type(ty) {
                        // The second arg is a StringLiteral with the type name
                        if let HirExpr::StringLiteral(type_name) = &args[1] {
                            let target_ty = match type_name.as_str() {
                                "int" => Type::Int,
                                "str" => Type::Str,
                                "float" => Type::Float,
                                "bool" => Type::Bool,
                                _ => return None,
                            };
                            // Check that this type is a member of the union
                            if members.contains(&target_ty) {
                                let variant = target_ty.union_variant_name();
                                let enum_name = ty.union_enum_name();
                                // Collect other variants for else branch destructuring
                                let other_variants: Vec<(String, Type)> = members.iter()
                                    .filter(|m| *m != &target_ty)
                                    .map(|m| (m.union_variant_name(), m.clone()))
                                    .collect();
                                return Some((name.clone(), variant, enum_name, other_variants));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Find the matching union variant name for an argument type.
fn find_union_variant(members: &[Type], arg_ty: &Type) -> Option<String> {
    for member in members {
        if arg_ty.is_assignable_to(member) {
            return Some(member.union_variant_name());
        }
    }
    None
}

/// Detect `x is None` pattern in a Compare expression. Returns the variable name.
fn detect_is_none_var(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Compare { left, ops, comparators, .. } = expr {
        if ops.len() == 1 && ops[0] == "is" && matches!(comparators[0], HirExpr::NoneLiteral) {
            if let HirExpr::Name { name, .. } = left.as_ref() {
                return Some(name.clone());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_hir::*;
    use sifr_type_system::Type;

    #[test]
    fn test_simple_function_codegen() {
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::StringLiteral("Hello, World!".to_string())],
                        ty: Type::None,
                    },
                }],
            }],
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("fn main()"));
        assert!(rust_code.contains("println!"));
        assert!(rust_code.contains("Hello, World!"));
    }

    #[test]
    fn test_arithmetic_codegen() {
        let module = HirModule {
            functions: vec![HirFunction {
                name: "add".to_string(),
                params: vec![
                    HirParam { name: "a".to_string(), ty: Type::Int },
                    HirParam { name: "b".to_string(), ty: Type::Int },
                ],
                return_type: Type::Int,
                body: vec![HirStmt::Return {
                    value: Some(HirExpr::BinOp {
                        left: Box::new(HirExpr::Name { name: "a".to_string(), ty: Type::Int }),
                        op: "+".to_string(),
                        right: Box::new(HirExpr::Name { name: "b".to_string(), ty: Type::Int }),
                        ty: Type::Int,
                    }),
                }],
            }],
        };

        let rust_code = generate_rust(&module);
        assert!(rust_code.contains("fn add(a: i64, b: i64) -> i64"));
        assert!(rust_code.contains("return a + b;"));
    }
}
