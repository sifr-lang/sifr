use super::*;
impl Renderer {
    pub fn render_expr(&mut self, expr: &RustExpr) {
        self.append(&Self::render_expr_string(expr));
    }

    pub fn render_type(&mut self, ty: &RustType) {
        self.append(&Self::render_type_string(ty));
    }

    pub(crate) fn append(&mut self, s: &str) {
        let _ = write!(self.output, "{s}");
    }

    pub(crate) fn emit_line(&mut self, s: &str) {
        self.write_indent();
        self.append(s);
        let _ = self.output.write_char('\n');
    }

    pub(crate) fn write_indent(&mut self) {
        for _ in 0..self.indent {
            let _ = write!(self.output, "    ");
        }
    }

    pub(crate) fn indent(&mut self) {
        self.indent += 1;
    }

    pub(crate) fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    pub(crate) fn render_visibility(visibility: &Visibility) -> &'static str {
        match visibility {
            Visibility::Private => "",
            Visibility::Pub => "pub ",
        }
    }

    pub(crate) fn render_derives(&mut self, derives: &[String]) {
        if !derives.is_empty() {
            self.emit_line(&format!("#[derive({})]", derives.join(", ")));
        }
    }

    pub(crate) fn render_param_string(param: &RustParam) -> String {
        match param {
            RustParam::SelfParam { mutable } => {
                if *mutable {
                    "&mut self".to_string()
                } else {
                    "&self".to_string()
                }
            }
            RustParam::SelfParamWithLifetime { mutable, lifetime } => {
                if *mutable {
                    format!("&{lifetime} mut self")
                } else {
                    format!("&{lifetime} self")
                }
            }
            RustParam::SelfValue => "self".to_string(),
            RustParam::MutableSelfValue => "mut self".to_string(),
            RustParam::Named { name, ty } => {
                format!(
                    "{}: {}",
                    Self::render_identifier(name),
                    Self::render_type_string(ty)
                )
            }
            RustParam::NamedMut { name, ty } => {
                format!(
                    "mut {}: {}",
                    Self::render_identifier(name),
                    Self::render_type_string(ty)
                )
            }
        }
    }

    pub(crate) fn render_type_string(ty: &RustType) -> String {
        match ty {
            RustType::I64 => "i64".to_string(),
            RustType::F64 => "f64".to_string(),
            RustType::Bool => "bool".to_string(),
            RustType::String_ => "String".to_string(),
            RustType::Str => "str".to_string(),
            RustType::Unit => "()".to_string(),
            RustType::Vec(inner) => format!("Vec<{}>", Self::render_type_string(inner)),
            RustType::Slice(inner) => format!("[{}]", Self::render_type_string(inner)),
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
            RustType::Array { element, len } => {
                format!("[{}; {len}]", Self::render_type_string(element))
            }
            RustType::Boxed(inner) => format!("Box<{}>", Self::render_type_string(inner)),
            RustType::Never => "!".to_string(),
            RustType::Ref { mutable, inner } => {
                if *mutable {
                    format!("&mut {}", Self::render_type_string(inner))
                } else {
                    format!("&{}", Self::render_type_string(inner))
                }
            }
            RustType::Named(name) => Self::render_compiler_path_string(name),
            RustType::Generic { base, params } => format!(
                "{}<{}>",
                Self::render_compiler_path_string(base),
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
            RustType::DynTrait {
                trait_,
                auto_traits,
            } => Self::render_trait_type("dyn", trait_, auto_traits),
            RustType::ImplTrait {
                trait_,
                auto_traits,
            } => Self::render_trait_type("impl", trait_, auto_traits),
        }
    }

    fn render_trait_type(
        prefix: &str,
        trait_: &crate::RustTrait,
        auto_traits: &[String],
    ) -> String {
        let base = match trait_ {
            crate::RustTrait::Named {
                name,
                params,
                associated_types,
            } => {
                let mut arguments = params
                    .iter()
                    .map(Self::render_type_string)
                    .collect::<Vec<_>>();
                arguments.extend(
                    associated_types
                        .iter()
                        .map(|(name, ty)| format!("{name} = {}", Self::render_type_string(ty))),
                );
                let name = Self::render_compiler_path_string(name);
                if arguments.is_empty() {
                    name
                } else {
                    format!("{name}<{}>", arguments.join(", "))
                }
            }
            crate::RustTrait::Callable { name, params, ret } => {
                let params = params
                    .iter()
                    .map(Self::render_type_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                let name = Self::render_compiler_path_string(name);
                match ret {
                    Some(ret) => format!("{name}({params}) -> {}", Self::render_type_string(ret)),
                    None => format!("{name}({params})"),
                }
            }
        };
        let suffix = auto_traits.iter().fold(String::new(), |mut suffix, bound| {
            let _ = write!(suffix, " + {bound}");
            suffix
        });
        format!("{prefix} {base}{suffix}")
    }

    pub(crate) fn render_expr_string(expr: &RustExpr) -> String {
        match expr {
            RustExpr::Literal(lit) => Self::render_literal(lit),
            RustExpr::Verbatim(source) => Self::render_compiler_path_string(source),
            RustExpr::Ident(name) => Self::render_identifier_or_compiler_path(name),
            RustExpr::Path(parts) => Self::render_path_parts(parts),
            RustExpr::MethodCall {
                receiver,
                method,
                args,
            } => {
                let receiver =
                    if matches!(receiver.as_ref(), RustExpr::Ref { .. } | RustExpr::Deref(_)) {
                        format!("({})", Self::render_expr_string(receiver))
                    } else {
                        Self::wrap_expr(receiver)
                    };
                format!(
                    "{receiver}.{}({})",
                    Self::render_identifier(method),
                    args.iter()
                        .map(Self::render_expr_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            RustExpr::FnCall { func, args } => format!(
                "{}({})",
                Self::wrap_expr(func),
                args.iter()
                    .map(Self::render_expr_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            RustExpr::MacroCall { name, args } => format!(
                "{}!({})",
                Self::render_compiler_path_string(name),
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
                if args.is_empty()
                    && format_str.is_empty()
                    && matches!(name.as_str(), "println" | "eprintln")
                {
                    return format!("{name}!()");
                }
                let escaped = format!("\"{}\"", format_str.escape_default());
                if args.is_empty() {
                    format!("{name}!({escaped})")
                } else {
                    format!(
                        "{name}!({escaped}, {})",
                        args.iter()
                            .map(Self::render_format_arg_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            RustExpr::BinOp { left, op, right } => {
                let left = Self::render_comparison_operand(op, left);
                let right = Self::render_comparison_operand(op, right);
                format!("{left} {op} {right}")
            }
            RustExpr::UnaryOp { op, operand } => format!("{op}{}", Self::wrap_expr(operand)),
            RustExpr::Field { expr, field } => {
                format!(
                    "{}.{}",
                    Self::wrap_expr(expr),
                    Self::render_identifier(field)
                )
            }
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
                if let Some(literal) = Self::render_typed_numeric_literal(expr, ty) {
                    return literal;
                }
                if matches!(ty, RustType::Named(name) if name == "usize") {
                    return format!(
                        "::sifr_runtime::to_usize_proven(&({}))",
                        Self::render_expr_string(expr)
                    );
                }
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
                    .map(Self::render_untyped_closure_param_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{move_kw}|{params}| {}", Self::render_expr_string(body))
            }
            RustExpr::ClosureBlock {
                params,
                body,
                is_move,
                is_async,
            } => {
                let move_kw = if *is_move { "move " } else { "" };
                let async_kw = if *is_async { "async " } else { "" };
                let params = params
                    .iter()
                    .map(Self::render_closure_param_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut renderer = Renderer::new();
                renderer.append(&format!("{async_kw}{move_kw}|{params}| {{\n"));
                renderer.indent();
                renderer.render_body(body);
                renderer.dedent();
                renderer.append("}");
                renderer.output
            }
            RustExpr::AsyncBlock { body, is_move } => {
                let move_kw = if *is_move { "move " } else { "" };
                format!("async {move_kw}{}", Self::render_block_expr(body, None))
            }
            RustExpr::StructInit { name, fields } => format!(
                "{} {{ {} }}",
                Self::render_compiler_path_string(name),
                fields
                    .iter()
                    .map(|(field, value)| Self::render_struct_field_init(field, value))
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
            RustExpr::Array(values) => format!(
                "[{}]",
                values
                    .iter()
                    .map(Self::render_expr_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            RustExpr::Vec(values) => format!(
                "vec![{}]",
                values
                    .iter()
                    .map(Self::render_expr_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            RustExpr::TimeoutAwait {
                duration,
                future,
                error,
            } => format!(
                "match ::tokio::time::timeout({}, {}).await {{ Ok(__sifr_timeout_value) => __sifr_timeout_value, Err(_) => return Err({}) }}",
                Self::render_expr_string(duration),
                Self::render_expr_string(future),
                Self::render_expr_string(error)
            ),
            RustExpr::Try(expr) => format!("{}?", Self::wrap_expr(expr)),
            RustExpr::Await(expr) => format!("{}.await", Self::wrap_expr(expr)),
            RustExpr::Paren(expr) => format!("({})", Self::render_expr_string(expr)),
            RustExpr::Range { start, end } => format!(
                "{}..{}",
                Self::render_expr_string(start),
                Self::render_expr_string(end)
            ),
        }
    }
}

impl Renderer {
    pub(crate) fn render_literal(lit: &RustLiteral) -> String {
        match lit {
            RustLiteral::Int(v) => v.to_string(),
            RustLiteral::Float(v) => {
                if v.is_nan() {
                    "f64::NAN".to_string()
                } else if v.is_infinite() {
                    if v.is_sign_negative() {
                        "f64::NEG_INFINITY".to_string()
                    } else {
                        "f64::INFINITY".to_string()
                    }
                } else if v.fract() == 0.0 {
                    format!("{v:.1}")
                } else {
                    v.to_string()
                }
            }
            RustLiteral::Bool(v) => v.to_string(),
            RustLiteral::StaticStr(v) => format!("\"{}\"", v.escape_default()),
            RustLiteral::Str(v) => format!("\"{}\".to_string()", v.escape_default()),
            RustLiteral::Char(v) => format!("'{}'", v.escape_default()),
            RustLiteral::Unit => "()".to_string(),
            RustLiteral::None => "None".to_string(),
        }
    }

    pub(crate) fn render_macro_arg(name: &str, idx: usize, arg: &RustExpr) -> String {
        // `write!` / `writeln!` require the format string as a literal token
        // (second argument after the destination writer), not a `String`.
        if matches!(name, "write" | "writeln") && idx == 1 {
            if let RustExpr::Literal(RustLiteral::Str(value)) = arg {
                return format!("\"{}\"", value.escape_default());
            }
        }
        // `assert!`/`assert_eq!`/`assert_ne!` message format arguments must be
        // literal tokens, not owned `String` expressions.
        if matches!(
            name,
            "assert" | "assert_eq" | "assert_ne" | "unreachable" | "compile_error"
        ) {
            if let RustExpr::Literal(RustLiteral::Str(value)) = arg {
                return format!("\"{}\"", value.escape_default());
            }
        }
        Self::render_expr_string(arg)
    }

    pub(crate) fn wrap_expr(expr: &RustExpr) -> String {
        if Self::expr_requires_parens(expr) {
            format!("({})", Self::render_expr_string(expr))
        } else {
            Self::render_expr_string(expr)
        }
    }

    pub(crate) fn render_assign_op<'a>(
        target: &RustExpr,
        value: &'a RustExpr,
    ) -> Option<(&'a str, &'a RustExpr)> {
        let RustExpr::BinOp { left, op, right } = value else {
            return None;
        };
        if !matches!(op.as_str(), "+" | "-" | "*" | "/" | "%") || left.as_ref() != target {
            return None;
        }
        Some((op.as_str(), right.as_ref()))
    }

    pub(crate) fn render_comparison_operand(op: &str, expr: &RustExpr) -> String {
        if matches!(op, "==" | "!=") {
            if let RustExpr::Literal(RustLiteral::Str(value)) = expr {
                return format!("\"{}\"", value.escape_default());
            }
        }
        Self::wrap_expr(expr)
    }

    pub(crate) fn render_struct_field_init(field: &str, value: &RustExpr) -> String {
        let rendered_field = Self::render_identifier(field);
        if let RustExpr::Ident(name) = value {
            if name == field {
                return rendered_field;
            }
        }
        format!("{rendered_field}: {}", Self::render_expr_string(value))
    }

    pub(crate) fn render_format_arg_string(expr: &RustExpr) -> String {
        if let RustExpr::Literal(RustLiteral::Str(value)) = expr {
            return format!("\"{}\"", value.escape_default());
        }
        Self::render_expr_string(expr)
    }

    pub(crate) fn render_typed_numeric_literal(expr: &RustExpr, ty: &RustType) -> Option<String> {
        let suffix = Self::numeric_literal_suffix(ty)?;
        match expr {
            RustExpr::Literal(RustLiteral::Int(value)) => Some(format!("{value}_{suffix}")),
            RustExpr::Literal(RustLiteral::Float(value)) if value.is_finite() => {
                let rendered = if value.fract() == 0.0 {
                    format!("{value:.1}")
                } else {
                    value.to_string()
                };
                Some(format!("{rendered}_{suffix}"))
            }
            _ => None,
        }
    }

    pub(crate) fn numeric_literal_suffix(ty: &RustType) -> Option<&str> {
        match ty {
            RustType::I64 => Some("i64"),
            RustType::F64 => Some("f64"),
            RustType::Named(name)
                if matches!(
                    name.as_str(),
                    "i8" | "i16"
                        | "i32"
                        | "i64"
                        | "i128"
                        | "isize"
                        | "u8"
                        | "u16"
                        | "u32"
                        | "u64"
                        | "u128"
                        | "usize"
                        | "f32"
                        | "f64"
                ) =>
            {
                Some(name.as_str())
            }
            _ => None,
        }
    }

    pub(crate) fn expr_requires_parens(expr: &RustExpr) -> bool {
        // Check if expr is one of the types that always needs parens
        if matches!(
            expr,
            RustExpr::BinOp { .. }
                | RustExpr::Cast { .. }
                | RustExpr::If { .. }
                | RustExpr::Match { .. }
                | RustExpr::Closure { .. }
                | RustExpr::ClosureBlock { .. }
                | RustExpr::AsyncBlock { .. }
                | RustExpr::Block { .. }
                | RustExpr::UnaryOp { .. }
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

    pub(crate) fn render_closure_param_string(param: &RustParam) -> String {
        match param {
            RustParam::SelfParam { .. }
            | RustParam::SelfParamWithLifetime { .. }
            | RustParam::SelfValue
            | RustParam::MutableSelfValue => "self".to_string(),
            RustParam::Named { name, ty } | RustParam::NamedMut { name, ty } => {
                let rendered_name = Self::render_identifier(name);
                if matches!(ty, RustType::Named(name) if name == "_") {
                    rendered_name
                } else {
                    format!("{rendered_name}: {}", Self::render_type_string(ty))
                }
            }
        }
    }

    pub(crate) fn render_untyped_closure_param_string(param: &RustParam) -> String {
        match param {
            RustParam::SelfParam { .. }
            | RustParam::SelfParamWithLifetime { .. }
            | RustParam::SelfValue
            | RustParam::MutableSelfValue => "self".to_string(),
            RustParam::Named { name, .. } | RustParam::NamedMut { name, .. } => {
                Self::render_identifier(name)
            }
        }
    }

    pub(crate) fn render_pattern_string(pattern: &str) -> String {
        let mut out = String::new();
        let mut token = String::new();
        let flush_token = |out: &mut String, token: &mut String| {
            if token.is_empty() {
                return;
            }
            if matches!(token.as_str(), "mut" | "ref" | "true" | "false") {
                out.push_str(token);
            } else {
                out.push_str(&Self::render_identifier(token));
            }
            token.clear();
        };

        for ch in pattern.chars() {
            if ch == '_' || ch.is_ascii_alphanumeric() {
                token.push(ch);
            } else {
                flush_token(&mut out, &mut token);
                out.push(ch);
            }
        }
        flush_token(&mut out, &mut token);
        Self::render_compiler_path_string(&out)
    }

    pub(crate) fn render_identifier(name: &str) -> String {
        if name.starts_with("r#") || !Self::is_plain_ascii_identifier(name) {
            return name.to_string();
        }
        // Rust does not permit `r#true` or `r#false`. Boolean values are
        // represented by `RustLiteral`, so an identifier with either spelling
        // is necessarily a source binding and must use a disjoint encoding.
        if let Some(encoded) = Self::reserved_literal_identifier(name) {
            return encoded.to_string();
        }
        if Self::is_escape_required_keyword(name) {
            format!("r#{name}")
        } else {
            name.to_string()
        }
    }

    fn reserved_literal_identifier(name: &str) -> Option<&'static str> {
        match name {
            "true" => Some("sifr_source_74727565"),
            "false" => Some("sifr_source_66616c7365"),
            _ => None,
        }
    }

    pub(crate) fn is_plain_ascii_identifier(name: &str) -> bool {
        let mut chars = name.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        if !(first == '_' || first.is_ascii_alphabetic()) {
            return false;
        }
        chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
    }

    pub(crate) fn is_escape_required_keyword(name: &str) -> bool {
        matches!(
            name,
            "as" | "break"
                | "const"
                | "continue"
                | "else"
                | "enum"
                | "extern"
                | "false"
                | "fn"
                | "for"
                | "if"
                | "impl"
                | "in"
                | "let"
                | "loop"
                | "match"
                | "mod"
                | "move"
                | "mut"
                | "pub"
                | "ref"
                | "return"
                | "static"
                | "struct"
                | "trait"
                | "true"
                | "type"
                | "unsafe"
                | "use"
                | "where"
                | "while"
                | "async"
                | "await"
                | "dyn"
                | "union"
                | "abstract"
                | "become"
                | "box"
                | "do"
                | "final"
                | "macro"
                | "override"
                | "priv"
                | "try"
                | "typeof"
                | "unsized"
                | "virtual"
                | "yield"
                | "gen"
        )
    }

    pub(crate) fn render_block_expr(
        stmts: &[RustStmt],
        trailing_expr: Option<&RustExpr>,
    ) -> String {
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

    pub(crate) fn render_match_arms(&mut self, arms: &[RustMatchArm]) {
        for arm in arms {
            let guard = arm
                .guard
                .as_ref()
                .map(|g| format!(" if {}", Self::render_condition_expr_string(g)))
                .unwrap_or_default();
            self.emit_line(&format!(
                "{}{} => {{",
                Self::render_pattern_string(&arm.pattern),
                guard
            ));
            self.indent();
            for stmt in &arm.body {
                self.render_stmt(stmt);
            }
            self.dedent();
            self.emit_line("},");
        }
    }

    pub(crate) fn render_condition_expr_string(expr: &RustExpr) -> String {
        if let RustExpr::Paren(inner) = expr {
            return Self::render_condition_expr_string(inner);
        }
        Self::render_expr_string(expr)
    }
}
