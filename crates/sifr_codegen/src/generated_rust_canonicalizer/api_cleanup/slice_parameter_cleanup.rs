use syn::visit::{self, Visit};

pub(super) fn rewrite_slice_only_vec_parameters(signature: &mut syn::Signature, body: &syn::Block) {
    for input in &mut signature.inputs {
        let syn::FnArg::Typed(parameter) = input else {
            continue;
        };
        let syn::Pat::Ident(binding) = parameter.pat.as_ref() else {
            continue;
        };
        let syn::Type::Reference(reference) = parameter.ty.as_mut() else {
            continue;
        };
        let syn::Type::Path(vector) = reference.elem.as_ref() else {
            continue;
        };
        let Some(segment) = vector.path.segments.last() else {
            continue;
        };
        let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            continue;
        };
        let Some(syn::GenericArgument::Type(element)) = arguments.args.first() else {
            continue;
        };
        if reference.mutability.is_none() || segment.ident != "Vec" {
            continue;
        }
        let mut use_ = SliceOnlyParameterUse {
            name: &binding.ident,
            valid: true,
            method_uses: 0,
        };
        use_.visit_block(body);
        if use_.valid && use_.method_uses > 0 {
            let element = element.clone();
            reference.elem = Box::new(syn::parse_quote!([#element]));
        }
    }
}

struct SliceOnlyParameterUse<'name> {
    name: &'name proc_macro2::Ident,
    valid: bool,
    method_uses: usize,
}

impl Visit<'_> for SliceOnlyParameterUse<'_> {
    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        if matches!(call.receiver.as_ref(), syn::Expr::Path(path) if path.path.is_ident(self.name))
        {
            if matches!(
                call.method.to_string().as_str(),
                "fill"
                    | "reverse"
                    | "rotate_left"
                    | "rotate_right"
                    | "sort"
                    | "sort_by"
                    | "sort_by_key"
                    | "sort_unstable"
                    | "sort_unstable_by"
                    | "sort_unstable_by_key"
                    | "swap"
            ) {
                self.method_uses += 1;
                for argument in &call.args {
                    self.visit_expr(argument);
                }
            } else {
                self.valid = false;
            }
            return;
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_path(&mut self, path: &syn::ExprPath) {
        if path.path.is_ident(self.name) {
            self.valid = false;
        }
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}
