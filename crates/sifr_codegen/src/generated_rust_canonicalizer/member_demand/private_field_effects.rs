use std::collections::{HashMap, HashSet};

use syn::visit::{self, Visit};

pub(super) fn type_has_trivial_drop(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Array(array) => type_has_trivial_drop(&array.elem),
        syn::Type::Never(_) | syn::Type::Ptr(_) | syn::Type::Reference(_) => true,
        syn::Type::Group(group) => type_has_trivial_drop(&group.elem),
        syn::Type::Paren(paren) => type_has_trivial_drop(&paren.elem),
        syn::Type::Tuple(tuple) => tuple.elems.iter().all(type_has_trivial_drop),
        syn::Type::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => {
            path.path.segments.first().is_some_and(|segment| {
                matches!(
                    segment.ident.to_string().as_str(),
                    "bool"
                        | "char"
                        | "f32"
                        | "f64"
                        | "i8"
                        | "i16"
                        | "i32"
                        | "i64"
                        | "i128"
                        | "isize"
                        | "u8"
                        | "u16"
                        | "u32"
                        | "u64"
                        | "u128"
                        | "usize"
                )
            })
        }
        _ => false,
    }
}

pub(super) fn retain_effectful_initializers(
    items: &[syn::Item],
    candidates: &mut HashMap<String, HashSet<String>>,
) {
    let mut effects = FieldInitializerEffectCollector {
        candidates,
        unsafe_to_remove: HashSet::new(),
        impl_owner: None,
    };
    for item in items
        .iter()
        .filter(|item| !matches!(item, syn::Item::Mod(_)))
    {
        effects.visit_item(item);
    }
    for (owner, field) in effects.unsafe_to_remove {
        if let Some(owner_candidates) = candidates.get_mut(&owner) {
            owner_candidates.remove(&field);
        }
    }
    candidates.retain(|_, fields| !fields.is_empty());
}

struct FieldInitializerEffectCollector<'candidates> {
    candidates: &'candidates HashMap<String, HashSet<String>>,
    unsafe_to_remove: HashSet<(String, String)>,
    impl_owner: Option<String>,
}

impl Visit<'_> for FieldInitializerEffectCollector<'_> {
    fn visit_item_impl(&mut self, implementation: &syn::ItemImpl) {
        let previous = self
            .impl_owner
            .replace(super::type_name(&implementation.self_ty));
        visit::visit_item_impl(self, implementation);
        self.impl_owner = previous;
    }

    fn visit_expr_struct(&mut self, expression: &syn::ExprStruct) {
        let owner = expression.path.segments.last().map(|segment| {
            if segment.ident == "Self" {
                self.impl_owner
                    .clone()
                    .unwrap_or_else(|| "Self".to_string())
            } else {
                segment.ident.to_string()
            }
        });
        let Some((owner, candidates)) = owner
            .as_ref()
            .and_then(|owner| self.candidates.get(owner).map(|fields| (owner, fields)))
        else {
            visit::visit_expr_struct(self, expression);
            return;
        };
        if expression.rest.is_some() {
            self.unsafe_to_remove.extend(
                candidates
                    .iter()
                    .cloned()
                    .map(|field| (owner.clone(), field)),
            );
        }
        for field in &expression.fields {
            let syn::Member::Named(name) = &field.member else {
                continue;
            };
            let name = name.to_string();
            if candidates.contains(&name) && !field_initializer_is_discardable(&field.expr) {
                self.unsafe_to_remove.insert((owner.clone(), name));
            }
        }
        visit::visit_expr_struct(self, expression);
    }
}

fn field_initializer_is_discardable(expression: &syn::Expr) -> bool {
    crate::discardability::syntax_expression_is_discardable(expression)
        || matches!(expression, syn::Expr::Path(path) if path.qself.is_none())
}
