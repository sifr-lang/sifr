use super::{api_cleanup, syntax_cleanup};

#[cfg(test)]
pub(crate) fn rewrite_project_borrowed_string_literals(
    sources: &mut [&mut String],
) -> Result<(), String> {
    let mut named = sources
        .iter_mut()
        .map(|source| ("", &mut **source))
        .collect::<Vec<_>>();
    rewrite_named_project_borrows(&mut named)
}

pub(crate) fn rewrite_named_project_borrows(
    sources: &mut [(&str, &mut String)],
) -> Result<(), String> {
    let mut files = sources
        .iter()
        .map(|(name, source)| {
            let mut file = syn::parse_file(source).map_err(|error| {
                format!("failed to parse assembled generated project Rust: {error}")
            })?;
            if !name.is_empty() {
                for segment in name.split('.').rev() {
                    let ident = syn::parse_str::<syn::Ident>(segment).map_err(|error| {
                        format!("invalid generated module identity {segment}: {error}")
                    })?;
                    let items = std::mem::take(&mut file.items);
                    file.items = vec![syn::parse_quote!(mod #ident { #(#items)* })];
                }
            }
            Ok::<_, String>(file)
        })
        .collect::<Result<Vec<_>, _>>()?;
    for file in &mut files {
        syntax_cleanup::apply_local_scalar_borrow_plans(file);
    }
    let shared_slices = api_cleanup::collect_project_shared_slice_params(&files);
    let scalar_borrows = syntax_cleanup::collect_project_scalar_borrow_plans(&files);
    let mutability_facts = syntax_cleanup::collect_project_mutability_facts(&files);
    for file in &mut files {
        api_cleanup::rewrite_project_shared_slice_calls(file, &shared_slices);
        syntax_cleanup::apply_project_scalar_borrow_plans(file, &scalar_borrows);
        syntax_cleanup::apply_project_mutability_facts(file, &mutability_facts);
    }
    let signatures = syntax_cleanup::collect_project_borrowed_string_params(&files);
    for file in &mut files {
        syntax_cleanup::rewrite_project_borrowed_string_literals(file, &signatures);
        syntax_cleanup::apply_lexical_type_cleanup(file);
    }
    for ((name, source), mut file) in sources.iter_mut().zip(files) {
        if !name.is_empty() {
            for _ in name.split('.') {
                let Some(syn::Item::Mod(module)) = file.items.pop() else {
                    return Err(
                        "generated project borrow analysis lost its module boundary".to_string()
                    );
                };
                let Some((_, items)) = module.content else {
                    return Err(
                        "generated project borrow analysis lost its module body".to_string()
                    );
                };
                file.items = items;
            }
        }
        **source = prettyplease::unparse(&file);
    }
    Ok(())
}
