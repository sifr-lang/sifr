use super::*;

#[test]
fn stable_length_alias_scopes_a_checked_index_guard() {
    let generated = generate_rust_from_source(
        r#"
def first(values: list[int]) -> int:
    size = len(values)
    if size > 0:
        value = values[0]
        return value
    return 0
"#,
    );
    assert!(
        generated.contains("if size > SifrInt::from_i64(0)"),
        "{generated}"
    );
    assert!(
        generated.contains("if let Some(__sifr_checked_value_0)"),
        "{generated}"
    );
    assert!(!generated.contains("compile_error!"), "{generated}");
    assert!(!generated.contains("values.as_slice()["), "{generated}");
}

#[test]
fn length_alias_proof_expires_on_mutation_and_rebinding() {
    fn proof_at_last_if(source: &str) -> bool {
        let parsed = sifr_python_parser::parse_module(source).expect("parse failed");
        let lowering = sifr_lowering::lower_module(parsed.suite()).expect("lowering failed");
        let function = &lowering.module.functions[0];
        let (analysis, _) =
            crate::body_analysis::BodyAnalysis::build(function, &Default::default());
        let condition = function
            .body
            .iter()
            .rev()
            .find_map(|stmt| match stmt {
                crate::HirStmt::If { condition, .. } => Some(condition),
                _ => None,
            })
            .expect("missing if");
        analysis
            .stable_length_aliases(condition)
            .contains_key("size")
    }

    assert!(proof_at_last_if(
        "def probe(values: list[int]) -> int | None:
    size = len(values)
    if size > 0:
        return values[0]
    return None
"
    ));
    for source in [
        "def probe(mut values: list[int]) -> int | None:
    size = len(values)
    values.clear()
    if size > 0:
        return values[0]
    return None
",
        "def probe(values: list[int]) -> int | None:
    size = len(values)
    size = 1
    if size > 0:
        return values[0]
    return None
",
        "def probe(mut values: list[int]) -> int | None:
    size = len(values)
    values = []
    if size > 0:
        return values[0]
    return None
",
        "def probe(values: list[int]) -> int | None:
    size = len(values)
    for size in range(1):
        pass
    if size > 0:
        return values[0]
    return None
",
    ] {
        assert!(!proof_at_last_if(source), "{source}");
    }
}

#[test]
fn loop_carried_collection_mutation_expires_length_alias_proof() {
    let parsed = sifr_python_parser::parse_module(
        "def probe(mut values: list[int]) -> int | None:
    size = len(values)
    while size > 0:
        values.clear()
        break
    return None
",
    )
    .expect("parse failed");
    let lowering = sifr_lowering::lower_module(parsed.suite()).expect("lowering failed");
    let function = &lowering.module.functions[0];
    let (analysis, _) = crate::body_analysis::BodyAnalysis::build(function, &Default::default());
    let condition = function
        .body
        .iter()
        .find_map(|stmt| match stmt {
            crate::HirStmt::While { condition, .. } => Some(condition),
            _ => None,
        })
        .expect("missing while");
    assert!(
        !analysis
            .stable_length_aliases(condition)
            .contains_key("size")
    );
}
