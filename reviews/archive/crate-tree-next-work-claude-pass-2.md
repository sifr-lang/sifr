Now I have enough to give a focused review.

---

## Proposal Review

### What holds up

- Point 1 (no utils crate) and Point 7 (test runner starts small) are sound.
- The sifr_stdlib extraction direction is correct — `crates/sifr_hir/src/stdlib/` genuinely doesn't belong in HIR and needs a home.
- Point 4's intent (preamble → real Rust APIs) is correct, and `preamble/` in codegen (`io_bytes_methods.rs`, `io_logging_random.rs`, `task_runtime.rs`, `types_and_errors.rs`) is the obvious target.

---

### Contradictions and gaps

**1. CFG/flow_graph in sifr_ir is wrong**

`cfg.rs` and `flow_graph/` (including `effects.rs`) are built *during* lowering — they're mutable analysis state, not stable output data. Putting them in a "maybe only immutable scope snapshots" crate contradicts their actual lifecycle. Either CFG structs live in sifr_ir as plain data types (no construction logic) while the *build logic* lives in sifr_lowering, or they stay in sifr_lowering entirely. The proposal conflates data layout with construction ownership. Decide explicitly.

**2. sifr_stdlib owns Cargo manifests, but codegen/intrinsics owns the same features**

The proposal gives sifr_stdlib "stdlib module → feature → Cargo dependency manifest". But `crates/sifr_codegen/src/intrinsics/registry/` already owns per-module codegen for those exact features (base64, datetime, json, subprocess, etc.), and `stdlib_filter/` in codegen filters which modules are active. You've split the contract for a single stdlib module across two crates with no coordination point. When someone adds a new intrinsic, they'll touch sifr_stdlib for the type signature, sifr_codegen for the codegen, and there's no compile-time enforcement they agree. This needs a single authority — either sifr_stdlib owns the full contract (types + dep manifest + filter metadata) and codegen queries it, or the split needs a concrete interface definition.

**3. preamble → runtime migration ignores driver plumbing**

When codegen emits calls into a real `sifr_runtime` crate instead of inlining preamble strings, generated `Cargo.toml` files must declare `sifr_runtime` as a dependency. That injection lives in `sifr_driver`, not in codegen or runtime. The proposal doesn't mention this and it's not trivial — `sifr_driver` currently invokes `rustc` directly; injecting a workspace-local crate path dependency into generated projects needs a deliberate mechanism. Without addressing this, "make runtime a real library" stalls at the last step.

**4. Async runtime has no home before Phase 40**

`preamble/task_runtime.rs` is async runtime glue that's already non-trivial and the roadmap says async amendment (network, Popen, signals, process pools/IPC) is coming *soon* — before Phase 40/41. The proposal designates `sifr_runtime_validation` and `sifr_runtime_web` for those phases but says nothing about where async runtime support lands in the interim. If it stays in preamble strings, Point 4 is only partially executed. If it goes into sifr_runtime proper, that crate needs `tokio` in its `Cargo.toml`, which makes it a heavy dep for every generated binary including trivial ones. This is a scope decision the proposal defers but shouldn't.

**5. Migration order is completely absent**

The five structural changes (sifr_hir split, sifr_stdlib extraction, sifr_runtime promotion, domain runtime crates, FFI non-split) are interdependent but no sequencing is given. sifr_stdlib extraction requires knowing what sifr_ir exports (since intrinsic type signatures depend on `sifr_type_system::Type`). preamble → runtime migration requires sifr_driver plumbing before it can be declared done. The sifr_hir split is the largest mechanical refactor (100+ files in `lower/`). Without an order, the proposal will generate a debate about what to start first on every PR.

---

### Suggested changes before finalizing

1. **CFG**: Explicitly state whether `cfg.rs`/`flow_graph/` data types live in sifr_ir as plain structs with sifr_lowering providing construction, or stay in sifr_lowering. Drop the "maybe" hedge.

2. **stdlib contract authority**: Assign a single crate as the authority for each stdlib module's full contract (type sig + dep requirement + filter). If sifr_stdlib, then `intrinsics/registry/` in codegen becomes a codegen-only implementation layer that consults sifr_stdlib. State this dependency direction explicitly.

3. **Driver plumbing for runtime**: Add a step: "sifr_driver injects `sifr_runtime` as a Cargo path dep into generated projects." Make this a prerequisite to claiming Point 4 done.

4. **Async runtime**: Name which crate owns async runtime support in the near-term window (before Phase 40). If sifr_runtime, state the tokio-dep tradeoff and how it's managed (feature flags, separate `sifr_runtime_async` earlier than Phase 40, or accept the dep weight).

5. **Migration order**: Add an explicit sequence, e.g.:
   - Step A: extract sifr_stdlib (low blast radius, isolated module)
   - Step B: define sifr_ir data types, migrate sifr_lowering (high blast radius, sequence separately)
   - Step C: driver plumbing + promote sifr_runtime
   - Step D: async runtime consolidation
   
   Without this, the proposal is a desired end-state, not a plan.
