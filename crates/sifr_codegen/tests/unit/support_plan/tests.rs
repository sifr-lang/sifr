use super::{ModuleSupportDemand, render_support};
use crate::StdlibRustSource;
use std::collections::HashSet;

#[test]
fn single_file_user_error_suppresses_late_runtime_demand() {
    let mut demand = ModuleSupportDemand::default();
    demand.runtime.task_scope = true;
    demand
        .locally_shadowed_error_classes
        .insert("SecondaryError".to_string());

    let rendered = render_support(
        &demand,
        &crate::StdlibEmissionView::new(&crate::StdlibEmissionCode::default()),
    );

    assert!(!rendered.source.contains("struct SecondaryError"));
}

#[test]
fn project_merge_does_not_promote_a_module_shadow_to_a_crate_veto() {
    let mut module = ModuleSupportDemand::default();
    module
        .locally_shadowed_error_classes
        .insert("ValueError".to_string());
    module
        .directly_used_stdlib_modules
        .insert("sifr.fixture".to_string());

    let mut project = ModuleSupportDemand::default();
    project.merge_project_module(&module);

    let mut stdlib = crate::StdlibCode::default();
    stdlib.module_rust_code.insert(
        "sifr.fixture".to_string(),
        StdlibRustSource {
            module: "sifr.fixture".to_string(),
            source_path: "stdlib/sifr/fixture.sifr".to_string(),
            source_sha256: "error-identity-fixture".to_string(),
            nominal_types: HashSet::new(),
            rust: "fn operation() -> Result<(), ValueError> { Ok(()) }\n".to_string(),
        },
    );

    let rendered = render_support(&project, &crate::StdlibEmissionView::new(&stdlib));

    assert!(rendered.source.contains("struct ValueError"));
}
