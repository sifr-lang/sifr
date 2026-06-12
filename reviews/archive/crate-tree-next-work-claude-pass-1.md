Now I have enough to give a thorough critique. Let me synthesize the actual code structure with the roadmap.

---

## Architecture Review: Sifr Crate Boundaries

### What the actual code reveals vs. what the proposal assumes

The proposal treats the sifr_hir split as a two-way divide: data model (`sifr_ir`) vs. transformation (`sifr_lowering`). But the actual directory tree shows **three** distinct concerns in `sifr_hir/src/`:

- `hir_nodes.rs`, `cfg.rs`, `flow_graph/`, `scope.rs`, `lowering_outcome.rs` — IR data model
- `lower/` — 30+ files of name resolution, type checking, ownership analysis, async lowering
- `stdlib/` — 7 files (`io_json.rs`, `sys_fs.rs`, `math.rs`, `crypto_regex_uuid.rs`, `collections_bytes_time.rs`, etc.) — intrinsic type signatures

The proposal mentions extracting `sifr_stdlib` for "host-side registry/bootstrap/cache" but doesn't name `sifr_hir/src/stdlib/` as the concrete content that needs a new home. This is imprecise in a way that will cause the refactor to stall: where do the stdlib intrinsic signatures go? They don't belong in `sifr_ir` (that's a pure data model crate) and they're not lowering mechanics (sifr_lowering). They are exactly `sifr_stdlib`'s job. The proposal is right about needing `sifr_stdlib` but for the wrong reasons — it's not a vague "registry/bootstrap" concern, it's the concrete home for what currently lives in `sifr_hir/src/stdlib/`.

**sifr_stdlib should own:** intrinsic type signatures, the module→feature mapping (which stdlib modules use which Rust capabilities), and critically the feature→Cargo-dep manifest (what Cargo dependencies a module's usage requires). `sifr_lowering` calls into it to resolve signatures during type inference. `sifr_driver` calls into it to assemble the generated `Cargo.toml`. This gives `sifr_stdlib` a crisp, non-vague job with two clear callers.

---

### The "move preamble snippets into runtime" proposal is a category error

`sifr_codegen/src/preamble/task_runtime.rs` and `io_bytes_methods.rs` are **Rust source code stored as string literals** that get copy-pasted into generated `.rs` files. `sifr_runtime` is a **compiled Rust library crate** that gets linked into generated projects as a Cargo dependency. These are not the same thing and moving the strings into `sifr_runtime` fixes nothing — you'd just have a library crate that also contains string constants nobody uses, while the codegen still inlines the same implementation.

The correct fix is the opposite direction: make `sifr_runtime` actually implement the task runtime, IO helpers, and channel primitives as real Rust methods, and have `sifr_codegen` emit *calls* to those methods rather than inlining the implementation. The generated code becomes `sifr_runtime::task::spawn(...)` instead of a copy-pasted 80-line task scheduler. This is work, not a string relocation. The proposal conflates two different operations. Name the right one or the migration will just shuffle the problem.

Connected to this: `lib_runtime_needs.rs` in `sifr_codegen` detects which runtime features a module uses (async, channels, bigint, etc.). This detection logic currently feeds into something that hardcodes Cargo dependency specs into the generated `Cargo.toml`. As Phase 40 (axum), Phase 41 (web framework), ORM, ML, and FFI land, that hardcoded list explodes. The proposal doesn't name this problem at all. The fix is a `FeatureDependencyManifest` in `sifr_stdlib` — a static, auditable table mapping language features to versioned Cargo specs. `sifr_codegen` emits feature flags; `sifr_driver` looks them up in the manifest and assembles the final `Cargo.toml`. This also makes it trivial to audit: "when I use `asyncio.Queue`, what Cargo deps does the generated project get?"

---

### Dependency graph issues the proposal doesn't address

`sifr_driver` currently has a direct `sifr_hir` dependency that bypasses `sifr_frontend`. After the split, this should resolve to `sifr_ir` (for reading HIR results) but should not depend on `sifr_lowering` directly — the facade principle means the driver gets IR through `sifr_frontend`. Whether that's achievable depends on what `sifr_driver` actually does with `sifr_hir` today, but the architectural intent should be stated.

`sifr_lint` depends on both `sifr_frontend` and `sifr_hir` directly. After the split, lint should depend on `sifr_ir` only — lint queries structural properties of the IR, it has no business knowing how lowering works. This is a free correctness improvement: lint can no longer accidentally call lowering internals.

---

### How the roadmap changes the calculus

**Async amendment** doesn't change crate structure but creates urgency. The incomplete work (subprocess Popen, signals, process pools, IPC) will add more files to `sifr_hir/src/lower/` and more strings to `sifr_codegen/src/preamble/`. If the hir split and runtime-as-library fix aren't done before this work starts, you'll be layering correct new behavior on top of structural debt that's harder to pay off later. The split should happen before the async amendment, not alongside or after it.

**FFI** is the roadmap item that most challenges the existing structure, but it doesn't require a new crate yet. Python FFI (pyo3-style) and Rust FFI are both new lowering passes in `sifr_lowering` (detecting `@python_module`, `@extern` annotations, new HIR node variants) and new codegen paths in `sifr_codegen` (emitting pyo3 glue vs. normal Rust). The package side goes into `sifr_package` — `SifrPackageGraph` needs to absorb Python package graph nodes when FFI lands, which Phase 37 already anticipated. A `sifr_ffi_codegen` crate is premature until the scope is known. What's not premature: the feature manifest in `sifr_stdlib` needs an entry for `ffi_python` → `pyo3 = { ... }` before FFI ships.

**Test runner** (bun/pytest parity) starts as a module in `sifr_driver`. Test discovery via HIR (detecting `@test`, `@fixture` decorators) means it needs `sifr_ir` access, which `sifr_driver` already has. Extract to `sifr_test` only when it exceeds the 900-line file guardrail or needs dependencies that `sifr_driver` shouldn't carry. Don't create the crate preemptively.

**Phase 40 typed data models** is the first time a domain runtime crate becomes justified. Serde-based validation, field constraints, and derived validators will have their own non-trivial Cargo dependencies (serde, validator, potentially others). `sifr_runtime_validation` as a target-side crate that generated projects depend on is correct: it's a natural unit, its deps are specific, and it doesn't contaminate `sifr_runtime`'s minimal footprint. Create it when Phase 40 starts, not before.

**Phase 41 web framework** is when `sifr_runtime_web` (axum/tower/hyper helpers) becomes justified for the same reason. It also has heavy deps that generated non-web projects should never link. The feature manifest approach is what prevents those deps from leaking.

**WASM** — Rust→WASM via `wasm-bindgen` doesn't require a new codegen backend. The generated Rust is the same; the difference is the target triple passed to rustc and the presence of `wasm-bindgen` in generated deps. `sifr_driver` handles the target triple. Add `wasm` to the feature manifest. No crate structural change.

**ORM/typed SQL**, **ML inference**, **SIMD** — these follow the same pattern as validation: domain runtime crates (`sifr_runtime_db`, `sifr_runtime_ml`) created when the phase begins, named in the manifest, never created speculatively.

---

### Recommended crate tree

**Do now (before async amendment):**

| Crate | Role | Change from today |
|---|---|---|
| `sifr_source` | SourceText/LineMap | No change |
| `sifr_diagnostics` | Diagnostic codes/model | No change |
| `sifr_type_system` | Type definitions, inference, subtyping | No change |
| `sifr_syntax` | Ruff fork wrappers | No change |
| `sifr_ir` | HIR nodes, CFG, flow_graph, scope, lowering_outcome — pure data model | Split from `sifr_hir` |
| `sifr_lowering` | `lower/` — name resolution, type/ownership/async passes | Split from `sifr_hir` |
| `sifr_stdlib` | Intrinsic type signatures (`sifr_hir/src/stdlib/` content), module→feature map, feature→Cargo-dep manifest | Extracted from `sifr_hir/src/stdlib/` + pulled from `lib_runtime_needs.rs` |
| `sifr_frontend` | Canonical facade, now wraps `sifr_ir` + `sifr_lowering` | Update deps |
| `sifr_codegen` | HIR→Rust, emits feature flags, calls `sifr_runtime` methods (no more pasted snippets) | Depends on `sifr_ir` not `sifr_hir`; preambles become runtime calls |
| `sifr_runtime` | Target-side library: `SifrInt`, task system, IO/bytes, channel runtime (proper methods, not strings) | Grows to absorb `preamble/` content |
| `sifr_format`, `sifr_lint`, `sifr_analysis`, `sifr_lsp` | Unchanged in role | `sifr_lint` drops direct `sifr_hir` dep, takes `sifr_ir` only |
| `sifr_package` | Cargo-backed package graph | No change now; extend for Python packages when FFI lands |
| `sifr_driver` | Orchestration, assembles `Cargo.toml` from `sifr_stdlib`'s feature manifest | Remove direct `sifr_hir` dep |
| `sifr` CLI | No change | No change |

**Before Phase 40 starts:**
- `sifr_runtime_validation` — target-side: validator trait, field constraints, serde integration

**Before Phase 41 starts:**
- `sifr_runtime_web` — target-side: axum/tower/hyper helpers, generated request/response types

**Later, by actual dependency pressure:**
- `sifr_runtime_db` — sqlx ORM helpers (Phase 42+)
- `sifr_runtime_ml` — candle/tch (when ML phase is scoped)
- `sifr_doc` — doc extraction from `sifr_ir` + HTML/llms.txt generation (Phase 38 work may want this)

**Never:**
- A vague `sifr_core`, `sifr_utils`, or `sifr_common` crate
- `sifr_ffi_codegen` until FFI scope is concrete
- `sifr_test` crate until test runner outgrows `sifr_driver`

---

### Migration order (concrete)

1. **Split `sifr_hir` → `sifr_ir` + `sifr_lowering`** first. This is the highest-leverage move: reduces compile times, clarifies what codegen and lint actually need, prevents async amendment from adding more code to the wrong crate. Update `sifr_codegen` to depend on `sifr_ir`; update `sifr_lint` to drop the direct `sifr_hir` dependency. `sifr_frontend` wraps both.

2. **Extract `sifr_stdlib`** from `sifr_hir/src/stdlib/` and `lib_runtime_needs.rs`. This is a prerequisite for async amendment work because new intrinsic signatures (subprocess, signals, process pools) need a real home, not a subdirectory in the IR crate. Also move the Cargo dep hardcoding from sifr_codegen/sifr_driver into the manifest.

3. **Make `sifr_runtime` a real library** (preamble/task_runtime.rs → actual methods, io_bytes_methods.rs → actual methods). This can be done incrementally: convert one preamble module at a time and update sifr_codegen to emit calls instead of inlined code. Do this alongside the async amendment since that's where the snippets are most painful.

4. **Remove `sifr_driver`'s direct `sifr_hir` dependency** after step 1. Route through `sifr_frontend` for IR results.

5. **Create domain runtime crates at phase boundaries**, not before. Phase 40 start → `sifr_runtime_validation`. Phase 41 start → `sifr_runtime_web`.

---

### One thing the proposal gets definitively right

No vague `sifr_core`/`sifr_utils` crate. Every time a codebase adds one of those, it becomes a dumping ground. The `sifr_source` pattern — one crate, one named concern, explicit about what it provides — is the right model. Apply that discipline to every split here: if you can't name what `sifr_stdlib` owns in one sentence ("intrinsic signatures and feature-to-Cargo-dep manifest"), don't create it. The fact that you can name it in one sentence is exactly why it's justified.
