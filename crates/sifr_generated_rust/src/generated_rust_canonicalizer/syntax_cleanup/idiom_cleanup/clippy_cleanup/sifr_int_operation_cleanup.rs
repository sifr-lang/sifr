#[derive(Default)]
struct SifrIntLocalCollector {
    names: HashSet<String>,
    tuple_vectors: HashSet<String>,
}

impl Visit<'_> for SifrIntLocalCollector {
    fn visit_local(&mut self, local: &syn::Local) {
        if let syn::Pat::Type(typed) = &local.pat
            && let Some(name) = simple_pattern_name(&typed.pat)
        {
            if type_is_sifr_int(&typed.ty) {
                self.names.insert(name.clone());
            }
            if type_is_vector_of_sifr_int_tuple(&typed.ty) {
                self.tuple_vectors.insert(name);
            }
        }
        visit::visit_local(self, local);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

struct SifrIntOperationRewriter<'names> {
    names: &'names HashSet<String>,
    tuple_vectors: &'names HashSet<String>,
    tuple_bindings: HashSet<String>,
}

impl VisitMut for SifrIntOperationRewriter<'_> {
    fn visit_expr_for_loop_mut(&mut self, for_loop: &mut syn::ExprForLoop) {
        self.visit_expr_mut(&mut for_loop.expr);
        let outer = self.tuple_bindings.clone();
        if iterator_root_name(&for_loop.expr).is_some_and(|name| self.tuple_vectors.contains(&name))
            && let Some(binding) = simple_pattern_name(&for_loop.pat)
        {
            self.tuple_bindings.insert(binding);
        }
        self.visit_block_mut(&mut for_loop.body);
        self.tuple_bindings = outer;
    }

    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        if let syn::Expr::MethodCall(call) = expression
            && matches!(
                call.method.to_string().as_str(),
                "checked_to_f64" | "to_bigint"
            )
            && call.args.is_empty()
            && let syn::Expr::MethodCall(clone) = call.receiver.as_ref()
            && clone.method == "clone"
            && clone.args.is_empty()
            && expression_root_name(&clone.receiver).is_some_and(|name| self.names.contains(&name))
        {
            call.receiver = clone.receiver.clone();
            return;
        }
        let syn::Expr::Binary(binary) = expression else {
            return;
        };
        if matches!(binary.op, syn::BinOp::AddAssign(_))
            && expression_root_name(&binary.left).is_some_and(|name| self.names.contains(&name))
        {
            let left = binary.left.clone();
            let right = binary.right.clone();
            *expression = syn::parse_quote!(#left = ::std::ops::Add::add(&#left, &#right));
            return;
        }
        let Some((operation, method)) = exact_integer_operation(&binary.op) else {
            return;
        };
        if !expression_root_name(&binary.left).is_some_and(|name| self.names.contains(&name))
            && !expression_root_name(&binary.right).is_some_and(|name| self.names.contains(&name))
            && !expression_root_name(&binary.left)
                .is_some_and(|name| self.tuple_bindings.contains(&name))
            && !expression_root_name(&binary.right)
                .is_some_and(|name| self.tuple_bindings.contains(&name))
        {
            return;
        }
        let left = binary.left.clone();
        let right = binary.right.clone();
        let operation = syn::Ident::new(operation, proc_macro2::Span::call_site());
        let method = syn::Ident::new(method, proc_macro2::Span::call_site());
        *expression = syn::parse_quote!(::std::ops::#operation::#method(#left, #right));
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

fn iterator_root_name(expression: &syn::Expr) -> Option<String> {
    match expression {
        syn::Expr::MethodCall(call) => iterator_root_name(&call.receiver),
        _ => expression_root_name(expression),
    }
}

fn type_is_vector_of_sifr_int_tuple(ty: &syn::Type) -> bool {
    let syn::Type::Path(vector) = ty else {
        return false;
    };
    let Some(segment) = vector.path.segments.last() else {
        return false;
    };
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    if segment.ident != "Vec" {
        return false;
    }
    matches!(arguments.args.first(), Some(syn::GenericArgument::Type(syn::Type::Tuple(tuple)))
        if !tuple.elems.is_empty() && tuple.elems.iter().all(type_is_sifr_int))
}
