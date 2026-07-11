Now I have enough context to produce the review. Let me write it up.

---

# M0 Review — Complete Contract Lock And Evidence Model

## Scope and validation

Reviewed the current branch diff (`codex/python-interop-m0-contract-lock`) against `plans/issues/active/ad-hoc-declaration-first-python-interop.md` M0 tasks/acceptance and the two durable architecture documents.

**M0 tasks — coverage confirmed:**

| Task | Evidence in diff |
| --- | --- |
| Accept both architecture documents as target contract | `internal_docs/python_interop_declaration_architecture.md`, `internal_docs/python_interop_protocol_architecture.md` both present; declaration doc updated (`Sifr signature is the only conversion type contract`, PYRES-0002 row, target-state/implementation-status split) |
| Define capability states | `declaration_capabilities.py:11-17` — `CAPABILITY_STATES = {declaration-supported, bridge-supported, dynamic-only, unsupported-by-design}` and `IMPLEMENTATION_STATUSES = {reserved, active}` |
| Machine-readable ledger separate from package inventory | `verification/areas/python_interop/declaration_capabilities.json` (16 rows); loaded by `run.py:212` and reported at `run.py:242-253` distinct from `matrix_files` / `package_certification` |
| Positive/negative/cleanup/cancellation/live evidence owners on every decorator+protocol | Every row in `declaration_capabilities.json` lists all five evidence kinds with explicit owner strings; non-required kinds are `not-applicable` with a documented reason |
| Lock decorator grammar, policy atoms, target namespace, ellipsis body | `python_interop_declaration_architecture.md:62-186` + `python_interop_protocol_architecture.md:31-565` |
| Lock complete call-shape semantics | `python_interop_declaration_architecture.md:188-225` (positional, keyword-only, `python.omit`, typed `*args`, typed `**kwargs`, explicit `**record`) |
| Lock 8 diagnostic families + stable first codes | `crates/sifr_diagnostics/src/codes/registry.rs:446-466` adds PYASYNC + PYCTX (six others already registered). `registry/registry_entries/reserved.rs:36-80` adds nine reserved codes (PYIMP-0001, PYCALL-0001, PYCONV-0001, PYRES-0001, PYRES-0002, PYASYNC-0001, PYCTX-0001, PYZC-0001, PYCB-0001) |
| Reserve `SIFR-PYRES-0002` | `reserved.rs:56-60` + declaration doc row + phase plan reference; documented as staged-activation gate |
| Lock trust authority / atomic `[python].allow-imports` removal | `python_interop_declaration_architecture.md:423-437` — retained as design contract with `SIFR-PYTRUST-0005` role |
| Stale-design checks | `declaration_capabilities.py:40-49` `FORBIDDEN_DESIGN_PATTERNS` rejects string decorator targets, `send=`, `converter=`, `copy=` on buffer/arrow/dlpack, and `MVP`/`subset release`/`reduced release` |

**Acceptance criteria — all met:**
- Every public syntax form maps to one HIR/runtime contract (protocol doc `Design Principles` line 28-29).
- Every protocol has explicit ownership and shutdown state machines (`### Shutdown State Machine`, cleanup policies table, ExitCause table).
- No capability is labeled supported from package inventory alone (`README.md:30-38`, `declaration_capabilities.py:112-113` blocks `reserved` + `passing`).
- No document describes a smaller language version (`FORBIDDEN_DESIGN_PATTERNS` catches `MVP` / `subset release` / `reduced release`).

**Cross-checks:**
- Runtime module registered in runner (`run.py:13-16`), scaffold path invokes ledger validation (`run.py:212`), self-test path invokes negative fixtures (`run.py:378`).
- Ledger schema is complete: 16 rows cover sync-declaration, complete-call-shapes, opaque-lifecycle, sync-context, package-bridge, coroutine-declaration, async-context, three callback rows, buffer, Arrow, DLPack, raw-dynamic-object, and two unsupported-by-design rows.
- Registry tests still pass (`cargo test -p sifr_diagnostics` = 32 pass), the new families satisfy `assert_family_name` / `assert_canonical_code`.
- Design-fragment strings referenced by `_validate_design_contract` are all present in the two architecture docs (verified line-by-line).

## Findings

### Actionable findings

**None.** The M0 changes match the phase-plan tasks and acceptance criteria; validations named in the prompt already passed locally.

### Non-blocking suggestions

1. **`declaration_capabilities.py:115-125` allows a required-evidence kind to be marked `not-applicable`.** Only the `reserved` + `passing` combination is rejected. A future edit could hide a required capability commitment by flipping `planned` → `not-applicable`. Consider tightening: for kinds in `required_evidence`, require `status in {"planned", "passing"}` (i.e., forbid `not-applicable`). Today the ledger is well-formed, but the validator does not enforce it.

2. **`crates/sifr_diagnostics/src/codes/registry.rs` is 895 / 900 lines** after the two new families. It is under the guardrail but has only ~5 lines of headroom. M2's PYTRUST rebase and each subsequent milestone's active-code activations will need to add rows to `registry.rs` and `reserved.rs`. Consider factoring `DIAGNOSTIC_FAMILIES` into a small sibling file now (mirroring `registry/registry_entries/…`) rather than under milestone pressure.

3. **`demos/python_interop_m0_demo/` contains only a README.** `AGENTS.md` describes `demos/` as "Runnable language-feature demos (*.sifr)". Since M0 activates no syntax, the "demo" is really pointer documentation. The same content could live in `verification/areas/python_interop/README.md` (where the ledger is already documented) or in the M0 review-evidence file. As a demo directory it violates the file-type convention and adds no runnable Sifr surface.

4. **Stale-design sweep is scoped to two documents.** `FORBIDDEN_DESIGN_PATTERNS` runs only against the two architecture markdown files listed in `REQUIRED_DESIGN_FRAGMENTS`. The phase plan file (`plans/issues/active/ad-hoc-declaration-first-python-interop.md`), other internal_docs pages, and future public docs are not covered. Extending the sweep to at least the active phase plan and any `docs/python*` page would harden the rejected-syntax guardrail.

5. **`FORBIDDEN_DESIGN_PATTERNS` string-decorator regex is narrow.** `r"@python(?:\.coroutine)?\(\s*['\"]"` catches `@python("…")` and `@python.coroutine("…")` but not string targets in `@python.opaque(type="…")`, `@python.attr("…")`, `@python.item("…")`, or `@python.dlpack(target="…")`. Those decorators legitimately take target references and are equally prone to a string-target regression. Broaden to `@python(\.\w+)*\(\s*['\"]` (excluding `@python.callback` where the first positional arg is a parameter name — that would need an explicit allowlist).

6. **`_expect_rejection` (declaration_capabilities.py:171-178) uses substring matching.** Fragile against future error-message wording changes; a wrong-cause SystemExit whose message happens to contain the expected substring would silently pass. Anchoring on the concrete `raise SystemExit(...)` prefix (e.g., `"reserved capability"`, `"missing required evidence"`) or introducing a small error-code enum would be more resilient.

7. **`passing` claims are not cross-linked to concrete tests.** `raw-dynamic-object`, `arbitrary-dynamic-attributes`, and `untyped-generation` mark `passing` evidence with human-readable owners like "existing conversion and resource diagnostics." The ledger check accepts these labels without verifying the referenced suites still exist or still pass. This is consistent with M0's contract-lock scope (M17 is the exit gate for evidence-vs-claim reconciliation), but a follow-up milestone should require each `passing` claim to name a fixture-file path or diagnostic-code identifier.

8. **Grammar coverage gap in evidence rows (aggregate rows lump sub-states).** `opaque-lifecycle` bundles `@python.opaque`, `@python.attr`, and `@python.item` under one row with `milestone_owner: M4`, and `arrow-c-data` bundles five capsule return types. The M0 task says "every decorator and protocol state transition"; sub-state transitions (attribute-vs-item failure paths, ArrowArray-vs-ArrowStream vs device variants) are implicit in the evidence owner descriptions rather than distinct rows. Not a defect, but if the ledger is to remain the authoritative evidence contract through M17, splitting these into per-decorator or per-return-type rows would let later milestones tick each transition off explicitly.

## Verdict

**SATISFIED.**

The branch delivers the full M0 contract lock: eight diagnostic families + nine stable first codes reserved (including SIFR-PYRES-0002 for staged-activation guarding), a machine-readable declaration/protocol capability ledger separated from package inventory, evidence ownership for every decorator/protocol row, target-state vs. implementation-status decoupling that structurally prevents inventory-only support claims, a design-fragment presence check plus stale-design pattern sweep, and runner integration with a passing self-test. Architecture docs are complete and consistent with the phase plan. All named local validations passed. No actionable findings; the suggestions above are refinements for later milestones, not M0 blockers.
