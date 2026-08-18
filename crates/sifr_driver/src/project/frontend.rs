#[cfg(test)]
use super::compile_order::compute_module_compile_order;
use super::compile_order::{compute_module_compile_order_with_sources, CompileOrderSourceModule};
use super::discovery::ParsedProjectModule;
use crate::diagnostics::{apply_diagnostic_recovery_limits, write_stderr_line, RenderedDiagnostic};
use sifr_diagnostics::DiagnosticCode;
#[cfg(test)]
use sifr_frontend::compile_module_hir;
use sifr_frontend::{
    collect_module_exports, compile_module_hir_with_source_and_options, erase_marker_imports,
    reveal_type_diagnostics, warning_diagnostics, FrontendDiagnosticStyle,
    FrontendModuleDiagnostics, FrontendSourceContext,
};
use sifr_ir::FlowGraph;
use sifr_lowering::{ExternalDefs, HirModule, LoweringOptions, LoweringResult};
use sifr_python_ast::Stmt;
use std::collections::HashMap;

pub(crate) struct ProjectLowering {
    pub(crate) hir_modules: HashMap<String, HirModule>,
    pub(crate) flow_graphs: HashMap<String, FlowGraph>,
    pub(crate) external_defs: ExternalDefs,
    pub(crate) compile_order: Vec<String>,
    pub(crate) module_diagnostics: HashMap<String, FrontendModuleDiagnostics>,
}

#[cfg(test)]
pub(crate) fn compile_frontend_modules(
    parsed_modules: &HashMap<String, Vec<Stmt>>,
    mut external_defs: ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
) -> Result<ProjectLowering, Vec<RenderedDiagnostic>> {
    let mut hir_modules: HashMap<String, HirModule> = HashMap::new();
    let mut flow_graphs: HashMap<String, FlowGraph> = HashMap::new();
    let mut module_diagnostics: HashMap<String, FrontendModuleDiagnostics> = HashMap::new();
    let compile_order = compute_module_compile_order(parsed_modules)?;

    for module_name in &compile_order {
        let Some(stmts) = parsed_modules.get(module_name.as_str()) else {
            return Err(vec![crate::diagnostics::diagnostic_with_code(
                format!("[{module_name}] module was not parsed"),
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
            )]);
        };
        let result = compile_module_hir(module_name, stmts, &external_defs, diagnostic_style)?;
        let LoweringResult {
            mut module,
            flow_graph,
            class_field_defaults,
            declaration_metadata,
            class_adapter_providers,
            class_adapter_markers,
            class_adapter_selections,
            descriptor_functions,
            declaration_descriptors,
            applied_adapter_metadata,
            type_aliases,
            specialization_requests,
            specialization_outputs,
            json_integer_boundary_requests,
            function_defaults,
            function_varargs,
            function_python_call_shapes,
            function_workloads,
            constant_integer_values,
            reveal_types,
            warnings,
        } = result;
        let lowering_result = LoweringResult {
            module: module.clone(),
            flow_graph: flow_graph.clone(),
            class_field_defaults,
            declaration_metadata,
            class_adapter_providers,
            class_adapter_markers,
            class_adapter_selections,
            descriptor_functions,
            declaration_descriptors,
            applied_adapter_metadata,
            type_aliases,
            specialization_requests,
            specialization_outputs,
            json_integer_boundary_requests,
            function_defaults,
            function_varargs,
            function_python_call_shapes,
            function_workloads,
            constant_integer_values,
            reveal_types: reveal_types.clone(),
            warnings: warnings.clone(),
        };
        collect_module_exports(module_name, &lowering_result, &mut external_defs);
        erase_marker_imports(&mut module, &external_defs);
        hir_modules.insert(module_name.clone(), module);
        flow_graphs.insert(module_name.clone(), flow_graph);
        module_diagnostics.insert(
            module_name.clone(),
            FrontendModuleDiagnostics {
                rendered_reveal_types: reveal_type_diagnostics(None, &reveal_types),
                reveal_types,
                rendered_warnings: warning_diagnostics(None, &warnings),
                warnings,
            },
        );
    }

    Ok(ProjectLowering {
        hir_modules,
        flow_graphs,
        external_defs,
        compile_order,
        module_diagnostics,
    })
}

pub(crate) fn compile_single_frontend_module_with_source_and_options(
    module_name: &str,
    stmts: &[Stmt],
    source_context: FrontendSourceContext<'_>,
    mut external_defs: ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
    lowering_options: LoweringOptions,
) -> Result<ProjectLowering, Vec<RenderedDiagnostic>> {
    let result = compile_module_hir_with_source_and_options(
        module_name,
        stmts,
        &external_defs,
        diagnostic_style,
        Some(source_context),
        lowering_options,
    )?;
    let LoweringResult {
        mut module,
        flow_graph,
        class_field_defaults,
        declaration_metadata,
        class_adapter_providers,
        class_adapter_markers,
        class_adapter_selections,
        descriptor_functions,
        declaration_descriptors,
        applied_adapter_metadata,
        type_aliases,
        specialization_requests,
        specialization_outputs,
        json_integer_boundary_requests,
        function_defaults,
        function_varargs,
        function_python_call_shapes,
        function_workloads,
        constant_integer_values,
        reveal_types,
        warnings,
    } = result;
    let lowering_result = LoweringResult {
        module: module.clone(),
        flow_graph: flow_graph.clone(),
        class_field_defaults,
        declaration_metadata,
        class_adapter_providers,
        class_adapter_markers,
        class_adapter_selections,
        descriptor_functions,
        declaration_descriptors,
        applied_adapter_metadata,
        type_aliases,
        specialization_requests,
        specialization_outputs,
        json_integer_boundary_requests,
        function_defaults,
        function_varargs,
        function_python_call_shapes,
        function_workloads,
        constant_integer_values,
        reveal_types: reveal_types.clone(),
        warnings: warnings.clone(),
    };
    collect_module_exports(module_name, &lowering_result, &mut external_defs);
    erase_marker_imports(&mut module, &external_defs);

    Ok(ProjectLowering {
        hir_modules: HashMap::from([(module_name.to_string(), module)]),
        flow_graphs: HashMap::from([(module_name.to_string(), flow_graph)]),
        external_defs,
        compile_order: vec![module_name.to_string()],
        module_diagnostics: HashMap::from([(
            module_name.to_string(),
            FrontendModuleDiagnostics {
                rendered_reveal_types: reveal_type_diagnostics(Some(source_context), &reveal_types),
                reveal_types,
                rendered_warnings: warning_diagnostics(Some(source_context), &warnings),
                warnings,
            },
        )]),
    })
}

#[cfg(test)]
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
    mut external_defs: ExternalDefs,
    lowering_options: &LoweringOptions,
) -> Result<ProjectLowering, Vec<RenderedDiagnostic>> {
    let suites: HashMap<String, CompileOrderSourceModule<'_>> = parsed_modules
        .iter()
        .map(|(name, module)| {
            (
                name.clone(),
                CompileOrderSourceModule {
                    suite: &module.suite,
                    source: &module.source,
                    display_path: &module.display_path,
                },
            )
        })
        .collect();
    let compile_order = compute_module_compile_order_with_sources(&suites)?;
    let mut hir_modules: HashMap<String, HirModule> = HashMap::new();
    let mut flow_graphs: HashMap<String, FlowGraph> = HashMap::new();
    let mut module_diagnostics: HashMap<String, FrontendModuleDiagnostics> = HashMap::new();

    for module_name in &compile_order {
        let Some(parsed_module) = parsed_modules.get(module_name.as_str()) else {
            return Err(vec![crate::diagnostics::diagnostic_with_code(
                format!("[{module_name}] module was not parsed"),
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
            )]);
        };
        let result = compile_module_hir_with_source_and_options(
            module_name,
            &parsed_module.suite,
            &external_defs,
            FrontendDiagnosticStyle::ModulePrefixed,
            Some(FrontendSourceContext {
                display_path: &parsed_module.display_path,
                source: &parsed_module.source,
            }),
            lowering_options.clone(),
        )?;
        let source_context = FrontendSourceContext {
            display_path: &parsed_module.display_path,
            source: &parsed_module.source,
        };
        let LoweringResult {
            mut module,
            flow_graph,
            class_field_defaults,
            declaration_metadata,
            class_adapter_providers,
            class_adapter_markers,
            class_adapter_selections,
            descriptor_functions,
            declaration_descriptors,
            applied_adapter_metadata,
            type_aliases,
            specialization_requests,
            specialization_outputs,
            json_integer_boundary_requests,
            function_defaults,
            function_varargs,
            function_python_call_shapes,
            function_workloads,
            constant_integer_values,
            reveal_types,
            warnings,
        } = result;
        let lowering_result = LoweringResult {
            module: module.clone(),
            flow_graph: flow_graph.clone(),
            class_field_defaults,
            declaration_metadata,
            class_adapter_providers,
            class_adapter_markers,
            class_adapter_selections,
            descriptor_functions,
            declaration_descriptors,
            applied_adapter_metadata,
            type_aliases,
            specialization_requests,
            specialization_outputs,
            json_integer_boundary_requests,
            function_defaults,
            function_varargs,
            function_python_call_shapes,
            function_workloads,
            constant_integer_values,
            reveal_types: reveal_types.clone(),
            warnings: warnings.clone(),
        };
        collect_module_exports(module_name, &lowering_result, &mut external_defs);
        erase_marker_imports(&mut module, &external_defs);
        hir_modules.insert(module_name.clone(), module);
        flow_graphs.insert(module_name.clone(), flow_graph);
        module_diagnostics.insert(
            module_name.clone(),
            FrontendModuleDiagnostics {
                rendered_reveal_types: reveal_type_diagnostics(Some(source_context), &reveal_types),
                reveal_types,
                rendered_warnings: warning_diagnostics(Some(source_context), &warnings),
                warnings,
            },
        );
    }

    Ok(ProjectLowering {
        hir_modules,
        flow_graphs,
        external_defs,
        compile_order,
        module_diagnostics,
    })
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
