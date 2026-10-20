// Exact runtime-integer operations require both operand identities. Primitive,
// opaque, and shadowed types retain their own arithmetic and diagnostics.
impl Rewriter<'_> {
    fn rewrite_exact_integer_arithmetic(&self, expression: &mut syn::Expr) {
        let syn::Expr::Binary(binary) = expression else {
            return;
        };
        let (operation, method) = match binary.op {
            syn::BinOp::Add(_) => ("Add", "add"),
            syn::BinOp::Sub(_) => ("Sub", "sub"),
            syn::BinOp::Mul(_) => ("Mul", "mul"),
            _ => return,
        };
        if ![&binary.left, &binary.right].iter().all(|operand| {
            self.ty(operand)
                .is_some_and(|ty| self.standard_named(unreference(&ty), "SifrInt"))
        }) {
            return;
        }
        let left = &binary.left;
        let right = &binary.right;
        let operation = syn::Ident::new(operation, proc_macro2::Span::call_site());
        let method = syn::Ident::new(method, proc_macro2::Span::call_site());
        *expression = syn::parse_quote!(::std::ops::#operation::#method(#left, #right));
    }

    fn is_standard_integer_range(&self, expression: &syn::Expr) -> bool {
        if self.scalar_shadowed("SifrRange") || self.scalar_shadowed("SifrInt") {
            return false;
        }
        let syn::Expr::Call(call) = expression else {
            return false;
        };
        let syn::Expr::Path(path) = call.func.as_ref() else {
            return false;
        };
        path.qself.is_none()
            && path.path.leading_colon.is_none()
            && path.path.segments.len() == 2
            && path.path.segments[0].ident == "SifrRange"
            && path.path.segments[1].ident == "new_known_nonzero"
            && path.path.segments.iter().all(|segment| {
                matches!(segment.arguments, syn::PathArguments::None)
            })
            && call.args.len() == 3
    }
}
