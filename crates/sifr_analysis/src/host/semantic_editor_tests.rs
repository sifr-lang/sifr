use super::*;
use crate::{FrontendInput, SourceText, TextPosition};
use sifr_frontend::{FrontendMode, SourcePath};

fn single_file_input(source: &str) -> FrontendInput {
    FrontendInput {
        path: SourcePath::new("main.sifr"),
        source: SourceText::new(source),
        mode: FrontendMode::SingleFile,
    }
}

#[test]
fn hover_returns_semantic_function_and_binding_details() {
    let source = "\
def generate_random() -> int | None:
    x: int = 1
    return x

def main() -> int:
    y = generate_random()
    return 0
";
    let mut host =
        AnalysisHost::open_single_file(single_file_input(source)).expect("host should load");
    let file = host.files()[0];

    let function_hover = host
        .hover(
            file,
            &TextPosition {
                line: 5,
                character: 8,
            },
        )
        .expect("hover should query")
        .into_value()
        .expect("function call hover should exist");
    assert!(function_hover.contents.contains("generate_random"));
    assert!(function_hover.contents.contains("->"));
    assert!(function_hover.contents.contains("int"));
    assert!(function_hover.contents.contains("None"));
    assert!(!function_hover.contents.contains("(Name)"));

    let binding_hover = host
        .hover(
            file,
            &TextPosition {
                line: 5,
                character: 4,
            },
        )
        .expect("hover should query")
        .into_value()
        .expect("binding hover should exist");
    assert!(binding_hover.contents.contains("y:"));
    assert!(binding_hover.contents.contains("int"));
    assert!(binding_hover.contents.contains("None"));
}

#[test]
fn signature_help_returns_parameter_labels_and_active_parameter() {
    let source = "\
def combine(left: int, right: int) -> int:
    return left + right

def main() -> int:
    return combine(1, 2)
";
    let mut host =
        AnalysisHost::open_single_file(single_file_input(source)).expect("host should load");
    let file = host.files()[0];
    let help = host
        .signature_help(
            file,
            &TextPosition {
                line: 4,
                character: 22,
            },
        )
        .expect("signature help should query")
        .into_value()
        .expect("signature help should exist");

    assert_eq!(help.parameters, vec!["left: int", "right: int"]);
    assert_eq!(help.active_parameter, Some(1));
    assert!(help.label.contains("combine"));
    assert!(help.label.contains("-> int"));
    assert!(!help.label.contains("..."));
}

#[test]
fn hover_and_signature_cover_stdlib_calls_inside_try_blocks() {
    let source = "\
from sifr.random import randint

def generate_random() -> int | None:
    try:
        x: int = randint(0, 100)
        return x
    except ValueError:
        return None
";
    let mut host =
        AnalysisHost::open_single_file(single_file_input(source)).expect("host should load");
    let file = host.files()[0];

    let call_hover = host
        .hover(
            file,
            &TextPosition {
                line: 4,
                character: 17,
            },
        )
        .expect("hover should query")
        .into_value()
        .expect("stdlib call hover should exist");
    assert!(call_hover.contents.contains("randint"));
    assert!(call_hover.contents.contains("minimum: int"));
    assert!(call_hover.contents.contains("maximum: int"));
    assert!(call_hover.contents.contains("Result[int, ValueError]"));

    let help = host
        .signature_help(
            file,
            &TextPosition {
                line: 4,
                character: 27,
            },
        )
        .expect("signature help should query")
        .into_value()
        .expect("stdlib signature help should exist");
    assert_eq!(help.parameters, vec!["minimum: int", "maximum: int"]);
    assert_eq!(help.active_parameter, Some(1));
}

#[test]
fn hover_returns_none_for_non_semantic_positions() {
    let source = "\
def main() -> int:
    word: str = \"main\"
    # main
    return 0
";
    let mut host =
        AnalysisHost::open_single_file(single_file_input(source)).expect("host should load");
    let file = host.files()[0];

    for position in [
        TextPosition {
            line: 0,
            character: 0,
        },
        TextPosition {
            line: 1,
            character: 17,
        },
        TextPosition {
            line: 2,
            character: 6,
        },
    ] {
        assert_eq!(
            host.hover(file, &position)
                .expect("hover should query")
                .into_value(),
            None
        );
    }
}
