//! Compiler-owned Engine policy and immutable, build-produced native components.
#![cfg(not(target_family = "wasm"))]
mod policy;
pub use policy::engine;
use wasmtime::{Engine, component::Component};

mod generated {
    include!(concat!(env!("OUT_DIR"), "/bundle.rs"));
}

/// Select only images produced and embedded by this crate's trusted build.
///
/// The caller still authenticates current Wasm and validates each request.
/// Unknown source or incompatible policy retains ordinary Wasm compilation.
/// An error loading a selected image must propagate to the caller.
#[allow(unsafe_code)]
pub fn load_embedded(
    engine: &Engine,
    source_hash: &str,
    stack: usize,
) -> wasmtime::Result<Option<Component>> {
    if stack != policy::DEFAULT_STACK || policy::key(engine) != generated::ENGINE_KEY {
        return Ok(None);
    }
    let Some(bytes) = generated::image(source_hash) else {
        return Ok(None);
    };
    // SAFETY: generated::image can return only immutable include_bytes constants
    // created by this crate's build script via Engine::precompile_component.
    // No runtime path or caller-supplied native buffer can reach this operation.
    // Source identity and Engine policy select the image; Wasmtime additionally
    // checks serialized compatibility. A selected-image error is never hidden.
    unsafe { Component::deserialize(engine, bytes) }.map(Some)
}

#[doc(hidden)]
pub fn compiled_input_tokens() -> Vec<(&'static str, &'static str)> {
    vec![(
        "sifr_compiler_component_engine",
        env!("SIFR_LOCAL_INPUT_TOKEN"),
    )]
}

#[cfg(test)]
mod tests;
