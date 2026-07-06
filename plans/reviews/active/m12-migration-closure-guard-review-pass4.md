## Findings (most severe first)

### 1. Line-based accumulator can leak strings from non-match-arm context (medium)

In `scripts/check_stdlib_migration_closure.py`, `_active_intrinsic_names` accumulates *any* line that contains `"` or `Named(` into `pending_pattern`, and only clears it when a line containing `=>` is processed. Because the accumulator does not respect statement/block boundaries, a non-match-arm line above a match arm can leak retired names into the extracted set.

**Failure scenario**: A future edit to `registry.rs` adds a doc comment or diagnostic message just above a live arm, e.g.

```rust
// deprecated: "sqrt" was removed in M9 wave 4, see stdlib_migration_closure guard
"cbrt" => lower_cbrt(args),
```

The line with the comment has `"`, so it enters `pending_pattern`. The next line has `=>`; the combined pattern regex-matches both `sqrt` and `cbrt`. `sqrt` is in `RETIRED_INTRINSICS`, so the guard fails with a false positive that has nothing to do with dispatch. The current self-test suite only covers single-line inputs and would not catch this. Consider clearing `pending_pattern` on lines with `;`/`}`/blank, restricting accumulation to lines that look like continuation of a pattern (leading `|`, only whitespace + `"…"`/`Named(…)`), or stripping `//` line comments before scanning.

### 2. Stale-phrase check is exact-substring only (low)

`STALE_ARCH_PHRASES` matches with `in arch_doc_text`, so paraphrases regress silently.

**Failure scenario**: A doc rewrite phrases the frozen state as "the complete surface-by-surface ownership decision continues to live in the TOML registry" (word swap "remains" → "continues to live"). The guard treats the doc as clean even though it still asserts the deleted TOML is the source of truth. Given this is a durable-wording guard, this is inherent to substring matching; worth at least documenting the tradeoff in the script header, or moving to a small regex per phrase with keyword anchoring (e.g. `TOML registry.*current.*truth`).

### 3. Guard is scoped to a single file (low)

`REGISTRY_DISPATCH_PATH` is hard-coded to `crates/sifr_codegen/src/intrinsics/registry.rs`. If dispatch is later split (e.g., a sibling `intrinsics/math_dispatch.rs`) or the `registry.rs` module is renamed, the guard silently keeps passing on an empty scan.

**Failure scenario**: A refactor moves math intrinsic arms to `crates/sifr_codegen/src/intrinsics/math.rs` and leaves `registry.rs` as a thin `pub mod` shell. A retired name is reintroduced in `math.rs`. `registry_text` contains no retired names, and no failure is raised. Consider globbing `crates/sifr_codegen/src/intrinsics/**/*.rs` or asserting `registry.rs` still contains at least one arm as a sanity check.

### 4. No cross-check that `RETIRED_INTRINSICS` matches the retained allowlist (low)

The set of ~140 retired names is maintained by hand and is disjoint from the retained-intrinsic allowlist. There is no assertion that a name cannot appear in both, and no test that a newly migrated intrinsic added to a later wave gets appended here.

**Failure scenario**: M12 wave 5 migrates a new leaf but the author forgets to add it to `RETIRED_INTRINSICS`. The guard happily lets the retired arm remain wired up in `registry.rs`. Would be strengthened by loading retained names from `stdlib_retained_compiler_intrinsics.toml` and asserting the two sets are disjoint at runtime.

---

VERDICT: PASS

The core M12 wave 4 intent (freeze retired names out of active dispatch, keep the ownership TOML deleted, prevent stale doc regressions) is met, self-tests cover the specific false-positive/negative modes flagged in earlier passes (bare, `Named(...)`, guarded, equality-guarded, guard-only), and the guard is wired into `guardrails.json` and `profile_runner.py`. Findings 1–4 are hardening opportunities for the next wave, not blockers for this diff.
