use super::discovery::ParsedProjectModule;
use crate::diagnostics::{RenderedDiagnostic, apply_diagnostic_recovery_limits, write_stderr_line};
use sifr_frontend::{
    FrontendContext, FrontendDiagnosticStyle, FrontendProjectCompilation, FrontendProjectModule,
    FrontendSourceContext,
};
use sifr_lowering::{ExternalDefs, LoweringOptions};
use sifr_python_ast::Stmt;
#[cfg(test)]
use sifr_python_ast::Suite;
use std::collections::{BTreeMap, HashMap};

pub(crate) type ProjectCompilation = FrontendProjectCompilation;

#[cfg(test)]
pub(crate) fn compute_module_compile_order(
    modules: &HashMap<String, Suite>,
) -> Result<Vec<String>, Vec<RenderedDiagnostic>> {
    let modules = modules
        .iter()
        .map(|(name, suite)| {
            (
                name.clone(),
                FrontendProjectModule {
                    suite: suite.clone(),
                    source: String::new(),
                    display_path: String::new(),
                },
            )
        })
        .collect();
    sifr_frontend::compute_project_compile_order(&modules)
}

#[cfg(test)]
pub(crate) fn compile_frontend_modules(
    parsed_modules: &HashMap<String, Suite>,
    external_defs: ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
) -> Result<ProjectCompilation, Vec<RenderedDiagnostic>> {
    let modules = parsed_modules
        .iter()
        .map(|(name, suite)| {
            (
                name.clone(),
                FrontendProjectModule {
                    suite: suite.clone(),
                    source: String::new(),
                    display_path: String::new(),
                },
            )
        })
        .collect();
    compile_project_source_modules(
        &modules,
        external_defs,
        diagnostic_style,
        &LoweringOptions::default(),
    )
}

pub(crate) fn compile_single_frontend_module_with_source_and_options(
    module_name: &str,
    stmts: &[Stmt],
    source_context: FrontendSourceContext<'_>,
    external_defs: ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
    lowering_options: &LoweringOptions,
) -> Result<ProjectCompilation, Vec<RenderedDiagnostic>> {
    let modules = BTreeMap::from([(
        module_name.to_string(),
        FrontendProjectModule {
            suite: stmts.iter().cloned().collect(),
            source: source_context.source.to_string(),
            display_path: source_context.display_path.to_string(),
        },
    )]);
    compile_project_source_modules(&modules, external_defs, diagnostic_style, lowering_options)
}

#[cfg(test)]
pub(crate) fn collect_project_hir_modules(
    parsed_modules: &HashMap<String, Suite>,
    external_defs: ExternalDefs,
) -> Result<ProjectCompilation, Vec<RenderedDiagnostic>> {
    compile_frontend_modules(
        parsed_modules,
        external_defs,
        FrontendDiagnosticStyle::ModulePrefixed,
    )
}

pub(crate) fn collect_project_hir_source_modules(
    parsed_modules: &HashMap<String, ParsedProjectModule>,
    external_defs: ExternalDefs,
) -> Result<ProjectCompilation, Vec<RenderedDiagnostic>> {
    collect_project_hir_source_modules_with_options(
        parsed_modules,
        external_defs,
        &LoweringOptions::default(),
    )
}

pub(crate) fn collect_project_hir_source_modules_with_options(
    parsed_modules: &HashMap<String, ParsedProjectModule>,
    external_defs: ExternalDefs,
    lowering_options: &LoweringOptions,
) -> Result<ProjectCompilation, Vec<RenderedDiagnostic>> {
    let modules = parsed_modules
        .iter()
        .map(|(name, module)| (name.clone(), module.clone()))
        .collect();
    compile_project_source_modules(
        &modules,
        external_defs,
        FrontendDiagnosticStyle::ModulePrefixed,
        lowering_options,
    )
}

pub(crate) fn compile_project_source_modules(
    modules: &BTreeMap<String, FrontendProjectModule>,
    external_defs: ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
    lowering_options: &LoweringOptions,
) -> Result<ProjectCompilation, Vec<RenderedDiagnostic>> {
    FrontendContext::compile_project_modules(
        modules,
        external_defs,
        diagnostic_style,
        lowering_options,
    )
}

pub(crate) fn emit_project_frontend_diagnostics(project_lowering: &ProjectCompilation) {
    let diagnostics = project_lowering
        .compile_order
        .iter()
        .filter_map(|module_name| project_lowering.module_diagnostics.get(module_name))
        .flat_map(|diagnostics| {
            diagnostics
                .rendered_warnings
                .clone()
                .into_iter()
                .chain(diagnostics.rendered_reveal_types.clone())
        })
        .collect::<Vec<_>>();
    for diagnostic in apply_diagnostic_recovery_limits(&diagnostics) {
        write_stderr_line(&format!(
            "{}: {}",
            diagnostic_severity_label(diagnostic.severity),
            diagnostic.message
        ));
        for child in diagnostic.children {
            write_stderr_line(&format!(
                "{}: {}",
                child_diagnostic_severity_label(child.severity),
                child.message
            ));
        }
        if let Some(help) = diagnostic.help {
            write_stderr_line(&format!("help: {help}"));
        }
    }
}

fn diagnostic_severity_label(severity: sifr_diagnostics::Severity) -> &'static str {
    match severity {
        sifr_diagnostics::Severity::Error => "error",
        sifr_diagnostics::Severity::Warning => "warning",
        sifr_diagnostics::Severity::Note => "note",
    }
}

fn child_diagnostic_severity_label(severity: sifr_diagnostics::ChildSeverity) -> &'static str {
    match severity {
        sifr_diagnostics::ChildSeverity::Note => "note",
        sifr_diagnostics::ChildSeverity::Help => "help",
    }
}
