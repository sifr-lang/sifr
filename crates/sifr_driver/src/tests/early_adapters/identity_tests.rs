use super::*;

#[test]
fn adapter_edits_but_not_consumer_source_movement_invalidate_program_identity() {
    fn identities(main: &str, contract: &str) -> ([u8; 32], [u8; 32]) {
        let modules = project(main, contract);
        let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
        let compiled = collect_project_hir_modules(&modules, stdlib_defs)
            .expect("adapter identity project should compile");
        let invocation = compiled
            .external_defs
            .class_adapter_selections
            .get("main")
            .and_then(|classes| classes.get("Model"))
            .expect("adapter selection exists")
            .adapter_invocation_identity;
        let program = compiled
            .external_defs
            .specialization_outputs
            .get("main")
            .and_then(|outputs| outputs.first())
            .expect("adapter-requested program exists")
            .program_identity;
        (invocation, program)
    }

    let source = r#"
from fixture.contract import Contract, contract_config
class Model(Contract):
    _config = contract_config(True)
    value: int
"#;
    let moved = format!("\n\n{source}");
    let edited = CONTRACT.replace(
        "    fields: list[PlannedField] = []",
        "    adapter_revision: int = 2\n    fields: list[PlannedField] = []",
    );
    let base = identities(source, CONTRACT);
    assert_eq!(base, identities(&moved, CONTRACT));
    assert_ne!(base, identities(source, &edited));
}
