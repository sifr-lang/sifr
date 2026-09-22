include!("typed_local_transfer.rs");

// Stored values with only compiler-known Clone and Drop behavior.
impl Rewriter<'_> {
    fn inert_owned_type(&self, ty: &syn::Type) -> bool {
        self.inert_type(ty, false)
    }

    fn inert_type(&self, ty: &syn::Type, inside_record: bool) -> bool {
        match ty {
            syn::Type::Tuple(tuple) => {
                return tuple
                    .elems
                    .iter()
                    .all(|ty| self.inert_type(ty, inside_record));
            }
            syn::Type::Array(array) => return self.inert_type(&array.elem, inside_record),
            syn::Type::Paren(paren) => return self.inert_type(&paren.elem, inside_record),
            _ => {}
        }
        let syn::Type::Path(path) = ty else {
            return false;
        };
        if path.qself.is_some() {
            return false;
        }
        if inside_record && path.path.is_ident("Self") {
            return true;
        }
        let Some(last) = path.path.segments.last() else {
            return false;
        };
        let name = last.ident.to_string();
        let spelling = path.path.to_token_stream().to_string().replace(' ', "");
        let standard = path.path.segments.len() == 1 && !self.scalar_shadowed(&name)
            || matches!(
                spelling.as_str(),
                "::std::string::String" | "::sifr_runtime::SifrInt"
            );
        if standard
            && matches!(
                name.as_str(),
                "String"
                    | "SifrInt"
                    | "bool"
                    | "char"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
                    | "u128"
                    | "usize"
                    | "i8"
                    | "i16"
                    | "i32"
                    | "i64"
                    | "i128"
                    | "isize"
                    | "f32"
                    | "f64"
            )
            && matches!(last.arguments, syn::PathArguments::None)
        {
            return true;
        }
        if standard
            && matches!(name.as_str(), "Option" | "Vec" | "Box" | "HashSet")
            && let syn::PathArguments::AngleBracketed(arguments) = &last.arguments
            && arguments.args.len() == 1
            && let Some(syn::GenericArgument::Type(inner)) = arguments.args.first()
        {
            return self.inert_type(inner, inside_record);
        }
        // Only Self recursion is resolved inside a record: unrelated bare field
        // names belong to the declaration's module, not the caller's scope.
        if inside_record || self.scalar_shadowed("Clone") {
            return false;
        }
        let parts = path
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>();
        let structure = (0..=self.scope.len()).rev().find_map(|depth| {
            let mut key = self.scope[..depth].to_vec();
            key.extend(parts.iter().cloned());
            self.structures.get(&key.join("::"))
        });
        let Some(structure) = structure else {
            return false;
        };
        if !structure.generics.params.is_empty() {
            return false;
        }
        let derived =
            structure.attrs.iter().any(|attribute| {
                attribute.path().is_ident("derive") && attribute.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated
            ).is_ok_and(|derives| derives.iter().any(|derive| derive.is_ident("Clone")))
            });
        derived
            && structure
                .fields
                .iter()
                .all(|field| self.inert_type(&field.ty, true))
    }

    fn discardable_unused_owned_binding(
        &self,
        local: &syn::Local,
        remaining: &[syn::Stmt],
    ) -> bool {
        let Some(init) = &local.init else {
            return false;
        };
        !super::identifier_names_in_pattern(&local.pat).is_empty()
            && init.diverge.is_none()
            && matches!(init.expr.as_ref(), syn::Expr::Path(path) if path.path.get_ident().is_some())
            && super::identifier_names_in_pattern(&local.pat)
                .iter()
                .all(|name| !statements_reference(remaining, name))
            && self.ty(&init.expr).or_else(|| match &local.pat {
                syn::Pat::Type(typed) => Some(*typed.ty.clone()),
                _ => None,
            }).is_some_and(|ty| self.inert_owned_type(&ty))
    }
}

impl Rewriter<'_> {
    fn owned_pattern_bindings(&self, patterns: impl IntoIterator<Item = syn::Pat>) -> std::collections::HashSet<String> {
        if !self.clone_is_unambiguous() { return std::collections::HashSet::new(); }
        patterns.into_iter().flat_map(|pattern| super::identifier_names_in_pattern(&pattern))
            .filter(|name| self.bindings.get(name).and_then(Option::as_ref)
                .is_some_and(|ty| self.inert_owned_type(ty))).collect()
    }

    fn owned_function_inputs(&self, signature: &syn::Signature) -> std::collections::HashSet<String> {
        self.owned_pattern_bindings(signature.inputs.iter().filter_map(|input| match input {
            syn::FnArg::Typed(parameter) => Some(*parameter.pat.clone()),
            syn::FnArg::Receiver(_) => None,
        }))
    }

    fn cleanup_block(&mut self, block: &mut syn::Block) {
        self.cleanup_block_with_fresh(block, std::collections::HashSet::new());
    }

    fn cleanup_block_with_fresh(&mut self, block: &mut syn::Block, mut fresh_locals: std::collections::HashSet<String>) {
        self.fold_proven_initializers(block);
        let outer = self.bindings.clone();
        super::idiom_cleanup::remove_known_vec_length_bindings(&mut block.stmts, |ty| {
            self.standard_generic(ty, "Vec").is_some() && !self.scalar_shadowed("Vec")
        });
        let mut owned_locals = std::collections::HashSet::new();
        let mut discard = Vec::new();
        for index in 0..block.stmts.len() {
            let (processed, remaining) = block.stmts.split_at_mut(index + 1);
            let statement = &mut processed[index];
            let fresh_before = fresh_locals.clone();
            self.transfer_fresh_terminal_branch(statement, &fresh_locals);
            if let syn::Stmt::Local(local) = statement {
                self.transfer_fresh_shadowed_option(local, &fresh_locals);
                if let Some(init) = &mut local.init {
                    if let syn::Pat::Type(typed) = &local.pat {
                        self.expected(&mut init.expr, &typed.ty);
                    } else {
                        self.visit_expr_mut(&mut init.expr);
                    }
                    if let Some((_, diverge)) = &mut init.diverge {
                        self.visit_expr_mut(diverge);
                    }
                }
                let ty = local.init.as_ref().and_then(|init| self.ty(&init.expr));
                self.move_unused_string_copy(local, remaining, &owned_locals);
                if self.discardable_unused_string_field(local, remaining)
                    || self.discardable_unused_owned_binding(local, remaining)
                {
                    discard.push(index);
                }
                self.bind(&local.pat, ty);
                if self.clone_is_unambiguous() {
                    let owned = super::identifier_names_in_pattern(&local.pat)
                        .into_iter()
                        .filter(|name| {
                            self.bindings
                                .get(name)
                                .and_then(Option::as_ref)
                                .is_some_and(|ty| self.inert_owned_type(ty))
                        })
                        .collect();
                    super::idiom_cleanup::clean_owned_suffix(remaining, owned);
                }
                fresh_locals.retain(|name| {
                    !statements_reference(
                        std::slice::from_ref(&syn::Stmt::Local(local.clone())),
                        name,
                    )
                });
                for name in super::identifier_names_in_pattern(&local.pat) {
                    if self
                        .bindings
                        .get(&name)
                        .and_then(Option::as_ref)
                        .is_some_and(|ty| self.inert_owned_type(ty))
                    {
                        fresh_locals.insert(name.clone());
                    }
                    owned_locals.remove(&name);
                    if self.bindings.get(&name).is_some_and(|ty| {
                        ty.as_ref()
                            .is_some_and(|ty| !matches!(ty, syn::Type::Reference(_)))
                    }) {
                        owned_locals.insert(name);
                    }
                }
            } else {
                self.visit_stmt_mut(statement);
                fresh_locals
                    .retain(|name| !statements_reference(std::slice::from_ref(statement), name));
            }
            let field_owners = owned_locals.union(&fresh_before).cloned().collect();
            self.remove_terminal_owned_field_clones(statement, remaining, &field_owners);
            self.record_discardable_assignment(statement);
        }
        for index in discard.into_iter().rev() {
            block.stmts.remove(index);
        }
        self.bindings = outer;
    }
}
