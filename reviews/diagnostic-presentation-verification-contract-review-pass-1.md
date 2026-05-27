

## Verification Contract Review

**SATISFIED** (with required precision edits)

---

### Blockers: None

The user correctly addressed the earlier gap: the phase now makes verification a first-class deliverable with a named mechanical gate. The verification contract is coherent and appropriate.

---

### Required Precision Edits

**1. Phase plan, Verification Contract section — mention existing baseline and its mode coverage**

The verification gate lists all required coverage elements, but the phase plan only references `decimal_invalid_literal` once (in Current Behavior). The plan does not enumerate that this existing fixture covers single-line span diagnostics in all three modes. This matters because implementers need to know what's already there vs. what M1 must create.

Add to the Verification Contract section or the M1 list:

> The existing `crates/sifr/tests/verification/diagnostics/decimal_invalid_literal` fixture provides locked single-line span baselines for `human`, `compact`, and `json`. M1 must create a new multiline span fixture and a JSON schema-lock fixture on top of this foundation.

**2. Execution tracker, Validation Log — add a forward reference for the tool's future existence**

The M1 validation entry says:
> M1 validation must include `python3 verification/tooling/check_diagnostic_presentation_contract.py`, ...

This is correct but doesn't record that the tool does not yet exist and is part of M1's deliverable. Add to the Validation Log:

> The `check_diagnostic_presentation_contract.py` tool does not exist yet. It is a M1 deliverable, not a pre-existing guardrail. M1 must author the tool, prove it fails in its initial state, and then wire it into the quick lane.

---

### Rationale

The verification contract is well-structured:

1. **Concrete enough**: Names the exact tool, command-line interface, and `--self-test` mode. Establishes negative self-tests as required.

2. **Enforceable**: Lists concrete checks: fixture existence, baseline per-mode coverage, JSON schema field enumeration, CLI regression coverage, and docs linkage. None of these are ambiguous.

3. **Appropriately wired**: Requires wiring into `scripts/run_all_tests.sh --profile quick` in M1. The quick lane already runs all tooling checks; adding the new tool there is a single-line addition.

4. **Covers important contracts**:
   - **human**: file/line/column, snippet, highlight marker, docs URL, spanless fallback, related spans, child notes/help, suggestions, CRLF safety
   - **compact**: severity-only summary, one line per retained diagnostic, stable first fields, no snippets/URLs by default, no old `CompactKey` grouping
   - **json**: exact field enumeration: `code`, `severity`, `message`, `message_template`, `args`, `url`, `spans`, `children`, `help`, `suggestions`
   - **CLI**: regression coverage for `check`, `build`, `run`, `emit` format selection
   - **negative self-tests**: proves the checker fails when requirements are missing

5. **Integration**: Execution tracker W-13 maps correctly to the gate. Review log records the user's verification request and the response. The gate remains active through phase closure per the plan.

The two precision edits above are not blockers—they are documentation hygiene so future readers understand that the tool and multiline fixture are *part of* M1 deliverables, not pre-existing scaffolding.
