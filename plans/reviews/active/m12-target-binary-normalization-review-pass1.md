## Findings (severity-ordered)

### 1. Low — `argv` fields in algorithmic compatibility result variants still hardcode `target/debug/sifr`

**File:** `verification/areas/algorithmic_compatibility/runner.py:360, 394`

The actual invocations in `run_manifest_fixture` and `run_leetcode_full` use the `sifr_bin` returned by `ensure_sifr_bin(DEFAULT_SIFR_BIN)` (which now honors `CARGO_TARGET_DIR`), but the recorded variant argv is a static `["target/debug/sifr", ...]`. Compare with `run_leetcode_check` at line 543 which correctly records `[str(sifr_bin), ...]`.

**Impact:** The JSON result payload advertises a binary path different from the one actually executed. A reader who tries to reproduce a failure by copy-pasting the argv from the result artifact would run the wrong binary when `CARGO_TARGET_DIR` is set, potentially masking the issue or hitting a stale binary. Not a runtime hazard for the current suite because execution is correct; strictly a data-fidelity gap.

Note: `validate_representative_row` at line 310 also asserts the manifest's `command` field is literally `f"target/debug/sifr check {...}"`. That's a manifest schema constraint on the JSON file rather than a live command; leaving it alone is consistent with the "narrow wave" scope.

### 2. Low — `resolve_sifr_binary` does not validate the explicit-env-var path

**File:** `verification/areas/common/sifr_binary.py:18-21`

If `SIFR_RULE_SUPPRESSION_BIN` / `SIFR_STDLIB_MODULE_BIN` is set to a non-existent or non-executable path, the helper returns it verbatim and the downstream `subprocess.run` fails with a cryptic OS-level error rather than the helper's own "Sifr verification binary was not produced" diagnostic. This matches the "explicit override trusts the caller" idiom, but it's a slight friendliness gap versus the target-dir and fallback branches, both of which build-then-verify.

### 3. Low — `--sifr-bin` explicit flag can be silently redirected when it equals the default

**File:** `verification/areas/stdlib_parity/tools/run_stdlib_namespace_corpus_validation.py:143-148`

Because `--sifr-bin` defaults to `str(DEFAULT_SIFR_BIN)`, the code cannot distinguish "user did not pass the flag" from "user explicitly passed `--sifr-bin target/debug/sifr`". In the second case, if `CARGO_TARGET_DIR` is also set, the explicit CLI value is dropped in favor of the CARGO_TARGET_DIR-derived binary. Setting `argparse` default to `None` and testing `if args.sifr_bin is None` would preserve explicit-flag intent. Not exercised by focused validation runs, so not blocking.

### 4. Info — Dead / inconsistent branches in the two `ensure_sifr_bin` shims

- `verification/areas/algorithmic_compatibility/runner.py:552-557` — always called with `DEFAULT_SIFR_BIN`, so the `elif not sifr_bin.exists()` and the trailing `return sifr_bin` are unreachable in current usage.
- `verification/areas/stdlib_parity/tools/run_stdlib_namespace_corpus_validation.py:102-106` — only called in the `!= DEFAULT_SIFR_BIN` branch of `main()`, so the `if sifr_bin == DEFAULT_SIFR_BIN` branch there is dead. Return types also differ between the two (`Path` vs `None`).

Code hygiene only; no runtime consequence.

## Review Question Summary

1. **Shared helper correctness.** `resolve_sifr_binary` honors `explicit_env_var` first, then normalizes `CARGO_TARGET_DIR` (absolute passed through; relative resolved against `repo_root`, matching cargo's own cwd-relative resolution because the helper always runs `cargo build` with `cwd=repo_root`), then falls back to `default_binary` / `target/debug/sifr`. When the configured or fallback binary is missing, it builds (subprocess inherits env, so `CARGO_TARGET_DIR` propagates to cargo). The stale-binary hazard identified in wave 1 is genuinely closed for the CARGO_TARGET_DIR path; there is no silent fallback to `target/debug/sifr` when a configured target dir is set.

2. **All four tools use the configured target binary.** Confirmed: rule/suppression rules, stdlib module parity, namespace corpus validation, and algorithmic-compat representative/full/leetcode-check paths all now go through the shared resolver for their actual Sifr invocations.

3. **Wave scope discipline.** Changes are appropriately narrow. Algorithmic-compat result argv fields (finding 1) were left as-is, which keeps the wave focused; the resulting data-fidelity gap is minor. Namespace corpus `--sifr-bin` semantics have shifted slightly (finding 3), but only in the edge case where a user explicitly re-types the default value while also setting `CARGO_TARGET_DIR`.

4. **Focused validation sufficiency.** The focused runs cover: py_compile, the developer-tooling tool (with and without `--self-test`), stdlib module parity `--scope merge`, the full 411-fixture leetcode namespace corpus run, the algorithmic representative subset via `sifr_verify`, file-size guardrail, and `git diff --check`. Combined with the fact that each affected tool was exercised with a fresh `CARGO_TARGET_DIR` that did not initially contain `debug/sifr` (forcing the build path), the resolver's build-if-missing behavior is directly demonstrated. No additional focused test is required before PR; run the create-pr profile as the merge gate per project convention.

VERDICT: PASS
