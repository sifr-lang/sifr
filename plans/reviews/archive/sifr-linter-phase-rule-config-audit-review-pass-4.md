

Now I have all the data needed. Let me write the findings.

---

## Findings: Pass 4 — Rule/Config Planning Decision Review

### Verification method

Cross-checked against:
- Filesystem: `/Users/yaseralnajjar/work/sifr/ruff/crates/ruff_linter/src/rules/` — 58 rule-family directories
- Audit table: `issues/ad-hoc-production-grade-sifr-linter.md` lines 168–225 — 58 rows
- Config source: `crates/ruff_workspace/src/options.rs` `LintOptions` + `LintCommonOptions` fields- Pass-1 through pass-3 reviews and pass-4 review artifact

---

### Q1: Does the rule-family audit cover every Ruff family?

**Yes —58/58 exactly.** All directories in the filesystem are present in the audit table. No discrepancies. Every row has a locked disposition (`sifr-native`, `formatter-owned`, `future-phase`, or `reject`) with a non-trivial rationale.

One precision note: `ignore-init-module-imports` in `LintCommonOptions` (line 710, deprecated since Ruff0.4.4) is not in the config-surface audit. Since it is deprecated and will be removed entirely, this absence is correct — no audit entry needed for actively deprecated options. No action required.

---

### Q2: Does the config-surface audit cover the relevant surfaces?

**Substantially yes, with one gap.**

All major lint surfaces (rule selection, fixes, suppression, discovery, output, output-format, cache, extends, overrides, plugins) are covered and locked with clear dispositions.

**Missing:** `extend-ignore` is present in `LintCommonOptions` as a deprecated-but-still-accepted field (lines 588–591). The audit table does not mention it. Because it is `extend-ignore`'s entire purpose is the same as `ignore` (suppressing rules) except `extend-ignore` appends rather than replacing, the existing `ignore` audit row's disposition (`adapt`) should carry over, but the gap in the table creates ambiguity: does accepting `extend-ignore` in Sifr lint config require explicit audit coverage? Based on Ruff's own deprecation note ("`extend-ignore` is now interchangeable with `ignore`"), the correct disposition is: Sifr lint should accept `extend-ignore` as a synonym for `ignore` during the deprecation window, then remove it — which maps to the existing `ignore` row's `adapt` classification.

**Recommended edit** for the config-surface audit table, in addtion to the row for `ignore`:
```
| `extend-ignore` | reject | Deprecated in Ruff (0.4.4); Sifr lint accepts it only as a silent `ignore` alias during a deprecation window; it must not be documented as a primary config key. |
```

---

### Q3: Are the decisions concrete enough for implementation PRs?

**Yes — with one precision note.**

Every `sifr-native` row gives a non-empty rationale that constrains implementation: it tells an implementer *what* must be proven before the rule ships (e.g., "equivalent Sifr AST/HIR semantics," "Sifr debug API names," "HIR control-flow support"). No row reads "figure it out later."

The manifest (`ruff_rule_config_audit.json`) enforces this: `check_linter_reuse_rules.py` must fail on any Sifr rule that references a row whose disposition is `reject`, `formatter-owned`, or `future-phase` without a reviewed update. This is a strong gate.

**One gap in the enforcement narrative:** The Phase contract references `ruff_rule_config_audit.json` enforcement twice (in the rule-family audit and config-surface audit sections), but the manifest schema is not described in the phase itself — only in M1's scope. If the manifest structure is underspecified, M5 implementers might ship `sifr-native` rules without proof of the equivalent-semantics claim. The fix: describe the manifest schema in the phase (not just M1).

---

### Q4: Is the manifest enforcement strong enough?

**Mostly strong, with one structural gap.**

The enforcement requirements in the phase are:
1. `check_linter_reuse_rules.py` must fail if a Ruff rule-family directory exists but is missing from the manifest.
2. M5 must fail if a new Sifr rule references a `reject`/`formatter-owned`/`future-phase` row.
3. `check_linter_reuse_rules.py` must fail on accepted config keys absent from the audit, and on Ruff/Python config keys accepted without `sifr-native` or `adapt`.

These are the right checks. However, item1 depends on the manifest listing every directory that exists *at the time M1 is implemented*. If a future PR adds a new rule family to the Ruff fork (unlikely, but possible), the manifest goes stale silently unless the phase adds:

**Recommended addendum to the phase, in the rule-family audit section:**
> "The manifest is pinned to the Ruff fork state at phase planning time. `check_linter_reuse_rules.py` must verify the manifest's listed families against the actual filesystem directories. Any filesystem directory not in the manifest causes a failure."

This closes the latent gap without changing the enforcement design.

---

### Q5: Are any `sifr-native` dispositions wrong or risky?

**One flag, not a blocker.**

`flake8_comprehensions` → `sifr-native` — the rationale says "only for Sifr AST constructs with equivalent semantics." Comprehensions (list/dict/set/generator) exist in both languages and have enough structural similarity that Sifr could plausibly ship equivalent simplification rules. This is correct. Same applies to `flake8_simplify`, `flake8_bugbear` (with "no direct rule port" constraint), `flake8_return` (with "HIR control-flow support" constraint), and `flake8_pie` (with "equivalent AST/HIR pattern" constraint). All are properly hedged.

`pylint` → `sifr-native` — the most permissive row. Rationale is "only individual Sifr-equivalent policy ideas." This is correct but creates the most implementation discretion. The M5 `check_linter_reuse_rules.py` gate (rows only from `sifr-native`) is the correct control. No change needed.

No `sifr-native` rows that should be `future-phase`, `formatter-owned`, or `reject`.

---

### Q6: Are there production-grade gaps before implementation starts?

**No blockers. Two precision improvements, both non-blocking.**

**Improvement 1:** The manifest schema should be described in the Phase contract body, not only in M1's scope. Currently M1 creates the manifest, but the phase itself does not specify the schema — so a reader cannot verify correctness before M1 runs. Adding a brief schema description to the Phase prevents M1 from encoding an underspecified manifest.

**Recommended addition** to the Phase after the rule-family audit table:
```
Manifest schema (to be encoded in `ruff_rule_config_audit.json` in M1):
- `version`: integer schema version
- `ruff_fork_pin`: commit hash or version tag
- `rule_families`: array of objects with `{ name, directory, disposition, rationale, sifr_requirement_note }`
- `config_keys`: array of objects with `{ key, disposition, rationale, sifr_requirement_note }`
```

**Improvement 2:** The `extend-ignore` gap noted in Q2 — add the one-row recommendation above.

---

### Verdict

**The phase is implementation-ready.**

All three prior pass blockers (`C-1` through `C-2`) are resolved by post-pass-2 phase updates. The rule-family audit is complete (58/58). The config-surface audit covers all active surfaces with one gap (`extend-ignore`, corrected above). The `check_linter_reuse_rules.py` enforcement requirements are strong enough to be enforceable. No `sifr-native` dispositions are wrong. The two recommended improvements are precision additions, not blockers — they improve auditability and prevent future manifest staleness but do not prevent M1 from starting.

**The phase is ready for implementation. One explicit update to the phase before M1 closes the only gap: add the manifest schema description and the `extend-ignore` config row.** ###
