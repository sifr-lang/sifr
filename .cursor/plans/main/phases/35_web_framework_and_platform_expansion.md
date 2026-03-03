# Phase 35: Web Framework and Platform Expansion

> Note: Needs more planning before execution (which fastapi subset to target, scope boundaries, parity target depth, and acceptance gates are still draft-level).

## Objective
Deliver the web framework with typed extractors, then land platform expansion tracks (data/ML and interoperability) under a single bounded feature phase.

## Depends on
- Phase 34
- Phase 27

## Milestones

### milestone_35_1: Web Framework Core
- Scope:
  - Routing, middleware, lifecycle/shutdown, and base request/response pipeline.
- Definition of done:
  - Core web scaffolding is stable and test-covered.

### milestone_35_2: Typed Extractors and Request Validation
- Scope:
  - `Json`/`Path`/`Query`/`Form` extractor behavior.
  - Validation and error mapping via Phase 33 model contract.
- Definition of done:
  - Extractors enforce typed validation with consistent error responses.

### milestone_35_3: Production Web Baseline
- Scope:
  - Logging/tracing, config, and operational hooks for production readiness.
- Definition of done:
  - Web stack production baseline is documented and smoke-covered.

### milestone_35_4: Data/ML Track (Scoped)
- Scope:
  - Initial data processing and ML inference workflows on top of web/model foundations.
- Definition of done:
  - Data/ML MVP workflows are validated with regression coverage.

### milestone_35_5: Interoperability Track (Scoped)
- Scope:
  - Initial FFI/interoperability boundary model and safety constraints.
- Definition of done:
  - Interop MVP workflows are documented, test-covered, and quality-gated.

## Exit Gate
- Web, data/ML, and interoperability MVP tracks are delivered without violating reliability/stability contracts.
