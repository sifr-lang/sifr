use super::compile_order::compute_module_compile_order;
use super::exports::collect_module_exports;
use crate::diagnostics::{write_stderr_line, RenderedDiagnostic};
use crate::frontend::{lower_frontend_module, FrontendDiagnosticStyle, FrontendModuleDiagnostics};
use sifr_diagnostics::DiagnosticCode;
use sifr_hir::{ExternalDefs, HirModule, LoweringResult};
use sifr_python_ast::Stmt;
use std::collections::HashMap;

pub(crate) struct ProjectLowering {
    pub(crate) hir_modules: HashMap<String, HirModule>,
    pub(crate) external_defs: ExternalDefs,
    pub(crate) compile_order: Vec<String>,
    pub(crate) module_diagnostics: HashMap<String, FrontendModuleDiagnostics>,
}

pub(crate) fn compile_frontend_modules(
    parsed_modules: &HashMap<String, Vec<Stmt>>,
    mut external_defs: ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
) -> Result<ProjectLowering, Vec<RenderedDiagnostic>> {
    let mut hir_modules: HashMap<String, HirModule> = HashMap::new();
    let mut module_diagnostics: HashMap<String, FrontendModuleDiagnostics> = HashMap::new();
    let compile_order = compute_module_compile_order(parsed_modules)?;

    for module_name in &compile_order {
        let Some(stmts) = parsed_modules.get(module_name.as_str()) else {
            return Err(vec![crate::diagnostics::diagnostic_with_code(
                format!("[{module_name}] module was not parsed"),
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
            )]);
        };
        let result = lower_frontend_module(module_name, stmts, &external_defs, diagnostic_style)?;
        let LoweringResult {
            module,
            function_defaults,
            function_varargs,
            reveal_types,
            warnings,
        } = result;
        let lowering_result = LoweringResult {
            module: module.clone(),
            function_defaults,
            function_varargs,
            reveal_types: reveal_types.clone(),
            warnings: warnings.clone(),
        };
        collect_module_exports(module_name, &lowering_result, &mut external_defs);
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
) -> Result<ProjectLowering, Vec<RenderedDiagnostic>> {
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
