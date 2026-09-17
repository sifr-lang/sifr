#[doc(hidden)]
pub fn compiled_input_tokens() -> Vec<(&'static str, &'static str)> {
    static TOKENS: std::sync::OnceLock<Vec<(&'static str, &'static str)>> =
        std::sync::OnceLock::new();
    TOKENS
        .get_or_init(|| {
            let mut tokens = vec![
                ("sifr_lsp", env!("SIFR_LOCAL_INPUT_TOKEN")),
                (
                    "sifr_lsp:cfg-test",
                    if cfg!(test) { "true" } else { "false" },
                ),
            ];
            tokens.extend(sifr_analysis::compiled_input_tokens());
            tokens.extend(sifr_diagnostics::compiled_input_tokens());
            tokens.extend(sifr_driver::compiled_input_tokens());
            tokens.extend(sifr_package::compiled_input_tokens());
            tokens.extend(sifr_source::compiled_input_tokens());
            tokens.sort_unstable();
            tokens.dedup();
            tokens
        })
        .clone()
}
