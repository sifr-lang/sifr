fn public_visibility() -> syn::Visibility {
    syn::Visibility::Public(syn::token::Pub::default())
}

fn parent_visibility() -> syn::Visibility {
    syn::parse_quote!(pub(super))
}

fn set_impl_function_visibility(items: &mut [syn::ImplItem], visibility: &syn::Visibility) {
    for item in items {
        if let syn::ImplItem::Fn(function) = item
            && function.sig.ident != "__sifr_with_state"
        {
            function.vis = visibility.clone();
        }
    }
}

fn set_struct_field_visibility(fields: &mut syn::Fields, visibility: &syn::Visibility) {
    match fields {
        syn::Fields::Named(fields) => {
            for field in &mut fields.named {
                field.vis = visibility.clone();
            }
        }
        syn::Fields::Unnamed(fields) => {
            for field in &mut fields.unnamed {
                field.vis = visibility.clone();
            }
        }
        syn::Fields::Unit => {}
    }
}

fn rewrite_impl_self_paths(items: &mut [syn::Item]) {
    use syn::visit_mut::VisitMut;

    for item in items {
        match item {
            syn::Item::Impl(item_impl) => {
                let owner = match item_impl.self_ty.as_ref() {
                    syn::Type::Path(path) => {
                        path.path.segments.last().map(|part| part.ident.clone())
                    }
                    _ => None,
                };
                let Some(owner) = owner else {
                    continue;
                };
                for impl_item in &mut item_impl.items {
                    if let syn::ImplItem::Fn(method) = impl_item {
                        ImplSelfPathRewriter { owner: &owner }.visit_signature_mut(&mut method.sig);
                        ImplSelfPathRewriter { owner: &owner }.visit_block_mut(&mut method.block);
                    }
                }
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &mut module.content {
                    rewrite_impl_self_paths(nested);
                }
            }
            _ => {}
        }
    }
}

struct ImplSelfPathRewriter<'owner> {
    owner: &'owner syn::Ident,
}

impl syn::visit_mut::VisitMut for ImplSelfPathRewriter<'_> {
    fn visit_expr_path_mut(&mut self, path: &mut syn::ExprPath) {
        syn::visit_mut::visit_expr_path_mut(self, path);
        rewrite_owner_path(&mut path.path, self.owner);
    }

    fn visit_type_path_mut(&mut self, path: &mut syn::TypePath) {
        syn::visit_mut::visit_type_path_mut(self, path);
        rewrite_owner_path(&mut path.path, self.owner);
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

fn rewrite_owner_path(path: &mut syn::Path, owner: &syn::Ident) {
    if path.leading_colon.is_none()
        && let Some(first) = path.segments.first_mut()
        && first.ident == *owner
    {
        first.ident = syn::Ident::new("Self", first.ident.span());
        first.arguments = syn::PathArguments::None;
    }
}

pub(crate) fn publicize_generated_module_source(source: &str) -> String {
    let mut file = syn::parse_file(source).unwrap_or_else(|error| {
        panic!("failed to parse generated module for publicization: {error}")
    });
    rewrite_impl_self_paths(&mut file.items);
    for item in &mut file.items {
        match item {
            syn::Item::Const(item) => item.vis = public_visibility(),
            syn::Item::Enum(item) => item.vis = public_visibility(),
            syn::Item::Fn(item) => item.vis = public_visibility(),
            syn::Item::Impl(item) if item.trait_.is_none() => {
                set_impl_function_visibility(&mut item.items, &public_visibility());
            }
            syn::Item::Static(item) => item.vis = public_visibility(),
            syn::Item::Struct(item) => {
                item.vis = public_visibility();
                set_struct_field_visibility(&mut item.fields, &public_visibility());
            }
            syn::Item::Trait(item) => item.vis = public_visibility(),
            syn::Item::Type(item) => item.vis = public_visibility(),
            syn::Item::Union(item) => {
                item.vis = public_visibility();
                for field in &mut item.fields.named {
                    field.vis = public_visibility();
                }
            }
            syn::Item::Use(item) => item.vis = public_visibility(),
            _ => {}
        }
    }
    prettyplease::unparse(&file)
}

pub(crate) fn crate_visible_generated_support_source(source: &str) -> String {
    let mut file = syn::parse_file(source).unwrap_or_else(|error| {
        panic!("failed to parse generated support for crate visibility: {error}")
    });
    rewrite_impl_self_paths(&mut file.items);
    for item in &mut file.items {
        let visibility = parent_visibility();
        match item {
            syn::Item::Const(item) => item.vis = visibility,
            syn::Item::Enum(item) => item.vis = visibility,
            syn::Item::Fn(item) => item.vis = visibility,
            syn::Item::Impl(item) if item.trait_.is_none() => {
                set_impl_function_visibility(&mut item.items, &visibility);
            }
            syn::Item::Static(item) => item.vis = visibility,
            syn::Item::Struct(item) => {
                item.vis = visibility.clone();
                set_struct_field_visibility(&mut item.fields, &visibility);
            }
            syn::Item::Trait(item) => item.vis = visibility,
            syn::Item::Type(item) => item.vis = visibility,
            syn::Item::Union(item) => {
                item.vis = visibility.clone();
                for field in &mut item.fields.named {
                    field.vis = visibility.clone();
                }
            }
            syn::Item::Use(item) => item.vis = visibility,
            _ => {}
        }
    }
    prettyplease::unparse(&file)
}
