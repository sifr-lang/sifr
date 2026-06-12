# Phase 41: Web Framework and Platform Expansion

> Note: Needs more planning before execution (which fastapi subset to target, scope boundaries, parity target depth, and acceptance gates are still draft-level).

## Objective
Deliver the web framework with typed extractors, then land platform expansion tracks (data/ML and interoperability) under a single bounded feature phase.

## Depends on
- Phase 40
- Phase 32
- Ad Hoc Production Network and HTTP Platform Substrate for `sifr.net`/`sifr.tls`/`sifr.url`/`sifr.http` protocol substrate, as summarized in [`../network_http_architecture.md`](../network_http_architecture.md). Multi-core serving throughput remains owned by the substrate phase's serving-scale follow-up.

## Milestones

### milestone_41_1: Web Framework Core
- Scope:
  - Routing, middleware, lifecycle/shutdown, and base request/response pipeline.
  - Build on the `sifr.http` protocol substrate and M4 transport handoff; do not expose `sifr.http_transport` or CPython-shaped `http.server`/`socketserver` APIs.
- Definition of done:
  - Core web scaffolding is stable and test-covered.

### milestone_41_2: Typed Extractors and Request Validation
- Scope:
  - `Json`/`Path`/`Query`/`Form` extractor behavior.
  - Validation and error mapping via Phase 40 model contract.
  - Multipart/form parsing remains outside the network substrate and must be accepted here or in the HTTP client phase before use.
- Definition of done:
  - Extractors enforce typed validation with consistent error responses.

### milestone_41_3: Production Web Baseline
- Scope:
  - Logging/tracing, config, and operational hooks for production readiness.
- Definition of done:
  - Web stack production baseline is documented and smoke-covered.

### milestone_41_4: Data/ML Track (Scoped)
- Scope:
  - Initial data processing and ML inference workflows on top of web/model foundations.
- Definition of done:
  - Data/ML MVP workflows are validated with regression coverage.

### milestone_41_5: Interoperability Track (Scoped)
- Scope:
  - Initial FFI/interoperability boundary model and safety constraints.
- Definition of done:
  - Interop MVP workflows are documented, test-covered, and quality-gated.

## Quality Contract
- Entry criteria: Phase 40 is completed and typed data model contracts are stable.
- Phase 27 non-regression baseline is required at phase start and must remain green through completion.
- Phase 27 non-regression invariants that must hold in this phase include: no user-triggerable panic paths; no data-dependent emitted `.unwrap()` / `.expect()` / `panic!` in user runtime paths; stable diagnostic contract (codes, severity, spans, URLs, suggestions, schema); canonical/lossless `json` diagnostics with `human` and `compact` as renderer views only; enforced recovery limits with deterministic ordering; and enforced exit-code and CLI stability contracts (`0/1/2/3`, and unknown `--diagnostic-format` exits `2` before semantic work).
- Any milestone that regresses these invariants is incomplete, even if its local scope passes.
- Exit criteria: Web, data/ML, and interoperability MVP tracks are delivered without violating reliability/stability contracts.
- Milestone quality checks:
  - No fallback, migration, or legacy compatibility code is allowed; implement the canonical architecture directly with clean code only.
  - No lazy or partial fixes are allowed; each milestone must resolve root causes completely, even when that requires significant rework.
  - All implementations must be production-grade compiler code: strict typing, deterministic behavior, explicit invariants, and unforgiving correctness standards, with architecture cleaned up toward the target design.
  - Every milestone in this phase must satisfy the scope and definition-of-done already documented in this file.
  - Validation evidence must be recorded in the phase execution checklist issue before merge.
  - Validation evidence for every milestone must include at least one positive-path case and one negative-path case mapped to the milestone validation planning goals.
- Validation planning goals:
  - `milestone_41_1` (Web Framework Core): validation goals cover: Routing, middleware, lifecycle/shutdown, and base request/response pipeline. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_41_2` (Typed Extractors and Request Validation): validation goals cover: `Json`/`Path`/`Query`/`Form` extractor behavior; Validation and error mapping via Phase 40 model contract. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_41_3` (Production Web Baseline): validation goals cover: Logging/tracing, config, and operational hooks for production readiness. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_41_4` (Data/ML Track (Scoped)): validation goals cover: Initial data processing and ML inference workflows on top of web/model foundations. Include negative-path goals that catch regressions against these guarantees.
  - `milestone_41_5` (Interoperability Track (Scoped)): validation goals cover: Initial FFI/interoperability boundary model and safety constraints. Include negative-path goals that catch regressions against these guarantees.
  - Exit-gate evidence explicitly demonstrates: Web, data/ML, and interoperability MVP tracks are delivered without violating reliability/stability contracts.

## Exit Gate
- Web, data/ML, and interoperability MVP tracks are delivered without violating reliability/stability contracts.
- Phase 27 non-regression contract remains green: panic-free user paths, no emitted data-dependent unwrap/expect/panic, and stable diagnostics/renderer/exit-code behavior.
