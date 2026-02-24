use crate::helpers::{
    collect_string_concat_parts, find_union_variant, is_option_type, needs_clone_for_type,
};
use crate::RustEmitter;
use sifr_hir::HirExpr;
use sifr_type_system::{ParamConvention, Type};

impl RustEmitter {
    pub(super) fn emit_expr_fallback(&mut self, expr: &HirExpr) {
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
                self.write(&format!("{val:?}.to_string()"));
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
                // Check for stdlib constants
                if self.intrinsic_functions.contains(name.as_str()) || self.is_stdlib_constant(name)
                {
                    self.emit_stdlib_constant(name);
                } else if let Some((_ty, rust_name)) = self.module_constants.get(name).cloned() {
                    // Module-level constant
                    self.write(&rust_name);
                } else {
                    self.write(name);
                }
            }
            HirExpr::BinOp {
                left,
                op,
                right,
                ty,
            } => {
                // BigInt arithmetic: always clone operands to avoid move issues
                if left.ty() == &Type::BigInt && right.ty() == &Type::BigInt && op != "**" {
                    if op == "//" {
                        // BigInt floor division uses /
                        self.emit_expr_with_bigint_clone(left);
                        self.write(" / ");
                        self.emit_expr_with_bigint_clone(right);
                    } else {
                        self.emit_expr_with_bigint_clone(left);
                        self.write(&format!(" {op} "));
                        self.emit_expr_with_bigint_clone(right);
                    }
                    return;
                }
                // Special handling for string concatenation
                if op == "+" && *ty == Type::Str {
                    // Flatten chained string concatenation into a single format! call
                    // Fold string literals directly into the format string
                    let mut parts: Vec<&HirExpr> = Vec::new();
                    collect_string_concat_parts(left, &mut parts);
                    collect_string_concat_parts(right, &mut parts);
                    let mut format_str = String::new();
                    let mut format_args: Vec<&HirExpr> = Vec::new();
                    for part in &parts {
                        if let HirExpr::StringLiteral(val) = part {
                            // Fold literal directly into format string
                            format_str.push_str(val);
                        } else {
                            format_str.push_str("{}");
                            format_args.push(part);
                        }
                    }
                    if format_args.is_empty() {
                        // All parts are literals, just emit a string literal
                        self.write(&format!("\"{format_str}\".to_string()"));
                    } else {
                        self.write(&format!("format!(\"{format_str}\""));
                        for arg in &format_args {
                            self.write(", ");
                            self.emit_expr(arg);
                        }
                        self.write(")");
                    }
                } else if op == "+" && matches!(ty, Type::List(_)) {
                    // List concatenation: a + b -> { let mut tmp = a.clone(); tmp.extend(b.iter().cloned()); tmp }
                    self.write("{ let mut __tmp = ");
                    self.emit_expr(left);
                    self.write(".clone(); __tmp.extend(");
                    self.emit_expr(right);
                    self.write(".iter().cloned()); __tmp }");
                } else if op == "//" {
                    // Floor division (int // int -> int division in Rust)
                    // Wrap sub-expressions in parens if they are BinOps to preserve precedence
                    if matches!(left.as_ref(), HirExpr::BinOp { .. }) {
                        self.write("(");
                    }
                    self.emit_expr(left);
                    if matches!(left.as_ref(), HirExpr::BinOp { .. }) {
                        self.write(")");
                    }
                    self.write(" / ");
                    if matches!(right.as_ref(), HirExpr::BinOp { .. }) {
                        self.write("(");
                    }
                    self.emit_expr(right);
                    if matches!(right.as_ref(), HirExpr::BinOp { .. }) {
                        self.write(")");
                    }
                } else if op == "**" {
                    // Power: int ** int -> i64::pow, otherwise float
                    if left.ty() == &Type::BigInt {
                        // bigint ** bigint or bigint ** int -> num_bigint pow
                        self.write("(");
                        self.emit_expr(left);
                        self.write(")");
                        self.write(".pow(u32::try_from(");
                        self.emit_expr(right);
                        self.write(").unwrap_or(0))");
                    } else if left.ty() == &Type::Int && right.ty() == &Type::Int {
                        self.write("(");
                        self.emit_expr(left);
                        self.write(")");
                        self.write(".pow(");
                        self.emit_expr(right);
                        self.write(" as u32)");
                    } else if left.ty() == &Type::Float && right.ty() == &Type::Int {
                        self.write("(");
                        self.emit_expr(left);
                        self.write(")");
                        self.write(".powi(");
                        self.emit_expr(right);
                        self.write(" as i32)");
                    } else {
                        self.write("(");
                        self.emit_expr(left);
                        self.write(" as f64).powf(");
                        self.emit_expr(right);
                        self.write(" as f64)");
                    }
                } else if op == "*" && left.ty() == &Type::Str && right.ty() == &Type::Int {
                    // String multiplication: "abc" * 3 -> "abc".repeat(3)
                    self.emit_expr(left);
                    self.write(".repeat(");
                    self.emit_expr(right);
                    self.write(" as usize)");
                } else if op == "*" && left.ty() == &Type::Int && right.ty() == &Type::Str {
                    // Reverse string multiplication: 3 * "abc"
                    self.emit_expr(right);
                    self.write(".repeat(");
                    self.emit_expr(left);
                    self.write(" as usize)");
                } else if op == "/" && left.ty() == &Type::Int && right.ty() == &Type::Int {
                    // Python: int / int -> float (true division)
                    // Rust: i64 / i64 -> i64 (integer division)
                    // Fix: cast both to f64 for true division
                    self.write("(");
                    self.emit_expr(left);
                    self.write(" as f64) / (");
                    self.emit_expr(right);
                    self.write(" as f64)");
                } else if matches!(left.ty(), Type::Class { .. }) {
                    // Class type with operator overloading: use reference-based ops
                    self.write("&");
                    self.emit_expr(left);
                    self.write(&format!(" {op} "));
                    self.write("&");
                    self.emit_expr(right);
                } else if is_option_type(left.ty()) || is_option_type(right.ty()) {
                    // Union/optional arithmetic: unwrap Option with .unwrap()
                    if is_option_type(left.ty()) {
                        self.emit_expr(left);
                        self.write(".unwrap()");
                    } else {
                        self.emit_expr(left);
                    }
                    self.write(&format!(" {op} "));
                    if is_option_type(right.ty()) {
                        self.emit_expr(right);
                        self.write(".unwrap()");
                    } else {
                        self.emit_expr(right);
                    }
                } else {
                    // Handle mixed int/float arithmetic: cast int side to f64
                    let left_is_int = left.ty() == &Type::Int;
                    let right_is_int = right.ty() == &Type::Int;
                    let left_is_float = left.ty() == &Type::Float;
                    let right_is_float = right.ty() == &Type::Float;
                    let needs_left_cast = left_is_int && right_is_float;
                    let needs_right_cast = right_is_int && left_is_float;

                    // Wrap sub-expressions in parens if they are BinOps to preserve precedence
                    // Also wrap if the expression is an IntLiteral (might be used with operators that need parens)
                    let needs_left_parens = matches!(left.as_ref(), HirExpr::BinOp { .. })
                        || matches!(left.as_ref(), HirExpr::IntLiteral { .. });
                    let needs_right_parens = matches!(right.as_ref(), HirExpr::BinOp { .. })
                        || matches!(right.as_ref(), HirExpr::IntLiteral { .. });
                    if needs_left_parens || needs_left_cast {
                        self.write("(");
                    }
                    self.emit_expr(left);
                    if needs_left_parens || needs_left_cast {
                        self.write(")");
                    }
                    if needs_left_cast {
                        self.write(" as f64");
                    }
                    self.write(&format!(" {op} "));
                    if needs_right_parens || needs_right_cast {
                        self.write("(");
                    }
                    self.emit_expr(right);
                    if needs_right_parens || needs_right_cast {
                        self.write(")");
                    }
                    if needs_right_cast {
                        self.write(" as f64");
                    }
                }
            }
            HirExpr::UnaryOp { op, operand, .. } => {
                if op == "not" {
                    // Collection truthiness: `not list_var` -> `list_var.is_empty()`
                    let is_collection = matches!(
                        operand.ty(),
                        Type::List(_)
                            | Type::Dict(_, _)
                            | Type::Set(_)
                            | Type::Tuple(_)
                            | Type::Str
                    );
                    if is_collection {
                        self.emit_expr(operand);
                        self.write(".is_empty()");
                    } else if matches!(operand.ty(), Type::Union(_)) {
                        // Optional truthiness: `not x` where x is T|None -> `x.is_none()`
                        self.emit_expr(operand);
                        self.write(".is_none()");
                    } else {
                        self.write("!");
                        self.emit_expr(operand);
                    }
                } else if op == "~" {
                    // Bitwise invert maps to `!` in Rust
                    self.write("!");
                    self.emit_expr(operand);
                } else if op == "+" {
                    // Unary + is a no-op in Python/Rust, just emit the operand
                    self.emit_expr(operand);
                } else {
                    self.write(op);
                    self.emit_expr(operand);
                }
            }
            HirExpr::Compare {
                left,
                ops,
                comparators,
                ..
            } => {
                // For single comparison
                if ops.len() == 1 {
                    let op = &ops[0];
                    // Handle `is None` / `is not None` for Option types
                    if (op == "is" || op == "is not")
                        && matches!(comparators[0], HirExpr::NoneLiteral)
                    {
                        // If left is already Type::None (not T|None), it's always None
                        if matches!(left.ty(), Type::None) {
                            if op == "is" {
                                self.write("true");
                            } else {
                                self.write("false");
                            }
                        } else {
                            self.emit_expr(left);
                            if op == "is" {
                                self.write(".is_none()");
                            } else {
                                self.write(".is_some()");
                            }
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
                        // Handle Option<T> vs T comparisons: wrap T in Some()
                        let left_is_option = is_option_type(left.ty());
                        let right_is_option = is_option_type(comparators[0].ty());
                        if left_is_option
                            && !right_is_option
                            && !matches!(comparators[0], HirExpr::NoneLiteral)
                        {
                            self.emit_expr(left);
                            self.write(&format!(" {op} Some("));
                            self.emit_expr(&comparators[0]);
                            self.write(")");
                        } else if !left_is_option
                            && right_is_option
                            && !matches!(left.as_ref(), HirExpr::NoneLiteral)
                        {
                            self.write("Some(");
                            self.emit_expr(left);
                            self.write(")");
                            self.write(&format!(" {op} "));
                            self.emit_expr(&comparators[0]);
                        } else {
                            // Dereference borrowed params in comparisons to avoid &String == String
                            self.write("(");
                            self.emit_expr_for_compare(left);
                            self.write(")");
                            self.write(&format!(" {op} "));
                            self.write("(");
                            self.emit_expr_for_compare(&comparators[0]);
                            self.write(")");
                        }
                    }
                } else {
                    // Chained comparisons: a < b < c -> a < b && b < c
                    self.write("(");
                    self.write("(");
                    self.emit_expr_for_compare(left);
                    self.write(")");
                    self.write(&format!(" {} ", ops[0]));
                    self.write("(");
                    self.emit_expr_for_compare(&comparators[0]);
                    self.write(")");
                    for i in 1..ops.len() {
                        self.write(" && ");
                        self.write("(");
                        self.emit_expr_for_compare(&comparators[i - 1]);
                        self.write(")");
                        self.write(&format!(" {} ", ops[i]));
                        self.write("(");
                        self.emit_expr_for_compare(&comparators[i]);
                        self.write(")");
                    }
                    self.write(")");
                }
            }
            HirExpr::BoolOp { op, values, .. } => {
                let rust_op = if op == "and" { "&&" } else { "||" };
                for (i, val) in values.iter().enumerate() {
                    if i > 0 {
                        self.write(&format!(" {rust_op} "));
                    }
                    self.emit_expr(val);
                }
            }
            HirExpr::Call { func, args, .. } => {
                if func == "print" {
                    // Map print() to println!
                    if args.is_empty() {
                        self.write("println!()");
                    } else if matches!(args[0], HirExpr::NoneLiteral)
                        || matches!(args[0].ty(), Type::None)
                    {
                        // print(None) -> println!("None")
                        self.write("println!(\"None\")");
                    } else if let HirExpr::StringLiteral(val) = &args[0] {
                        // Inline string literal directly: println!("hello") instead of println!("{}", "hello")
                        // Escape backslashes and double quotes for valid Rust string
                        let escaped = val
                            .replace('\\', "\\\\")
                            .replace('"', "\\\"")
                            .replace('{', "{{")
                            .replace('}', "}}");
                        self.write(&format!("println!(\"{escaped}\")"));
                    } else if let HirExpr::FString { parts, .. } = &args[0] {
                        // Inline f-string directly into println! to avoid double-format
                        self.emit_fstring_macro("println!", parts);
                    } else if matches!(args[0].ty(), Type::Class { .. } | Type::Newtype { .. }) {
                        // Check if class has Display impl
                        let class_name = match args[0].ty() {
                            Type::Class { name, .. } | Type::Newtype { name, .. } => name.clone(),
                            _ => String::new(),
                        };
                        if self.display_classes.contains(&class_name) {
                            self.write("println!(\"{}\", ");
                        } else {
                            self.write("println!(\"{:?}\", ");
                        }
                        self.emit_expr(&args[0]);
                        self.write(")");
                    } else if matches!(
                        args[0].ty(),
                        Type::List(_) | Type::Dict(_, _) | Type::Tuple(_) | Type::Set(_)
                    ) {
                        // Collections use Debug format
                        self.write("println!(\"{:?}\", ");
                        self.emit_expr(&args[0]);
                        self.write(")");
                    } else {
                        // Use emit_display_expr for all other cases:
                        // - Option<T> gets map_or wrapping
                        // - String literals omit .to_string()
                        // - Everything else emits normally
                        self.write("println!(\"{}\", ");
                        self.emit_display_expr(&args[0]);
                        self.write(")");
                    }
                } else if func == "isinstance" {
                    // isinstance() is handled by narrowing at the HIR level.
                    // At codegen time, we emit `true` since the narrowing has
                    // already validated the types. In practice, isinstance checks
                    // appear in if-conditions and the narrowing determines which
                    // branch to take.
                    self.write("true");
                } else if func == "str" {
                    // str() conversion -> format!("{}", arg) or format!("{:?}", arg) for lists
                    if args.is_empty() {
                        self.write("String::new()");
                    } else {
                        if matches!(args[0].ty(), Type::List(_)) {
                            self.write("format!(\"{:?}\", ");
                        } else {
                            self.write("format!(\"{}\", ");
                        }
                        self.emit_display_expr(&args[0]);
                        self.write(")");
                    }
                } else if func == "pow" {
                    // pow(base, exp)
                    if args.len() == 2 {
                        if args[0].ty() == &Type::Int && args[1].ty() == &Type::Int {
                            // Wrap base in parens to handle cases like "(2 as i64).pow(...)"
                            self.write("(");
                            self.emit_expr(&args[0]);
                            self.write(").pow(");
                            self.emit_expr(&args[1]);
                            self.write(" as u32)");
                        } else {
                            self.write("(");
                            self.emit_expr(&args[0]);
                            self.write(" as f64).powf(");
                            self.emit_expr(&args[1]);
                            self.write(" as f64)");
                        }
                    }
                } else if func == "abs" {
                    if !args.is_empty() {
                        self.write("(");
                        self.emit_expr(&args[0]);
                        self.write(").abs()");
                    }
                } else if func == "hash" {
                    // hash(x) -> { use std::hash::{Hash, Hasher}; let mut h = std::collections::hash_map::DefaultHasher::new(); x.hash(&mut h); h.finish() as i64 }
                    if !args.is_empty() {
                        self.write("{ use std::hash::{Hash, Hasher}; let mut _h = std::collections::hash_map::DefaultHasher::new(); ");
                        self.emit_expr(&args[0]);
                        self.write(".hash(&mut _h); _h.finish() as i64 }");
                    }
                } else if func == "round" {
                    if args.len() == 1 {
                        self.emit_expr(&args[0]);
                        self.write(".round() as i64");
                    } else if args.len() == 2 {
                        // round(x, n) -> (x * 10^n).round() / 10^n
                        self.write("((");
                        self.emit_expr(&args[0]);
                        self.write(" as f64 * 10.0_f64.powi(");
                        self.emit_expr(&args[1]);
                        self.write(" as i32)).round() / 10.0_f64.powi(");
                        self.emit_expr(&args[1]);
                        self.write(" as i32))");
                    }
                } else if func == "repr" {
                    if !args.is_empty() {
                        self.write("format!(\"{:?}\", ");
                        self.emit_expr(&args[0]);
                        self.write(")");
                    }
                } else if func == "int" {
                    if !args.is_empty() {
                        match args[0].ty() {
                            Type::Float => {
                                self.write("(");
                                self.emit_expr(&args[0]);
                                self.write(") as i64");
                            }
                            Type::Str => {
                                // int(str) -> Result<i64, ParseError>
                                self.emit_expr(&args[0]);
                                self.write(".parse::<i64>().map_err(|e| ParseError { message: e.to_string() })");
                            }
                            Type::Bool => {
                                self.write("if ");
                                self.emit_expr(&args[0]);
                                self.write(" { 1_i64 } else { 0_i64 }");
                            }
                            Type::BigInt => {
                                // int(bigint) -> Result<i64, OverflowError>
                                self.write("i64::try_from(&");
                                self.emit_expr(&args[0]);
                                self.write(").map_err(|_| OverflowError { message: \"bigint value out of range for int\".to_string() })");
                            }
                            _ => {
                                self.emit_expr(&args[0]);
                            }
                        }
                    }
                } else if func == "bigint" {
                    if !args.is_empty() {
                        // bigint(n) -> BigInt::from(n)
                        self.write("BigInt::from(");
                        self.emit_expr(&args[0]);
                        self.write(")");
                    }
                } else if func == "float" {
                    if !args.is_empty() {
                        match args[0].ty() {
                            Type::Int => {
                                self.write("(");
                                self.emit_expr(&args[0]);
                                self.write(") as f64");
                            }
                            Type::Str => {
                                // float(str) -> Result<f64, ParseError>
                                self.emit_expr(&args[0]);
                                self.write(".parse::<f64>().map_err(|e| ParseError { message: e.to_string() })");
                            }
                            _ => {
                                self.emit_expr(&args[0]);
                            }
                        }
                    }
                } else if func == "bool" {
                    if !args.is_empty() {
                        match args[0].ty() {
                            Type::Int => {
                                self.emit_expr(&args[0]);
                                self.write(" != 0");
                            }
                            Type::Float => {
                                self.emit_expr(&args[0]);
                                self.write(" != 0.0");
                            }
                            Type::Str | Type::List(_) | Type::Dict(_, _) => {
                                self.write("!");
                                self.emit_expr(&args[0]);
                                self.write(".is_empty()");
                            }
                            Type::Tuple(elems) => {
                                // Non-empty tuples are always truthy, empty tuples are falsy
                                if elems.is_empty() {
                                    self.write("false");
                                } else {
                                    self.write("true");
                                }
                            }
                            Type::Bool => {
                                self.emit_expr(&args[0]);
                            }
                            Type::None => {
                                self.write("false");
                            }
                            _ => {
                                self.emit_expr(&args[0]);
                            }
                        }
                    }
                } else if func == "min" {
                    if args.len() == 2 {
                        // min(a, b) -> std::cmp::min(a, b) or a.min(b) for floats
                        if matches!(args[0].ty(), Type::Float) {
                            self.emit_expr(&args[0]);
                            self.write(".min(");
                            self.emit_expr(&args[1]);
                            self.write(")");
                        } else {
                            self.write("std::cmp::min(");
                            self.emit_expr(&args[0]);
                            self.write(", ");
                            self.emit_expr(&args[1]);
                            self.write(")");
                        }
                    } else if matches!(args[0].ty(), Type::List(ref e) if matches!(e.as_ref(), Type::Float))
                    {
                        // min(list[float]) -> Option[float] (safe: None on empty)
                        self.emit_expr(&args[0]);
                        self.write(".iter().cloned().reduce(f64::min)");
                    } else {
                        // min(list[T]) -> Option[T] (safe: None on empty)
                        self.emit_expr(&args[0]);
                        self.write(".iter().min().cloned()");
                    }
                } else if func == "max" {
                    if args.len() == 2 {
                        // max(a, b) -> std::cmp::max(a, b) or a.max(b) for floats
                        if matches!(args[0].ty(), Type::Float) {
                            self.emit_expr(&args[0]);
                            self.write(".max(");
                            self.emit_expr(&args[1]);
                            self.write(")");
                        } else {
                            self.write("std::cmp::max(");
                            self.emit_expr(&args[0]);
                            self.write(", ");
                            self.emit_expr(&args[1]);
                            self.write(")");
                        }
                    } else if matches!(args[0].ty(), Type::List(ref e) if matches!(e.as_ref(), Type::Float))
                    {
                        // max(list[float]) -> Option[float] (safe: None on empty)
                        self.emit_expr(&args[0]);
                        self.write(".iter().cloned().reduce(f64::max)");
                    } else {
                        // max(list[T]) -> Option[T] (safe: None on empty)
                        self.emit_expr(&args[0]);
                        self.write(".iter().max().cloned()");
                    }
                } else if func == "sum" {
                    // sum(list) -> list.iter().sum()
                    self.emit_expr(&args[0]);
                    self.write(".iter().sum::<");
                    if let Type::List(ref elem) = args[0].ty() {
                        self.write(&elem.rust_type());
                    } else {
                        self.write("_");
                    }
                    self.write(">()");
                } else if func == "sorted" {
                    // sorted(list) -> { let mut v = list.clone(); v.sort(); v }
                    // For f64 lists, use sort_by since f64 doesn't implement Ord
                    let is_float_list =
                        matches!(args[0].ty(), Type::List(inner) if **inner == Type::Float);
                    self.write("{ let mut _sorted = ");
                    self.emit_expr(&args[0]);
                    if is_float_list {
                        self.write(".clone(); _sorted.sort_by(|a, b| a.total_cmp(b)); _sorted }");
                    } else {
                        self.write(".clone(); _sorted.sort(); _sorted }");
                    }
                } else if func == "reversed" {
                    // reversed(list) -> { let mut v = list.clone(); v.reverse(); v }
                    self.write("{ let mut _rev = ");
                    self.emit_expr(&args[0]);
                    self.write(".clone(); _rev.reverse(); _rev }");
                } else if func == "enumerate" {
                    // enumerate(list) -> list.iter().enumerate().map(|(i, v)| (i as i64, v.clone())).collect()
                    self.emit_expr(&args[0]);
                    self.write(".iter().enumerate().map(|(i, v)| (i as i64, v.clone())).collect::<Vec<_>>()");
                } else if func == "zip" {
                    // zip(a, b) -> a.iter().zip(b.iter()).map(|(a, b)| (a.clone(), b.clone())).collect()
                    self.emit_expr(&args[0]);
                    self.write(".iter().zip(");
                    self.emit_expr(&args[1]);
                    self.write(".iter()).map(|(a, b)| (a.clone(), b.clone())).collect::<Vec<_>>()");
                } else if func == "any" {
                    // any(list) -> list.iter().any(|x| *x)
                    self.emit_expr(&args[0]);
                    self.write(".iter().any(|x| *x)");
                } else if func == "all" {
                    // all(list) -> list.iter().all(|x| *x)
                    self.emit_expr(&args[0]);
                    self.write(".iter().all(|x| *x)");
                } else if func == "map" {
                    // map(func, list) -> list.clone().into_iter().map(func).collect()
                    self.emit_expr(&args[1]);
                    self.write(".clone().into_iter().map(");
                    self.emit_lambda_untyped(&args[0]);
                    self.write(").collect::<Vec<_>>()");
                } else if func == "filter" {
                    // filter(func, list) -> list.clone().into_iter().filter(|&x| body).collect()
                    // Inline the lambda body directly instead of closure-within-closure
                    self.emit_expr(&args[1]);
                    if let HirExpr::Lambda { params, body, .. } = &args[0] {
                        let param_name = if params.is_empty() {
                            "x"
                        } else {
                            &params[0].name
                        };
                        // Use .clone().into_iter() for owned values, then filter with |&var| destructuring
                        self.write(&format!(".clone().into_iter().filter(|&{param_name}| "));
                        self.emit_expr(body);
                        self.write(").collect::<Vec<_>>()");
                    } else {
                        self.write(".clone().into_iter().filter(|x| (");
                        self.emit_lambda_untyped(&args[0]);
                        self.write(")(x)).collect::<Vec<_>>()");
                    }
                } else if self.intrinsic_functions.contains(func.as_str()) || func == "builtin_open"
                {
                    // Intrinsic function call — emit the correct Rust code
                    self.emit_intrinsic_call(func, args);
                } else {
                    self.write(func);
                    self.write("(");
                    // Look up param types and conventions to wrap union enum arguments.
                    // First check func_signatures (regular functions), then callable_var_conventions
                    // (Callable-typed parameters/locals whose conventions are tracked per-function).
                    let param_info: Option<Vec<(Type, ParamConvention)>> = self
                        .func_signatures
                        .get(func)
                        .map(|(pts, _)| pts.clone())
                        .or_else(|| self.callable_var_conventions.get(func).cloned());
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        // Wrap arguments to match parameter types
                        if let Some(ref pts) = param_info {
                            if i < pts.len() {
                                let (ref param_ty, convention) = pts[i];
                                // Option param with non-Option arg -> wrap in Some()
                                if is_option_type(param_ty)
                                    && !is_option_type(arg.ty())
                                    && !matches!(arg, HirExpr::NoneLiteral)
                                {
                                    // Use param_ty for ownership check: the wrapped Some(...) is Option<T> (Move),
                                    // not the inner arg type which may be Copy
                                    self.emit_borrow_prefix(convention, param_ty, Some(param_ty));
                                    self.write("Some(");
                                    self.emit_expr(arg);
                                    self.write(")");
                                    continue;
                                }
                                // None literal passed to Option param -> emit &None for borrowed params
                                if is_option_type(param_ty) && matches!(arg, HirExpr::NoneLiteral) {
                                    self.emit_borrow_prefix(convention, param_ty, Some(param_ty));
                                    self.emit_expr(arg);
                                    continue;
                                }
                                // Result[T, Error] param with a concrete Result[T, SomeError] arg:
                                // convert the error branch so Rust types line up (Result invariance).
                                if convention == ParamConvention::Own {
                                    if let (Type::Result(_, param_err), Type::Result(_, arg_err)) =
                                        (param_ty, arg.ty())
                                    {
                                        if param_err.display_name() == "Error"
                                            && arg_err.display_name() != "Error"
                                        {
                                            self.write("(");
                                            self.emit_expr(arg);
                                            self.write(").map_err(|e| Error::new(e.to_string()))");
                                            continue;
                                        }
                                    }
                                }
                                // Non-Option union param -> wrap in enum variant
                                if let Type::Union(members) = param_ty {
                                    if !is_option_type(param_ty) {
                                        let arg_ty = arg.ty();
                                        if let Some(variant) = find_union_variant(members, arg_ty) {
                                            let enum_name = param_ty.union_enum_name();
                                            // Use param_ty for ownership check: the wrapped enum value is a Union (Move),
                                            // not the inner arg type which may be Copy (e.g., Int inside IntOrStr)
                                            self.emit_borrow_prefix(
                                                convention,
                                                param_ty,
                                                Some(param_ty),
                                            );
                                            self.write(&format!("{enum_name}::{variant}("));
                                            self.emit_expr(arg);
                                            self.write(")");
                                            continue;
                                        }
                                    }
                                }
                                // Protocol param with concrete class arg -> wrap in Box::new()
                                if matches!(param_ty, Type::Protocol { .. })
                                    && !matches!(arg.ty(), Type::Protocol { .. })
                                {
                                    self.write("Box::new(");
                                    self.emit_expr(arg);
                                    self.write(")");
                                    continue;
                                }
                                // Callable param with TypeVar params: wrap concrete function in
                                // adapter closure so Copy-type args get dereferenced to match the
                                // generic `impl Fn(&T) -> R` signature.
                                if let Type::Callable(
                                    callable_params,
                                    callable_convs,
                                    _callable_ret,
                                ) = param_ty
                                {
                                    let has_typevar_param = callable_params
                                        .iter()
                                        .any(|p| matches!(p, Type::TypeVar(_)));
                                    if has_typevar_param {
                                        if let HirExpr::Name {
                                            name: arg_func_name,
                                            ..
                                        } = arg
                                        {
                                            if let Some((concrete_params, _)) = self
                                                .func_signatures
                                                .get(arg_func_name.as_str())
                                                .cloned()
                                            {
                                                let needs_wrapper = callable_params.iter().zip(concrete_params.iter()).any(|(cp, (ct, _))| {
                                                    matches!(cp, Type::TypeVar(_)) && ct.ownership() == sifr_type_system::OwnershipKind::Copy
                                                });
                                                if needs_wrapper {
                                                    self.write("|");
                                                    for (pi, (cp, cc)) in callable_params
                                                        .iter()
                                                        .zip(callable_convs.iter())
                                                        .enumerate()
                                                    {
                                                        if pi > 0 {
                                                            self.write(", ");
                                                        }
                                                        let pname = format!("__a{pi}");
                                                        if matches!(cp, Type::TypeVar(_)) || (*cc == ParamConvention::Borrow && cp.ownership() == sifr_type_system::OwnershipKind::Move) {
                                                            self.write(&format!("{pname}: &_"));
                                                        } else {
                                                            self.write(&format!("{pname}: _"));
                                                        }
                                                    }
                                                    self.write("| ");
                                                    self.write(arg_func_name);
                                                    self.write("(");
                                                    for (pi, (cp, (ct, _))) in callable_params
                                                        .iter()
                                                        .zip(concrete_params.iter())
                                                        .enumerate()
                                                    {
                                                        if pi > 0 {
                                                            self.write(", ");
                                                        }
                                                        let pname = format!("__a{pi}");
                                                        if matches!(cp, Type::TypeVar(_)) && ct.ownership() == sifr_type_system::OwnershipKind::Copy {
                                                            self.write(&format!("*{pname}"));
                                                        } else {
                                                            self.write(&pname);
                                                        }
                                                    }
                                                    self.write(")");
                                                    continue;
                                                }
                                            }
                                        }
                                    }
                                }
                                // Convention-aware borrow prefix for regular arguments.
                                // Pass the arg name (if it's a Name expr) so we can detect
                                // already-borrowed parameters and avoid double-borrowing.
                                let arg_name_opt = if let HirExpr::Name { name, .. } = arg {
                                    Some(name.as_str())
                                } else {
                                    None
                                };
                                // For borrowed generic params (&T), wrapping expressions
                                // avoids Rust precedence pitfalls like `&(x) as i64`.
                                // This includes literals which otherwise produce invalid code like `&3_i64`.
                                if convention == ParamConvention::Borrow
                                    && matches!(param_ty, Type::TypeVar(_))
                                {
                                    self.write("&(");
                                    self.emit_expr(arg);
                                    self.write(")");
                                    continue;
                                }
                                self.emit_borrow_prefix_for_name(
                                    convention,
                                    arg.ty(),
                                    Some(param_ty),
                                    arg_name_opt,
                                );
                                self.emit_expr(arg);
                                continue;
                            }
                        }
                        self.emit_expr(arg);
                    }
                    // For recursive nested functions with captures, pass captured vars as extra args
                    if let Some(captures) = self.nested_fn_captures.get(func).cloned() {
                        for (idx, (cap_name, _)) in captures.iter().enumerate() {
                            if !args.is_empty() || idx > 0 {
                                self.write(", ");
                            }
                            self.write(cap_name);
                        }
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
            HirExpr::RangeLiteral {
                start, end, step, ..
            } => {
                if let Some(step) = step {
                    self.write("(");
                    self.emit_expr(start);
                    self.write("..");
                    self.emit_expr(end);
                    self.write(").step_by(");
                    self.emit_expr(step);
                    self.write(" as usize)");
                } else {
                    self.emit_expr(start);
                    self.write("..");
                    self.emit_expr(end);
                }
            }
            HirExpr::ListLiteral { elements, .. } => {
                self.write("vec![");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(elem);
                    if let HirExpr::Name { name, ty } = elem {
                        if matches!(ty, Type::TypeVar(_))
                            && (self.borrowed_params.contains(name.as_str())
                                || self.mut_borrowed_params.contains(name.as_str()))
                        {
                            self.write(".clone()");
                        }
                    }
                }
                self.write("]");
            }
            HirExpr::SetLiteral { elements, .. } => {
                self.collection_needs.needs_hashset = true;
                self.write("HashSet::from([");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(elem);
                }
                self.write("])");
            }
            HirExpr::DictLiteral { keys, values, .. } => {
                self.collection_needs.needs_hashmap = true;
                self.write("HashMap::from([");
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
                        // Safe dict indexing: d[key] -> d.get(key_ref).cloned()
                        // For self.field dict, we don't need to clone the field -- just borrow it.
                        let is_self_field = matches!(object.as_ref(), HirExpr::FieldAccess { object: inner, .. }
                            if matches!(inner.as_ref(), HirExpr::Name { name, .. } if name == "self"));
                        if is_self_field {
                            self.pending_self_field_clone_suppression += 1;
                        }
                        self.emit_expr(object);
                        self.write(".get(");
                        self.emit_key_ref_expr(index);
                        self.write(").cloned()");
                    }
                    Type::Tuple(_) => {
                        // Tuple indexing: t.0, t.1, etc. (handle negative)
                        // Tuples are fixed-size, so indexing is always safe at compile time
                        if let HirExpr::IntLiteral(val) = index.as_ref() {
                            if *val < 0 {
                                if let Type::Tuple(elems) = obj_ty {
                                    let resolved = i64::try_from(elems.len())
                                        .ok()
                                        .and_then(|len| len.checked_add(*val))
                                        .and_then(|idx| usize::try_from(idx).ok())
                                        .unwrap_or(0);
                                    self.emit_expr(object);
                                    self.write(&format!(".{resolved}"));
                                }
                            } else {
                                // Emit raw integer for tuple field access (e.g., .0 not .0_i64)
                                self.emit_expr(object);
                                self.write(&format!(".{val}"));
                            }
                        } else {
                            // Non-literal index: emit as raw integer (tuples require compile-time indices)
                            self.emit_expr(object);
                            self.write(".");
                            self.emit_expr(index);
                        }
                    }
                    Type::Str => {
                        // Safe string indexing: returns Option<String>
                        // Handle negative indices
                        self.write("{ let _s = &");
                        self.emit_expr(object);
                        self.write("; let _i = ");
                        self.emit_expr(index);
                        self.write("; let _idx = if _i < 0 { (_s.chars().count() as i64 + _i) as usize } else { _i as usize }; _s.chars().nth(_idx).map(|c| c.to_string()) }");
                    }
                    // Union/Optional type indexing: unwrap the Option first
                    ty if is_option_type(ty) => {
                        self.write("{ let __opt = ");
                        self.emit_expr(object);
                        self.write("; let _v = __opt.as_ref().unwrap(); let _i = ");
                        self.emit_expr(index);
                        self.write("; let _idx = if _i < 0 { (_v.len() as i64 + _i) as usize } else { _i as usize }; _v.get(_idx).cloned() }");
                    }
                    _ => {
                        // Safe list indexing: returns Option<T>
                        // Handle negative indices
                        self.write("{ let _v = &");
                        self.emit_expr(object);
                        self.write("; let _i = ");
                        self.emit_expr(index);
                        self.write("; let _idx = if _i < 0 { (_v.len() as i64 + _i) as usize } else { _i as usize }; _v.get(_idx).cloned() }");
                    }
                }
            }
            HirExpr::MethodCall {
                object,
                method,
                args,
                ..
            } => {
                self.emit_method_call(object, method, args);
            }
            HirExpr::ContainsOp {
                element,
                collection,
                ..
            } => {
                let coll_ty = collection.ty();
                match coll_ty {
                    Type::Dict(_, _) => {
                        self.emit_expr(collection);
                        self.write(".contains_key(");
                        self.emit_key_ref_expr(element);
                        self.write(")");
                    }
                    Type::Str => {
                        self.emit_expr(collection);
                        self.write(".contains(");
                        self.emit_str_ref_expr(element);
                        self.write(")");
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
            HirExpr::Slice {
                object,
                start,
                stop,
                step,
                ty,
            } => {
                let obj_ty = object.ty();
                match obj_ty {
                    Type::Str => {
                        self.emit_string_slice(
                            object,
                            start.as_deref(),
                            stop.as_deref(),
                            step.as_deref(),
                        );
                    }
                    Type::Tuple(_) => {
                        // Compile-time tuple slicing: direct field access
                        if let Type::Tuple(result_elems) = ty {
                            let start_idx = start
                                .as_ref()
                                .and_then(|e| match e.as_ref() {
                                    HirExpr::IntLiteral(v) => usize::try_from(*v).ok(),
                                    _ => None,
                                })
                                .unwrap_or(0);
                            self.write("(");
                            for (i, _) in result_elems.iter().enumerate() {
                                if i > 0 {
                                    self.write(", ");
                                }
                                self.emit_expr(object);
                                self.write(&format!(".{}", start_idx + i));
                            }
                            if result_elems.len() == 1 {
                                self.write(",");
                            }
                            self.write(")");
                        }
                    }
                    _ => {
                        // List slicing
                        self.emit_list_slice(
                            object,
                            start.as_deref(),
                            stop.as_deref(),
                            step.as_deref(),
                        );
                    }
                }
            }
            HirExpr::WalrusExpr { name, value: _, .. } => {
                // Walrus operator: the variable is already hoisted by emit_walrus_hoists
                // Just emit the variable name (the assignment was already emitted)
                self.write(name);
            }
            HirExpr::FieldAccess { object, field, ty } => {
                // Handle enum .name and .value as method calls
                if matches!(object.ty(), Type::Enum { .. }) {
                    self.emit_expr(object);
                    self.write(".");
                    self.write(field);
                    self.write("()");
                    return;
                }

                // Determine if we need .clone() (non-Copy field accessed on &self)
                let is_self_access =
                    matches!(object.as_ref(), HirExpr::Name { name, .. } if name == "self");
                let suppress_self_clone =
                    if is_self_access && self.pending_self_field_clone_suppression > 0 {
                        self.pending_self_field_clone_suppression -= 1;
                        true
                    } else {
                        false
                    };
                let needs_clone =
                    is_self_access && needs_clone_for_type(ty) && !suppress_self_clone;

                // Determine the class name for parent field resolution
                // Either from current_class_name (inside a method) or from the object's type
                let class_name_for_parent = if let Some(ref cn) = self.current_class_name {
                    if is_self_access {
                        Some(cn.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
                .or_else(|| {
                    // For external access like obj.field, check the object's type
                    if let Type::Class { name, .. } = object.ty() {
                        Some(name.clone())
                    } else {
                        None
                    }
                });

                // Check if this is accessing a parent field via inheritance
                if let Some(ref class_name) = class_name_for_parent {
                    if let Some((parent_name, parent_field_names)) =
                        self.parent_fields.get(class_name).cloned()
                    {
                        if parent_field_names.contains(field.as_str()) {
                            // Access via embedded parent: obj.parent.field
                            self.emit_expr(object);
                            self.write(".");
                            self.write(&parent_name.to_lowercase());
                            self.write(".");
                            self.write(field);
                            if needs_clone {
                                self.write(".clone()");
                            }
                            return;
                        }
                    }
                }
                self.emit_expr(object);
                self.write(".");
                self.write(field);
                if needs_clone {
                    self.write(".clone()");
                }
            }
            HirExpr::ConstructorCall {
                class_name, args, ..
            } => {
                // IOError subclasses map to IOError with a specific kind field
                let io_subclass_kind = match class_name.as_str() {
                    "FileNotFoundError" => Some("FileNotFound"),
                    "PermissionError" => Some("PermissionDenied"),
                    "FileExistsError" => Some("FileExists"),
                    "IsADirectoryError" => Some("IsADirectory"),
                    "NotADirectoryError" => Some("NotADirectory"),
                    "DirectoryNotEmptyError" => Some("DirectoryNotEmpty"),
                    _ => None,
                };
                if let Some(kind) = io_subclass_kind {
                    // Emit: IOError { message: <arg>.to_string(), kind: "<kind>".to_string() }
                    self.write("IOError { message: ");
                    if args.is_empty() {
                        self.write("String::new()");
                    } else {
                        self.emit_expr(&args[0]);
                        self.write(".to_string()");
                    }
                    self.write(&format!(", kind: \"{kind}\".to_string() }}"));
                    return;
                }
                self.write(class_name);
                self.write("::new(");
                let field_names = self.class_field_order.get(class_name).cloned();
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    // Check if this argument corresponds to a recursive field
                    let is_recursive = field_names.as_ref().is_some_and(|names| {
                        names.get(i).is_some_and(|fname| {
                            self.recursive_fields
                                .contains(&(class_name.clone(), fname.clone()))
                        })
                    });
                    if is_recursive {
                        if matches!(arg, HirExpr::NoneLiteral) {
                            // None stays as None for Option<Box<T>> fields
                            self.write("None");
                        } else {
                            // Wrap in Some(Box::new(...)) for Option<Box<T>> fields
                            // or Box::new(...) for direct recursive fields
                            self.write("Some(Box::new(");
                            self.emit_expr(arg);
                            self.write("))");
                        }
                    } else {
                        // If the argument is a borrowed parameter (non-Copy type),
                        // clone it since constructors expect owned values
                        let needs_clone = if let HirExpr::Name { name, ty } = arg {
                            self.borrowed_params.contains(name)
                                && ty.ownership() != sifr_type_system::OwnershipKind::Copy
                        } else {
                            false
                        };
                        self.emit_expr(arg);
                        if needs_clone {
                            self.write(".clone()");
                        }
                    }
                }
                self.write(")");
            }
            HirExpr::QuestionMark { expr, .. } => {
                self.emit_expr(expr);
                self.write("?");
            }
            HirExpr::OkWrap { value, .. } => {
                if matches!(value.as_ref(), HirExpr::NoneLiteral) {
                    self.write("Ok(())");
                } else {
                    self.write("Ok(");
                    self.emit_expr(value);
                    self.write(")");
                }
            }
            HirExpr::ErrWrap { value, .. } => {
                self.write("Err(");
                self.emit_expr(value);
                self.write(")");
            }
            HirExpr::FString { parts, .. } => {
                self.emit_fstring_macro("format!", parts);
            }
            HirExpr::SuperCall {
                parent_class,
                method,
                args,
                ..
            } => {
                // super().__init__(args) -> ParentType::new(args)
                self.write(parent_class);
                self.write("::");
                self.write(method);
                self.write("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(arg);
                }
                self.write(")");
            }
            HirExpr::Lambda { params, body, .. } => {
                self.write("|");
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&param.name);
                    // Only emit type annotation if it's not Any
                    if param.ty != Type::Any {
                        self.write(": ");
                        // For reference types, use &Type
                        if matches!(param.ty, Type::Str | Type::Class { .. }) {
                            self.write("&");
                        }
                        self.write(&param.ty.rust_type());
                    }
                }
                self.write("| ");
                self.emit_expr(body);
            }
            HirExpr::ListComp {
                expr,
                generators,
                ty,
            } => {
                if generators.len() == 1 {
                    // Single generator: use functional style
                    let (ref var, ref iter_e, ref filter) = generators[0];
                    let is_range = matches!(iter_e.ty(), Type::Range);
                    let var_pattern = if var.contains(',') {
                        let names: Vec<&str> = var.split(',').collect();
                        format!("({})", names.join(", "))
                    } else {
                        var.clone()
                    };
                    if is_range {
                        self.write("(");
                        self.emit_expr(iter_e);
                        self.write(")");
                    } else {
                        self.emit_expr(iter_e);
                        self.write(".clone().into_iter()");
                    }
                    if let Some(ref cond) = filter {
                        let elem_is_copy = if let Type::List(ref elem) = iter_e.ty() {
                            !needs_clone_for_type(elem)
                        } else {
                            is_range
                        };
                        if elem_is_copy && !var.contains(',') {
                            self.write(".filter(|&");
                        } else {
                            self.write(".filter(|");
                        }
                        self.write(&var_pattern);
                        self.write("| ");
                        self.emit_expr(cond);
                        self.write(")");
                    }
                    self.write(".map(|");
                    self.write(&var_pattern);
                    self.write("| ");
                    self.emit_expr(expr);
                    self.write(")");
                    if let Type::List(ref elem) = ty {
                        self.write(&format!(".collect::<Vec<{}>>()", elem.rust_type()));
                    } else {
                        self.write(".collect::<Vec<_>>()");
                    }
                } else {
                    // Multi-generator: use imperative style
                    self.write("{ let mut _result = Vec::new(); ");
                    for (var, iter_e, filter) in generators {
                        let var_pattern = if var.contains(',') {
                            let names: Vec<&str> = var.split(',').collect();
                            format!("({})", names.join(", "))
                        } else {
                            var.clone()
                        };
                        let is_range = matches!(iter_e.ty(), Type::Range);
                        self.write("for ");
                        self.write(&var_pattern);
                        self.write(" in ");
                        if is_range {
                            self.write("(");
                            self.emit_expr(iter_e);
                            self.write(")");
                        } else {
                            self.emit_expr(iter_e);
                            self.write(".clone().into_iter()");
                        }
                        self.write(" { ");
                        if let Some(ref cond) = filter {
                            self.write("if ");
                            self.emit_expr(cond);
                            self.write(" { ");
                        }
                    }
                    self.write("_result.push(");
                    self.emit_expr(expr);
                    self.write("); ");
                    // Close filter ifs and for loops (in reverse)
                    for (_, _, ref filter) in generators.iter().rev() {
                        if filter.is_some() {
                            self.write("} ");
                        }
                        self.write("} ");
                    }
                    self.write("_result }");
                }
            }
            HirExpr::SetComp {
                expr,
                generators,
                ty,
            } => {
                self.collection_needs.needs_hashset = true;
                if generators.len() == 1 {
                    let (ref var, ref iter_e, ref filter) = generators[0];
                    let is_range = matches!(iter_e.ty(), Type::Range);
                    let var_pattern = if var.contains(',') {
                        let names: Vec<&str> = var.split(',').collect();
                        format!("({})", names.join(", "))
                    } else {
                        var.clone()
                    };
                    if is_range {
                        self.write("(");
                        self.emit_expr(iter_e);
                        self.write(")");
                    } else {
                        self.emit_expr(iter_e);
                        self.write(".clone().into_iter()");
                    }
                    if let Some(ref cond) = filter {
                        self.write(".filter(|");
                        self.write(&var_pattern);
                        self.write("| ");
                        self.emit_expr(cond);
                        self.write(")");
                    }
                    self.write(".map(|");
                    self.write(&var_pattern);
                    self.write("| ");
                    self.emit_expr(expr);
                    self.write(")");
                    if let Type::Set(ref elem) = ty {
                        self.write(&format!(".collect::<HashSet<{}>>()", elem.rust_type()));
                    } else {
                        self.write(".collect::<HashSet<_>>()");
                    }
                } else {
                    self.write("{ let mut _result = HashSet::new(); ");
                    for (var, iter_e, filter) in generators {
                        self.write("for ");
                        self.write(var);
                        self.write(" in ");
                        self.emit_expr(iter_e);
                        self.write(".clone().into_iter() { ");
                        if let Some(ref cond) = filter {
                            self.write("if ");
                            self.emit_expr(cond);
                            self.write(" { ");
                        }
                    }
                    self.write("_result.insert(");
                    self.emit_expr(expr);
                    self.write("); ");
                    for (_, _, ref filter) in generators.iter().rev() {
                        if filter.is_some() {
                            self.write("} ");
                        }
                        self.write("} ");
                    }
                    self.write("_result }");
                }
            }
            HirExpr::DictComp {
                key_expr,
                val_expr,
                generators,
                ty,
            } => {
                self.collection_needs.needs_hashmap = true;
                if generators.len() == 1 {
                    let (ref var, ref iter_e, ref filter) = generators[0];
                    let is_range = matches!(iter_e.ty(), Type::Range);
                    let var_pattern = if var.contains(',') {
                        let names: Vec<&str> = var.split(',').collect();
                        format!("({})", names.join(", "))
                    } else {
                        var.clone()
                    };
                    if is_range {
                        self.write("(");
                        self.emit_expr(iter_e);
                        self.write(")");
                    } else {
                        self.emit_expr(iter_e);
                        self.write(".clone().into_iter()");
                    }
                    if let Some(ref cond) = filter {
                        self.write(".filter(|");
                        self.write(&var_pattern);
                        self.write("| ");
                        self.emit_expr(cond);
                        self.write(")");
                    }
                    self.write(".map(|");
                    self.write(&var_pattern);
                    self.write("| (");
                    self.emit_expr(key_expr);
                    self.write(", ");
                    self.emit_expr(val_expr);
                    self.write("))");
                    if let Type::Dict(ref k, ref v) = ty {
                        self.write(&format!(
                            ".collect::<HashMap<{}, {}>>()",
                            k.rust_type(),
                            v.rust_type()
                        ));
                    } else {
                        self.write(".collect::<HashMap<_, _>>()");
                    }
                } else {
                    self.write("{ let mut _result = HashMap::new(); ");
                    for (var, iter_e, filter) in generators {
                        let var_pattern = if var.contains(',') {
                            let names: Vec<&str> = var.split(',').collect();
                            format!("({})", names.join(", "))
                        } else {
                            var.clone()
                        };
                        self.write("for ");
                        self.write(&var_pattern);
                        self.write(" in ");
                        self.emit_expr(iter_e);
                        self.write(".clone().into_iter() { ");
                        if let Some(ref cond) = filter {
                            self.write("if ");
                            self.emit_expr(cond);
                            self.write(" { ");
                        }
                    }
                    self.write("_result.insert(");
                    self.emit_expr(key_expr);
                    self.write(", ");
                    self.emit_expr(val_expr);
                    self.write("); ");
                    for (_, _, ref filter) in generators.iter().rev() {
                        if filter.is_some() {
                            self.write("} ");
                        }
                        self.write("} ");
                    }
                    self.write("_result }");
                }
            }
            HirExpr::GeneratorExpr {
                expr,
                var,
                iter,
                filter,
                ..
            } => {
                // (expr for var in iter) -> iter.clone().into_iter().map(|var| expr)
                // Lazy iterator - no .collect()
                self.emit_expr(iter);
                if filter.is_some() {
                    self.write(".iter()");
                    if let Some(ref cond) = filter {
                        self.write(".filter(|");
                        self.write(var);
                        self.write("| { let ");
                        self.write(var);
                        self.write(" = **");
                        self.write(var);
                        self.write("; ");
                        self.emit_expr(cond);
                        self.write(" })");
                    }
                    self.write(".map(|");
                    self.write(var);
                    self.write("| { let ");
                    self.write(var);
                    self.write(" = *");
                    self.write(var);
                    self.write("; ");
                    self.emit_expr(expr);
                    self.write(" })");
                } else {
                    self.write(".clone().into_iter()");
                    self.write(".map(|");
                    self.write(var);
                    self.write("| ");
                    self.emit_expr(expr);
                    self.write(")");
                }
                // No .collect() - lazy iterator
            }
            HirExpr::EnumVariant {
                enum_name, variant, ..
            } => {
                // Color.RED -> Color::RED
                self.write(enum_name);
                self.write("::");
                self.write(variant);
            }
        }
    }
}
