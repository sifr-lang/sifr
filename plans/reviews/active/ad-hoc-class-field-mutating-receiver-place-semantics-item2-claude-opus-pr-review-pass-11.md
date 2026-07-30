# Independent final merge-evidence review — Item 2 / PR #3082 (pass 11)

**Reviewed head:** `afc70aa29253501a4cb691c3a97c08730a20914e`
**Base:** `f1c34cf9aaabadda546e670fca190decc580c935` (confirmed to also be the merge-base: `git merge-base HEAD f1c34cf9a` → `f1c34cf9a…`)
**Diff scope:** 190 files, +6349/−1061.
**PR identity:** `gh pr view 3082 --json headRefOid,baseRefOid,state,isDraft,mergeable` → head `afc70aa29…`, base `f1c34cf9a…`, `OPEN`, `isDraft:false`, **`mergeable:"CONFLICTING"`**.
**Working tree:** `git status --porcelain` empty before and after. No file modified, no commit, no push, no PR-state change. Only read-only probes (`git merge-tree`, `git show`, JSON reads of existing evidence artifacts).

---

## 1. Performance adjudication — host variance, CONFIRMED; not an Item 2 regression

The three misses are **not** attributable to this PR, and the evidence establishing that is stronger than the summary claims.

**1.1 The performance surface is byte-identical to base.**

```
$ git diff --stat f1c34cf9a afc70aa29 -- verification/areas/performance/ scripts/
(no output)
```
No budget, baseline, threshold, sample-count policy, waiver, or runner changed.

**1.2 Same compiler binary produced both the passing and the failing samples — decisive.** Hashing `metadata.compiler_fingerprint` from the checked-in evidence artifacts:

| evidence file | fingerprint sha256[:16] | `cargo_lock_sha256` |
|---|---|---|
| `bench-1785413115-79173.json` (earlier isolated) | `523a217b8328a642` | `602c5cc8…` |
| `bench-1785413202-86195.json` (earlier isolated) | `523a217b8328a642` | `602c5cc8…` |
| `bench-1785443751-89457.json` (final merge run) | `523a217b8328a642` | `602c5cc8…` |
| `bench-1785443973-1317.json` (retry) | `523a217b8328a642` | `602c5cc8…` |

Identical fingerprint ⇒ identical measured artifact. Yet `diagnostic-non-regression-002-json-diagnostic-schema` reads **1590.210 → 1363.124 → 1359.825 → 1317.663 ms** across those four runs — a **272 ms spread on unchanged code**, versus a 23.9 ms overshoot. The misses sit far inside the demonstrated same-code variance band.

**1.3 The failing metric is fixed-overhead-dominated, so the misses are not workload-proportional.** From the final merge and retry evidence:

| benchmark | baseline median | threshold | merge run | retry |
|---|---|---|---|---|
| `check-project-004-project-graph` | 1234.113 | 1357.524 | 1369.857 | 1359.258 |
| `check-single-file-001-arithmetic` | 1212.854 | 1334.139 | 1369.644 | 1358.635 |
| `diagnostic-non-regression-002-json-diagnostic-schema` | 1214.504 | 1335.954 | 1363.124 | 1359.825 |

Three structurally unrelated workloads — a multi-module project graph check, a 10-line single-file check, and a JSON-diagnostic check — all converge to ≈1359 ms in the retry. That means ~1355 ms is workload-independent fixed cost; the elevation is a uniform host-level shift, not compile work.

**1.4 The most-overshooting fixture cannot execute any Item 2 code path.** `crates/sifr/tests/e2e/pass/arithmetic.sifr` (the +24.5 ms case) is:

```python
def add(a: int, b: int) -> int: return a + b
def multiply(a: int, b: int) -> int: return a * b
def main(): ...
```
Zero classes, zero methods, zero receivers, zero mutable arguments. Item 2 adds receiver-place proof and checked-place emission on class/protocol mutable-receiver and `mut`-argument paths; on this fixture that code is unreachable. A genuine Item 2 cost would have to be workload-proportional and would show *least* here — instead it shows *most*.

**1.5 The failing host was demonstrably loaded.** `merge.latest.json` report: `wall_time=971.41s cpu=911.51s (warm_target<=15m budget_ok=no)`, `advisories=warm wall-time budget exceeded`, with `python_interop=630650ms` (vs 493893ms for the same lane in the create-PR run on the same head) — a 27% inflation on an unrelated lane.

**1.6 Everything else in both runs passed.** `grep "sifr-lane-step"` on the merge log: every lane `status=pass` except `performance_budget_checks`. In the create-PR log all 20 lane steps pass, all within budget, `e2e report_signature=7c39b8c1dd4fec7c`, `hardening=variants=6 failures=0 blocking_failures=0`.

**Adjudication: host variance. Sufficiently established.** One precision note, non-actionable: the "earlier official isolated exact-head samples passed all three thresholds" claim is composed from two runs — `bench-1785413115` recorded the JSON-diagnostic case at **1590.210 ms** (a miss) and only the separate `bench-1785413202` run reached 1317.663 ms. This does not weaken the conclusion; it strengthens it, since that 1590 ms reading came from the same fingerprinted binary. But no single isolated sample cleared all three simultaneously, and the summary reads as if one did.

---

## 2. Actionable findings

### F1 — HIGH: the reviewed head is unmergeable, and the conflicts land in the exact codegen path Item 2 rewrites, so the merge-gate evidence does not cover the tree that would land

`main` advanced 20 commits past the reviewed base since pass 10 (which recorded `MERGEABLE`). GitHub now reports `mergeable:"CONFLICTING"`. Read-only confirmation:

```
$ git merge-tree --write-tree --name-only origin/main afc70aa29253501a4cb691c3a97c08730a20914e
3182a88e1cf9dc94d0d4d44b6c2cae9b21860e19
crates/sifr_codegen/src/intrinsic_method_emitters/collection_methods.rs
crates/sifr_codegen/src/stmt_support_emitter/print_calls.rs
crates/sifr_codegen/src/stmt_support_emitter/stmt_expr_method_and_question_mark.rs
CONFLICT (content): Merge conflict in …/collection_methods.rs
CONFLICT (content): Merge conflict in …/print_calls.rs
CONFLICT (content): Merge conflict in …/stmt_expr_method_and_question_mark.rs
```

Six conflict hunks total (4 in `collection_methods.rs`, 1 each in the other two). All three files were rewritten by this PR (+59, +60, +79 lines respectively). The sole upstream cause is `origin/main` tip `441f667f0` — "Fix order-independent defaultdict declaration inference (#3081)":

```
$ git log --oneline f1c34cf9a..origin/main -- <each of the three files>
441f667f0 Fix order-independent defaultdict declaration inference (#3081)
```

These are **semantic** conflicts, not textual noise. The two branches independently redesigned the same mutable-bucket lowering function, and each side's edit is the safety gate of its own change:

```rust
    pub(crate) fn try_lower_defaultdict_index_method_call_expr(
        &mut self, object: &HirExpr, method: &str, args: &[HirExpr],
// <<<<<<< origin/main
        method_return_ty: &Type,
// =======
        places: MethodCallPlaces<'_>,
// >>>>>>> afc70aa29…
    ) -> Option<crate::RustExpr> {
        …
// <<<<<<< origin/main
        let (alias_name, key_ty, value_ty) = registry_defaultdict_alias_parts(base_object.ty())?;
        if !methods::is_in_place_collection_method(value_ty, method) { return None; }
        let lowered_object = self.try_lower_registry_expr_strict(base_object)?;
// =======
        let (alias_name, key_ty, _) = registry_defaultdict_alias_parts(base_object.ty())?;
        let MutableReceiverTarget::SpecializedIndexedStorage(base_place) = places.receiver_target?
        else { return None; };
        let lowered_object = self.emit_checked_place(base_object, base_place)?;
// >>>>>>> afc70aa29…
```

Upstream gates on `is_in_place_collection_method`; the head gates on the proven `SpecializedIndexedStorage` place and routes through `emit_checked_place`. `print_calls.rs` is the same shape: upstream changed `try_lower_registry_method_call_expr(...)` to `...?` inside the `needs_self_field_clone_suppression` block that this PR **deletes** outright.

Why this blocks merge approval rather than being a routine rebase chore:

- The three conflicted files sit on the mutable-receiver / defaultdict-bucket-mutation lowering path — precisely the ownership-soundness surface Item 2 introduces. A resolution that keeps the upstream form drops the checked-place requirement (silent-clone / lost-mutation exposure); one that keeps only the head form drops the upstream in-place-mutator gate and the `?` propagation from #3081.
- The entire approval case rests on **exact-head** evidence (`--profile create-pr` exit 0 at `afc70aa29`; the functional lanes of the merge profile at `afc70aa29`; 923/941 unit counts; corpus 407/411). None of that evidence applies to a tree containing a hand-authored reconciliation of two competing designs in `sifr_codegen`. The merged result is, by construction, a tree no gate has ever run against.

**Required before merge:** rebase/merge `origin/main` (`441f667f0`) into the branch, resolve the six hunks so that both the upstream `is_in_place_collection_method` gate + `?` propagation *and* the head's `MethodCallPlaces` / `emit_checked_place` fail-closed requirement survive, then re-run `scripts/run_all_tests.sh --profile create-pr` and the merge gate on the new head — and re-review, since the resolution is new unreviewed logic in the audited path.

### F2 — LOW: the tracking ledger's lowering test count is stale by exactly the test the final commit added

`plans/issues/active/ad-hoc-class-field-mutating-receiver-place-semantics.md:786-789` states:

> full lowering tests pass (`922 passed`, `1 ignored`)

The actual count at this head is **923 passed / 1 ignored** (stated in the merge evidence and independently measured by pass 10). The final commit `afc70aa29` added `same_named_nested_helpers_keep_lexical_mutable_call_metadata` (`crates/sifr_lowering/src/lower/nested_function_tests.rs`, +11) and updated the doc's narrative sections but not this count:

```
$ git show --stat afc70aa29
 .../src/lower/nested_function_tests.rs   | 11 +++
 ...class-field-mutating-receiver-place-semantics.md | 45 ++++++++++++
```

The ledger therefore under-reports the head it describes by the one test that proves the pass-9 verifier remediation. Fix the figure to `923 passed`, `1 ignored`.

---

## 3. Areas re-audited that found nothing new

- **Corpus repin exactness.** `git submodule status` → `e75af095d1bcf779a631fc1d1ed79ed392bd3ed6 verification/areas/algorithmic_compatibility/corpora/leetcode`, matching the reviewed-and-merged LeetCode PR #41 SHA exactly; the diff moves it from `a20d9d5020…`.
- **Create-PR lane completeness at exact head.** All 20 lane steps `status=pass`, each inside its blocking budget; `e2e_pass_suite=407602ms/budget=600000ms/pass` with `report_signature=7c39b8c1dd4fec7c`; `hardening=variants=6 failures=0`.
- **Merge-profile functional lanes at exact head.** coverage matrix, core guardrails, diagnostic rules, CPython differential, Python interop, Rust interop, frontend guardrails, developer tooling (`variants=32, failures=0`) — all `status=pass`; the only failure in the profile is the timing subset adjudicated in §1.
- **Prior-pass conclusions I re-derived rather than accepted:** merge-base identity, perf-surface immutability, submodule pin, arithmetic-fixture reachability, benchmark medians and fingerprints from the raw evidence JSON. I did not re-run the complete merge profile, the complete corpus, or another representative performance sample, per instruction.

Pass 10's substantive implementation verdict is not contradicted by anything I found: the timing misses are host variance, and no implementation defect surfaced. But pass 10's merge precondition — `MERGEABLE` — has since become false, and the conflicts are in the audited codegen path, so its evidence no longer establishes that the mergeable tree is validated.

---

**Severity-ranked actionable findings: 2 (1 HIGH, 1 LOW).**

VERDICT: NOT SATISFIED
