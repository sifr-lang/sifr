use crate::{
    CompileOrderSourceModule, FrontendDiagnosticStyle, FrontendModuleDiagnostics,
    FrontendSourceContext, collect_module_exports, compile_module_hir_with_source_and_options,
    compute_module_compile_order_with_sources, erase_marker_imports, prepare_external_defs,
    reveal_type_diagnostics, warning_diagnostics,
};
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use sifr_ir::FlowGraph;
use sifr_lowering::{ExternalDefs, HirModule, LoweringOptions, LoweringResult};
use sifr_python_ast::Stmt;
use std::collections::HashMap;

/// A borrowed view of a pinned input snapshot. All consumers compile these parsed statements
/// and source bytes rather than resolving a second filesystem view.
pub struct FrontendProductInput<'a> {
    pub suite: &'a [Stmt],
    pub source: &'a str,
    pub display_path: &'a str,
    pub source_backed: bool,
}

/// The completed frontend result used by single-file, project, package and test
/// compilation. Export collection runs while the complete lowering result is owned
/// here; only the HIR and flow graph are retained for later code generation.
pub struct FrontendProduct {
    pub hir_modules: HashMap<String, HirModule>,
    pub flow_graphs: HashMap<String, FlowGraph>,
    pub external_defs: ExternalDefs,
    pub compile_order: Vec<String>,
    pub module_diagnostics: HashMap<String, FrontendModuleDiagnostics>,
}

pub fn compile_frontend_product(
    inputs: &HashMap<String, FrontendProductInput<'_>>,
    mut external_defs: ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
    lowering_options: &LoweringOptions,
) -> Result<FrontendProduct, Vec<RenderedDiagnostic>> {
    let order_inputs = inputs
        .iter()
        .map(|(name, input)| {
            (
                name.clone(),
                CompileOrderSourceModule {
                    suite: input.suite,
                    source: input.source,
                    display_path: input.display_path,
                },
            )
        })
        .collect();
    let compile_order = compute_module_compile_order_with_sources(&order_inputs)?;
    let mut hir_modules = HashMap::new();
    let mut flow_graphs = HashMap::new();
    let mut module_diagnostics = HashMap::new();
    for module_name in &compile_order {
        let Some(input) = inputs.get(module_name) else {
            return Err(vec![crate::diagnostic_with_code(
                format!("[{module_name}] module was not parsed"),
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
            )]);
        };
        let source_context = input.source_backed.then_some(FrontendSourceContext {
            display_path: input.display_path,
            source: input.source,
        });
        let (mut result, diagnostics) = compile_frontend_product_module(
            module_name,
            input.suite,
            source_context,
            &mut external_defs,
            diagnostic_style,
            lowering_options.clone(),
        )?;
        erase_marker_imports(&mut result.module, &external_defs);
        hir_modules.insert(module_name.clone(), result.module);
        flow_graphs.insert(module_name.clone(), result.flow_graph);
        module_diagnostics.insert(module_name.clone(), diagnostics);
    }
    Ok(FrontendProduct {
        hir_modules,
        flow_graphs,
        external_defs,
        compile_order,
        module_diagnostics,
    })
}

pub fn compile_frontend_product_module(
    module_name: &str,
    stmts: &[Stmt],
    source_context: Option<FrontendSourceContext<'_>>,
    external_defs: &mut ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
    lowering_options: LoweringOptions,
) -> Result<(LoweringResult, FrontendModuleDiagnostics), Vec<RenderedDiagnostic>> {
    prepare_external_defs(stmts, external_defs)?;
    let result = compile_module_hir_with_source_and_options(
        module_name,
        stmts,
        external_defs,
        diagnostic_style,
        source_context,
        lowering_options,
    )?;
    crate::validate_sql_schema_witness_module(
        &result.module,
        Some(&result.module.type_param_bounds),
    )
    .map_err(|error| {
        vec![crate::diagnostic_with_code(
            error.message,
            DiagnosticCode::SQL_PROVIDER_CONTRACT,
        )]
    })?;
    collect_module_exports(module_name, &result, external_defs);
    let diagnostics = FrontendModuleDiagnostics {
        rendered_reveal_types: reveal_type_diagnostics(source_context, &result.reveal_types),
        reveal_types: result.reveal_types.clone(),
        rendered_warnings: warning_diagnostics(source_context, &result.warnings),
        warnings: result.warnings.clone(),
    };
    Ok((result, diagnostics))
}
