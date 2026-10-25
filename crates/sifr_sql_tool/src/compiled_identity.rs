#[doc(hidden)]
pub fn compiled_input_tokens() -> Vec<(&'static str, &'static str)> {
    static TOKENS: std::sync::OnceLock<Vec<(&'static str, &'static str)>> =
        std::sync::OnceLock::new();
    TOKENS
        .get_or_init(|| {
            let mut tokens = vec![
                ("sifr_sql_tool", env!("SIFR_LOCAL_INPUT_TOKEN")),
                (
                    "sifr_sql_tool:cfg-test",
                    if cfg!(test) { "true" } else { "false" },
                ),
            ];
            tokens.extend(sifr_ipc::compiled_input_tokens());
            tokens.extend(sifr_process::compiled_input_tokens());
            tokens.extend(sifr_sql_contract::compiled_input_tokens());
            tokens.sort_unstable();
            tokens.dedup();
            tokens
        })
        .clone()
}
