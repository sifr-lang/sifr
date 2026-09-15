use super::cargo_resolution::CargoResolutionPolicy;
use super::entrypoint::compile_single_file_frontend;
use super::project_codegen::{
    codegen_single_file_frontend, generated_project_binary_project,
    generated_single_file_binary_project,
};
use super::rust_interop::apply_package_rust_interop_metadata_with_resolution;
use super::rust_interop_probe_policy::DirectProbePolicy;
use super::single_file_interop_cache::{resolve_single_file_metadata, stdlib_interop_cache_key};
use super::sysroot_interop::attach_stdlib_rust_interop;
use crate::stdlib::StdlibCompiled;
use sifr_codegen::{InteropBuildPlan, RustInteropOwner, RustInteropPlan};
use sifr_ir::{HirModule, RustInteropArgument, RustInteropValue};
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

pub(super) fn owners(plan: &RustInteropPlan) -> BTreeSet<String> {
    plan.declarations
        .iter()
        .map(|entry| {
            let module = entry
                .module_name
                .as_deref()
                .expect("stdlib module identity");
            match &entry.owner {
                RustInteropOwner::Function { name } | RustInteropOwner::Class { name } => {
                    format!("{module}.{name}")
                }
                RustInteropOwner::Method { class_name, name } => {
                    format!("{module}.{class_name}.{name}")
                }
            }
        })
        .collect()
}

pub(super) fn generated(source: &str) -> (sifr_codegen::CodegenResult, Arc<StdlibCompiled>) {
    let frontend = compile_single_file_frontend(source).expect("application lowers");
    let generated = codegen_single_file_frontend(&frontend).expect("application generates");
    (generated, frontend.stdlib)
}

pub(super) fn resolve(
    generated: sifr_codegen::CodegenResult,
    stdlib: &StdlibCompiled,
    policy: DirectProbePolicy,
) -> Result<InteropBuildPlan, Vec<crate::diagnostics::RenderedDiagnostic>> {
    let project = generated_single_file_binary_project(generated);
    let (project, context) = attach_stdlib_rust_interop(project, None, &stdlib.interop);
    apply_package_rust_interop_metadata_with_resolution(
        project,
        context,
        &CargoResolutionPolicy::normal(),
        policy,
    )
    .map(|project| project.interop)
}

#[test]
fn stdlib_interop_demand_additional_modules_excludes_unrelated_backends() {
    let source = include_str!("../../../../demos/additional_modules/main.sifr");
    let (generated, stdlib) = generated(source);
    let selected = owners(&generated.interop.stdlib_demand);
    assert!(!selected.is_empty());
    for backend in ["_sifr.python.", "_sifr.http.", "_sifr.i18n."] {
        assert!(
            !selected.iter().any(|owner| owner.starts_with(backend)),
            "{backend}: {selected:?}"
        );
    }
    for required in [
        "_sifr.calendar.",
        "_sifr.html.",
        "_sifr.compress.",
        "_sifr.sys.",
        "_sifr.fs.",
    ] {
        assert!(
            selected.iter().any(|owner| owner.starts_with(required)),
            "{required}: {selected:?}"
        );
    }
    assert!(stdlib.interop.plan.rust.declarations.len() > selected.len());
    // Exercise actual native resolution and its probes; unused inventory cannot
    // introduce backend validation work merely because it exists in the sysroot.
    let resolved = resolve(generated, &stdlib, DirectProbePolicy::ExecuteAll)
        .expect("selected bridges validate");
    assert_eq!(owners(&resolved.rust), selected);
    assert!(!resolved.rust.probe_plan.probes.is_empty());
    assert!(resolved.rust.probe_plan.probes.iter().all(|probe| {
        !["_sifr.python", "_sifr.http", "_sifr.i18n"]
            .contains(&probe.module_name.as_deref().unwrap_or_default())
    }));
}

#[test]
fn stdlib_interop_demand_transitive_reexports_and_support() {
    let private_source = r#"
from sifr.meta import Structural

class Nested:
    flag: bool

class Row:
    nested: Nested

@rust.opaque(type=sifr_stdlib.Resource, close=close, send=False, sync=False)
class Resource:
    @rust(Self.close, panic=trusted_no_panic)
    def close(own self) -> Result[None, ValueError]: ...

    @rust(Self.read, panic=trusted_no_panic)
    def read(self) -> Result[Row, ValueError]: ...

@rust(sifr_stdlib.acquire)
def acquire() -> Resource: ...

@rust.structural
@rust(sifr_stdlib.inspect)
def inspect[T: Structural](value: T) -> Result[bool, ValueError | RustPanicError]: ...

@rust(missing_backend.unrelated)
def unrelated() -> bool: ...
"#;
    let public_source = r#"
from _sifr.demand_support import acquire, inspect, Row

def expose() -> Result[bool, ValueError | RustPanicError]:
    resource = acquire()
    try:
        row: Row = resource.read()
        result: bool = inspect(row)
        _ = resource.close()
        return result
    except ValueError as error:
        raise error
    except RustPanicError as error:
        raise error
"#;
    let facade_source = "from sifr.demand_support import expose as available\n";
    let mut stdlib = crate::stdlib::compile_stdlib_uncached().expect("stdlib inventory");
    for (name, source, private) in [
        ("_sifr.demand_support", private_source, true),
        ("sifr.demand_support", public_source, false),
        ("sifr.demand_facade", facade_source, false),
    ] {
        let parsed = sifr_syntax::parse_module_raw(source, None).expect("parse test stdlib");
        assert!(parsed.has_valid_syntax(), "{:?}", parsed.errors());
        let lowered = if private {
            sifr_lowering::lower_module_sysroot_private_declaration_with_externals(
                parsed.suite(),
                &stdlib.defs,
            )
        } else {
            sifr_lowering::lower_module_sysroot_public_stdlib_with_externals(
                parsed.suite(),
                &stdlib.defs,
            )
        }
        .expect("checked test stdlib");
        sifr_frontend::collect_module_exports(name, &lowered, &mut stdlib.defs);
        Arc::make_mut(&mut stdlib.code.hir_modules)
            .insert(name.to_string(), Arc::new(lowered.module));
    }
    // The reexport is represented by checked import identities, not source text.
    let mut app = empty_module();
    app.imports.push(sifr_ir::HirImport {
        module: "sifr.demand_facade".to_string(),
        names: vec!["available".to_string()],
        aliases: Vec::new(),
    });
    let output = sifr_codegen::generate_rust_multi_with_metadata(&[("main", &app)], &stdlib.code);
    let plan = &output.interop.stdlib_demand;
    let selected = owners(plan);
    for required in [
        "acquire",
        "inspect",
        "Resource",
        "Resource.close",
        "Resource.read",
    ] {
        assert!(
            selected.contains(&format!("_sifr.demand_support.{required}")),
            "{selected:?}"
        );
    }
    assert!(!selected.contains("_sifr.demand_support.unrelated"));
    assert_eq!(selected.len(), 5);
    let layouts = &plan.bridge_contracts.generated_types;
    assert!(
        layouts
            .iter()
            .any(|ty| ty.name == "RowBridge" && ty.fields.len() == 1),
        "{layouts:?}"
    );
    assert!(
        layouts
            .iter()
            .any(|ty| ty.name == "NestedBridge" && ty.fields.len() == 1),
        "{layouts:?}"
    );
    assert!(plan.structural_identity_algorithm_version.is_some());
    assert!(
        plan.structural_shape_identities
            .iter()
            .any(|shape| shape.type_name == "Row")
    );
    let close = plan
        .declarations
        .iter()
        .find(|d| matches!(&d.owner, RustInteropOwner::Method { name, .. } if name == "close"))
        .expect("cleanup declaration");
    assert!(close.declaration.consumes_receiver);
    let read = plan
        .bridge_contracts
        .signatures
        .iter()
        .find(|s| matches!(&s.owner, RustInteropOwner::Method { name, .. } if name == "read"))
        .expect("generated nominal read contract");
    assert!(
        read.return_type
            .rust_return_type
            .as_deref()
            .is_some_and(|ty| ty.contains("Row"))
    );
    // A same-named foreign declaration is not part of this closure.
    assert!(
        plan.declarations
            .iter()
            .all(|d| d.module_name.as_deref() == Some("_sifr.demand_support"))
    );
    // Builtin open has no explicit import/call to its native constructor or
    // cleanup function in application HIR, but generated support requires both.
    let (opened, _) = generated(
        "def main():\n    try:\n        handle = open(\"unused\", \"rb\")\n        handle.close()\n    except IOError:\n        pass\n",
    );
    let support = owners(&opened.interop.stdlib_demand);
    for required in ["_open_file", "_file_close", "_file_read_bytes"] {
        assert!(
            support.contains(&format!("_sifr.fs.{required}")),
            "{support:?}"
        );
    }
}

#[test]
fn stdlib_interop_demand_real_python_retains_contracts() {
    let source = "from sifr.python import from_bool\ndef main():\n    _ = from_bool(True)\n";
    let (output, stdlib) = generated(source);
    let selected = owners(&output.interop.stdlib_demand);
    assert!(
        selected.contains("_sifr.python.py_from_bool"),
        "{selected:?}"
    );
    assert!(selected.contains("_sifr.python.Object"));
    assert!(
        output
            .interop
            .stdlib_demand
            .bridge_contracts
            .generated_types
            .iter()
            .any(|ty| ty.name == "PythonErrorBridge" && ty.fields.len() == 5)
    );
    let resolved = resolve(output, &stdlib, DirectProbePolicy::ExecuteAll)
        .expect("real Python contracts validate");
    assert!(resolved.rust.probe_plan.probes.iter().any(|probe| probe.module_name.as_deref() == Some("_sifr.python") && matches!(probe.owner, RustInteropOwner::Function { ref name } if name == "py_from_bool")));
    assert!(
        resolved
            .rust
            .trust_requirements
            .iter()
            .all(|requirement| requirement.trusted)
    );
    let (mut invalid, _) = generated(source);
    let signature = invalid
        .interop
        .stdlib_demand
        .bridge_contracts
        .signatures
        .iter_mut()
        .find(|s| matches!(&s.owner, RustInteropOwner::Function { name } if name == "py_from_bool"))
        .expect("selected bool signature");
    signature.params[0].ty.rust_owned_type = Some("String".to_string());
    signature.params[0].ty.rust_borrowed_type = Some("&str".to_string());
    let errors = resolve(invalid, &stdlib, DirectProbePolicy::ExecuteAll)
        .expect_err("selected sysroot signature must be probed even under sysroot trust");
    assert!(
        errors
            .iter()
            .any(|e| e.code == "SIFR-RUST-TYPE-0001" && e.message.contains("probe failed")),
        "{errors:?}"
    );
}

#[test]
fn stdlib_interop_demand_project_single_file_parity() {
    let source = "from sifr.calendar import isleap\nfrom sifr.gzip import compress\ndef main():\n    assert isleap(2024)\n    _ = compress(\"sample\")\n";
    let (single, stdlib) = generated(source);
    let selected = single.interop.stdlib_demand.clone();
    let parsed = crate::frontend::parse_source(source).expect("parse project");
    let lowering = crate::project::compile_frontend_modules(
        &HashMap::from([("main".to_string(), parsed)]),
        stdlib.defs.clone(),
        sifr_frontend::FrontendDiagnosticStyle::ModulePrefixed,
    )
    .expect("project lowering");
    let project =
        generated_project_binary_project(&stdlib.code, lowering).expect("project generation");
    assert_eq!(project.interop.stdlib_demand, selected);
    let (project, context) = attach_stdlib_rust_interop(project, None, &stdlib.interop);
    let checked = apply_package_rust_interop_metadata_with_resolution(
        project,
        context,
        &CargoResolutionPolicy::normal(),
        DirectProbePolicy::ExecuteAll,
    )
    .expect("project/check resolver");
    let single =
        resolve(single, &stdlib, DirectProbePolicy::ExecuteAll).expect("single native resolver");
    assert_eq!(checked.interop.rust, single.rust);
    // Emit deferral preserves exactly the same contracts without doing probes.
    let (emit, _) = generated(source);
    let emitted =
        resolve(emit, &stdlib, DirectProbePolicy::DeferTrustedSysroot).expect("emit policy");
    assert_eq!(owners(&emitted.rust), owners(&single.rust));
    assert_eq!(emitted.rust.bridge_contracts, single.rust.bridge_contracts);
}

#[test]
fn stdlib_interop_demand_cache_disjoint_programs() {
    const A: &str = "from sifr.calendar import isleap\ndef main():\n    assert isleap(2024)\n";
    const B: &str = "from sifr.html import escape\ndef main():\n    _ = escape(\"<p>\")\n";
    let mut expected = HashMap::new();
    for source in [A, B, B, A, A] {
        let (output, stdlib) = generated(source);
        let expected_owners = owners(&output.interop.stdlib_demand);
        let key = stdlib_interop_cache_key(&stdlib.interop, &output.interop);
        let resolved =
            resolve_single_file_metadata(output, None, &stdlib.interop).expect("cached resolution");
        assert_eq!(owners(&resolved.interop.rust), expected_owners);
        let fragment = resolved.interop.cache_key_fragment();
        if let Some((prior_key, prior_fragment)) =
            expected.insert(source, (key.clone(), fragment.clone()))
        {
            assert_eq!(key, prior_key);
            assert_eq!(fragment, prior_fragment);
        }
    }
    assert_ne!(expected[A], expected[B]);
    let (output, stdlib) = generated(A);
    let original = stdlib_interop_cache_key(&stdlib.interop, &output.interop);
    let mut changed_interop = stdlib.interop.clone();
    changed_interop
        .sysroot
        .as_mut()
        .expect("sysroot")
        .manifest
        .sysroot_content_sha256 = "changed-content".to_string();
    assert_ne!(
        original,
        stdlib_interop_cache_key(&changed_interop, &output.interop)
    );
    let (mut output, stdlib) = generated(A);
    let original = stdlib_interop_cache_key(&stdlib.interop, &output.interop);
    let declaration = &mut output.interop.stdlib_demand.declarations[0].declaration;
    declaration.arguments.push(RustInteropArgument {
        name: Some("panic".to_string()),
        value: RustInteropValue::Symbol("trusted_no_panic".to_string()),
        span: declaration.span,
    });
    assert_ne!(
        original,
        stdlib_interop_cache_key(&stdlib.interop, &output.interop)
    );
    let resolved = resolve_single_file_metadata(output, None, &stdlib.interop)
        .expect("changed contract resolution");
    assert!(!resolved.interop.rust.trust_requirements.is_empty());
}

fn empty_module() -> HirModule {
    HirModule {
        functions: Vec::new(),
        classes: Vec::new(),
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    }
}
