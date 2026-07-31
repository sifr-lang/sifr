## Final Review — Sifr PR #3092 (Pass 3, terminal, read-only, exact head)

**Head reviewed:** `fb37126923131b51070548b0c6de05ea2e36271c` — matches the stated exact pushed head. Base `44ab8ad38`. Working tree carries only the two untracked review-slot files; no tracked file differs from the committed head.

### Methodology

Read-only only. No files were modified, no git or GitHub state changed, and no broad suite (create-pr, default, corpus, full workspace, e2e pass) was started or awaited.

- Confirmed head identity and that the committed range is exactly two commits (`ac31b0908` implementation + fixtures + manifest, `fb3712692` tests + pass-1 artifact).
- Read the full committed diff (`git diff 44ab8ad38..fb3712692`, 505 insertions / 219 deletions across 7 files) and confirmed `fb3712692` itself adds only 68 lines of lowering tests plus the 53-line pass-1 artifact — implementation, fixtures, and manifest are byte-identical to `ac31b0908`, so pass-1's semantic analysis carries forward unchanged.
- Re-read the semantic delta at head (`method_receiver_places/footprint.rs:53-95`): the `methods` guard, the `Callable | AsyncCallable` field predicate, the `resolve_field_identity` narrowing, and the non-static-base fallback.
- Verified the shared-engine claim structurally: `expression_overlaps` has exactly one definition (`footprint.rs:15`) and four call sites (`method_receiver_places.rs:219, 227, 240, 270`) covering receiver, specialized indexed storage, shared-receiver-vs-mut-arg, and mut-arg-vs-arg.
- Verified the two new tests against the guards they lock, including that `range_for` (`method_receiver_analysis_tests.rs:41`) takes the first occurrence and that both needles (`self.run(2)`, `self.pick().callback(2)`) are unique in their sources.
- One narrowly filtered run at exact head: `cargo test -p sifr_lowering --lib callable_field` → **7 passed, 0 failed, 939 filtered out** (946 total, consistent with pass-2's 945 passed / 1 ignored).
- Read both prior artifacts; confirmed pass-2's mutation results are consistent with the code at head.
- File-size check by inspection: `method_receiver_analysis_tests.rs` 846, `method_receiver_places.rs` 684, `footprint.rs` 276 — all under the 900-line cap.

### Pass-1 observations

**Observation 1 (`footprint.rs:64-66`, methods guard untested) — CLOSED.** `actual_method_shadowing_callable_field_stays_conservative` (`method_receiver_analysis_tests.rs:329`) constructs `Child(Base)` with a `run: Callable[[int], int]` field shadowing inherited method `Base.run`, and asserts `SIFR-OWN-0002` at the exact range of `self.run(2)`. Pass-2 mutation-verified that deleting the guard fails this test and only this test out of 946. The test also confirms the guard consults the resolved parent chain, not just own methods — otherwise the inherited-`run` case would narrow and the test would not pass.

**Observation 2 (`footprint.rs:89-91`, non-static-base fallback untested) — CLOSED.** `callable_field_on_dynamic_base_keeps_conservative_object_footprint` (`method_receiver_analysis_tests.rs:365`) exercises `self.update(self.pick().callback(2))` and asserts `SIFR-OWN-0002` at the exact range of `self.pick().callback(2)`. Pass-2 mutation-verified this test fails uniquely when the fallback is removed. The second half of pass-1's observation 2 — that mut-arg / indexed-storage propagation of the narrowing (probes p14/p15) is behavior-verified but not fixture-locked — does not survive as a finding: the four validation sites all funnel through the single `expression_overlaps` engine with no per-caller narrowing logic, so there is no independent code path a separate test could lock. Removing the narrowing already fails the two `invoked_callable_field_*` tests added in `ac31b0908`.

### Findings

**Blocking: none.**

**Non-blocking against the committed head: none.** The remaining items carried in pass-1 are not findings against this diff:

- The lambda-assigned-to-`Callable`-local codegen failure (`SIFR-BUILD-0005` / rustc `E0562`) is pre-existing and untouched — this PR contains no codegen change.
- `cargo clippy -p sifr_lowering --all-targets` lints are pre-existing in test-support files, none in files this PR touches; the project gate (lib target, workspace `-D warnings`) is clean per pass-2.
- Carrying no plan/doc update matches this phase's established pattern of recording review artifacts in a separate post-review commit (`0b0d7b8fb` for #3090).

One record note, outside the reviewed committed range and therefore not a finding against the head: the untracked pass-2 slot file still contains only the interim "create-pr validation profile is still running" note, whose premise was mistaken — no PR #3092 create-pr process was actually running — and the pass-3 slot file is empty. Both should be finalized before any post-merge record commit so the artifact trail does not preserve the incorrect interim framing. I made no edits to either file.

### Verdict

The blocking behavior from earlier passes is correctly fixed and now test-locked at exact head: callable-field invocations record `object place + exact FieldIdentity`, true prefix overlaps still emit structured `SIFR-OWN-0002` at exact ranges, disjoint siblings compile and mutate the original storage, real (including inherited) methods are not misclassified as callable fields, non-place bases stay conservative with arguments still traversed, the module split is a verbatim move plus the single intended `MethodCall` arm, and all touched files are under the size cap. Both pass-1 observations are closed by mutation-verified, uniquely-failing tests.

**SATISFIED — both blocking and non-blocking findings are absent for the committed exact head `fb3712692`.**
