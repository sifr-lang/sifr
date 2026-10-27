"""Verify the single Engine policy and compiler-owned native-image boundary."""
from __future__ import annotations
from pathlib import Path

ENGINE = "crates/sifr_compiler_component_engine"
POLICY_CONTROLS = {
    "config.consume_fuel(true)",
    "config.wasm_component_model(true)",
    "config.cranelift_nan_canonicalization(true)",
    "config.wasm_exceptions(true)",
    "config.wasm_relaxed_simd(false)",
    "config.wasm_memory64(false)",
    "config.wasm_multi_memory(true)",
    "config.max_wasm_stack(stack)",
    "config.cranelift_opt_level(OptLevel::Speed)",
    "Engine::new(&config)",
}
LOADER_CONTROLS = {
    'include!(concat!(env!("OUT_DIR"), "/bundle.rs"))',
    "stack != policy::DEFAULT_STACK || policy::key(engine) != generated::ENGINE_KEY",
    "let Some(bytes) = generated::image(source_hash) else { return Ok(None); };",
    "unsafe { Component::deserialize(engine, bytes) }.map(Some)",
}

def validate_bundle_boundary(
    root: Path, *, policy_source: str | None = None, loader_source: str | None = None,
) -> None:
    policy = (root / ENGINE / "src/policy.rs").read_text() if policy_source is None else policy_source
    loader = (root / ENGINE / "src/lib.rs").read_text() if loader_source is None else loader_source
    for source, controls in ((policy, POLICY_CONTROLS), (loader, LOADER_CONTROLS)):
        compact = "".join(source.split())
        if any("".join(control.split()) not in compact for control in controls):
            raise ValueError("component Engine policy or immutable image selection is incomplete")
    if any(token in policy + loader for token in ("wasmtime_wasi", "deserialize_file", "std::fs", "File::open")):
        raise ValueError("component runtime must not obtain native images from ambient paths")
    if loader.count("Component::deserialize(") != 1:
        raise ValueError("component runtime must have exactly one immutable native-image boundary")
    producer = (root / ENGINE / "build.rs").read_text()
    for control in (
        '#[path = "src/policy.rs"]', "policy::engine(policy::DEFAULT_STACK)?",
        "engine.precompile_component(&wasm)?", "if actual != expected",
        'fs::write(out.join("bundle.rs"), code)?',
        'env::var("HOST")? == env::var("TARGET")?',
        "if total_bytes > MAX_BUNDLE_BYTES",
    ):
        if "".join(control.split()) not in "".join(producer.split()):
            raise ValueError("trusted component producer boundary is incomplete")
    if "deserialize" in producer:
        raise ValueError("trusted component producer must compile source Wasm")

def self_test(root: Path) -> int:
    policy = (root / ENGINE / "src/policy.rs").read_text()
    loader = (root / ENGINE / "src/lib.rs").read_text()
    mutations = [
        ("fuel", "policy", "config.consume_fuel(true)", "config.consume_fuel(false)"),
        ("simd", "policy", "config.wasm_relaxed_simd(false)", "config.wasm_relaxed_simd(true)"),
        ("stack", "loader", "stack != policy::DEFAULT_STACK", "false"),
        ("engine", "loader", "policy::key(engine) != generated::ENGINE_KEY", "false"),
        ("source", "loader", "generated::image(source_hash)", 'generated::image("unverified")'),
    ]
    validate_bundle_boundary(root)
    for label, owner, old, new in mutations:
        source = policy if owner == "policy" else loader
        if old not in source:
            raise ValueError(f"bundle mutation target absent: {label}")
        try:
            validate_bundle_boundary(root, **{f"{owner}_source": source.replace(old, new)})
        except ValueError:
            continue
        raise ValueError(f"bundle boundary accepted mutation: {label}")
    try:
        validate_bundle_boundary(root, loader_source=loader + '\nfn injected() { std::fs::read("native"); }\n')
    except ValueError:
        return len(mutations) + 1
    raise ValueError("bundle boundary accepted ambient native image path")
