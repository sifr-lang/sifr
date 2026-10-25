use crate::registration::verify_component_hash;
use crate::{
    AnalysisCache, CacheKey, ComponentCompilation, ComponentError, ComponentErrorKind,
    ComponentHostLimits, ComponentRegistration, DiagnosticRegistry, EmbeddedAnalysisRequest,
    EmbeddedAnalysisResponse, validate_request, validate_response,
};
use wasmtime::component::Linker;
use wasmtime::{Store, StoreLimits, StoreLimitsBuilder, Trap};

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
    compilation: ComponentCompilation,
    limits: ComponentHostLimits,
    cache: Option<AnalysisCache>,
}

impl ComponentHost {
    pub fn new(
        limits: ComponentHostLimits,
        cache: Option<AnalysisCache>,
    ) -> Result<Self, ComponentError> {
        let compilation = ComponentCompilation::new(&limits)?;
        Self::with_compilation(compilation, limits, cache)
    }

    pub fn with_compilation(
        compilation: ComponentCompilation,
        limits: ComponentHostLimits,
        cache: Option<AnalysisCache>,
    ) -> Result<Self, ComponentError> {
        compilation.validate_policy(&limits)?;
        Ok(Self {
            compilation,
            limits,
            cache,
        })
    }

    #[must_use]
    pub fn compilation(&self) -> ComponentCompilation {
        self.compilation.clone()
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
        let input = serde_json::to_vec(request).map_err(protocol_serialization_error)?;
        if input.len() > self.limits.max_input_bytes {
            return Err(ComponentError::new(
                ComponentErrorKind::ResourceLimit,
                "component request exceeds the input byte limit",
            ));
        }
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
        let response = self.execute(component_bytes, request, input, &registration.diagnostics)?;
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
        input: Vec<u8>,
        diagnostics: &DiagnosticRegistry,
    ) -> Result<EmbeddedAnalysisResponse, ComponentError> {
        let component = self
            .compilation
            .component(&request.component.sha256, component_bytes)?;
        let engine = self.compilation.engine();
        let linker = Linker::<HostState>::new(engine);
        let store_limits = StoreLimitsBuilder::new()
            .memory_size(self.limits.max_memory_bytes)
            .instances(self.limits.max_instances)
            .memories(self.limits.max_memories)
            .tables(self.limits.max_tables)
            .build();
        let mut store = Store::new(
            engine,
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
pub(crate) fn engine_error(error: wasmtime::Error) -> ComponentError {
    ComponentError::new(ComponentErrorKind::Execution, format!("{error:#}"))
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn component_error(error: wasmtime::Error) -> ComponentError {
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
