# Ruff 0.15.12 update — review pass 1

**Branch reviewed:** `codex/update-ruff-0.15.12` (HEAD `49a874bb`)
**Submodule:** `third_party/ruff` on `sifr/0.15.12-maintenance`, commit `45e76046cc`
**Diff scope:** `main..HEAD` (single commit "Update Ruff fork to 0.15.12") plus the single Sifr commit on top of upstream `v0.15.12` (`66f93cf7ed`).

The mechanical part of the upgrade — root `Cargo.toml` collapse onto submodule paths, `Parsed::is_valid()` → `has_valid_syntax()`, `Name::clone()` → `Name::to_string()`, `FStringElement` → `InterpolatedStringElement` — is consistent and applied everywhere I could find it. The Sifr-specific parser extension (`mut`/`own` parameter modifiers) cleanly preserves through the upgrade with new tests covering the documented behaviors. No correctness regressions found in the call sites I traced.

There are no **blocking** findings. Below are non-blocking items in roughly descending order of importance.

---

## Findings

### 1. Stale Ruff version references in `README.md` and `AGENTS.md`

[`internal_docs/architecture.md:219`](internal_docs/architecture.md:219) was updated to "Ruff 0.15.12" and `sifr/0.15.12-maintenance`, but two other surfaces still claim the v0.4.10 lineage:

- [README.md:113](README.md:113) — *"The submodule tracks the `sifr/v0.4.10-maintenance` branch."*
- [AGENTS.md:53](AGENTS.md:53) — *"vendored from ruff v0.4.10, may diverge"*

`AGENTS.md` is loaded into context for every assistant session, so the stale claim will keep being repeated to humans and agents until corrected. Suggest updating both to the new branch and base version in this PR (or a follow-up).

Severity: low (documentation only).

### 2. Soft-keyword parameter names interact badly with `mut`/`own` modifiers

The Sifr extension at [crates/ruff_python_parser/src/parser/statement.rs:3099-3129](third_party/ruff/crates/ruff_python_parser/src/parser/statement.rs:3099) gates modifier consumption on:

```rust
while self.at(TokenKind::Name) && self.peek() == TokenKind::Name {
```

The peek check requires the *next* token to be exactly `TokenKind::Name`. Soft keywords (`match`, `case`, `type`, …) tokenize as their own kinds (`TokenKind::Match`, etc.), so a parameter declaration like `def f(mut match: int): ...` would silently treat `mut` as the *parameter name* and then fail to parse `match` in that position. The new test [`test_soft_keyword_parameter_names_still_parse_without_modifier_context`](third_party/ruff/crates/ruff_python_parser/src/parser/tests.rs:233) only covers the case where the soft keyword *is* the parameter name with no modifier — it doesn't cover modifier-followed-by-soft-keyword.

This is genuinely niche (no Sifr fixture appears to use `match`/`case`/`type` as a parameter name), so it isn't a regression vs. the previous fork. But since the lookahead was newly authored for 0.15.12, consider switching the peek to the broader check: `self.peek_kind().is_name() || self.peek_kind().is_soft_keyword()` (or whichever helper exists for "could-be-an-identifier-here"). Even just adding a comment narrowing the contract would help.

Severity: low (niche, no current callers affected).

### 3. Upstream-workspace crates outside Sifr's path-dep set won't compile against the new `Parameter::convention` field

The new field on `ast::Parameter` is destructured exhaustively (no `..`) in several upstream crates that Sifr does *not* pull in via path dependencies. Spot-checked:

- [crates/ruff_python_formatter/src/other/parameter.rs:10-15](third_party/ruff/crates/ruff_python_formatter/src/other/parameter.rs:10) — `let Parameter { range: _, node_index: _, name, annotation } = item;` would fail compilation with `missing field: convention`.
- Likely similar shapes in `ruff_python_codegen`, `ty_*`, etc. (didn't enumerate exhaustively).

Sifr's own workspace excludes `third_party/ruff`, and the path dependencies it does pull (`ruff_python_ast`, `ruff_python_parser`, `ruff_text_size`, `ruff_source_file`, `ruff_python_trivia`, `ruff_python_literal`) all compile cleanly — the local validation results confirm that. The only place this would bite is if someone runs `cargo build --workspace` / `cargo check --workspace` against `third_party/ruff/Cargo.toml`, e.g. for an upstream-merge dry run.

The matching change in [crates/ruff_python_ast/src/node.rs:359-365](third_party/ruff/crates/ruff_python_ast/src/node.rs:359) (which adds `convention: _` to the destructure in `ast::Parameter::visit_source_order`) shows the author was aware of this pattern; it just wasn't propagated to the consumers Sifr doesn't compile. Not a bug for Sifr's own pipeline today, but a forward-compat trap. Either:

- mirror the same `convention: _` insertion (or `..`) into `ruff_python_formatter` etc. so the whole upstream workspace still builds, or
- explicitly note in `internal_docs/architecture.md` that only the Sifr-consumed crates are guaranteed to build, so future upstream-merges expect the breakage.

Severity: low (only matters if/when upstream workspace is built).

### 4. MSRV bump implied by submodule path-deps is undocumented

Each Ruff crate inherits its `edition`/`rust-version` from `third_party/ruff/Cargo.toml`, which now declares `edition = "2024"` and `rust-version = "1.93"`. Sifr's root [Cargo.toml:14](Cargo.toml:14) still declares `rust-version = "1.75"`. Because `ruff_python_ast` and `ruff_python_parser` are path dependencies, the *effective* Sifr MSRV is `max(1.75, 1.93) = 1.93`.

The current value isn't load-bearing for the build (any contributor with rustc ≥1.93 is fine), but it overstates the supported toolchain range. Suggest bumping Sifr's `rust-version` to match the new floor, or at least dropping a sentence in `architecture.md` calling out the submodule-driven floor.

Severity: low (cosmetic/policy).

---

## Residual risks (non-blocking)

- **T-strings (PEP 750 / Python 3.14)** are now parseable in the upgraded grammar but Sifr's lowering at [crates/sifr_hir/src/lower/expressions.rs:64-98](crates/sifr_hir/src/lower/expressions.rs:64) has no `Expr::TString` arm, so any `t"..."` literal falls through to the `_ => ctx.error("unsupported expression type")` arm. Behavior is safe (clean compile error rather than panic), but the diagnostic is unhelpful — worth a dedicated arm with a clearer "t-strings are not supported" message at some point. No need to block this PR.
- The mechanical `n.id.clone()` → `n.id.to_string()` substitution is correct, but it always allocates a fresh `String` even where the surrounding code only reads the value as `&str` (e.g., short-lived comparisons or `format!` arguments). With `Name: Deref<Target = str> + Display + PartialEq<&str>`, several call sites could be left as `&n.id`, `n.id.as_str()`, or direct `format!("{}", n.id)` without the intermediate `String`. A future cleanup pass could trim those allocations; nothing here is wrong.
- The new fixture marker [crates/sifr/tests/verification/project/workspace_unresolved_import/lib/.gitkeep](crates/sifr/tests/verification/project/workspace_unresolved_import/lib/.gitkeep) is a one-byte newline rather than the conventional zero-byte placeholder. Functionally identical; just noting.
- `Cargo.lock` shows a single coherent set of `ruff_*` `0.0.0` path-dep entries — no leftover `v0.4.10` git-source duplicates. Confirmed clean at [Cargo.lock:875-934](Cargo.lock:875).
- The Sifr submodule commit `45e76046cc` is pleasingly minimal: only the parser/AST extensions (parameter `convention` field, `mut`/`own` lookahead, four parser tests, snapshot regenerations, and the two `#![allow(warnings)]` shims at crate roots). Nothing extraneous landed during the rebase onto 0.15.12.
- All four new parser tests cover the documented behaviors I'd expect: both source orders (`own mut` / `mut own`), duplicate-modifier rejection for each modifier, and soft-keyword-as-parameter-name (when no second name follows). The duplicate diagnostics fire from the same loop that consumes the second modifier, so there's no risk of the parser stopping early and silently dropping the second token. Note the gap called out in finding #2 above.
- `sifr_python_ast` / `sifr_python_parser` Cargo aliases preserved — no consumer in the Sifr crates was missed (`grep` for `FStringElement` and `is_valid()` in `crates/` returns nothing relevant).

---

## Codex follow-up

Addressed after this review:

- Updated stale Ruff 0.4.10 references in `README.md` and `AGENTS.md`.
- Raised the Sifr workspace `rust-version` to `1.93` and documented the effective Ruff-driven toolchain floor in `internal_docs/architecture.md`.
- Fixed the `mut`/`own` parser lookahead so a modifier can precede a soft-keyword parameter name, with a focused parser test for `def f(mut match: int)`.

The Ruff submodule review-fix commit is now `d2a5f1fb7f`.
