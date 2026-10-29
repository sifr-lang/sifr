//! Const field reads require a proven built-in tuple, not an overloaded dereference.
use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};

pub(super) fn borrowed_tuples(
    signature: &syn::Signature,
    body: &syn::Block,
) -> HashMap<String, syn::Type> {
    let mut tuples = HashMap::new();
    for argument in &signature.inputs {
        if let syn::FnArg::Typed(argument) = argument
            && let syn::Pat::Ident(binding) = argument.pat.as_ref()
            && let syn::Type::Reference(reference) = argument.ty.as_ref()
            && matches!(reference.elem.as_ref(), syn::Type::Tuple(_))
        {
            tuples.insert(binding.ident.to_string(), reference.elem.as_ref().clone());
        }
    }
    // A same-named local, pattern or nested parameter invalidates this proof.
    // Keep the optimization conservative instead of attributing a shadow to the input.
    let mut bindings = Bindings(HashSet::new());
    bindings.visit_block(body);
    tuples.retain(|name, _| !bindings.0.contains(name));
    tuples
}

pub(super) fn field_is_builtin(
    expression: &syn::ExprField,
    tuples: &HashMap<String, syn::Type>,
) -> bool {
    fn field_type<'a>(
        expression: &syn::Expr,
        tuples: &'a HashMap<String, syn::Type>,
    ) -> Option<&'a syn::Type> {
        match expression {
            syn::Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => {
                tuples.get(&path.path.segments[0].ident.to_string())
            }
            syn::Expr::Paren(paren) => field_type(&paren.expr, tuples),
            syn::Expr::Field(field) => {
                let syn::Type::Tuple(tuple) = field_type(&field.base, tuples)? else {
                    return None;
                };
                let syn::Member::Unnamed(index) = &field.member else {
                    return None;
                };
                tuple.elems.iter().nth(usize::try_from(index.index).ok()?)
            }
            _ => None,
        }
    }
    let Some(syn::Type::Tuple(tuple)) = field_type(&expression.base, tuples) else {
        return false;
    };
    matches!(&expression.member, syn::Member::Unnamed(index) if usize::try_from(index.index).is_ok_and(|index| index < tuple.elems.len()))
}

struct Bindings(HashSet<String>);
impl<'ast> Visit<'ast> for Bindings {
    fn visit_pat_ident(&mut self, binding: &'ast syn::PatIdent) {
        self.0.insert(binding.ident.to_string());
        visit::visit_pat_ident(self, binding);
    }
}
