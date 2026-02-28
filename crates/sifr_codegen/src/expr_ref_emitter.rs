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

impl RustEmitter {
    /// Emit an expression wrapped in parentheses.
    ///
    /// This is used before method chaining to avoid Rust precedence bugs such as
    /// `x as f64.round()` where the cast must be grouped first.
    pub(super) fn emit_parenthesized_expr(&mut self, expr: &HirExpr) {
        self.write("(");
        self.emit_expr(expr);
        self.write(")");
    }

    /// Emit an expression as a `HashMap` key reference.
    /// String literals are emitted directly (e.g., `"key"`) since `HashMap::get` accepts &str via Borrow.
    /// Other expressions are emitted with `&` prefix (e.g., `&var`).
    pub(super) fn emit_key_ref_expr(&mut self, expr: &HirExpr) {
        if let HirExpr::StringLiteral(val) = expr {
            self.write(&format!("{val:?}"));
        } else if let HirExpr::Name { name, ty } = expr {
            // If the name is already a borrowed parameter (&String or &mut String),
            // emitting `&name` would produce `&&String` which fails Borrow<str> bounds.
            // For borrowed string params, emit `name.as_str()` or just `name` (deref coerces).
            if (self.borrowed_params.contains(name.as_str())
                || self.mut_borrowed_params.contains(name.as_str()))
                && matches!(ty, Type::Str)
            {
                // already &String -- deref-coerces to &str via as_str()
                self.write(name);
                self.write(".as_str()");
            } else if self.borrowed_params.contains(name.as_str())
                || self.mut_borrowed_params.contains(name.as_str())
            {
                // already a reference -- pass directly (no extra &)
                self.emit_expr(expr);
            } else {
                self.write("&");
                self.emit_expr(expr);
            }
        } else {
            self.write("&");
            self.emit_expr(expr);
        }
    }

    /// Emit an expression as a `&str` reference.
    /// String literals are emitted directly (e.g., `"hello"`).
    /// Other string expressions are emitted with `.as_str()` (e.g., `s.as_str()`).
    pub(super) fn emit_str_ref_expr(&mut self, expr: &HirExpr) {
        if let HirExpr::StringLiteral(val) = expr {
            self.write(&format!("{val:?}"));
        } else {
            self.emit_parenthesized_expr(expr);
            self.write(".as_str()");
        }
    }

    /// Emit an expression as a `&str` for stdlib call sites.
    /// String literals are emitted as bare `"literal"` (no `.to_string()`).
    /// Borrowed parameters are emitted directly (already `&String`, deref-coerces to `&str`).
    /// Other expressions are emitted as `&expr` (borrow the String, deref-coerces to `&str`).
    /// Use this for Rust APIs that accept `&str`, `AsRef<str>`, `AsRef<Path>`, `AsRef<OsStr>`, etc.
    pub(super) fn emit_expr_as_str_ref(&mut self, expr: &HirExpr) {
        if let HirExpr::StringLiteral(val) = expr {
            self.write(&format!("{val:?}"));
        } else if let HirExpr::Name { name, .. } = expr {
            if self.borrowed_params.contains(name) {
                // Already &String, no extra & needed
                self.emit_expr(expr);
            } else {
                self.write("&");
                self.emit_expr(expr);
            }
        } else {
            self.write("&");
            self.emit_expr(expr);
        }
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
                self.write("*");
                self.emit_expr(expr);
                return;
            }
        }
        self.emit_expr(expr);
    }

    /// Emit an expression for use on the left side of a comparison operator.
    /// `IntLiteral` and other expressions that result in type casts need parentheses
    /// to avoid Rust parsing `1 as i64 < x` as a generic argument.
    pub(super) fn emit_expr_with_parens_for_compare(&mut self, expr: &HirExpr) {
        // Check if emitting this expression will result in a type cast that needs parens
        // This includes IntLiteral (which becomes "N_i64") and FloatLiteral (which becomes "N_f64")
        if matches!(expr, HirExpr::IntLiteral(_) | HirExpr::FloatLiteral(_)) {
            self.write("(");
            self.emit_expr(expr);
            self.write(")");
        } else {
            self.emit_expr(expr);
        }
    }

    /// Emit an expression as bytes for stdlib call sites (hash, encoding).
    /// String literals are emitted as `"literal".as_bytes()` (no `.to_string()`).
    /// Other expressions are emitted as `expr.as_bytes()` (String has `.as_bytes()`).
    pub(super) fn emit_expr_as_bytes(&mut self, expr: &HirExpr) {
        if let HirExpr::StringLiteral(val) = expr {
            self.write(&format!("{val:?}.as_bytes()"));
        } else {
            self.emit_parenthesized_expr(expr);
            self.write(".as_bytes()");
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
        self.emit_expr(expr);
        if !Self::is_list_literal(expr) {
            self.write(".clone()");
        }
    }

    /// Emit an expression suitable for use inside format!/println! contexts.
    /// Wraps Option<T> expressions so they display as the inner value or "None".
    /// Omits `.to_string()` on string literals since format macros accept &str.
    pub(super) fn emit_display_expr(&mut self, expr: &HirExpr) {
        if let Some(inner) = option_inner_type(expr.ty()) {
            // Wrap: expr.map_or("None".to_string(), |_v| format!("{}", _v))
            self.write("(");
            self.emit_expr(expr);
            self.write(").map_or(\"None\".to_string(), |_v| ");
            if uses_debug_display_format(inner) {
                self.write("format!(\"{:?}\", _v))");
            } else {
                self.write("format!(\"{}\", _v))");
            }
        } else if let HirExpr::StringLiteral(val) = expr {
            // In display contexts, string literals don't need .to_string()
            self.write(&format!("{val:?}"));
        } else if uses_debug_display_format(expr.ty()) {
            // Collections use Debug-style formatting in display contexts.
            self.write("format!(\"{:?}\", ");
            self.emit_expr(expr);
            self.write(")");
        } else {
            self.emit_expr(expr);
        }
    }
}
