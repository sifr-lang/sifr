fn pattern_binding_names(pattern: &syn::Pat) -> HashSet<String> {
    struct Collector(HashSet<String>);
    impl Visit<'_> for Collector {
        fn visit_pat_ident(&mut self, binding: &syn::PatIdent) {
            self.0.insert(binding.ident.to_string());
            visit::visit_pat_ident(self, binding);
        }
    }
    let mut collector = Collector(HashSet::new());
    collector.visit_pat(pattern);
    collector.0
}

struct BorrowedCopyUnionCloneRewriter<'names> {
    borrowed_roots: &'names HashSet<String>,
}

impl VisitMut for BorrowedCopyUnionCloneRewriter<'_> {
    fn visit_expr_match_mut(&mut self, match_: &mut syn::ExprMatch) {
        self.visit_expr_mut(&mut match_.expr);
        let borrowed_match = expression_root_name(&match_.expr)
            .is_some_and(|name| self.borrowed_roots.contains(&name));
        for arm in &mut match_.arms {
            if borrowed_match && pattern_is_copy_union_variant(&arm.pat) {
                let mut bindings = HashSet::new();
                collect_owned_pattern_names(&arm.pat, &mut bindings);
                BorrowedCopyBindingCloneRewriter {
                    bindings: &bindings,
                }
                .visit_expr_mut(&mut arm.body);
            }
            self.visit_expr_mut(&mut arm.body);
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

fn pattern_is_copy_union_variant(pattern: &syn::Pat) -> bool {
    let syn::Pat::TupleStruct(tuple) = pattern else {
        return false;
    };
    tuple.path.segments.last().is_some_and(|segment| {
        let name = segment.ident.to_string();
        name.contains("atom4X3abool") || name.contains("atom5X3afloat")
    })
}

struct BorrowedCopyBindingCloneRewriter<'names> {
    bindings: &'names HashSet<String>,
}

impl VisitMut for BorrowedCopyBindingCloneRewriter<'_> {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        let syn::Expr::MethodCall(clone) = expression else {
            return;
        };
        if clone.method == "clone"
            && clone.args.is_empty()
            && matches!(clone.receiver.as_ref(), syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|name|
                    self.bindings.contains(&name.to_string())))
        {
            let receiver = clone.receiver.as_ref();
            *expression = syn::parse_quote!(*#receiver);
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

#[derive(Default)]
struct UsizeLocalCollector {
    names: HashSet<String>,
}

impl Visit<'_> for UsizeLocalCollector {
    fn visit_local(&mut self, local: &syn::Local) {
        if let Some(name) = simple_pattern_name(&local.pat) {
            let typed = matches!(&local.pat, syn::Pat::Type(typed)
                if matches!(typed.ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("usize")));
            let inferred = local
                .init
                .as_ref()
                .is_some_and(|init| match init.expr.as_ref() {
                    syn::Expr::Lit(literal) => matches!(&literal.lit, syn::Lit::Int(value)
                    if value.suffix() == "usize"
                        || (name.starts_with("sifr_generated_count")
                            && value.base10_digits() == "0")),
                    syn::Expr::MethodCall(call) => matches!(
                        call.method.to_string().as_str(),
                        "len" | "clamp_slice_bound"
                    ),
                    syn::Expr::Path(path) => path
                        .path
                        .get_ident()
                        .is_some_and(|source| self.names.contains(&source.to_string())),
                    _ => false,
                });
            if typed || inferred {
                self.names.insert(name);
            }
        }
        visit::visit_local(self, local);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

struct UsizeCounterRewriter<'names> {
    names: &'names HashSet<String>,
}

impl VisitMut for UsizeCounterRewriter<'_> {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        let syn::Expr::Binary(binary) = expression else {
            return;
        };
        if !matches!(binary.op, syn::BinOp::AddAssign(_))
            || !matches!(binary.left.as_ref(), syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|name| self.names.contains(&name.to_string())))
            || !matches!(binary.right.as_ref(), syn::Expr::Lit(literal)
                if matches!(&literal.lit, syn::Lit::Int(value)
                    if value.base10_digits() == "1"))
        {
            return;
        }
        let left = binary.left.as_ref();
        *expression = syn::parse_quote!(#left = (#left).saturating_add(1usize));
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

fn type_is_owned_string(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path) if path.qself.is_none() && path.path.is_ident("String"))
}

fn type_is_sifr_int(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path)
        if path.path.segments.last().is_some_and(|segment| segment.ident == "SifrInt"))
}
