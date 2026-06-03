

Looking at this document as an implementation plan against your six criteria.

---

## Review: ad-hoc-leetcode-benchmark-slowness-root-cause-analysis.md

###1. Fix-focused vs. analysis-only? ✓The phase is clearly implementation-oriented. Purpose, D1-D6, M1-M5, and the Compiler/LeetCode Work Tracks all describe actual fix work. The generated analyzer snapshot validates the inventory. No issues here.

### 2. Implementation decisions captured clearly? Partial

**Good:**
- D1-D6 are concrete and lock decisions for execution- M0-M5 dependency chain is explicit- Post-fix re-benchmark protocol is defined**Gaps:**

| Gap | Location | What's missing |
|---|---|---|
| Heap helper API | L1, D3 | The document says "add or use a Sifr heap/priority-queue helper" but never defines the helper's API shape, placement (which module), or what the benchmark ports call |
| Trie port structure | L2 | "Port the Python trie node algorithms directly to Sifr" is underspecified — does it mean one file per problem, a shared local trie module, or inline implementations? |
| String lowering strategy | C1 | "Introduce an indexed string view or cached character vector lowering" — the document doesn't say which HIR/codegen phase implements this, or what the exact emitted-Rust contract is |
| Container clone elision | C2 | "Distinguish borrowed field reads from owned field moves in HIR/codegen" — no before/after emitted-code example showing the transition from `self.map.clone().contains_key(...)` to borrowed access |
| Tree/list traversal | C3 | "Add borrow-preserving accessors" — no API shape defined |
| Regression test infrastructure | D5, M2, M3 | Every fix needs generated-code regression tests, but the document never says where these live, what tool runs them, or what the assertion language is |

### 3. Hidden gaps, circular dependencies, ownership, acceptance gates

**Circular dependency:**
- M1 L2 (trie parity) must complete before M2 trie clone work is credited — correctly documented ✓
- M1 L1 (heap parity) depends on a heap helper that doesn't exist yet — correctly documented as an L1 deliverable ✓

**Ownership gap:**
- D3 says helpers need "focused fixture tests before broad benchmark use" but doesn't assign test authorship or define "focused"
- M2 and M3 can run in parallel "only for disjoint problem families" — but the document never enumerates which problems are in which track, leaving the parallelism claim unsubstantiated for mixed cases like `0208`/`0211` (trie in both M1 and M2)

**Acceptance gates:**
- Runtime + Peak RSS both visible in report ✓
- Correctness before runtime comparison ✓- Partial benchmarks excluded from apples-to-apples ✓
- Analyzer snapshot agrees with report ✓**Missing gate:** D5 regression tests have no runner, no location, and no pass/fail criterion. A fix can close without proving the pathological lowering is gone.

### 4. Compiler vs. LeetCode Sifr code separation? ✓

M1 = LeetCode Sifr code parity (L1-L3). M2/M3 = compiler lowering (C1-C4). Clean separation. D2 enforces ordering: known-divergent rows don't get compiler credit until Sifr code is fixed.

### 5. Milestones executable without re-litigating decisions? Partial

**Executable:**
- M0 (baseline lock + ticket slicing) is clear
- M1 problem lists are specific (e.g., `1985`, `0973`, `0703`...)
- M5 closure record fields are defined

**Requires re-litigation:**
- M2 C1/C2: "required compiler/runtime direction" is architectural intent, not a commit. An engineer picking this up needs to make decisions about indexed string view API, HIR representation, and codegen lowering strategy before writing a line of code.
- M3 C3/C4: same issue — "borrow-preserving accessors" and "mutable indexed place updates" are descriptions of the goal, not the implementation plan.

###6. Generated-code regression tests, benchmark reruns, memory checks, report metadata gates sufficient? Partial

**Benchmark reruns:** Post-fix protocol (lines 526-543) is data-driven and well-specified.

**Memory checks:** Mentioned in the re-benchmark protocol and acceptance criteria, but no concrete threshold. "Material Peak RSS regression" is not quantified.

**Report metadata:** D6 and M4 fully specify the metadata fields and analyzer enforcement. ✓

**Regression tests:** D5 lists negative assertions (no `chars().nth`, no `self.map.clone()`, no subtree clones) but:
- No test location- No runner command
- No pass/fail definition
- No existing test infrastructure referenced

---

## Summary

**No blocking issues.** The phase is fix-oriented, properly sequenced, and cleanly separated between compiler and LeetCode code work.

**Polishing required before implementation:**

1. **M2/M3 need concrete before/after emitted-code examples** for at least one representative problem per C1-C4. Without this, engineers must invent the codegen contract during implementation, which defeats the purpose of pre-litigating decisions.

2. **Heap helper API must be defined** in L1 or D3 before M1 can execute — even a minimal stub API (e.g., `Heap[T]`, `push`, `pop`, `heapify`) is enough to unblock the ports.

3. **Trie port structure** (inline per problem vs. local module) must be decided in L2.

4. **Regression test infrastructure** needs a location and a runner command. This is a small addition — even pointing to an existing codegen test pattern in the codebase would suffice.

5. **M2/M3 parallelism** is claimed but not grounded — specify which problems are in M2 vs. M3, or remove the parallelism claim.

6. **Memory threshold** for "material Peak RSS regression" should be quantified (e.g., >10% delta at same fixture size).

None of these are blockers. All are resolvable with targeted edits before M0 ticket slicing.
