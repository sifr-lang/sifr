use quote::ToTokens;

// The absolute Tokio task_local macro expands only the parsed static names;
// unknown paths or grammar remain opaque to lexical standard-type proofs.
pub(super) fn task_local_static_names(rust_macro: &syn::Macro) -> Option<Vec<syn::Ident>> {
    use syn::parse::Parser;
    if rust_macro
        .path
        .to_token_stream()
        .to_string()
        .replace(' ', "")
        != "::tokio::task_local"
    {
        return None;
    }
    let parser = |input: syn::parse::ParseStream<'_>| -> syn::Result<Vec<syn::Ident>> {
        let mut names = Vec::new();
        while !input.is_empty() {
            let attributes = input.call(syn::Attribute::parse_outer)?;
            if !attributes.is_empty() {
                return Err(input.error("attributed task-local declarations remain opaque"));
            }
            let _visibility = input.parse::<syn::Visibility>()?;
            input.parse::<syn::Token![static]>()?;
            names.push(input.parse::<syn::Ident>()?);
            input.parse::<syn::Token![:]>()?;
            let _ty = input.parse::<syn::Type>()?;
            if !input.is_empty() {
                input.parse::<syn::Token![;]>()?;
            }
        }
        Ok(names)
    };
    parser.parse2(rust_macro.tokens.clone()).ok()
}
