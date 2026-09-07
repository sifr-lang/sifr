fn public_visibility() -> syn::Visibility {
    syn::Visibility::Public(syn::token::Pub::default())
}

fn crate_visibility() -> syn::Visibility {
    syn::parse_quote!(pub(crate))
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

pub(crate) fn publicize_generated_module_source(source: &str) -> String {
    let mut file = syn::parse_file(source).unwrap_or_else(|error| {
        panic!("failed to parse generated module for publicization: {error}")
    });
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
    for item in &mut file.items {
        let visibility = crate_visibility();
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
            syn::Item::Macro(item) => {
                if let Some(mut declarations) = crate::task_local_support::declarations(&item.mac) {
                    for declaration in &mut declarations.0 {
                        declaration.visibility = visibility.clone();
                    }
                    let declarations = declarations.0;
                    item.mac.tokens = quote::quote!(#(#declarations)*);
                }
            }
            _ => {}
        }
    }
    prettyplease::unparse(&file)
}
