//! Bounded compiled code owned by one compiler/editor lifetime.
//!
//! Entries share exactly one Engine. Neither guest Stores nor response values
//! live here. Limits bound retained entries and mapped compiled images, not
//! compilation working memory or references held by active executions.
use crate::{ComponentError, ComponentErrorKind, ComponentHostLimits};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use wasmtime::component::Component;
use wasmtime::{Config, Engine};

const MAX_ENTRIES: usize = 8;
const MAX_IMAGE_BYTES: usize = 128 * 1024 * 1024;

#[derive(Clone)]
pub struct ComponentCompilation(Arc<CompilationInner>);

struct CompilationInner {
    engine: Engine,
    max_wasm_stack_bytes: usize,
    entries: Mutex<CompiledEntries>,
}

struct Entry {
    hash: String,
    component: Component,
    image_bytes: usize,
}

struct CompiledEntries {
    entries: VecDeque<Entry>,
    image_bytes: usize,
    max_entries: usize,
    max_image_bytes: usize,
    #[cfg(test)]
    compilations: usize,
    #[cfg(test)]
    hits: usize,
}

impl std::fmt::Debug for ComponentCompilation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentCompilation")
            .field("max_wasm_stack_bytes", &self.0.max_wasm_stack_bytes)
            .finish_non_exhaustive()
    }
}

impl ComponentCompilation {
    pub fn new(limits: &ComponentHostLimits) -> Result<Self, ComponentError> {
        let mut config = Config::new();
        config.consume_fuel(true);
        config.wasm_component_model(true);
        config.wasm_exceptions(true);
        config.wasm_memory64(false);
        config.wasm_multi_memory(true);
        config.wasm_relaxed_simd(false);
        config.max_wasm_stack(limits.max_wasm_stack_bytes);
        config.cranelift_nan_canonicalization(true);
        let engine_span = DiagnosticSpan::new("engine", 0, "");
        let engine = Engine::new(&config).map_err(crate::host::engine_error)?;
        drop(engine_span);
        Ok(Self(Arc::new(CompilationInner {
            engine,
            max_wasm_stack_bytes: limits.max_wasm_stack_bytes,
            entries: Mutex::new(CompiledEntries {
                entries: VecDeque::new(),
                image_bytes: 0,
                max_entries: MAX_ENTRIES,
                max_image_bytes: MAX_IMAGE_BYTES,
                #[cfg(test)]
                compilations: 0,
                #[cfg(test)]
                hits: 0,
            }),
        })))
    }

    pub(crate) fn validate_policy(
        &self,
        limits: &ComponentHostLimits,
    ) -> Result<(), ComponentError> {
        if limits.max_wasm_stack_bytes != self.0.max_wasm_stack_bytes {
            return Err(ComponentError::new(
                ComponentErrorKind::Execution,
                "compiled-component owner has an incompatible Engine stack policy",
            ));
        }
        Ok(())
    }

    pub(crate) fn engine(&self) -> &Engine {
        &self.0.engine
    }

    // Only ComponentHost calls this, after authenticating bytes and validating
    // the current request/registration. Hold the lock through compilation to
    // avoid duplicate publication; release it before any guest execution.
    pub(crate) fn component(&self, hash: &str, bytes: &[u8]) -> Result<Component, ComponentError> {
        let cache_span =
            DiagnosticSpan::new("component_lookup", Arc::as_ptr(&self.0) as usize, hash);
        let mut cache = self.0.entries.lock().map_err(|_| {
            ComponentError::new(
                ComponentErrorKind::Execution,
                "compiled-component owner is poisoned",
            )
        })?;
        if let Some(index) = cache.entries.iter().position(|entry| entry.hash == hash) {
            if let Some(entry) = cache.entries.remove(index) {
                let component = entry.component.clone();
                cache.entries.push_back(entry);
                #[cfg(test)]
                {
                    cache.hits += 1;
                }
                drop(DiagnosticSpan::new(
                    "component_hit",
                    Arc::as_ptr(&self.0) as usize,
                    hash,
                ));
                return Ok(component);
            }
        }
        #[cfg(test)]
        {
            cache.compilations += 1;
        }
        let compile_span =
            DiagnosticSpan::new("component_compile", Arc::as_ptr(&self.0) as usize, hash);
        let component =
            Component::new(self.engine(), bytes).map_err(crate::host::component_error)?;
        drop(compile_span);
        let imports = component
            .component_type()
            .imports(self.engine())
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
        let image = component.image_range();
        let image_bytes = (image.end as usize).saturating_sub(image.start as usize);
        // Oversized code executes once under ordinary request limits but is not
        // retained. A failed compile/import check never changes retained entries.
        if cache.max_entries != 0 && image_bytes <= cache.max_image_bytes {
            while cache.entries.len() >= cache.max_entries
                || cache.image_bytes > cache.max_image_bytes - image_bytes
            {
                if let Some(entry) = cache.entries.pop_front() {
                    cache.image_bytes -= entry.image_bytes;
                } else {
                    break;
                }
            }
            cache.image_bytes += image_bytes;
            cache.entries.push_back(Entry {
                hash: hash.to_string(),
                component: component.clone(),
                image_bytes,
            });
        }
        drop(cache_span);
        Ok(component)
    }

    #[cfg(test)]
    pub(crate) fn stats(&self) -> (usize, usize, usize, usize) {
        let c = self.0.entries.lock().expect("test owner lock");
        (c.compilations, c.hits, c.entries.len(), c.image_bytes)
    }

    #[cfg(test)]
    pub(crate) fn set_test_capacity(&self, entries: usize, image_bytes: usize) {
        let mut c = self.0.entries.lock().expect("test owner lock");
        assert!(c.entries.is_empty());
        c.max_entries = entries;
        c.max_image_bytes = image_bytes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ComponentHost;

    #[test]
    fn last_owner_releases_engine_and_retained_components() {
        let limits = ComponentHostLimits::default();
        let owner = ComponentCompilation::new(&limits).unwrap();
        let weak = Arc::downgrade(&owner.0);
        let bytes = wat::parse_str("(component)").unwrap();
        drop(
            owner
                .component("authenticated-test-fixture", &bytes)
                .unwrap(),
        );
        assert_eq!(owner.stats().2, 1);
        let host = ComponentHost::with_compilation(owner.clone(), limits, None).unwrap();
        drop(owner);
        assert!(weak.upgrade().is_some());
        drop(host);
        assert!(weak.upgrade().is_none());
    }
}

// Temporary V1cl attribution only. Removed before any production acceptance.
pub(crate) struct DiagnosticSpan {
    output: Option<std::path::PathBuf>,
    start: std::time::Instant,
    epoch_ns: u128,
    event: &'static str,
    owner: usize,
    hash: String,
}
impl DiagnosticSpan {
    pub(crate) fn new(event: &'static str, owner: usize, hash: &str) -> Self {
        Self {
            output: std::env::var_os("SIFR_V1CL_TRACE_DIR").map(std::path::PathBuf::from),
            start: std::time::Instant::now(),
            epoch_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            event,
            owner,
            hash: hash.to_string(),
        }
    }
}
impl Drop for DiagnosticSpan {
    fn drop(&mut self) {
        let Some(root) = &self.output else { return };
        let elapsed_ns = self.start.elapsed().as_nanos();
        let row = serde_json::json!({
            "pid": std::process::id(), "event": self.event, "owner": self.owner,
            "component": self.hash, "start_epoch_ns": self.epoch_ns, "elapsed_ns": elapsed_ns
        });
        use std::io::Write as _;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(root.join(format!("{}.jsonl", std::process::id())))
        {
            let _ = writeln!(file, "{row}");
        }
    }
}
