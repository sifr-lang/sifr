use super::early_adapters::{attached_contract, project};
use super::project_build_check::mktemp_dir;
use crate::{build_project, collect_project_hir_modules, compile_stdlib};

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

#[test]
fn imported_generic_type_alias_is_erased_before_rust_codegen() {
    let dir = mktemp_dir("imported_generic_type_alias_codegen");
    let main_file = dir.join("main.sifr");
    let build_out = dir.join("build_out");
    std::fs::write(
        &main_file,
        r#"
from facade import Adapter
from models import Model

def keep(own value: Adapter[Model]) -> Adapter[Model]:
    return value

def main():
    model: Model = keep(Model(7))
    assert model.value == 7
"#,
    )
    .expect("main module should be written");
    std::fs::write(dir.join("facade.sifr"), "type Adapter[T] = T\n")
        .expect("facade module should be written");
    std::fs::write(dir.join("models.sifr"), "class Model:\n    value: int\n")
        .expect("model module should be written");

    let binary = build_project(
        &main_file,
        &build_out,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("an imported generic alias should not become a Rust import");
    assert!(binary.exists());
    let generated = std::fs::read_to_string(build_out.join("sifr_output/src/main.rs"))
        .expect("generated main module should exist");
    assert!(!generated.contains("use crate::facade::Adapter;"));

    let _ = std::fs::remove_dir_all(dir);
}
