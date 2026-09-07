fn public_visibility() -> syn::Visibility {
    syn::Visibility::Public(syn::token::Pub::default())
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

pub(crate) fn crate_visible_generated_support_source(source: &str, consumers: &[&str]) -> String {
    let names = crate::stdlib_filter::rust_source_defined_item_names(source);
    let mut external_names = consumers
        .iter()
        .flat_map(|consumer| {
            crate::stdlib_filter::rust_source_referenced_item_names(consumer, &names)
        })
        .collect::<std::collections::HashSet<_>>();
    for consumer in consumers {
        external_names.extend(
            crate::stdlib_filter::rust_source_required_trait_names(consumer, source)
                .unwrap_or_else(|error| {
                    panic!("invalid generated support trait visibility: {error}")
                }),
        );
    }
    let mut file = syn::parse_file(source).unwrap_or_else(|error| {
        panic!("failed to parse generated support for crate visibility: {error}")
    });
    extend_visible_interface_types(&file, &names, &mut external_names);
    // The owner module remains private. Public declarations inside it grant the
    // necessary parent/sibling access without pretending this is an exported API
    // or redundantly spelling the effective crate boundary on every declaration.
    let visibility_for = |name: &syn::Ident| {
        if external_names.contains(&name.to_string()) {
            public_visibility()
        } else {
            syn::Visibility::Inherited
        }
    };
    for item in &mut file.items {
        match item {
            syn::Item::Const(item) => item.vis = visibility_for(&item.ident),
            syn::Item::Enum(item) => item.vis = visibility_for(&item.ident),
            syn::Item::Fn(item) => {
                item.vis = visibility_for(&item.sig.ident);
            }
            syn::Item::Impl(item) if item.trait_.is_none() => {
                let visibility = match item.self_ty.as_ref() {
                    syn::Type::Path(path) => path
                        .path
                        .segments
                        .last()
                        .map_or(syn::Visibility::Inherited, |segment| {
                            visibility_for(&segment.ident)
                        }),
                    _ => syn::Visibility::Inherited,
                };
                set_impl_function_visibility(&mut item.items, &visibility);
            }
            syn::Item::Static(item) => item.vis = visibility_for(&item.ident),
            syn::Item::Struct(item) => {
                let visibility = visibility_for(&item.ident);
                item.vis = visibility.clone();
                set_struct_field_visibility(&mut item.fields, &visibility);
            }
            syn::Item::Trait(item) => item.vis = visibility_for(&item.ident),
            syn::Item::Type(item) => item.vis = visibility_for(&item.ident),
            syn::Item::Union(item) => {
                let visibility = visibility_for(&item.ident);
                item.vis = visibility.clone();
                for field in &mut item.fields.named {
                    field.vis = visibility.clone();
                }
            }
            syn::Item::Use(item) => item.vis = syn::Visibility::Inherited,
            syn::Item::Macro(item) => {
                if let Some(mut declarations) = crate::task_local_support::declarations(&item.mac) {
                    for declaration in &mut declarations.0 {
                        declaration.visibility = visibility_for(&declaration.name);
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

fn extend_visible_interface_types(
    file: &syn::File,
    names: &std::collections::HashSet<String>,
    visible: &mut std::collections::HashSet<String>,
) {
    use syn::visit::{self, Visit};
    struct Interface<'a> {
        names: &'a std::collections::HashSet<String>,
        referenced: std::collections::HashSet<String>,
    }
    impl<'ast> Visit<'ast> for Interface<'_> {
        fn visit_block(&mut self, _: &'ast syn::Block) {}
        fn visit_expr(&mut self, _: &'ast syn::Expr) {}
        fn visit_path(&mut self, path: &'ast syn::Path) {
            self.referenced.extend(
                path.segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .filter(|name| self.names.contains(name)),
            );
            visit::visit_path(self, path);
        }
    }
    loop {
        let mut interface = Interface {
            names,
            referenced: Default::default(),
        };
        for item in &file.items {
            let name = match item {
                syn::Item::Fn(item) => Some(&item.sig.ident),
                syn::Item::Struct(item) => Some(&item.ident),
                syn::Item::Enum(item) => Some(&item.ident),
                syn::Item::Type(item) => Some(&item.ident),
                syn::Item::Trait(item) => Some(&item.ident),
                syn::Item::Union(item) => Some(&item.ident),
                syn::Item::Const(item) => Some(&item.ident),
                syn::Item::Static(item) => Some(&item.ident),
                syn::Item::Impl(item) if item.trait_.is_none() => match item.self_ty.as_ref() {
                    syn::Type::Path(path) => {
                        path.path.segments.last().map(|segment| &segment.ident)
                    }
                    _ => None,
                },
                syn::Item::Macro(item) => {
                    if let Some(declarations) = crate::task_local_support::declarations(&item.mac) {
                        for declaration in &declarations.0 {
                            if visible.contains(&declaration.name.to_string()) {
                                interface.visit_type(&declaration.ty);
                            }
                        }
                    }
                    None
                }
                _ => None,
            };
            if name.is_some_and(|name| visible.contains(&name.to_string())) {
                interface.visit_item(item);
            }
        }
        let before = visible.len();
        visible.extend(interface.referenced);
        if visible.len() == before {
            break;
        }
    }
}

pub(crate) fn generated_support_import(source: &str, support: &str) -> String {
    let names = crate::stdlib_filter::rust_source_defined_item_names(support);
    let mut required = crate::stdlib_filter::rust_source_unqualified_item_names(source, &names)
        .unwrap_or_else(|error| panic!("invalid generated support import: {error}"));
    let mut scope = syn::parse_file(source)
        .unwrap_or_else(|error| panic!("invalid generated support import scope: {error}"));
    scope
        .items
        .retain(|item| !matches!(item, syn::Item::Mod(_)));
    required.extend(
        crate::stdlib_filter::rust_source_required_trait_names(
            &prettyplease::unparse(&scope),
            support,
        )
        .unwrap_or_else(|error| panic!("invalid generated support trait import: {error}")),
    );
    let mut required = required.into_iter().collect::<Vec<_>>();
    required.sort();
    if required.is_empty() {
        String::new()
    } else {
        format!(
            "use crate::__sifr_generated_support::{{{}}};",
            required.join(", ")
        )
    }
}
