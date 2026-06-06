## Review: Production Concurrency, Process, And Runtime Substrate

---

## Verdict: FAIL

Twelve blocking gaps remain. No milestone can be fully implemented while gaps 1–5 are open; the execution ledger has at least one demonstrable inaccuracy (gap 6); and several milestones have definitions of done that cannot be verified against their own stated scope (gaps 7–12).

---

## Blocking Findings

### 1. Sendability/Shareability Contract Has No Implementation Milestone — Severity: Critical

**File/section:** Phase contract, "Sendability And Shareability Contract" (lines 172–183); M3 DoD; M4 DoD.

The contract designates sendability/shareability as a "phase-wide gate before raw task spawning, blocking offload, CPU offload, thread pools, process workers, or cross-boundary closure captures ship." It lists eight required compiler-visible rules including typed compile-time diagnostics for non-send captures and shared mutable state violations. However, no milestone (M0–M7) owns the HIR type-checker and codegen changes required to enforce these rules. M0 defines the model on paper. M3 and M4 each have DoD items that assume enforcement exists ("Offloaded work returns typed results," "No adapter can bypass owned process/pipe lifecycle"), but nothing in M1–M6 adds the compiler enforcement.

Without compiler enforcement, the "phase-wide gate" is prose. M3 and M4 cannot satisfy their own definitions of done.

**Remediation:** Either expand M1 scope to include the HIR/type-checker changes for send/sync capture analysis (the natural place, since task spawning is where send constraints first apply), or create an explicit sub-phase entry in M0 that gates M2 onwards. Add "sendability/shareability compiler diagnostics pass for representative fixtures" to M3 and M4 DoDs as a verification step.

---

### 2. `TaskGroup[T, E]` Homogeneous vs. Heterogeneous Design Unresolved — Severity: Critical

**File/section:** M1 scope (line 262–278); M1 definition of done.

`TaskGroup[T, E]` carries a single success type parameter `T`, which implies all child tasks must return the same type. This is never stated, and `join_all`, `race`, and `select` appear in the same scope without clarifying whether they accept heterogeneous futures. In a static type system, a homogeneous-only group is a significant constraint that affects real-world usage patterns. Two implementers reading this doc would make incompatible design decisions: one would implement a homogeneous group, the other a union/existential over child types.

M1 cannot be implemented until this is decided. The decision also propagates to `race`/`select` cancellation semantics for losing branches of different types.

**Remediation:** Add this decision to M0 as a required output, explicitly listed in M0's definition of done. State the chosen model (e.g., "TaskGroup is homogeneous; heterogeneous composition uses `select` over typed handles") and add a sentence to M1 scope reflecting it.

---

### 3. Detached Task Policy Is an Unresolved Either/Or Inside M1 Scope — Severity: Critical

**File/section:** M1 scope (line 273).

"Define detached-task policy: forbidden by default or explicit with failure observation." This is an open design choice, not a scope item. It must be decided before M1 starts because the task lifecycle model, handle-drop semantics, and failure propagation all depend on it. If detached tasks are forbidden, drop of a `TaskHandle` without join is a compiler error or diagnostic. If they are allowed with explicit failure observation, handle types and spawn APIs are structurally different.

**Remediation:** Move this decision to M0. Pre-commit the chosen policy in the phase contract before M1 begins. Remove the either/or phrasing from M1.

---

### 4. Text-Mode Subprocess Has No Owner in This Phase — Severity: High

**File/section:** M4 scope (line 391); Cross-Phase Dependency Contract (lines 65–66).

M4 says "Preserve binary pipe mode in this phase; text/encoding/error mode waits for text/i18n `milestone_text_i18n_1`." But `milestone_text_i18n_1` is a hard prerequisite to this entire phase starting — it must already be complete before M0 begins. After M4, text-mode subprocess and pipe encoding modes are neither scheduled in M5–M7 nor recorded as a formal deferred/waived item with a revisit rule. The work disappears.

M7's integration scope and demo list include "async subprocess pipeline" but do not add text-mode encoding tests or subprocess text API surface.

**Remediation:** Since `milestone_text_i18n_1` is a pre-condition of this phase, remove the conditional deferral from M4. Either: (a) include text-mode subprocess work in M4 scope directly, or (b) move it to M7 as an explicit integration item, or (c) record it as a formal `deferred` entry in the inventory with a named revisit milestone. Any of the three is acceptable; silence is not.

---

### 5. `JoinSet[T, E]` (M3) vs. `TaskGroup[T, E]` (M1) Semantic Distinction Undefined — Severity: High

**File/section:** M1 scope (line 263); M3 scope (line 343).

Both are listed as collection-of-tasks APIs with similar type parameters. Their functional distinction is never stated anywhere in the phase doc. An M3 implementer who has not read external Tokio docs would have no basis to distinguish them from the phase contract alone, and could produce an API that duplicates or conflicts with M1's TaskGroup.

Typical distinctions (structured/scoped vs. dynamically-growable/unscoped, cancellation-on-failure vs. collect-as-complete) need to be explicit in the phase contract, not assumed from Rust ecosystem knowledge.

**Remediation:** Add one sentence to each milestone defining the distinction. Example: "TaskGroup provides structured scoped concurrency with automatic cancellation on child failure. JoinSet is a dynamically-growable collection for collecting heterogeneous results as tasks complete, without automatic cancellation."

---

### 6. Waiver Index Is Empty but `signal.pause` Was Explicitly Resolved as Unsupported/Waived — Severity: High

**File/section:** Execution ledger, "Waiver Index" (line 208); Remediation checklist (line 129).

The remediation checklist records: "Resolve `signal.pause()` to unsupported/waived in this phase with diagnostics and a future safe signal-handler or structured signal-stream revisit rule." This was accepted as a planning remediation. The Waiver Index says "No waivers recorded yet." These contradict each other directly.

An empty Waiver Index means no future reviewer can verify that the `signal.pause` decision was correctly closed. It also means the execution ledger does not accurately reflect the current planning state after 21 review passes.

**Remediation:** Add the `signal.pause` entry to the Waiver Index immediately, with terminal state `unsupported`, rationale, revisit rule (structured signal-stream revisit), CPython evidence pointer, and the regression fixture placeholder.

---

### 7. M0 Definition of Done Does Not Cover Its Own Code Change Requirement — Severity: High

**File/section:** M0 scope (lines 248–249); M0 definition of done (lines 252–256).

M0 scope requires: "Add import-resolution tests for canonical `sifr.*` module names and negative diagnostics for bare CPython stdlib import attempts." These are code and test changes. M0's four-item definition of done covers only documentation and classification outputs — no DoD item verifies that these tests exist or pass.

M0 could be marked complete with classification done but tests never written, leaving a silent gap the first time an implementer tries `from queue import Queue` in a Sifr file.

**Remediation:** Add "import-resolution tests for canonical `sifr.*` names pass and negative-diagnostic tests for bare CPython stdlib import forms pass" to M0's definition of done. Alternatively, move the test requirement to M1 if M0 is intended to be documentation-only.

---

### 8. Phase 32 Async Model Is Referenced as Binding With No Citation — Severity: High

**File/section:** Lines 94–99 ("Current Sifr Baseline"); every milestone that inherits the async model constraints.

"The Phase 32 async model remains binding" appears at the phase level and implicitly governs M1–M4. There is no link or file reference to where Phase 32 is documented. Implementers must know whether `@blocking_io`/`@cpu_heavy` effect annotations already exist in the compiler, what the existing suspension point rules are, and what infrastructure they inherit. Without a citation, every implementer has to go discover this independently, and any discrepancy between Phase 32 docs and this phase's assumptions will go undetected until M1 implementation.

**Remediation:** Add a cross-reference (e.g., `internal_docs/phases/phase32.md` or the relevant roadmap entry) immediately after the first mention of "Phase 32 async model." Confirm whether `@blocking_io` and `@cpu_heavy` are existing compiler features or must be added in this phase.

---

### 9. Task Context Is Conditional in M5 but Network/HTTP May Require It — Severity: Medium

**File/section:** M5 scope (lines 433–437); cross-phase dependency contract (line 63); open planning question #6.

M5 adds task/request context "if needed by tracing, deadlines, cancellation metadata, and future web observability." Open planning question #6 asks "What task/request context model is needed before web observability work?" but is not listed as a required output of M0's definition of done. If an M0 implementer skips question #6 (there is no DoD gate for it), M5 will enter implementation still conditional, and the network/HTTP phase will have no observability/tracing substrate.

**Remediation:** Add "all 8 open planning questions answered in the execution ledger" to M0's definition of done. Specifically, question #6 must produce a definitive answer ("task context is required before network/HTTP M1" or "deferred with explicit network/HTTP design waiver") that removes the conditionality from M5's scope before M5 starts.

---

### 10. `sifr.parallel` M3 Scope Is Underspecified for Implementation — Severity: Medium

**File/section:** M3 scope (lines 340–351); M3 definition of done.

M3 names `parallel.map` and `parallel.try_map` as the `sifr.parallel` surface but provides no type signatures, no specification of accepted input types (iterables? slices? custom collections?), no ownership/capture model for closures, no pool-sizing or concurrency-control API, and no statement of whether work items must satisfy sendability. M3's definition of done has no surface completeness check — it only covers that blocking diagnostics work and worker failures are typed.

An implementer cannot write a spec-conformant `parallel.map` from this text alone.

**Remediation:** Add minimum specification to M3: accepted input collection types, type signatures for `map`/`try_map`, whether items and closures must satisfy sendability (and which gate covers this), and the pool-sizing API name and behavior. These can be brief bullet points.

---

### 11. Supported Host Matrix Is Referenced but Undefined — Severity: Medium

**File/section:** M4 definition of done (line 407).

"Sync and async subprocess loopback tests pass on the supported host matrix." The supported host matrix is not defined in either file. It may exist in `internal_docs/architecture.md` or the roadmap, but without a reference here, M4 has an unverifiable DoD item — passing on an undocumented matrix is not a gate.

**Remediation:** Either add a cross-reference to where the host matrix is defined, or add a sentence in M0 scope requiring the host matrix to be recorded in the inventory as part of the M4 host-limited classification work.

---

### 12. M0's Eight Open Planning Questions Are Not Listed as Required Outputs in M0's DoD — Severity: Medium

**File/section:** M0 definition of done (lines 252–256); open planning questions (lines 593–603).

The phase contract closes with eight planning questions that "must be answered in the phase execution ledger before implementing the affected milestone." M0's definition of done contains four items, none of which mention these questions. An implementer completing M0 to DoD satisfaction could skip all eight questions and still mark M0 done. Questions 1 (sendability rules), 2 (stable vs. internal task APIs), 3 (adapter migration value), and 6 (task context before web) are load-bearing for M1–M5 respectively.

**Remediation:** Add "all 8 open planning questions recorded and answered in the execution ledger" as a fifth DoD item for M0.

---

## Non-Blocking Polish

**a.** `sifr.asyncio` currently exists as a "compatibility veneer" (baseline, line 89). If M0 classifies it as `adapter-later` or `rejected`, no milestone plans the cleanup of the existing module. Add a note that M0 classification triggers a named cleanup action.

**b.** M6 definition of done lacks the "adopted/adapted/waived CPython test family" requirement that every other milestone (M1–M5) carries. M6 lists five CPython test families to mine but does not require them to be classified.

**c.** M5's deferred helpers (`redirect_stdout`, `redirect_stderr`, `chdir`, `suppress`) are described as "defer or reject unless M0 proves" — but M0 scope does not include classifying these. Add them to M0 CPython evidence scan or specify that M5 will record them as `intentional-diff` or `deferred` during its own inventory pass.

**d.** The "stdlib workload database" referenced in the Quality Contract (lines 587–588) has no definition or cross-reference in either file. Implementers cannot classify blocking/async functions into a database that isn't described.

**e.** M7 names "the designated compiler/runtime reviewer recorded in the execution ledger" but the execution ledger has no such designation. This should be filled in as the first action of M0 or during planning.

**f.** M6 references "typed local serialization...only after typed IPC design approval" without defining what approval means or who grants it. Add a one-line process statement (e.g., "approval = a named design artifact reviewed by the phase owner and compiler reviewer, recorded in the execution ledger before the serialization crate is selected").

**g.** The `signal` constants/enum-like values in M5 are described as "supported signal constants/enum-like values" and "strsignal where host-supported" without specifying which signals are in scope on which hosts. At minimum, note that M0 must record the signal-to-host matrix as a host-limited classification in the inventory.

---

## Why FAIL

Gaps 1–3 make at least M1 and M3 unimplementable as written. Gap 4 silently drops text-mode subprocess work after a prerequisite that is already complete. Gap 5 risks incompatible design between M1 and M3. Gap 6 is a concrete factual inaccuracy in the execution ledger. Gaps 7–12 leave DoD items unverifiable or introduce design decisions that propagate to later phases without a forcing gate in M0. The phase cannot start M1 implementation until at minimum gaps 1–5 are resolved and the execution ledger is corrected (gap 6).
