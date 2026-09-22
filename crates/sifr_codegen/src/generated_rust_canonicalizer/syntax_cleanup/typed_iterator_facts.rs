// Iterator item facts derived from resolved collection and closure types.

impl Rewriter<'_> {
    fn iterator_element(&self, expression: &syn::Expr) -> Option<syn::Type> {
        if let syn::Expr::Paren(paren) = expression {
            return self.iterator_element(&paren.expr);
        }
        if let syn::Expr::Call(call) = expression
            && matches!(call.func.as_ref(), syn::Expr::Path(path) if path.path.segments.len() == 2 && path.path.segments[0].ident == "Box" && path.path.segments[1].ident == "new")
            && call.args.len() == 1
            && !(0..=self.scope.len()).any(|depth| {
                let mut path = self.scope[..depth].to_vec();
                path.push("Box".to_string());
                self.structures.contains_key(&path.join("::"))
            })
        {
            return self.iterator_element(&call.args[0]);
        }
        if let syn::Expr::MethodCall(call) = expression {
            match call.method.to_string().as_str() {
                "iter" | "iter_mut" | "into_iter" if call.args.is_empty() => {
                    let ty = self.ty(&call.receiver)?;
                    let element =
                        self.standard_generic(unreference(&ty), "Vec").or_else(
                            || match unreference(&ty) {
                                syn::Type::Slice(slice) => Some(slice.elem.as_ref()),
                                _ => None,
                            },
                        )?;
                    return Some(match call.method.to_string().as_str() {
                        "iter_mut" => syn::parse_quote!(&mut #element),
                        "iter" => syn::parse_quote!(&#element),
                        _ if matches!(ty, syn::Type::Reference(_)) => syn::parse_quote!(&#element),
                        _ => element.clone(),
                    });
                }
                "cloned" | "copied" if call.args.is_empty() => {
                    return match self.iterator_element(&call.receiver)? {
                        syn::Type::Reference(reference) => Some(*reference.elem),
                        _ => None,
                    };
                }
                "enumerate" if call.args.is_empty() => {
                    let element = self.iterator_element(&call.receiver)?;
                    return Some(syn::parse_quote!((usize, #element)));
                }
                "map" if call.args.len() == 1 => {
                    let syn::Expr::Closure(closure) = &call.args[0] else {
                        return None;
                    };
                    if closure.inputs.len() != 1 {
                        return None;
                    }
                    let element = self.iterator_element(&call.receiver)?;
                    let mut nested = Rewriter {
                        iteration_dispatch_closed: self.iteration_dispatch_closed,
                        ambiguous_clone_scopes: self.ambiguous_clone_scopes,
                        scalar_shadows: self.scalar_shadows,
                        functions: self.functions,
                        structures: self.structures,
                        self_type: self.self_type.clone(),
                        scope: self.scope.clone(),
                        module_depth: self.module_depth,
                        bindings: self.bindings.clone(),
                        discardable_assignments: HashMap::new(),
                        exact_float_comparison: false,
                    };
                    nested.bind(&closure.inputs[0], Some(element));
                    return nested.ty(&closure.body);
                }
                _ => {}
            }
        }
        let ty = self.ty(expression)?;
        let element = self.standard_generic(unreference(&ty), "Vec")?;
        Some(if matches!(ty, syn::Type::Reference(_)) {
            syn::parse_quote!(&#element)
        } else {
            element.clone()
        })
    }
}

fn shared_vector_slice_coercion(source: &syn::Type, target: &syn::Type) -> bool {
    let (syn::Type::Reference(source), syn::Type::Reference(target)) = (source, target) else {
        return false;
    };
    if source.mutability.is_some() || target.mutability.is_some() {
        return false;
    }
    let Some(element) = generic(&source.elem, "Vec") else {
        return false;
    };
    matches!(target.elem.as_ref(), syn::Type::Slice(slice) if same_type(element, &slice.elem))
}

fn runtime_call_inputs(path: &syn::Path) -> Option<Vec<syn::Type>> {
    // These are the actual encoding ABI entries, not a namespace-wide rule.
    match path
        .to_token_stream()
        .to_string()
        .replace(' ', "")
        .trim_start_matches("::")
    {
        "sifr_runtime::encoding::encode_bytes" => Some(vec![
            syn::parse_quote!(&str),
            syn::parse_quote!(&str),
            syn::parse_quote!(&str),
        ]),
        "sifr_runtime::encoding::decode_text" => Some(vec![
            syn::parse_quote!(&[u8]),
            syn::parse_quote!(&str),
            syn::parse_quote!(&str),
        ]),
        _ => None,
    }
}

impl Rewriter<'_> {
    fn standard_runtime_inputs(&self, path: &syn::Path) -> Option<Vec<syn::Type>> {
        let spelling = path.to_token_stream().to_string().replace(' ', "");
        if spelling == "::sifr_runtime::SifrInt::parse_decimal"
            || (spelling == "SifrInt::parse_decimal" && !self.scalar_shadowed("SifrInt"))
        {
            return Some(vec![syn::parse_quote!(&str), syn::parse_quote!(usize)]);
        }
        None
    }

    fn rewrite_standard_copy_iterator(&self, expression: &mut syn::Expr) {
        if !self.iteration_dispatch_closed || !self.clone_is_unambiguous() {
            return;
        }
        let syn::Expr::MethodCall(cloned) = expression else {
            return;
        };
        if cloned.method != "cloned" || !cloned.args.is_empty() {
            return;
        }
        let syn::Expr::MethodCall(iterated) = cloned.receiver.as_ref() else {
            return;
        };
        if iterated.method != "iter" || !iterated.args.is_empty() {
            return;
        }
        let Some(receiver) = self.ty(&iterated.receiver) else {
            return;
        };
        let element = self
            .standard_generic(unreference(&receiver), "Vec")
            .or_else(|| {
                if let syn::Type::Slice(slice) = unreference(&receiver) {
                    Some(slice.elem.as_ref())
                } else {
                    None
                }
            });
        if element.is_some_and(|ty| {
            [
                "bool", "char", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32",
                "i64", "i128", "isize", "f32", "f64",
            ]
            .into_iter()
            .any(|name| self.standard_named(ty, name))
        }) {
            cloned.method = syn::Ident::new("copied", cloned.method.span());
        }
    }
}

// Extension traits can win method resolution before Vec's slice dereference.
// Unknown imported traits are covered separately by the opaque-scope guard.
fn iteration_dispatch_is_closed(files: &[syn::File]) -> bool {
    struct Methods(bool);
    impl<'ast> syn::visit::Visit<'ast> for Methods {
        fn visit_trait_item_fn(&mut self, method: &'ast syn::TraitItemFn) {
            self.0 |= matches!(
                method.sig.ident.to_string().as_str(),
                "iter" | "cloned" | "copied"
            );
            syn::visit::visit_trait_item_fn(self, method);
        }
        fn visit_item_impl(&mut self, implementation: &'ast syn::ItemImpl) {
            if implementation.trait_.is_some() {
                self.0 |= implementation.items.iter().any(|item|
                    matches!(item, syn::ImplItem::Fn(method)
                        if matches!(method.sig.ident.to_string().as_str(), "iter" | "cloned" | "copied")));
            }
            syn::visit::visit_item_impl(self, implementation);
        }
    }
    let mut methods = Methods(false);
    for file in files {
        syn::visit::Visit::visit_file(&mut methods, file);
    }
    !methods.0
}
