## Pass 2 — diagnostic MDX migration prerequisite

Reviewed the complete four-file diff (`explain_cli.rs`, `gen-error-docs.rs`, `check_diagnostic_source_canonicalization_rules.py`, `code_coverage.py`) and re-ran validation myself.

### Actionable findings

**1. [Low — blocking under the no-findings bar] New clippy `uninlined_format_args` warning on the rewritten `format!`** — `crates/sifr/src/explain_cli.rs:68`

```
warning: variables can be used directly in the `format!` string
  --> crates/sifr/src/explain_cli.rs:68:10
```

This is newly introduced, not pre-existing: I reproduced both forms in an isolated crate, and only the new one lints. The old code passed `title.trim_start_matches("# ")` (a non-inlinable expression), which suppresses the lint for the whole call; now both args are plain bindings, so it fires. Fix:

```rust
    Some(format!(
        "{title}\n\n{summary}\n\nDocs: https://docs.sifr.sh/errors/{code}"
    ))
```

Caveat for fairness: `cargo clippy --workspace -- -D warnings` is already red on `main` (`sifr_lowering` fails it), and clippy is not wired into `scripts/run_all_tests.sh`, so this will not fail the create-PR lane. It is still a new warning in a touched line and a one-line fix.

**2. [Low] Debug explain output now repeats itself and diverges from the release path** — `explain_cli.rs:66-71`

`title` frontmatter is `"{id}: {summary}"` and `description` is `{summary}`, so `sifr --explain SIFR-IMPORT-0001` in debug prints:

```
SIFR-IMPORT-0001: Forbidden private sysroot declaration import.

Forbidden private sysroot declaration import.
```

Previously line 2 was the body prose ("`SIFR-IMPORT-0001` belongs to the … family. It means: …"), so the second line carried new information. Release builds print bare `entry.id` on line 1. Reading `sidebarTitle: ` (which is exactly the id) instead of `title: ` would remove the duplication and make debug output match the registry fallback exactly. Cosmetic and debug-only, but it is a behavior change the diff doesn't acknowledge.

### Non-blocking observations

- **No YAML unescaping.** `gen-error-docs.rs:483` escapes `\` and `"` into the quoted scalar; the reader does `.trim().trim_matches('"')` with no inverse. A summary containing a quote would render with literal `\"`, and `trim_matches` strips repeated trailing quotes. I scanned all 204 generated `SIFR-*.mdx` pages with the reader's exact semantics: every one yields non-empty `title` and `description`, and none contains an escape. Zero current impact; only latent.
- **CRLF.** `lines.next()? != "---"` and `line == "---"` would both miss `---\r`. Repo files are LF-generated, so not reachable today.
- **Test strength.** The single assertion is a prefix check on `title` only, hardcoding registry summary text, with no `assert!` message. It does prove `description` parsed non-`None` (otherwise `source_tree_diagnostic_explanation` returns `None` and `expect` fires), but not its content, and nothing covers the frontmatter terminator (a body line `title: …` after the closing `---` must not be picked up). Adequate for the prerequisite; worth a second case if this parser grows.
- **Fixtures no longer mirror real pages.** `seed_minimal_repo` writes `# {code}\n` into `.mdx` files with no frontmatter. Existing checks only test existence/substring, so it passes, but the fixture shape has drifted from generated output. Seeded index links use `({code}.mdx)`, which does match the real `diagnostic-codes.md:76` link style — good.
- **`gen-error-docs.rs` scope.** Both hunks are pure whitespace/`rustfmt` output, justified because `main` fails `cargo fmt --check`. Worth one line in the PR description so it doesn't read as an unexplained touch.

### Scope and stale-consumer sweep

- Only the four intended files are in scope. The parallel `ad-hoc-class-field-mutating-receiver-place-semantics` plan edit and the seven review artifacts are untouched by this diff and excluded.
- Repo-wide sweep for per-code `.md` consumers is clean. Every live reference now uses `.mdx` (`registry.rs:626`, `registry_tests.rs:60`, `code_baseline_coverage.py:172`, `code_catalog.json`, plus the two migrated checks). The only remaining `SIFR-*-NNNN.md` hits are historical prose in `plans/issues/archive/` and `plans/reviews/archive/`. The dual-emitted `docs/errors/diagnostic-codes.md` index correctly stays `.md` (`gen-error-docs.rs:56`, `registry.rs:877`, canonicalization check line 135 and the seeded index write).

### Validation I ran

- `cargo fmt --check` (workspace) — pass.
- `check_diagnostic_source_canonicalization_rules.py` normal — PASS; `--self-test` — PASS.
- `verification/areas/diagnostics/checks/code_coverage.py` — exit 0.
- `cargo test -p sifr debug_explanation_reads_generated_mdx_frontmatter` — 1 passed.
- Independent scan of all 204 generated pages against the reader's parsing semantics — all extract non-empty title/description.

I did not run the full diagnostics area 5/5, the docs error link guard, or the file-size guard; I'm taking those from your report. `explain_cli.rs` is 108 lines, well under the cap.

### Verdict

**Changes requested.** One actionable fix — inline the format args at `explain_cli.rs:68`. Finding 2 is a judgment call worth a decision (keep `title:` and accept the duplicated line, or switch to `sidebarTitle:` for release parity). The migration itself is correct, complete, and root-cause: all stale per-code `.md` consumers are gone, the dual-emitted index is correctly preserved, no unwrap/panic/fallback was introduced, and scope is clean. With finding 1 applied it is ready for full create-PR validation and PR.
