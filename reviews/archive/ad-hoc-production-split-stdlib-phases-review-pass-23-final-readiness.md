Checking all three files across seven dimensions: phase ordering, cross-reference accuracy, state vocabulary completeness, dependency contract consistency, M0 definition-of-done alignment, quality contract prohibitions, and inventory state policies.

---

**1. Phase ordering — all three files**

All three state the same explicit order: 1 → text/i18n, 2 → concurrency/runtime, 3 → network/HTTP. No contradiction.

**2. Cross-file links**

- Text/i18n links to both sibling phases by exact filename. ✓
- Concurrency links to both sibling phases by exact filename. ✓
- Network's "Split-Out Phases" section links to both sibling phases by exact filename and lists the recommended order. ✓

**3. State vocabulary — network concurrency/runtime table (the pass-22 patch target)**

The six rows in the Concurrency/Runtime Dependency Decisions table now carry `blocked-on-concurrency-runtime-m1`, `m2`, `m3`, `m5`, and `m6` — all milestone-specific. The allowed-states list (lines 213–220) covers exactly those labels plus `production-substrate`, `deferred-to-http-client-phase`, `deferred-to-phase-41`, and `rejected`. No generic `blocked-on-concurrency-runtime` residue remains. The M0 definition-of-done (line 542) enumerates the same set. ✓

No `m4` slot in the network blocked-states list — correct, because M4 is subprocess, and no network surface needs to wait on subprocess before it can ship.

**4. Text/i18n → network unblock points**

Network's text-dependency table uses `blocked-on-text-i18n-m1`, `m2`, `m2_5`, `m3`. Text/i18n defines milestones `milestone_text_i18n_1` through `milestone_text_i18n_5` plus `2_5`. All referenced unblock labels resolve to real milestones. The M0 definition-of-done for network (line 541) enumerates the same set. ✓

**5. Concurrency M4 named-revisit entry**

M4 references a ledger entry named `milestone_concurrency_runtime_text_subprocess_integration`. This is a ledger artifact name, not a phase milestone — consistent with how named revisit points work in these docs. Not a gap. ✓

**6. Inventory exit-state policies**

- Text/i18n: `open` forbidden at phase exit; every CPython test family must reach one of five states; every public surface must reach one of five states. ✓
- Concurrency: every surface must be classified by tier and terminal state before any milestone closes. ✓
- Network: `open` allowed during implementation only, forbidden at phase exit; CPython test families must reach `mined`, `blocked`, `rejected`, or `external-signal`; public surfaces must reach one of the listed states. ✓

**7. Quality contract prohibitions**

Network quality contract explicitly prohibits local encoding registries, local Unicode tables, locale-derived defaults, fallback decoders (text provider), and also local cancellation tokens, timeout coordinators, shutdown managers, offload pools, executors, queue/channel primitives, task contexts, process/worker pools, IPC mechanisms, and diagnostic routing (runtime provider). Both provider rules are present and coherent with what the concurrency and text/i18n phases own. ✓

---

**Result: PASS**

No material blockers, stale labels, or implementation-blocking contradictions found. All three documents are consistent on phase order, cross-references, state vocabulary, dependency contracts, M0 gates, and quality prohibitions.
