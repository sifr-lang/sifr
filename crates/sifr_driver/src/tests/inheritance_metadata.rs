use super::support::parse_suite;
use crate::{collect_project_hir_modules, compile_stdlib};
use std::collections::HashMap;

#[test]
fn imported_parent_defaults_are_flattened_into_child_hir() {
    let modules = HashMap::from([
        (
            "models".to_string(),
            parse_suite("class Parent:\n    value: int = 7\n"),
        ),
        (
            "main".to_string(),
            parse_suite(
                "from models import Parent\n\nclass Child(Parent):\n    label: str = \"child\"\n\n    def __init__(self, value: int = 7, label: str = \"child\"):\n        super().__init__(value)\n        self.label = label\n",
            ),
        ),
    ]);
    let stdlib = compile_stdlib().expect("stdlib must compile");
    let compiled = collect_project_hir_modules(&modules, stdlib.defs)
        .expect("imported inheritance project must lower");
    let child = compiled.hir_modules["main"]
        .classes
        .iter()
        .find(|class| class.name == "Child")
        .expect("child HIR must exist");

    assert_eq!(child.field_defaults.len(), 2);
    assert!(matches!(
        child.field_defaults[0],
        (0, sifr_ir::HirExpr::IntLiteral(7))
    ));
    assert!(matches!(
        &child.field_defaults[1],
        (1, sifr_ir::HirExpr::StringLiteral(value)) if value == "child"
    ));
}
