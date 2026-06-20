# Phase 42: Web Framework and Platform Expansion

> Note: Needs more planning before execution (which fastapi subset to target, scope boundaries, parity target depth, and acceptance gates are still draft-level).

## Objective
Deliver the web framework with typed extractors and production platform hooks on top of the existing HTTP/runtime substrate.

## Depends on
- Phase 41
- Phase 32
- Ad Hoc Production Network and HTTP Platform Substrate for `sifr.net`/`sifr.tls`/`sifr.url`/`sifr.http` protocol substrate, as summarized in [`network_http_architecture.md`](../../internal_docs/network_http_architecture.md). Multi-core serving throughput remains owned by the substrate phase's serving-scale follow-up.

## Milestones

### milestone_42_1: Web Framework Core
- Scope:
  - Routing, middleware, lifecycle/shutdown, and base request/response pipeline.
  - Build on the `sifr.http` protocol substrate and M4 transport handoff; do not expose `sifr.http_transport` or CPython-shaped `http.server`/`socketserver` APIs.
- Definition of done:
  - Core web scaffolding is stable and test-covered.

### milestone_42_2: Typed Extractors and Request Validation
- Scope:
  - `Json`/`Path`/`Query`/`Form` extractor behavior.
  - Validation and error mapping via Phase 41 model contract.
  - Multipart/form parsing remains outside the network substrate and must be accepted here or in the HTTP client phase before use.
- Definition of done:
  - Extractors enforce typed validation with consistent error responses.

### milestone_42_3: Production Web Baseline
- Scope:
  - Logging/tracing, config, and operational hooks for production readiness.
- Definition of done:
  - Web stack production baseline is documented and smoke-covered.

## Quality Contract
- Entry criteria: Phase 41 is completed and typed data model contracts are stable.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract (codes, severity, spans, URLs, suggestions, schema); canonical/lossless `json` diagnostics with `human` and `compact` as renderer views only; enforced recovery limits with deterministic ordering; and enforced exit-code and CLI stability contracts (`0/1/2/3`, and unknown `--diagnostic-format` exits `2` before semantic work).
- Any milestone that regresses these invariants is incomplete, even if its local scope passes.
- Exit criteria: Web framework and platform expansion paths are delivered without violating reliability/stability contracts.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_42_1` (Web Framework Core): validation goals cover: Routing, middleware, lifecycle/shutdown, and base request/response pipeline. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_42_2` (Typed Extractors and Request Validation): validation goals cover: `Json`/`Path`/`Query`/`Form` extractor behavior; Validation and error mapping via Phase 41 model contract. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_42_3` (Production Web Baseline): validation goals cover: Logging/tracing, config, and operational hooks for production readiness. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Web framework and platform expansion paths are delivered without violating reliability/stability contracts.

## Exit Gate
- Web framework and platform expansion paths are delivered without violating reliability/stability contracts.
- Phase 27 non-regression contract remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic, and stable diagnostics/renderer/exit-code behavior.
