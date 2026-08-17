# Phase 41: Native Pydantic-Sifr

## Status

Superseded as an implementation plan by the
[`Native Pydantic-Sifr ad hoc phase`](../issues/archive/ad-hoc-native-pydantic-sifr-architecture.md).
The durable design is
[`native_pydantic_sifr_architecture.md`](../../internal_docs/native_pydantic_sifr_architecture.md).

The former draft duplicated typed models, validation, serialization, errors,
and Pydantic conformance inside the Sifr repository. The accepted architecture
now delivers the general compiler/sysroot prerequisites in `sifr-lang/sifr`
and all Pydantic-specific package/core implementation in the standalone
`sifr-lang/pydantic-sifr` repository.

## Depends on

- Phase 40 compiler, package, and ecosystem foundations released.

## Entry Criteria

- Phase 40 is released.
- The canonical ad hoc architecture has passed independent review and
  `milestone_ps_0` is approved.
- No Pydantic-specific implementation has started in `sifr-lang/sifr`.

## Dependency handoff

Phase 42 depends on the released public model, validation, serialization, and
error contract certified through `milestone_ps_11` of the canonical ad hoc plan.
It must not introduce a second validator, serializer, or model contract.

The former `Serialize`/`Deserialize` derives and stdlib `dumps`/`loads`
deliverable are intentionally subsumed by the single
`TypeAdapter[T]`/`BaseModel` Core Schema path. They are not an independently
deliverable compiler or stdlib serialization API. `sifr-lang/sifr` continues
to own only the general `JsonValue` JSON surface; typed model JSON belongs
exclusively to the external package. Phase 42 deliberately waits for that
external package's certified release and has no fallback. No milestone from
the superseded draft is independently executable.

## Quality Contract

- Phase 27 runtime-safety, deterministic-diagnostic, and no-user-panic
  invariants remain mandatory.
- Every canonical ad hoc milestone defines its own implementation checklist,
  focused conformance families, and exit gate.
- Work is sequential: implement and validate one milestone, open and review
  its PR, merge and release when required, update tracking, then start the
  next milestone.
- Compiler-repository PRs run `scripts/run_all_tests.sh --profile create-pr`;
  merge readiness requires `scripts/run_all_tests.sh`.
- External-repository milestones establish equivalent checked-in local gates
  before implementation and must pass them before review.

Milestone quality and exit checks are defined by the canonical ad hoc issue;
this redirect cannot be used to bypass one.

## Exit Gate

- Every canonical milestone through `milestone_ps_11` is reviewed, merged,
  linked from the ad hoc issue, and released where required.
- The compiler repository contains only package-neutral substrate and passing
  conformance evidence.
- The external `sifr-lang/pydantic-sifr` repository contains the sole typed
  model validation/serialization implementation and its canonical demo.
- Both repositories pass their authoritative local validation gates, including
  Phase 27 safety and diagnostic invariants.
