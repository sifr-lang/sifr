SATISFIED

**1. Resolves the medium note?** Yes. The invariant — manifest rows must match a reviewed source of truth — is restored, now anchored on a durable verification-owned reference (`verification/areas/developer_tooling/formatter_rules.md`) instead of the removed plans path. The checker reads `formatter_rules.md` once and asserts every capability name + requirement, every CLI surface + fixture, and every AST id + syntax + required behavior appears in it. Re-ran locally:

- `python3 -m py_compile`: clean
- `--self-test`: PASS (invalid classification still detected)
- positive run: PASS (35 capability, 22 CLI, 11 AST rows all cross-reference; 0 missing)

**2. Blocking issues introduced?** None.

Two non-blocking observations (mention only, do not act):

- Cross-reference uses substring `in rules_text`. Short labels like `Check mode` / `Diff mode` would also match if those tokens appeared elsewhere in the doc; current doc body is a clean reference table so the risk is theoretical. If you ever expand `formatter_rules.md` with prose, consider tightening to per-row line matching.
- `run_self_test` only exercises the invalid-classification path. It does not negatively cover the new "string missing from reference" assertions. Extending self-test to also seed a row whose capability/requirement is absent from a stubbed `rules_text` would lock the new invariant against silent regressions. Not required for this follow-up.
