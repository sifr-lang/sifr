Here is the status against each prior issue:

---

**1. Overview rewrite vs retitle — unresolved inconsistency (blocker-level clarity)**

Section 8's first bullet (line 493) says "retitle it as `Concurrency Overview`," which implies only a heading change. But line 567 within the same section says it "should become a mental-model entry point," and Slice 3 acceptance (line 827) explicitly says it "is rewritten as a concise mental-model entry point with navigation cards; it is not merely the old long page with a new title." The word "retitle" in section 8's opening bullet directly contradicts the Slice 3 acceptance criterion. An implementer reading section 8 top-to-bottom could deliver a renamed page and still satisfy the section 8 spec, but fail the Slice 3 gate. The opening bullet needs to say "rewrite" to match the acceptance criterion.

**2. Async and Await sourcing — partially resolved, not pinned per-page**

The content disposition table (line 558) says "`Async Functions` examples — Move to `Async and Await`; expand with demo-sourced suspension examples." Slice 3 acceptance (line 828) says it is "partly net-new writing sourced from demos." The general demo list (lines 834–835) names `structured_concurrency_demo`, `task_core_demo`, `cancellation_cleanup_demo`, and `ownership_concurrency_demo`. None of those are primarily async/await demos. The codebase scan (line 121) lists `async_*` and `blocking_offload_demo` but those names never appear in the Slice 3 acceptance criteria. The implementer has to independently discover which demos contain suspension-focused content. This is a refinement gap, not a blocker, but it should either name the `async_*` demos explicitly or acknowledge the lookup is required.

**3. Cancellation and Timeouts net-new writing — resolved**

Content disposition (line 561) says "mostly demo-sourced net-new writing." Slice 3 acceptance (line 828) says "partly net-new writing sourced from demos." The phrasing differs slightly ("mostly" vs "partly") but both affirm net-new writing from demos, and the available demos (`task_core_demo`, `cancellation_cleanup_demo`) are named. No conflict, no blocker.

**4. Route asymmetry — resolved**

Section 8 (lines 495–496) explicitly states: "The route asymmetry is intentional: the overview keeps the existing `docs/language/concurrency.mdx` path for stability, while new sibling pages can live under `docs/concurrency/`." Slice 3 (lines 803–808) repeats this cleanly. No ambiguity.

**5. stdlib sidebar naming collision — resolved**

Section 8 (line 493) says to label it `Concurrency API`. The sidebar shape (line 634) shows `Concurrency API` under Standard Library. Slice 3 acceptance (line 824) repeats the requirement explicitly. Consistent throughout.

**6. Content disposition — resolved**

The disposition table (lines 555–566) maps every chunk of existing material to a destination. The rules at lines 574–576 (concept vs reference vs guide vs diagnostics) bound the scope cleanly. No gaps.

---

**Summary**

One blocker remains: the word "retitle" in section 8's opening bullet contradicts the "rewritten" requirement in Slice 3's acceptance criteria. Before implementation, that bullet should be changed to reflect a genuine rewrite. All other prior issues are resolved. The one high-value refinement is naming the `async_*` / `blocking_offload_demo` demos in the Async and Await sourcing note so the implementer does not have to rediscover them.
