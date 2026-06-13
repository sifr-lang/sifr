**CHANGES_REQUESTED**

Two concrete issues to fix before this contract is implementation-ready:

---

**Issue 1 — Locked Decision 5 contradicts the milestone_ir_boundary_3 guardrail**

Decision 5 forbids `sifr_stdlib` from depending on `{sifr_lowering, sifr_frontend, sifr_codegen, sifr_driver, CLI}`. The guardrail in milestone 5 enforces a stricter set that also forbids `{sifr_package, sifr_analysis, sifr_lsp}`. An implementor reading the canonical locked decisions could add `sifr_package` to `sifr_stdlib` (a plausible dep for dependency spec rendering) and believe it's spec-compliant — only to have the guardrail script reject it in milestone 5. The M1 validation also references "the forbidden dependency set from this phase" with no single source to resolve against.

Fix: extend Decision 5's forbidden list to include `sifr_package`, `sifr_analysis`, and `sifr_lsp`.

---

**Issue 2 — `sifr_analysis` direct-lowering boundary is unguarded**

Decision 12 explicitly prohibits `sifr_lint` from directly depending on `sifr_lowering` (with a stated transitive-via-`sifr_frontend` exception). `sifr_analysis` is listed alongside lint in milestone_ir_boundary_1 as a consumer to migrate, but receives no equivalent locked decision and no guardrail enforcement in milestone_ir_boundary_3. After the phase exits, `sifr_analysis` can silently regress to a direct lowering dependency without failing any check. Decision 13 implies analysis should go through the frontend facade, but implication is not enforcement.

Fix: add a locked decision for `sifr_analysis` parallel to Decision 12, and add the corresponding rule to the milestone_ir_boundary_3 guardrail scope.

---

Everything from the first pass is correctly addressed. The rest of the contract — milestone ordering, exit gate, dependency manifest centralization, parity tests, binary-size gate, Cargo.lock sweep, and quality contract — is solid.

Review artifact written to `reviews/ad-hoc-stdlib-ir-lowering-boundary-refactor-review-pass-2.md`.
