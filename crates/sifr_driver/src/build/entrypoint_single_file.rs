use super::super::frontend::FrontendCompiled;
use super::super::project::ProjectLowering;
use super::super::stdlib::StdlibCompiled;
use crate::diagnostics::RenderedDiagnostic;
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::LoweringResult;

pub(super) fn into_frontend(
    stdlib: StdlibCompiled,
    mut project_lowering: ProjectLowering,
) -> Result<FrontendCompiled, Vec<RenderedDiagnostic>> {
    let main_module = project_lowering.hir_modules.remove("main").ok_or_else(|| {
        vec![crate::diagnostics::diagnostic_with_code(
            "internal error: frontend lowering missing 'main' module",
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        )]
    })?;
    let flow_graph = project_lowering.flow_graphs.remove("main").ok_or_else(|| {
        vec![crate::diagnostics::diagnostic_with_code(
            "internal error: frontend lowering missing 'main' flow graph",
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        )]
    })?;
    let main_diag = project_lowering
        .module_diagnostics
        .remove("main")
        .unwrap_or_default();
    let specialization_metadata = project_lowering
        .external_defs
        .take_module_specialization_metadata("main");
    let lowering_result = LoweringResult {
        module: main_module,
        flow_graph,
        class_field_defaults: specialization_metadata.class_field_defaults,
        declaration_metadata: specialization_metadata.declaration_metadata,
        class_adapter_providers: specialization_metadata.class_adapter_providers,
        class_adapter_markers: specialization_metadata.class_adapter_markers,
        attached_api_sets: specialization_metadata.attached_api_sets,
        attached_apis: specialization_metadata.attached_apis,
        class_adapter_selections: specialization_metadata.class_adapter_selections,
        descriptor_functions: specialization_metadata.descriptor_functions,
        declaration_descriptors: specialization_metadata.declaration_descriptors,
        applied_adapter_metadata: specialization_metadata.applied_adapter_metadata,
        type_aliases: std::collections::HashMap::new(),
        generic_type_aliases: std::collections::HashMap::new(),
        specialization_requests: specialization_metadata.specialization_requests,
        specialization_outputs: specialization_metadata.specialization_outputs,
        json_integer_boundary_requests: specialization_metadata.json_integer_boundary_requests,
        function_defaults: std::collections::HashMap::new(),
        function_varargs: std::collections::HashMap::new(),
        function_python_call_shapes: std::collections::HashMap::new(),
        function_workloads: std::collections::HashMap::new(),
        constant_integer_values: std::collections::HashMap::new(),
        reveal_types: main_diag.reveal_types,
        warnings: main_diag.warnings,
    };
    Ok(FrontendCompiled {
        stdlib,
        lowering_result,
    })
}
