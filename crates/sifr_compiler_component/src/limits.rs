//! Resource policy values shared by protocol validation and native execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentHostLimits {
    pub fuel: u64,
    pub max_component_bytes: usize,
    pub max_memory_bytes: usize,
    pub max_wasm_stack_bytes: usize,
    pub max_instances: usize,
    pub max_memories: usize,
    pub max_tables: usize,
    pub max_input_bytes: usize,
    pub max_output_bytes: usize,
    pub max_template_parts: usize,
    pub max_static_segment_bytes: usize,
    pub max_holes: usize,
    pub max_context_artifacts: usize,
    pub max_context_artifact_bytes: usize,
    pub max_dependencies: usize,
    pub max_diagnostics: usize,
    pub max_diagnostic_bytes: usize,
    pub max_source_map_entries: usize,
    pub max_type_depth: usize,
    pub max_type_width: usize,
    pub max_operation_depth: usize,
    pub max_operations: usize,
}

impl Default for ComponentHostLimits {
    fn default() -> Self {
        Self {
            fuel: 10_000_000,
            max_component_bytes: 64 * 1024 * 1024,
            max_memory_bytes: 256 * 1024 * 1024,
            max_wasm_stack_bytes: 2 * 1024 * 1024,
            max_instances: 32,
            max_memories: 16,
            max_tables: 32,
            max_input_bytes: 64 * 1024 * 1024,
            max_output_bytes: 64 * 1024 * 1024,
            max_template_parts: 16_384,
            max_static_segment_bytes: 1024 * 1024,
            max_holes: 4_096,
            max_context_artifacts: 256,
            max_context_artifact_bytes: 16 * 1024 * 1024,
            max_dependencies: 16_384,
            max_diagnostics: 4_096,
            max_diagnostic_bytes: 64 * 1024,
            max_source_map_entries: 65_536,
            max_type_depth: 64,
            max_type_width: 4_096,
            max_operation_depth: 128,
            max_operations: 65_536,
        }
    }
}
