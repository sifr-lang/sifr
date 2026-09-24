use super::discovery::ParsedProjectModule;
use crate::diagnostics::{RenderedDiagnostic, apply_diagnostic_recovery_limits, write_stderr_line};
pub(crate) use sifr_frontend::FrontendProduct as ProjectLowering;
use sifr_frontend::{
    FrontendDiagnosticStyle, FrontendProductInput, FrontendSourceContext, compile_frontend_product,
};
use sifr_lowering::{ExternalDefs, LoweringOptions};
use sifr_python_ast::Stmt;
#[cfg(test)]
use sifr_python_ast::Suite;
use std::collections::HashMap;

#[cfg(test)]
pub(crate) fn compile_frontend_modules(
    parsed_modules: &HashMap<String, Suite>,
    external_defs: ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
) -> Result<ProjectLowering, Vec<RenderedDiagnostic>> {
    // Test-only AST inputs have no source bytes. Preserve the no-source cycle
    // diagnostic before entering the source-backed production product.
    sifr_frontend::compute_module_compile_order(parsed_modules)?;
    let inputs = parsed_modules
        .iter()
        .map(|(name, suite)| {
            (
                name.clone(),
                FrontendProductInput {
                    suite,
                    source: "",
                    display_path: name,
                    source_backed: false,
                },
            )
        })
        .collect();
    compile_frontend_product(
        &inputs,
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
) -> Result<ProjectLowering, Vec<RenderedDiagnostic>> {
    let inputs = HashMap::from([(
        module_name.to_string(),
        FrontendProductInput {
            suite: stmts,
            source: source_context.source,
            display_path: source_context.display_path,
            source_backed: true,
        },
    )]);
    compile_frontend_product(&inputs, external_defs, diagnostic_style, lowering_options)
}

#[cfg(test)]
pub(crate) fn collect_project_hir_modules(
    parsed_modules: &HashMap<String, Suite>,
    external_defs: ExternalDefs,
) -> Result<ProjectLowering, Vec<RenderedDiagnostic>> {
    compile_frontend_modules(
        parsed_modules,
        external_defs,
        FrontendDiagnosticStyle::ModulePrefixed,
    )
}

pub(crate) fn collect_project_hir_source_modules(
    parsed_modules: &HashMap<String, ParsedProjectModule>,
    external_defs: ExternalDefs,
) -> Result<ProjectLowering, Vec<RenderedDiagnostic>> {
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
) -> Result<ProjectLowering, Vec<RenderedDiagnostic>> {
    let inputs = parsed_modules
        .iter()
        .map(|(name, module)| {
            (
                name.clone(),
                FrontendProductInput {
                    suite: &module.suite,
                    source: &module.source,
                    display_path: &module.display_path,
                    source_backed: true,
                },
            )
        })
        .collect();
    compile_frontend_product(
        &inputs,
        external_defs,
        FrontendDiagnosticStyle::ModulePrefixed,
        lowering_options,
    )
}

pub(crate) fn emit_project_frontend_diagnostics(project_lowering: &ProjectLowering) {
    let mut diagnostics = Vec::new();
    for module_name in &project_lowering.compile_order {
        let Some(diag) = project_lowering
            .module_diagnostics
            .get(module_name.as_str())
        else {
            continue;
        };
        diagnostics.extend(diag.rendered_warnings.clone());
        diagnostics.extend(diag.rendered_reveal_types.clone());
    }
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
