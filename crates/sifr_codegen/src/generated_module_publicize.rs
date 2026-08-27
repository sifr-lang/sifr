fn public_visibility() -> syn::Visibility {
    syn::Visibility::Public(syn::token::Pub::default())
}

fn publicize_impl_items(items: &mut [syn::ImplItem]) {
    for item in items {
        if let syn::ImplItem::Fn(function) = item {
            function.vis = public_visibility();
        }
    }
}

fn publicize_struct_fields(fields: &mut syn::Fields) {
    match fields {
        syn::Fields::Named(fields) => {
            for field in &mut fields.named {
                field.vis = public_visibility();
            }
        }
        syn::Fields::Unnamed(fields) => {
            for field in &mut fields.unnamed {
                field.vis = public_visibility();
            }
        }
        syn::Fields::Unit => {}
    }
}

pub(crate) fn publicize_generated_module_source(
    source: &str,
) -> Result<String, crate::CodegenError> {
    let mut file = syn::parse_file(source).map_err(|error| {
        crate::CodegenError::new(format!(
            "failed to parse generated module for publicization: {error}"
        ))
    })?;
    for item in &mut file.items {
        match item {
            syn::Item::Const(item) => item.vis = public_visibility(),
            syn::Item::Enum(item) => item.vis = public_visibility(),
            syn::Item::Fn(item) => item.vis = public_visibility(),
            syn::Item::Impl(item) => {
                if item.trait_.is_none() {
                    publicize_impl_items(&mut item.items);
                }
            }
            syn::Item::Static(item) => item.vis = public_visibility(),
            syn::Item::Struct(item) => {
                item.vis = public_visibility();
                publicize_struct_fields(&mut item.fields);
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
    Ok(prettyplease::unparse(&file))
}

#[cfg(test)]
mod tests {
    use super::publicize_generated_module_source;

    #[test]
    fn malformed_generated_module_publicization_returns_error() {
        let error = publicize_generated_module_source("fn broken(")
            .expect_err("malformed generated Rust must be rejected");

        assert!(
            error
                .message
                .contains("failed to parse generated module for publicization")
        );
    }
}
