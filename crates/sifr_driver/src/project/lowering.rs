use crate::diagnostics::{write_stderr_line, CompileError, CompilePhase};
use crate::frontend::{lower_frontend_module, FrontendDiagnosticStyle, FrontendModuleDiagnostics};
use crate::project::graph::compute_module_compile_order;
use sifr_hir::{ExternalDefs, HirModule, LoweringResult};
use sifr_python_ast::Stmt;
use sifr_type_system::{FunctionType, ParamConvention, Type};
use std::collections::HashMap;

pub(crate) struct ProjectLowering {
    pub(crate) hir_modules: HashMap<String, HirModule>,
    pub(crate) external_defs: ExternalDefs,
    pub(crate) compile_order: Vec<String>,
    pub(crate) module_diagnostics: HashMap<String, FrontendModuleDiagnostics>,
}

fn collect_module_exports(module_name: &str, module: &HirModule, external_defs: &mut ExternalDefs) {
    let mut fn_exports = HashMap::new();
    let mut class_exports = HashMap::new();
    let mut class_type_param_exports = HashMap::new();
    let mut const_exports = HashMap::new();

    for func in &module.functions {
        if !func.name.starts_with('_') {
            let params: Vec<(String, Type, ParamConvention)> = func
                .params
                .iter()
                .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                .collect();
            fn_exports.insert(
                func.name.clone(),
                FunctionType {
                    params,
                    return_type: Box::new(func.return_type.clone()),
                },
            );
        }
    }

    for class in &module.classes {
        if !class.name.starts_with('_') {
            let mut methods: Vec<(String, FunctionType)> = class
                .methods
                .iter()
                .filter(|m| m.name != "new")
                .map(|m| {
                    let params: Vec<(String, Type, ParamConvention)> = m
                        .params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                        .collect();
                    (
                        m.name.clone(),
                        FunctionType {
                            params,
                            return_type: Box::new(m.return_type.clone()),
                        },
                    )
                })
                .collect();
            for (dunder_name, op_func) in &class.operator_impls {
                let params: Vec<(String, Type, ParamConvention)> = op_func
                    .params
                    .iter()
                    .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                    .collect();
                methods.push((
                    dunder_name.clone(),
                    FunctionType {
                        params,
                        return_type: Box::new(op_func.return_type.clone()),
                    },
                ));
            }
            let class_ty = Type::Class {
                name: class.name.clone(),
                fields: class.fields.clone(),
                methods,
                parent_class: None,
            };
            class_exports.insert(class.name.clone(), class_ty);
            if !class.type_params.is_empty() {
                class_type_param_exports.insert(class.name.clone(), class.type_params.clone());
            }
        }
    }

    for (name, ty, _) in &module.constants {
        if !name.starts_with('_') {
            const_exports.insert(name.clone(), ty.clone());
        }
    }

    external_defs
        .functions
        .insert(module_name.to_string(), fn_exports);
    external_defs
        .classes
        .insert(module_name.to_string(), class_exports);
    if !class_type_param_exports.is_empty() {
        external_defs
            .class_type_params
            .insert(module_name.to_string(), class_type_param_exports);
    }
    external_defs
        .constants
        .insert(module_name.to_string(), const_exports);
}

pub(crate) fn compile_frontend_modules(
    parsed_modules: &HashMap<String, Vec<Stmt>>,
    mut external_defs: ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
) -> Result<ProjectLowering, Vec<CompileError>> {
    let mut hir_modules: HashMap<String, HirModule> = HashMap::new();
    let mut module_diagnostics: HashMap<String, FrontendModuleDiagnostics> = HashMap::new();
    let compile_order = compute_module_compile_order(parsed_modules)?;

    for module_name in &compile_order {
        let Some(stmts) = parsed_modules.get(module_name.as_str()) else {
            return Err(vec![CompileError {
                message: format!("[{module_name}] module was not parsed"),
                phase: CompilePhase::Build,
            }]);
        };
        let result = lower_frontend_module(module_name, stmts, &external_defs, diagnostic_style)?;
        let LoweringResult {
            module,
            reveal_types,
            warnings,
        } = result;
        collect_module_exports(module_name, &module, &mut external_defs);
        hir_modules.insert(module_name.clone(), module);
        module_diagnostics.insert(
            module_name.clone(),
            FrontendModuleDiagnostics {
                reveal_types,
                warnings,
            },
        );
    }

    Ok(ProjectLowering {
        hir_modules,
        external_defs,
        compile_order,
        module_diagnostics,
    })
}

pub(crate) fn collect_project_hir_modules(
    parsed_modules: &HashMap<String, Vec<Stmt>>,
    external_defs: ExternalDefs,
) -> Result<ProjectLowering, Vec<CompileError>> {
    compile_frontend_modules(
        parsed_modules,
        external_defs,
        FrontendDiagnosticStyle::ModulePrefixed,
    )
}

pub(crate) fn emit_project_frontend_diagnostics(project_lowering: &ProjectLowering) {
    for module_name in &project_lowering.compile_order {
        let Some(diag) = project_lowering
            .module_diagnostics
            .get(module_name.as_str())
        else {
            continue;
        };
        for message in &diag.reveal_types {
            write_stderr_line(message);
        }
        for warning in &diag.warnings {
            write_stderr_line(warning);
        }
    }
}
