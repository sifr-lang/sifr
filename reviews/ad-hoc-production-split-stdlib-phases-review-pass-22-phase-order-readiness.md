**Verification of all three pass 21 remediations:**

---

**Remediation 1: Network doc concurrency/runtime dependency matrix**

Confirmed present. Lines 199–222 contain a full table with six rows covering TCP/TLS/HTTP cancellation (`blocked-on-concurrency-runtime-m1`), stream backpressure (`m1` or `m2` as classified in M0), sync network helpers (`m3`), graceful shutdown (`m5`), network observability (`m5`), and executor/process-backed serving (`m3` or `m6`). Each row specifies the status before the gate, the provider dependency, and the decision. The status vocabulary and the prohibition against local substitutes are present. ✅

**Remediation 2: Cancellation/timeouts/shutdown as provider-owned**

Confirmed in three locations in the network doc:
- Objective (line 30–31): explicitly calls these "network-layer applications of the concurrency/runtime provider phase" and forbids a local cancellation token model, shutdown coordinator, or offload pool.
- Architecture (lines 301–302): names `milestone_concurrency_runtime_1` and `milestone_concurrency_runtime_5` as the exact owner milestones.
- Quality Contract (line 805): explicitly bans introducing any local cancellation model, deadline coordinator, shutdown manager, offload pool, queue/channel primitive, task context, or IPC mechanism. ✅

**Remediation 3: Naming notes on legacy filenames**

Text doc (line 7) and concurrency doc (line 7) both carry the naming note. Network doc filename was already renamed to `platform-substrate` and needs no note. ✅

---

**Phase-ordering consistency across all three docs:**

All three state the same explicit order: text → concurrency → network, with the same rationale: text provides the shared encoding substrate; concurrency closes task/cancellation/shutdown/offload so network consumes provider primitives rather than inventing substitutes. ✅

**Cross-phase blocking gates:**

- Text doc correctly gates concurrency (binary subprocess pipes only before text M1) and network (binary TCP/TLS/HTTP only before text M1).
- Concurrency doc correctly names text M1 as the hard prerequisite for text-mode subprocess, warning encoding, and locale-sensitive formatting.
- Network doc correctly blocks non-UTF-8 body decoding, URL codec-label lookup, and text-heavy demos on text M1; blocks executor-backed serving and graceful shutdown on concurrency M3/M5.
All gates are consistent and non-contradictory. ✅

---

**One minor doc inconsistency — not a material blocker:**

Network doc, lines 244–255: the surface state vocabulary list uses generic `blocked-on-concurrency-runtime` and only `blocked-on-text-i18n-m1`, while the M0 DoD (lines 537–538) and the dependency tables require numbered milestone labels (`blocked-on-concurrency-runtime-m1` through `m6`, `blocked-on-text-i18n-m1` through `m3`). This creates a labeling ambiguity during M0 classification work. However, M0 itself references the numbered forms in both its DoD and the dependency tables, so an implementer doing M0 would naturally resolve to the granular labels. This is a cosmetic editorial inconsistency, not a blocker for architecture decisions, dependency selection, or implementation start.

---

**PASS**

All three pass 21 remediations are in place and correct. The three documents are internally consistent on phase order, provider ownership, cross-phase blocking gates, and M0 classification requirements. The one minor inconsistency in the surface state vocabulary list does not block implementation.
