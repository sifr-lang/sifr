# agent Review: Package Ambiguous Import Source Map Boundary

Date: 2026-05-30

Scope: focused review of the package source-map and diagnostic identity decision added to the TypeScript-Go compiler architecture transfer phase.

## Round 1

Reviewer finding: the diagnostic ownership rules are logically correct and non-duplicative, but the closeout gates were too weak.

Actionable gaps:

- Add mechanical coverage for the documented `PackageSourceMap::resolve_import` result states: resolved, ambiguous, unresolved, private access, and fatal package-map failure.
- Require proof that otherwise valid package source maps retain ambiguous candidate sets instead of rejecting or dropping them.
- Require end-to-end runtime fixtures that validate `SIFR-IMPORT-0005` diagnostic output, not only fixture existence.
- Add a negative check that one package import ambiguity cannot emit both `SIFR-IMPORT-0005` and a `SIFR-PACKAGE-*` diagnostic.

Outcome: incorporated into the phase contract through M2 closeout, M17 fixture language, and AC-30/AC-31.

## Round 2

Reviewer finding: AC-29 through AC-31 were present, but the non-duplication invariant was still not tied directly to milestone closeout.

Actionable gaps:

- M2 should prove that otherwise valid package ambiguity does not emit `SIFR-PACKAGE-*` during source-map construction.
- M17 should prove that one runtime ambiguous import emits `SIFR-IMPORT-0005` without a companion `SIFR-PACKAGE-*`.
- M17 should also prove that fatal package-map construction errors emit `SIFR-PACKAGE-*` only, without a companion `SIFR-IMPORT-0005`.

Outcome: incorporated into the M2 and M17 closeout gates.
