use super::early_adapters::{attached_contract, project};
use crate::{collect_project_hir_modules, compile_stdlib};

#[test]
fn local_generic_type_alias_forwards_attached_type_calls() {
    let contract = attached_contract();
    let modules = project(
        r#"
from fixture.contract import Contract, contract_config

class Model(Contract):
    _config = contract_config(True)
    value: int

type Adapter[T] = T

def main():
    assert Adapter[Model].describe() == "attached"
    assert Adapter[Model].echo("value") == "value"
"#,
        &contract,
    );
    let stdlib_defs = compile_stdlib().expect("stdlib should compile").defs;
    let compiled = collect_project_hir_modules(&modules, stdlib_defs)
        .expect("a local generic type alias should forward attached type calls");
    let main = compiled
        .hir_modules
        .get("main")
        .expect("main module exists");
    let debug_hir = format!("{main:?}");
    assert!(debug_hir.contains("BoolLiteral(true)"), "{debug_hir}");
    assert!(debug_hir.contains("name: \"Model\""), "{debug_hir}");
}
