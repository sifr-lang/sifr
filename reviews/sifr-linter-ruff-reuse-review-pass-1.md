Based on my review of the Phase 36 contracts, Sifr's current lint implementation, and Ruff's linter infrastructure, here is my assessment:

---

## Sifr Lint/Ruff Reuse Strategy Review

### Summary Verdict: **CONDITIONALLY SOUND — One blocker identified**

The strategy is correct in direction and rationale, but the suppression comment parsing lacks a critical detail that will become a blocker as Sifr adds more rules.

---

## Findings

### 🔴 BLOCKER: Suppression comment parsing lacks parser integration

**Location:** `crates/sifr_lint/src/lib.rs:247-281`

Sifr's current suppression parsing is text-based:

```rust
fn parse_suppressions(source: &str, file: Option<&Path>) -> Vec<Suppression> {
    for (line_index, line) in source.lines().enumerate() {
        // ...
        if let Some(stripped) = suffix.strip_prefix('[') {
            // Parses on a single text line
        }
    }
}
```

This is insufficient. When a suppression comment appears on a multi-line construct:

```sifr
result = call(
    arg,
)  # sifr: ignore[unused-result]
```

The suppression should cover the entire multi-line statement, not just the physical line containing the comment. Ruff's `ruff_linter::suppression.rs` handles this by:
1. Using Python AST indentation tracking (`Dedent`/`Indent` tokens)
2. Computing "logical line" ranges for own-line suppressions
3. Handling nested blocks and indentation scopes

**Impact:** Token-only rules like `trailing-whitespace` work fine with text-based parsing. As soon as Sifr adds syntax-only rules (e.g., redundant parentheses, unreachable branches) or HIR rules (unused variables), the suppression boundary will be wrong.

**Fix required:** Suppression comment parsing must integrate with `sifr_syntax` to determine statement ranges, similar to Ruff's approach. This is not a showstopper for the first rule but will become one.

---

### 🟡 CONCERN: `ruff_linter::fs` should be reconsidered

**Location:** `third_party/ruff/crates/ruff_linter/src/fs.rs`

This module is listed as "reject without review" in the non-reuse list, but it contains genuinely language-neutral utilities:

```rust
pub fn get_cwd() -> &'static Path { ... }
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf { ... }
pub fn relativize_path<P: AsRef<Path>>(path: P) -> String { ... }
```

The dependency on `CompiledPerFileIgnoreList` from `ruff_linter::settings` is the Python coupling. If Sifr extracts just the path utilities (cleanly separable), they would be reusable.

**Suggestion:** Consider extracting or reimplementing the path utilities in `sifr_lint` or a shared utility crate. The existing Sifr implementation in `sifr_lint::collect_sifr_files_inner` duplicates this work.

---

### 🟢 ACCEPTED: All other non-reuse decisions

The following are correctly rejected:

| Component | Rationale |
|---|---|
| `ruff_linter::registry` | Flake8-prefixed IDs (F401, E501) are Python-specific |
| `ruff_linter::rules::*` | Python semantic rule implementations |
| `ty_python_semantic` lint engine | Explicitly forbidden by Phase 36 |
| `ruff_linter::noqa` | `# noqa: F401` is Flake8-specific |
| `ruff_linter::suppression::SuppressionParser` | Python AST-dependent (Dedent/Indent token tracking) |
| Python target-version settings | No Python version concept in Sifr |

---

### 🟢 ACCEPTED: Infrastructure pattern adaptations

The strategy correctly allows adapting:
- Rule metadata shape
- Severity resolution concepts
- Rule selection patterns (select/ignore/extend-select)
- Fix applicability concepts (safe/unsafe) — but only after Sifr defines its own
- Testing patterns (snapshot testing via `insta`)

---

### 🟡 SUGGESTION: Rule family classification is incomplete

**Point 6** classifies rules by analysis depth, but the classification is silent on **suppression boundary complexity**:

| Category | Suppression complexity |
|---|---|
| Token/trivia-only | Trivial — single physical line |
| Syntax-only (single-line) | Simple — parser can determine |
| Syntax-only (multi-line) | Complex — needs indentation tracking |
| HIR/semantic | Depends on rule scope |

**Recommendation:** Add a suppression complexity dimension to the classification. This directly informs whether Sifr needs parser-integrated suppression parsing.

---

### 🟢 ACCEPTED: Config ownership is correct

`[lint]` in `sifr.toml` with Sifr rule IDs is correct. No implicit Ruff migration path should exist.

---

## Question-by-Question Assessment

**1. Is this reuse/non-reuse boundary correct and production-grade?**

Yes for the high-level boundary. Production-grade requires fixing the suppression parsing blocker before adding multi-line rules.

**2. Are there Ruff linter components we should reuse directly that I am incorrectly rejecting?**

`ruff_linter::fs` path utilities are the main candidate for reconsideration. They're separable from Python-specific settings.

**3. Are there components I am incorrectly allowing?**

No. The allowed adaptations (registry shape, severity resolution, rule selection) are appropriately abstract and would be Sifr-owned implementations.

**4. Is the rule-family classification complete enough for a future Sifr lint phase?**

Incomplete. Needs a suppression complexity dimension as noted above.

**5. What would you change before presenting this to the user?**

Add one bullet to Point 6:

> **Suppression complexity:** Rules that apply to multi-line constructs require parser-integrated suppression parsing that determines statement/function/class ranges. Token-only rules (trailing whitespace) work with line-based suppression. Syntax-only and HIR rules require integration with `sifr_syntax` for correct suppression boundaries.

---

## Recommendation

**Merge the strategy as-is**, with a Phase 37 action item: "Parser-integrate suppression comment parsing before adding non-trivial policy rules." The current trailing-whitespace implementation is sufficient for Phase 36, but the next phase of lint rules will need the infrastructure.

Alternatively, add the suppression complexity requirement to Point 6 before finalizing.
