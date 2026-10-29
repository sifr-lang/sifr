use super::*;

fn source(include_cleanup_evidence: bool) -> String {
    let items: [RustItem; 1] = build_secondary_error_type_items(include_cleanup_evidence)
        .try_into()
        .expect("one secondary error support item");
    let [RustItem::Attr(source)] = items else {
        panic!("secondary error support must be one source item");
    };
    source
}

#[test]
fn synchronous_secondary_error_has_no_async_runtime_reference() {
    let source = source(false);
    syn::parse_file(&source).expect("valid standalone error items");
    assert!(!source.contains("sifr_runtime"));
    assert!(!source.contains("from_async_cleanup"));
    assert!(source.contains("fn new(message: String)"));
    for field in [
        "message",
        "kind",
        "location",
        "resource",
        "operation",
        "budget_millis",
    ] {
        assert!(source.contains(&format!("    {field}:")));
    }
}

#[test]
fn async_conversion_stays_in_the_secondary_error_constructor_impl() {
    let source = source(true);
    let file = syn::parse_file(&source).expect("valid async error items");
    let inherent = file
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Impl(item) if item.trait_.is_none() => Some(item),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(inherent.len(), 1);
    let methods = inherent[0]
        .items
        .iter()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(item) => Some(item.sig.ident.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(methods, ["new", "from_async_cleanup"]);
    assert!(source.contains("AsyncCleanupEvidence::CleanupFailed"));
    assert!(source.contains("AsyncCleanupEvidence::CleanupTimedOut"));
}
