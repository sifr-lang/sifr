#[doc(hidden)]
pub fn compiled_input_tokens() -> Vec<(&'static str, &'static str)> {
    static TOKENS: std::sync::OnceLock<Vec<(&'static str, &'static str)>> =
        std::sync::OnceLock::new();
    TOKENS
        .get_or_init(|| {
            let mut tokens = vec![
                ("sifr_frontend", env!("SIFR_LOCAL_INPUT_TOKEN")),
                (
                    "sifr_frontend:cfg-test",
                    if cfg!(test) { "true" } else { "false" },
                ),
            ];
            tokens.extend(sifr_compiler_component::compiled_input_tokens());
            tokens.extend(sifr_diagnostics::compiled_input_tokens());
            tokens.extend(sifr_lowering::compiled_input_tokens());
            tokens.extend(sifr_ir::compiled_input_tokens());
            tokens.extend(sifr_source::compiled_input_tokens());
            tokens.extend(sifr_syntax::compiled_input_tokens());
            tokens.extend(sifr_type_system::compiled_input_tokens());
            tokens.extend(sifr_sql_contract::compiled_input_tokens());
            tokens.sort_unstable();
            tokens.dedup();
            tokens
        })
        .clone()
}
