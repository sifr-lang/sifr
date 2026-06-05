FAIL. Two doc edits required:

---

**Edit 1** — `issues/ad-hoc-production-text-i18n-stdlib-parity.md`, `milestone_text_i18n_1` scope, codec error-handler bullet list.

Remove this line:

> the different timing between compile-time diagnostics for static handler literals and typed runtime errors for adopted dynamic lookup is intentional and must be documented

The phrase "typed runtime errors for **adopted** dynamic lookup" implies a runtime dynamic-handler-lookup path exists in this phase. The two flanking bullets already state clearly that dynamic handler names are unsupported and there is no silent fallback to strict. This line creates a false implication that a partial runtime fallback path should be implemented. Removing it leaves the policy unambiguous.

---

**Edit 2** — `issues/ad-hoc-production-text-i18n-stdlib-parity-execution.md`, planning review remediation list, line 103.

Current:

> [x] Define `open(path)`/`open(path, mode="r")` without explicit `encoding=` as **blocked until the M3 locale/default-encoding contract**, with compile-time diagnostics for static omissions and typed unsupported-default-encoding errors for dynamic cases.

Change to:

> [x] Define `open(path)`/`open(path, mode="r")` without explicit `encoding=` as permanently `unsupported`/`intentional-diff` from CPython's locale-derived default; M3 documents this as the final intentional difference, it does not unblock these forms; compile-time diagnostics for static omissions and typed unsupported-default-encoding errors for dynamic cases.

"Blocked until M3" implies these APIs could be unblocked when M3's locale contract lands. The plan document (same file) states they "remain unsupported after M3" and that M3's job is to **document** the intentional difference, not to implement locale-derived defaults. The execution ledger must match the plan to prevent an implementer from interpreting M3 completion as a signal to add locale-default text-open behavior, which is exactly the deprecated CPython path that must not be implemented.
