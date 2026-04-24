# Pass-1 Review: Ad-hoc LeetCode Divergence Closure Phase (2026-04-24)

Reviewer date: 2026-04-24
Review angle: implementation readiness for a developer
Phase under review: `issues/ad-hoc-leetcode-divergence-closure-2026-04-24.md`
Cross-checked against:
- `verification/leetcode/leetcode_divergence_decision_analysis_20260409.md`
- `verification/leetcode/leetcode_pair_diff_scan_20260409.json`
- `audits/leetcode/` (fixture presence spot-checks)

## Summary

The phase is substantively implementation-ready and faithfully inherits the analysis's category model, feature ledger, and Sifr boundary preservation rules. Category totals reconcile (13 + 19 + 21 + 4 + 6 = 63). Below-cutoff inclusions reconcile with the analysis's explicit parity-debt list plus the two Cat 4b fixtures that scan below 80 lines (`0138` at 65, `0141` at 79). Feature IDs map cleanly onto workstreams (`C0` → WS0, `D0`/`N1-N4`/`I1-I2` → WS1, `O1`/`S1-S6` → WS2, `C1-C3`/`R1`/`B1` → WS3, rewrite-debt table → WS4, Cat 4 → WS5, closure → WS6). Non-goals restate every boundary the analysis enumerates.

That said, the phase has a handful of gaps that will actively slow or stall a developer starting execution without re-planning. The blocking list below must be resolved before implementation; the non-blocking list can be folded in during execution.

## Readiness Verdict

**Conditionally ready.** Five blocking edits are needed; non-blocking edits are clarifying and can land during execution.

## Blocking Edits

### B1. `O1` scope is undefined at feature level

Analysis `O1` enumerates concrete helpers: `drain`, `take_at`, `split_first`, `iter_mut_indexed`. The phase says only "owned collection helpers with explicit ownership signatures" and in WS2 states `O1` "helpers only when a target fixture needs the helper". WS4 then lists `O1` as a prerequisite for `0146_lru_cache` and `0706_design_hashmap` with no enumeration of which helpers those fixtures actually need.

Impact: a developer has no minimum deliverable for an `O1` PR and no way to know when `0146` / `0706` are unblocked.

Fix: in WS2, enumerate the `O1` helper set explicitly (copy from analysis), and in WS4 name the specific helpers each fixture requires.

### B2. `0146` and `0706` list undefined design artifacts as prerequisites

WS4 lists "recency-structure design" (for `0146`) and "bucket/open-addressing design" (for `0706`) as prerequisites. Neither is a feature ID, neither is a workstream output, and no PR in "Ready-To-Implement First PRs" produces these designs. These rewrites will stall on phantom prerequisites.

Fix: either add explicit design-note PRs under WS4 (mirroring the `WS1_D0` pattern) that produce `0146_recency_structure_design.md` and `0706_bucket_design.md` before the rewrite, or fold the design decisions into the rewrite PR itself and remove them from the prerequisite column.

### B3. HIR maintainability guardrail check is missing from per-PR validation

`AGENTS.md` lists `python3 scripts/check_hir_maintainability_guardrails.py` as a required local check. Phase "Required Validation Per PR" lists `cargo fmt`, `cargo clippy`, and the quick test script, but not the HIR guardrail. WS1 narrowing changes, WS2 compiler-touching stdlib work, and WS3 cursor ergonomics will almost certainly hit HIR lowering.

Fix: add `python3 scripts/check_hir_maintainability_guardrails.py` to "Required Validation Per PR" with a scope note ("for any compiler/HIR changes").

### B4. WS0 scan-regeneration command is not named

WS0 step 4 says "Rebuild `verification/leetcode/leetcode_pair_diff_scan_*.json` or record the exact command and generated artifact name". The command exists (`python3 scripts/scan_leetcode_pair_diffs.py`), so leaving it to developer discovery is wasted friction and risks divergent invocation flags across PRs.

Fix: hard-code the canonical invocation into WS0, along with the expected output path convention (`verification/leetcode/leetcode_pair_diff_scan_<YYYYMMDD>.json`).

### B5. `WS3_B1` ready-to-implement PR does not name the representative fixture

`WS3_B1_fixture_helper_convention` acceptance requires "one representative fixture migration" but does not name the fixture. The helper convention decision and the pilot fixture are coupled: a poor pilot choice (e.g., one that also needs `C3` + `N2`) will stall the convention PR on unrelated prerequisites.

Fix: name the pilot fixture explicitly (a `C1`-only target such as `0206_reverse_linked_list` or `0021_merge_two_sorted_lists` is the natural pick) and scope the pilot PR to exclude cursor features not yet landed.

## Non-blocking Edits

### N1. 16 `sifr_only` `_v2` fixtures are silently out of scope

The analysis "Not In This Plan" notes the 16 `_v2` Sifr-only fixtures and defers them to separate triage. The phase omits them entirely. A developer running WS0 or WS6 scan regeneration will see them in the scan output and may be tempted to touch them.

Fix: add a one-line "Out of scope" note referencing the 16 `_v2` fixtures and pointing at separate triage.

### N2. WS4 mixes testable and review-only acceptance criteria without labeling

Criteria like "no full merge", "no drain/sort/rebuild", "eviction is `O(1)`", "no linear scans" cannot be asserted inside `main()`; they are enforceable only by source inspection or microbenchmarks. Behavioral criteria (odd/even cases, empty-side cases, canonical value-chain match) belong in `main()` asserts.

Fix: split acceptance into "Behavioral (test)" and "Structural (review)" columns in the WS4 table. Optionally add a lightweight asymptotic probe (e.g., linear-time indicator via repeated-doubling input sizes and simple counter) where `O(1)` vs `O(n)` matters.

### N3. WS1 representative fixtures include cross-category items without labeling

WS1 2b representative list includes `0516` (primary Cat 3) and `0673` (primary Cat 4a). The analysis notes both have layered Cat 2b pressure, so including them for narrowing-rule acceptance is correct — but a future reader may read WS1 and think the primary classification changed.

Fix: annotate `0516` and `0673` as "layered 2b pressure only; primary classification unchanged".

### N4. Non-LeetCode regression rule is documented as a step, not a gate

WS1 implementation step 5 ("Add at least one non-LeetCode regression per narrowing rule") is excellent and exactly right for preventing corpus-specific pattern matching, but it lives inside a steps list. A developer under schedule pressure can legitimately "skip step 5" and still claim WS1 exit.

Fix: move the non-LeetCode regression rule into WS1 exit criteria as a hard bullet ("each narrowing rule lands with ≥1 non-LeetCode unit/e2e regression in the same PR").

### N5. Cat 4a pattern-continuity fixtures are dropped

Analysis calls out `0052_n_queens_ii`, `0543_diameter_of_binary_tree`, `0783_minimum_distance_between_bst_nodes`, and `1466_reorder_routes_to_make_all_paths_lead_to_the_city_zero` as below-cutoff Cat 4a continuity examples (pure-return rewrite of `nonlocal` closure state). Phase WS5 does not list them.

Fix: add them under WS5 as "pattern continuity (not escalated)" with a single-sentence note that if a future scan promotes them they retain Cat 4a classification.

### N6. WS2 S1/S2 are serialized without stated dependency

WS2 Implementation order says "`S1` heap and `S2` DSU first", but Execution Order step 3 reads as sequential ("`S1` heap and `S2` DSU, followed by unlocked rewrites"). Analysis Wave 3 treats them as parallel. No dependency between heap and DSU is stated.

Fix: note explicitly that `S1` and `S2` are parallelizable and can land in any order.

### N7. WS3 `C1` adds `0206`; WS3 `C2` adds `0707`; WS3 `C3` adds `0148`; WS3 `R1` drops `0894`-adjacent reads

Phase WS3 cursor lists diverge from the analysis feature ledger:
- `C1`: phase has `0021, 0023, 0024, 0206`; analysis has `0021, 0023, 0024` (OK: `0206` is a natural cursor pilot).
- `C2`: phase has `0019, 0203, 0707`; analysis has `0019, 0203` (OK: `0707` is a reasonable addition).
- `C3`: phase has `0025, 0092, 0148, 1669, 1721`; analysis has `0025, 0092, 1669, 1721` (OK: `0148` needs `C3`).
- `R1`: phase has `0450, 0669`; analysis has `0450, 0669, 0894 boundary-adjacent reads` (`0894` dropped — reasonable since WS5 owns it as Cat 4b, but the boundary-adjacent-read use case is now homeless).

Fix: briefly annotate the intentional delta from the analysis ledger so this is not mistaken for drift.

### N8. Roadmap / architecture / phase docs not in Required Artifacts

`AGENTS.md` requires that `internal_docs/architecture.md`, `internal_docs/roadmap.md`, and `internal_docs/phases/` be kept current. Phase "Required Artifacts" only lists the execution report, scan, full run, taxonomy, and reviews.

Fix: add roadmap/architecture/phases updates to Required Artifacts with the scope rule that they are updated when a workstream completes (not per PR).

### N9. Baseline Snapshot does not flag layered-category fixtures

"Primary total: 63" is numerically correct but silent on fixtures carrying layered pressure (`0516` Cat 3 + 2b-adjacent; `0673` Cat 4a + 2b). A reader may conclude those fixtures are entirely handled by their primary workstream.

Fix: add a one-line footnote under Baseline Snapshot noting that `0516` and `0673` appear under their primary category plus are addressed for layered 2b pressure under WS1.

### N10. WS0 acceptance focuses only on `changed_py_lines`

WS0 exit criterion reads "Pair scan shows expected `changed_py_lines` reduction". Analysis Preconditions explicitly flags Sifr-side kitchen-sink Node classes / `nodeVal` / `unwrapInt` / `hasNode` helper boilerplate as an independent inflator of `changed_sifr_lines`. WS0 scope mentions "mirrored helper boilerplate" but the acceptance criterion drops the Sifr side.

Fix: split WS0 exit into `changed_py_lines` reduction (from Python-side stacked implementations) and `changed_sifr_lines` reduction where Sifr-side boilerplate was normalized. Name the Sifr-side fixtures if any are in scope for this phase (the four fixtures listed are primarily Python-side noise per the analysis, so the cleanest fix may be to state Sifr-side is out of scope for WS0 and defer to a separate helper-boilerplate sweep).

### N11. `WS2_S1_heap_stdlib` file targets are hand-wavy

The ready-to-implement PR description says "Files: stdlib/compiler/runtime locations discovered during implementation." For a stdlib feature of this scope, the phase should at least point at the crate(s) involved (likely `sifr_hir` for type registration, `sifr_codegen` for lowering, and whichever runtime shim file hosts collections). Without this, the first hour of the PR is archaeology.

Fix: add at least the candidate crate list (`crates/sifr_hir`, `crates/sifr_codegen`, runtime shim) to each `WS2_Sx` ready-to-implement PR description.

## Fixture Classification Spot-Check

All below-cutoff inclusions verified against the scan:
- `0004` (not in above-list; listed below-cutoff) ✓
- `0024`, `0146` (79), `0206`, `0208`, `0211`, `0295`, `0706` ✓ (below-cutoff Cat 1)
- `0138` (65), `0141` (79) ✓ (below-cutoff Cat 4b)
- Cat 4b above-cutoff (`0133`=83, `0160`=113, `0894`=88) correctly not listed under "below-cutoff" ✓
- Cat 1 above-cutoff (`0023`, `0147`=102, `0148`=136, `0212`=127, `0707`=108) correctly routed into WS4 ✓
- Cat 3 (`0104`, `0130`, `0200`, `0516`) correctly routed into WS0 ✓

No fixtures are misclassified or missing from the phase relative to the analysis.

## Safety / Boundary Spot-Check

Every boundary from analysis "Boundaries To Preserve" is mirrored in phase "Non-goals". No proposed feature violates:
- Python truthiness / value fallback — explicitly rejected.
- Implicit nullable access — `N1-N4`/`I1`/`I2` are gated on explicit proofs.
- Mutable `nonlocal` — explicitly rejected; `0673` remains pure-return with post-pass accumulator.
- Ownership weakening / `Rc<RefCell<...>>` / `Cell` — explicitly rejected for `O1`, `S1-S6`, and all cursor work.
- `defaultdict` semantics — explicitly rejected for `S6` trie decision.
- Canonical object-identity rewrites for Cat 4b — deferred to a separate approved arena/handle phase.

## Validation Gate Sufficiency

Given fixes B3 (add HIR guardrail) and B4 (name scan command), the validation gates are sufficient to prove closure:
- Per-PR: targeted fixture check/run + focused compiler/e2e regression + `cargo fmt` + `cargo clippy` + HIR guardrail + `run_all_tests.sh --profile quick`.
- Per closure: full `run_all_tests.sh` + full LeetCode run + pair scan + taxonomy regeneration + review files under `reviews/`.

WS6 exit criterion "Pair scan confirms all high-diff outliers are either canonicalized, intentionally bounded, or separately tracked" is a strong closure gate.

## Summary of Required Actions

Blocking before implementation:
1. Enumerate `O1` helper set; per-fixture `O1` dependency list in WS4 (B1).
2. Resolve phantom design prereqs for `0146` and `0706` (B2).
3. Add HIR maintainability guardrail to per-PR validation (B3).
4. Hard-code scan regeneration command in WS0 (B4).
5. Name `WS3_B1` pilot fixture (B5).

Non-blocking, fold in during execution: N1-N11 above.
