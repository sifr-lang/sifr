I have enough context. Let me finalize the round-2 review.

# M0 Review Round 2 — Complete Contract Lock And Evidence Model

## Scope

Re-reviewed the current branch diff (`codex/python-interop-m0-contract-lock`, all uncommitted) with the three adopted round-1 refinements in focus, and cross-checked them against `declaration_capabilities.json`, both architecture docs, the reserved diagnostic codes/families, and the runner integration.

## Verification of the three adopted refinements

**1. Required evidence may not be `not-applicable` — `verification/areas/python_interop/runner/declaration_capabilities.py:129-137`.**
The check is stronger than round-1 suggested: it rejects `required + not-applicable` (line 129-133) *and* symmetrically rejects `non-required + status != not-applicable` (line 134-137). All 16 rows in `declaration_capabilities.json` satisfy both directions:
- Every required kind has status `planned` or `passing`.
- Every non-required kind carries status `not-applicable` with a documented owner reason.

Negative self-test at line 189-191 mutates `capabilities[0].evidence[0]` (positive/required on `sync-declaration`) to `not-applicable` and asserts the "cannot be not-applicable" rejection fires. Confirmed by simulation.

**2. Design sweep includes the active phase plan — `declaration_capabilities.py:39-42, 151-167`.**
`DESIGN_SWEEP_PATHS = (*REQUIRED_DESIGN_FRAGMENTS, "plans/issues/active/ad-hoc-declaration-first-python-interop.md")` unpacks the two dict keys plus the phase plan. Line 157-161 explicitly skips the `reduced-version term` pattern for `plans/issues/` so `plans/issues/active/…:751` ("not a reduced release" — the historical review-checklist item) does not trip. Ran the four other patterns manually across all three files: zero hits.

**3. String-target rejection broadened — `declaration_capabilities.py:44-49`.**
`r"@python(?:\.[a-z_]+)*\(\s*['\"]"` catches `@python(...)`, `@python.coroutine(...)`, `@python.attr/.item/.callback/.buffer/.arrow/.dlpack/.context.enter/.aenter/.exit/.aexit/.dlpack.stream(...)` positional string targets. The `@python.opaque(type="...")` case that the first regex misses (opaque's first arg is `type=`, not the string) is covered by the second regex `r"@python\.opaque\([^\n)]*\btype\s*=\s*['\"]"`. Legitimate non-string callable arguments (`@python.callback(handler, …)`, `parameter(name)` in `stream=`) do not trip because the char after `(\s*` is an identifier, not a quote.

## Cross-checks

- **Fragment presence** — every fragment in `REQUIRED_DESIGN_FRAGMENTS` is present in the target file (verified by search): declaration doc has `is the only conversion type contract.` at line 75, `SIFR-PYRES-0002` at 564; protocol doc has the four headings and the "async Python declaration owns exactly one" phrase.
- **Diagnostic families/codes** — `registry.rs:446-455` adds `PYASYNC` + `PYCTX`; `reserved.rs:17-18, 36-80` reserves 9 first codes including `SIFR-PYRES-0002` (staged-activation gate). Names/codes match the declaration doc's line 549-568 and phase-plan tasks. Indentation of new `reserved_code(…)` entries matches the pre-existing `SIFR-INT-0002…0010` block.
- **Runner integration** — `run.py:212` loads the ledger under `scaffold`, `run.py:242-253` reports counts distinct from `matrix_files` / `package_certification`; `run.py:377-378` invokes the four self-tests under `--self-test`.
- **State-name consistency** — `declaration-supported | bridge-supported | dynamic-only | unsupported-by-design` is consistent across declaration doc (line 575-578), phase plan (line 135-136), README (line 32-33), ledger, and validator constants.
- **File-size guardrail** — `registry.rs` = 895/900; `declaration_capabilities.py` = 201; `declaration_capabilities.json` = 246. All within cap.
- **All named validations already passing** per the prompt.

## Findings

### Actionable findings

**None.** The three adopted refinements are correctly implemented, the ledger and validator agree, all fragment/pattern checks are internally consistent, and no regression was introduced against the round-1 baseline.

### Material non-blocking suggestions

1. **`declaration_capabilities.py:120` does not reject `active + planned`.** The validator forbids `reserved + passing`, but there is no complementary check that an `active` row's required evidence is at `passing` (or at least, not `planned`). All three current `active` rows happen to comply, but the design intent per `declaration_capabilities.json:3` ("only active rows with passing required evidence are implemented claims") is not enforced by code. Consider tightening in M2 or later: `if implementation == "active" and kind in required_kinds and status == "planned": reject`.

2. **Self-tests reference `capabilities[0]` by index (`declaration_capabilities.py:177, 190`).** Both `unsupported_claim` and `inapplicable_required` rely on capabilities[0] being reserved *and* its evidence[0] being a required kind — currently `sync-declaration` + `positive` — but nothing anchors that. A JSON reorder in a later milestone could silently make one or both tests no-op (the "invalid" mutation would produce a still-valid matrix). Prefer selecting by attribute (`next(row for row in matrix["capabilities"] if row["implementation_status"] == "reserved" and row["evidence"][0]["kind"] in row["required_evidence"])`) as done for `missing_cleanup` at line 181-186.

3. **`crates/sifr_diagnostics/src/codes/registry.rs` is 895/900 lines** after the two new families (unchanged from round 1). M2's PYTRUST rebase and later milestones' active-code activations will add rows here and in `reserved.rs`. Factoring `DIAGNOSTIC_FAMILIES` into a sibling file now is easier than under M2 pressure.

4. **`demos/python_interop_m0_demo/` still contains only a README** (unchanged from round 1). `AGENTS.md` describes `demos/` as "Runnable language-feature demos (*.sifr)"; a pointer-only directory reads as violating that convention. The same content could live in `verification/areas/python_interop/README.md` or in an M0 exit-evidence note.

## Verdict

**SATISFIED.**

The three adopted refinements land correctly with no regressions: required evidence can no longer be `not-applicable` (validator + negative self-test); the stale-design sweep now covers the active phase plan with a scoped exception for the historically-referenced "reduced release" term; and string-target rejection covers every `@python.*` immediate positional target plus `@python.opaque(type="…")`. Diagnostic families/codes, ledger classifications, architecture-doc fragments, runner integration, state-name usage, and file-size guardrails are all consistent. M0 is ready for PR. The four non-blocking suggestions above are refinements for later milestones, not M0 blockers.
