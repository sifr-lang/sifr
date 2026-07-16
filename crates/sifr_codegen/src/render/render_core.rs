use super::*;
use crate::{RustFile, RustItem, RustStmt};

pub struct Renderer {
    pub(crate) output: String,
    pub(crate) indent: usize,
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
                self.emit_line(&format!(
                    "use {} as {};",
                    path.join("::"),
                    Self::render_identifier(alias)
                ));
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
                    Self::render_identifier(name)
                ));
                self.indent();
                for (field_name, field_ty) in fields {
                    self.emit_line(&format!(
                        "{}: {},",
                        Self::render_identifier(field_name),
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
                    Self::render_identifier(name),
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
                    Self::render_identifier(name)
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
                        format!("{}({})", Self::render_identifier(&variant.name), fields)
                    } else if !variant.fields.is_empty() {
                        let fields = variant
                            .fields
                            .iter()
                            .map(|(f, t)| {
                                format!(
                                    "{}: {}",
                                    Self::render_identifier(f),
                                    Self::render_type_string(t)
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!(
                            "{} {{ {} }}",
                            Self::render_identifier(&variant.name),
                            fields
                        )
                    } else if let Some(value) = &variant.value {
                        format!(
                            "{} = {}",
                            Self::render_identifier(&variant.name),
                            Self::render_expr_string(value)
                        )
                    } else {
                        Self::render_identifier(&variant.name)
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
                    Self::render_identifier(name),
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
                    format!(
                        "impl{generics} {trait_name} for {} {{",
                        Self::render_identifier(target)
                    )
                } else {
                    format!("impl{generics} {} {{", Self::render_identifier(target))
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
                    Self::render_identifier(name),
                    type_params,
                    params,
                    ret
                ));
                self.indent();
                self.render_body(body);
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
                self.emit_line(&format!(
                    "fn {}({params}){ret};",
                    Self::render_identifier(name)
                ));
            }
            RustItem::TypeAlias { name, ty } => {
                self.emit_line(&format!(
                    "type {} = {};",
                    Self::render_identifier(name),
                    Self::render_type_string(ty)
                ));
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
                    Self::render_identifier(name),
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
                    Self::render_identifier(name),
                    Self::render_type_string(ty),
                    Self::render_expr_string(value)
                ));
            }
            RustItem::Attr(attr) => self.emit_line(attr),
        }
    }
}

impl Renderer {
    pub fn render_stmt(&mut self, stmt: &RustStmt) {
        self.render_stmt_with_tail(stmt, false);
    }

    pub(crate) fn render_body(&mut self, body: &[RustStmt]) {
        let last_idx = body.len().saturating_sub(1);
        for (idx, stmt) in body.iter().enumerate() {
            self.render_stmt_with_tail(stmt, idx == last_idx);
        }
    }

    fn let_type_should_be_omitted(ty: Option<&RustType>, value: &RustExpr) -> bool {
        let Some(RustType::Vec(_)) = ty else {
            return false;
        };
        matches!(
            value,
            RustExpr::MethodCall { method, args, receiver }
                if method == "unwrap_or"
                    && matches!(args.as_slice(), [RustExpr::Ident(default)] if default == "&[]")
                    && matches!(receiver.as_ref(), RustExpr::MethodCall { method, args, .. }
                        if method == "map"
                            && matches!(args.as_slice(), [RustExpr::Path(path)] if path == &["Vec".to_string(), "as_slice".to_string()]))
        )
    }

    pub(crate) fn render_stmt_with_tail(&mut self, stmt: &RustStmt, tail: bool) {
        match stmt {
            RustStmt::Let {
                mutable,
                name,
                ty,
                value,
            } => {
                let mutability = if *mutable { "mut " } else { "" };
                let rendered_ty = if Self::let_type_should_be_omitted(ty.as_ref(), value) {
                    String::new()
                } else {
                    ty.as_ref()
                        .map(|t| format!(": {}", Self::render_type_string(t)))
                        .unwrap_or_default()
                };
                self.emit_line(&format!(
                    "let {mutability}{name}{rendered_ty} = {value};",
                    name = Self::render_identifier(name),
                    value = Self::render_expr_string(value)
                ));
            }
            RustStmt::LetPattern { pattern, value } => {
                self.emit_line(&format!(
                    "let {pattern} = {value};",
                    pattern = Self::render_pattern_string(pattern),
                    value = Self::render_expr_string(value)
                ));
            }
            RustStmt::LetElse {
                pattern,
                value,
                else_body,
            } => {
                self.emit_line(&format!(
                    "let {pattern} = {} else {{",
                    Self::wrap_expr(value)
                ));
                self.indent();
                for stmt in else_body {
                    self.render_stmt(stmt);
                }
                self.dedent();
                self.emit_line("};");
            }
            RustStmt::Assign { target, value } => {
                if let Some((op, rhs)) = Self::render_assign_op(target, value) {
                    self.emit_line(&format!(
                        "{} {op}= {};",
                        Self::render_expr_string(target),
                        Self::render_expr_string(rhs)
                    ));
                    return;
                }
                self.emit_line(&format!(
                    "{} = {};",
                    Self::render_expr_string(target),
                    Self::render_expr_string(value)
                ));
            }
            RustStmt::AugAssign { target, op, value } => {
                let rendered_op = if op.ends_with('=') {
                    op.clone()
                } else {
                    format!("{op}=")
                };
                self.emit_line(&format!(
                    "{} {} {};",
                    Self::render_expr_string(target),
                    rendered_op,
                    Self::render_expr_string(value)
                ));
            }
            RustStmt::Expr(expr) => self.emit_line(&format!("{};", Self::render_expr_string(expr))),
            RustStmt::TailExpr(expr) => self.emit_line(&Self::render_expr_string(expr)),
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
                if tail {
                    self.emit_line(&Self::render_expr_string(expr));
                    return;
                }
                self.emit_line(&format!("return {};", Self::render_expr_string(expr)));
            }
            RustStmt::Return(None) => {
                if !tail {
                    self.emit_line("return;");
                }
            }
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
                    "for {} in {} {{",
                    Self::render_identifier(var),
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
                        let mutable = if item.mutable { "mut " } else { "" };
                        self.emit_line(&format!(
                            "let {mutable}{} = {guard_var}.ctx.__enter__();",
                            item.binding
                        ));
                    } else {
                        let mutable = if item.mutable { "mut " } else { "" };
                        self.emit_line(&format!("let {mutable}{} = {value};", item.binding));
                    }
                }
                self.render_body(body);
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
                is_async,
            } => {
                let rendered_params = params
                    .iter()
                    .map(Self::render_param_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                if let Some(ret) = ret {
                    self.emit_line(&format!(
                        "{}fn {}({rendered_params}) -> {} {{",
                        if *is_async { "async " } else { "" },
                        Self::render_identifier(name),
                        Self::render_type_string(ret)
                    ));
                } else {
                    self.emit_line(&format!(
                        "{}fn {}({rendered_params}) {{",
                        if *is_async { "async " } else { "" },
                        Self::render_identifier(name)
                    ));
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
        }
    }
}
