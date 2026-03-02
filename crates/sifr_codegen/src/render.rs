//! Renderer for structured Rust IR nodes.

use crate::{
    RustExpr, RustFile, RustItem, RustLiteral, RustMatchArm, RustParam, RustStmt, RustType,
    Visibility,
};
use std::fmt::Write as _;

pub struct Renderer {
    output: String,
    indent: usize,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
        }
    }

    pub fn render_file(&mut self, file: &RustFile) -> String {
        self.output.clear();
        self.indent = 0;
        for (idx, item) in file.items.iter().enumerate() {
            self.render_item(item);
            let keeps_tight_spacing = matches!(item, RustItem::Attr(_));
            if idx + 1 < file.items.len() && !self.output.ends_with("\n\n") && !keeps_tight_spacing
            {
                let _ = self.output.write_char('\n');
            }
        }
        self.output.clone()
    }

    pub fn render_item(&mut self, item: &RustItem) {
        match item {
            RustItem::Use(path) => self.emit_line(&format!("use {};", path.join("::"))),
            RustItem::UseAlias { path, alias } => {
                self.emit_line(&format!("use {} as {};", path.join("::"), alias));
            }
            RustItem::Struct {
                name,
                visibility,
                derives,
                fields,
            } => {
                self.render_derives(derives);
                self.emit_line(&format!(
                    "{}struct {} {{",
                    Self::render_visibility(visibility),
                    name
                ));
                self.indent();
                for (field_name, field_ty) in fields {
                    self.emit_line(&format!(
                        "{}: {},",
                        field_name,
                        Self::render_type_string(field_ty)
                    ));
                }
                self.dedent();
                self.emit_line("}");
            }
            RustItem::TupleStruct {
                name,
                visibility,
                derives,
                inner,
            } => {
                self.render_derives(derives);
                self.emit_line(&format!(
                    "{}struct {}({});",
                    Self::render_visibility(visibility),
                    name,
                    Self::render_type_string(inner)
                ));
            }
            RustItem::Enum {
                name,
                visibility,
                derives,
                repr,
                variants,
            } => {
                self.render_derives(derives);
                if let Some(repr_name) = repr {
                    self.emit_line(&format!("#[repr({repr_name})]"));
                }
                self.emit_line(&format!(
                    "{}enum {} {{",
                    Self::render_visibility(visibility),
                    name
                ));
                self.indent();
                for variant in variants {
                    let rendered = if !variant.tuple_fields.is_empty() {
                        let fields = variant
                            .tuple_fields
                            .iter()
                            .map(Self::render_type_string)
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!("{}({})", variant.name, fields)
                    } else if !variant.fields.is_empty() {
                        let fields = variant
                            .fields
                            .iter()
                            .map(|(f, t)| format!("{f}: {}", Self::render_type_string(t)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!("{} {{ {} }}", variant.name, fields)
                    } else if let Some(value) = &variant.value {
                        format!("{} = {}", variant.name, Self::render_expr_string(value))
                    } else {
                        variant.name.clone()
                    };
                    self.emit_line(&format!("{rendered},"));
                }
                self.dedent();
                self.emit_line("}");
            }
            RustItem::Trait {
                name,
                visibility,
                supertraits,
                methods,
            } => {
                let supers = if supertraits.is_empty() {
                    String::new()
                } else {
                    format!(": {}", supertraits.join(" + "))
                };
                self.emit_line(&format!(
                    "{}trait {}{} {{",
                    Self::render_visibility(visibility),
                    name,
                    supers
                ));
                self.indent();
                for method in methods {
                    self.render_item(method);
                }
                self.dedent();
                self.emit_line("}");
            }
            RustItem::Impl {
                target,
                type_params,
                trait_,
                items,
            } => {
                let generics = if type_params.is_empty() {
                    String::new()
                } else {
                    let params = type_params
                        .iter()
                        .map(|p| {
                            if p.bounds.is_empty() {
                                p.name.clone()
                            } else {
                                format!("{}: {}", p.name, p.bounds.join(" + "))
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("<{params}>")
                };
                let head = if let Some(trait_name) = trait_ {
                    format!("impl{generics} {trait_name} for {target} {{")
                } else {
                    format!("impl{generics} {target} {{")
                };
                self.emit_line(&head);
                self.indent();
                for impl_item in items {
                    self.render_item(impl_item);
                }
                self.dedent();
                self.emit_line("}");
            }
            RustItem::Fn {
                name,
                visibility,
                type_params,
                params,
                ret,
                body,
                is_async,
            } => {
                let async_prefix = if *is_async { "async " } else { "" };
                let type_params = if type_params.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<{}>",
                        type_params
                            .iter()
                            .map(|p| {
                                if p.bounds.is_empty() {
                                    p.name.clone()
                                } else {
                                    format!("{}: {}", p.name, p.bounds.join(" + "))
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                let params = params
                    .iter()
                    .map(Self::render_param_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                let ret = ret
                    .as_ref()
                    .map(|t| format!(" -> {}", Self::render_type_string(t)))
                    .unwrap_or_default();

                self.emit_line(&format!(
                    "{}{}fn {}{}({}){} {{",
                    Self::render_visibility(visibility),
                    async_prefix,
                    name,
                    type_params,
                    params,
                    ret
                ));
                self.indent();
                for stmt in body {
                    self.render_stmt(stmt);
                }
                self.dedent();
                self.emit_line("}");
            }
            RustItem::TraitMethodSig { name, params, ret } => {
                let params = params
                    .iter()
                    .map(Self::render_param_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                let ret = ret
                    .as_ref()
                    .map(|t| format!(" -> {}", Self::render_type_string(t)))
                    .unwrap_or_default();
                self.emit_line(&format!("fn {name}({params}){ret};"));
            }
            RustItem::TypeAlias { name, ty } => {
                self.emit_line(&format!("type {name} = {};", Self::render_type_string(ty)));
            }
            RustItem::Const {
                name,
                visibility,
                ty,
                value,
            } => {
                self.emit_line(&format!(
                    "{}const {}: {} = {};",
                    Self::render_visibility(visibility),
                    name,
                    Self::render_type_string(ty),
                    Self::render_expr_string(value)
                ));
            }
            RustItem::Static {
                name,
                visibility,
                ty,
                value,
            } => {
                self.emit_line(&format!(
                    "{}static {}: {} = {};",
                    Self::render_visibility(visibility),
                    name,
                    Self::render_type_string(ty),
                    Self::render_expr_string(value)
                ));
            }
            RustItem::Attr(attr) => self.emit_line(attr),
            RustItem::SynItem(code) | RustItem::RawCode(code) => self.write_raw_top_level(code),
        }
    }

    pub fn render_stmt(&mut self, stmt: &RustStmt) {
        match stmt {
            RustStmt::Let {
                mutable,
                name,
                ty,
                value,
            } => {
                let mutability = if *mutable { "mut " } else { "" };
                let ty = ty
                    .as_ref()
                    .map(|t| format!(": {}", Self::render_type_string(t)))
                    .unwrap_or_default();
                self.emit_line(&format!(
                    "let {mutability}{name}{ty} = {};",
                    Self::render_expr_string(value)
                ));
            }
            RustStmt::LetPattern { pattern, value } => {
                self.emit_line(&format!(
                    "let {pattern} = {};",
                    Self::render_expr_string(value)
                ));
            }
            RustStmt::Assign { target, value } => {
                self.emit_line(&format!(
                    "{} = {};",
                    Self::render_expr_string(target),
                    Self::render_expr_string(value)
                ));
            }
            RustStmt::AugAssign { target, op, value } => {
                self.emit_line(&format!(
                    "{} {}= {};",
                    Self::render_expr_string(target),
                    op,
                    Self::render_expr_string(value)
                ));
            }
            RustStmt::Expr(expr) => self.emit_line(&format!("{};", Self::render_expr_string(expr))),
            RustStmt::Assert { cond, msg: None } => {
                self.emit_line(&format!("assert!({});", Self::render_expr_string(cond)));
            }
            RustStmt::Assert {
                cond,
                msg: Some(msg),
            } => {
                self.emit_line(&format!(
                    "assert!({}, \"{{}}\", {});",
                    Self::render_expr_string(cond),
                    Self::render_expr_string(msg)
                ));
            }
            RustStmt::Return(Some(expr)) => {
                self.emit_line(&format!("return {};", Self::render_expr_string(expr)));
            }
            RustStmt::Return(None) => self.emit_line("return;"),
            RustStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                self.emit_line(&format!("if {} {{", Self::render_expr_string(cond)));
                self.indent();
                for stmt in then_body {
                    self.render_stmt(stmt);
                }
                self.dedent();
                if let Some(else_body) = else_body {
                    self.emit_line("} else {");
                    self.indent();
                    for stmt in else_body {
                        self.render_stmt(stmt);
                    }
                    self.dedent();
                    self.emit_line("}");
                } else {
                    self.emit_line("}");
                }
            }
            RustStmt::IfLet {
                pattern,
                expr,
                then_body,
                else_body,
            } => {
                self.emit_line(&format!(
                    "if let {} = {} {{",
                    pattern,
                    Self::render_expr_string(expr)
                ));
                self.indent();
                for stmt in then_body {
                    self.render_stmt(stmt);
                }
                self.dedent();
                if let Some(else_body) = else_body {
                    self.emit_line("} else {");
                    self.indent();
                    for stmt in else_body {
                        self.render_stmt(stmt);
                    }
                    self.dedent();
                    self.emit_line("}");
                } else {
                    self.emit_line("}");
                }
            }
            RustStmt::Match { expr, arms } => {
                self.emit_line(&format!("match {} {{", Self::render_expr_string(expr)));
                self.indent();
                self.render_match_arms(arms);
                self.dedent();
                self.emit_line("}");
            }
            RustStmt::For { var, iter, body } => {
                self.emit_line(&format!(
                    "for {var} in {} {{",
                    Self::render_expr_string(iter)
                ));
                self.indent();
                for stmt in body {
                    self.render_stmt(stmt);
                }
                self.dedent();
                self.emit_line("}");
            }
            RustStmt::With { items, body } => {
                self.emit_line("{");
                self.indent();
                for (idx, item) in items.iter().enumerate() {
                    let value = Self::render_expr_string(&item.value);
                    if item.has_cm {
                        let Some(class_name) = item.class_name.as_ref() else {
                            panic!("with-item missing class_name for context manager rendering");
                        };
                        let ctx_name = format!("__ctx_{idx}");
                        let guard_type = format!("__WithGuard{idx}");
                        let guard_var = format!("__guard_{idx}");
                        self.emit_line(&format!("let mut {ctx_name} = {value};"));
                        self.emit_line(&format!("struct {guard_type} {{ ctx: {class_name} }}"));
                        self.emit_line(&format!("impl Drop for {guard_type} {{"));
                        self.indent();
                        self.emit_line("fn drop(&mut self) { self.ctx.__exit__(); }");
                        self.dedent();
                        self.emit_line("}");
                        self.emit_line(&format!(
                            "let mut {guard_var} = {guard_type} {{ ctx: {ctx_name} }};"
                        ));
                        self.emit_line(&format!(
                            "let {} = {guard_var}.ctx.__enter__();",
                            item.binding
                        ));
                    } else {
                        self.emit_line(&format!("let {} = {value};", item.binding));
                    }
                }
                for stmt in body {
                    self.render_stmt(stmt);
                }
                self.dedent();
                self.emit_line("}");
            }
            RustStmt::While { cond, body } => {
                self.emit_line(&format!("while {} {{", Self::render_expr_string(cond)));
                self.indent();
                for stmt in body {
                    self.render_stmt(stmt);
                }
                self.dedent();
                self.emit_line("}");
            }
            RustStmt::Loop { body } => {
                self.emit_line("loop {");
                self.indent();
                for stmt in body {
                    self.render_stmt(stmt);
                }
                self.dedent();
                self.emit_line("}");
            }
            RustStmt::LocalFn {
                name,
                params,
                ret,
                body,
            } => {
                let rendered_params = params
                    .iter()
                    .map(Self::render_param_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                if let Some(ret) = ret {
                    self.emit_line(&format!(
                        "fn {name}({rendered_params}) -> {} {{",
                        Self::render_type_string(ret)
                    ));
                } else {
                    self.emit_line(&format!("fn {name}({rendered_params}) {{"));
                }
                self.indent();
                for stmt in body {
                    self.render_stmt(stmt);
                }
                self.dedent();
                self.emit_line("}");
            }
            RustStmt::Break => self.emit_line("break;"),
            RustStmt::Continue => self.emit_line("continue;"),
            RustStmt::Block(stmts) => {
                self.emit_line("{");
                self.indent();
                for stmt in stmts {
                    self.render_stmt(stmt);
                }
                self.dedent();
                self.emit_line("}");
            }
            RustStmt::RawCode(code) => self.write_raw_stmt(code),
        }
    }

    pub fn render_expr(&mut self, expr: &RustExpr) {
        self.append(&Self::render_expr_string(expr));
    }

    pub fn render_type(&mut self, ty: &RustType) {
        self.append(&Self::render_type_string(ty));
    }

    fn append(&mut self, s: &str) {
        let _ = write!(self.output, "{s}");
    }

    fn emit_line(&mut self, s: &str) {
        self.write_indent();
        self.append(s);
        let _ = self.output.write_char('\n');
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            let _ = write!(self.output, "    ");
        }
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    fn render_visibility(visibility: &Visibility) -> &'static str {
        match visibility {
            Visibility::Private => "",
            Visibility::Pub => "pub ",
        }
    }

    fn render_derives(&mut self, derives: &[String]) {
        if !derives.is_empty() {
            self.emit_line(&format!("#[derive({})]", derives.join(", ")));
        }
    }

    fn render_param_string(param: &RustParam) -> String {
        match param {
            RustParam::SelfParam { mutable } => {
                if *mutable {
                    "&mut self".to_string()
                } else {
                    "&self".to_string()
                }
            }
            RustParam::SelfValue => "self".to_string(),
            RustParam::Named { name, ty } => format!("{name}: {}", Self::render_type_string(ty)),
        }
    }

    fn render_type_string(ty: &RustType) -> String {
        match ty {
            RustType::I64 => "i64".to_string(),
            RustType::F64 => "f64".to_string(),
            RustType::Bool => "bool".to_string(),
            RustType::String_ => "String".to_string(),
            RustType::Unit => "()".to_string(),
            RustType::Vec(inner) => format!("Vec<{}>", Self::render_type_string(inner)),
            RustType::HashMap(key, value) => {
                format!(
                    "HashMap<{}, {}>",
                    Self::render_type_string(key),
                    Self::render_type_string(value)
                )
            }
            RustType::HashSet(inner) => format!("HashSet<{}>", Self::render_type_string(inner)),
            RustType::VecDeque(inner) => format!("VecDeque<{}>", Self::render_type_string(inner)),
            RustType::Option(inner) => format!("Option<{}>", Self::render_type_string(inner)),
            RustType::Result(ok, err) => format!(
                "Result<{}, {}>",
                Self::render_type_string(ok),
                Self::render_type_string(err)
            ),
            RustType::Tuple(items) => {
                let rendered = items
                    .iter()
                    .map(Self::render_type_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                if items.len() == 1 {
                    format!("({rendered},)")
                } else {
                    format!("({rendered})")
                }
            }
            RustType::Ref { mutable, inner } => {
                if *mutable {
                    format!("&mut {}", Self::render_type_string(inner))
                } else {
                    format!("&{}", Self::render_type_string(inner))
                }
            }
            RustType::Named(name) => name.clone(),
            RustType::Generic { base, params } => format!(
                "{base}<{}>",
                params
                    .iter()
                    .map(Self::render_type_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            RustType::Fn { params, ret } => format!(
                "fn({}) -> {}",
                params
                    .iter()
                    .map(Self::render_type_string)
                    .collect::<Vec<_>>()
                    .join(", "),
                Self::render_type_string(ret)
            ),
            RustType::DynTrait(name) => format!("dyn {name}"),
            RustType::Impl(name) => format!("impl {name}"),
            RustType::RawCode(code) => code.clone(),
        }
    }

    fn render_expr_string(expr: &RustExpr) -> String {
        match expr {
            RustExpr::Literal(lit) => Self::render_literal(lit),
            RustExpr::Ident(name) => name.clone(),
            RustExpr::Path(parts) => parts.join("::"),
            RustExpr::MethodCall {
                receiver,
                method,
                args,
            } => format!(
                "{}.{method}({})",
                Self::wrap_expr(receiver),
                args.iter()
                    .map(Self::render_expr_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            RustExpr::FnCall { func, args } => format!(
                "{}({})",
                Self::wrap_expr(func),
                args.iter()
                    .map(Self::render_expr_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            RustExpr::MacroCall { name, args } => format!(
                "{name}!({})",
                args.iter()
                    .enumerate()
                    .map(|(idx, arg)| Self::render_macro_arg(name, idx, arg))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            RustExpr::FormatMacro {
                name,
                format_str,
                args,
            } => {
                let escaped = format!("\"{}\"", format_str.escape_default());
                if args.is_empty() {
                    format!("{name}!({escaped})")
                } else {
                    format!(
                        "{name}!({escaped}, {})",
                        args.iter()
                            .map(Self::render_expr_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            RustExpr::BinOp { left, op, right } => {
                format!("{} {op} {}", Self::wrap_expr(left), Self::wrap_expr(right))
            }
            RustExpr::UnaryOp { op, operand } => format!("{op}{}", Self::wrap_expr(operand)),
            RustExpr::Field { expr, field } => format!("{}.{}", Self::wrap_expr(expr), field),
            RustExpr::Index { expr, index } => {
                format!(
                    "{}[{}]",
                    Self::wrap_expr(expr),
                    Self::render_expr_string(index)
                )
            }
            RustExpr::Slice { expr, start, stop } => {
                let start_rendered = start
                    .as_ref()
                    .map(|s| Self::render_expr_string(s))
                    .unwrap_or_default();
                let stop_rendered = stop
                    .as_ref()
                    .map(|s| Self::render_expr_string(s))
                    .unwrap_or_default();
                format!(
                    "{}[{}..{}]",
                    Self::wrap_expr(expr),
                    start_rendered,
                    stop_rendered
                )
            }
            RustExpr::Ref { mutable, expr } => {
                if *mutable {
                    format!("&mut {}", Self::wrap_expr(expr))
                } else {
                    format!("&{}", Self::wrap_expr(expr))
                }
            }
            RustExpr::Deref(expr) => format!("*{}", Self::wrap_expr(expr)),
            RustExpr::Clone(expr) => format!("{}.clone()", Self::wrap_expr(expr)),
            RustExpr::Cast { expr, ty } => {
                format!(
                    "{} as {}",
                    Self::wrap_expr(expr),
                    Self::render_type_string(ty)
                )
            }
            RustExpr::Block { stmts, expr } => Self::render_block_expr(stmts, expr.as_deref()),
            RustExpr::If {
                cond,
                then_expr,
                else_expr,
            } => {
                let mut out = format!(
                    "if {} {{ {} }}",
                    Self::render_expr_string(cond),
                    Self::render_expr_string(then_expr)
                );
                if let Some(else_expr) = else_expr {
                    let _ = write!(out, " else {{ {} }}", Self::render_expr_string(else_expr));
                }
                out
            }
            RustExpr::Match { expr, arms } => {
                let mut renderer = Renderer::new();
                renderer.append(&format!("match {} {{\n", Self::render_expr_string(expr)));
                renderer.indent();
                renderer.render_match_arms(arms);
                renderer.dedent();
                renderer.append("}");
                renderer.output
            }
            RustExpr::Closure {
                params,
                body,
                is_move,
            } => {
                let move_kw = if *is_move { "move " } else { "" };
                let params = params
                    .iter()
                    .map(Self::render_closure_param_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{move_kw}|{params}| {}", Self::render_expr_string(body))
            }
            RustExpr::ClosureBlock {
                params,
                body,
                is_move,
            } => {
                let move_kw = if *is_move { "move " } else { "" };
                let params = params
                    .iter()
                    .map(Self::render_closure_param_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut renderer = Renderer::new();
                renderer.append(&format!("{move_kw}|{params}| {{\n"));
                renderer.indent();
                for stmt in body {
                    renderer.render_stmt(stmt);
                }
                renderer.dedent();
                renderer.append("}");
                renderer.output
            }
            RustExpr::StructInit { name, fields } => format!(
                "{name} {{ {} }}",
                fields
                    .iter()
                    .map(|(field, value)| format!("{field}: {}", Self::render_expr_string(value)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            RustExpr::Tuple(values) => {
                let rendered = values
                    .iter()
                    .map(Self::render_expr_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                if values.len() == 1 {
                    format!("({rendered},)")
                } else {
                    format!("({rendered})")
                }
            }
            RustExpr::Vec(values) => format!(
                "vec![{}]",
                values
                    .iter()
                    .map(Self::render_expr_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            RustExpr::Try(expr) => format!("{}?", Self::wrap_expr(expr)),
            RustExpr::Await(expr) => format!("{}.await", Self::wrap_expr(expr)),
            RustExpr::Paren(expr) => format!("({})", Self::render_expr_string(expr)),
            RustExpr::Range { start, end } => format!(
                "{}..{}",
                Self::render_expr_string(start),
                Self::render_expr_string(end)
            ),
            RustExpr::RawCode(code) => code.clone(),
        }
    }

    fn render_literal(lit: &RustLiteral) -> String {
        match lit {
            RustLiteral::Int(v) => v.to_string(),
            RustLiteral::Float(v) => {
                if v.fract() == 0.0 {
                    format!("{v:.1}")
                } else {
                    v.to_string()
                }
            }
            RustLiteral::Bool(v) => v.to_string(),
            RustLiteral::Str(v) => format!("\"{}\".to_string()", v.escape_default()),
            RustLiteral::Char(v) => format!("'{}'", v.escape_default()),
            RustLiteral::Unit => "()".to_string(),
            RustLiteral::None => "None".to_string(),
        }
    }

    fn render_macro_arg(name: &str, idx: usize, arg: &RustExpr) -> String {
        // `write!` / `writeln!` require the format string as a literal token
        // (second argument after the destination writer), not a `String`.
        if matches!(name, "write" | "writeln") && idx == 1 {
            if let RustExpr::Literal(RustLiteral::Str(value)) = arg {
                return format!("\"{}\"", value.escape_default());
            }
        }
        // `assert!`/`assert_eq!`/`assert_ne!` message format arguments must be
        // literal tokens, not owned `String` expressions.
        if matches!(name, "assert" | "assert_eq" | "assert_ne") {
            if let RustExpr::Literal(RustLiteral::Str(value)) = arg {
                return format!("\"{}\"", value.escape_default());
            }
        }
        Self::render_expr_string(arg)
    }

    fn wrap_expr(expr: &RustExpr) -> String {
        if Self::expr_requires_parens(expr) {
            format!("({})", Self::render_expr_string(expr))
        } else {
            Self::render_expr_string(expr)
        }
    }

    fn expr_requires_parens(expr: &RustExpr) -> bool {
        // Check if expr is one of the types that always needs parens
        if matches!(
            expr,
            RustExpr::BinOp { .. }
                | RustExpr::Cast { .. }
                | RustExpr::If { .. }
                | RustExpr::Match { .. }
                | RustExpr::Closure { .. }
                | RustExpr::ClosureBlock { .. }
                | RustExpr::Block { .. }
                | RustExpr::Range { .. }
        ) {
            return true;
        }
        // Also check if an Ident contains a cast expression (contains " as ")
        // This handles cases like "(2 as i64)" passed as an Ident string
        if let RustExpr::Ident(name) = expr {
            if name.contains(" as ") {
                return true;
            }
        }
        false
    }

    fn render_closure_param_string(param: &RustParam) -> String {
        match param {
            RustParam::SelfParam { .. } | RustParam::SelfValue => "self".to_string(),
            RustParam::Named { name, .. } => name.clone(),
        }
    }

    fn render_block_expr(stmts: &[RustStmt], trailing_expr: Option<&RustExpr>) -> String {
        let mut renderer = Renderer::new();
        renderer.append("{\n");
        renderer.indent();
        for stmt in stmts {
            renderer.render_stmt(stmt);
        }
        if let Some(expr) = trailing_expr {
            renderer.write_indent();
            renderer.append(&Self::render_expr_string(expr));
            let _ = renderer.output.write_char('\n');
        }
        renderer.dedent();
        renderer.append("}");
        renderer.output
    }

    fn render_match_arms(&mut self, arms: &[RustMatchArm]) {
        for arm in arms {
            let guard = arm
                .guard
                .as_ref()
                .map(|g| format!(" if {}", Self::render_expr_string(g)))
                .unwrap_or_default();
            self.emit_line(&format!("{}{} => {{", arm.pattern, guard));
            self.indent();
            for stmt in &arm.body {
                self.render_stmt(stmt);
            }
            self.dedent();
            self.emit_line("},");
        }
    }

    fn write_raw_top_level(&mut self, code: &str) {
        self.append(code);
        if !code.ends_with('\n') {
            let _ = self.output.write_char('\n');
        }
    }

    fn write_raw_stmt(&mut self, code: &str) {
        for line in code.lines() {
            self.emit_line(line);
        }
        if code.is_empty() {
            self.emit_line("");
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_items(items: &[RustItem]) -> String {
    let file = RustFile {
        items: items.to_vec(),
    };
    Renderer::new().render_file(&file)
}

pub fn render_stmts(stmts: &[RustStmt]) -> String {
    let mut renderer = Renderer::new();
    for stmt in stmts {
        renderer.render_stmt(stmt);
    }
    renderer.output
}

pub fn render_expr(expr: &RustExpr) -> String {
    let mut renderer = Renderer::new();
    renderer.render_expr(expr);
    renderer.output
}

pub fn render_type(ty: &RustType) -> String {
    let mut renderer = Renderer::new();
    renderer.render_type(ty);
    renderer.output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RustEnumVariant, RustMatchArm, RustTypeParam, Visibility};
    use insta::assert_snapshot;

    #[test]
    fn renders_struct_enum_trait_and_impl() {
        let items = vec![
            RustItem::Struct {
                name: "Point".to_string(),
                visibility: Visibility::Pub,
                derives: vec!["Debug".to_string(), "Clone".to_string()],
                fields: vec![
                    ("x".to_string(), RustType::I64),
                    ("y".to_string(), RustType::I64),
                ],
            },
            RustItem::Enum {
                name: "Token".to_string(),
                visibility: Visibility::Private,
                derives: vec!["Debug".to_string()],
                repr: None,
                variants: vec![
                    RustEnumVariant {
                        name: "Int".to_string(),
                        tuple_fields: vec![],
                        fields: vec![("value".to_string(), RustType::I64)],
                        value: None,
                    },
                    RustEnumVariant {
                        name: "Eof".to_string(),
                        tuple_fields: vec![],
                        fields: vec![],
                        value: None,
                    },
                    RustEnumVariant {
                        name: "Bytes".to_string(),
                        tuple_fields: vec![RustType::Vec(Box::new(RustType::I64))],
                        fields: vec![],
                        value: None,
                    },
                ],
            },
            RustItem::Trait {
                name: "Renderable".to_string(),
                visibility: Visibility::Pub,
                supertraits: vec![],
                methods: vec![RustItem::Fn {
                    name: "render".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: false }],
                    ret: Some(RustType::String_),
                    body: vec![RustStmt::Return(Some(RustExpr::Literal(RustLiteral::Str(
                        "ok".to_string(),
                    ))))],
                    is_async: false,
                }],
            },
            RustItem::Impl {
                target: "Point".to_string(),
                type_params: vec![RustTypeParam {
                    name: "T".to_string(),
                    bounds: vec!["Clone".to_string()],
                }],
                trait_: Some("Renderable".to_string()),
                items: vec![RustItem::Fn {
                    name: "render".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: false }],
                    ret: Some(RustType::String_),
                    body: vec![RustStmt::Return(Some(RustExpr::Literal(RustLiteral::Str(
                        "Point".to_string(),
                    ))))],
                    is_async: false,
                }],
            },
        ];

        let rendered = render_items(&items);
        assert_snapshot!(rendered, @r###"
        #[derive(Debug, Clone)]
        pub struct Point {
            x: i64,
            y: i64,
        }

        #[derive(Debug)]
        enum Token {
            Int { value: i64 },
            Eof,
            Bytes(Vec<i64>),
        }

        pub trait Renderable {
            fn render(&self) -> String {
                return "ok".to_string();
            }
        }

        impl<T: Clone> Renderable for Point {
            fn render(&self) -> String {
                return "Point".to_string();
            }
        }
        "###);
    }

    #[test]
    fn renders_use_alias_item() {
        let rendered = render_items(&[RustItem::UseAlias {
            path: vec![
                "crate".to_string(),
                "utils".to_string(),
                "helper".to_string(),
            ],
            alias: "h".to_string(),
        }]);
        assert_eq!(rendered, "use crate::utils::helper as h;\n");
    }

    #[test]
    fn renders_function_with_control_flow_statements() {
        let item = RustItem::Fn {
            name: "control".to_string(),
            visibility: Visibility::Pub,
            type_params: vec![],
            params: vec![RustParam::Named {
                name: "items".to_string(),
                ty: RustType::Vec(Box::new(RustType::I64)),
            }],
            ret: Some(RustType::Unit),
            body: vec![
                RustStmt::Let {
                    mutable: true,
                    name: "acc".to_string(),
                    ty: Some(RustType::I64),
                    value: RustExpr::Literal(RustLiteral::Int(0)),
                },
                RustStmt::If {
                    cond: RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("acc".to_string())),
                        op: "==".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                    },
                    then_body: vec![RustStmt::Expr(RustExpr::MacroCall {
                        name: "println".to_string(),
                        args: vec![RustExpr::Literal(RustLiteral::Str("empty".to_string()))],
                    })],
                    else_body: None,
                },
                RustStmt::For {
                    var: "value".to_string(),
                    iter: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("items".to_string())),
                        method: "iter".to_string(),
                        args: vec![],
                    },
                    body: vec![RustStmt::AugAssign {
                        target: RustExpr::Ident("acc".to_string()),
                        op: "+".to_string(),
                        value: RustExpr::Deref(Box::new(RustExpr::Ident("value".to_string()))),
                    }],
                },
                RustStmt::While {
                    cond: RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("acc".to_string())),
                        op: "<".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Int(10))),
                    },
                    body: vec![RustStmt::Break],
                },
                RustStmt::Loop {
                    body: vec![RustStmt::Continue],
                },
                RustStmt::Match {
                    expr: RustExpr::Ident("acc".to_string()),
                    arms: vec![
                        RustMatchArm {
                            pattern: "0".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Return(None)],
                        },
                        RustMatchArm {
                            pattern: "_".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: vec![RustStmt::Expr(RustExpr::MacroCall {
                                name: "println".to_string(),
                                args: vec![RustExpr::Literal(RustLiteral::Str(
                                    "non-zero".to_string(),
                                ))],
                            })],
                        },
                    ],
                },
            ],
            is_async: false,
        };

        let rendered = render_items(&[item]);
        assert_snapshot!(rendered, @r###"
        pub fn control(items: Vec<i64>) -> () {
            let mut acc: i64 = 0;
            if acc == 0 {
                println!("empty".to_string());
            }
            for value in items.iter() {
                acc += *value;
            }
            while acc < 10 {
                break;
            }
            loop {
                continue;
            }
            match acc {
                0 => {
                    return;
                },
                _ => {
                    println!("non-zero".to_string());
                },
            }
        }
        "###);
    }

    #[test]
    fn renders_function_type_param_bounds() {
        let rendered = render_items(&[RustItem::Fn {
            name: "identity".to_string(),
            visibility: Visibility::Pub,
            type_params: vec![RustTypeParam {
                name: "T".to_string(),
                bounds: vec!["Clone".to_string(), "std::fmt::Display".to_string()],
            }],
            params: vec![RustParam::Named {
                name: "value".to_string(),
                ty: RustType::Named("T".to_string()),
            }],
            ret: Some(RustType::Named("T".to_string())),
            body: vec![RustStmt::Return(Some(RustExpr::Ident("value".to_string())))],
            is_async: false,
        }]);

        assert_snapshot!(rendered, @r###"
        pub fn identity<T: Clone + std::fmt::Display>(value: T) -> T {
            return value;
        }
        "###);
    }

    #[test]
    fn renders_expression_variants() {
        let expr = RustExpr::FormatMacro {
            name: "format".to_string(),
            format_str: "{}-{}".to_string(),
            args: vec![
                RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("left".to_string())),
                    method: "trim".to_string(),
                    args: vec![],
                },
                RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "x".to_string(),
                        ty: RustType::I64,
                    }],
                    body: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("x".to_string())),
                        op: "+".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                    }),
                    is_move: true,
                },
            ],
        };

        let rendered = render_expr(&expr);
        assert_snapshot!(rendered, @r###"format!("{}-{}", left.trim(), move |x| x + 1)"###);
    }

    #[test]
    fn renders_method_call_on_range_receiver_with_parentheses() {
        let expr = RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Range {
                start: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                end: Box::new(RustExpr::Literal(RustLiteral::Int(10))),
            }),
            method: "step_by".to_string(),
            args: vec![RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(2))),
                ty: RustType::Named("usize".to_string()),
            }],
        };

        let rendered = render_expr(&expr);
        assert_eq!(rendered, "(1..10).step_by(2 as usize)");
    }

    #[test]
    fn renders_parenthesized_expression_node() {
        let expr = RustExpr::Paren(Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("a".to_string())),
            op: "+".to_string(),
            right: Box::new(RustExpr::Ident("b".to_string())),
        }));

        let rendered = render_expr(&expr);
        assert_eq!(rendered, "(a + b)");
    }

    #[test]
    fn renders_slice_expression() {
        let expr = RustExpr::Slice {
            expr: Box::new(RustExpr::Ident("values".to_string())),
            start: Some(Box::new(RustExpr::Literal(RustLiteral::Int(1)))),
            stop: Some(Box::new(RustExpr::Literal(RustLiteral::Int(3)))),
        };

        let rendered = render_expr(&expr);
        assert_eq!(rendered, "values[1..3]");
    }

    #[test]
    fn renders_write_macro_format_arg_as_literal() {
        let expr = RustExpr::MacroCall {
            name: "write".to_string(),
            args: vec![
                RustExpr::Ident("f".to_string()),
                RustExpr::Literal(RustLiteral::Str("{}".to_string())),
                RustExpr::Ident("v".to_string()),
            ],
        };

        let rendered = render_expr(&expr);
        assert_eq!(rendered, "write!(f, \"{}\", v)");
    }

    #[test]
    fn raw_code_passthrough_for_items_stmts_and_exprs() {
        let items = vec![
            RustItem::RawCode("fn passthrough() {\n    println!(\"ok\");\n}".to_string()),
            RustItem::Fn {
                name: "main".to_string(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![],
                ret: None,
                body: vec![RustStmt::RawCode("println!(\"line\");".to_string())],
                is_async: false,
            },
        ];

        let rendered_items = render_items(&items);
        let rendered_expr = render_expr(&RustExpr::RawCode("custom_expr()".to_string()));

        assert_snapshot!(rendered_items, @r###"
        fn passthrough() {
            println!("ok");
        }

        fn main() {
            println!("line");
        }
        "###);
        assert_snapshot!(rendered_expr, @"custom_expr()");
    }

    #[test]
    fn render_items_raw_code_only_returns_input() {
        let raw = "mod generated {\n    pub fn helper() {}\n}\n";
        let rendered = render_items(&[RustItem::RawCode(raw.to_string())]);
        assert_eq!(rendered, raw);
    }

    #[test]
    fn render_stmts_helper_renders_block() {
        let stmts = vec![
            RustStmt::Let {
                mutable: false,
                name: "x".to_string(),
                ty: Some(RustType::I64),
                value: RustExpr::Literal(RustLiteral::Int(1)),
            },
            RustStmt::LetPattern {
                pattern: "(a, b)".to_string(),
                value: RustExpr::Tuple(vec![
                    RustExpr::Literal(RustLiteral::Int(2)),
                    RustExpr::Literal(RustLiteral::Bool(true)),
                ]),
            },
            RustStmt::Expr(RustExpr::Try(Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("may_fail".to_string())),
                args: vec![],
            }))),
        ];
        let rendered = render_stmts(&stmts);
        assert_snapshot!(rendered, @r###"
        let x: i64 = 1;
        let (a, b) = (2, true);
        may_fail()?;
        "###);
    }

    #[test]
    fn render_stmts_renders_assert_variants() {
        let stmts = vec![
            RustStmt::Assert {
                cond: RustExpr::Literal(RustLiteral::Bool(true)),
                msg: None,
            },
            RustStmt::Assert {
                cond: RustExpr::Literal(RustLiteral::Bool(false)),
                msg: Some(RustExpr::Literal(RustLiteral::Str("boom".to_string()))),
            },
        ];
        let rendered = render_stmts(&stmts);
        assert_snapshot!(rendered, @r###"
        assert!(true);
        assert!(false, "{}", "boom".to_string());
        "###);
    }
}
