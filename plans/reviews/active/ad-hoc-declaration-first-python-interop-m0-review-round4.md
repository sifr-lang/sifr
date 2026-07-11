## M0 Review Round 4 — Post authoritative-gate corrections

### Verification of the five checklisted concerns

**1. Durable activation owners satisfy M0's evidence-owner requirement — PASS**

`verification/areas/python_interop/declaration_capabilities.json` retains, for all 16 rows, per-evidence-kind owner strings across `positive/negative/cleanup/cancellation/live`. Row-level `activation_owner` is a separate, durable kebab-case slug enforced by `declaration_capabilities.py:95` (`re.fullmatch(r"[a-z][a-z0-9-]+", owner)`). No slug encodes delivery order; each names the durable subsystem. The M0 task ("assign … evidence owners to every decorator and protocol state transition") is met by the per-kind owner fields, not by activation_owner — so replacing M-labels with subsystem slugs did not remove any evidence commitment.

**2. Phase-plan sequencing remains unambiguous — PASS**

Distinct activation_owner slugs in the JSON (13 unique: `sync-declarations, opaque-lifecycle, sync-context, package-bridge, async-runtime, async-context, callback-runtime, buffer-protocol, arrow-c-data, dlpack-transfer, raw-api, static-language, binding-authoring`) map 1:1 to the plan's Delivery Rule (`plans/issues/active/…M0.md:117–128`) and its section titles (M3–M12, M14, M16). The plans directory remains the sole sequencing authority (47 `M#` occurrences confined there); no ambiguity introduced.

**3. Taxonomy corrections do not weaken capability/evidence guarantees — PASS**

All prior guards remain in `runner/declaration_capabilities.py`: (a) enum-restricted target/implementation/kind/status sets; (b) reserved-cannot-pass (`:117–118`); (c) active-must-pass (`:131–138`); (d) required-cannot-be-NA (`:127–130`); (e) non-required-must-be-NA (`:139–142`); (f) required-set-must-be-covered (`:120–125`); (g) forbidden-design patterns (string targets, `send=`, `converter=`, hidden `copy=`, reduced-version terms). New guard added on top: activation_owner regex shape (`:95`). Guarantees are strengthened, not weakened. `cargo test -p sifr_diagnostics` = 32/32 pass. Self-tests round-trip clean (verified locally: `rows: 16` reported).

**4. Generated diagnostic docs are correct — PASS**

`crates/sifr_diagnostics/src/codes/registry.rs` now registers `PYASYNC` (`:446–450`) and `PYCTX` (`:451–455`) family bases; `registry_entries/reserved.rs` adds their `-0000` bases (`:17–18`) plus nine stable first codes `SIFR-{PYIMP,PYCALL,PYCONV,PYRES-0001,PYRES-0002,PYASYNC,PYCTX,PYZC,PYCB}-0001` (`:36–80`). Both `docs/errors/diagnostic-codes.md` and `internal_docs/diagnostic_codes.md` list the two new families and all nine reserved rows in the same order with matching descriptions. Registry stays at 895/900 lines; file-size guardrail passes.

**5. No stale delivery labels remain in tracked active roots — PASS (with one local-workspace nit)**

Grep for `\bM[0-9]+\b|milestone` across the six affected files returns zero. Grep for `MVP|subset release|reduced release` across the same set returns zero. `python3 verification/areas/coverage_matrix/checks/verification_taxonomy.py` exits 0. Durable `SIFR-PYRES-0002` doc rows describe activation as "sole production lowering is not active yet" — durable phrasing.

### Findings

**Actionable findings: none.**

**Non-blocking observation** (unchanged from round 3): `crates/sifr_diagnostics/src/codes/registry.rs` sits at 895/900. Factoring `DIAGNOSTIC_FAMILIES` into a sibling file before further family additions remains a good idea, but not an M0 blocker.

**Local-workspace nit (does not ship):** `demos/python_interop_m0_demo/` is an empty directory left over from the round-4 rename to `demos/python_interop_contract_demo/`. Its name contains the token `m0`. It is untracked (git does not track empty dirs), does not appear in the PR diff, and does not trigger `verification_taxonomy.FILENAME_PATTERNS` because no files are walked inside it. However, it is a stale delivery label present in the working tree under an active root; a plain `rmdir demos/python_interop_m0_demo` would remove the last local trace of the pre-rename state. Not a PR-readiness blocker.

### Cross-checks

- `git diff` for the two architecture docs only softens M-number references to durable phrasing ("Ordered changes activate", "declaration contract reserves the first diagnostic codes") — no capability, ownership, or state-machine content is weakened.
- Verification README's new `declaration_capabilities.json` paragraph and the contract demo README both describe target/implementation/activation ownership and evidence coverage in durable vocabulary.
- Delivery Rule in the plan (M3→M12 mapping to declaration forms) is preserved and remains authoritative sequencing; the JSON is silent on ordering, as it must be.

### Verdict

**SATISFIED.** M0 is PR-ready.
