

Now I have everything I need. Let me compile the structured review.

---

## Ad-hoc Phase Review: LeetCode 18-Failure Root-Cause Closure

### Verdict: **NOT_READY**

---

### Critical Corrections

**CC1 — `0705_design_hashset` dropped from WS1.** (per pass1b, confirmed against phase doc fixture table and taxonomy)
- The fixture table row 8 correctly assigns `0705` lane=`both`. WS1 lists `{0049, 0144, 0145, 0286, 0973, 1137}` (6 items) but omits `0705`. The execution doc baseline count is 7 run-stage failures, making the gap mechanically detectable. `0705`'s `Vec<Box<dyn Any>>` lowering defect is a compiler-owned run-stage issue; it cannot be left orphaned.
- **Fix:** add `0705` to WS1, update WS1 exit criterion from "six" to "seven".

**CC2 — C7 section discusses wrong fixture.** (per pass1b, cross-checked against taxonomy)
- Taxonomy: C7 (`other_type_surface_and_api_mismatch`) = `{0973}`, C1 (`codegen_runtime_build_gap`) = `{0049, 0144, 0145, 0286, 0705, 1137}` (6 items).
- Phase doc C1 text lists `{0049, 0144, 0145, 0286, 0973, 1137}` — silently swapping `0705` for `0973` — then C7 text discusses `0973` ("`0973` is primarily a compiler/codegen Optional-index issue") when it should discuss `0705`.
- Live run confirms `0705` fails with `E0277 dyn Any: Clone` (run stage, `codegen_runtime_build_gap` category) and `architecture.md` line 924 contract regression (empty collection literal accepted at check stage when it should be a compile error).
- **Fix:** rewrite C7 section to discuss `0705` as the sole member. After this fix the narrative is internally consistent: C1 = `{0049, 0144, 0145, 0286, 0705, 1137}` (6 items, matches taxonomy) and C7 = `{0705}` (1 item, narrative correctly describes a compile-safety hole plus fixture typing miss). `0973` stays in C1 as the narrative already describes.

**CC3 — `1137` lane classification requires architecture-lock resolution.** (per pass1b)
- `architecture.md` line 151: "`global` / `nonlocal` keywords — Not supported; use closures … or pass values explicitly." The same row covers both keywords.
- If the architecture row means `global` is also unsupported, `1137` (module-level mutable dict writes against unresolved `Memo` with synthesized `__const_Memo()` reads) is an adaptation fixture — the compiler is correctly refusing to emit global mutation.
- If `global` IS supported, `1137` is a compiler emission bug (current classification stands).
- **Fix:** add one sentence to the architecture-lock section: either "module-global mutable binding is also intentionally unsupported" (→ reclassify `1137` as adaptation, lane split becomes 6/6/6) or "module-global mutable binding is supported; `1137` is a compiler emission bug" (→ keep current classification). Without this, WS1/WS3 boundary is indeterminate.

---

### Non-Critical Improvements

**NC1 — `0049` RCA is unverified against generated Rust.** (per pass1b N1)
- The error is a run-stage assertion panic with no compiler diagnostic. The described lowering path (`groups.get(...).cloned().push(...)`) is plausible but unconfirmed. Pre-patch step needed: dump generated `main.rs` for `0049` and confirm the lowering shape before writing the fix.

**NC2 — `0707` carries an unattributed compiler gap.** (per pass1b N2)
- Live run for `0707` shows `cannot compare 'None' and 'ListNode' with !=` diagnostics that are not explicitly attributed to either WS2 (compiler) or WS3 (adaptation). WS2 should explicitly absorb `Class | None != None` comparison support, or WS3 should call out the rewrite-to-`is not None` recipe. Leaving it implicit risks WS2 closing without resolving it.

**NC3 — `0230` adaptation recipe is incomplete.** (per pass1b N3)
- The fixture uses `while curr:` and `while stack or curr:` (truthiness on class instances), which violates the prior phase's operator-truthiness policy lock. WS3 adaptation recipe must explicitly include `while curr:` → `while curr is not None:` and `while stack or curr:` → `while len(stack) > 0 or curr is not None:` rewrites.

**NC4 — WS3 has a hard structural dependency on WS2; make it explicit.**
- WS3's mixed-fixture adaptation patches (particularly `0018`, `0056`, `0230`, `0707`) cannot validate until the corresponding WS2 compiler patches land. The sequential ordering is correct but the dependency is structural, not incidental. One sentence in the execution doc suffices.

**NC5 — Per-category regression gates missing from validation contract.** (per pass1b N5)
- Prior phase locked explicit category-delta gates. Current execution doc's validation contract lists per-wave checks but no category-level closure targets. Recommend adding: `codegen_runtime_build_gap: 6→0 after WS1`, `ownership_and_mutability_boundary: 4→0 after WS3`, `nonlocal_mutable_capture_not_supported: 2→0 after WS3`, etc.

**NC6 — `str.rfind` is low-risk.** (per pass1b N6)
- `str.find` handler is at `crates/sifr_hir/src/lower/expressions.rs:3170-3180`. `rfind` is a one-arm symmetric implementation. The phase doc treats this as a routine WS2 task; it warrants no special treatment beyond the existing entry.

**NC7 — `1849` classification is correct.** Confirmed: `int(str) → Result[int, ParseError]` is a core parse-safety principle. No action needed.

**NC7 (additional) — pass1b has its own lane errors.** (found during this review)
- pass1b lists compiler = `{0049, 0144, 0145, 0286, 0973, 1137, 1930}` — `0973` instead of `0705` and missing `0705` in both. The correct compiler list per fixture table is `{0049, 0144, 0145, 0286, 0973, 1137, 1930}` (7) only if `1137` is confirmed compiler. If CC3 resolves that `1137` is adaptation, the correct list is `{0049, 0144, 0145, 0286, 0705, 1930}` (6). pass1b's lane split math is also off: it computes compiler=7/adaptation=5/both=6 but then proposes a revision of compiler=7/adaptation=8/both=3, which doesn't match any consistent reclassification.

---

### Revised Lane Split Counts

| Scenario | compiler | adaptation | both | total |
|---|---|---|---|---|
| **Current (phase doc)** | 7 | 5 | 6 | 18 |
| After CC2 fix (C7→0705) | 6 | 6 | 6 | 18 |
| After CC2+CC3-fix-B (`global` unsupported, `1137`→adaptation) | 6 | 6 | 6 | 18 |

CC2 alone corrects the narrative inconsistency. CC3 scenario B (reclassify `1137` as adaptation) produces a clean 6/6/6 split. Both scenarios require updating the lane counts from the phase doc's stated 7/5/6.

---

### Revised Workstream Ordering

**No structural change to 1→2→3→4 is required.** However:

- **WS1 and WS2 can parallelize** (codegen run-stage vs check-stage type system; zero fixture overlap; zero expected file overlap in patch surfaces). Rename "Workstream Order" to "Workstream Dependency Graph" with a note that only WS4 has strict dependence. (per pass1b NC5)
- **WS3 is strictly after WS1+WS2** for mixed fixtures — make this explicit in the execution doc, not just implied by ordering. (per pass1b N4)
- **Mandatory under CC1:** WS1 fixture list grows from 6 to 7 (`0705`). Exit criterion updated to "all seven fixtures pass `sifr run`".
- **Conditional under CC3 scenario B:** `1137` moves from WS1 to WS3. WS1 → `{0049, 0144, 0145, 0286, 0705, 0973}` (6); WS3 gains `1137` alongside `0402, 0442, 0543, 0673, 1849`.

---

### What Is Already Sound

- **Fixture-level table arithmetic** totals 18 correctly against the corpus.
- **Stage math:** run (7) / check (11) = 18 consistent with corpus.
- **`nonlocal` architecture lock** on `0543`/`0673` correctly cited; adaptation-only decision aligns with Sifr explicit-data-flow principle.
- **`int(str) → Result`** decision for `1849` correct; parse-safety is a core principle.
- **`str.rfind` parity** for `1930` correctly classified as compiler parity add.
- **Mutability contract** (`mut` parameter requirement) correctly kept for `0018, 0056, 0402, 0442`; load-bearing for ownership story.
- **WS4 rerun-and-regenerate pattern** consistent with prior phases; artifact targets are correct closure signal.
