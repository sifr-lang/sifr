use std::collections::HashSet;

pub(super) fn borrowed_parameter_names(signature: &syn::Signature) -> HashSet<String> {
    signature
        .inputs
        .iter()
        .filter_map(|argument| match argument {
            syn::FnArg::Typed(parameter)
                if matches!(parameter.ty.as_ref(), syn::Type::Reference(_)) =>
            {
                match parameter.pat.as_ref() {
                    syn::Pat::Ident(binding) => Some(binding.ident.to_string()),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}
