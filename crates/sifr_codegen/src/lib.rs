//! Sifr Code Generation
//!
//! Translates the typed HIR into Rust source code.

use sifr_hir::*;
use sifr_type_system::Type;

/// Generate Rust source code from a HIR module.
pub fn generate_rust(module: &HirModule) -> String {
    let mut emitter = RustEmitter::new();
    emitter.emit_module(module);
    emitter.output
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
}

impl RustEmitter {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
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
                self.emit_expr(value);
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
                self.write_indent();
                if let Some(val) = value {
                    self.write("return ");
                    self.emit_expr(val);
                    self.write(";\n");
                } else {
                    self.write("return;\n");
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
                self.emit_expr(iter);
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
                self.write("()");
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
                    self.emit_expr(left);
                    self.write(&format!(" {} ", ops[0]));
                    self.emit_expr(&comparators[0]);
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
                } else {
                    self.write(func);
                    self.write("(");
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
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
        }
    }
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
