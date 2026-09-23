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
    loop {
        let value_borrows = syntax_cleanup::collect_project_value_borrow_plans(&files);
        if value_borrows.is_empty() {
            break;
        }
        let before = files
            .iter()
            .map(quote::ToTokens::to_token_stream)
            .map(|tokens| tokens.to_string())
            .collect::<Vec<_>>();
        for file in &mut files {
            syntax_cleanup::apply_project_value_borrow_plans(file, &value_borrows);
        }
        if files
            .iter()
            .map(quote::ToTokens::to_token_stream)
            .map(|tokens| tokens.to_string())
            .collect::<Vec<_>>()
            == before
        {
            return Err("generated project value borrow plans made no progress".to_string());
        }
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

#[cfg(test)]
mod tests {
    use super::rewrite_named_project_borrows;

    #[test]
    fn project_value_borrows_rewrite_imported_and_local_calls() {
        let mut helper = r#"
            pub trait Unwrappable { fn unwrap(&self) -> i64; }
            pub struct Carrier(i64);
            impl Unwrappable for Carrier { fn unwrap(&self) -> i64 { self.0 } }
            pub fn through(value: Box<dyn Unwrappable>) -> i64 { value.unwrap() }
            fn optional_read(value: &Option<String>) -> usize {
                value.as_ref().map_or(0, String::len)
            }
            pub fn optional(value: Option<String>) -> usize { optional_read(&value) }
            pub fn outer(value: Option<String>) -> usize { optional(value) }
            pub fn local() -> i64 { through(Box::new(Carrier(2))) }
        "#
        .to_string();
        let mut main = r#"
            use crate::helper::{through as imported_through, optional, outer};
            fn main() {
                assert_eq!(imported_through(Box::new(helper::Carrier(3))), 3);
                assert_eq!(optional(Some("abc".to_string())), 3);
                assert_eq!(outer(Some("abcd".to_string())), 4);
                assert_eq!(helper::local(), 2);
            }
        "#
        .to_string();

        rewrite_named_project_borrows(&mut [("", &mut main), ("helper", &mut helper)])
            .expect("project borrow rewrite");

        let main_compact = main
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<String>();
        let helper_compact = helper
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<String>();
        assert!(
            main_compact.contains("imported_through(&helper::Carrier(3))"),
            "{main}"
        );
        assert!(main_compact.contains("optional(&Some("), "{main}");
        assert!(main_compact.contains("outer(&Some("), "{main}");
        assert!(helper_compact.contains("through(&Carrier(2))"), "{helper}");
        assert!(
            helper_compact.contains("pubfnthrough(value:&dynUnwrappable)"),
            "{helper}"
        );
        assert!(
            helper_compact.contains("pubfnoptional(value:&Option<String>)"),
            "{helper}"
        );
        assert!(
            helper_compact.contains("pubfnouter(value:&Option<String>)"),
            "{helper}"
        );
        assert!(helper_compact.contains("optional(value)"), "{helper}");
        assert!(
            !main_compact.contains("imported_through(Box::new("),
            "{main}"
        );
    }
}
