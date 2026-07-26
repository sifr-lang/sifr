> **Post-rebase status:** This round records the historical pre-rebase working
> tree. PR #3017 subsequently landed the diagnostics MDX and Ruff revalidation
> repairs on `main`; they are not part of the final `hardening_1` diff. The
> fixture `Cargo.lock` is tracked in the milestone commit. See round 4 and the
> later approval round for the authoritative post-rebase scope and verdict.

**Not approved** — 7 actionable findings, one of them a confirmed user-visible regression.

## What holds up

The core milestone wiring is correct and I verified it independently:

- `rust_interop_checks` is in the single fixed step tuple (`profile_runner.py:71`), reached through the only entrypoint (`scripts/run_all_tests.sh:105/109`). The refactor from an inline list to module constants is what lets `selftest.py` assert step membership without executing a lane — a good structural choice.
- All four authoritative profiles select all four manifest-declared suites, and `required_rust_interop_suites()` derives the requirement from `manifest.json` rather than duplicating it, so adding a fifth suite fails all four profiles closed until they're updated.
- The create-PR budget is genuinely measured: I timed the exact area invocation three times warm — 0.50s / 0.32s / 0.31s against `budget_ms: 5000`, ~10× headroom, and earlier `uv`-based steps warm the environment first. Not flaky.
- Fresh-result semantics work: `unlink(missing_ok=True)` before the run, explicit `--result-json`, and `run_command` raises on non-zero before validation is reached.
- Runner self-tests pass (8/8, including the new one), the emitted create-pr plan carries both the area selection and the blocking budget, the two repaired validators pass, file-size guardrail and `git diff --check` are clean.
- The `.mdx` validator repairs are the right root cause: `registry.rs:626` and `gen-error-docs.rs:164-179` already treat `.mdx` as canonical for per-code pages and actively flag `.md` stubs as drift, while `diagnostic-codes.md` legitimately remains `.md`. 205 `.mdx` pages, 1 `.md` index.
- The fixture lockfile is real and valid — `cargo metadata --locked --offline` succeeds unchanged in the fixture, and the `.gitignore` negation does make the path addable.

## What needs fixing

The blocker is `crates/sifr/src/explain_cli.rs:64`. That edit was not needed to unblock the gate (nothing under `verification/` or `scripts/` reads that file), and it actively breaks output. MDX pages put the title in YAML frontmatter, so the `# `-heading scraper finds nothing on 197 pages — meaning the "repair" changes nothing there — but on the 8 `SIFR-LINT-000*` pages the first `# ` line is a Python comment inside a fenced code block. I ran the binary:

```
$ cargo run -q -p sifr -- --explain SIFR-LINT-0001
sifr: ignore

def f(flag: bool):

Docs: https://docs.sifr.sh/errors/SIFR-LINT-0001
```

`origin/main` prints the correct registry text. Either delete the dead `source_tree_diagnostic_explanation` or parse frontmatter while skipping fenced blocks — and add the test that's missing (finding 2 explains why nothing caught this).

The remaining findings are smaller: the step's silent skip-on-no-suites fallback, `validate_rust_interop_result` accepting a `bless: true` document, four untested `total_variants` branches, the hardcoded `.gitignore` path plus the still-untracked `Cargo.lock`, and the 0-byte review placeholder.

One non-blocking observation: the new "must select the rust_interop area" assertion in `profiles.py:174-183` is asymmetric with `python_interop`, which only checks suites when the area is present. If more areas become mandatory, that pair wants to become one data-driven rule rather than two hand-written ones.
