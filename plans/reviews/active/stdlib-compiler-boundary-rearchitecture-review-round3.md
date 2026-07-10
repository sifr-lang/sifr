I have sufficient independent evidence. Let me finalize the review.

---

# VERDICT: SATISFIED

The plan converges. Every round-2 non-blocking item has been folded into the plan text, the buildability of each individual milestone holds, and the concrete metadata design respects the `sifr_ir` → `sifr_lowering` dependency direction. One non-blocking observation on re-export propagation, no blocking findings.

## Independently verified

- **Round-2 non-blocking items are all incorporated in the plan text:**
  - Explicit `dependency_plan.rs::retained_dependency_specs` pruning: named in M1 (metrics/tracing) and M4 (serde/serde_json) with file path in body.
  - First-class value use of a `@compiler_intrinsic` callable is rejected with a structured diagnostic (plan §3, "Treating a compiler-intrinsic callable as a first-class value—assignment, argument, return, container storage, or closure capture—is rejected"). M3 acceptance repeats it. M6 negative self-tests enumerate it.
  - `bytes_to_hex_strict` publication and bridge-target documentation are in M3 tasks ("public Rust adapter is documented as the `_sifr.bytes.bytes_to_hex_strict` bridge target").
  - Manifest cross-crate schema extension is explicitly deferred to M6 in M4 body ("M6 owns the schema extension that can enumerate cross-crate lowering and codegen files exactly"), and M6 owns "Extend the retained-manifest schema with explicit lowering/codegen ownership fields, then backfill every retained surface."
  - Concrete metadata representation locked in §2: `CompilerIntrinsicId` and `HirFunction.compiler_intrinsic` in `sifr_ir`; `ExternalDefs.compiler_intrinsics` and `LowerCtx.compiler_intrinsics` in the lowering layer; `FunctionType` remains signature-only.

- **Crate dependency direction is respected.** Verified: `sifr_lowering/Cargo.toml` depends on `sifr_ir`; `sifr_ir/Cargo.toml` does not depend on `sifr_lowering`. Placing `CompilerIntrinsicId` in `sifr_ir` upstream and the identity maps (`ExternalDefs`, `LowerCtx`) in `sifr_lowering` downstream is a valid dep-graph direction.

- **Per-milestone buildability holds.**
  - M1: `_sifr.runtime` gains a real `@rust` declaration before its fallback module entry is removed. Deleting the runtime dispatch arm is safe because the `has_compiled_exports` gate in `bootstrap.rs:162` short-circuits the fallback path once a live declaration exists.
  - M2: E2E-only refactor; production planning is authoritative and untouched, and the bounded 6+1+29 inference-rule inventory is enumerated in-plan (M2 tasks).
  - M3: `hashlib.sifr` migration is folded in *before* raw-name dispatch removal; test assertions and `current_context` become `@compiler_intrinsic` source declarations in the public modules they already live in — no re-export propagation required for these 8 identifiers.
  - M4: Counter arms and JSON-string defaultdict are provably dead (no live callers under `stdlib/**`).
  - M5: `sifr_retained_intrinsics` deletion is gated by all three call sites (`bootstrap.rs`, `private_stdlib_imports.rs`, `mod_impl.rs`) being removed, and the acceptance `rg 'sifr_retained_intrinsics' crates Cargo.toml Cargo.lock` = 0 is a hard backstop.
  - M6: guard rewrite + schema extension + negative self-tests; runs only after architecture is final.

- **27 → 17 accounting is exact against `crates/sifr_codegen/src/intrinsics/registry.rs`** (7 asserts + 2 opens + 8 counter + 4 encoding + 4 bytes + runtime + task = 27; removing 1 runtime + 8 counter + 1 `bytes_to_hex_strict` = 17).

- **19 fallback modules verified in `crates/sifr_retained_intrinsics/src/lib.rs:59-88`.** After the plan lands, `fallback_signature_modules = 0`.

- **Historical phase separation is preserved.** The plan explicitly reads: "This is a new phase following the native-boundary migration. The prior implementation remains historical input; this phase owns the final compiler/stdlib boundary architecture." No prior-phase status is retroactively rewritten.

- **Acceptance criteria are executable.** Every acceptance line I sampled resolves to a checkable command or a concrete file/diagnostic invariant: `rg` greps, standalone `sifr build/run` on named fixtures, guard scripts, negative self-tests enumerated in M6.

- **`@compiler_intrinsic` identity ambiguity is closed** across the four vectors called out:
  - Local declarations — M6 explicit negative self-test for "user/local declarations using former intrinsic names."
  - Imports — `ExternalDefs.compiler_intrinsics` keys on canonical (module, name).
  - Aliases — `LowerCtx.compiler_intrinsics` includes import aliases explicitly.
  - First-class value use — rejected with structured diagnostic (§3, M3 acceptance, M6 self-test).

## Blocking findings

None.

## Non-blocking observations

1. **Re-export metadata propagation is implicit, not stated.** The `ReExportMaps` struct at `crates/sifr_driver/src/stdlib/re_exports.rs:5-13` currently copies `functions`, `classes`, `class_type_params`, `defaults`, `varargs`, `workloads`, `constants` across module boundaries at bootstrap. It has no `compiler_intrinsics` field. In the plan's concrete design, the 8 `@compiler_intrinsic` identifiers are declared directly in the public modules (`stdlib/sifr/test.sifr`, `stdlib/sifr/task.sifr`) that user code imports from — so re-export propagation is not exercised for the current target set, and the plan is not wrong. But `re_export_stdlib_imports` is invoked broadly (e.g., `sifr.python` → `sifr.python_core` at `bootstrap.rs:221`), so if a future retained callable is ever placed in one sysroot module and re-exported from another, the metadata would silently drop. An M3 acceptance sub-item along the lines of "`ReExportMaps` propagates `compiler_intrinsics` for re-exported callables, or a lint rejects re-exporting a `@compiler_intrinsic` callable" would foreclose this class of drift without changing scope.

2. **`_sifr.io` fallback branch in `lib.rs:61`.** The plan lists `_sifr.io` in the empty-placeholder deletion (M5). `_sifr.io => intrinsic_io()` still returns a populated `IntrinsicModule` in `sifr_retained_intrinsics/src/io_json.rs` — dead once `_sifr.io` is deleted, but worth a one-line note that deleting the placeholder module also invalidates that specific fallback branch.

3. **The dependency diagram uses "M4 Counter/defaultdict/bytes cleanup"** but the M4 body defers the `bytes_to_hex_strict` migration to M3 and only handles the manifest bytes-row rename. The mermaid label is fine, but the M4 title could drop "bytes" to avoid implying the M3-owned migration lives here.

## What was independently verified

- Built the actual dependency edge between `sifr_ir` and `sifr_lowering` from Cargo manifests.
- Located `ExternalDefs` (in `sifr_lowering/src/lower/external_defs.rs`) and `LowerCtx` (in `sifr_lowering/src/lower/mod_context.rs`) to confirm the plan places both identity maps in the downstream crate.
- Read `bootstrap.rs:150-227` to confirm the re-export mechanism runs for both `_sifr.*` and cross-public-module cases and does not carry intrinsic metadata today.
- Counted the retained-intrinsics module dispatch table at `lib.rs:59-88` (19 modules, matching the plan's 19-module claim).
- Confirmed `stdlib/sifr/test.sifr` and `stdlib/sifr/task.sifr` are the physical homes of the 8 `@compiler_intrinsic` targets and that the current `task.sifr` still imports `task_current_context` from `_sifr.task` (which M3 removes).
- Cross-checked the round-2 non-blocking items against the plan text and confirmed each is now in the plan (§2 concrete metadata; §3 first-class rejection; M1/M4 `retained_dependency_specs` file references; M4 manifest schema deferral; M3 `bytes_to_hex_strict` bridge-target doc).

Plan is approved for execution.
