pub fn compiled_input_tokens() -> Vec<(&'static str, &'static str)> {
    let mut tokens = vec![
        ("sifr_semantic_services", env!("SIFR_LOCAL_INPUT_TOKEN")),
        (
            "sifr_semantic_services:cfg-test",
            if cfg!(test) { "true" } else { "false" },
        ),
    ];
    tokens.extend(sifr_diagnostics::compiled_input_tokens());
    tokens.extend(sifr_lowering::compiled_input_tokens());
    tokens.extend(sifr_ir::compiled_input_tokens());
    tokens.extend(sifr_stdlib_imports::compiled_input_tokens());
    tokens.extend(sifr_stdlib_manifest::compiled_input_tokens());
    tokens.extend(sifr_frontend::compiled_input_tokens());
    tokens.extend(sifr_syntax::compiled_input_tokens());
    tokens.extend(sifr_package::compiled_input_tokens());
    tokens.extend(sifr_compiler_component::compiled_input_tokens());
    tokens.extend(sifr_sql_contract::compiled_input_tokens());
    tokens.extend(sifr_sysroot::compiled_input_tokens());
    tokens.extend(sifr_metadata::compiled_input_tokens());
    tokens.extend(sifr_type_system::compiled_input_tokens());
    tokens.sort_unstable();
    tokens.dedup();
    tokens
}
