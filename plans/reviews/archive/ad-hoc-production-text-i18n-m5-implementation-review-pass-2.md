## Code Review: M5 Text/I18n Production Gate (Pass 2)

**Branch:** `text-i18n-m5-production-gate` vs `main`
**Scope:** M5 closeout artifacts + post-pass-1 harness remediation in `crates/sifr/tests/e2e_support/`

---

### 1. Verdict: **PASS**

All M5 scope items in `issues/ad-hoc-production-text-i18n-platform-substrate.md` are delivered, every artifact the phase contract names is present and well-formed, the post-pass-1 harness remediation does exactly what the user described, and the user-supplied local validation matrix is green (notably `scripts/run_all_tests.sh --profile create-pr` after remediation: 195.01s, 72/72 e2e pass, 3 pass / 2 skipped platform golden).

**M5 substrate closeout is ready for the PR.** Pass-1 PASS still stands after the remediation; nothing in the new diff regresses it.

#### Key validation of the harness remediation

`fixture_compilation.rs:322-334` and `fixture_compilation.rs:440-455` make grouped e2e Cargo.toml generation emit `sifr_runtime = { path = …, features = […] }` when grouped fixtures depend on `sifr.unicode` (`unicode` feature) and/or `sifr.i18n` (`i18n` feature). `sifr.encoding`-only groups correctly get the featureless `sifr_runtime` spec, matching `text_i18n_dependency_snapshots.json:6-15`. The dedup guard at `fixture_compilation.rs:406-413` keeps a `sifr_runtime` listed as a `required_crate` from clobbering the featured spec. `harness_behavior_tests.rs:486-513` locks the three meaningful cases (unicode-only, i18n-only, combined ordering `["i18n", "unicode"]`) and asserts `sifr_runtime = ` appears exactly once. Behavior matches the same feature gating that fixed the M2 peak-RSS regression — no regression risk for non-text fixtures because the gate is conditional on the text/i18n module names.

#### Phase-completeness spot-check (M5 issue file → evidence)

| Required | Evidence |
| --- | --- |
| Public docs | `docs/text_i18n.md` (encoding, unicode, io.open_text, i18n, Python-shaped diffs); `docs/stdlib_imports.md:14,34` extends rejected-bare-import list and links new doc |
| Internal docs | `internal_docs/architecture.md:754-788` (§7.1) + `internal_docs/text_i18n_architecture.md` |
| Demos (all six dimensions) | `demos/text_i18n/main.sifr` covers non-UTF-8 encode/decode incl. ASCII+replace, `open_text` round-trip, NFC + `category`, `graphemes` + `words`, `NumberFormatter` w/ `LocaleId("en-US")`, `Bundle` + `with_fallback` + `translate_plural` — 12-element `assert_bool_vector_eq` |
| Dependency snapshots | `verification/stdlib/text_i18n_dependency_snapshots.json` has 7 combinations (single, all pairwise, full); `features.rs:705-814` locks identical strings |
| Generated-code quality | `verification/generated_code_quality/manifest.json:11,63-67` adds `demo-007-text-i18n` + `e2e-051..055` across all five M1–M4 evidence categories |
| Validation lane manifests | create-pr 72 fixtures (incl. 5 text/i18n), merge 78 fixtures (incl. 5 text/i18n) |
| Inventory closure | `text_i18n_substrate_inventory.{md,json}` — `status: m5-complete`, terminal states/stabilities set, revisit rules + fixtures on every Python-shaped row, "M5 Closure Evidence" table aggregates artifacts |
| External review | pass-1 PASS recorded (this is pass-2, post-remediation) |

---

### 2. Blocking findings

**None.**

---

### 3. Non-blocking observations

1. **Execution ledger lacks the full M5 baseline validation matrix.** `issues/ad-hoc-production-text-i18n-platform-substrate-execution.md:367-376` records only demo, dep-snapshot subtest, JSON-tool checks, gcq corpus, panic-scan, fmt, and the two guardrail scripts. The phase contract M5 validation list also requires `cargo clippy --workspace -- -D warnings`, `cargo test -p sifr_stdlib` (whole crate), `cargo test -p sifr -- stdlib`, `scripts/run_e2e_pass.sh`, `scripts/run_all_tests.sh --profile create-pr`, and `scripts/run_all_tests.sh`. The user-supplied evidence covers the create-pr lane (195.01s, 72/72); the merge-gate run is not in evidence and not in the ledger. Add the missing rows before merging M5, or run the merge gate at minimum.

2. **Harness-remediation evidence is filed under M1, not M5.** `execution.md:279-282` puts `test_generate_cargo_toml_text_i18n_modules_enable_runtime_features` and the 195.01s/72-fixture post-harness create-pr run under "M1 focused validation on branch `text-i18n-m1-encoding-io`". That work is post-pass-1 M5 remediation (M1 was already merged via PR 2298). Move these two bullets into the M5 section so provenance matches the branch they were run on.

3. **`Implementation Reviews` row for M5 pass 2 still reads "pending".** `execution.md:214` says `M5 pass 2: pending after validation-lane harness remediation.` Once this review lands (file `reviews/ad-hoc-production-text-i18n-m5-implementation-review-pass-2.md` is currently a 0-byte placeholder), populate the file and replace that "pending" line with the pass-2 result.

4. **Inventory status vs. milestone checklist mismatch.** Three artifacts already declare `m5-complete`: `text_i18n_substrate_inventory.{md:3,json:3}` and `text_i18n_dependency_snapshots.json:3`. But the milestone checklist at `execution.md:29` still has `[ ] milestone_text_i18n_5`. Flip the checkbox when the PR opens (or document that "M5 closure evidence is checked in but PR is pending" in the inventory status to keep the two consistent).

5. **All M5 changes are uncommitted.** `git log main..HEAD` is empty; everything is in working tree. Pass-1 N5 still applies — stage + commit the M5-only files before opening the PR. (Per scope, leave the seven concurrency-related files unstaged.)

6. **`crates/sifr_stdlib/src/features.rs` diff is purely the new lockdown test + a `normalize_runtime_dependency` helper inside `#[cfg(test)]`.** No runtime behavior changes — appropriate for a closeout milestone. Verified: production `generated_cargo_dependencies` paths are unchanged.

7. **`reviews/ad-hoc-production-text-i18n-m5-implementation-review-pass-1.md:48`** has a stray trailing fragment (`ther structural changes. N2–N5 are non-blocking and can be addressed opportunistically.`). Cosmetic; trim before phase close.

---

### 4. Re-review required?

**No.** The remaining items are ledger bookkeeping (move two lines, fill in pass-2 file, flip a checkbox, optionally run the merge gate) and committing the working tree. They can be verified on the final PR diff without another full review pass.
