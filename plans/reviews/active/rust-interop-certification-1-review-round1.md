## Independent review — Rust-interop Track A, `certification_1`

Reviewed the uncommitted diff against `082988df1`. Note up front: **the working tree changed while I was reviewing** — `verification/areas/rust_interop/checks/check_fixture_matrix.py` and ~90 more lines of `_scenario_checks.py` (the `run_self_test` mutation harness) appeared mid-pass. Findings below reflect the tree as of `02:38`.

### What I verified as passing

- All five area checkers: `check_fixture_matrix` (fixtures=36, scenario_examples=11), `check_compatibility_matrix`, `check_stable_support_claims` (claims=24), `check_tiers`, `check_stale_drafts`; `--self-test` ok, cases=89 (now includes the 6 new scenario mutation cases: pin drift, serde feature drift, bridge-path drift, trust drift, missing lockfile, baseline).
- The mandatory evidence test really executes: `cargo test -p sifr_driver --lib -- --ignored test_build_bridge_type_matrix_positive_cargo_probe` → 1 passed (16.8 s). Suite `sifr_driver_generated_builds` is blocking in both `create-pr` and `merge`.
- `install_evidence_source` genuinely overwrites `src/main.sifr` with the checked-in positive evidence; the two sources differ only in header lines and the verifier name — no drift.
- Sifr `assert` lowers to a real `assert!` (`render_core.rs:437`), so the release-built probe binary does check its assertions; the printed literal is guarded.
- New codegen unit tests pass; clippy on `sifr_codegen --all-targets` reports nothing for the new file; `cargo fmt --check` clean; new file is 77 lines, `rust_interop_direct.rs` 898 (under the 900 cap, but with 2 lines of headroom).
- thiserror/bytes/serde_json/indexmap all physically cross `src/bridges/types.rs`; the error path is genuinely `thiserror`'s `#[error("invalid nested payload: {0}")]` Display mapped into `DiagnosticError.message`.

---

## Blockers

**B1 — The insertion-order claim is false, and the evidence cannot observe order.**
Sifr dicts lower to `std::collections::HashMap` (generated signature is literally `fn indexmap_roundtrip(input: &HashMap<String, Vec<String>>) -> HashMap<String, Vec<String>>`), so nothing carries order *into* the `IndexMap`, and `bridge_index_map_to_hash_map_expr` drops it again on return. The positive fixture asserts `nested_result == nested`, which is order-insensitive HashMap equality.

Reproduced in a scratch copy of the scenario with an order-observing bridge (`input.keys().join(",")`), input inserted `alpha,beta,gamma,delta,epsilon`:

```
gamma,beta,alpha,delta,epsilon
beta,gamma,alpha,epsilon,delta
delta,gamma,beta,epsilon,alpha
beta,epsilon,delta,alpha,gamma
```

Nondeterministic, never the insertion order. This contradicts `internal_docs/architecture.md:214` ("Dict order is unspecified (`HashMap`)"). The diff nonetheless newly asserts ordering in four places: `docs/rust-interop.mdx` ("insertion-ordered `dict[str, list[str]]` values"), the new `internal_docs/rust_interop_architecture.md` paragraph, the fixture `README.md` ("preserves insertion order"), and the issue checklist item "insertion-ordered nested dictionaries". (`rust_interop_architecture.md:454` already claimed this pre-change; this PR certifies it.)
Fix: drop every ordering claim and the checklist wording, **or** implement ordered dict semantics end-to-end and add an assertion that actually observes key order.

**B2 — The scenario lockfile is not hermetic against the repo's offline setup.**
Five transitive versions in `examples/bridge_type_roundtrip/Cargo.lock` are absent from the root `Cargo.lock`: `memchr 2.8.3` (root 2.8.0), `proc-macro2 1.0.107` (1.0.106), `quote 1.0.47` (1.0.45), `syn 2.0.119` (2.0.117), `zmij 1.0.23` (1.0.21). Profile setup is exactly `cargo fetch --locked` at the workspace root (`verification/runner/sifr_verify/cargo_setup.py:9`, `merge.json` / `create-pr.json`), then `CARGO_NET_OFFLINE` is forced; the test then runs `cargo metadata --format-version=1 --locked --offline` against this manifest and lock. Those five versions are never fetched by root setup, so on a clean cache the mandatory test fails at the metadata step (`error: no matching package named ... found ... you're using offline mode`, reproduced with an empty `CARGO_HOME`). It passes here only because this machine has them cached — and the newer-than-root versions indicate the lock was resolved against the live index, which is also what the issue's own prerequisite forbids ("confirm every required Cargo crate/version is present in the checked-in lockfile and cacheable by the repository's locked/offline setup"). Every one of the other ten scenario lockfiles is a strict subset of the root graph.
Fix: regenerate the fixture lock so its graph matches the root lock exactly.

**B3 — Nested dict-value conversion is not general; the row over-claims "bridge type generation and conversion".**
`hash_map_to_bridge_index_map_expr` special-cases only a top-level `Type::Int` value; anything else is a blanket clone. But `bridge_dict_type` reports `dict[str, list[int]]` as fully bridge-compatible (value owned type `Vec<SifrIntBridge>`), so no `SIFR-RUST-TYPE-0001` fires. Reproduced end-to-end on a scratch package with a contract-correct bridge signature:

```
error[E0308]: mismatched types  --> src/main.rs:47:5
  expected `&IndexMap<_, Vec<SifrIntBridge>>`, found `&IndexMap<_, Vec<i64>>`
error[E0308]: mismatched types
  expected `HashMap<_, Vec<i64>>`, found `HashMap<_, Vec<SifrIntBridge>>`
error: could not compile `sifr_output` ... SIFR-BUILD-0005
```

Both directions fail (the return-side `bridge_index_map_to_hash_map_expr` has the same top-level-only limitation), and the user sees raw rustc text — an "if it compiles, it works" violation on a shape the row now advertises as supported. `dict[str, dict[str, str]]` has the same shape of mismatch (`HashMap` inner vs required `IndexMap` inner).
Fix: recurse the conversion through nested list/dict/int values on both directions, or emit `SIFR-RUST-TYPE-0001` for the shapes the lowering can't produce.

**B4 — Issue counts not updated.**
`certification_1` has an implementation checklist but no `Post-item inventory` block, while the issue requires recording current counts per item (line 66) and `certification_0` set the precedent. The recorded numbers are now stale: stable claims 23 → 24 (checker prints `claims=24`), `supported-through-bridge` 5 → 6, `future-owned-by-separate-phase` 13 → 12, passing/planned evidence directions 47/25 → 48/24. The final checklist line is correctly still unchecked.

---

## Optional findings

1. Re-review the final diff before commit — the checks self-test landed mid-review.
2. The negative binding `package_rust_interop_rejects_unsupported_bridge_type_contract` is real, blocking, and `create-pr`-mandatory, but it is a synthetic contract test over `set[int]`; it never consumes the checked-in `negative/unsupported_container_rejections.sifr` (`dict[str, list[object]]`). Pre-existing pattern, but the negative side is weaker than the provenance record implies.
3. The new test omits the pristine `check_package_project(&entrypoint).is_empty()` assertion that `test_build_local_bridge_blake3_positive_cargo_probe` performs before installing evidence.
4. `serde` is only observable indirectly (derive under `serde_json`); no independently observed serde value crosses the boundary.
5. Borrowed `dict` params now clone the entire map into a statement-scoped temporary on every call. Lifetime-correct (verified by the passing build), but "borrowed" now allocates — worth stating in the architecture doc.
6. `bridge_type_matrix` is inserted out of alphabetical order in `stable_support_claims.json` and the generated docs table.

NOT SATISFIED
