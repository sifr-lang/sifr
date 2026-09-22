impl Rewriter<'_> {
    fn ty(&self, expression: &syn::Expr) -> Option<syn::Type> {
        match expression {
            syn::Expr::Closure(closure) => {
                let inputs = closure
                    .inputs
                    .iter()
                    .map(|input| {
                        if let syn::Pat::Type(typed) = input {
                            Some(*typed.ty.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Option<Vec<_>>>()?;
                let output = &closure.output;
                Some(syn::parse_quote!(fn(#(#inputs),*) #output))
            }
            syn::Expr::Path(path) => self
                .bindings
                .get(&path.path.get_ident()?.to_string())?
                .clone(),
            syn::Expr::Reference(reference) => {
                let inner = self.ty(&reference.expr)?;
                if reference.mutability.is_none() {
                    Some(syn::parse_quote!(&#inner))
                } else {
                    Some(syn::parse_quote!(&mut #inner))
                }
            }
            syn::Expr::Paren(paren) => self.ty(&paren.expr),
            syn::Expr::Tuple(tuple) => {
                let elements = tuple
                    .elems
                    .iter()
                    .map(|element| self.ty(element))
                    .collect::<Option<Vec<_>>>()?;
                Some(syn::parse_quote!((#(#elements,)*)))
            }
            syn::Expr::Field(field) => self.field_type(field),
            syn::Expr::Try(try_) => {
                let carrier = self.ty(&try_.expr)?;
                generic(&carrier, "Option")
                    .or_else(|| generic(&carrier, "Result"))
                    .cloned()
            }
            syn::Expr::Group(group) => self.ty(&group.expr),
            syn::Expr::Lit(lit) if matches!(lit.lit, syn::Lit::Str(_)) => {
                Some(syn::parse_quote!(&str))
            }
            syn::Expr::Lit(lit) if matches!(lit.lit, syn::Lit::Char(_)) => {
                Some(syn::parse_quote!(char))
            }
            syn::Expr::Macro(macro_) if macro_.mac.path.is_ident("vec") => {
                let elements = macro_
                    .mac
                    .parse_body_with(
                        syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
                    )
                    .ok()?;
                let element = self.ty(elements.first()?)?;
                if elements
                    .iter()
                    .all(|value| self.ty(value).is_some_and(|ty| same_type(&element, &ty)))
                {
                    Some(syn::parse_quote!(Vec<#element>))
                } else {
                    None
                }
            }
            syn::Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Deref(_)) => {
                match self.ty(&unary.expr)? {
                    syn::Type::Reference(r) => Some(*r.elem),
                    _ => None,
                }
            }
            syn::Expr::Call(call) => {
                let syn::Expr::Path(path) = call.func.as_ref() else {
                    return None;
                };
                if path.path.is_ident("Some") && call.args.len() == 1 {
                    let inner = self.ty(&call.args[0])?;
                    return Some(syn::parse_quote!(Option<#inner>));
                }
                let key = path.path.to_token_stream().to_string().replace(' ', "");
                if let Some(ident) = path.path.get_ident()
                    && let Some(Some(syn::Type::FnPtr(signature))) =
                        self.bindings.get(&ident.to_string())
                {
                    return match &signature.output {
                        syn::ReturnType::Type(_, ty) => Some(*ty.clone()),
                        syn::ReturnType::Default => None,
                    };
                }
                if matches!(
                    key.as_str(),
                    "SifrInt::from_i64" | "SifrInt::from_usize" | "SifrInt::from"
                ) {
                    return Some(syn::parse_quote!(SifrInt));
                }
                if matches!(
                    key.as_str(),
                    "::std::ops::Add::add"
                        | "::std::ops::Sub::sub"
                        | "::std::ops::Mul::mul"
                        | "::std::ops::Neg::neg"
                ) && call.args.iter().all(|arg| {
                    self.ty(arg)
                        .is_some_and(|ty| named(unreference(&ty), "SifrInt"))
                }) {
                    return Some(syn::parse_quote!(SifrInt));
                }
                let function = self.resolve(&path.path)?;
                if !function.signature.generics.params.is_empty() {
                    return None;
                }
                match &function.signature.output {
                    syn::ReturnType::Type(_, ty) => Some(*ty.clone()),
                    syn::ReturnType::Default => Some(syn::parse_quote!(())),
                }
            }
            syn::Expr::MethodCall(call) => {
                if call.method == "collect"
                    && let Some(arguments) = &call.turbofish
                    && let Some(syn::GenericArgument::Type(ty)) = arguments.args.first()
                {
                    return Some(ty.clone());
                }
                let receiver = self.ty(&call.receiver)?;
                let base = unreference(&receiver);
                if let Some(method) = self.declared_method(base, &call.method)
                    && method.signature.generics.params.is_empty()
                {
                    return match &method.signature.output {
                        syn::ReturnType::Type(_, ty) if named(ty, "Self") => Some(base.clone()),
                        syn::ReturnType::Type(_, ty) => Some(*ty.clone()),
                        syn::ReturnType::Default => Some(syn::parse_quote!(())),
                    };
                }
                match call.method.to_string().as_str() {
                    "to_string"
                        if named(base, "String") || named(base, "str") || named(base, "char") =>
                    {
                        Some(syn::parse_quote!(String))
                    }
                    "to_owned" if named(base, "String") || named(base, "str") => {
                        Some(syn::parse_quote!(String))
                    }
                    "clone" if call.args.is_empty() => Some(match receiver {
                        syn::Type::Reference(reference) => *reference.elem,
                        _ => receiver,
                    }),
                    "get" => {
                        let inner = generic(base, "Vec")
                            .or_else(|| {
                                generic(base, "HashMap").and_then(|_| {
                                    let syn::Type::Path(path) = base else {
                                        return None;
                                    };
                                    let syn::PathArguments::AngleBracketed(args) =
                                        &path.path.segments.last()?.arguments
                                    else {
                                        return None;
                                    };
                                    match args.args.iter().nth(1)? {
                                        syn::GenericArgument::Type(ty) => Some(ty),
                                        _ => None,
                                    }
                                })
                            })
                            .or_else(|| {
                                if let syn::Type::Slice(slice) = base {
                                    Some(slice.elem.as_ref())
                                } else {
                                    None
                                }
                            })?;
                        Some(syn::parse_quote!(Option<&#inner>))
                    }
                    "as_slice" if generic(base, "Vec").is_some() => {
                        let inner = generic(base, "Vec")?;
                        Some(syn::parse_quote!(&[#inner]))
                    }
                    "as_str" if named(base, "String") => Some(syn::parse_quote!(&str)),
                    "as_ref" if generic(base, "Option").is_some() => {
                        let inner = generic(base, "Option")?;
                        Some(syn::parse_quote!(Option<&#inner>))
                    }
                    "cloned" => {
                        let syn::Type::Reference(inner) = generic(base, "Option")? else {
                            return None;
                        };
                        let inner = &inner.elem;
                        Some(syn::parse_quote!(Option<#inner>))
                    }
                    "copied" => {
                        let syn::Type::Reference(inner) = generic(base, "Option")? else {
                            return None;
                        };
                        let inner = &inner.elem;
                        Some(syn::parse_quote!(Option<#inner>))
                    }
                    "map" if generic(base, "Option").is_some() && call.args.len() == 1 => {
                        let syn::Expr::Closure(closure) = &call.args[0] else {
                            return None;
                        };
                        let mut nested = Rewriter {
                            functions: self.functions,
                            structures: self.structures,
                            self_type: self.self_type.clone(),
                            scope: self.scope.clone(),
                            module_depth: self.module_depth,
                            bindings: self.bindings.clone(),
                        };
                        if closure.inputs.len() != 1 {
                            return None;
                        }
                        nested.bind(&closure.inputs[0], generic(base, "Option").cloned());
                        let ty = nested.ty(&closure.body)?;
                        Some(syn::parse_quote!(Option<#ty>))
                    }
                    _ => None,
                }
            }
            syn::Expr::Block(block) => {
                let mut nested = Rewriter {
                    functions: self.functions,
                    structures: self.structures,
                    self_type: self.self_type.clone(),
                    scope: self.scope.clone(),
                    module_depth: self.module_depth,
                    bindings: self.bindings.clone(),
                };
                for stmt in &block.block.stmts {
                    if let syn::Stmt::Local(local) = stmt {
                        nested.bind(
                            &local.pat,
                            local.init.as_ref().and_then(|init| nested.ty(&init.expr)),
                        );
                    }
                }
                match block.block.stmts.last()? {
                    syn::Stmt::Expr(expr, None) => nested.ty(expr),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
