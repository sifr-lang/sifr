

Here's my comprehensive review. Let me be direct: the memo's strategy is sound in direction but needs concrete doc changes before it's ready.

---

## 1. Is the recommended strategy the best architecture?

**Mostly yes, with one structural gap.**

The `Sifr Ruff fork -> sifr_syntax -> sifr_frontend -> sifr_analysis -> sifr_lsp` pipeline is the correct shape. Option D is the right call - don't use ty as Sifr's brain, but do aggressively mine ty_server for protocol/session architecture. The parallel-lane model for unlimited developers is appropriate.

**The structural gap**: The memo treats `lsp-server` and `lsp-types` as "maybe" dependencies that Sifr might use or might build from scratch over. This is wrong. These are direct, unambiguous, zero-risk dependencies that should be stated as first-class parts of the architecture:

- `lsp-server` is a generic Rust JSON-RPC dispatch library with no Python semantics whatsoever. It's already in the fork.
- `lsp-types` is a pure LSP data type crate. No semantics. Already in the fork.
- They are the foundation on which `ty_server` itself is built.

The "reuse ty_server shell" debate is actually about the more complex pieces - session state management, document indexing, request queues, cancellation scheduling - not about whether we use `lsp-server`. We do. The question is only how much of ty_server's specific wiring we borrow vs. implement independently.

**The deeper problem with Option D**: The memo claims `ty_server` protocol/session shell pieces "separate cleanly" from Python semantics. Based on my reading of `ty_server/src/session.rs` and the dependency graph, this is likely an optimistic assumption. ty_server's Session owns a salsa `Db` that is deeply typed to Python semantics - workspace projects, Python module resolution, Python source types, Python settings all live in the project layer. The LSP session/document state is not cleanly separable from the Python project state because they share the same salsa database. The audit spike needs to test this assumption on a concrete example, not assume it.

---

## 2. Should Phase 36 add an explicit ty/Ruff reuse audit/spike milestone?

**Yes, and the memo is correct to propose it. But the phase doc needs concrete scope, not just the name.**

The current Phase 36 structure begins with milestone sequencing, but the audit milestone has no defined scope in the phase doc. The memo's proposal (`milestone_36_0: Tooling Reuse Audit And Architecture Spike`) is the right thing to do, but it must be:

1. **Entered as a named milestone in the phase doc's milestone section** - not just implied by the memo. The phase doc must own this decision.
2. **Given a concrete exit criteria that forces a binary choice** - either adapt code, follow patterns, or defer. "Evidence-based, not preference-based" in the memo's words is correct but needs a sharper mechanism.
3. **Given a specific test-the-assumption deliverable** - the spike must produce a working minimal `AnalysisHost` wired through a selected LSP shell path to prove clean separation is actually achievable.

Without these concrete elements, the audit milestone will become an indefinite research loop rather than a decision milestone.

---

## 3. Should Phase 35 change too?

**Yes, one targeted addition.**

Phase 35's `verification/performance/check_split_brain_guardrail.py` currently guards against new parser/lowering/type-check/semantic diagnostic entrypoints outside approved boundaries. That's the Sifr semantic layer.

Phase 36's concern is different: preventing Python semantic dependencies from creeping into `sifr_lsp` via library-level imports (e.g., accidentally depending on `ty_python_semantic` types, `ty_project` module resolution, or Python environment discovery).

Phase 35 should explicitly note that the split-brain guardrail infrastructure it establishes must extend to guard against Python semantic dependencies in tooling/LSP paths. This is not a change to Phase 35's scope - it's an explicit statement that the guardrail mechanism it creates will be used by Phase 36's tooling checks. The Phase 35 exit criteria already mention "parity, protocol, performance, and split-brain guardrails" but the guardrails' scope with respect to Python dependencies should be explicit.

---

## 4. Are there better options?

**Yes on one point: make `lsp-server` and `lsp-types` explicit first-class dependencies, not "maybe" dependencies.**

The five options in the memo are correctly evaluated and Option D is the right call. But the option evaluation treats `lsp-server` and `lsp-types` as something to potentially build over rather than directly use. This creates false ambiguity.

The fork already contains these. `ty_server` uses them. There is no meaningful reason to build an alternative. The architecture should state:

```text
sifr_lsp
  - depends directly on lsp-server and lsp-types (already in fork, no Python semantics)
  - may reuse/adapt audited ty_server protocol/session shell patterns
  - must not depend on ty_python_semantic or Python project semantics
```

The "may reuse/adapt audited ty_server shell patterns" part is the real design question that the audit spike resolves. The `lsp-server`/`lsp-types` part is settled.

The option evaluation also doesn't address the Salsa coupling problem explicitly. `ty_server`'s session state, document indexing, and request handling are built around a salsa database typed to Python project semantics. The memo says "Not all ty_server pieces will separate cleanly from ty_project" but treats this as a risk to manage rather than a likely outcome. The spike should be designed to test the clean-separation hypothesis on a concrete example (e.g., wire a mock `AnalysisHost` through ty_server's document-index -> request-dispatch path) before committing to the shell-reuse path.

---

## 5. Concrete doc changes required

Here is the specific list of changes needed before this is PR-ready:

### Phase 36 doc changes

**A. Add the audit milestone to the milestones section with explicit scope:**

```markdown
### milestone_36_0: Tooling Reuse Audit And Architecture Spike
- Scope:
  - Audit `ty_server/src/session.rs` (session state, document index, workspace/projects state),
    `ty_server/src/server.rs` (server init, capability negotiation, main loop),
    `ty_server/src/document/` (document sync, version handling),
    `ty_server/src/api/` (request handlers, diagnostics publication),
    `ty_ide/src/` (completion/hover/goto query algorithms, semantic token shapes),
    `ty_project/` (project discovery, module resolution, settings - reject as Sifr dependency).
  - Classify code into:
    - `reuse-direct`: may be adapted into sifr_lsp with clean boundaries.
    - `reference-only`: use as design pattern only, implement independently.
    - `reject`: Python semantic/project/runtime assumptions.
  - Build a spike that wires `sifr_frontend`/`sifr_analysis` through the selected shell path
    (document open -> query dispatch -> LSP response) to test whether extraction is clean.
  - Remove or convert the spike before phase exit.
- Definition of done:
  - A decision matrix documents which shell pieces can be adapted vs. must be implemented independently.
  - The spike proves or disproves clean separation of ty_server shell from ty_python_semantic.
  - The decision is evidence-based, with audit findings recorded in `internal_docs/tooling_reuse_strategy.md`.
  - Forbidden dependency graph is documented: `sifr_lsp` must not transitively depend on
    `ty_python_semantic`, `ty_project` (Python semantics), or Python environment discovery.
```

**B. Add explicit direct dependency statement in the Architecture Ownership section:**

Add a clause noting that `sifr_lsp` uses `lsp-server` and `lsp-types` as direct dependencies, not optional ones. These are already in the fork and carry no Python semantic risk.

**C. Add `internal_docs/tooling_reuse_strategy.md` to the required documentation list:**

The audit findings and decision matrix must be recorded in this doc for future maintainers.

### Phase 35 doc changes

**D. Extend the split-brain guardrail description:**

In the "split-brain guardrail" references, add a sentence: "The guardrail mechanism created in this phase must additionally guard against Python semantic dependencies (ty_python_semantic, ty_project Python project semantics, Python environment discovery) in tooling and LSP paths as Phase 36 extends the guardrail scope."

This is a one-line addition that makes Phase 35's contribution to the tooling architecture explicit.

---

## Summary

The memo's overall strategy is the right call and the architecture is sound. The specific blocker is that the phase documents don't reflect the audit milestone with concrete scope, and the `lsp-server`/`lsp-types` direct dependency should be explicit rather than treated as ambiguous. The audit spike's design should test the clean-separation assumption against a concrete wire-through example rather than assuming it.

If those four doc changes (A, B, C, D) are made, this is ready to merge. Nothing in the memo's fundamental strategy needs to change - it's about making the phase docs own the audit decision rather than leaving it implicit.
