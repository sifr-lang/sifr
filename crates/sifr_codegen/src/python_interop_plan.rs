use sifr_ir::{
    HirFunction, HirModule, PythonCallbackConcurrency, PythonCallbackDispatch,
    PythonCallbackLifetime, PythonCleanupPolicy, PythonInteropDeclaration,
};
use sifr_type_system::Type;
use std::collections::BTreeSet;
use std::fmt::Write;

use crate::hir_analysis::traversal::{walk_stmts, TraversalConfig};

const PYTHON_BINDING_CONTRACT_VERSION: &str = "sifr-python-binding-v1";

/// Build-time authority for declaration-first Python probing and wrapper inputs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PythonInteropPlan {
    pub declarations: Vec<PythonInteropPlanDeclaration>,
    pub required_import_roots: Vec<String>,
    pub target_probes: Vec<PythonTargetProbe>,
    pub bridge_packages: Vec<PythonBridgePackagePlan>,
    pub requires_async_loop: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonBridgePackagePlan {
    pub package_id: String,
    pub resolved_package_key: String,
    pub runtime_package: String,
    pub inventory_digest: String,
    pub modules: Vec<PythonBridgeModulePlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonBridgeModulePlan {
    pub module: String,
    pub runtime_module: String,
    pub source_path: String,
    pub source_digest: String,
    pub source: String,
    pub is_package: bool,
    pub imports: Vec<PythonBridgeImportPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PythonBridgeImportPlan {
    SamePackage {
        module: String,
        runtime_module: String,
    },
    ThirdParty {
        root: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonInteropPlanDeclaration {
    pub module_name: Option<String>,
    pub function_name: String,
    pub declaration: PythonInteropDeclaration,
    /// Exact package certification key. Receiver declarations use their
    /// enclosing opaque Python type target instead of the syntactic `Self`.
    pub certification_target: Option<String>,
    pub parameter_types: Vec<Type>,
    pub return_type: Type,
    pub callback_attachments: Vec<PythonCallbackAttachmentPlan>,
}

/// Ownership and execution metadata used when callback wrappers are generated.
///
/// Retained in the build plan even while callback declarations remain gated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonCallbackAttachmentPlan {
    pub parameter_name: String,
    pub lifetime: PythonCallbackLifetime,
    pub dispatch: PythonCallbackDispatch,
    pub concurrency: Option<PythonCallbackConcurrency>,
    pub owner_class: Option<String>,
    pub owner_cleanup: Option<PythonCleanupPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonTargetProbe {
    pub import_root: Option<String>,
    pub target_path: String,
    pub requires_inspectable_signature: bool,
    pub expects_type: bool,
    pub status: PythonTargetProbeStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonTargetProbeStatus {
    Planned,
    Verified,
    RuntimeChecked,
}

pub(crate) fn python_interop_plan_for_named_modules<'a>(
    modules: impl IntoIterator<Item = (Option<&'a str>, &'a HirModule)>,
) -> PythonInteropPlan {
    let mut plan = PythonInteropPlan::default();
    let modules = modules.into_iter().collect::<Vec<_>>();
    let record_expansion_functions = record_expansion_functions(&modules);
    for (module_name, module) in modules {
        plan.requires_async_loop |= module_requires_raw_coroutine_loop(module);
        for function in &module.functions {
            collect_function(
                module_name,
                function,
                &record_expansion_functions,
                &mut plan,
            );
        }
        for class in &module.classes {
            plan.requires_async_loop |= class.methods.iter().any(|method| {
                method
                    .python_interop
                    .iter()
                    .any(|declaration| declaration.effect == sifr_ir::PythonInteropEffect::Async)
            });
            let Some(declaration) = class.python_opaque_declaration() else {
                continue;
            };
            plan.requires_async_loop |= declaration.effect == sifr_ir::PythonInteropEffect::Async;
            if let Some(root) = &declaration.required_import_root {
                plan.required_import_roots.push(root.clone());
            }
            if let Some(target) = &declaration.target {
                plan.target_probes.push(PythonTargetProbe {
                    import_root: declaration.required_import_root.clone(),
                    target_path: target.dotted(),
                    requires_inspectable_signature: false,
                    expects_type: true,
                    status: PythonTargetProbeStatus::Planned,
                });
                plan.declarations.push(PythonInteropPlanDeclaration {
                    module_name: module_name.map(str::to_string),
                    function_name: class.name.clone(),
                    declaration: declaration.clone(),
                    certification_target: Some(target.dotted()),
                    parameter_types: Vec::new(),
                    return_type: Type::None,
                    callback_attachments: callback_attachments(declaration),
                });
                for method in &class.methods {
                    collect_arrow_method(
                        module_name,
                        &class.name,
                        method,
                        &target.dotted(),
                        &mut plan,
                    );
                }
            }
        }
    }
    plan.required_import_roots.sort();
    plan.required_import_roots.dedup();
    plan
}

fn collect_function(
    module_name: Option<&str>,
    function: &HirFunction,
    record_expansion_functions: &BTreeSet<String>,
    plan: &mut PythonInteropPlan,
) {
    for declaration in &function.python_interop {
        plan.requires_async_loop |= declaration.effect == sifr_ir::PythonInteropEffect::Async;
        plan.requires_async_loop |= declaration
            .callbacks
            .iter()
            .any(|callback| callback.dispatch == PythonCallbackDispatch::Asyncio);
        if let Some(root) = &declaration.required_import_root {
            plan.required_import_roots.push(root.clone());
        }
        let Some(target) = declaration.target.as_ref() else {
            continue;
        };
        plan.target_probes.push(PythonTargetProbe {
            import_root: declaration.required_import_root.clone(),
            target_path: target.dotted(),
            requires_inspectable_signature: record_expansion_functions.contains(&function.name),
            expects_type: false,
            status: PythonTargetProbeStatus::Planned,
        });
        plan.declarations.push(PythonInteropPlanDeclaration {
            module_name: module_name.map(str::to_string),
            function_name: function.name.clone(),
            declaration: declaration.clone(),
            certification_target: Some(target.dotted()),
            parameter_types: function
                .params
                .iter()
                .map(|param| param.ty.clone())
                .collect(),
            return_type: function.return_type.clone(),
            callback_attachments: callback_attachments(declaration),
        });
    }
}

fn collect_arrow_method(
    module_name: Option<&str>,
    class_name: &str,
    method: &HirFunction,
    owner_target: &str,
    plan: &mut PythonInteropPlan,
) {
    for declaration in method
        .python_interop
        .iter()
        .filter(|declaration| declaration.kind == sifr_ir::PythonInteropDecoratorKind::Arrow)
    {
        plan.declarations.push(PythonInteropPlanDeclaration {
            module_name: module_name.map(str::to_string),
            function_name: format!("{class_name}.{}", method.name),
            declaration: declaration.clone(),
            certification_target: Some(owner_target.to_string()),
            parameter_types: method
                .params
                .iter()
                .map(|parameter| parameter.ty.clone())
                .collect(),
            return_type: method.return_type.clone(),
            callback_attachments: callback_attachments(declaration),
        });
    }
}

fn callback_attachments(
    declaration: &PythonInteropDeclaration,
) -> Vec<PythonCallbackAttachmentPlan> {
    declaration
        .callbacks
        .iter()
        .map(|callback| PythonCallbackAttachmentPlan {
            parameter_name: callback.parameter_name.clone(),
            lifetime: callback.lifetime,
            dispatch: callback.dispatch,
            concurrency: callback.concurrency,
            owner_class: callback.owner_class.clone(),
            owner_cleanup: callback.owner_cleanup,
        })
        .collect()
}

fn module_requires_raw_coroutine_loop(module: &HirModule) -> bool {
    let raw_call_names = module
        .imports
        .iter()
        .filter(|import| import.module == "sifr.python")
        .flat_map(|import| {
            import
                .names
                .iter()
                .filter(|name| *name == "run_coroutine_blocking")
                .map(|name| {
                    import
                        .aliases
                        .iter()
                        .find_map(|(original, alias)| (original == name).then(|| alias.clone()))
                        .unwrap_or_else(|| name.clone())
                })
        })
        .collect::<BTreeSet<_>>();
    if raw_call_names.is_empty() {
        return false;
    }
    let mut required = false;
    let mut inspect = |expression: &sifr_ir::HirExpr| {
        if matches!(expression, sifr_ir::HirExpr::Call { func, .. } if raw_call_names.contains(func))
        {
            required = true;
        }
    };
    for function in &module.functions {
        walk_stmts(
            &function.body,
            TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
            &mut |_| {},
            &mut inspect,
        );
    }
    for class in &module.classes {
        for method in &class.methods {
            walk_stmts(
                &method.body,
                TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
                &mut |_| {},
                &mut inspect,
            );
        }
    }
    required
}

fn record_expansion_functions(modules: &[(Option<&str>, &HirModule)]) -> BTreeSet<String> {
    let mut functions = BTreeSet::new();
    for (_, module) in modules {
        for owner in &module.functions {
            walk_stmts(
                &owner.body,
                TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
                &mut |_| {},
                &mut |expression| {
                    if let sifr_ir::HirExpr::PythonCall {
                        func,
                        record_expansions,
                        ..
                    } = expression
                    {
                        if !record_expansions.is_empty() {
                            functions.insert(func.clone());
                        }
                    }
                },
            );
        }
    }
    functions
}

pub(crate) fn push_python_plan_cache_key(out: &mut String, plan: &PythonInteropPlan) {
    out.push_str("python.binding_contract=");
    out.push_str(PYTHON_BINDING_CONTRACT_VERSION);
    out.push('\n');
    out.push_str("python.declarations=");
    out.push_str(&plan.declarations.len().to_string());
    out.push('\n');
    out.push_str("python.requires_async_loop=");
    out.push_str(if plan.requires_async_loop {
        "yes"
    } else {
        "no"
    });
    out.push('\n');
    for declaration in &plan.declarations {
        out.push_str("python.module=");
        out.push_str(declaration.module_name.as_deref().unwrap_or("<single>"));
        out.push('\n');
        out.push_str("python.function=");
        out.push_str(&declaration.function_name);
        out.push('\n');
        if let Some(target) = &declaration.declaration.target {
            out.push_str("python.target=");
            out.push_str(&target.dotted());
            out.push('\n');
        }
        if let Some(target) = &declaration.certification_target {
            out.push_str("python.certification_target=");
            out.push_str(target);
            out.push('\n');
        }
        out.push_str("python.declaration_kind=");
        let _ = write!(out, "{:?}", declaration.declaration.kind);
        out.push('\n');
        out.push_str("python.effect=");
        let _ = write!(out, "{:?}", declaration.declaration.effect);
        out.push('\n');
        out.push_str("python.cleanup=");
        let _ = write!(out, "{:?}", declaration.declaration.cleanup);
        out.push('\n');
        out.push_str("python.consumes_receiver=");
        out.push_str(if declaration.declaration.consumes_receiver {
            "yes"
        } else {
            "no"
        });
        out.push('\n');
        for parameter_type in &declaration.parameter_types {
            out.push_str("python.parameter_type=");
            let _ = write!(out, "{parameter_type}");
            out.push('\n');
        }
        out.push_str("python.return_type=");
        let _ = write!(out, "{}", declaration.return_type);
        out.push('\n');
        for parameter in &declaration.declaration.parameters {
            out.push_str("python.param=");
            out.push_str(&parameter.name);
            out.push(':');
            let _ = write!(out, "{:?}", parameter.kind);
            out.push(':');
            out.push_str(if parameter.omit_when_absent {
                "omit"
            } else {
                "pass"
            });
            out.push(':');
            out.push_str(if parameter.has_default {
                "default"
            } else {
                "required"
            });
            out.push('\n');
        }
        for callback in &declaration.callback_attachments {
            out.push_str("python.callback=");
            out.push_str(&callback.parameter_name);
            out.push(':');
            let _ = write!(out, "{:?}", callback.lifetime);
            out.push(':');
            let _ = write!(out, "{:?}", callback.dispatch);
            out.push(':');
            let _ = write!(out, "{:?}", callback.concurrency);
            out.push(':');
            out.push_str(callback.owner_class.as_deref().unwrap_or("<call-scope>"));
            out.push(':');
            let _ = write!(out, "{:?}", callback.owner_cleanup);
            out.push('\n');
        }
    }
    for root in &plan.required_import_roots {
        out.push_str("python.required_import=");
        out.push_str(root);
        out.push('\n');
    }
    for probe in &plan.target_probes {
        out.push_str("python.probe=");
        out.push_str(&probe.target_path);
        out.push(':');
        out.push_str(if probe.requires_inspectable_signature {
            "inspectable"
        } else {
            "runtime-checkable"
        });
        out.push(':');
        out.push_str(if probe.expects_type {
            "type"
        } else {
            "callable"
        });
        out.push(':');
        out.push_str(match probe.status {
            PythonTargetProbeStatus::Planned => "planned",
            PythonTargetProbeStatus::Verified => "verified",
            PythonTargetProbeStatus::RuntimeChecked => "runtime-checked",
        });
        out.push('\n');
    }
    out.push_str("python.bridge_packages=");
    out.push_str(&plan.bridge_packages.len().to_string());
    out.push('\n');
    for package in &plan.bridge_packages {
        out.push_str("python.bridge_package=");
        out.push_str(&package.package_id);
        out.push(':');
        out.push_str(&package.resolved_package_key);
        out.push(':');
        out.push_str(&package.runtime_package);
        out.push(':');
        out.push_str(&package.inventory_digest);
        out.push('\n');
        for module in &package.modules {
            out.push_str("python.bridge_module=");
            out.push_str(&module.module);
            out.push(':');
            out.push_str(&module.runtime_module);
            out.push(':');
            out.push_str(&module.source_path);
            out.push(':');
            out.push_str(&module.source_digest);
            out.push(':');
            out.push_str(if module.is_package {
                "package"
            } else {
                "module"
            });
            out.push('\n');
            for import in &module.imports {
                out.push_str("python.bridge_import=");
                match import {
                    PythonBridgeImportPlan::SamePackage {
                        module,
                        runtime_module,
                    } => {
                        out.push_str("same-package:");
                        out.push_str(module);
                        out.push(':');
                        out.push_str(runtime_module);
                    }
                    PythonBridgeImportPlan::ThirdParty { root } => {
                        out.push_str("third-party:");
                        out.push_str(root);
                    }
                }
                out.push('\n');
            }
        }
    }
}
