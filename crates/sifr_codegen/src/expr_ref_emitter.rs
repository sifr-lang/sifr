use crate::RustEmitter;
use sifr_ir::HirExpr;
use sifr_type_system::Type;

fn uses_debug_display_format(ty: &Type) -> bool {
    match crate::resolve_alias_type_for_plain_call(ty) {
        Type::Int
        | Type::FixedInt(_)
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
        | Type::BigInt
        | Type::Decimal
        | Type::BigDecimal => false,
        Type::List(_)
        | Type::Bytes
        | Type::Dict(_, _)
        | Type::Set(_)
        | Type::Tuple(_)
        | Type::Iterable(_)
        | Type::Iterator(_)
        | Type::Function(_)
        | Type::AsyncFunction(_)
        | Type::Coroutine(_, _)
        | Type::Task(_, _)
        | Type::TaskResult(_, _)
        | Type::Failure(_)
        | Type::TimeoutResult(_)
        | Type::Select2(_, _)
        | Type::BlockingTask(_, _)
        | Type::JoinSet(_, _)
        | Type::Awaitable(_)
        | Type::AsyncIterator(_, _)
        | Type::AsyncGenerator(_, _)
        | Type::PythonBuffer(_)
        | Type::PythonArrow(_)
        | Type::PythonDlpackTensor(_)
        | Type::PythonDlpackStream
        | Type::Callable(..)
        | Type::AsyncCallable(..)
        | Type::Result(_, _)
        | Type::Protocol { .. }
        | Type::Any
        | Type::Unknown
        | Type::Intersection(_)
        | Type::Never => true,
        Type::Alias { body, .. } => uses_debug_display_format(body),
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
            Type::Bytes => return Some(Type::FixedInt(sifr_type_system::FixedIntType::U8)),
            Type::Dict(_, value) => return Some((**value).clone()),
            Type::Str => return Some(Type::Str),
            _ => {}
        }
    }
    option_inner_from_rust_type(expr.ty())
}

impl RustEmitter {
    pub(crate) fn lower_ref_expr_or_panic(
        &mut self,
        expr: &HirExpr,
        context: &str,
    ) -> crate::RustExpr {
        if let Some(lowered) = self.try_lower_registry_expr_strict(expr) {
            return lowered;
        }
        if let HirExpr::Index {
            object, index, ty, ..
        } = expr
        {
            match self.try_lower_structured_index_expr(object, index, ty) {
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

    /// Emit an expression suitable for use inside format!/println! contexts.
    /// Wraps Option<T> expressions so they display as the inner value or "None".
    /// Omits `.to_string()` on string literals since format macros accept &str.
    pub(crate) fn lower_display_expr(&mut self, expr: &HirExpr) -> crate::RustExpr {
        if matches!(
            crate::resolve_alias_type_for_plain_call(expr.ty()),
            Type::None
        ) {
            return crate::RustExpr::Literal(crate::RustLiteral::Str("None".to_string()));
        }
        let inferred_option_inner = display_option_inner_type(expr);
        if let Some(inner) = inferred_option_inner {
            let lowered = self.lower_ref_expr_or_panic(expr, "display option expr");
            let format_str = if uses_debug_display_format(&inner) {
                "{:?}".to_string()
            } else {
                "{}".to_string()
            };
            crate::RustExpr::MethodCall {
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
            }
        } else if let HirExpr::StringLiteral(val) = expr {
            // In display contexts, string literals don't need .to_string()
            crate::RustExpr::Literal(crate::RustLiteral::Str(val.clone()))
        } else if uses_debug_display_format(expr.ty()) {
            // Collections use Debug-style formatting in display contexts.
            let lowered = self.lower_ref_expr_or_panic(expr, "display debug expr");
            crate::RustExpr::FormatMacro {
                name: "format".to_string(),
                format_str: "{:?}".to_string(),
                args: vec![lowered],
            }
        } else {
            self.lower_ref_expr_or_panic(expr, "display expr")
        }
    }
}
