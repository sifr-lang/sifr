**PASS** — with two editorial observations that should be addressed before M0 starts.

---

## Review Findings

### Verdict: PASS

Both docs are implementation-ready under the stated model. The CPython-parity pivot is consistently expressed throughout, no rejected requirements survive into production API scope, and all implementation-blocking decisions are either resolved or properly deferred to `milestone_text_i18n_0`. The execution ledger's remediation checklist is internally coherent with the phase contract.

---

### Observation 1 — `sifr.unicode` Public API Policy is under-specified relative to M2 scope

**File**: `issues/ad-hoc-production-text-i18n-stdlib-parity.md`
**Sections**: "Public API Policy" (`sifr.unicode` list, ~line 118) vs. "milestone_text_i18n_2" scope (~lines 397-407)

The Public API Policy lists `category`, `name`, `lookup`, `numeric_value`, `case_fold`, and the segmentation APIs as the `sifr.unicode` center. M2's implementation scope also adds `bidirectional`, `combining`, `east_asian_width`, `mirrored`, `decomposition`, `decimal`, and `digit` without those properties appearing in the policy section.

Not a contradiction — M2 is the implementation spec and the policy explicitly says "center" (non-exhaustive). But during M0's API-name lockdown or an inter-milestone review, a reviewer could flag those additional properties as not in-policy and require them to be deferred or separately accepted. M0 decision 1 ("exact public API names and type shapes for `sifr.unicode`") already covers this, but the policy section should either list them or note that M2 scope determines the full property surface.

**Remediation**: In the Public API Policy `sifr.unicode` block, either append the remaining properties or add a one-line note: `# M2 scope defines the full property set; see milestone_text_i18n_2`.

---

### Observation 2 — Execution ledger validation baseline omits per-crate test commands that appear in M5

**File**: `issues/ad-hoc-production-text-i18n-stdlib-parity-execution.md`
**Section**: "Validation Evidence" (~lines 162-175)

The ledger's baseline block lists four commands (`cargo fmt --check`, `cargo clippy`, `check_hir_maintainability_guardrails.py`, `run_all_tests.sh --profile create-pr`). M5's validation list in the phase doc also includes `cargo test -p sifr_stdlib`, `cargo test -p sifr -- stdlib`, and `scripts/run_e2e_pass.sh`. These missing commands are likely subsumed by `run_all_tests.sh` internally, so this is non-blocking, but the ledger's baseline will be misleading to anyone running per-milestone validation before M5.

**Remediation**: Add the three commands to the ledger's Required baseline commands block, or add a parenthetical noting they are covered by `run_all_tests.sh`.

---

### Confirmed clean

- No old CPython-parity requirements survive in production scope. `sifr.codecs`, `sifr.encodings`, `sifr.unicodedata`, `sifr.locale`, `sifr.gettext` are consistently held at `deferred-adapter` or `rejected`.
- Bare CPython imports, dynamic codec registry mutation, `setlocale`, `gettext.install`, implicit locale-derived default encodings, `surrogateescape`/`surrogatepass` in normal `str`, and the full `encodings.*` module zoo are all enumerated in Non-Goals with explicit diagnostic/waiver requirements.
- `EncodeOutcome`/`DecodeOutcome` absence from the Public API Policy is not an issue — the policy lists functions, M0 decision 6 locks the recovery-carrying value shape, and M1 scope names them.
- Cross-phase dependency contracts (M1 as the network and concurrency unblock point, M3 as the additional gate for locale-sensitive warning formatting) are consistent between both docs and the dependency graph.
- Milestone dependency order (M0 → M1 → M2/M2.5/M3 parallel → M4 → M5) is unambiguous and correctly sequenced.
- `io.StringIO` carve-out, `io.TextIOWrapper` explicit-encoding-required policy, static-mode-string requirement, and binary I/O smoke-gate before M1 are all correctly stated and consistent between the two files.
