I've verified the two adopted refinements against the current tree.

# M0 Review Round 3 — Verification of adopted refinements

## Change 1: active rows require passing required evidence

`declaration_capabilities.py:134-141` adds

```
if (
    implementation == "active"
    and item["kind"] in required_kinds
    and item["status"] != "passing"
):
    raise SystemExit(f"active capability {capability_id} requires passing {item['kind']} evidence")
```

Ledger compliance for all three `active` rows:
- `raw-dynamic-object` — required = {positive, negative, cleanup}; all three at `passing`; cancellation/live are non-required and at `not-applicable`.
- `arbitrary-dynamic-attributes` — required = {negative}; `negative: passing`; other four `not-applicable`.
- `untyped-generation` — required = {negative}; `negative: passing`; other four `not-applicable`.

All 13 `reserved` rows still comply with the pre-existing `reserved + passing` rejection (all required kinds are `planned`, all non-required are `not-applicable`).

New negative self-test `incomplete_active` (lines 221-233): picks an `active` row by property, selects a required evidence item by property, mutates its status to `planned`, and expects `"requires passing"`. Rejection is triggered by the new branch above — verified by simulation on `raw-dynamic-object`.

## Change 2: property-based self-test selection

- `unsupported_claim` (lines 185-196) — now selects the reserved row and required item via `next(...)`.
- `inapplicable_required` (lines 207-219) — same property-based selection.
- `missing_cleanup` (lines 198-205) — already property-based (unchanged from round 1).
- `incomplete_active` (new) — property-based.
- `duplicate` (line 181) — still uses `capabilities[0]`, correctly, since duplicate-ID detection is orthogonal to row properties (round-2 suggestion #2 explicitly scoped this to the two tests that depended on row content, not to the duplicate test).

Error-string matching is intact: `unsupported_claim → "cannot claim passing evidence"`, `inapplicable_required → "cannot be not-applicable"`, `incomplete_active → "requires passing"`, `missing_cleanup → "missing required evidence"`, `duplicate → "duplicate declaration capability id"`. Order-of-checks is safe: on a reserved row the `reserved + passing` guard (line 120) fires before any second-pass check; on an active row with a required kind set to `not-applicable`, the `not-applicable` guard (line 130) fires before the new `requires passing` guard, and either message would still correctly reject the mutation.

## Cross-checks (unchanged from round 2)

- Nothing else moved: `registry.rs` (895/900), `reserved.rs`, both architecture docs, README, and `run.py` are identical to the round-2 snapshot.
- Design-sweep + fragment checks still hold; the ledger row order is compatible with `next(...)` selection (first reserved = `sync-declaration`, first active = `raw-dynamic-object`, both have their first `evidence` entry as a required kind, but the property-based selection no longer depends on that anyway).

## Findings

### Actionable findings

**None.**

### Material non-blocking notes

Carried over from round 2 (not regressions, not M0 blockers, called out only so they don't get lost):

1. `crates/sifr_diagnostics/src/codes/registry.rs` still sits at **895/900** lines. Factoring `DIAGNOSTIC_FAMILIES` out before M2's PYTRUST rebase will be easier than under milestone pressure.
2. `demos/python_interop_m0_demo/` remains README-only. `AGENTS.md` describes `demos/` as runnable `*.sifr` demos; consider moving the M0 exit-evidence content into the area README or a phase-plan note.

SATISFIED
