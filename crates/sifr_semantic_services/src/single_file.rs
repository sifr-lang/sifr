//! One canonical source-backed frontend entry for native and semantic callers.
use sifr_diagnostics::RenderedDiagnostic;
use sifr_frontend::{
    FrontendDiagnosticStyle, FrontendProduct, FrontendProductInput, FrontendSourceContext,
};
use sifr_lowering::{ExternalDefs, LoweringOptions};
use sifr_python_ast::Stmt;
use std::collections::HashMap;

pub fn compile_product(
    module_name: &str,
    stmts: &[Stmt],
    source_context: FrontendSourceContext<'_>,
    external_defs: ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
    lowering_options: &LoweringOptions,
) -> Result<FrontendProduct, Vec<RenderedDiagnostic>> {
    let inputs = HashMap::from([(
        module_name.to_string(),
        FrontendProductInput {
            suite: stmts,
            source: source_context.source,
            display_path: source_context.display_path,
            source_backed: true,
        },
    )]);
    sifr_frontend::compile_frontend_product(
        &inputs,
        external_defs,
        diagnostic_style,
        lowering_options,
    )
}

pub fn compile_sql_migration_source(
    compiler: &crate::SemanticContext,
    source: &str,
) -> Result<Vec<sifr_frontend::MigrationSourceDeclaration>, Vec<RenderedDiagnostic>> {
    let lowered = lower_source(compiler, source)?;
    sifr_frontend::sql_migration_declarations(&lowered.module).map_err(|error| {
        vec![crate::diagnostics::diagnostic_with_code(
            error.message,
            sifr_diagnostics::DiagnosticCode::SQL_PROVIDER_CONTRACT,
        )]
    })
}

pub fn lower_source(
    compiler: &crate::SemanticContext,
    source: &str,
) -> Result<sifr_ir::LoweringResult, Vec<RenderedDiagnostic>> {
    let defs = compiler.external_defs()?;
    let suite = sifr_frontend::parse_source(source, None)?;
    let mut product = compile_product(
        "main",
        &suite,
        FrontendSourceContext {
            source,
            display_path: "main",
        },
        defs,
        FrontendDiagnosticStyle::Bare,
        &LoweringOptions::default(),
    )?;
    crate::editor::compile_application_queries(
        &mut product,
        &crate::sql_editor::PreparedSqlProfiles::default(),
    )?;
    into_lowering(product)
}

/// Preserve the native single-file projection, including required flow graphs.
pub fn into_lowering(
    mut project_lowering: FrontendProduct,
) -> Result<sifr_ir::LoweringResult, Vec<RenderedDiagnostic>> {
    use sifr_diagnostics::DiagnosticCode;
    use sifr_ir::LoweringResult;
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
    Ok(lowering_result)
}
