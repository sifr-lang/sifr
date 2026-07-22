# Review: M17 Wave 1 — PR #2997, pass 2 (75198e226..d51e679f7)

## Scope verified

I read the complete three-commit diff (30 files), the M17 Wave 1 contract in `plans/issues/active/ad-hoc-declaration-first-python-interop.md`, the pass-1 artifact, the full post-diff runner (`example_packages.py`, `ordinary_example_policy.py`, all three ordinary suite files, `run.py` wiring), all eleven migrated `.sifr` callers and their bridges, the stdlib surface (`sifr/python.sifr`, `sifr/python_core.sifr`, private `_sifr/python.sifr`), and the compiler's import-policy crate. I executed the new guard against ~40 adversarial sources, compile-checked the residual bypass candidates against the actual `sifr` CLI, and independently re-ran the runner self-test lane.

## Pass-1 findings: all remediated and verified

**MAJOR 1 (fail-open denylist) — closed.** The guard is now a tokenizer-driven allowlist (`ordinary_python_api_policy_violations`, `example_packages.py:260`) restricting `sifr.python`/`sifr.python_core` imports to six names (`PythonError`, `ResourceDiagnostics`, `resource_diagnostics`, `ExitCause`, `ExitCauseKind`, `ExitDecision` — each used by real fixtures, none exposing an `Object`-typed field). I reproduced every pass-1 bypass and probed new ones; all are caught: unlisted names (`from_value`, `to_value`, every copy/zero-copy/protocol helper — anything not allowlisted), `sifr.python_core`, spaced dots, backslash continuations, aliases (including aliasing to an allowed name), star imports, parenthesized multiline imports with comments, semicolon-separated statements, direct module imports with comma lists and `as`, `from sifr import python/python_core`, nested/conditional/class-body imports, and `@trust_python_dynamic` in spaced and continued decorator spellings. Strings and comments are correctly not flagged; tokenization failures fail closed as `unparseable-source`.

The import-free M16 method-style path is also closed by construction: the only sources of an `Object` value are imports the allowlist blocks, `_sifr.*` is compiler-rejected for user code (`SIFR-IMPORT-0001`, verified by compile), and the allowed types carry only str/enum/bool/int fields. I also compiler-verified the guard's blind spots are unreachable: bare `import python` (`SIFR-IMPORT-0008`), `import sifr` (`SIFR-IMPORT-0003`), `from sifr import *` and `sifr.python.sub` (`SIFR-IMPORT-0002`), `from sifr.python import *` (`SIFR-NAME-0004`) — and the guard independently flags `*` anyway. The README/report certification now matches enforced behavior.

**MINOR 2 (no end-to-end negative tests) — closed.** Eight `POLICY_REJECTION_SEEDS` covering every pass-1 bypass class are driven through the full `build_examples_report` path per suite, asserting `examples-failed`, the declaration-first reason string, and that the example runner is never invoked (`_assert_policy_seed_rejected`, `example_packages.py:381`). I re-ran `runner/run.py --self-test` through the locked uv project: passes.

**MINOR 3 (unverified marker facts) — closed.** The torch bridge now asserts `doubled.dtype != torch.float32` and the PyArrow bridge asserts both PyCapsule names via `repr`, in the correct Arrow C-interface order (schema first, array second).

## Rest of implementation and migrations

All eleven `.sifr` callers are uniform and policy-clean: allowed imports only, one typed `@python(bridge.<module>.run)` declaration, before/after `resource_diagnostics()` equality on live and leaked objects, `PythonError` propagation through `Result`, deterministic markers checked by the runner. All bridges are hermetic and deterministic (Stubber, fakeredis, hiredis parse-only, in-memory SQLite, `make_conninfo` without connecting, fixed `random_state`), copied via `bridge_files` whitelists. The guard also now applies to the six protocol/async example suites sharing `build_examples_report` — I verified all seventeen of those fixtures are policy-clean, consistent with the 19/19 result. `demos/m16_raw_api` is the single intentional raw example (the m12–m14 demos import only `PythonError`); remaining raw fixtures are sanctioned certification inputs per the README. The plan's Wave 1/validation record matches the stated results, the Wave 1 checkbox stays unchecked per closure convention, `example_packages.py` is 698 lines (under the cap), and no compiler code or lockfile was touched.

## Residual observations (not actionable for Wave 1)

- The policy remains a runner-owned tokenizer guard rather than a compiler-enforced declaration-first package mode; pass-1's long-term note stands and fits Wave 3+.
- `execution_model` is still asserted rather than derived (pass-1 INFO, unchanged, acceptable).
- The eight policy seeds re-run once per suite (11×) in `--self-test` — redundant but cheap.

No blocker or major remains. Wave 1's deliverables — eleven honest migrations, one intentional raw example, and a genuinely fail-closed source-policy guard with end-to-end negative coverage — are all present and verified.

VERDICT: SATISFIED
