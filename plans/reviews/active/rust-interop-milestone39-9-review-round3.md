## Phase 39 milestone_39_9 review — round 3

**Verdict:** No blockers. The cleanup items called out for this round are present and correct; the contract surface, diagnostic wiring, fixture/catalog data, and docs are internally consistent and panic-free in user paths. The milestone can be signed off and taken through `create-pr` validation.

Verified scope of work since round 1: the new zero-copy/view validator, its diagnostic code (`SIFR-RUST-ZC-0001`), the contract test module, the fixture matrix flip to `contract-only`, the two new fixture READMEs, the architecture/phase/inventory doc updates, and the round-2 scratch-naming changes to `temp_package_root` and `unique_probe_nonce`.

---

### Severity-ranked findings

#### Blockers
None.

#### Medium
None.

#### Low — `opaque_probe_obligations` now special-cases `View` more visibly
`crates/sifr_driver/src/build/rust_interop/opaque_validation.rs:91-99`

Round 1 finding 7 was a maintainability nit about the View probe send/sync derivation living in two places. This round's cleanup did not move/rename anything; instead it inserted an explicit `if kind == View { return (false, false); }` inside the existing `!= Opaque` branch. The behavior is correct — it cancels the prior implicit `(async_boundary, view)` propagation so the explicit `@rust.view(send=…, sync=…)` contract owns those flags, and the architecture doc records that migration — but the function is now named `opaque_probe_obligations` while carrying a dedicated View short-circuit, which is harder to read, not easier. Worth resolving in milestone_39_10 by either renaming to a kind-neutral helper or moving the dispatch into `push_probe`. Not blocking.

#### Low — recommended negative-coverage gaps from round 1 finding 6 are still open
`crates/sifr_driver/src/build/rust_interop_zero_copy_contract_tests.rs`

The async + `lifetime=static` positive test was added as requested (`package_rust_interop_accepts_async_static_lifetime_view`, lines 74-97), but the other coverage suggestions from round 1 — missing required keys on `@rust.zero_copy(...)`/`@rust.view(...)`, duplicate decorator-of-the-same-kind on one function, and a bare `@rust.view(...)` with no `@rust.zero_copy(...)` — were not added. The validator code paths exist (`parse_*_contract` returns `Err` on missing keys; the `if zero_copy.is_some()` / `if view.is_some()` guards push duplicate diagnostics). They are simply unprotected by regression tests. Round 1 explicitly framed these as recommended-not-blocking; same call here.

#### Low — cascade of diagnostics from a single bad `@rust.view(...)` is still possible
`crates/sifr_driver/src/build/rust_interop/zero_copy_validation.rs:121-162`

After the cleanup, the "invalid parse → suppress paired-view-missing" cascade is fixed (verified by `package_rust_interop_rejects_legacy_mutable_bool_key`, which now asserts `diagnostics.len() == 1`). However, the post-parse semantic checks still fall through one another: a single declaration that combines `lifetime=call` with async, or owner-mismatch with a mutable-from-shared-borrow owner, can push two `SIFR-RUST-ZC-0001` diagnostics for the same view span. None of the current tests exercise the combined case, so the cascade only matters for diagnostic-stability tooling that may dedupe by `(code, span)`. Same diagnostic-stability concern that round 1 raised in finding 5; not blocking, worth a deliberate decision before `SIFR-RUST-ZC` gets wider adoption.

#### Low — `rust_interop.rs` headroom unchanged at 892/900 lines
The new validator is correctly housed in `rust_interop/zero_copy_validation.rs`, so the parent file did not grow, but it also did not shrink. Milestone_39_10/11 will need to extract another responsibility from `rust_interop.rs` before adding another inline validator. Not blocking for this milestone.

---

### Round-1 cleanup checklist (verified)

- **Invalid `@rust.view(...)` parse suppresses paired-view diagnostic** — `zero_copy_validation.rs:64,85,108`: `saw_view_declaration` flag flips on the `View` arm before parse attempts, and the post-loop "requires a paired" branch is guarded by `if !saw_view_declaration`. Confirmed by `package_rust_interop_rejects_legacy_mutable_bool_key` asserting `diagnostics.len() == 1`.
- **Dead `view.is_empty()` and `let _ = (view.send, view.sync);` removed** — `ZeroCopyContract` no longer carries a `view` field (only `owner`), and the discard is gone. Verified by re-reading the whole `validate_zero_copy_group` + `parse_zero_copy_contract`.
- **Async + `lifetime=static` positive test added** — `rust_interop_zero_copy_contract_tests.rs:74-97`, asserts a View probe is present.
- **Architecture-doc note on explicit Send/Sync sourcing** — `internal_docs/rust_interop_architecture.md:661`: "View probes now derive Send/Sync obligations from the explicit `@rust.view(...)` contract rather than implicit ABI flags…".
- **Direct-filesystem inventory line anchors updated** — `internal_docs/typescript_go_architecture_transfer_guardrails.md:68` line numbers re-pointed to 788/792/800 and `rust_interop_probe.rs:41`, matching the new function bodies.

### Round-2 scratch-naming changes (verified, no regressions)

- `temp_package_root` (`rust_interop_contract_tests.rs:751`, `rust_interop_tests.rs:745`) appends `as_nanos()` to the PID — collision-free across same-PID rapid runs and across any future caller that re-uses a `name`. `map_or(0, …)` is panic-free; `.expect("remove stale temp root")` remains acceptable as test-only programmer-invariant guard.
- `unique_probe_nonce` (`rust_interop_probe.rs:128`) returns `format!("{timestamp_nanos}_{counter}")`. Process-local `AtomicU64` with `Relaxed` is correct — only uniqueness matters, not happens-before — and the sole caller interpolates via `format!`, so the type change from `u128` to `String` is API-safe. Cross-process uniqueness still comes from the surrounding `std::process::id()`.

### Other items spot-checked

- `SIFR-RUST-ZC-0001` wiring is consistent across the registry (`registry.rs:89,713`), the entry table (`registry_entries/rust_interop.rs:73-83`), the catalog (`code_catalog.json:2579`), the baseline-coverage deferral (`code_baseline_coverage.json:1658`), and the generated `docs/errors/SIFR-RUST-ZC-0001.md`. Message template, args, and representative-fixture pointer agree.
- Validator runs before `resolve_declaration` and the early-return on `!self.diagnostics.is_empty()` prevents malformed contracts from leaking into probe planning, trust checks, or `cargo` probe execution (`rust_interop.rs:133-136`).
- View probe Send/Sync union-merge at `rust_interop.rs:563-575` preserves the `requires_send = false` override for `AsyncThreadAffinity::TokioCurrentThread` after the OR-in.
- HIR-level lowering test now matches the new `@rust.view(owner=…, lifetime=…, mutability=…, send=…, sync=…)` shape (`crates/sifr_lowering/src/lower/rust_interop_tests.rs:34`).
- Fixture matrix flip to `execution_kind: contract-only` with status `passing` is internally consistent with the two new READMEs, which explicitly call out that runtime-observed crate certification is still staged. Docs do not overclaim — `plans/phases/39_rust_interop.md:200` and `internal_docs/rust_interop_architecture.md:661` both say "Runtime-observed crate-backed certification … remains staged."
- No data-dependent `unwrap`/`expect`/`panic!` introduced on the user diagnostic path; all parse branches return `Result<_, &'static str>` and convert to diagnostics through `push_zero_copy_diagnostic`.
- File-size guardrail respected: largest touched file is `rust_interop.rs` at 892/900 (unchanged); new module is 325 lines, new test module 301 lines.

---

### Conclusion

**No further review round is required for milestone_39_9.** Findings above are deferrable to milestone_39_10 or a follow-up cleanup PR. The contract-only milestone is ready for `create-pr` validation.
