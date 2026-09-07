use super::implementation::{collect_macro_token_refs_rec, collect_use_paths};
use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};

/// Return references from `rust_code` to names defined by a separate generated
/// support owner. Declaration identifiers and local bindings are not references.
pub(crate) fn rust_source_referenced_item_names(
    rust_code: &str,
    candidate_names: &HashSet<String>,
) -> HashSet<String> {
    let Ok(parsed) = syn::parse_file(rust_code) else {
        return HashSet::new();
    };
    let mut collector = ExternalItemRefCollector {
        candidate_names,
        refs: HashSet::new(),
    };
    collector.visit_file(&parsed);
    collector.refs
}

struct ExternalItemRefCollector<'a> {
    candidate_names: &'a HashSet<String>,
    refs: HashSet<String>,
}

impl ExternalItemRefCollector<'_> {
    fn collect_path(&mut self, path: &syn::Path) {
        self.refs.extend(
            path.segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .filter(|name| self.candidate_names.contains(name)),
        );
    }
}

impl<'ast> Visit<'ast> for ExternalItemRefCollector<'_> {
    fn visit_item_use(&mut self, item_use: &'ast syn::ItemUse) {
        let mut paths = Vec::new();
        collect_use_paths(&item_use.tree, &mut Vec::new(), &mut paths);
        for path in paths {
            self.refs.extend(
                path.into_iter()
                    .filter(|name| self.candidate_names.contains(name)),
            );
        }
        visit::visit_item_use(self, item_use);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.collect_path(path);
        visit::visit_path(self, path);
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        collect_macro_token_refs_rec(&rust_macro.tokens, &HashSet::new(), |name| {
            if self.candidate_names.contains(name) {
                self.refs.insert(name.to_string());
            }
        });
        visit::visit_macro(self, rust_macro);
    }
}

/// Return support traits that a consumer may require solely through method-call
/// syntax. Rust method resolution requires those traits to be in scope even
/// though the generated call does not contain the trait identifier.
pub(crate) fn rust_source_required_trait_names(
    consumer_source: &str,
    support_source: &str,
) -> Result<HashSet<String>, String> {
    let consumer = syn::parse_file(consumer_source)
        .map_err(|error| format!("failed to parse generated support consumer: {error}"))?;
    let support = syn::parse_file(support_source).map_err(|error| {
        format!("failed to parse generated support while collecting traits: {error}")
    })?;
    let mut method_traits = HashMap::<String, HashSet<String>>::new();
    for item in &support.items {
        match item {
            syn::Item::Trait(trait_) => {
                let trait_name = trait_.ident.to_string();
                for item in &trait_.items {
                    if let syn::TraitItem::Fn(method) = item {
                        method_traits
                            .entry(method.sig.ident.to_string())
                            .or_default()
                            .insert(trait_name.clone());
                    }
                }
            }
            syn::Item::Mod(module) => {
                if let Some(trait_name) = nested_trait_name(module) {
                    return Err(format!(
                        "generated support trait `{trait_name}` is nested in module `{}`; support traits must be top-level items",
                        module.ident
                    ));
                }
            }
            _ => {}
        }
    }
    if method_traits.is_empty() {
        return Ok(HashSet::new());
    }
    let mut collector = MethodCallCollector {
        method_traits: &method_traits,
        required_traits: HashSet::new(),
    };
    collector.visit_file(&consumer);
    Ok(collector.required_traits)
}

fn nested_trait_name(module: &syn::ItemMod) -> Option<String> {
    let (_, items) = module.content.as_ref()?;
    for item in items {
        match item {
            syn::Item::Trait(trait_) => return Some(trait_.ident.to_string()),
            syn::Item::Mod(nested) => {
                if let Some(name) = nested_trait_name(nested) {
                    return Some(name);
                }
            }
            _ => {}
        }
    }
    None
}

struct MethodCallCollector<'a> {
    method_traits: &'a HashMap<String, HashSet<String>>,
    required_traits: HashSet<String>,
}

impl<'ast> Visit<'ast> for MethodCallCollector<'_> {
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if let Some(traits) = self.method_traits.get(&call.method.to_string()) {
            self.required_traits.extend(traits.iter().cloned());
        }
        visit::visit_expr_method_call(self, call);
    }
}

#[cfg(test)]
mod tests {
    use super::{rust_source_referenced_item_names, rust_source_required_trait_names};
    use std::collections::HashSet;

    #[test]
    fn local_name_collisions_retain_support_conservatively() {
        let candidates = HashSet::from(["support_value".to_string()]);
        let referenced = rust_source_referenced_item_names(
            "fn run() { let support_value = 1; consume(support_value); }",
            &candidates,
        );

        assert_eq!(referenced, candidates);
    }

    #[test]
    fn method_calls_require_matching_support_traits() {
        let required = rust_source_required_trait_names(
            "fn run(value: Wrapper) { value.render(); }",
            "trait RenderSupport { fn render(&self); }",
        )
        .expect("flat support traits should be accepted");

        assert_eq!(required, HashSet::from(["RenderSupport".to_string()]));
    }

    #[test]
    fn nested_support_traits_violate_the_flat_owner_invariant() {
        let error = rust_source_required_trait_names(
            "fn run(value: Wrapper) { value.render(); }",
            "mod nested { trait RenderSupport { fn render(&self); } }",
        )
        .expect_err("nested support traits must fail closed");

        assert!(error.contains("support traits must be top-level items"));
        assert!(error.contains("RenderSupport"));
    }
}
