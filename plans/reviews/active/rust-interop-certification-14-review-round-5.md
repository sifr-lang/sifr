## Independent Final Review — Round 5, Rust Interop `certification_14` / Track A closeout

**Head reviewed:** `8f16565969641c464f1107fe25b15820323d1b9a` (merge of `origin/main` `ca7731aa8`-descendant into `5c94b0695` "Certify Rust interop runtime ecosystem"). Read-only: no file edited, no lane launched. `git status` is byte-identical to the session snapshot (the leetcode submodule marker is only `.DS_Store` noise inside the submodule, unchanged by me). Excluded per instruction: `editor_integrations`, the leetcode corpus, `.cert5probe/`, `.claude/`, stray `*.webp`, untracked `plans/phases/43_interoperability.md` (a draft Phase 43 doc that contradicts nothing here), and dormant Track B.

### Merge-integration verdict: no regression, no file overlap

`git diff 5c94b0695 8f1656596` touches only `sifr_lowering` dict/`defaultdict` inference plus three planning docs (algorithmic-corpus waves 3/4, Phase 40 pre-GA closeout PR #3073). Zero overlap with any Rust-interop file, and neither incoming doc mentions Rust-interop certification status. Because the incoming lowering work changes empty-dict/`defaultdict` inference and the bridge fixture roundtrips nested dictionaries, I re-ran the affected executable evidence rather than reasoning about it:

| Gate rerun at this head | Result |
|---|---|
| `test_build_bridge_type_matrix_positive_cargo_probe`, `test_build_zero_copy_crate_backed_view_lifecycle` (`--ignored`) | 2 passed, 0 failed (83.47 s) |
| `cargo test -p sifr_driver --lib` | **450 passed, 0 failed, 65 ignored** — exactly the recorded figure |
| Full `rust_interop` area | `variants=10, failures=0, blocking_failures=0, non_blocking_failures=0` |
| Fixture-matrix inventory | `fixtures=36 diagnostics=10 crates=44 package_examples=61 scenario_examples=18` |
| Self-tests | fixture **233**, compatibility 7, tiers 6, stable claims **33**, stale drafts 20 |
| Matrix recomputed from JSON | 21 `supported` / 14 `supported-through-bridge` / 1 `unsupported-by-design`; **72/72 `passing`, 0 planned**; execution 13 / 4 / 10 / 9; 36/36 manifests `schema_version: 2`; 36 claims; no `future_owner` field anywhere |
| Resource backstop | `PASS (surfaces=1, future_runtime_rows=0)`; completed-matrix `--self-test` PASS |
| `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`, file-size (3019 files/900), lowering maintainability, `git diff --check` | all PASS |

**Four build-script grants — re-proved exact.** Of `bridge_type_roundtrip`'s five direct deps, exactly `serde 1.0.228`, `serde_json 1.0.149`, `thiserror 2.0.18` ship `build.rs`; `bytes 1.11.1` and `indexmap 2.14.0` do not. Of `crate_backed_view_runtime`'s direct deps, exactly `zerocopy 0.8.48` does (`bytemuck 1.25.2`, `memmap2 0.9.11`, `bytes`, and the `sifr_runtime` path crate have none). The declared sets are byte-exact — necessary and minimal — and no `rust-proc-macros` key exists in either fixture, so round 1's blocking finding stays closed. The four new mutations (`_scenario_checks.py:163-183`, `_scenario_zero_copy.py:178-184`) are non-vacuous: the bridge helper `_require_trust_targets` (`_scenario_checks.py:805-816`) emits the exact `missing <crate>` string asserted, and the zero-copy `_require_trust` uses list equality, so it also rejects over-declaration. 229 → 233 is exact.

**Tracking, stable claims, re-homing.** Roadmap row 39, Phase 39 (`:439-446`), Phase 40 (`:54-58`, `:70-80`), and the status table agree: certifications 0–13 merged, 14 in progress, Track B dormant. Round 3's re-homing sentence (`certification.md:1790-1794`) still maps conjunct-for-conjunct onto `adhoc_performance_budget_host_variance.md`; `certification_7`'s checklist `:811` is `[x]`; the only remaining `- [ ]` is `:1738` (lanes/PR/merge identity), correct while in progress. All cited review artifacts, including `rust-interop-certification-13-review-round-10.md`, are now **committed** in `5c94b0695`, closing round 2's tracking finding. Diff adds no generated-runtime code; the only `assert!` edits are the five pristine assertions that now interpolate `{pristine_errors:#?}` — no user-path panic surface.

### Findings

**1. MEDIUM — the canonical compatibility matrix still asserts that three now-certified surfaces "remain future-owned", contradicting the closeout's central claim.**
`verification/areas/rust_interop/data/rust_interop_compatibility_matrix.json`:

- `:135` (`opaque_resource_core`): "…ecosystem resources **remain future-owned by** `opaque_resource_matrix`." That row is `supported-through-bridge` / `runtime-observed` with both directions passing (`certification_5`, PR #3042).
- `:231` (`async_runtime_core`): "runtime cancellation/drop and reqwest loopback evidence **remain future-owned**." Certified by `certification_4` (PR #3036).
- `:296` (`callback_subscription_core`): "Runtime subscription lifecycle and ecosystem behavior **remain future-owned**." Certified by `certification_6` (PR #3046).

The same file defines `future-owned-by-separate-phase` at `:9` as "at least one required evidence direction is not passing", so within this file's own vocabulary all three notes are false. They directly contradict the closeout inventory (`certification.md:1751-1753`, "the declared `future-owned-by-separate-phase` category is intentionally unused") and Phase 39 `:439-441` ("No current Track A runtime/ecosystem row remains `future-owned-by-separate-phase`"). This is the structured artifact Phase 40 digests into advertising, and the closeout item at `certification.md:1728-1729` owns exactly this cleanup — it removed the stale `future_owner` *fields* but not the stale prose in the same rows. No gate catches it: the stale-draft scan only covers `docs`/`internal_docs`/`plans` for stale syntax and panic surface. Fix: restate each as scope delegation to the certified row (e.g. "ecosystem resources are certified by `opaque_resource_matrix`"), preserving the narrower contract-only scope of the core rows.

**2. LOW — durable internal architecture doc repeats the same stale deferral.**
`internal_docs/sifr_sysroot_and_stdlib_architecture.md:914-916`: "…the broad `opaque_resource_matrix` row for package ecosystem resources **remains future-owned by separate certification work**." Introduced by PR #2851, before `certification_5`; the preceding sentence (`:908`) uses "future-owned" in its normative matrix sense, so this reads as a category claim. It is the architecture home of the resource backstop this closeout re-verified, and it now contradicts `future_runtime_rows=0`. `:213-215` in the same file is fine — it assigns *issue* ownership, not a category.

### Not findings

- The closeout's create-PR evidence (`certification.md:1814-1821`) was measured pre-merge and does not name a base commit, unlike `certification_5`'s record. It makes no claim about this head, and checklist item `:1738` is explicitly still open, so the record is honest rather than contradictory. Recorded for awareness only; per instruction I ran no lane.
- `plans/reviews/active/rust-interop-certification-14-review-round-5.md` is 0 bytes — this round's own artifact, unwritten because the check is read-only. Needs content (or removal) before the PR, same as the round-1/3/4 precedent.
- Round 2's transitive `ring`/`libsqlite3-sys` grants remain informational, on round 3's reasoning (documented contract is not direct-scoped; paired `native-links` entries are transitive by design). Unrelated to findings 1–2, which are stale *under*-claims rather than trust grants.
- `cargo clippy --workspace --all-targets` still fails in `sifr_codegen` (pre-existing, not the documented gate, untouched by this diff).

Both findings are one-line corrections in files this closeout is responsible for aligning, but finding 1 leaves the authoritative structured matrix asserting deferral for three surfaces the same closeout certifies as fully evidenced — the exact class of record contradiction this item exists to eliminate.

VERDICT: NOT SATISFIED
