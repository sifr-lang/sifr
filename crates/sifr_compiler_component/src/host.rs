use crate::registration::verify_component_hash;
use crate::{
    AnalysisCache, CacheKey, ComponentError, ComponentErrorKind, ComponentRegistration,
    DiagnosticRegistry, EmbeddedAnalysisRequest, EmbeddedAnalysisResponse, validate_request,
    validate_response,
};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store, StoreLimits, StoreLimitsBuilder, Trap};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentRun {
    pub response: EmbeddedAnalysisResponse,
    pub cache_key: CacheKey,
    pub cache_hit: bool,
}

struct HostState {
    limits: StoreLimits,
}

pub struct ComponentHost {
    engine: Engine,
    limits: ComponentHostLimits,
    cache: Option<AnalysisCache>,
}

impl ComponentHost {
    pub fn new(
        limits: ComponentHostLimits,
        cache: Option<AnalysisCache>,
    ) -> Result<Self, ComponentError> {
        let mut config = Config::new();
        config.consume_fuel(true);
        config.wasm_component_model(true);
        config.wasm_exceptions(true);
        config.wasm_memory64(false);
        config.wasm_multi_memory(true);
        config.wasm_relaxed_simd(false);
        config.max_wasm_stack(limits.max_wasm_stack_bytes);
        config.cranelift_nan_canonicalization(true);
        let engine = Engine::new(&config).map_err(engine_error)?;
        Ok(Self {
            engine,
            limits,
            cache,
        })
    }

    pub fn analyze(
        &mut self,
        registration: &ComponentRegistration,
        component_bytes: &[u8],
        request: &EmbeddedAnalysisRequest,
    ) -> Result<ComponentRun, ComponentError> {
        if component_bytes.len() > self.limits.max_component_bytes {
            return Err(ComponentError::new(
                ComponentErrorKind::ResourceLimit,
                "component artifact exceeds the component byte limit",
            ));
        }
        registration.protocol.validate()?;
        DiagnosticRegistry::compiler()
            .validate_with(std::slice::from_ref(&registration.diagnostics))?;
        if !registration.protocol.contains(request.protocol_major) {
            return Err(ComponentError::new(
                ComponentErrorKind::ProtocolVersion,
                "component protocol range does not contain the request version",
            ));
        }
        if registration.identity != request.component {
            return Err(ComponentError::new(
                ComponentErrorKind::Registration,
                "request identity does not match the resolved component",
            ));
        }
        if registration.diagnostics != request.provider_diagnostics {
            return Err(ComponentError::new(
                ComponentErrorKind::DiagnosticRegistry,
                "request diagnostic registry does not match the resolved component",
            ));
        }
        verify_component_hash(&registration.identity.sha256, component_bytes)?;
        validate_request(request, &self.limits)?;
        let key = CacheKey::for_request(request)?;
        if let Some(cache) = &mut self.cache {
            let max_cache_bytes = u64::try_from(self.limits.max_output_bytes)
                .unwrap_or(u64::MAX)
                .saturating_add(4_096);
            if let Some(response) = cache.get(&key, max_cache_bytes)? {
                let response_bytes =
                    serde_json::to_vec(&response).map_err(protocol_serialization_error)?;
                if response_bytes.len() > self.limits.max_output_bytes {
                    return Err(ComponentError::new(
                        ComponentErrorKind::ResourceLimit,
                        "cached component response exceeds the output byte limit",
                    ));
                }
                validate_response(request, &response, &self.limits, &registration.diagnostics)?;
                return Ok(ComponentRun {
                    response,
                    cache_key: key,
                    cache_hit: true,
                });
            }
        }
        let response = self.execute(component_bytes, request, &registration.diagnostics)?;
        if let Some(cache) = &mut self.cache {
            cache.put(&key, &response)?;
        }
        Ok(ComponentRun {
            response,
            cache_key: key,
            cache_hit: false,
        })
    }

    pub fn pin_cache_entry(&mut self, key: CacheKey) {
        if let Some(cache) = &mut self.cache {
            cache.pin(key);
        }
    }

    pub fn release_cache_entry(&mut self, key: &CacheKey) {
        if let Some(cache) = &mut self.cache {
            cache.unpin(key);
        }
    }

    fn execute(
        &self,
        component_bytes: &[u8],
        request: &EmbeddedAnalysisRequest,
        diagnostics: &DiagnosticRegistry,
    ) -> Result<EmbeddedAnalysisResponse, ComponentError> {
        let input = serde_json::to_vec(request).map_err(protocol_serialization_error)?;
        if input.len() > self.limits.max_input_bytes {
            return Err(ComponentError::new(
                ComponentErrorKind::ResourceLimit,
                "component request exceeds the input byte limit",
            ));
        }
        let component = Component::new(&self.engine, component_bytes).map_err(component_error)?;
        let imports = component
            .component_type()
            .imports(&self.engine)
            .map(|(name, _)| name.to_string())
            .collect::<Vec<_>>();
        if !imports.is_empty() {
            return Err(ComponentError::new(
                ComponentErrorKind::Capability,
                format!(
                    "component imports are forbidden; requested {}",
                    imports.join(", ")
                ),
            ));
        }
        let linker = Linker::<HostState>::new(&self.engine);
        let store_limits = StoreLimitsBuilder::new()
            .memory_size(self.limits.max_memory_bytes)
            .instances(self.limits.max_instances)
            .memories(self.limits.max_memories)
            .tables(self.limits.max_tables)
            .build();
        let mut store = Store::new(
            &self.engine,
            HostState {
                limits: store_limits,
            },
        );
        store.limiter(|state| &mut state.limits);
        store.set_fuel(self.limits.fuel).map_err(resource_error)?;
        let instance = linker
            .instantiate(&mut store, &component)
            .map_err(capability_or_execution_error)?;
        let function = instance
            .get_typed_func::<(Vec<u8>,), (Vec<u8>,)>(&mut store, "analyze")
            .map_err(protocol_shape_error)?;
        let (output,) = function
            .call(&mut store, (input,))
            .map_err(capability_or_execution_error)?;
        if output.len() > self.limits.max_output_bytes {
            return Err(ComponentError::new(
                ComponentErrorKind::ResourceLimit,
                "component response exceeds the output byte limit",
            ));
        }
        let response = serde_json::from_slice(&output).map_err(protocol_serialization_error)?;
        validate_response(request, &response, &self.limits, diagnostics)?;
        Ok(response)
    }
}

#[allow(clippy::needless_pass_by_value)]
fn engine_error(error: wasmtime::Error) -> ComponentError {
    ComponentError::new(ComponentErrorKind::Execution, format!("{error:#}"))
}

#[allow(clippy::needless_pass_by_value)]
fn component_error(error: wasmtime::Error) -> ComponentError {
    let message = format!("{error:#}");
    let kind = if message.contains("shared memor")
        || message.contains("threads support")
        || message.contains("threads must be enabled")
        || message.contains("memory64")
    {
        ComponentErrorKind::Capability
    } else {
        ComponentErrorKind::ProtocolEnvelope
    };
    ComponentError::new(kind, format!("component binary is invalid: {message}"))
}

#[allow(clippy::needless_pass_by_value)]
fn protocol_shape_error(error: wasmtime::Error) -> ComponentError {
    ComponentError::new(
        ComponentErrorKind::ProtocolEnvelope,
        format!("component does not implement the compiler-owned WIT contract: {error:#}"),
    )
}

#[allow(clippy::needless_pass_by_value)]
fn capability_or_execution_error(error: wasmtime::Error) -> ComponentError {
    if matches!(error.downcast_ref::<Trap>(), Some(Trap::OutOfFuel)) {
        return ComponentError::new(ComponentErrorKind::ResourceLimit, format!("{error:#}"));
    }
    let message = format!("{error:#}");
    let kind = if message.contains("fuel")
        || message.contains("resource limit")
        || message.contains("memory limits")
        || message.contains("table limits")
        || message.contains("instance limits")
    {
        ComponentErrorKind::ResourceLimit
    } else {
        ComponentErrorKind::Execution
    };
    ComponentError::new(kind, message)
}

#[allow(clippy::needless_pass_by_value)]
fn resource_error(error: wasmtime::Error) -> ComponentError {
    ComponentError::new(ComponentErrorKind::ResourceLimit, format!("{error:#}"))
}

#[allow(clippy::needless_pass_by_value)]
fn protocol_serialization_error(error: serde_json::Error) -> ComponentError {
    ComponentError::new(ComponentErrorKind::ProtocolEnvelope, error.to_string())
}
