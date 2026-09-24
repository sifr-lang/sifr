// Exact source float semantics; Clippy alternatives can change observable bits.

impl Rewriter<'_> {
    fn requires_exact_float_comparison(&self, binary: &syn::ExprBinary) -> bool {
        matches!(binary.op, syn::BinOp::Eq(_) | syn::BinOp::Ne(_))
            && self.requires_exact_float_operands(&binary.left, &binary.right)
    }

    fn requires_exact_float_operands(&self, left: &syn::Expr, right: &syn::Expr) -> bool {
        fn exact_constant(expression: &syn::Expr) -> bool {
            match expression {
                syn::Expr::Paren(paren) => exact_constant(&paren.expr),
                syn::Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => {
                    exact_constant(&unary.expr)
                }
                syn::Expr::Lit(literal) => matches!(&literal.lit, syn::Lit::Float(value)
                    if value.base10_parse::<f64>().is_ok_and(|number| number == 0.0)),
                syn::Expr::Path(path) => {
                    let spelling = path.path.to_token_stream().to_string().replace(' ', "");
                    matches!(
                        spelling.as_str(),
                        "f64::INFINITY"
                            | "f64::NEG_INFINITY"
                            | "f32::INFINITY"
                            | "f32::NEG_INFINITY"
                    )
                }
                _ => false,
            }
        }
        !exact_constant(left) && !exact_constant(right)
            && [left, right].into_iter().all(|value| {
                self.ty(value).is_some_and(|ty| {
                    self.standard_named(unreference(&ty), "f64")
                        || self.standard_named(unreference(&ty), "f32")
                })
            })
    }
}

impl Rewriter<'_> {
    fn known_float(&self, expression: &syn::Expr) -> bool {
        self.ty(expression).is_some_and(|ty| {
            self.standard_named(unreference(&ty), "f64")
                || self.standard_named(unreference(&ty), "f32")
        })
    }

    fn record_source_float_arithmetic(&mut self, expression: &syn::Expr) {
        fn unparen(expression: &syn::Expr) -> &syn::Expr {
            match expression {
                syn::Expr::Paren(paren) => unparen(&paren.expr),
                syn::Expr::Group(group) => unparen(&group.expr),
                _ => expression,
            }
        }
        fn literal(expression: &syn::Expr, expected: f64) -> bool {
            matches!(unparen(expression), syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Float(value), .. })
                if value.base10_parse::<f64>().is_ok_and(|number| number.to_bits() == expected.to_bits()))
        }
        if let syn::Expr::Binary(binary) = expression
            && self.known_float(&binary.left) && self.known_float(&binary.right)
        {
            let multiply = |value: &syn::Expr| matches!(unparen(value), syn::Expr::Binary(product)
                if matches!(product.op, syn::BinOp::Mul(_))
                    && self.known_float(&product.left) && self.known_float(&product.right));
            self.float_expectations.arithmetic |= match binary.op {
                syn::BinOp::Add(_) | syn::BinOp::Sub(_) => multiply(&binary.left) || multiply(&binary.right),
                syn::BinOp::AddAssign(_) | syn::BinOp::SubAssign(_) => multiply(&binary.right),
                _ => false,
            };
            self.float_expectations.midpoint |= matches!(binary.op, syn::BinOp::Div(_))
                && literal(&binary.right, 2.0)
                && matches!(unparen(&binary.left), syn::Expr::Binary(sum)
                    if matches!(sum.op, syn::BinOp::Add(_)));
        }
        if let syn::Expr::MethodCall(call) = expression
            && call.method == "powf" && call.args.len() == 1
            && self.known_float(&call.receiver) && literal(&call.args[0], 0.5)
        {
            self.float_expectations.arithmetic = true;
        }
    }
}
