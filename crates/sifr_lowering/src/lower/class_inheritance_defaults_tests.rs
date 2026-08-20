use crate::lower::lower_module;
use sifr_python_parser::parse_module;

#[test]
fn child_hir_flattens_inherited_field_defaults() {
    let parsed = parse_module(
        "class Parent:\n    value: int = 7\n\nclass Child(Parent):\n    label: str = \"child\"\n\n    def __init__(self, value: int = 7, label: str = \"child\"):\n        super().__init__(value)\n        self.label = label\n",
    )
    .expect("source must parse");
    let module = lower_module(parsed.suite())
        .expect("inherited defaults must lower")
        .module;
    let child = module
        .classes
        .iter()
        .find(|class| class.name == "Child")
        .expect("child HIR must exist");

    assert_eq!(child.field_defaults.len(), 2);
    assert!(matches!(
        child.field_defaults[0],
        (0, crate::HirExpr::IntLiteral(7))
    ));
    assert!(matches!(
        &child.field_defaults[1],
        (1, crate::HirExpr::StringLiteral(value)) if value == "child"
    ));
}
