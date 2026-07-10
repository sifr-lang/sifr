# VERDICT: SATISFIED

The revised plan substantively resolves every round-1 blocking finding, its architectural representation is sound, and its per-milestone buildability holds. New observations below are non-blocking.

## Status of every round-1 blocking item

1. **M3 broke hashlib via `bytes_to_hex_strict`** — **RESOLVED.** M3 now folds the migration in *before* raw-name dispatch removal: "publish `sifr_stdlib::bytes::bytes_to_hex_strict`, declare it in `_sifr.bytes`, route the live `stdlib/sifr/hashlib.sifr` caller through that declaration, and delete its fallback signature and compiler dispatch arm." M3 acceptance requires "hashlib.sifr checks, emits, builds, and runs after M3 with no raw-name intrinsic dispatch or fallback signature." Verified: `sifr_stdlib/src/bytes.rs:17` is still private `fn`, so M3 must flip it to `pub`.

2. **Fallback resolution has ≥3 sites** — **RESOLVED.** M5 enumerates all sites: bootstrap.rs "missing-module branch and `re_export_intrinsic_fallbacks`" (covers both `bootstrap.rs:186` and `bootstrap.rs:193-208`), `resolve_retained_fallback` in `private_stdlib_imports.rs:218`, and the independent `mod_impl.rs:293` branch. Verified against actual code: 4 live production call sites, all named. Acceptance grep `rg 'sifr_retained_intrinsics' crates Cargo.toml Cargo.lock` = no match is a hard backstop.

3. **Retained direct dependency packages** — **RESOLVED.** M1 explicitly removes `metrics`/`tracing` from `retained_direct_dependency_packages`; M4 explicitly removes `serde`/`serde_json`. M6 acceptance: "Require every retained direct dependency package to have a corresponding live typed-intrinsic feature requirement; reject orphan package rows."

4. **Lowering entry-point enumeration for M3** — **RESOLVED.** M3 enumerates the current sites: `builtin_calls/bytes_len_range.rs`, `builtin_calls/constructors.rs`, `expressions/methods_lambdas_and_comprehensions.rs`, and `expressions/call_shadowable_builtins.rs`. Verified against actual code: `HirExpr::Call { func: "bytes_from_hex"|"bytes_from_ints"|"bytes_with_size"|"str_encode_utf8_result"|"str_encode_utf8_result_with_encoding"|"decode_utf8"|"decode_utf8_with_encoding"|"builtin_open"|"builtin_open_text" }` all live in those four files. Acceptance criterion "No lowering path constructs a string-named `HirExpr::Call` for any of the 17 typed IDs" is executable.

5. **M2 production-side gaps** — **RESOLVED.** M2 bounded inventory is exact against `harness_model.rs:398-516`: 6 inferred modules, 1 runtime crate, 29 direct crates. Verified count-for-count. M2 acceptance: "Every former inference rule has a checked evidence row naming its typed production metadata source or the production metadata repair made in M2." Executable and scope-bounded.

All round-1 non-blocking items are also addressed except the very minor "add a doc-comment on `sifr_stdlib::bytes::bytes_to_hex_strict` when publishing."

## Architectural spot-check (objective 2)

The plan states: "Export intrinsic identity alongside callable signature metadata." Verified path is coherent:
- Declaration form `@compiler_intrinsic(<id>)` in canonical sysroot source lowers to a `HirFunction` with an `Option<CompilerIntrinsicId>`.
- Compiled sysroot exports carry that ID on the exported callable.
- Call lowering that resolves a name (or alias) to a callable with an intrinsic ID emits typed intrinsic HIR instead of `HirExpr::Call { func: String }`.
- Acceptance "Test/task aliases preserve intrinsic identity" plus "No lowering path constructs a string-named `HirExpr::Call` for any of the 17 typed IDs" are the hard backstops.

Confirmed against `stdlib/sifr/test.sifr:72` (`assert_vector_eq` calls `assert_eq` from the same module) and `stdlib/sifr/task.sifr` (`current_context` calls `task_current_context`): both are intra-module or same-package callable resolutions. The identity travels with the resolved callable, so no second signature source is required after `sifr_retained_intrinsics` is deleted.

## Buildability spot-check (objective 3)

- **After M1**: `_sifr.runtime` has a live `@rust(sifr_stdlib.runtime_observability.emit_diagnostic)` declaration; `stdlib/sifr/runtime.sifr` still compiles; the guard's observed set drops runtime, metrics, tracing and the manifest drops the same rows → guard passes.
- **After M2**: E2E harness uses `SysrootDependencyPlan.cargo_dependency_lines()` and typed `HashSet<StdlibFeature>`; only executes if every former inference rule has been proved-covered or repaired.
- **After M3**: `hashlib.sifr` compiles (per acceptance); test/task callable IDs preserved through import; raw-name dispatch fully removed.
- **After M4**: Counter/legacy defaultdict debt gone; typed `__sifr_defaultdict_*` lowering preserved.
- **After M5**: `sifr_retained_intrinsics` deleted; all four production sites removed; missing-declaration produces a structured diagnostic.
- **After M6**: guards rewritten around final architecture; recertified.

## Non-blocking improvements

- **Retained-dependency `retained_dependency_specs()` prune**: The guard reads packages from `dependency_plan.rs::retained_dependency_specs()`. The plan says "remove `metrics`/`tracing` from `retained_direct_dependency_packages`" (manifest) and "no retained typed intrinsic requires them," which correctly implies removal from the source function too — but naming the exact file (`crates/sifr_stdlib_manifest/src/features/dependency_plan.rs`) would prevent an implementer from updating only the manifest and hitting a "observed vs allowlist" guard failure. Same holds for `serde`/`serde_json` in M4.
- **Higher-order use of `@compiler_intrinsic` callables**: The plan removes emitted bodies, so `f = assert_eq; f(a, b)` has no callable to reference. State explicitly that user code passing a `@compiler_intrinsic` callable as a value is rejected with a structured diagnostic (analog to the "user/package `@compiler_intrinsic` is rejected" test).
- **`sifr_stdlib::bytes::bytes_to_hex_strict` visibility**: currently `fn`; M3 must flip to `pub`. A one-line acceptance sub-item ("`grep 'pub fn bytes_to_hex_strict' crates/sifr_stdlib/src/bytes.rs`") avoids a review-round comment.
- **Manifest schema for lowering files (M4/M6)**: M4 says the defaultdict row must "enumerate its lowering/codegen implementation files, not only the central dispatch modules," but current schema `registry_files`/`preamble_files` are `sifr_codegen`-relative. Either extend the schema in M4 or state explicitly that lowering-file enumeration is deferred to the M6 guard rewrite so the intermediate manifest is not asked to hold data it cannot.
- **Callable-metadata shape**: Making explicit that `Option<CompilerIntrinsicId>` is added to the exported callable type (or to a parallel `intrinsic_ids: HashMap<..., CompilerIntrinsicId>` field) would reduce M3 review latency. Currently implicit in "Export intrinsic identity alongside callable signature metadata."

No round-3 checklist required.
