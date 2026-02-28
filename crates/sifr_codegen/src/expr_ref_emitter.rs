use crate::RustEmitter;
use sifr_hir::HirExpr;
use sifr_type_system::Type;

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

impl RustEmitter {
    fn lower_ref_expr_or_panic(&mut self, expr: &HirExpr, context: &str) -> crate::RustExpr {
        if let Some(lowered) = self.try_lower_registry_expr_strict(expr) {
            return lowered;
        }
        if let HirExpr::Index { object, index, .. } = expr {
            match self.try_lower_structured_index_expr(object, index) {
                Ok(Some(lowered)) => return lowered,
                Ok(None) => {}
                Err(err) => {
                    panic!(
                        "structured expr-ref index lowering error for {context}: {expr:?}: {err:?}"
                    );
                }
            }
        }
        panic!("structured expr-ref lowering missing for {context}: {expr:?}")
    }

    /// Emit an expression wrapped in parentheses.
    ///
    /// This is used before method chaining to avoid Rust precedence bugs such as
    /// `x as f64.round()` where the cast must be grouped first.
    pub(super) fn emit_parenthesized_expr(&mut self, expr: &HirExpr) {
        let lowered = self.lower_ref_expr_or_panic(expr, "parenthesized expression");
        self.write_registry_expr(&crate::RustExpr::Paren(Box::new(lowered)));
    }

    /// Emit an expression as a `HashMap` key reference.
    /// String literals are emitted directly (e.g., `"key"`) since `HashMap::get` accepts &str via Borrow.
    /// Other expressions are emitted with `&` prefix (e.g., `&var`).
    pub(super) fn emit_key_ref_expr(&mut self, expr: &HirExpr) {
        let lowered = if let HirExpr::StringLiteral(val) = expr {
            crate::RustExpr::Literal(crate::RustLiteral::Str(val.clone()))
        } else if let HirExpr::Name { name, ty } = expr {
            // If the name is already a borrowed parameter (&String or &mut String),
            // emitting `&name` would produce `&&String` which fails Borrow<str> bounds.
            // For borrowed string params, emit `name.as_str()` or just `name` (deref coerces).
            if (self.borrowed_params.contains(name.as_str())
                || self.mut_borrowed_params.contains(name.as_str()))
                && matches!(ty, Type::Str)
            {
                // already &String -- deref-coerces to &str via as_str()
                crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                    method: "as_str".to_string(),
                    args: vec![],
                }
            } else if self.borrowed_params.contains(name.as_str())
                || self.mut_borrowed_params.contains(name.as_str())
            {
                // already a reference -- pass directly (no extra &)
                self.lower_ref_expr_or_panic(expr, "key ref borrowed param")
            } else {
                crate::RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(self.lower_ref_expr_or_panic(expr, "key ref")),
                }
            }
        } else {
            crate::RustExpr::Ref {
                mutable: false,
                expr: Box::new(self.lower_ref_expr_or_panic(expr, "key ref")),
            }
        };
        self.write_registry_expr(&lowered);
    }

    /// Emit an expression as a `&str` reference.
    /// String literals are emitted directly (e.g., `"hello"`).
    /// Other string expressions are emitted with `.as_str()` (e.g., `s.as_str()`).
    pub(super) fn emit_str_ref_expr(&mut self, expr: &HirExpr) {
        if let HirExpr::StringLiteral(val) = expr {
            self.write_registry_expr(&crate::RustExpr::Literal(crate::RustLiteral::Str(
                val.clone(),
            )));
        } else {
            let lowered = self.lower_ref_expr_or_panic(expr, "str ref");
            self.write_registry_expr(&crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                method: "as_str".to_string(),
                args: vec![],
            });
        }
    }

    /// Emit an expression as a `&str` for stdlib call sites.
    /// String literals are emitted as bare `"literal"` (no `.to_string()`).
    /// Borrowed parameters are emitted directly (already `&String`, deref-coerces to `&str`).
    /// Other expressions are emitted as `&expr` (borrow the String, deref-coerces to `&str`).
    /// Use this for Rust APIs that accept `&str`, `AsRef<str>`, `AsRef<Path>`, `AsRef<OsStr>`, etc.
    pub(super) fn emit_expr_as_str_ref(&mut self, expr: &HirExpr) {
        let lowered = if let HirExpr::StringLiteral(val) = expr {
            crate::RustExpr::Literal(crate::RustLiteral::Str(val.clone()))
        } else if let HirExpr::Name { name, .. } = expr {
            if self.borrowed_params.contains(name) {
                // Already &String, no extra & needed
                self.lower_ref_expr_or_panic(expr, "str ref borrowed param")
            } else {
                crate::RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(self.lower_ref_expr_or_panic(expr, "str ref")),
                }
            }
        } else {
            crate::RustExpr::Ref {
                mutable: false,
                expr: Box::new(self.lower_ref_expr_or_panic(expr, "str ref")),
            }
        };
        self.write_registry_expr(&lowered);
    }

    /// Emit an expression for use in comparisons, dereferencing borrowed params.
    /// When a function parameter is `&String` (borrow-by-default), comparing it
    /// directly with a `String` fails in Rust (`&String != String`).
    /// This method emits `*name` for borrowed params so the comparison works.
    pub(super) fn emit_expr_for_compare(&mut self, expr: &HirExpr) {
        if let HirExpr::Name { name, ty } = expr {
            if self.borrowed_params.contains(name)
                && (matches!(ty, Type::Str) || matches!(ty, Type::TypeVar(_)))
            {
                let lowered = self.lower_ref_expr_or_panic(expr, "compare deref");
                self.write_registry_expr(&crate::RustExpr::Deref(Box::new(lowered)));
                return;
            }
        }
        let lowered = self.lower_ref_expr_or_panic(expr, "compare expr");
        self.write_registry_expr(&lowered);
    }

    /// Emit an expression for use on the left side of a comparison operator.
    /// `IntLiteral` and other expressions that result in type casts need parentheses
    /// to avoid Rust parsing `1 as i64 < x` as a generic argument.
    pub(super) fn emit_expr_with_parens_for_compare(&mut self, expr: &HirExpr) {
        // Check if emitting this expression will result in a type cast that needs parens
        // This includes IntLiteral (which becomes "N_i64") and FloatLiteral (which becomes "N_f64")
        if matches!(expr, HirExpr::IntLiteral(_) | HirExpr::FloatLiteral(_)) {
            let lowered = self.lower_ref_expr_or_panic(expr, "compare parens");
            self.write_registry_expr(&crate::RustExpr::Paren(Box::new(lowered)));
        } else {
            let lowered = self.lower_ref_expr_or_panic(expr, "compare plain");
            self.write_registry_expr(&lowered);
        }
    }

    /// Emit an expression as bytes for stdlib call sites (hash, encoding).
    /// String literals are emitted as `"literal".as_bytes()` (no `.to_string()`).
    /// Other expressions are emitted as `expr.as_bytes()` (String has `.as_bytes()`).
    pub(super) fn emit_expr_as_bytes(&mut self, expr: &HirExpr) {
        if let HirExpr::StringLiteral(val) = expr {
            self.write_registry_expr(&crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Str(
                    val.clone(),
                ))),
                method: "as_bytes".to_string(),
                args: vec![],
            });
        } else {
            let lowered = self.lower_ref_expr_or_panic(expr, "bytes ref");
            self.write_registry_expr(&crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
                method: "as_bytes".to_string(),
                args: vec![],
            });
        }
    }

    /// Check if an expression is a list literal (`HirExpr::ListLiteral`).
    fn is_list_literal(expr: &HirExpr) -> bool {
        matches!(expr, HirExpr::ListLiteral { .. })
    }

    /// Emit a collection expression for set operations.
    /// List literals are emitted directly (no `.clone()`).
    /// Other expressions are emitted with `.clone()`.
    pub(super) fn emit_collection_expr(&mut self, expr: &HirExpr) {
        let lowered = self.lower_ref_expr_or_panic(expr, "collection expr");
        if Self::is_list_literal(expr) {
            self.write_registry_expr(&lowered);
            return;
        }
        self.write_registry_expr(&crate::RustExpr::MethodCall {
            receiver: Box::new(lowered),
            method: "clone".to_string(),
            args: vec![],
        });
    }

    /// Emit an expression suitable for use inside format!/println! contexts.
    /// Wraps Option<T> expressions so they display as the inner value or "None".
    /// Omits `.to_string()` on string literals since format macros accept &str.
    pub(super) fn emit_display_expr(&mut self, expr: &HirExpr) {
        let inferred_option_inner = if let Some(inner) = display_option_inner_type(expr) {
            Some(inner)
        } else if matches!(
            crate::resolve_alias_type_for_plain_call(expr.ty()),
            Type::Str | Type::LiteralStr(_)
        ) {
            None
        } else {
            let probe = self.render_expr_via_direct_emit(expr);
            if probe.contains(".get(") && probe.contains(").cloned()") {
                Some(Type::Unknown)
            } else if probe.contains(".chars().nth(") {
                Some(Type::Str)
            } else {
                None
            }
        };
        if let Some(inner) = inferred_option_inner {
            let lowered = self.lower_ref_expr_or_panic(expr, "display option expr");
            let format_str = if uses_debug_display_format(&inner) {
                "{:?}".to_string()
            } else {
                "{}".to_string()
            };
            let lowered = crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered))),
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
                            name: "_v".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(crate::RustExpr::FormatMacro {
                            name: "format".to_string(),
                            format_str,
                            args: vec![crate::RustExpr::Ident("_v".to_string())],
                        }),
                        is_move: false,
                    },
                ],
            };
            self.write_registry_expr(&lowered);
        } else if let HirExpr::StringLiteral(val) = expr {
            // In display contexts, string literals don't need .to_string()
            self.write_registry_expr(&crate::RustExpr::Literal(crate::RustLiteral::Str(
                val.clone(),
            )));
        } else if uses_debug_display_format(expr.ty()) {
            // Collections use Debug-style formatting in display contexts.
            let lowered = self.lower_ref_expr_or_panic(expr, "display debug expr");
            self.write_registry_expr(&crate::RustExpr::FormatMacro {
                name: "format".to_string(),
                format_str: "{:?}".to_string(),
                args: vec![lowered],
            });
        } else {
            let lowered = self.lower_ref_expr_or_panic(expr, "display expr");
            self.write_registry_expr(&lowered);
        }
    }
}
