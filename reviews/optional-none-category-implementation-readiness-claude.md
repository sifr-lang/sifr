## Review: Optional/None Category Breakdown

**Verdict: Mostly Ready**

### Findings (ordered by severity)

1. **Missing acceptance criteria per workstream** — Each workstream describes symptoms and expected fixture impact but doesn't define what "done" looks like技术上. Engineers need concrete criteria to know when a workstream is complete (e.g., "branch merge must preserve narrowed type", "join of `int` and `None` must not produce `Unknown | None`").

2. **No technical implementation sketch** — The document describes *what* is broken but not *how* to fix it. For a compiler workstream this size, even a rough sketch of the approach (e.g., "extend the CFG to carry type facts per program point", "add meet-of-joins for Optional types") would give engineers a starting anchor. Currently every workstream reads as "research needed first."

3. **No cross-stream interaction mapping** — Workstreams 1 and 2 are acknowledged as tightly coupled, and workstream 4 depends on a "recursive-surface owner" confirmation. These dependencies aren't tracked. An engineer starting on workstream 1 could spend effort that becomes invalid once workstream 2 reveals its scope.

4. **No test strategy** — The document references fixture counts but doesn't describe:
   - How to validate narrowing correctness unit-level (vs. end-to-end)
   - Whether existing test infrastructure can isolate each workstream
   - How to prevent regressions in already-passing fixtures

5. **Execution order justification is thin** — "1 should lead because tightly coupled with 2" is stated but not explained. A one-liner on *why* path-sensitive narrowing must precede inference cleanup (e.g., narrowing produces the facts that inference joins on) would make the order actionable rather than asserted.

6. **Workstream sizing is missing** — Impact is listed as "roughly N fixtures" but there's no complexity or effort estimate. This makes it hard to sequence sprint planning or know if workstream 3 can truly start after workstream 1.

### Minimum Changes to Make Fully Implementation-Ready

- Add a **per-workstream "definition of done"** section with concrete type-behavior expectations
- Add a **one-paragraph technical approach** sketch per workstream (not full design, just enough to orient a compiler engineer)
- Add a **cross-stream dependency table** noting which workstreams block or strongly inform others
- Add a **validation approach** section describing how each workstream is tested in isolation before integration
- Clarify why workstream 1 precedes workstream 2 with a minimal causal argument (not just "tightly coupled")

### What Works Well

- The "What Not To Do" section is excellent and protects Sifr's architecture
- Workstream decomposition is sound and crisply separated
- Representative failures per workstream give engineers concrete targets
- The "residual lane" framing correctly de-prioritizes fixture rewriting
- The "Bottom Line" model (dominant root cause + independent workstreams) is the right level of abstraction
