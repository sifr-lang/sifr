## Independent Final Review — Rust Interop certification_14 / Track A closeout

**Scope reviewed.** Working-tree diff vs `origin/main` (the branch has zero commits ahead; all certification_14 work is uncommitted): `crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs`, both scenario validators, two fixture `sifr.toml`/`README.md` pairs, and the four planning documents. Excluded per instruction: `editor_integrations`, `verification/areas/algorithmic_compatibility/corpora/leetcode`, `.cert5probe/`, `.claude/`, stray `*.webp`, `plans/phases/43_interoperability.md`, and dormant Track B.

### Validation I reproduced independently

| Gate | Result |
|---|---|
| 3 generated build tests (`--ignored`) | 3 passed, 0 failed (26.3s) |
| Full `rust_interop` area | `variants=10, failures=0, blocking_failures=0, non_blocking_failures=0` |
| Runner inventory | `fixtures=36 diagnostics=10 crates=44 package_examples=61 scenario_examples=18`; `rows=36 fixture_rows=36 categories=3`; `claims=36` |
| Self-tests | fixture 233, compatibility 7, tiers 6, stable-claims 33, stale-drafts 20 |
| Resource-certification gate | `PASS (surfaces=1, future_runtime_rows=0)`; self-test PASS |
| Matrix content | 21 `supported` / 14 `supported-through-bridge` / 1 `unsupported-by-design`; 72/72 `passing`, 0 planned; zero `future_owner` in rows or claims; all 36 manifests `schema_version: 2`; execution kinds 13/4/10/9 |
| `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`, file-size (3011 files, 900-line cap), HIR + driver maintainability guardrails | all PASS |

Every numeric claim in the closeout inventory and validation-evidence block matches what I measured. Historical handoff rewording, stable-claim scoping, and phase/roadmap status links are accurate and appropriately non-overstated; the one unchecked issue item (PR/merge/final review) correctly reflects pre-PR state.

### Findings

**1. HIGH — Three of the seven new trust declarations are not required, and the closeout narrative misstates the merge-lane regression.**

I built a standalone harness replicating `package_entrypoint_from_cargo_layout` + `check_package_project` + `build_cached_package_project` and mutated each declared entry on copies of both fixtures:

- Removing `serde`, `serde_json`, or `thiserror` from `rust-build-scripts` → `SIFR-RUST-TRUST-0001`. Removing `zerocopy` → `SIFR-RUST-TRUST-0001`. **Required.**
- Setting `rust-proc-macros = []` in `bridge_type_roundtrip/sifr.toml:19` → `CHECK: no errors`, `BUILD OK stdout=serde:nested|bytes:6|invalid nested payload`.
- Setting `rust-proc-macros = []` in `crate_backed_view_runtime/sifr.toml:18` → `CHECK: no errors`, `BUILD OK stdout=Ok("bytes=alias+owner;…released=1;active=0")`.

Root cause: `derive_backend_crates` (`crates/sifr_package/src/graph/derive.rs:325-355`) builds `backend_crates` only from direct Cargo resolve edges out of the root package, and `validate_package_dependency_trust` (`crates/sifr_driver/src/build/rust_interop/trust_validation.rs:63-119`) iterates only that set. `serde_derive`, `thiserror-impl`, and `zerocopy-derive` are transitive (pulled in by the `derive` features of `serde`/`zerocopy` and by `thiserror`), so the compiler never demands trust for them.

Consequences:
- `plans/issues/active/rust-interop-runtime-ecosystem-certification.md:1766-1768` states the packages "omitted the locked … `serde_derive`/`thiserror-impl` proc macros" and "`zerocopy-derive` proc macro". Only the build-script omissions were real; the proc-macro half of that regression claim is not reproducible. The following sentence — "The manifests now declare those exact graph entries" — is likewise inaccurate for three of seven.
- The certified fixtures now carry **unrequired trust grants**, i.e. a broader trust surface than the model demands, in the two packages that are the reference examples for the trust contract.
- `_scenario_checks.py:490-497` and `_scenario_zero_copy.py:104-111` now *mandate* those over-declarations, and two of the four new adversarial mutations (`_scenario_checks.py:164-169`, `_scenario_zero_copy.py:186-192`) assert only that the Python checker rejects a missing entry — there is no compiler counterpart, unlike the build-script mutations. The doc's "their validators reject each missing trust family" is true of the validators but does not establish a trust obligation.
- Directly answering the review question: the declarations are **sufficient** but **not necessary**. Fixture READMEs (`bridge_type_roundtrip/README.md:14-16`, `crate_backed_view_runtime/README.md:14-15`) inherit the same overstatement.

This is a certification whose entire premise is exact, non-overstated claims about the trust surface, so I treat it as blocking. Either drop the three proc-macro entries and the two proc-macro validator requirements/mutations and correct the narrative, or — if transitive proc-macro trust is the intended contract — fix `derive_backend_crates`/`validate_package_dependency_trust` to actually require it, which would make the declarations correct and the claim true.

**2. MEDIUM — Cited durable review evidence is untracked.**

`plans/issues/active/rust-interop-runtime-ecosystem-certification.md:1702-1703` links `../../reviews/active/rust-interop-certification-13-review-round-10.md` as the published-head audit that unblocked certification_14. That file exists locally (165 lines) but is untracked (`?? plans/reviews/active/rust-interop-certification-13-review-round-10.md`) and is not in the `origin/main` history. As committed, the closeout would cite a file no one else has.

**3. LOW — Empty placeholder artifact.** `plans/reviews/active/rust-interop-certification-14-review-round-1.md` is untracked and 0 bytes. It should be populated or removed before the PR.

**4. LOW — The diagnostic-preserving assertion fix is correct but incomplete.** The change at `crates/sifr_driver/src/tests/package_rust_interop_build_tests.rs:173-177` is right and matches the doc's scoped claim. The sibling pristine assertion at line 107-110 (`test_build_local_bridge_…`) still discards the diagnostic list, so the next stale-manifest failure in that test will again report only "should pass package checking".

### Not findings
No user-path panic regressions (the diff adds no generated-runtime code; `assert!` changes are test-only). No file-size, maintainability, or lint regressions. Diff hygiene is otherwise clean and confined to the certification surface. Dormant Track B and the excluded dirty paths were not weighed.

Finding 1 leaves an incorrect factual claim in the Track A closeout record and bakes unrequired trust grants into two certified reference fixtures, which is exactly the class of overstatement this closeout exists to prevent.

VERDICT: NOT SATISFIED
