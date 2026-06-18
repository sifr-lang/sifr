Here is my findings after inspecting all requested files against the source.

---

## Findings

### No hard blockers found. One soft flag.

---

### ASYNC pages (0001–0007)

**Per-code examples**: All seven codes now have distinct, per-code erroneous/fixed examples. That Pass 1 requirement is met.

**"Why It Happens" — soft flag on SIFR-ASYNC-0006**: The text is identical across all seven ASYNC files:

> "Sifr tracks suspension and blocking effects statically. This diagnostic fires when async code either does not really suspend, awaits work that cannot suspend, or calls blocking/CPU-heavy work without an explicit offload boundary."

That sentence is accurate for 0001–0005 and 0007, but **factually misleading for 0006**. SIFR-ASYNC-0006 fires because `@blocking_io` (or `@cpu_heavy`) is applied to an `async def`, which is an annotation-placement constraint — nothing to do with whether the code suspends or calls blocking work. A user reading only 0006 gets a wrong mental model of what triggered the error. The fix and code examples are correct, so this won't mislead on remediation, but the diagnostic explanation is off.

This is not a merge blocker (actionable content is accurate), but it is an inaccuracy in a "was previously broken" section. Worth a one-line fix to 0006 if standards require per-code accuracy.

**Fixture references**: All seven fixture paths match what is in the repo.

---

### SIFR-ASYNC-0003 vs. fixture

The doc's erroneous example calls `read_text("config.txt")` directly from async. The fixture (`blocking_io_direct_call_in_async_rejected.sifr`) calls a `@blocking_io`-annotated function `read_blocking()`. These illustrate the same rule from different angles. The doc example is arguably clearer for end users, and the fixture path listed in Details is correct. Not a blocker.

---

### SIFR-ASYNC-0005 vs. fixture

The doc's erroneous code uses `lambda: 42`; the fixture uses a named unannotated function `compute_value`. Both are unannotated, so the concept is accurate. Minor illustrative divergence, not a blocker.

---

### BUILD, INTERNAL, RESULT, TYPE, ENCODING, PACKAGE, WORKSPACE

All "Why It Happens" text is family-specific and accurate. Code fences are correct (`bash` for BUILD, `toml` for PACKAGE/WORKSPACE, `text` for INTERNAL, `python` for user-code families). No issues.

---

### `docs/cli/packages-workspaces.mdx` vs. source

Every command, flag, and constraint documented was verified against `cli_model_and_entrypoint.rs` and `self_update_cli.rs`:

- `--format json` requires `--dry-run` on `sifr self update` — confirmed (`update_args_diagnostic` enforces it).
- `sifr self version --short` cannot combine with `--format json` — confirmed (`version_args_diagnostic` enforces it).
- `--channel` and `--version` are mutually exclusive — confirmed.
- All `--locked`/`--offline`/`--frozen` flags present on all relevant commands — confirmed.

No inaccuracies.

---

### `docs/stdlib/module-index.mdx`

CardGroup is positioned under "## Choosing a Module" near the top of the page, before the table sections. Correct.

---

## Summary

**Wave 3 is PR-ready.** The one soft flag — SIFR-ASYNC-0006's "Why It Happens" text describing suspension/blocking effects when the actual cause is a decorator placement constraint — is a factual inaccuracy in a secondary explanation section. It does not block merge since the title, erroneous code, fix, and fixed code for 0006 are all correct. If the bar for this wave requires accurate per-code "Why" text (which was a Pass 1 stated goal), fix `docs/errors/SIFR-ASYNC-0006.mdx`'s Why It Happens to something like: *"The `@blocking_io` and `@cpu_heavy` annotations can only be applied to synchronous functions. An async function carries its own scheduling contract and cannot be additionally classified as a blocking or CPU-heavy workload."* Otherwise the PR can merge as-is.
