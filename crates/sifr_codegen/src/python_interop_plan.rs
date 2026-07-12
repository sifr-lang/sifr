use sifr_ir::{HirFunction, HirModule, PythonInteropDeclaration};
use sifr_type_system::Type;
use std::collections::BTreeSet;
use std::fmt::Write;

use crate::hir_analysis::traversal::{walk_stmts, TraversalConfig};

/// Build-time authority for declaration-first Python probing and wrapper inputs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PythonInteropPlan {
    pub declarations: Vec<PythonInteropPlanDeclaration>,
    pub required_import_roots: Vec<String>,
    pub target_probes: Vec<PythonTargetProbe>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonInteropPlanDeclaration {
    pub module_name: Option<String>,
    pub function_name: String,
    pub declaration: PythonInteropDeclaration,
    pub parameter_types: Vec<Type>,
    pub return_type: Type,
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
        for function in &module.functions {
            collect_function(
                module_name,
                function,
                &record_expansion_functions,
                &mut plan,
            );
        }
        for class in &module.classes {
            let Some(declaration) = class.python_opaque_declaration() else {
                continue;
            };
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
                    parameter_types: Vec::new(),
                    return_type: Type::None,
                });
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
            parameter_types: function
                .params
                .iter()
                .map(|param| param.ty.clone())
                .collect(),
            return_type: function.return_type.clone(),
        });
    }
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
    out.push_str("python.declarations=");
    out.push_str(&plan.declarations.len().to_string());
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
}
