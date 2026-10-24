//! Preserve one parsed item owner through sealing, path rewriting and deduplication.
use super::{
    ExternalCratePathAbsolutizer, dedup_item_key, parse_item_name, render_items, sealed_stdlib_file,
};
use quote::ToTokens;
use std::collections::{HashMap, HashSet};
use syn::Item;
use syn::visit_mut::VisitMut;

pub(crate) fn seal_and_dedup_stdlib_support(
    source: &str,
    module: &str,
    nominal_types: &HashSet<String>,
    emitted_items: &mut RustItemDeduper,
    skip_types: &HashSet<String>,
) -> String {
    // Sealing has always left the original source unchanged when its token
    // rewrite cannot be parsed. Preserve that existing behavior; the final
    // support parser still rejects malformed compiler-owned source.
    let mut parsed = sealed_stdlib_file(source, module, nominal_types).unwrap_or_else(|| {
        syn::parse_file(source).unwrap_or_else(|error| {
            panic!("failed to parse stdlib support before deduplication: {error}")
        })
    });
    ExternalCratePathAbsolutizer.visit_file_mut(&mut parsed);
    dedup_items(parsed.items, emitted_items, skip_types)
}

/// Strip top-level items from Rust source whose names are already in `emitted_items`.
/// Items that survive are added to `emitted_items` so subsequent calls can deduplicate further.
///
/// Uses composite keys to distinguish struct/fn definitions from impl blocks:
/// - `struct X` / `fn X` -> key = "X"
/// - `impl X {` -> key = "impl X"
/// - `impl Trait for X {` -> key = "impl Trait for X"
///
/// The `skip_types` set contains type names (e.g., "`IOError`") for which ALL items
/// (struct, impl, trait impls) should be unconditionally stripped.
#[derive(Default)]
pub(crate) struct RustItemDeduper {
    fingerprints: HashMap<String, String>,
}

#[cfg(test)]
pub(crate) fn dedup_rust_items(
    rust_code: &str,
    emitted_items: &mut RustItemDeduper,
    skip_types: &HashSet<String>,
) -> String {
    let parsed = syn::parse_file(rust_code).unwrap_or_else(|error| {
        panic!("failed to parse stdlib support before deduplication: {error}")
    });

    dedup_items(parsed.items, emitted_items, skip_types)
}

fn dedup_items(
    items: Vec<Item>,
    emitted_items: &mut RustItemDeduper,
    skip_types: &HashSet<String>,
) -> String {
    let mut kept_items: Vec<Item> = Vec::new();
    for item in items {
        if let Some(name) = parse_item_name(&item) {
            if skip_types.contains(&name) {
                continue;
            }

            let dedup_key = dedup_item_key(&item);
            let fingerprint = item.to_token_stream().to_string();
            if let Some(previous) = emitted_items.fingerprints.get(&dedup_key) {
                if previous != &fingerprint {
                    assert!(
                        is_redundant_forwarding_adapter(&item, previous),
                        "conflicting generated support bodies share canonical key `{dedup_key}`"
                    );
                }
            } else {
                emitted_items.fingerprints.insert(dedup_key, fingerprint);
                kept_items.push(item);
            }
            continue;
        }
        let fingerprint = item.to_token_stream().to_string();
        let dedup_key = format!("unnamed:{fingerprint}");
        if emitted_items
            .fingerprints
            .insert(dedup_key, fingerprint)
            .is_none()
        {
            kept_items.push(item);
        }
    }

    render_items(&kept_items)
}

fn is_redundant_forwarding_adapter(item: &Item, previous: &str) -> bool {
    let Item::Fn(function) = item else {
        return false;
    };
    let Ok(Item::Fn(previous)) = syn::parse_str::<Item>(previous) else {
        return false;
    };
    if function.sig.to_token_stream().to_string() != previous.sig.to_token_stream().to_string() {
        return false;
    }
    let parameters = function
        .sig
        .inputs
        .iter()
        .map(|argument| match argument {
            syn::FnArg::Typed(argument) => match argument.pat.as_ref() {
                syn::Pat::Ident(ident) if ident.subpat.is_none() => Some(&ident.ident),
                _ => None,
            },
            syn::FnArg::Receiver(_) => None,
        })
        .collect::<Option<Vec<_>>>();
    let Some(parameters) = parameters else {
        return false;
    };
    let [syn::Stmt::Expr(expression, _)] = function.block.stmts.as_slice() else {
        return false;
    };
    let expression = match expression {
        syn::Expr::Return(return_expr) => return_expr.expr.as_deref(),
        expression => Some(expression),
    };
    let Some(syn::Expr::Call(call)) = expression else {
        return false;
    };
    let syn::Expr::Path(callee) = call.func.as_ref() else {
        return false;
    };
    if callee.qself.is_some()
        || callee.path.leading_colon.is_some()
        || callee.path.segments.len() != 1
        || call.args.len() != parameters.len()
    {
        return false;
    }
    call.args
        .iter()
        .zip(parameters)
        .all(|(argument, parameter)| match argument {
            syn::Expr::Path(path) => {
                path.qself.is_none()
                    && path.path.leading_colon.is_none()
                    && path.path.segments.len() == 1
                    && path.path.segments[0].ident == *parameter
            }
            _ => false,
        })
}
