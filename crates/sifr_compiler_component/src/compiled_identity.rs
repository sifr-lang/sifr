#[doc(hidden)]
pub fn compiled_input_tokens() -> Vec<(&'static str, &'static str)> {
    static TOKENS: std::sync::OnceLock<Vec<(&'static str, &'static str)>> =
        std::sync::OnceLock::new();
    TOKENS
        .get_or_init(|| {
            let mut tokens = vec![
                ("sifr_compiler_component", env!("SIFR_LOCAL_INPUT_TOKEN")),
                (
                    "sifr_compiler_component:host",
                    if cfg!(feature = "host") {
                        "true"
                    } else {
                        "false"
                    },
                ),
                (
                    "sifr_compiler_component:cfg-test",
                    if cfg!(test) { "true" } else { "false" },
                ),
            ];
            tokens.extend(sifr_diagnostics::compiled_input_tokens());
            tokens.sort_unstable();
            tokens.dedup();
            tokens
        })
        .clone()
}

#[cfg(test)]
mod tests {
    #[test]
    fn identity_records_selected_host_capability() {
        let tokens = super::compiled_input_tokens();
        let modes = tokens
            .iter()
            .filter(|(name, _)| *name == "sifr_compiler_component:host")
            .map(|(_, value)| *value)
            .collect::<Vec<_>>();
        assert_eq!(
            modes,
            vec![if cfg!(feature = "host") {
                "true"
            } else {
                "false"
            }]
        );
    }
}
