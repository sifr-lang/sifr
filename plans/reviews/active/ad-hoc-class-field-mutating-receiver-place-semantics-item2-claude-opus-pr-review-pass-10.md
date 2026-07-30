# Independent exact-published-head review — Item 2 / PR #3082 (pass 10)

**Reviewed head:** `afc70aa29253501a4cb691c3a97c08730a20914e`
**Base:** `f1c34cf9aaabadda546e670fca190decc580c935` (also the merge-base)
**PR state:** OPEN, not draft, MERGEABLE; `gh pr view 3082 --json headRefOid,baseRefOid` → head `afc70aa29…`, base `f1c34cf9a…` — exact-head identity confirmed.
**Working tree:** `git status --porcelain` empty before and after review. No file modified, no commit, no push, no PR-state change. All probes were written to `/tmp/sifr_probe_rev10`; the base compiler comparison reused the throwaway tree `/private/tmp/sifr_base_wt` (`git rev-parse HEAD` → `f1c34cf9a…`).

Diff scope: `git diff --stat f1c34cf9a..afc70aa29` → 190 files, +6349/−1061.

---

## 1. Disposition of the pass-9 finding and the 0297 gap

### 1.1 Rotate Array corpus migration — CONFIRMED REMEDIATED

Submodule repin is present and points at the reviewed-and-merged corpus head:

```
$ git diff f1c34cf9a..afc70aa29 -- verification/areas/algorithmic_compatibility/corpora/leetcode
-Subproject commit a20d9d5020dae9c19913a598d262ab931924cfe9
+Subproject commit e75af095d1bcf779a631fc1d1ed79ed392bd3ed6

$ git -C verification/areas/algorithmic_compatibility/corpora/leetcode log --oneline -3
e75af09 Merge pull request #41 from sifr-lang/codex/rotate-array-same-call-snapshot
4fdb439 Snapshot rotate length before mutable calls
7772857 Snapshot LRU head before mutable receiver calls (#40)
```

`e75af095` is the merge of PR #41 whose reviewed head `4fdb439` returned SATISFIED at corpus review pass 2 (`plans/reviews/active/…rotate-corpus-pr-41-claude-opus-review-pass-2.md`). The pin is exact.

`0189_rotate_array.sifr:22` now reads `nums_len = len(nums)` before the three `_reverse_range` calls; the only remaining `len(nums)` reads (lines 17, 19–20) are outside any call argument list. Rotation semantics are preserved — `_reverse_range` only swaps in place and never resizes, so the snapshot equals `len(nums)` at each later call site, and the snapshot is taken after the empty guard and after the `while rot >= len(nums)` normalization.

```
$ cargo run -q -p sifr -- run …/corpora/leetcode/src/0189_rotate_array.sifr ; echo $?
0        # asserts arg0 == [5,6,7,1,2,3,4] and [3,99,-1,-100] hold
$ ./target/debug/sifr check …/src/0189_rotate_array.sifr        → exit 0
```
Check, native build (`sifr run` performs a release native build) and run all pass.

### 1.2 0297 same-named nested-helper verifier collision — CONFIRMED REMEDIATED

```
$ ./target/debug/sifr check …/src/0297_serialize_and_deserialize_binary_tree.sifr → exit 0
$ cargo run -q -p sifr -- run …/src/0297_…sifr ; echo $?
0        # zero SIFR-INTERNAL matches in output; only the pre-existing SIFR-TYPE-0901 overflow warning
```

The collision is real and is exactly the shape the fix targets. `sifr emit` on the reduced two-`dfs` program shows two lexically distinct helpers with *different* conventions under one spelling:

```
13:        fn dfs(node: Option<i64>) -> i64 {        # Codec.serialize — no mut argument
26:        fn dfs(values: &mut Vec<String>) -> i64 { # Codec.deserialize — mut argument at index 0
45:        dfs(&mut values)
```

And the table the removed code consulted is genuinely module-global with no scope restore:

- `crates/sifr_lowering/src/lower/typing_and_functions/signatures_and_effects.rs:636` — `register_local_function_signature` inserts every nested helper into `ctx.functions` under its **bare name**;
- `statement_dispatch.rs:198-212` (`predeclare_nested_function_symbols`) re-registers per enclosing body, overwriting;
- no `ctx.functions.remove` / snapshot-restore exists anywhere in nested lowering (`grep -rn "ctx.functions.remove\|saved_functions"` → no hits), so the last-registered `dfs` (deserialize's, mut at index 0) survives to `verify_module_method_calls` (`mod_impl.rs:830-834`), which then flags serialize's `dfs(root)` / `dfs(None)`.

This matches, independently, the clean-predecessor observation recorded in corpus review pass 1 (`SIFR-INTERNAL-0001: internal compiler error: mutable source call 'dfs' has no checked argument place` on `581b363aa`). Lowering itself resolved each `dfs` correctly — the emitted conventions above prove it — so lexical metadata carried from lowering is the authoritative proof and post-hoc re-resolution was the only broken part.

### 1.3 Item 4 — does removing the module-wide re-resolution weaken a required invariant? NO

`method_call_verifier.rs:70-101` now checks only the `anext` special case plus target/expression agreement. The required invariant — *every `mut_borrow` argument at a source call either carries a proven target or produces a user diagnostic* — is enforced where the proof is produced, not in the verifier:

- `method_receiver_places.rs:161-198` (`prove_mutable_arguments`) maps over every argument; for a `mut_borrow` convention it returns `Some(Place)`, `Some(OwnedTemporary)`, or `None` **with a diagnostic on every `None` branch** (`report_immutable_root` → `immutable_parameter_mutation`, line 180-183; `unsupported_mutable_receiver_place`, line 187-194).
- `InvalidPlace` has exactly two variants (`method_receiver_places.rs:14-17`), both handled — the match is exhaustive, so there is no silent `None`.
- Every plain-`Call` place-proof producer routes through `validate_regular_call_arguments`: `regular_calls.rs:180`, `regular_calls.rs:456`, `call_builtins.rs:189`, `async_generator_advances.rs:69`. No other site produces a non-empty `mutable_arg_places` for a `Call`.
- All 203 `mutable_arg_places: Vec::new()` construction sites are compiler-authored calls whose `func` is a builtin or `__sifr_*` intrinsic spelling; none targets a user function with a `mut_borrow` parameter. The one compiler-authored plain `Call` that *does* take a `mut_borrow` argument is `anext`, and that is precisely the case the retained `requires_first_mutable_place` branch still covers (`async_generator_advances.rs:60-82` supplies the signature; verifier line 81-91 asserts the proof).
- The verifier only runs when `ctx.errors.is_empty()` (`mod_impl.rs:821`), so it was never the enforcement path for rejected programs — it is a post-hoc consistency assertion over compiler-authored HIR.

Beyond the nested-helper collision, the removed branch had two further latent false-positive modes it can no longer cause: calls that omit defaulted `mut` parameters (`args.len() < params.len()` → `mutable_arg_places.get(index)` is `None` → spurious violation), and any compiler-authored `Call` whose spelling collides with a user function of the same name. **The removal is a strict improvement, not a weakening.** Recorded as non-blocking observation N1 below.

I probed for the adversarial direction — a leaked nested signature causing *silent* lost mutation — and found none:

```
# p4.sifr: top-level helper(mut values) + earlier nested shared-convention helper of the same name
HEAD: 3 / [4, 5, 6, 99] / 4     BASE: 3 / [4, 5, 6, 99] / 4     (identical, mutation persists)
```

### 1.4 New lowering regression test — SENSITIVE and on-shape

`crates/sifr_lowering/src/lower/nested_function_tests.rs:112-121`, `same_named_nested_helpers_keep_lexical_mutable_call_metadata`, lowers a two-method class with same-named nested `dfs` helpers where the second's `list[str]` parameter is inferred `mut` from `values.pop(0)` and the first's is not. `sifr emit` on the identical source (§1.2) confirms the two conventions actually diverge, so `ctx.functions["dfs"]` necessarily holds one of them for both call sites — the exact precondition of the false positive. The assertion is `result.is_ok()` with a message naming the invariant, which fails under the pre-fix code path.

### 1.5 Complete corpus classification: 407/411 with four base-identical failures — CONFIRMED

Per instruction I did not rerun all 411 fixtures; I spot-verified the classification on both compilers using the runner's own invocation (`sifr check <path>` with `SIFR_ARTIFACT_CACHE=1`, per `runner.py:556-568`):

| fixture | head `afc70aa29` | base `f1c34cf9a` |
|---|---|---|
| `0002_add_two_numbers` | exit 1 `SIFR-TYPE-0002` | exit 1 `SIFR-TYPE-0002` |
| `0036_valid_sudoku` | exit 1 `SIFR-TYPE-0005` | exit 1 `SIFR-TYPE-0005` |
| `0086_partition_list` | exit 1 `SIFR-TYPE-0002` | exit 1 `SIFR-TYPE-0002` |
| `0377_combination_sum_iv` | exit 1 `SIFR-TYPE-0004` | exit 1 `SIFR-TYPE-0004` |
| `0146_lru_cache` | exit 0 | exit 0 |
| `0189_rotate_array` | exit 0 | exit 0 |
| `0297_serialize_and_deserialize…` | exit 0 | exit 0 |
| `0143_reorder_list`, `0778_swim_in_rising_water` | exit 0 | exit 0 |

The four remaining failures reproduce **identically, code-for-code**, on the untouched base compiler, so `411 − 4 = 407` is accurate, and the "four base-identical failures" characterization holds. `581b363aa`'s additional fifth failure (`0297`, `SIFR-INTERNAL-0001`) is the one this head fixes — consistent with the `406/411` figure recorded for the clean predecessor. The migrated `0189`/`0146` fixtures also pass on base, so the corpus repin is not head-coupled.

Note: pass 9's incidental mention of `0143_reorder_list` (`SIFR-OWN-0001`) and `0778_swim_in_rising_water` (`SIFR-OWN-0004`) as base-identical artifacts does not reproduce — both exit 0 on both compilers under the runner's invocation. That claim was scoped to `SIFR-OWN-*`/`SIFR-PROTO-*` grepping and is not load-bearing for any current count; the tracking doc and PR body do not repeat it.

---

## 2. Re-audit of the full Item 2 implementation

All results are from the head compiler unless marked BASE.

**Canonical place modeling / checking / emission / overlap.** `method_receiver_places.rs` proves a `BindingId` root plus nominal field projections (`extract_place:426-457`); `places_overlap:353-357` is a symmetric prefix rule. Overlap coverage is complete over the four pairings — mutable receiver vs every argument, specialized indexed base vs every argument, shared/owned receiver vs every mutable argument place, and mutable argument vs every other argument with double-report suppression (`validate_call_overlaps:200-270`).

- Nested / grandparent-declared fields emit against real storage: `class A / B(A) / C(B)` with `self.a_items.append(1)` … `self.c_items.append(3)` → `[1][2][3]` and `[1] [2] [3]`.
- Field-as-mutable-argument to a plain function, plus a shared-protocol delegation, mutate the original storage: `[5, 6, 7]`, total `19`, `[5, 6] 11` — no clone, no lost mutation.
- Overlap is caught and the documented snapshot remedy works: `self.node.push(self.node.size())` → `SIFR-OWN-0002` at the argument range; after hoisting, `1 2 [1, 2] n` / `[0, 3]`.
- `place_emitter.rs` is fail-closed by construction: `emit_checked_place_projection:85-93` returns `None` on any field-identity mismatch and every mutable caller propagates `None` rather than falling back to value/clone lowering (`:105-142`, `:159-200`). Shared receivers use the separate structural borrow path `emit_shared_receiver_path:31-40`.
- No panics in the new code: `grep -n "unwrap()\|expect(\|panic!\|unreachable!\|\[0\]"` over `place_emitter.rs` and `method_receiver_places.rs` → zero hits. `scripts/check_codegen_rawcode_gate.sh` → `[rawcode-gate] PASS`.

**Constructor materialization and repeated fields.** Source order is preserved across the materialization boundary and repeated assignment after complete storage works: `Child(Base)` with `super().__init__()`, three initializers, three mutations, then `self.b = [0]; self.b.append(9)` → `[4][0, 9]["init", "done"]`. The gap check (`class_semantics.rs:96-166`) seeds `initialized` from same-named parameters, matching the language's implicit param→field initialization, and `explicit_initializers` blocks a repeated field from re-satisfying the requirement. A field with no same-named parameter read before assignment is rejected with a source-accurate span:

```
error[SIFR-OWN-0014]: … --> p12.sifr:4:9   print(self.x)
  = help: initialize every declared field and inherited storage before the first statement that reads or mutates self
```

This area is a clear **improvement over base**, which leaked raw rustc for both shapes:
```
BASE p11.sifr → error[E0424]: expected value, found module `self`  (SIFR-BUILD-0005)
BASE p5.sifr  → error[E0062]: field `b` specified more than once; 4× E0424  (SIFR-BUILD-0005)
HEAD p11.sifr → 4 / 8 / 11        HEAD p5.sifr → [4][0, 9]["init", "done"]
```

**Receiver convention and protocol contracts.** `PROTO-0005` fires on structural conformance where a class infers `MutableBorrow` against a shared-receiver protocol method, and the documented remedy (`def push(mut self, …)` on the protocol) restores acceptance with correct bridging in both directions — verified end-to-end (`[5, 6, 7] / 19 / [5, 6] 11`). This eager structural check matches the plan (`plans/issues/…:190-201`) and the shipped fail fixture `protocol_receiver_mutability_mismatch.sifr`. `protocol_receiver_conformance_controls.sifr` covers both a shared impl behind a mutable protocol and a mutable impl behind it.

**Optimizer protected roots.** Checked-place emission records non-`self` roots into `protected_mutable_place_roots` (`place_emitter.rs:47`, `:81`), consumed by `remove_unneeded_mutability_in_items` (`entrypoints.rs:159`), test-module assembly (`lib_modules_and_codegen.rs:630`), and except-handler binding mutability (`try_handlers.rs:170`). `COMPILER_GENERATED_MUTATING_METHODS` is documented as the *fallback* for synthesized locals without HIR provenance, not the primary mechanism; a miss there strips `mut` and fails loudly at rustc rather than silently.

**Diagnostics.** `SIFR-OWN-0014`, `SIFR-PROTO-0005`, `SIFR-PROTO-0006` are registered, documented (`docs/errors/*.mdx`, `internal_docs/diagnostic_codes.md`), listed in `docs/docs.json`, and backed by three new compact baselines. `SIFR-OWN-0002` docs were extended with the same-call rule and the snapshot remediation. `python3 scripts/check_docs_error_code_links.py` → `Docs error-code link guardrail passed.`

**Corpus migrations and stdlib.** `stdlib/sifr/heapq.sifr:193,239` hoist `len(heap)` into `heap_len` before `_sift_down{,_max}`; `_sift_down` never resizes so the snapshot is invariant. All three heap demos pass: `heapq parity demo: pass`, `[8, 7, 4, 2, 1, 1] / [7, 6, 4]`, `heap_option_drain: ok`.

**Test sensitivity.** The seven new pass fixtures all run clean (exit 0 each) and assert behavior rather than compilation — 4/15/6/8/9/3 asserts respectively, so a swallowed mutation would fail them. The e2e fail suite (562 fixtures, including ~24 new `expect-error` fixtures with column assertions) passes.

**Validation reruns at this exact head.**

| check | result | matches claimed evidence |
|---|---|---|
| `cargo test -q -p sifr_lowering` | 923 passed, 0 failed, 1 ignored | ✅ |
| `cargo test -q -p sifr_codegen` | 941 passed, 0 failed | ✅ |
| `cargo test -q -p sifr -- --skip test_e2e_pass` | 114+12+36+6+1+3 passed, 0 failed | ✅ |
| `cargo fmt --check` | exit 0 | ✅ |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 | ✅ |
| `python3 scripts/check_hir_maintainability_guardrails.py` | PASS | ✅ |
| `python3 scripts/check_file_size_guardrails.py` | PASS (3054 files, limit 900) | ✅ |
| `python3 scripts/check_docs_error_code_links.py` | PASS | ✅ |
| `python3 scripts/check_submodule_ownership.py` | PASS | ✅ |
| `scripts/check_codegen_rawcode_gate.sh` | PASS | ✅ |
| diagnostics verification area | variants=183, failures=0 | ✅ |

Largest touched hand-maintained files: `performance_codegen_tests.rs` 900, `registry.rs` 894, `hir_nodes.rs` 892, `class_body_lowering.rs` 889, `method_receiver_places.rs` 876, `mutability_and_clone_rewrites.rs` 874 — all at or under the 900 cap, and the guardrail passes. `class_field_emitter.rs` was decomposed by responsibility (`class_struct_field_rust_type` extracted and reused by the constructor emitter), which is the right split.

**Tracking accuracy.** `plans/issues/active/ad-hoc-class-field-mutating-receiver-place-semantics.md` records pass 9 as `NOT SATISFIED`, both corpus PR #41 passes with the correct verdicts and merge SHA `e75af095`, the `406/411` clean-predecessor vs `407/411` candidate distinction, and the base-identity of the four remaining failures — all of which I independently reproduced. `internal_docs/architecture.md:446-505` accurately describes canonical receiver conventions, constructor materialization and re-rooting, place proof rules, the audited indexed-storage exception, the same-call prefix-overlap rule, and checked-place emission including the optimizer protection and its method-name fallback scope.

**PR body.** Every claim I sampled checks out: head identity, 923+1 ignored, 941, `407/411` with four base-identical failures, `0189` check/build/run, `0297` fixed with the predecessor reproducing the diagnostic, rustfmt/clippy/HIR/file-size/doc-link/diff guardrails, and the lexical-verifier summary bullet. The Python-lane budget history is stated with the actual numbers (`635.263s > 600s` first attempt, `493.893s` on the unchanged-head retry) and explicitly says no waiver or budget changed — no misattribution remains.

---

## 3. Actionable findings

**None.**

## 4. Non-blocking observations (no action required for approval)

- **N1 — the plain-call argument backstop is now absent rather than repaired.** The verifier no longer cross-checks plain-call proofs against any signature. That is correct today (§1.3), but it means a *future* codegen-facing plain-call construction path that forgot to populate `mutable_arg_places` for a `mut_borrow` user-function parameter would surface at rustc (`SIFR-BUILD-0005`) instead of `SIFR-INTERNAL-0001`. Restoring the backstop soundly would require keying callee resolution on the callee's `BindingId`/scope rather than its spelling. Worth a follow-up issue, not a change to this PR.
- **N2 — the underlying module-wide nested-name collision is untouched (pre-existing, base-identical).** A nested helper permanently overwrites a same-named top-level function's entry in `ctx.functions`, and the reverse direction leaks raw rustc identically on both compilers: `p_leak.sifr` → `E0308 … found reference &Vec<_> … closure parameter defined here` on HEAD **and** BASE. Not a regression from this PR; the fix correctly stops depending on the broken table rather than papering over it. Recommend tracking as a separate name-resolution issue.
- **N3 — documented intentional narrowings reject programs base compiled.** Verified live: `bucket = self.tags.get(key); … bucket.append(v)` → `SIFR-OWN-0014` (BASE printed `[1, 3] / {"a": [1]}`); `self.node.push(self.node.size())` → `SIFR-OWN-0002`; a class matching a shared-receiver `Protocol` → `SIFR-PROTO-0005` even when never used at that protocol. All three are specified in the plan (`:325`, `:341`, `:356`, `:368`, `:703`, `:850`, `:190-201`), carry fail fixtures, and have documented remedies in the error pages. Recorded as approved narrowing, consistent with pass 9's read.
- **N4 — every constructor field assignment now emits a `let __sifr_field_init_N: Ty = …;` temporary, unconditionally.** `class_method_emitter.rs:369-376`. Diffing head vs base emit for `demos/generic_classes`: `Self { first, second }` → three statements. The temporaries are load-bearing whenever ≥2 initializers have side-effecting values (`constructor_instance_fields` emits in *declaration* order, not assignment order), so this is a correctness-driven cost; a 0/1-initializer or pure-value fast path would recover idiomaticity. Clippy on the generated project shows no new lint class from them (only 2 pre-existing `this operation has no effect` warnings from the integer model; base could not even compile the same probe). Not gated by any lane.
- **N5 — `demos/*/emitted.rs` documentation artifacts are stale.** Already stale at base (`return Self { first: first, second: second };` vs base's actual `Self { first, second }`, plus a missing `Display` impl), and no lane consumes them (`grep -rn "emitted.rs" scripts verification crates/sifr/tests` → only the file-size guardrail's exclusion list). Pre-existing, not caused by this PR.
- **N6 — one claimed lane not independently rerun.** Per instruction I did not rerun the complete 411-fixture corpus, the create-PR profile, or the full E2E suite. I additionally started the `generated_code_quality` area but stopped it after ~20 min without a result; I rely on the authoritative `5/5` evidence and on my own targeted substitute (clippy over a generated project, plus the raw-code gate) for the concern in N4.

## 5. Checks that found nothing

Ownership unsoundness and aliasing (no lost mutation across nested, inherited, grandparent, generic, borrowed, owned, protocol, and field-as-mutable-argument shapes; the nested-shadow adversarial probe is base-identical and correct); silent clone (shared receiver borrows storage, `emit_checked_place` returns `None` rather than cloning); raw-rustc leakage (every accepted probe produced a clean release binary; two base leakage cases are *fixed* here); user-triggerable panics (no `unwrap`/`expect`/indexing in the new modules; `InvalidPlace` match exhaustive with a diagnostic on every failure branch); diagnostic stability (183 diagnostics-area variants, 0 failures; spans reproduced exactly across repeated runs); file responsibility and guardrail limits; tracking-doc and PR-body accuracy.

---

**Severity-ranked actionable findings: 0.** Pass 9's single HIGH finding is fully remediated with the exact reviewed-and-merged corpus pin; the second defect the complete runner exposed is fixed at the root (lowering-time lexical proof) rather than suppressed, is covered by a sensitive regression test, and the removal of the module-wide re-resolution does not weaken the fail-closed mutable-argument invariant.

VERDICT: SATISFIED
