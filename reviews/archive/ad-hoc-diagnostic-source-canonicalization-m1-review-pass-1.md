

Based on my comprehensive inspection, here is my review:

---

## M1 Review: Diagnostic Source Canonicalization — Gap Contract Lock

**Verdict: SATISFIED. Ready to proceed to M2.**

### What I checked

1. **Phase requirements alignment** — `issues/ad-hoc-diagnostic-source-canonicalization.md` M1 section correctly specifies all required artifacts and the phase file documents the correct status.

2. **Contract checker** — `verification/tooling/check_diagnostic_source_canonicalization_contract.py`:
   - 480 lines, well-structured with clear sections
   - Static checks: run_all wiring, fixture presence, registry/doc coverage, legacy migration docs
   - Runtime checks: parser spans, project import diagnostics (4 fixtures), package import diagnostics (2 fixtures), package help preservation
   - Self-tests: missing fixture, missing active code, missing legacy docs, spanless diagnostic rejection

3. **Wiring** — `scripts/run_all_tests.sh` lines 127-129 correctly wire both the checker and self-test into the fast lane.

4. **Registry completeness** — All three new codes (SIFR-IMPORT-0005/0006/0007) are present in:
   - `registry.rs`: DiagnosticCode constants + ACTIVE_DIAGNOSTIC_CODES list
   - `parsing_names_and_types.rs`: active_entry! declarations with correct args and owners
   - Docs pages generated and populated
   - `diagnostic-codes.md` index includes all three

5. **Legacy code migration docs** — All four SIFR-WORKSPACE-0101 through 0104 contain "legacy" language and name their canonical replacement.

6. **Fixtures** — All 12 required fixtures exist:
   - 5 parser fixtures: `parser_bad_indent`, `parser_unterminated_string`, `parser_invalid_call_order`, `parser_empty_declaration`, `parser_invalid_declaration`
   - 4 project fixtures: `workspace_missing_import_canonical`, `workspace_ambiguous_import_canonical`, `workspace_namespace_collision_canonical`, `import_cycle_source_spans` (with a.sifr and b.sifr)
   - 3 package fixtures: `package_missing_import_canonical`, `package_ambiguous_import_canonical`, `package_diagnostic_help_preserved`

7. **Runtime behavior verified**: checker FAILS with "diagnostic has no spans" — this is the expected pre-implementation state.

8. **Self-test verified**: checker --self-test PASSES.

### Self-test coverage assessment

The self-test currently checks:
- Missing fixture → failure
- Missing active code → failure
- Missing legacy migration docs → failure
- Spanless diagnostic rejection → failure

By M6 closeout, runtime tests will additionally cover:
- Old-code leakage (forbidden_codes assertion)
- Missing spans (primary_span validation)
- Dropped package help (require_help assertion)
- Missing required JSON args (required_args assertions)

This gives meaningful negative coverage. Not exhaustive (can't test every permutation without test amplification), but sufficient for the contract gate.

### No blockers identified

- All required files are present and correctly structured
- Registry, docs, and fixtures are synchronized
- The runtime contract correctly fails until producers are updated in M2-M5
- Self-test correctly passes

### Proceed to M2

M1 satisfies its purpose: the gap contract is locked, verification tooling exists, and the phase can proceed to implement producers (M2: parser spans, M3: canonical import diagnostics, M4: import cycle source context, M5: package diagnostic context preservation).
