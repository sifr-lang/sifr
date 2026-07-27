I've completed the review. All verification was read-only; no files modified.

---

## Final PR Review — Sifr Rust Interop Certification `certification_1`, PR #3027

**Baseline:** `082988df1` **Pushed head:** `3ce0aa72f` **PR:** #3027 (open, draft, base `main`, head `agent/rust-interop-certification-1`, 28 files)

### 1. Commit scope and coherence

Two commits, cleanly separated:

| Commit | Scope |
| --- | --- |
| `c4efd70a1` "certify Rust bridge type roundtrips" | 28 files, +1321/−77 — the implementation, evidence, data, docs, and rounds 1–4 artifacts |
| `3ce0aa72f` "record certification review and PR" | 1 file, +13/−1 — status row `in progress → review` with the #3027 link, plus the "Review and gate evidence" block |

I diffed the second commit in full: it touches only `plans/issues/active/rust-interop-runtime-ecosystem-certification.md` and adds exactly the review/gate prose described. It matches the stated delta since round 4 — no implementation change slipped in.

`gh pr view 3027` reports the same head OID and the same 28 paths as `git diff --stat 082988df1 3ce0aa72f`. Every path is inside the row's own blast radius: `sifr_codegen` interop lowering, the one `sifr_driver` evidence test, `bridge_type_matrix` fixture/scenario, the three `rust_interop` data files, the two docs, the area checker, and the review artifacts. **No accidental scope.**

### 2. Untracked files are correctly excluded

`plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md` and `plans/reviews/active/rust-interop-certification-1-review-round5.md` (currently 0 bytes) are untracked in `git status` and absent from both the commit range and the PR's 28-file list. The ad-hoc algorithmic-corpus issue is a separately-owned pre-existing failure set already documented as non-blocking in `certification_0`'s validation notes; **it does not block this milestone and is not part of this PR.**

### 3. Prior blockers — all still resolved at the pushed head

**B1 — false insertion-order certification.** Resolved and still resolved. `internal_docs/rust_interop_architecture.md:454-467` now states plainly that Sifr's internal representation is a `HashMap` and order is *not* preserved in either direction, and that the certification "does not certify key iteration order." `docs/rust-interop.mdx` says order "remains unspecified across this bridge"; both READMEs carry the disclaimer. A targeted grep over `docs/`, `internal_docs/`, `verification/areas/rust_interop/`, and the issue returns no surviving ordering claim for the bridge — only unrelated concurrency `map`-ordering text and the correct negative statements. Consistent with `internal_docs/architecture.md:214`.

**B2 — non-hermetic scenario lockfile.** Resolved, and enforced structurally. I re-diffed all **11** scenario lockfiles against the root `Cargo.lock` by `(name, version, has-source)`: **0 drifted**. The rule is hoisted out of the per-fixture branch (`_scenario_checks.py:362-386`) so it applies to every scenario, missing-lockfile is a failure, and both the `memchr 2.8.0 → 2.8.3` drift and the deleted-lock mutations are in `run_self_test`, now wired into `check_fixture_matrix.py:_run_self_test`.

**B3 / B3-residual — non-recursive composite conversion.** Resolved. `composite_conversion_required` (`rust_interop_direct.rs:104`) returns true exactly for `Int`, `List` with a converting element, `str`-keyed `Dict`, and `Option` with a converting payload; the return side dispatches through the same predicate before `Type::Result` (which `_ => false` prevents it from swallowing). `rust_interop_direct_collections.rs` recurses through list, dict, exact-`int`, and `Option` payloads in both directions, with `value_iter` selecting `iter()`/`into_iter()` by convention and the outer `&` added only for `List`/`Dict` — correctly omitted for `Option`, whose bridge borrowed type is the owned `Option<T>`. Rounds 2–4 built real packages for the exact E0308 shapes (`list[dict[str,str]]`, `list[list[int]]`, `dict[str,list[int]]`, depth-3 nesting, `Result`-wrapped composites) and all round-trip green.

**B4 — stale inventory counts.** Resolved. I recomputed from the data files rather than trusting the doc; every number in the `certification_1` post-item block matches exactly:

| Metric | Issue | Recomputed |
| --- | --- | --- |
| fixture rows / compat rows / schema-v2 manifests | 36 / 36 / 36 | 36 / 36 / 36 |
| evidence passing / planned | 48 / 24 | 48 / 24 |
| categories (supported / through-bridge / unsupported / future-owned) | 17 / 6 / 1 / 12 | 17 / 6 / 1 / 12 |
| execution kinds (probe / diagnostic / contract / runtime) | 13 / 4 / 10 / 9 | 13 / 4 / 10 / 9 |
| crate aliases | 44 | 44 |
| stable claims | 24 | 24 |

No stale `future_owner` field survives on any promoted row.

**B5 — Rust-keyword parameter names caused an ICE.** Resolved. `sifr_composite_to_bridge_expr` takes `&RustExpr` and calls `render_expr`, so the root goes through `render_identifier` and emits `r#…`; `direct_rust_arg_expr` passes the existing `RustExpr::Ident(param.name)`. Only compiler-owned binders (`__sifr_bridge_item_N`, `__sifr_bridge_key_N`, `__sifr_value_N`) reach the `Verbatim`. Critically, the fix is **observed by the certified probe, not just by a unit test**: the checked-in scenario declares `def indexmap_list_roundtrip(type: list[dict[str, str]])`, so the mandatory cargo-probe exercises a Rust-keyword name on a nested composite shape. `composite_root_identifiers_escape_rust_keywords` asserts `&r#type.iter()`.

### 4. Gates I reproduced independently at the pushed head

- `cargo test -p sifr_codegen rust_interop_direct --lib` → **26 passed, 0 failed**
- `cargo test -p sifr_driver --lib -- --ignored test_build_bridge_type_matrix_positive_cargo_probe` → **1 passed (22.0 s)** — builds and executes the release package binary, asserts the pristine `check_package_project(...).is_empty()` precondition and the exact literal `serde:nested|bytes:6|invalid nested payload`
- Full area runner → `variants=10, failures=0, blocking_failures=0, non_blocking_failures=0`
- All four checkers plus self-tests: `fixtures=36 diagnostics=10 crates=44 package_examples=60 scenario_examples=11`, self-test `cases=90`; `rows=36`; `tiers=5`; `claims=24`
- `RUSTFLAGS=--cfg reviewprobe cargo clippy -p sifr_codegen --lib -- -D warnings` (forced fresh compile, not cache replay) → clean
- `cargo fmt --all --check`, `git diff --check 082988df1 3ce0aa72f`, `check_file_size_guardrails.py` (2853 files, limit 900), `check_hir_maintainability_guardrails.py` → all pass. Sizes: `rust_interop_direct.rs` **873** (was 898 at baseline — the refactor *bought* headroom), collections module 277, `_scenario_checks.py` 728.

### 5. Provenance, hermeticity, and claim honesty

- **Executable provenance.** `fixture.json` binds `test_build_bridge_type_matrix_positive_cargo_probe` in `crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs`, suite `sifr_driver_generated_builds`, step `crate_tests`, profile `merge`. That suite is registered `"status": "blocking", "executed_in_merge": true` in `create-pr.json:90`, `merge.json:73`, and `nightly.json:75`. README prose repeats the structured record verbatim. Promotion rules 1–4 are satisfied by data, not prose.
- **Real crate crossings.** `src/bridges/types.rs` physically uses `serde` (derive), `serde_json`, `thiserror` (`#[error("invalid nested payload: {0}")]` → `DiagnosticError.message`), `bytes::Bytes`, and `indexmap::IndexMap` — including `IndexMap<String, IndexMap<String,String>>` and `&[IndexMap<String,String>]`, so the recursive conversion is genuinely exercised across the boundary.
- **Hermetic.** All five deps exact-pinned (`=1.11.1`, `=2.14.0`, `=1.0.228` + `derive`, `=1.0.149`, `=2.0.18`), enforced by `_require_exact_dependency` / `_require_dependency_features` with drift mutations in the self-test; lock is a strict root subset; no network, no external services; the test copies to a temp tree and removes it.
- **No drift between fixture and scenario.** `diff positive/supported_type_roundtrips.sifr examples/.../src/main.sifr` shows only header lines and the verifier name — 4 hunks, nothing semantic.
- **No-panic.** `rust_interop_direct_collections.rs` emits only `iter()`/`into_iter()`/`map()`/`collect()`/`clone()`/`as_ref()`/`SifrIntBridge::from`/`to_i64_saturating` — no `unwrap`, `expect`, `panic!`, or indexing. `to_i64_saturating` (`sifr_runtime/src/interop.rs:56`) is genuinely total, saturating to `i64::MIN`/`MAX`. The only `assert!`s in the probe are the six user-authored evidence assertions. Round 4 confirmed the same on emitted output.
- **Claim scope.** Exactly one row promoted (`bridge_type_matrix`, `future-owned-by-separate-phase`/`planned` → `supported-through-bridge`/`passing`), with the matching stable claim, docs table entry, and honest notes. The negative `unsupported_container_rejections` evidence is retained unchanged. `certification_2` remains `blocked`; the final checklist item is correctly still unchecked pending merge.

### 6. Non-blocking observations (carried, none new to this push)

1. `python_object_callback_adapter_expr` (`rust_interop_direct.rs:157-158`, untouched from baseline) still interpolates a raw `param.name` into a `Verbatim` — the last instance of B5's root cause, outside this row. Natural follow-up.
2. `is_message_error_alias` name-whitelists bridge error classes; the fixture works because its class is named `DiagnosticError`. Pre-existing, untouched.
3. `hash_map_to_bridge_index_map_expr` and `bridge_index_map_to_hash_map_expr` are now referenced only by the module's own unit tests; the crate-level `#![allow(dead_code)]` (pre-existing, `lib.rs:2`) masks this. They are thin wrappers over the live composite helpers, so the tests still exercise real paths — cosmetic only.
4. Carried cosmetics: `bridge_type_matrix` sits out of alphabetical order in `stable_support_claims.json` and the generated docs table; negative evidence binds the synthetic `set[int]` contract test rather than the checked-in `negative/unsupported_container_rejections.sifr`; the certified probe cannot observe exact-`int` payloads without a `sifr_runtime` path dep (verified working out of band in rounds 2–4); `--all-targets` clippy still has the same 14 pre-existing errors present verbatim at `082988df1`, and is not the gated invocation.

None of these is inside the promoted row's certified contract, and none is a regression introduced by this PR.

---

The pushed diff is complete, internally coherent, free of accidental scope, and resolves every prior blocker. The untracked ad-hoc algorithmic-corpus issue and the empty round-5 stub are correctly outside the commit and PR and must not block this milestone. PR #3027 is ready for the authoritative merge profile and merge.

## SATISFIED
