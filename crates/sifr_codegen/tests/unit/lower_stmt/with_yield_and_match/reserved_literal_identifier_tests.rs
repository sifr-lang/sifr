use super::*;

#[test]
fn capture_names_cannot_be_reinterpreted_as_boolean_literals() {
    for (name, expected) in [
        ("true", "sifr_source_74727565"),
        ("false", "sifr_source_66616c7365"),
    ] {
        let pattern = HirPattern::Capture {
            name: name.to_string(),
            ty: Type::Bool,
        };
        let (rendered, bindings) =
            try_lower_match_pattern(&pattern).expect("capture pattern should lower");
        assert_eq!(rendered, expected);
        assert_eq!(bindings, [name]);
    }
}
