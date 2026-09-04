use super::early_adapters::{attached_contract, compile_errors, project};
use crate::{
    PackageEntrypoint, check_package_project, collect_project_hir_modules, compile_stdlib,
};
use std::path::{Path, PathBuf};

fn static_adapter_negative_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../verification/areas/core_language/fixtures/static_class_adapter/negative/non_string_leaf_rejected",
    )
}

fn package_entrypoint(package_root: &Path) -> PackageEntrypoint {
    let output = std::process::Command::new("cargo")
        .args(["metadata", "--format-version=1", "--locked", "--offline"])
        .arg("--manifest-path")
        .arg(package_root.join("Cargo.toml"))
        .output()
        .expect("negative fixture Cargo metadata must execute");
    assert!(
        output.status.success(),
        "negative fixture Cargo metadata must pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata = sifr_package::parse_metadata_json(&String::from_utf8_lossy(&output.stdout))
        .expect("negative fixture Cargo metadata must parse");
    let graph =
        sifr_package::derive_package_graph(metadata, &mut sifr_frontend::DiskSourceProvider::new())
            .expect("negative fixture package graph must derive");
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("negative fixture source map must build");
    let package_id = graph
        .packages
        .values()
        .find(|metadata| metadata.sifr_name.0 == "static_class_adapter_non_string_negative")
        .expect("negative Sifr package must exist")
        .package_id
        .clone();
    PackageEntrypoint {
        main_file: package_root.join("src/main.sifr"),
        package_id,
        graph,
        source_map,
        python_runtime: None,
        lock_mode: sifr_package::CargoLockMode::Normal,
    }
}

#[test]
fn attached_type_api_codegen_keeps_the_concrete_owner_and_exact_bounds() {
    let modules = project(
        r#"
from fixture.contract import Contract, contract_config

class Model(Contract):
    _config = contract_config(True)
    value: int

def main():
    assert Model.describe() == "attached"
    assert Model.echo("value") == "value"
"#,
        &attached_contract(),
    );
    let stdlib = compile_stdlib().expect("stdlib must compile");
    let compiled = collect_project_hir_modules(&modules, stdlib.defs)
        .expect("attached API project must lower");
    let mut module_names = compiled.hir_modules.keys().cloned().collect::<Vec<_>>();
    module_names.sort();
    let module_refs = module_names
        .iter()
        .map(|name| (name.as_str(), &compiled.hir_modules[name]))
        .collect::<Vec<_>>();
    let generated = sifr_codegen::generate_rust_multi_with_metadata(&module_refs, &stdlib.code)
        .expect("project generation should succeed");
    let provider = &generated.rust_files["fixture.contract"];
    let main = &generated.rust_files["main"];

    assert!(
        provider.contains("pub fn describe<")
            && provider.contains(
                "T: ::sifr_runtime::interop::structural::StaticProgramType + Clone + 'static,"
            ),
        "{provider}"
    );
    assert!(
        provider.contains("pub fn echo<")
            && provider.contains("Input: ::sifr_runtime::interop::structural::StructuralConstruct")
            && provider.contains(
                "+ ::sifr_runtime::interop::structural::StructuralProject + Clone + 'static,"
            ),
        "{provider}"
    );
    assert!(
        main.contains("__sifr_attached_api_fixture_contract_describe::< Model >"),
        "{main}"
    );
}

#[test]
fn attached_defaults_reject_provider_local_expressions_before_export() {
    let contract = attached_contract().replace(
        "def describe[T: StaticProgram](enabled: bool = True) -> str:",
        "def describe_default() -> bool:\n    return True\n\ndef describe[T: StaticProgram](enabled: bool = describe_default()) -> str:",
    );
    let modules = project(
        r#"
from fixture.contract import Contract, contract_config

class Model(Contract):
    _config = contract_config(True)
    value: int
"#,
        &contract,
    );

    let errors = compile_errors(
        &modules,
        "provider-local attached defaults must not escape their declaration module",
    );

    assert!(errors.iter().any(|error| {
        error.code == sifr_diagnostics::DiagnosticCode::TYPE_UNSUPPORTED_DEFAULT_ARGUMENT.code()
            && error.message.contains("describe")
    }));
}

#[test]
fn non_string_leaf_negative_is_package_compilable() {
    let root = static_adapter_negative_root();
    let errors = check_package_project(
        &package_entrypoint(&root),
        &mut sifr_frontend::DiskSourceProvider::new(),
    );

    assert!(
        errors.iter().any(|error| {
            error.code == sifr_diagnostics::DiagnosticCode::PROTO_BOUND_NOT_SATISFIED.code()
                && error.message.contains("StringStructural")
        }),
        "{errors:#?}"
    );
    assert!(
        errors
            .iter()
            .all(|error| !error.message.contains("cannot resolve import")),
        "{errors:#?}"
    );
}
