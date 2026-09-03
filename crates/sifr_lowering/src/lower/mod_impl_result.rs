use super::{HirModule, LowerCtx, LoweringResult};
use crate::lower::type_aliases::TypeAliasDecl;

pub(super) fn finish(
    module: HirModule,
    ctx: LowerCtx,
    alias_decls: &[TypeAliasDecl],
) -> LoweringResult {
    let flow_graph = crate::flow_graph::build_module_flow_graph(&module, &ctx.flow_effects);
    LoweringResult {
        module,
        flow_graph,
        class_field_defaults: ctx.class_field_defaults.clone(),
        declaration_metadata: ctx.declaration_metadata.clone(),
        class_adapter_providers: ctx.class_adapter_providers.clone(),
        class_adapter_markers: ctx.class_adapter_markers.clone(),
        attached_api_sets: ctx.attached_api_sets.clone(),
        attached_apis: ctx.attached_apis.clone(),
        class_adapter_selections: ctx.class_adapter_selections.clone(),
        descriptor_functions: ctx.descriptor_functions.clone(),
        declaration_descriptors: Vec::new(),
        applied_adapter_metadata: Vec::new(),
        type_aliases: ctx.scope.type_aliases().clone(),
        generic_type_aliases: alias_decls
            .iter()
            .filter(|decl| !decl.type_params.is_empty())
            .filter_map(|decl| {
                ctx.scope
                    .generic_type_aliases()
                    .get(&decl.name)
                    .map(|resolved| (decl.name.clone(), resolved.clone()))
            })
            .collect(),
        specialization_requests: ctx.specialization_requests.clone(),
        specialization_outputs: Vec::new(),
        json_integer_boundary_requests: ctx.json_integer_boundary_requests.clone(),
        function_defaults: ctx.function_defaults.clone(),
        function_varargs: ctx.vararg_functions.clone(),
        function_python_call_shapes: ctx.python_call_shapes.clone(),
        function_workloads: ctx
            .function_workload_annotations
            .iter()
            .map(|(name, workload)| (name.clone(), workload.label().to_string()))
            .collect(),
        constant_integer_values: ctx.const_integer_values.exported(),
        reveal_types: ctx.reveal_types,
        warnings: ctx.warnings,
    }
}
