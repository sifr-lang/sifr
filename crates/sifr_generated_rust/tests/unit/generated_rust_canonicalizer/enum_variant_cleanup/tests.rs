use super::canonicalize_local_enum_variants;
use quote::ToTokens;

#[test]
fn same_named_nested_enums_do_not_share_an_unqualified_variant_map() {
    let mut file = syn::parse_file(
        r#"
                mod left {
                    #[derive(Debug)]
                    enum State { NOT_READY }
                    fn state() -> State { State::NOT_READY }
                }
                mod right {
                    #[derive(Debug)]
                    enum State { NOT_READY, READY }
                    fn state() -> State { State::READY }
                }
            "#,
    )
    .expect("test source should parse");

    canonicalize_local_enum_variants(&mut file);
    let canonical = file.to_token_stream().to_string();

    assert_eq!(canonical.matches("enum State").count(), 2, "{canonical}");
    assert!(canonical.contains("State :: NOT_READY"), "{canonical}");
    assert!(canonical.contains("State :: READY"), "{canonical}");
    assert!(
        !canonical.contains("impl :: std :: fmt :: Debug"),
        "{canonical}"
    );
}
