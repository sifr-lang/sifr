    use super::*;

    #[test]
    fn integer_float_power_uses_powi() {
        let mut expression: syn::Expr = syn::parse_quote!(2_f64.powf(10_f64));
        assert!(rewrite_integer_float_power(&mut expression));
        assert_eq!(
            expression.to_token_stream().to_string(),
            "2_f64 . powi (10)"
        );
    }
