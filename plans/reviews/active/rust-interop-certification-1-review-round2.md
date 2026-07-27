Review complete. I re-verified all four round-1 blockers with independent probes (fresh package builds, lock-graph diffing, count recomputation). Here is the review.

---

## Independent review — Rust-interop Track A, `certification_1` (round 2)

Baseline `082988df1`; uncommitted working tree as reviewed. No files modified. The untracked `plans/issues/active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md` was ignored per scope.

### Re-evaluation of round-1 blockers

**B1 — false insertion-order certification — RESOLVED.**
`grep -rn insertion docs internal_docs plans verification` now returns no ordering claim for the bridge. `internal_docs/rust_interop_architecture.md:454-464` states plainly that the internal representation is a `HashMap` and order is not preserved in either direction, and that the certification "does not certify key iteration order". `docs/rust-interop.mdx:92` says iteration order "remains unspecified across this bridge"; the fixture README and the scenario README both carry the disclaimer. This is now consistent with `internal_docs/architecture.md:214`. The checklist item was reworded to "without claiming key iteration order".

**B2 — non-hermetic scenario lockfile — RESOLVED.**
I re-diffed every scenario lock against the root lock by `(name, version, has-source)`:

```
OK   bridge_type_matrix/examples/bridge_type_roundtrip/Cargo.lock []
OK   (10 other scenarios) []
```

All registry packages in the scenario lock are now a strict subset of the root lock (the five previously drifted versions — `memchr`, `proc-macro2`, `quote`, `syn`, `zmij` — are gone). `serde_derive` is present in both, so the `derive` feature is covered by root `cargo fetch --locked`. The rule is now enforced, not just fixed: `_require_root_lock_subset` (`_scenario_checks.py:625`) compares against `REPO_ROOT / "Cargo.lock"` (path arithmetic verified correct), and `run_self_test` includes a `memchr 2.8.0 → 2.8.3` mutation plus pin-drift, feature-drift, bridge-path, trust, and missing-lock mutations, wired into `check_fixture_matrix.py:_run_self_test`. I re-ran the area: `variants=10, failures=0`, `claims=24`, `rows=36`.

**B3 — non-recursive nested conversion — PARTIALLY RESOLVED; a residual blocker remains (see below).**
The dict-value path is genuinely fixed. I did not rely on the unit tests: I copied the scenario to a scratch tree and built real packages against contract-correct bridges. All of these now build and run green (`serde:nested|bytes:6|invalid nested payload`, exit 0):

- `dict[str, float]`, `dict[str, bytes]`, `dict[str, tuple[str, float]]`, `dict[str, str | None]`
- with a `sifr_runtime` path dep to make `SifrIntBridge` referenceable: `dict[str, int]`, `dict[str, list[int]]`, and `dict[str, dict[str, list[int]]]` — i.e. the exact shapes that produced E0308 in round 1 now compile and round-trip in both directions.

`to_i64_saturating` is the correct inverse for the exact-int bridge (`sifr_runtime/src/interop.rs:56`), and `tuple_item_rust_type` maps `int → i64`, so the pass-through for tuple values is right rather than accidental.

**B4 — stale post-item inventory — RESOLVED.**
Recomputed from the data files rather than trusting the doc: 36 fixture rows / 36 compatibility rows / 36 schema-v2 manifests; 48 passing and 24 planned evidence directions; categories `supported`=17, `supported-through-bridge`=6, `unsupported-by-design`=1, `future-owned-by-separate-phase`=12; execution kinds `cargo-probe`=13, `compiler-diagnostic`=4, `contract-only`=10, `runtime-observed`=9; 44 distinct required crate aliases; 24 stable claims. Every number in the new `certification_1` "Post-item inventory" block matches exactly, and the `certification_0` row is correctly flipped to merged with PR #3026.

Also re-confirmed: the positive fixture and the scenario `src/main.sifr` differ only in header lines and the verifier name; the ignored evidence test asserts the exact literal; `rust_interop_direct.rs` is 868 lines (was 898) and the new module is 194; the four new unit tests pass locally.

---

### Blocking findings

**B3-residual — the same conversion hole survives one container level out; `list[dict[...]]` and `list[list[int]]` still leak raw rustc errors.**

The fix recurses *inside* dict values, but the top-level dispatch in `crates/sifr_codegen/src/rust_interop_direct.rs:55-91` and `:169-181` still matches only `Type::Int`, `list[int]` (depth-1, via `is_int_list`), `dict[str, T]`, `str | None`, and `int | None`. Everything else passes through unconverted, while `bridge_list_type` (`rust_interop_bridge_contract.rs:542`) reports *any* bridge-compatible element as compatible, so no `SIFR-RUST-TYPE-0001` fires.

Reproduced end-to-end on scratch copies of the certified scenario, with contract-correct bridge signatures:

```
# list[dict[str, str]]  ->  &[IndexMap<String, String>]
error[E0308]: mismatched types
   expected `Vec<HashMap<String, String>>`, found `Vec<IndexMap<String, String>>`
error: could not compile `sifr_output` ... SIFR-BUILD-0005

# list[list[int]]  ->  &[Vec<SifrIntBridge>]
error[E0308]: mismatched types
   expected `&[Vec<SifrIntBridge>]`, found `&Vec<Vec<i64>>`
error: could not compile `sifr_output` ... SIFR-BUILD-0005
```

Both directions fail and the user sees raw rustc text — the same "if it compiles, it works" violation round 1 blocked on, on shapes the now-`supported-through-bridge` row advertises as "bridge type generation and conversion" (notes: "Generated package glue roundtrips nested serde/serde_json, thiserror, bytes, and indexmap values"). This is pre-existing on `main`, but the row was `future-owned-by-separate-phase` there; promoting it to a certified stable claim is what brings the gap into scope, and it is the identical root cause with the identical remedy the previous round specified ("recurse … **or** emit `SIFR-RUST-TYPE-0001` for the shapes the lowering can't produce").

Fix (small, the machinery already exists): route list elements and `Option` payloads through `rust_interop_direct_collections`' recursion in both `direct_rust_arg_expr` and `direct_rust_return_expr`, replacing the `is_int_list` / `is_optional_*` special cases — or, if the scope is to stay narrow, emit `SIFR-RUST-TYPE-0001` for container shapes the lowering cannot produce and narrow the row's capability and notes to match. Docs narrowing alone is not sufficient: the raw-rustc leak would remain on a certified row.

---

### Optional suggestions

1. `fixtures/bridge_type_matrix/README.md:5` still reads "ordered `indexmap::IndexMap` conversion" two lines above the "order is not preserved" disclaimer. Drop "ordered" — the adjective is the exact wording B1 was about.
2. The root-lock subset rule is inside the `fixture_id == "bridge_type_matrix"` branch only. All ten other scenario locks happen to be clean today, but nothing prevents the next one from drifting. Hoisting `_require_root_lock_subset` out of the per-fixture branch makes B2 a structural guarantee rather than a row-local one.
3. No executable evidence covers exact-`int` dict values: the scenario crate cannot name `SifrIntBridge` without depending on `sifr_runtime` (I confirmed — `E0433: could not find sifr_runtime`), so the `Type::Int` recursion has string-assertion unit coverage only. It does work (I verified with a path dep), but the certified probe cannot observe it. Worth a note in the fixture provenance, or a future scenario that legitimately exposes it.
4. Carried from round 1 and still true: `bridge_type_matrix` is inserted out of alphabetical order in `stable_support_claims.json` and the generated docs table; the negative evidence binding remains the synthetic `set[int]` contract test rather than the checked-in `negative/unsupported_container_rejections.sifr`; the new test omits the pristine `check_package_project(...).is_empty()` assertion its `local_bridge_blake3` sibling performs; and borrowed `dict` params now clone the whole map per call, which the architecture doc does not mention.

---

## NOT SATISFIED
