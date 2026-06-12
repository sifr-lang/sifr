**FAIL**

Three implementation-blocking gaps remain.

---

**Blocker 1 — Phase 1 / M2: TLS wrap failure state is an open decision**

The plan explicitly flags "Failed wrap state must be chosen before M2" but records no decision. The failure-path ownership model (what type is returned, whether the original socket is consumed or not, what error variant carries the state) is undefined. M2 implementation cannot begin without it.

*Required doc change:* Add the chosen failure-state ownership rule to the Phase 1 plan before the M2 gate, with the typed error variant and socket consumption semantics specified.

---

**Blocker 2 — Phase 2: signal.pause adoption is unresolved**

The plan says "signal.pause is sync-only blocking with async diagnostic **if adopted**." The conditional "if adopted" leaves scope open. Implementers cannot know whether to build this API or write a waiver. Neither an adoption decision nor a CPython-evidenced waiver is recorded.

*Required doc change:* Replace the conditional with either (a) an explicit adoption entry with sync-only behavior spec and async-context diagnostic defined, or (b) a waiver entry with CPython evidence and revisit rule, consistent with the pattern used for contextmanager/asynccontextmanager.

---

**Blocker 3 — Cross-phase: text_i18n_1 dependency milestone is unspecified**

Both Phase 1 ("Non-UTF-8 URL/HTTP text behavior waits on text_i18n_1") and Phase 2 ("subprocess text waits on text_i18n_1") have hard forward dependencies on Phase 3, but neither names *which milestone* of text_i18n_1 is required. Without the milestone pin, the dependent phases cannot sequence their own milestones relative to Phase 3 — specifically, the Phase 1 URL/HTTP feature gates and Phase 2 subprocess text gates have no schedulable unblock point.

*Required doc change:* In both Phase 1 and Phase 2 plans, replace "waits on text_i18n_1" with "waits on text_i18n_1 M[N]" naming the exact milestone, and add the reciprocal dependency note in the Phase 3 plan at that milestone boundary.
