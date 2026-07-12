use crate::python_interop_plan::{
    push_python_plan_cache_key, python_interop_plan_for_named_modules, PythonInteropPlan,
};
use crate::rust_interop_bridge_contract::{
    bridge_contract_plan_for_named_modules, push_bridge_contract_plan, RustBridgeContractPlan,
};
use sifr_ir::{
    HirClass, HirFunction, HirModule, RustInteropDeclaration, RustInteropValue, RustTargetPath,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InteropBuildPlan {
    pub rust: RustInteropPlan,
    pub python: PythonInteropPlan,
}

impl InteropBuildPlan {
    #[must_use]
    pub fn cache_key_fragment(&self) -> String {
        let mut out = String::new();
        push_python_plan_cache_key(&mut out, &self.python);
        out.push_str("rust.declarations=");
        out.push_str(&self.rust.declarations.len().to_string());
        out.push('\n');
        for declaration in &self.rust.declarations {
            out.push_str("module=");
            out.push_str(declaration.module_name.as_deref().unwrap_or("<single>"));
            out.push('\n');
            out.push_str("owner=");
            push_owner(&mut out, &declaration.owner);
            out.push('\n');
            push_declaration(&mut out, &declaration.declaration);
            out.push('\n');
        }
        out.push_str("rust.resolved_targets=");
        out.push_str(&self.rust.resolved_targets.len().to_string());
        out.push('\n');
        for target in &self.rust.resolved_targets {
            push_resolved_target(&mut out, target);
            out.push('\n');
        }
        out.push_str("rust.generated_bridge_modules=");
        out.push_str(&self.rust.generated_bridge_modules.len().to_string());
        out.push('\n');
        for module in &self.rust.generated_bridge_modules {
            push_generated_bridge_module(&mut out, module);
            out.push('\n');
        }
        push_bridge_contract_plan(&mut out, &self.rust.bridge_contracts);
        out.push_str("rust.trust_requirements=");
        out.push_str(&self.rust.trust_requirements.len().to_string());
        out.push('\n');
        for requirement in &self.rust.trust_requirements {
            push_trust_requirement(&mut out, requirement);
            out.push('\n');
        }
        out.push_str("rust.probes=");
        out.push_str(&self.rust.probe_plan.probes.len().to_string());
        out.push('\n');
        for probe in &self.rust.probe_plan.probes {
            push_probe(&mut out, probe);
            out.push('\n');
        }
        out.push_str("rust.bridge_sources=");
        out.push_str(&self.rust.bridge_sources.len().to_string());
        out.push('\n');
        for bridge_source in &self.rust.bridge_sources {
            push_bridge_source(&mut out, bridge_source);
            out.push('\n');
        }
        if let Some(cargo) = &self.rust.cargo_inputs {
            push_cargo_inputs(&mut out, cargo);
        }
        out
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RustInteropPlan {
    pub declarations: Vec<RustInteropPlanDeclaration>,
    pub resolved_targets: Vec<RustInteropResolvedTarget>,
    pub generated_bridge_modules: Vec<RustGeneratedBridgeModule>,
    pub bridge_contracts: RustBridgeContractPlan,
    pub trust_requirements: Vec<RustInteropTrustRequirement>,
    pub probe_plan: RustBridgeProbePlan,
    pub bridge_sources: Vec<RustBridgeSourceDigest>,
    pub cargo_inputs: Option<RustInteropCargoInputs>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustInteropPlanDeclaration {
    pub module_name: Option<String>,
    pub owner: RustInteropOwner,
    pub declaration: RustInteropDeclaration,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustInteropOwner {
    Function { name: String },
    Class { name: String },
    Method { class_name: String, name: String },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustInteropResolvedTarget {
    pub module_name: Option<String>,
    pub owner: RustInteropOwner,
    pub written_path: String,
    pub canonical_target_path: String,
    pub root: RustInteropResolvedRoot,
    pub span: ruff_text_size::TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RustGeneratedBridgeModule {
    pub module_name: Option<String>,
    pub rust_module_path: Vec<String>,
    pub bridge_version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustInteropResolvedRoot {
    DirectCargoDependency {
        dependency_name: String,
        cargo_package_id: String,
        cargo_package_name: String,
        cargo_version: String,
        cargo_source: Option<String>,
        cargo_manifest_path: String,
    },
    SysrootCrate {
        dependency_name: String,
        cargo_package_name: String,
        cargo_version: String,
        cargo_manifest_path: String,
        sysroot_root: String,
        toolchain_id: String,
        sysroot_content_sha256: String,
    },
    PackageBridge {
        package_id: String,
        dependency_name: String,
        cargo_package_name: String,
        cargo_manifest_path: String,
        bridge_roots: Vec<String>,
    },
    SelfMethod {
        class_name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustInteropTrustRequirement {
    pub canonical_target_path: String,
    pub kind: RustInteropTrustRequirementKind,
    pub trusted: bool,
    pub required_entry: String,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustInteropTrustRequirementKind {
    BuildScript,
    ProcMacro,
    NativeLinks,
    BuildEnv,
    UnsafeBridge,
    NoPanic,
    PanicAbort,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RustBridgeProbePlan {
    pub probes: Vec<RustBridgeProbe>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustBridgeProbe {
    pub canonical_target_path: String,
    pub module_name: Option<String>,
    pub owner: RustInteropOwner,
    pub kind: RustBridgeProbeKind,
    pub requires_send: bool,
    pub requires_sync: bool,
    pub span: ruff_text_size::TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustBridgeProbeKind {
    Function,
    AsyncFunction,
    Method,
    OpaqueHandle,
    ZeroCopy,
    View,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustBridgeSourceDigest {
    pub package_id: String,
    pub bridge_root: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustInteropCargoInputs {
    pub package_id: String,
    pub cargo_metadata_digest: Option<String>,
    pub package_graph_digest: Option<String>,
    pub package_source_map_digest: Option<String>,
    pub cargo_lock_digest: Option<String>,
    pub target_triple: Option<String>,
    pub target_features: Vec<String>,
    pub cargo_profile: String,
    pub panic_strategy: Option<String>,
    pub profile_codegen_settings: Vec<(String, String)>,
    pub cargo_version: Option<String>,
    pub rustc_version: Option<String>,
    pub bridge_version: Option<u32>,
    pub trust_policy_digest: String,
    pub declared_build_env: Vec<String>,
}

pub(crate) fn interop_build_plan_for_module(module: &HirModule) -> InteropBuildPlan {
    interop_build_plan_for_named_modules([(None, module)])
}

pub fn interop_build_plan_for_named_modules<'a>(
    modules: impl IntoIterator<Item = (Option<&'a str>, &'a HirModule)>,
) -> InteropBuildPlan {
    let module_entries = modules.into_iter().collect::<Vec<_>>();
    let python = python_interop_plan_for_named_modules(module_entries.iter().copied());
    let mut rust = RustInteropPlan::default();
    for (module_name, module) in &module_entries {
        collect_module_declarations(*module_name, module, &mut rust.declarations);
    }
    rust.bridge_contracts =
        bridge_contract_plan_for_named_modules(module_entries, &rust.declarations);
    InteropBuildPlan { rust, python }
}

fn collect_module_declarations(
    module_name: Option<&str>,
    module: &HirModule,
    declarations: &mut Vec<RustInteropPlanDeclaration>,
) {
    for function in &module.functions {
        collect_function_declarations(module_name, function, declarations);
    }
    for class in &module.classes {
        collect_class_declarations(module_name, class, declarations);
    }
}

fn collect_function_declarations(
    module_name: Option<&str>,
    function: &HirFunction,
    declarations: &mut Vec<RustInteropPlanDeclaration>,
) {
    extend_declarations(
        module_name,
        &RustInteropOwner::Function {
            name: function.name.clone(),
        },
        &function.rust_interop,
        declarations,
    );
}

fn collect_class_declarations(
    module_name: Option<&str>,
    class: &HirClass,
    declarations: &mut Vec<RustInteropPlanDeclaration>,
) {
    extend_declarations(
        module_name,
        &RustInteropOwner::Class {
            name: class.name.clone(),
        },
        &class.rust_interop,
        declarations,
    );
    for method in &class.methods {
        collect_method_declarations(module_name, &class.name, method, declarations);
    }
    for (_, method) in &class.operator_impls {
        collect_method_declarations(module_name, &class.name, method, declarations);
    }
}

fn collect_method_declarations(
    module_name: Option<&str>,
    class_name: &str,
    method: &HirFunction,
    declarations: &mut Vec<RustInteropPlanDeclaration>,
) {
    extend_declarations(
        module_name,
        &RustInteropOwner::Method {
            class_name: class_name.to_string(),
            name: method.name.clone(),
        },
        &method.rust_interop,
        declarations,
    );
}

fn extend_declarations(
    module_name: Option<&str>,
    owner: &RustInteropOwner,
    rust_interop: &[RustInteropDeclaration],
    declarations: &mut Vec<RustInteropPlanDeclaration>,
) {
    declarations.extend(rust_interop.iter().cloned().map(|declaration| {
        RustInteropPlanDeclaration {
            module_name: module_name.map(str::to_string),
            owner: owner.clone(),
            declaration,
        }
    }));
}

fn push_owner(out: &mut String, owner: &RustInteropOwner) {
    match owner {
        RustInteropOwner::Function { name } => {
            out.push_str("function:");
            out.push_str(name);
        }
        RustInteropOwner::Class { name } => {
            out.push_str("class:");
            out.push_str(name);
        }
        RustInteropOwner::Method { class_name, name } => {
            out.push_str("method:");
            out.push_str(class_name);
            out.push('.');
            out.push_str(name);
        }
    }
}

fn push_declaration(out: &mut String, declaration: &RustInteropDeclaration) {
    out.push_str("kind=");
    out.push_str(match declaration.kind {
        sifr_ir::RustInteropDecoratorKind::Function => "function",
        sifr_ir::RustInteropDecoratorKind::Opaque => "opaque",
        sifr_ir::RustInteropDecoratorKind::Async => "async",
        sifr_ir::RustInteropDecoratorKind::Callback => "callback",
        sifr_ir::RustInteropDecoratorKind::ZeroCopy => "zero_copy",
        sifr_ir::RustInteropDecoratorKind::View => "view",
    });
    out.push_str(";target=");
    if let Some(target) = &declaration.target {
        push_target_path(out, target);
    } else {
        out.push_str("<none>");
    }
    out.push_str(";span=");
    push_span(out, declaration.span);
    out.push_str(";effect=");
    out.push_str(match declaration.effect {
        sifr_ir::RustInteropEffect::Sync => "sync",
        sifr_ir::RustInteropEffect::Async => "async",
        sifr_ir::RustInteropEffect::BlockingIo => "blocking_io",
        sifr_ir::RustInteropEffect::CpuHeavy => "cpu_heavy",
    });
    out.push_str(";abi=");
    out.push_str(if declaration.abi_requirements.async_boundary {
        "async:1"
    } else {
        "async:0"
    });
    out.push(',');
    out.push_str(if declaration.abi_requirements.opaque_handle {
        "opaque:1"
    } else {
        "opaque:0"
    });
    out.push(',');
    out.push_str(if declaration.abi_requirements.zero_copy {
        "zero_copy:1"
    } else {
        "zero_copy:0"
    });
    out.push(',');
    out.push_str(if declaration.abi_requirements.view {
        "view:1"
    } else {
        "view:0"
    });
    for argument in &declaration.arguments {
        out.push_str(";arg=");
        out.push_str(argument.name.as_deref().unwrap_or("<pos>"));
        out.push(':');
        push_value(out, &argument.value);
        out.push('@');
        push_span(out, argument.span);
    }
}

fn push_resolved_target(out: &mut String, target: &RustInteropResolvedTarget) {
    out.push_str("resolved=");
    out.push_str(target.module_name.as_deref().unwrap_or("<single>"));
    out.push(':');
    push_owner(out, &target.owner);
    out.push_str(";written=");
    out.push_str(&target.written_path);
    out.push_str(";canonical=");
    out.push_str(&target.canonical_target_path);
    out.push_str(";root=");
    match &target.root {
        RustInteropResolvedRoot::DirectCargoDependency {
            dependency_name,
            cargo_package_id,
            cargo_package_name,
            cargo_version,
            cargo_source,
            cargo_manifest_path,
        } => {
            out.push_str("cargo:");
            out.push_str(dependency_name);
            out.push(':');
            out.push_str(cargo_package_id);
            out.push(':');
            out.push_str(cargo_package_name);
            out.push('@');
            out.push_str(cargo_version);
            out.push(':');
            out.push_str(cargo_source.as_deref().unwrap_or("<path>"));
            out.push(':');
            out.push_str(cargo_manifest_path);
        }
        RustInteropResolvedRoot::SysrootCrate {
            dependency_name,
            cargo_package_name,
            cargo_version,
            cargo_manifest_path,
            sysroot_root,
            toolchain_id,
            sysroot_content_sha256,
        } => {
            out.push_str("sysroot:");
            out.push_str(dependency_name);
            out.push(':');
            out.push_str(cargo_package_name);
            out.push('@');
            out.push_str(cargo_version);
            out.push(':');
            out.push_str(cargo_manifest_path);
            out.push(':');
            out.push_str(sysroot_root);
            out.push(':');
            out.push_str(toolchain_id);
            out.push(':');
            out.push_str(sysroot_content_sha256);
        }
        RustInteropResolvedRoot::PackageBridge {
            package_id,
            dependency_name,
            cargo_package_name,
            cargo_manifest_path,
            bridge_roots,
        } => {
            out.push_str("bridge:");
            out.push_str(package_id);
            out.push(':');
            out.push_str(dependency_name);
            out.push(':');
            out.push_str(cargo_package_name);
            out.push(':');
            out.push_str(cargo_manifest_path);
            out.push(':');
            out.push_str(&bridge_roots.join(","));
        }
        RustInteropResolvedRoot::SelfMethod { class_name } => {
            out.push_str("self:");
            out.push_str(class_name);
        }
    }
    out.push_str(";span=");
    push_span(out, target.span);
}

fn push_generated_bridge_module(out: &mut String, module: &RustGeneratedBridgeModule) {
    out.push_str("generated_bridge=");
    match &module.module_name {
        Some(module_name) => {
            out.push_str("module:");
            out.push_str(module_name);
        }
        None => out.push_str("binary-entry"),
    }
    out.push_str(";version=");
    out.push_str(&module.bridge_version.to_string());
    out.push_str(";path=");
    out.push_str(&module.rust_module_path.join("::"));
}

fn push_trust_requirement(out: &mut String, requirement: &RustInteropTrustRequirement) {
    out.push_str("trust=");
    out.push_str(&requirement.canonical_target_path);
    out.push(':');
    out.push_str(match requirement.kind {
        RustInteropTrustRequirementKind::BuildScript => "build_script",
        RustInteropTrustRequirementKind::ProcMacro => "proc_macro",
        RustInteropTrustRequirementKind::NativeLinks => "native_links",
        RustInteropTrustRequirementKind::BuildEnv => "build_env",
        RustInteropTrustRequirementKind::UnsafeBridge => "unsafe_bridge",
        RustInteropTrustRequirementKind::NoPanic => "no_panic",
        RustInteropTrustRequirementKind::PanicAbort => "panic_abort",
    });
    out.push(':');
    out.push_str(if requirement.trusted {
        "trusted"
    } else {
        "untrusted"
    });
    out.push(':');
    out.push_str(&requirement.required_entry);
    out.push(':');
    out.push_str(&requirement.evidence);
}

fn push_probe(out: &mut String, probe: &RustBridgeProbe) {
    out.push_str("probe=");
    out.push_str(&probe.canonical_target_path);
    out.push(':');
    out.push_str(probe.module_name.as_deref().unwrap_or("<single>"));
    out.push(':');
    push_owner(out, &probe.owner);
    out.push(':');
    out.push_str(match probe.kind {
        RustBridgeProbeKind::Function => "function",
        RustBridgeProbeKind::AsyncFunction => "async_function",
        RustBridgeProbeKind::Method => "method",
        RustBridgeProbeKind::OpaqueHandle => "opaque_handle",
        RustBridgeProbeKind::ZeroCopy => "zero_copy",
        RustBridgeProbeKind::View => "view",
    });
    out.push_str(":send=");
    out.push_str(if probe.requires_send { "1" } else { "0" });
    out.push_str(":sync=");
    out.push_str(if probe.requires_sync { "1" } else { "0" });
    out.push_str(":span=");
    push_span(out, probe.span);
}

fn push_bridge_source(out: &mut String, bridge_source: &RustBridgeSourceDigest) {
    out.push_str("bridge_source=");
    out.push_str(&bridge_source.package_id);
    out.push(':');
    out.push_str(&bridge_source.bridge_root);
    out.push(':');
    out.push_str(&bridge_source.digest);
}

fn push_cargo_inputs(out: &mut String, cargo: &RustInteropCargoInputs) {
    out.push_str("rust.cargo.package=");
    out.push_str(&cargo.package_id);
    out.push('\n');
    out.push_str("rust.cargo.metadata_digest=");
    out.push_str(cargo.cargo_metadata_digest.as_deref().unwrap_or("<none>"));
    out.push('\n');
    out.push_str("rust.cargo.graph_digest=");
    out.push_str(cargo.package_graph_digest.as_deref().unwrap_or("<none>"));
    out.push('\n');
    out.push_str("rust.cargo.source_map_digest=");
    out.push_str(
        cargo
            .package_source_map_digest
            .as_deref()
            .unwrap_or("<none>"),
    );
    out.push('\n');
    out.push_str("rust.cargo.lock_digest=");
    out.push_str(cargo.cargo_lock_digest.as_deref().unwrap_or("<none>"));
    out.push('\n');
    out.push_str("rust.cargo.target=");
    out.push_str(cargo.target_triple.as_deref().unwrap_or("<host>"));
    out.push('\n');
    out.push_str("rust.cargo.features=");
    out.push_str(&cargo.target_features.join(","));
    out.push('\n');
    out.push_str("rust.cargo.profile=");
    out.push_str(&cargo.cargo_profile);
    out.push('\n');
    out.push_str("rust.cargo.panic=");
    out.push_str(cargo.panic_strategy.as_deref().unwrap_or("<default>"));
    out.push('\n');
    for (name, value) in &cargo.profile_codegen_settings {
        out.push_str("rust.cargo.profile_setting=");
        out.push_str(name);
        out.push('=');
        out.push_str(value);
        out.push('\n');
    }
    out.push_str("rust.cargo.cargo_version=");
    out.push_str(cargo.cargo_version.as_deref().unwrap_or("<unknown>"));
    out.push('\n');
    out.push_str("rust.cargo.rustc_version=");
    out.push_str(cargo.rustc_version.as_deref().unwrap_or("<unknown>"));
    out.push('\n');
    out.push_str("rust.cargo.bridge_version=");
    out.push_str(
        &cargo
            .bridge_version
            .map_or_else(|| "<none>".to_string(), |version| version.to_string()),
    );
    out.push('\n');
    out.push_str("rust.cargo.trust_policy=");
    out.push_str(&cargo.trust_policy_digest);
    out.push('\n');
    out.push_str("rust.cargo.build_env=");
    out.push_str(&cargo.declared_build_env.join(","));
    out.push('\n');
}

fn push_value(out: &mut String, value: &RustInteropValue) {
    match value {
        RustInteropValue::Boolean(value) => {
            out.push_str(if *value { "bool:true" } else { "bool:false" });
        }
        RustInteropValue::Symbol(value) => {
            out.push_str("symbol:");
            out.push_str(value);
        }
        RustInteropValue::Integer(value) => {
            out.push_str("int:");
            out.push_str(&value.to_string());
        }
        RustInteropValue::IntegerList(values) => {
            out.push_str("int-list:");
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(&value.to_string());
            }
        }
        RustInteropValue::PolicyCall {
            name,
            argument,
            span,
        } => {
            out.push_str("policy:");
            out.push_str(name);
            out.push('(');
            push_value(out, argument);
            out.push_str(")@");
            push_span(out, *span);
        }
        RustInteropValue::TargetPath(path) => {
            out.push_str("path:");
            push_target_path(out, path);
        }
    }
}

fn push_target_path(out: &mut String, path: &RustTargetPath) {
    out.push_str(&path.segments.join("."));
    out.push('@');
    push_span(out, path.span);
}

fn push_span(out: &mut String, span: ruff_text_size::TextRange) {
    out.push_str(&span.start().to_u32().to_string());
    out.push_str("..");
    out.push_str(&span.end().to_u32().to_string());
}

#[cfg(test)]
#[path = "rust_interop_plan_tests.rs"]
mod rust_interop_plan_tests;
