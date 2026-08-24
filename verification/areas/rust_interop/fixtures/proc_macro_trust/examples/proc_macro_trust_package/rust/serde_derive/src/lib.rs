extern crate proc_macro;

use proc_macro::TokenStream;
use std::path::Path;

#[proc_macro_derive(SifrGenerated)]
pub fn sifr_generated(input: TokenStream) -> TokenStream {
    let wrapper_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    if wrapper_root.join("ARM_PROC_MACRO_SENTINEL").is_file() {
        let sentinel = wrapper_root.join("PROC_MACRO_EXECUTED");
        if std::fs::write(
            sentinel,
            "serde_derive=1.0.229;upstream=compiled;sifr_wrapper_macro=executed",
        )
        .is_err()
        {
            return compile_error("proc-macro sentinel write failed");
        }
    }

    let input = input.to_string();
    let Some(type_name) = declared_type_name(&input) else {
        return compile_error("SifrGenerated requires a struct or enum");
    };
    let output = format!(
        "impl {type_name} {{ pub fn sifr_proc_macro_marker() -> &'static str {{ \"serde_derive=1.0.229;upstream=compiled;sifr_wrapper_macro=executed\" }} }}"
    );
    output
        .parse()
        .unwrap_or_else(|_| compile_error("SifrGenerated output parsing failed"))
}

fn declared_type_name(input: &str) -> Option<&str> {
    let mut tokens = input.split_whitespace();
    while let Some(token) = tokens.next() {
        if token == "struct" || token == "enum" {
            return tokens.next().map(|name| {
                name.trim_matches(|character: char| {
                    !character.is_ascii_alphanumeric() && character != '_'
                })
            });
        }
    }
    None
}

fn compile_error(message: &str) -> TokenStream {
    format!("compile_error!({message:?});")
        .parse()
        .unwrap_or_default()
}
