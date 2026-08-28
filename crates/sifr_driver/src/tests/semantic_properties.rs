use crate::{CompileResult, compile};

#[test]
fn semantic_property_codegen_is_deterministic() {
    let values = [0_i64, 1, 7, 42, 1_000, i32::MAX.into()];
    for value in values {
        let source = format!(
            "def identity(value: int) -> int:\n    return value\n\ndef main():\n    result: int = identity({value})\n    print(str(result))\n"
        );
        let first = compile(&source);
        let second = compile(&source);
        match (first, second) {
            (
                CompileResult::Success {
                    rust_source: first_source,
                },
                CompileResult::Success {
                    rust_source: second_source,
                },
            ) => assert_eq!(first_source, second_source),
            (CompileResult::Errors { errors: first }, CompileResult::Errors { errors: second }) => {
                assert_eq!(first, second);
            }
            _ => panic!("the same source changed compilation outcome"),
        }
    }
}
