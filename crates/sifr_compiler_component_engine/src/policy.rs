//! One Engine policy for trusted production and ordinary component execution.
use std::hash::{Hash, Hasher};
use wasmtime::{Config, Engine, OptLevel};

pub const DEFAULT_STACK: usize = 2 * 1024 * 1024;

pub fn engine(stack: usize) -> wasmtime::Result<Engine> {
    let mut config = Config::new();
    config.consume_fuel(true);
    config.wasm_component_model(true);
    config.wasm_exceptions(true);
    config.wasm_memory64(false);
    config.wasm_multi_memory(true);
    config.wasm_relaxed_simd(false);
    config.max_wasm_stack(stack);
    config.cranelift_nan_canonicalization(true);
    config.cranelift_opt_level(OptLevel::Speed);
    Engine::new(&config)
}

pub fn key(engine: &Engine) -> u64 {
    let mut state = std::collections::hash_map::DefaultHasher::new();
    engine.precompile_compatibility_hash().hash(&mut state);
    state.finish()
}
