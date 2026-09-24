#[derive(Default)]
struct CopyVectorSourceCollector {
    sources: HashSet<String>,
    invalidated: HashSet<String>,
}

impl Visit<'_> for CopyVectorSourceCollector {
    fn visit_local(&mut self, local: &syn::Local) {
        if let syn::Pat::Type(typed) = &local.pat
            && type_is_copy_vector(&typed.ty)
            && let Some(name) = simple_pattern_name(&typed.pat)
        {
            if !self.invalidated.contains(&name) {
                self.sources.insert(name);
            }
        } else if let Some(init) = &local.init
            && let syn::Expr::Reference(reference) = init.expr.as_ref()
            && let syn::Expr::Path(path) = reference.expr.as_ref()
            && path
                .path
                .get_ident()
                .is_some_and(|name| self.sources.contains(&name.to_string()))
            && let Some(name) = simple_pattern_name(&local.pat)
        {
            if !self.invalidated.contains(&name) {
                self.sources.insert(name);
            }
        } else if let Some(init) = &local.init
            && matches!(init.expr.as_ref(), syn::Expr::Reference(reference)
                if matches!(reference.expr.as_ref(), syn::Expr::Path(_)))
            && let Some(name) = simple_pattern_name(&local.pat)
        {
            self.sources.remove(&name);
            self.invalidated.insert(name);
        }
        visit::visit_local(self, local);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

fn type_is_copy_vector(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    segment.ident == "Vec"
        && matches!(arguments.args.first(), Some(syn::GenericArgument::Type(syn::Type::Path(element)))
            if element.path.segments.last().is_some_and(|segment|
                matches!(segment.ident.to_string().as_str(),
                    "bool" | "char" | "f32" | "f64" | "i8" | "i16" | "i32" | "i64"
                        | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
                        | "usize")))
}

fn type_is_copy_slice(ty: &syn::Type) -> bool {
    let syn::Type::Reference(reference) = ty else {
        return false;
    };
    let syn::Type::Slice(slice) = reference.elem.as_ref() else {
        return false;
    };
    matches!(slice.elem.as_ref(), syn::Type::Path(element)
        if element.path.segments.last().is_some_and(|segment|
            matches!(segment.ident.to_string().as_str(),
                "bool" | "char" | "f32" | "f64" | "i8" | "i16" | "i32" | "i64"
                    | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
                    | "usize")))
}

struct CopyIteratorRewriter<'names> {
    sources: &'names HashSet<String>,
}

impl VisitMut for CopyIteratorRewriter<'_> {
    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        visit_mut::visit_local_mut(self, local);
        if local_pattern_is_copy_option(&local.pat)
            && let Some(init) = &mut local.init
        {
            ForceCopiedIterator.visit_expr_mut(&mut init.expr);
        }
    }

    fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
        visit_mut::visit_expr_method_call_mut(self, call);
        if call.method == "map"
            && let Some(syn::Expr::Closure(closure)) = call.args.first()
            && closure.inputs.len() == 1
            && let Some(syn::Pat::Ident(binding)) = closure.inputs.first()
            && let syn::Expr::Unary(unary) = closure.body.as_ref()
            && matches!(unary.op, syn::UnOp::Deref(_))
            && matches!(unary.expr.as_ref(), syn::Expr::Path(path)
                if path.path.is_ident(&binding.ident))
            && let syn::Expr::MethodCall(source) = call.receiver.as_ref()
            && matches!(source.method.to_string().as_str(), "get" | "iter")
            && expression_root_name(&source.receiver)
                .is_some_and(|name| self.sources.contains(&name))
        {
            call.method = syn::Ident::new("copied", call.method.span());
            call.args.clear();
            return;
        }
        if call.method != "cloned" || !call.args.is_empty() {
            return;
        }
        let mut receiver = call.receiver.as_ref();
        while let syn::Expr::MethodCall(parent) = receiver {
            receiver = parent.receiver.as_ref();
        }
        if expression_root_name(receiver).is_some_and(|name| self.sources.contains(&name)) {
            call.method = syn::Ident::new("copied", call.method.span());
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}

    fn visit_macro_mut(&mut self, rust_macro: &mut syn::Macro) {
        rewrite_macro_expressions(self, rust_macro);
    }
}

struct ForceCopiedIterator;

impl VisitMut for ForceCopiedIterator {
    fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
        visit_mut::visit_expr_method_call_mut(self, call);
        if call.method == "cloned" && call.args.is_empty() {
            call.method = syn::Ident::new("copied", call.method.span());
        }
    }
}

fn local_pattern_is_copy_option(pattern: &syn::Pat) -> bool {
    let syn::Pat::Type(typed) = pattern else {
        return false;
    };
    let syn::Type::Path(path) = typed.ty.as_ref() else {
        return false;
    };
    let Some(option) = path.path.segments.last() else {
        return false;
    };
    let syn::PathArguments::AngleBracketed(arguments) = &option.arguments else {
        return false;
    };
    option.ident == "Option"
        && matches!(arguments.args.first(), Some(syn::GenericArgument::Type(syn::Type::Path(element)))
            if element.path.segments.last().is_some_and(|segment|
                matches!(segment.ident.to_string().as_str(),
                    "bool" | "char" | "f32" | "f64" | "i8" | "i16" | "i32" | "i64"
                        | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
                        | "usize")))
}
