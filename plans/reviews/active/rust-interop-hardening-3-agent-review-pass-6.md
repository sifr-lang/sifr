## Review — `hardening_3`, round 6

Scope: working tree vs `HEAD` (`fa288b02c`), excluding `ad-hoc-class-*` plan/review files, `internal_docs/typescript_go_architecture_transfer_guardrails.md`, and phase-40 artifacts. No files modified.

### Re-audit of the three round-5 findings — all resolved

**Round-5 finding 1 (create-PR lane broken by taxonomy collision) — resolved and independently verified end to end.** The colliding identifiers are gone; the two remaining execution-kind mutation cases use hyphenated labels (`_provenance_checks.py:746` `"contract-only-source"`, `:763` `"cargo-negative-source"`) and locals `nonruntime_kind_failures` / `cargo_negative_runtime_failures`. `verification_taxonomy.py` passes. I ran `scripts/run_all_tests.sh --profile create-pr` myself: exit 0, **22/22 lane steps pass**, `name=rust_interop_checks elapsed_ms=2944 budget_ms=5000 status=pass`, crate tests pass, E2E 131/131, hardening variants 6/0 failures. Only advisory: `warm wall-time budget exceeded` (non-blocking; `python_interop` is 391565ms of the 655s wall).

**Round-5 finding 2 (negative `cargo-probe` could bind a runtime test) — resolved.** `_validate_execution_kind_source` (`_provenance_checks.py:384-416`) is now symmetric: `runtime-observed` must bind `crates/sifr_runtime/`, every other kind is rejected from it, and positive `cargo-probe` additionally requires a generated-build suite or the explicit marker. Verified by direct invocation against the real repo: negative cargo-probe → runtime test is rejected; contract-only → runtime test is rejected; non-probe positive cargo-probe is rejected; wrong package and wrong weakest profile are rejected. Mutation coverage at `:743-775`.

**Round-5 finding 3 (declared outcome unbound to the test) — resolved, and it works on real data.** `_rust_test_outcomes.py` resolves reachable same-file bodies, masks comments/literals, extracts only balanced `assert!/assert_eq!/assert_ne!/assert_matches!` spans, and resolves `DiagnosticCode::` constants through `registry.rs`. I checked all 25 outcome-bearing bindings: each resolves to **exactly one** asserted code (no helper pollution), every control passes, and **every mutated declaration is rejected** (23 diagnostic swaps + both runtime-state swaps). Targeted parser probes: a bare string literal, a comment, and a code held in a `let` outside the assertion all fail closed; `assert_matches!` + constant succeeds. Constant resolution cross-checked against a stricter statement-scoped parse of `registry.rs` — 0 mis-mappings across 184 constants.

### Independently re-verified

- 47 passing evidence directions across 34 schema-v2 manifests, each with a distinct test (0 shared); 23 claimed rows × 2, plus the `bridge_type_matrix` negative on a future-owned row.
- All 47 README canonical-provenance sentences match `fixture.json` (test name, file, suite, profile) — script-checked, 0 mismatches.
- Weakest-profile bindings are correct: `sifr_driver_generated_builds` is `full`-only/blocking → `merge`; `sifr_driver_lib`/`sifr_runtime`/`sifr_lowering` → `create-pr`.
- The `executes-cargo-probe` marker is truthful on all three uses (`rust_interop_contract_tests.rs:99,261`, `rust_interop_async_contract_tests.rs:104` — each builds a real backend crate root and reaches `execute_direct_cargo_probe`).
- Tier-1 evidence really executes: `cargo test -p sifr_driver --lib -- --ignored --test-threads=1` → **40 passed / 0 failed** (373s), and the six `*_cargo_probe` rows re-run green in 52s.
- Execution-kind data changes are only *downgrades* (`async_runtime_core`, `callback_subscription_core`: runtime-observed → contract-only) with capability, notes, `.sifr` headers, READMEs, `internal_docs`, and the certification issue updated consistently. No row was upgraded.
- Gates: fixture matrix self-test 68 cases; compatibility 4; tiers 6; full area 7 variants / 0 failures; `sifr_verify --self-test` pass; `cargo fmt --check`; `git diff --check`; file-size guardrail (2828 files); HIR guardrails.

### Every `hardening_3` exit condition

Schema v2 with the exact `validation` object ✔; distinct positive/negative tests added where a README pointed at a broad module (new `rust_interop_evidence_contract_tests.rs`, the local-bridge pair, the lowering hidden-blocking test, the renamed async probe test) ✔; all passing evidence migrated across 34 rows ✔; `check_fixture_matrix.py` validates suite/profile/file/test ownership ✔; `check_compatibility_matrix.py` requires two-sided provenance for all claimed categories ✔; README removed as validator input (only existence is checked) ✔; self-tests exist for missing suite, wrong profile mode, non-blocking suite, missing/duplicate test, ignored-command mismatch, path escape, shared test, status/provenance mismatch, README-only claim ✔. Exit gate — two executable tests per claimed row, mutations pass, area passes through the profile step, no planned evidence falsely bound ✔.

### Non-blocking observations

- `crates/sifr_driver/src/build/rust_interop.rs:303-313` still embeds `let … else { return }` in a struct-literal field. Correct and fails closed, but hoisting it above the `match` would read better. The new `plan_package_bridge_probe` (`probe_planning.rs:52-98`) also duplicates ~25 lines of the direct-probe planning block at `rust_interop.rs:401-429`.
- Two independent Rust maskers now exist in the same check package (`_rust_test_evidence.py:323` line-oriented, `_rust_test_outcomes.py:174` offset-preserving). Justified by different needs, but worth unifying later.
- `_rust_test_outcomes._mask_rust_noncode` treats `'a` lifetimes with the char-literal rule; an unpaired lifetime inside an assertion would blank following code. Zero current instances, and the realistic failure direction is fail-closed.
- `expected_result: "pass"` positives (21 rows) still have no mechanical outcome binding — the current tests do assert acceptance via `.expect(...)`, but there is no canonical shape to check.
- `verification/areas/rust_interop/fixtures/opaque_resource_core/README.md:6` still says "the executable evidence is the named runtime test filter below" above a broad `cargo test -p sifr_runtime interop`; the file's own canonical section (`:19-25`) contradicts it by declaring the structured record authoritative.
- `_validate_command_filters` still handles only `--skip`; `dependency_features` now applies its `segments[1]`-as-feature heuristic to package bridges (would enable a same-named feature of the user's own package). Zero current instances for both.
- Near-cap hand-maintained files: `rust_interop.rs` 896, `rust_interop_contract_tests.rs` 872, `_provenance_checks.py` 865, `check_fixture_matrix.py` 861, `rust_interop_probe.rs` 857.
- `examples/blake3.sifr:6-7` remains an illustrative direct binding to `blake3.hash` on a bridge-required row; package examples are not compiled.

The milestone is met: no claim is stronger than executed evidence, both previously-observed classes of mis-binding are now mechanically impossible, and the authoritative lane is green.

Actionable findings: 0. SATISFIED.
