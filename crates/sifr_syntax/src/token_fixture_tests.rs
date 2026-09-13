use crate::parse_module;
use serde_json::Value;
use sifr_python_ast::token::TokenKind;
use sifr_python_parser::{Mode, lexer::lex};
use std::path::Path;

#[test]
fn selected_ruff_token_fixtures_match_parser() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../verification/areas/performance/sifr_syntax_token_fixtures");
    let mut paths: Vec<_> = std::fs::read_dir(root)
        .expect("token fixture directory")
        .map(|entry| entry.expect("token fixture entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    assert!(
        paths.len() >= 5,
        "representative token fixtures are required"
    );
    for path in paths {
        let data: Value = serde_json::from_slice(&std::fs::read(&path).expect("token fixture"))
            .expect("fixture JSON");
        let source = data["source"].as_str().expect("fixture source");
        parse_module(source, None)
            .unwrap_or_else(|errors| panic!("{}: {errors:?}", path.display()));
        // Fixtures describe the lexer stream, including its EOF sentinel.
        let mut lexer = lex(source, Mode::Module);
        let mut actual = Vec::new();
        loop {
            let kind = lexer.next_token();
            actual.push(format!("{kind:?}"));
            if kind == TokenKind::EndOfFile {
                break;
            }
        }
        let expected: Vec<_> = data["expected_token_kinds"]
            .as_array()
            .expect("expected token kinds")
            .iter()
            .map(|kind| kind.as_str().expect("token kind"))
            .collect();
        assert_eq!(actual, expected, "{}", path.display());
    }
}
