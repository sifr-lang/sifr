use super::*;

#[test]
fn collects_value_width_and_precision_captures() {
    let rust_macro: syn::Macro = syn::parse_quote!(println!(
        "{{escaped}} {value:>width$.precision$} {:fallback_width$}"
    ));
    assert_eq!(
        names(&rust_macro),
        HashSet::from([
            "fallback_width".to_string(),
            "precision".to_string(),
            "value".to_string(),
            "width".to_string(),
        ])
    );

    let format_args: syn::Macro = syn::parse_quote!(format_args!("{value:width$}"));
    assert_eq!(
        names(&format_args),
        HashSet::from(["value".to_string(), "width".to_string()])
    );
}

#[test]
fn renames_every_capture_role() {
    let mut rust_macro: syn::Macro = syn::parse_quote!(println!("{value:value$.value$}"));
    assert!(rename(&mut rust_macro, "value", "renamed"));
    assert!(
        rust_macro
            .tokens
            .to_string()
            .contains("{renamed:renamed$.renamed$}")
    );
}
#[test]
fn failure_and_debug_macro_families_preserve_implicit_captures() {
    for rust_macro in [
        syn::parse_quote!(panic!("{value}")),
        syn::parse_quote!(unreachable!("{value}")),
        syn::parse_quote!(todo!("{value}")),
        syn::parse_quote!(unimplemented!("{value}")),
        syn::parse_quote!(debug_assert!(false, "{value}")),
        syn::parse_quote!(debug_assert_eq!(1, 2, "{value}")),
        syn::parse_quote!(debug_assert_ne!(1, 2, "{value}")),
    ] {
        assert_eq!(names(&rust_macro), HashSet::from(["value".to_string()]));
    }
    let named: syn::Macro = syn::parse_quote!(panic!("{value}", value = 1));
    assert!(names(&named).is_empty());
}
