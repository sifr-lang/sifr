fn collect(items: &[syn::Item], scope: &[String], functions: &mut HashMap<String, Callable>) {
    for item in items {
        match item {
            syn::Item::Impl(implementation)
                if implementation.trait_.is_none() && implementation.generics.params.is_empty() =>
            {
                let syn::Type::Path(owner) = implementation.self_ty.as_ref() else {
                    continue;
                };
                let mut path = scope.to_vec();
                path.extend(
                    owner
                        .path
                        .segments
                        .iter()
                        .map(|segment| segment.ident.to_string()),
                );
                for item in &implementation.items {
                    if let syn::ImplItem::Fn(function) = item {
                        functions.insert(
                            format!("{}::{}", path.join("::"), function.sig.ident),
                            Callable {
                                signature: function.sig.clone(),
                            },
                        );
                    }
                }
            }
            syn::Item::Enum(enum_) if enum_.generics.params.is_empty() => {
                let mut owner = scope.to_vec();
                owner.push(enum_.ident.to_string());
                let segments = owner
                    .iter()
                    .map(|name| syn::Ident::new(name, enum_.ident.span()))
                    .collect::<Vec<_>>();
                let output: syn::Type = syn::parse_quote!(#(#segments)::*);
                for variant in &enum_.variants {
                    if !matches!(variant.fields, syn::Fields::Unnamed(_)) {
                        continue;
                    }
                    let parameters = variant
                        .fields
                        .iter()
                        .enumerate()
                        .map(|(index, field)| {
                            let name =
                                syn::Ident::new(&format!("field_{index}"), variant.ident.span());
                            let ty = &field.ty;
                            syn::parse_quote!(#name: #ty)
                        })
                        .collect::<Vec<syn::FnArg>>();
                    let name = &variant.ident;
                    functions.insert(
                        format!("{}::{name}", owner.join("::")),
                        Callable {
                            signature: syn::parse_quote!(fn #name(#(#parameters),*) -> #output),
                        },
                    );
                }
            }
            syn::Item::Fn(function) => {
                let mut path = scope.to_vec();
                path.push(function.sig.ident.to_string());
                functions.insert(
                    path.join("::"),
                    Callable {
                        signature: function.sig.clone(),
                    },
                );
                let nested = function
                    .block
                    .stmts
                    .iter()
                    .filter_map(|stmt| {
                        if let syn::Stmt::Item(item) = stmt {
                            Some(item.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                collect(&nested, &path, functions);
            }
            syn::Item::Mod(module) => {
                if let Some((_, items)) = &module.content {
                    let mut path = scope.to_vec();
                    path.push(module.ident.to_string());
                    collect(items, &path, functions);
                }
            }
            _ => {}
        }
    }
}

fn collect_structures(
    items: &[syn::Item],
    scope: &[String],
    structures: &mut HashMap<String, syn::ItemStruct>,
) {
    for item in items {
        match item {
            syn::Item::Struct(structure) => {
                let mut path = scope.to_vec();
                path.push(structure.ident.to_string());
                structures.insert(path.join("::"), structure.clone());
            }
            syn::Item::Mod(module) => {
                if let Some((_, items)) = &module.content {
                    let mut path = scope.to_vec();
                    path.push(module.ident.to_string());
                    collect_structures(items, &path, structures);
                }
            }
            _ => {}
        }
    }
}

impl Rewriter<'_> {
    fn expected_block(&mut self, block: &mut syn::Block, expected: &syn::Type) {
        let outer = self.bindings.clone();
        let returned_binding = block.stmts.last().and_then(|statement| match statement {
            syn::Stmt::Expr(syn::Expr::Path(path), None) => {
                path.path.get_ident().map(ToString::to_string)
            }
            _ => None,
        });
        let returned_declaration = returned_binding.as_ref().and_then(|name| block.stmts.iter().rposition(|statement| {
            matches!(statement, syn::Stmt::Local(local) if super::identifier_names_in_pattern(&local.pat).contains(name))
        }));
        let last = block.stmts.len().saturating_sub(1);
        for (index, statement) in block.stmts.iter_mut().enumerate() {
            match statement {
                syn::Stmt::Local(local) => {
                    if let Some(init) = &mut local.init {
                        if returned_declaration == Some(index) {
                            self.expected(&mut init.expr, expected);
                        } else {
                            self.visit_expr_mut(&mut init.expr);
                        }
                    }
                    self.bind(
                        &local.pat,
                        local.init.as_ref().and_then(|init| self.ty(&init.expr)),
                    );
                }
                syn::Stmt::Expr(expression, None) if index == last => {
                    self.expected(expression, expected);
                }
                _ => self.visit_stmt_mut(statement),
            }
        }
        self.bindings = outer;
    }
    fn align_comparison_references(
        &self,
        arguments: &mut syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>,
    ) {
        let (Some(left), Some(right)) = (self.ty(&arguments[0]), self.ty(&arguments[1])) else {
            return;
        };
        let base = unreference(&left);
        if !same_type(base, unreference(&right))
            || !(named(base, "SifrInt") || named(base, "String"))
        {
            return;
        }
        let depth = |mut ty: &syn::Type| {
            let mut count: usize = 0;
            while let syn::Type::Reference(reference) = ty {
                count += 1;
                ty = &reference.elem;
            }
            count
        };
        let (left_depth, right_depth) = (depth(&left), depth(&right));
        let index = usize::from(right_depth > left_depth);
        for _ in 0..left_depth.abs_diff(right_depth) {
            arguments[index] = match &arguments[index] {
                syn::Expr::Reference(reference) => *reference.expr.clone(),
                value => syn::parse_quote!(*#value),
            };
        }
    }

    fn move_unused_string_copy(
        &self,
        local: &mut syn::Local,
        remaining: &[syn::Stmt],
        owned: &std::collections::HashSet<String>,
    ) {
        let unused = super::identifier_names_in_pattern(&local.pat)
            .iter()
            .all(|name| !statements_reference(remaining, name));
        if !unused {
            return;
        }
        let Some(init) = &mut local.init else { return };
        let syn::Expr::MethodCall(clone) = init.expr.as_ref() else {
            return;
        };
        if clone.method != "clone"
            || !clone.args.is_empty()
            || !self
                .ty(&clone.receiver)
                .is_some_and(|ty| named(&ty, "String"))
        {
            return;
        }
        let Some(root) = value_root(&clone.receiver) else {
            return;
        };
        if owned.contains(&root) && !statements_reference(remaining, &root) {
            init.expr = clone.receiver.clone();
        }
    }
    fn field_type(&self, field: &syn::ExprField) -> Option<syn::Type> {
        let owner = self.ty(&field.base)?;
        if let (syn::Type::Tuple(tuple), syn::Member::Unnamed(index)) =
            (unreference(&owner), &field.member)
        {
            return tuple.elems.iter().nth(index.index as usize).cloned();
        }
        let syn::Type::Path(owner) = unreference(&owner) else {
            return None;
        };
        let parts = owner
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        let structure = (0..=self.scope.len()).rev().find_map(|depth| {
            let mut key = self.scope[..depth].to_vec();
            key.extend(parts.iter().cloned());
            self.structures.get(&key.join("::"))
        })?;
        let field_type = match &field.member {
            syn::Member::Named(name) => structure
                .fields
                .iter()
                .find(|field| field.ident.as_ref() == Some(name))?
                .ty
                .clone(),
            syn::Member::Unnamed(index) => structure
                .fields
                .iter()
                .nth(index.index as usize)?
                .ty
                .clone(),
        };
        let syn::Type::Path(field_path) = &field_type else {
            return Some(field_type);
        };
        let Some(parameter) = field_path.path.get_ident() else {
            return Some(field_type);
        };
        let Some(index) = structure
            .generics
            .type_params()
            .position(|param| param.ident == *parameter)
        else {
            return Some(field_type);
        };
        let syn::PathArguments::AngleBracketed(arguments) = &owner.path.segments.last()?.arguments
        else {
            return None;
        };
        match arguments.args.iter().nth(index)? {
            syn::GenericArgument::Type(ty) => Some(ty.clone()),
            _ => None,
        }
    }

    fn rewrite_vector_collect(&self, expression: &mut syn::Expr, expected: &syn::Type) {
        let Some(target) = generic(expected, "Vec") else {
            return;
        };
        let syn::Expr::MethodCall(collect) = expression else {
            return;
        };
        if collect.method != "collect" || !collect.args.is_empty() {
            return;
        }
        let syn::Expr::MethodCall(copied) = collect.receiver.as_ref() else {
            return;
        };
        if !matches!(copied.method.to_string().as_str(), "cloned" | "copied")
            || !copied.args.is_empty()
        {
            return;
        }
        let syn::Expr::MethodCall(iterated) = copied.receiver.as_ref() else {
            return;
        };
        if iterated.method != "iter" || !iterated.args.is_empty() {
            return;
        }
        let Some(source) = self.ty(&iterated.receiver) else {
            return;
        };
        let source_element = generic(unreference(&source), "Vec").or_else(|| {
            if let syn::Type::Slice(slice) = unreference(&source) {
                Some(slice.elem.as_ref())
            } else {
                None
            }
        });
        let Some(element) = source_element else {
            return;
        };
        if !matches!(target, syn::Type::Infer(_)) && !same_type(target, element) {
            return;
        }
        let receiver = &iterated.receiver;
        *expression = if generic(&source, "Vec").is_some() {
            syn::parse_quote!(#receiver.clone())
        } else {
            syn::parse_quote!(#receiver.to_vec())
        };
    }
}

fn result_error(ty: &syn::Type) -> Option<&syn::Type> {
    generic(ty, "Result")?;
    let syn::Type::Path(path) = ty else {
        return None;
    };
    let syn::PathArguments::AngleBracketed(arguments) = &path.path.segments.last()?.arguments
    else {
        return None;
    };
    match arguments.args.iter().nth(1)? {
        syn::GenericArgument::Type(ty) => Some(ty),
        _ => None,
    }
}

fn value_root(value: &syn::Expr) -> Option<String> {
    match value {
        syn::Expr::Path(path) => path.path.get_ident().map(ToString::to_string),
        syn::Expr::Field(field) => value_root(&field.base),
        syn::Expr::Paren(paren) => value_root(&paren.expr),
        _ => None,
    }
}

fn statements_reference(statements: &[syn::Stmt], name: &str) -> bool {
    use syn::visit::Visit;
    struct References<'a> {
        name: &'a str,
        found: bool,
    }
    impl<'ast> Visit<'ast> for References<'_> {
        fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
            self.found |= path.path.is_ident(self.name);
        }
        fn visit_macro(&mut self, macro_: &'ast syn::Macro) {
            if let Ok(arguments) = macro_.parse_body_with(
                syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
            ) {
                for argument in &arguments {
                    self.visit_expr(argument);
                }
                // Captured format arguments are uses too.
                self.found |= macro_.tokens.to_string().contains(self.name);
            } else {
                self.found = true;
            }
        }
    }
    let mut references = References { name, found: false };
    for statement in statements {
        references.visit_stmt(statement);
    }
    references.found
}

fn exclude_drop_structures(
    items: &[syn::Item],
    scope: &[String],
    structures: &mut HashMap<String, syn::ItemStruct>,
) {
    for item in items {
        match item {
            syn::Item::Impl(implementation)
                if implementation.trait_.as_ref().is_some_and(|(path, _)| {
                    path.segments
                        .last()
                        .is_some_and(|segment| segment.ident == "Drop")
                }) =>
            {
                if let syn::Type::Path(owner) = implementation.self_ty.as_ref() {
                    let mut path = scope.to_vec();
                    path.extend(
                        owner
                            .path
                            .segments
                            .iter()
                            .map(|segment| segment.ident.to_string()),
                    );
                    structures.remove(&path.join("::"));
                }
            }
            syn::Item::Mod(module) => {
                if let Some((_, items)) = &module.content {
                    let mut path = scope.to_vec();
                    path.push(module.ident.to_string());
                    exclude_drop_structures(items, &path, structures);
                }
            }
            _ => {}
        }
    }
}
