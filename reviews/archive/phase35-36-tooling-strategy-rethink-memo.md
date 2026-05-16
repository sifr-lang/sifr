# Phase 35/36 Tooling Strategy Rethink Memo

## Question

Can Sifr get to production-grade editor tooling faster and with less custom work by using `ty`, Ruff, or another existing toolchain component more aggressively, without creating an unsustainable split-brain compiler/tooling architecture?

## Current local facts

- Sifr already carries a Sifr Ruff fork under `third_party/ruff`.
- The workspace already depends on Sifr-aliased Ruff parser/AST crates (`sifr_python_ast`, `sifr_python_parser`) for syntax.
- The fork includes `ty_server`, `ty_ide`, `ty_project`, `ty_python_semantic`, `ruff_server`, `ruff_db`, `lsp-server`, `lsp-types`, and `salsa`-based infrastructure.
- `ty_server` has useful production-shaped LSP concerns: initialization, capability negotiation, session state, document indexing, request queues, configuration, diagnostics lifecycle, and cancellation/scheduling machinery.
- `ty_ide` has useful editor-query shape: completion, hover, goto definition/declaration/type-definition, references, rename, document symbols, workspace symbols, semantic tokens, inlay hints, code actions, folding ranges, and signature help.
- `ty_project` has useful project/session machinery, but it is deeply Python-specific: Python environments, Python module resolution, Python source types, Python settings, and Python semantic database dependencies.
- `ty_ide` editor queries are deeply Python-semantic. For example, completion calls `ty_python_semantic::SemanticModel`, Python AST/token types, Python module resolution, and Python import logic.
- Official current docs describe `ty` as an extremely fast Python type checker and language server, with `ty server` as its language server command. They also say ty is beta and its API/diagnostics are not stable yet.
- Official Ruff docs describe `ruff server` as a Rust LSP backend for Ruff diagnostics, fixes, and formatting, intended to be used alongside another Python language server for navigation and autocompletion.

## Non-negotiable product requirements

- Sifr semantics must remain Sifr semantics: static typing, explicit `Any`, `Result`/`Option`, ownership/move/mutability rules, Rust codegen obligations, no user-triggerable runtime panics.
- The editor must not tell users one semantic story while `sifr check` or `sifr build` tells another.
- Future VS Code, Neovim, Zed, Helix, Emacs, formatter, linter, automation, and generated-Rust-preview surfaces must share one compiler/tooling brain.
- Basic syntax and protocol plumbing can be reused. Sifr semantic answers cannot be delegated to Python language tooling.

## Options

### Option A: Build Sifr tooling from scratch over `lsp-server`

Sifr implements `sifr_syntax`, `sifr_frontend`, `sifr_analysis`, and `sifr_lsp` directly. It uses `lsp-server`, `lsp-types`, and maybe Salsa, but does not copy/adapt `ty_server` or `ty_ide` internals.

Pros:

- Cleanest ownership boundary.
- Lowest risk of Python semantic leakage.
- Easier to reason about long-term.

Cons:

- Duplicates a lot of solved LSP/session/query-shell work already present in the fork.
- Slower to reach polished editor behavior.
- More likely to miss protocol edge cases that `ty_server` has already solved.

Verdict: safe, but probably too conservative given the local assets.

### Option B: Use `ty` or `ruff server` as the Sifr language server

Sifr launches or embeds `ty server`, `ruff server`, Pyright, or another Python-oriented server as the main editor intelligence backend.

Pros:

- Fastest apparent startup path.
- Mature Python editor UX in some areas.

Cons:

- Wrong semantic authority.
- Python imports/types/exceptions/narrowing/runtime assumptions do not match Sifr.
- `ruff server` intentionally does not own navigation/completion.
- `ty` is beta and its public API/diagnostic behavior is not stable.
- Would fail the core Sifr guarantee: compiler/editor consistency.

Verdict: reject.

### Option C: Fork `ty` wholesale and replace semantics over time

Sifr creates a fork of `ty_server`/`ty_ide`/`ty_project` and progressively replaces Python parser/semantic/project layers with Sifr layers.

Pros:

- Reuses the most implementation immediately.
- Could produce a feature-rich server quickly with enough developers.

Cons:

- High semantic-leak risk during migration.
- Large ongoing merge/rebase burden against a beta project.
- Python-specific abstractions may shape Sifr architecture in bad ways.
- The temporary period would likely violate the no-fallback/no-split-brain rule unless tightly quarantined.

Verdict: only acceptable as a throwaway spike, not as the production architecture.

### Option D: Reuse `ty_server` as an LSP shell, replace semantic/project brain immediately

Sifr treats `ty_server` as a code asset for LSP/session/document/request infrastructure. We extract, adapt, or copy only protocol-shell patterns and bind them to `sifr_analysis`. Python project and semantic crates are not in the production dependency path.

`lsp-server` and `lsp-types` are not part of the open question. They should be direct Sifr LSP dependencies because they are generic protocol/data crates with no Python semantics. The open question is how much of `ty_server`'s higher-level session shell can be reused cleanly.

Reusable candidates:

- LSP handshake and capability negotiation structure.
- Position encoding negotiation.
- Request queue and cancellation scheduling ideas.
- Open-document index and document version handling.
- Diagnostic publication lifecycle.
- Protocol smoke-test shape.
- Logging/tracing/settings patterns.
- LSP conversion tests.

Rejected production dependencies:

- `ty_python_semantic`.
- Python module resolution as Sifr module resolution.
- Python environment/typeshed/venv assumptions.
- Python import completion logic.
- Python diagnostic rules as Sifr diagnostics.

Pros:

- Captures much of the boring, hard protocol/session work.
- Keeps Sifr semantics centralized in `sifr_frontend` and `sifr_analysis`.
- Lets unlimited developer capacity parallelize cleanly: one lane audits/adapts shell, another builds Sifr analysis, another builds tests.

Cons:

- Requires disciplined extraction boundaries.
- `ty_server` session and project state are likely coupled through Python-specific `ty_project` and Salsa database types; separation must be proven by spike, not assumed.
- Needs explicit audit before implementation to avoid accidental Python dependencies.

Verdict: best likely path.

### Option E: Reuse `ty_ide` query algorithms directly

Sifr adapts completion/hover/definition/reference/semantic-token modules from `ty_ide`.

Pros:

- Attractive feature velocity.
- Good reference for result ranking, fuzzy matching, semantic token shapes, and import/code-action UX.

Cons:

- The query code is heavily Python AST and Python semantic model based.
- Direct reuse would pull in Python type model assumptions.
- Rewriting the core anyway may cost as much as Sifr-native implementation.

Verdict: use as design/test inspiration and copy only generic utilities after audit, not as direct semantic query implementation.

## Recommended strategy

Adopt a reuse-first, semantics-owned architecture:

```text
Sifr Ruff fork parser/AST/trivia
        |
        v
sifr_syntax
        |
        v
sifr_frontend
        |
        v
sifr_analysis
        |
        v
sifr_lsp
  - uses lsp-server and lsp-types directly
  - may reuse/adapt audited ty_server protocol/session shell patterns
  - must not depend on ty_python_semantic or Python project semantics
```

The correction to the current Phase 35/36 plan is not "use ty as the brain." It is "do not waste developer time rebuilding protocol/session infrastructure until an explicit `ty_server` reuse audit proves which parts are cheap and clean to reuse."

## Unlimited developer capacity changes the plan, but not the architecture

Unlimited developers reduce wall-clock time. They do not reduce maintenance risk from wrong ownership boundaries.

With enough developers, the optimal shape is parallel lanes:

1. `sifr_syntax` extraction lane: stable Sifr wrapper over the Sifr Ruff fork.
2. `sifr_frontend` lane: canonical project/module/source-map/query/cache API.
3. `ty_server` reuse audit lane: prove which LSP shell pieces can be extracted without Python semantics.
4. `sifr_analysis` lane: Sifr-native editor queries over frontend/HIR views.
5. LSP adapter lane: bind audited shell to `sifr_analysis`.
6. VS Code lane: grammar, launcher, settings, extension contract.
7. Verification lane: parity, protocol smoke, split-brain, and performance gates.

These lanes can run concurrently, but the dependency rule remains strict: no production editor semantic answer ships unless it comes from `sifr_analysis` over `sifr_frontend`.

## Proposed doc change

Phase 36 should add a new first milestone:

### `milestone_36_0: Tooling Reuse Audit And Architecture Spike`

Scope:

- Audit `third_party/ruff/crates/ty_server`, `ty_ide`, `ty_project`, and `ruff_server`.
- Classify code into:
  - `reuse-now`: generic protocol/session/document/test shell safe to adapt.
  - `reference-only`: useful design but too coupled to Python semantics.
  - `reject`: Python semantic/project/runtime assumptions.
- Produce `internal_docs/tooling_reuse_strategy.md`.
- Build a short-lived spike that wires a mock or minimal Sifr `AnalysisHost` through the selected LSP shell path.
- The spike must be removed or converted into clean production code before phase exit.
- The production dependency graph must prove no dependency from `sifr_lsp` or `sifr_analysis` to `ty_python_semantic`, Python module resolver semantics, Python environment discovery, or Python diagnostic rules.

Exit criteria:

- A reviewed decision matrix chooses one of:
  - adapt selected `ty_server` shell code,
  - implement a Sifr-native shell using `ty_server` as reference,
  - defer reuse because extraction is not clean.
- The decision is evidence-based, not preference-based.
- The no-split-brain guardrail is updated to cover forbidden Python semantic dependencies.

## Recommended final decision

Do not use `ty` as Sifr's language server. Do not fork `ty` wholesale for production.

Do aggressively mine `ty_server` for LSP/session/protocol architecture, and let the first Phase 36 milestone decide whether to adapt code or simply follow its patterns.

This is the smart path because it minimizes custom work where semantics do not matter, while preserving Sifr's long-term correctness boundary where semantics matter most.
