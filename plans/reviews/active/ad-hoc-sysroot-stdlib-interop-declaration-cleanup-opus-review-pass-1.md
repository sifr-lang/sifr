# Opus Review Pass 1 — Ad-Hoc Sysroot Stdlib Interop Declaration Cleanup

## Overall

The phase is coherent, its Design Rules match the durable contract, and the five milestones cover the right axes (lowering → policy → codegen → source migration → validation). Docs (`rust_interop_architecture.md`, `sifr_sysroot_and_stdlib_architecture.md`) now describe the ellipsis-only, sysroot-effective-policy contract in-place, largely without migration framing. The findings below are the gaps that would cause implementation to rediscover work or produce a subtly-wrong final state.

## Actionable findings (ordered by severity)

### 1. M3 does not close the synthetic `Ok(())` tail-append hole it claims to close
`crates/sifr_codegen/src/function_emitter/generator_bodies.rs:341-352` appends `Ok(())` whenever the return type is `Result[None, _]` and `func.body.last()` is not a `HirStmt::Return`/`Raise`. That guard reads the **HIR** body (Sifr statements), not the direct-body Rust `Vec<RustStmt>` that `direct_rust_function_body(func)` returned at line 324. Once M4 replaces `raise IOError("")` placeholders with a single ellipsis stub, `func.body.last()` becomes an ellipsis (or is empty), so the tail-append will still fire and stack a bare `Ok(())` after the wrapped Rust call for e.g. every `_sifr.compress.zip_*` declaration. M3's "no codegen panic reachable from a bodyless declaration" acceptance criterion does not catch this — the code will type-check and emit doubled tails. Rewrite M3 to say explicitly: when `direct_rust_function_body(func)` supplies the body, the `Result[None, _]` tail-append must be suppressed (or the direct body must own its own tail). Add a `stateless_private_codegen_tests` case that snapshots `zip_create`/`zip_add_file` after the migration.

### 2. Direct-interop error-field mapping loses its current input when the body becomes `...`
`crates/sifr_codegen/src/rust_interop_direct.rs` today maps message-shaped error subclass fields (see `direct_rust_function_body_maps_string_error_fields`, `direct_rust_function_body_maps_json_decode_error_fields`) — used by e.g. `_sifr.regex.re_*` producing `Result[..., RegexError]`. That mapping is HIR-type-driven, but the phase never states this explicitly, and HIR construction currently walks the placeholder `raise RegexError("")` body to resolve the error type. Once the body is ellipsis-only, M1 must guarantee that (a) the annotated return type populates HIR unchanged and (b) any downstream direct-interop pass that resolves error-subclass field metadata does so from the annotated `Result[T, E]` alone. Add an explicit M1 task ("Populate `HirFunction.return_type` from the annotation before body lowering runs, so ellipsis stubs still expose the error class needed by direct-interop mapping") and add a codegen regression for `re_find`/`re_replace` returning `Result[..., RegexError]`.

### 3. Ambiguity: is the ellipsis stub form public or sysroot-only?
`internal_docs/rust_interop_architecture.md:99-103` says ellipsis "is accepted only for Rust interop declarations" without qualifying by package origin, while the phase's motivation ("compiler enforce the declaration form so the stdlib cannot drift") and M4's guard (adapter-policy test enforces "no placeholder `return` or `raise`" only for private declarations) point at a sysroot-only affordance in practice. The phase and doc must pick one:
- If public: `@rust(crc32fast.hash, panic=trusted_no_panic) def crc32(...) -> uint32: ...` is a valid package declaration form, and the adapter-policy test in M4 must enforce ellipsis for user/package interop too (via a separate guard), and public docs (`docs/rust-interop.mdx`) must show it.
- If sysroot-only: `rust_interop_architecture.md` must state that user/package `@rust(...)` declarations still require a concrete body (or the compiler must reject ellipsis in user code with a targeted diagnostic).
As written today, packages could adopt the ellipsis form silently and lowering would accept it, contradicting "Keep public-facing docs focused on package-authored Rust interop; do not expose sysroot shorthand as a user feature."

### 4. M0's rg validation command does not verify any of the invariants the milestone locks
`rg -n "trusted_no_panic|panic surface|ellipsis|\\.\\.\\." …` produces text hits but asserts nothing. The invariants M0 is supposed to lock (declaration body is exactly `...`; sysroot omits `panic=`; user/package keeps `panic=`) are neither positive-matched nor negative-matched by that grep. Replace it with concrete checks a reviewer can run and interpret, e.g.:
- `rg -n '@rust\(sifr_stdlib\.[^)]*panic=' internal_docs docs` (expect zero matches),
- `rg -nF '@rust(sifr_stdlib.' stdlib/_sifr` (expect one match per migrated decl, no `panic=`),
- explicit assertion that `docs/rust-interop.mdx` shows only package-scope `panic=` policies.

### 5. Sysroot vs `sifr_runtime.*` and `Self` targets are excluded by inventory but not by design-rule wording
Design Rules require the target root be "exactly `sifr_stdlib`", which is correct. But the phase does not spell out the diagnostic behavior for a private `_sifr.*` declaration that omits `panic=` and targets `sifr_runtime.*`, `bridge.*`, or `Self`. Per the architecture, `sifr_runtime.*` is a valid sysroot-owned root for private declarations (used for retained runtime glue), so a stray policy-omitted declaration there must fail with a *different* diagnostic than a user/package missing-policy failure. M2 says "Ensure diagnostics distinguish user/package missing policy from invalid sysroot target/context" but does not reserve a code. Reserve a new `SIFR-RUST-PANIC-*` or `SIFR-RUST-TRUST-*` variant (e.g. "sysroot no-panic policy applies only to canonical `sifr_stdlib.*` targets") so this branch is testable and auditable.

### 6. Compiler-surface list is drifted vs. the tests/fixtures the milestones actually touch
The Affected Inventory omits several files that M2/M3/M4 validation explicitly names:
- `crates/sifr_driver/src/build/sysroot_interop_tests.rs` (M2 acceptance evidence),
- `crates/sifr_driver/src/build/rust_interop_panic_contract_tests.rs` (M2),
- `crates/sifr_driver/src/build/rust_interop_tests.rs` (M2),
- `crates/sifr_driver/src/stdlib/stateless_private_adapter_policy_tests.rs` (M4 — the guard actually rewritten from "must have `panic=trusted_no_panic`" to "must not have `panic=`"; this is a **direction reversal** of an existing assertion, worth flagging explicitly),
- `crates/sifr_codegen/src/rust_interop_direct.rs` unit tests (M3 body regression coverage),
- `crates/sifr_lowering/src/lower/rust_interop_tests.rs` (M1).
Add these so a reviewer can navigate the change set without reconstructing it from prose.

### 7. `annotations_and_function_lowering.rs` return-type inference path is not wired to the ellipsis skip
That module calls `infer_function_return_type` (line 649) and `requires_exhaustive_return_annotation` (line 616). Both currently walk `func.body` and will produce spurious diagnostics on an ellipsis-only body (e.g. "missing return in non-void function", "cannot infer return type from empty body"). M1's task list says "skip normal statement lowering, missing return checking, and return-type inference from body expressions" — which is the right idea, but "skip" is under-specified given both of those functions live *outside* the statement-lowering loop and are invoked unconditionally. Concretize M1 to name the two call sites and specify the skip as "for eligible interop stubs, bypass both `infer_function_return_type` and `requires_exhaustive_return_annotation` because the declared return type is authoritative."

### 8. `Expr::EllipsisLiteral` diagnostic path does not exist today
`grep -rn EllipsisLiteral crates/` returns zero matches; ordinary expression lowering doesn't touch ellipsis at all. M1 phrases the guard as "Keep ordinary expression lowering unsupported for ellipsis outside this specialized declaration path" — but there is nothing to keep; the diagnostic must be **added**. The listed surface `crates/sifr_lowering/src/lower/expressions/core_and_calls.rs` may not even be the right file (the expression dispatcher's default arm probably lives elsewhere). Fix the wording ("Add a targeted diagnostic in the expression-lowering default arm for `Expr::EllipsisLiteral`") and confirm the correct file — likely `crates/sifr_lowering/src/lower/expressions.rs` or its dispatch module.

## Non-blocking polish

- `internal_docs/sifr_sysroot_and_stdlib_architecture.md:141-144` still frames sysroot no-panic policy as covering "public APIs whose error surface does not include `RustPanicError`", which reads as scope-limited. Rewrite so the policy applies uniformly to canonical private `sifr_stdlib.*` bindings regardless of Result shape.
- Same doc, line 137 "existing sysroot interop declarations" retains a migration voice. Prefer "sysroot stdlib interop is `bridge-version = 1`; callee-injection is a future bridge-versioned design and does not add fallback conversion behavior."
- The `Pre-Migration Baseline` section (§ starting line 20) is explicitly transitional. It contradicts the criterion "Docs describe only the durable declaration contract." Either move the baseline table under `plans/phases/` or add a phase task to retire it in M4/M5.
- Review Checklist is missing: (a) "Ellipsis in a non-interop function body produces a targeted diagnostic"; (b) "Direct-interop `Result[None, _]` body does not double-emit `Ok(())`"; (c) "Cache fingerprint changes when a declaration flips between explicit `panic=` and effective sysroot policy."
- The phase does not mention scanning `internal_docs/architecture.md`, `plans/phases/`, and `internal_docs/stdlib_retained_compiler_intrinsics.toml` for stale references to `panic=trusted_no_panic` on private stdlib declarations. Add a sweep task to M0 or M5.
- `_sifr.crypto.sifr:2-3` retains a comment "Random helpers still use compiler intrinsics during the incremental stateless-leaf migration." M4 should update or remove that comment since post-cleanup the file is a pure declaration module with no migration narrative.
- M5 says "Update roadmap/phase tracking if this phase is added to a tracked milestone." Given `19e346f0f Archive sysroot stdlib toolchain phase` just landed, state explicitly whether this ad-hoc is a follow-up of that archived phase or independent, and where its closeout evidence should be linked from `plans/roadmap.md`.
- Consider naming M4's test guard something like `completed_private_declarations_use_ellipsis_stub_and_no_panic_policy` so the reversed direction (was: must contain `panic=trusted_no_panic`; now: must not) is obvious from the test name.
