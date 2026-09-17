use super::{Record, RecordId, References, SemanticExports, StructuralMethodExport, sealed};
impl References for SemanticExports {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.functions.references(out);
        self.compiler_intrinsics.references(out);
        self.classes.references(out);
        self.generic_type_aliases.references(out);
        self.class_instance_methods.references(out);
        self.rust_consuming_methods.references(out);
        self.rust_opaque_classes.references(out);
        self.rust_structural_classes.references(out);
        self.class_type_params.references(out);
        self.structural_methods.references(out);
        self.class_field_defaults.references(out);
        self.declaration_metadata.references(out);
        self.class_adapter_providers.references(out);
        self.class_adapter_markers.references(out);
        self.attached_api_sets.references(out);
        self.attached_apis.references(out);
        self.class_adapter_selections.references(out);
        self.descriptor_functions.references(out);
        self.declaration_descriptors.references(out);
        self.applied_adapter_metadata.references(out);
        self.const_functions.references(out);
        self.specialization_requests.references(out);
        self.specialization_outputs.references(out);
        self.json_integer_boundary_requests.references(out);
        self.constants.references(out);
        self.constant_integer_values.references(out);
        self.error_types.references(out);
        self.type_param_bounds.references(out);
        self.generic_functions.references(out);
        self.function_varargs.references(out);
        self.function_python_call_shapes.references(out);
        self.rust_threadsafe_callback_targets.references(out);
        self.function_workloads.references(out);
        self.function_defaults.references(out);
    }
}
impl sealed::Sealed for SemanticExports {}
impl Record for SemanticExports {
    const KIND: u16 = 115;
}
impl References for StructuralMethodExport {
    fn references(&self, out: &mut Vec<(RecordId, u16)>) {
        let _ = &out;
        self.handler_target.references(out);
        self.name.references(out);
        self.params.references(out);
        self.return_type.references(out);
        self.is_async.references(out);
        self.method_kind.references(out);
        self.receiver.references(out);
    }
}
impl sealed::Sealed for StructuralMethodExport {}
impl Record for StructuralMethodExport {
    const KIND: u16 = 125;
}
