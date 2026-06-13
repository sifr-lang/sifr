# Review: unwrap/expect cleanup + workspace `unwrap_used`/`expect_used` lints

**Date:** 2026-04-27
**Scope:** Uncommitted diff against `main` covering 9 files. The diff (a) enables workspace-wide `clippy::unwrap_used` and `clippy::expect_used` lints, and (b) rewrites a small number of `.unwrap()` / `.expect(...)` sites to satisfy them.

**Verdict:** Functionally safe under current invariants. Most `unwrap`/`expect` removals were provably safe before and remain provably safe after. The cleanup, however, repeatedly trades **loud assertion failures** for **silent fallbacks that emit invalid Rust or drop information**, which weakens our defenses if those invariants ever drift. The lint enablement is also incomplete: ~1009 sites under `--all-targets` still trigger the lints, so `cargo clippy --workspace --all-targets -- -D warnings` fails. The default `cargo clippy --workspace -- -D warnings` (CI's command) passes.

---

## Findings (ordered by severity)

### 🟠 Medium — `--all-targets` clippy gate is now broken

**File:** [Cargo.toml](Cargo.toml:86-87)

```toml
unwrap_used = "warn"
expect_used = "warn"
```

These lints inherit into every workspace crate via `[lints] workspace = true`.

- `cargo clippy --workspace -- -D warnings` (the command from `AGENTS.md` and `.github/workflows/local-first-validation.yml:23`) — **passes**. Production lib code is clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — **fails with 1009 warnings → 140 errors in `sifr_hir` (lib test) alone**. Test files (`*_tests.rs`, `tests/*.rs`, `crates/sifr/tests/e2e.rs`, etc.) and bin targets are full of `.unwrap()` calls that the new lints now flag.

The CI workflow currently runs the non-`--all-targets` form and the step is wrapped in `continue-on-error: true`, so this won't break the pipeline today. But:

1. Anyone running `cargo clippy --all-targets` locally (a very common defensive pattern, and what most editor LSP integrations send) will be flooded with warnings the moment they touch a test file.
2. If/when the CI step is promoted from advisory to required, or gains `--all-targets`, it will fail catastrophically.
3. The advisory comment in the workflow says "until legacy workspace lint debt is burned down" — this change *adds* ~1000 lint hits in tests rather than burning anything down.

**Recommendation:** Either (a) defer enabling these lints until tests are also cleaned, (b) add `#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]` (or per-test-module `#[allow]`) before flipping the workspace flag, or (c) explicitly carve `tests` out via `[lints.clippy]` overrides in test-only Cargo configurations.

---

### 🟡 Low — Dead-but-buggy fallback emits invalid `Some()` (no args)

**Files / sites:**
- [crates/sifr_codegen/src/class_method_emitter.rs:99-102](crates/sifr_codegen/src/class_method_emitter.rs:99-102) (`ensure_some_box_inner`)
- [crates/sifr_codegen/src/intrinsic_method_emitters.rs:629-632](crates/sifr_codegen/src/intrinsic_method_emitters.rs:629-632) (`registry_ensure_some_box_inner`)
- [crates/sifr_codegen/src/stmt_support_emitter.rs:5060-5063](crates/sifr_codegen/src/stmt_support_emitter.rs:5060-5063) (`ensure_some_box_inner_for_ir`)

All three sites have the same shape:

```rust
let mut args_iter = args.into_iter();
let Some(inner) = args_iter.next() else {
    return RustExpr::FnCall { func, args: vec![] };
};
```

The arm is guarded by `args.len() == 1`, so `next()` is provably `Some(_)` and the `else { ... }` branch is unreachable. But what the unreachable branch *does* is produce a `Some()` call with **zero arguments**, which is invalid Rust (`Some` is unary) — meaning if an upstream invariant ever broke, we'd silently emit code that fails at the rustc stage with a confusing error far from the cause, rather than panicking at the codegen site with the clear `expect` message that used to be there.

**Recommendation:** Replace the silent `else { ... }` with `unreachable!("Some(_) call must have exactly one argument")` (clippy::unwrap_used does not flag `unreachable!`), or restructure to consume the single argument without re-emitting a `Some(...)` envelope. As written, the dead branch is a small bug waiting for an invariant slip.

---

### 🟡 Low — Compare lowering silently bails to generic fallback

**File:** [crates/sifr_codegen/src/stmt_support_emitter.rs:3799-3805](crates/sifr_codegen/src/stmt_support_emitter.rs:3799-3805)

Original:
```rust
let rhs_expr = comparators
    .get(idx)
    .expect("compare ops/comparators length should match");
```

New:
```rust
let Some(rhs_expr) = comparators.get(idx) else {
    return Ok(None);
};
```

The enclosing `if !ops.is_empty() && ops.len() == comparators.len()` guard at line 3799 makes `comparators.get(idx)` provably `Some(_)` for `idx ∈ 0..ops.len()`. Currently safe.

The risk is that `Ok(None)` from this branch tells the parent to fall through to a generic lowering path. So if an upstream change ever produced `ops.len() != comparators.len()` (or any other inconsistent AST) **after** the guard — for instance, if the guard is loosened in a future refactor and someone forgets this site — the code would emit a *different* lowering for the comparison without raising any compile error, with potentially silent wrong-result behavior in user code. The original `.expect()` would have panicked loudly during compilation, exposing the bug.

**Recommendation:** Same as above — `unreachable!("ops/comparators lengths checked equal")` preserves the loud failure while satisfying the lint.

---

### 🟡 Low — Numeric-truthiness lowering silently drops the special case

**File:** [crates/sifr_codegen/src/stmt_support_emitter.rs:6336-6356](crates/sifr_codegen/src/stmt_support_emitter.rs:6336-6356)

The early-out hoist:
```rust
let Some(zero_literal) =
    Self::zero_literal_for_numeric_truthiness_type_for_ir(option_inner_ty)
else {
    return Ok(None);
};
```

Cross-referencing the helper at [stmt_support_emitter.rs:6517-6539](crates/sifr_codegen/src/stmt_support_emitter.rs:6517-6539):
- Outer guard accepts `Type::Int | Type::LiteralInt(_) | Type::BigInt | Type::Float` (after `resolve_alias_type_for_plain_call`).
- `zero_literal_for_numeric_truthiness_type_for_ir` returns `Some(...)` for exactly the same set.

So `None` is unreachable today — the change is semantically equivalent. The latent risk is the same as the previous two findings: if either the outer guard or the helper drifts, a numeric-`Option` truthiness check (e.g., `if x:` where `x: int | None`) would silently fall back to the generic `is_some_and(|v| v)` or default lowering paths instead of `is_some_and(|v| v != 0)`. That's a *behavioral* divergence — `Some(0)` is supposed to be falsy in Sifr, and the original `expect` would have caught the regression at compile time.

**Recommendation:** `unreachable!()` (or keep the `expect` and `#[allow(clippy::expect_used)]` it). The early-return masks a class of bugs the `expect` was specifically designed to catch.

---

### 🟢 Very Low — Unreachable single-element `Type::Never` fallback in `make_union`

**File:** [crates/sifr_type_system/src/union.rs:25-32](crates/sifr_type_system/src/union.rs:25-32)

```rust
1 => {
    let mut iter = members.into_iter();
    if let Some(member) = iter.next() {
        member
    } else {
        Type::Never
    }
}
```

Inside the `1 =>` arm, `members.len() == 1` is established by the surrounding `match`, so `iter.next()` is always `Some`. The `Type::Never` fallback is dead code, and worse, **semantically misleading**: `Type::Never` means "this expression has no possible value" — a pathological value to return when you nominally have one type. A reader scanning `make_union` cold could plausibly conclude that single-element vectors sometimes normalize to `Never`, which is wrong.

The existing test [union.rs:276-279](crates/sifr_type_system/src/union.rs:276-279) (`test_make_union_single_element`) covers the correct path; nothing exercises the dead branch.

**Recommendation:** Either `_ => members.into_iter().next().expect("len == 1")` with `#[allow(clippy::expect_used)]`, or restructure with `match members.into_iter().next() { Some(t) if … => t, … }` so there's no dead arm. Failing that, leave a comment that the `Type::Never` is unreachable.

---

### 🟢 Very Low — Pattern-field lookup `let-else` rewrite

**File:** [crates/sifr_hir/src/lower/statements.rs:953-973](crates/sifr_hir/src/lower/statements.rs:953-973)

Replaces a `find(...).map(...)` + `is_none` check + `.unwrap()` with a `let-else`. Semantically identical, slightly cleaner, no regression. The error path (calling `ctx.error(...)` and returning `None`) is preserved verbatim.

---

### 🟢 Very Low — Narrowing-condition single-element rewrite

**File:** [crates/sifr_hir/src/lower/statements.rs:1903-1924](crates/sifr_hir/src/lower/statements.rs:1903-1924)

```rust
} else if conditions.len() == 1 {
    conditions.into_iter().next()  // was: Some(conditions.into_iter().next().unwrap())
}
```

`conditions.into_iter().next()` returns `Option<NarrowingCondition>`, which under the `len() == 1` guard is always `Some(_)`. The original wrapped this in an explicit `Some(...)`, which was redundant. Fully equivalent semantically, and the surrounding function returns `Option<NarrowingCondition>` so types line up. **No regression.**

---

### 🟢 Very Low — Nested function inference `let-else` rewrite

**File:** [crates/sifr_hir/src/lower/nested_function_inference.rs:772-775](crates/sifr_hir/src/lower/nested_function_inference.rs:772-775)

Replaces `if !contains_key { return; }` + `.expect("state present")` with a single `let Some(state) = states.get(...).cloned() else { return; }`. Equivalent semantically, **strictly better**: it removes the redundant double hash lookup (`contains_key` then `get`) that the original used to satisfy the borrow checker. No regression.

---

### 🟢 Very Low — `discover_test_root_modules` skips paths without a stem

**File:** [crates/sifr_driver/src/project/discovery.rs:255-262](crates/sifr_driver/src/project/discovery.rs:255-262)

Original `path.file_stem().unwrap()` would panic if `file_stem()` returned `None`. New code does `continue` instead.

`Path::file_stem()` returns `None` only when the final path component is `..`, which `read_dir` won't produce (it filters self/parent). The producer at [discovery.rs:235-247](crates/sifr_driver/src/project/discovery.rs:235-247) further filters by `extension == "sifr"`, which is impossible for `..` anyway. So the `continue` branch is unreachable today.

The behavioral divergence (`continue` vs panic) only surfaces if some future caller bypasses `discover_project_sifr_files` or the dir layout changes. In that hypothetical, silently dropping a file is arguably worse than a panic — but it's a low-impact edge case in a pure discovery routine, not codegen.

---

### 🟢 Very Low — Test-runner comment fallback uses full path

**File:** [crates/sifr_driver/src/test_runner/orchestrator.rs:125-130](crates/sifr_driver/src/test_runner/orchestrator.rs:125-130)

The result is purely a `// Tests from: <name>` comment in the generated Rust source. Original would panic for paths ending in `..`; new code falls back to `test_file.display().to_string()` (full path with directory components). The fallback is a generated comment, so the worst case is a slightly noisier comment line. **No functional impact, no regression.**

---

## Cross-cutting observations

### 1. The replacement style uniformly weakens diagnostics

Six of the eight code-path changes (the three `Some(...)` envelope rewrites in codegen, the Compare lowering, the numeric-truthiness lowering, the Compare get-by-index) replace an `expect`/`unwrap` whose only role was to assert an *invariant maintained by code a few lines up*. The originals were intentional "this can never happen" panics — exactly the pattern AGENTS.md endorses ("`assert!` is only for programmer invariants").

The new code converts those panics into one of:
- `return Ok(None)` → silently bail out and let the caller emit different code,
- `return RustExpr::FnCall { func, args: vec![] }` → silently emit invalid `Some()`,
- `Type::Never` fallback → silently produce a semantically wrong type.

This swaps "noisy compile-time bug" for "subtle codegen divergence". For a compiler whose stated guarantee is "if it compiles, it works," this is the wrong direction.

The clippy `unwrap_used`/`expect_used` lints are not opposed to this guarantee — they're meant to push *user-input-driven* unwraps toward `Result`. For programmer-invariant assertions in compiler internals, the project-correct replacement is `unreachable!()` (which clippy permits) or a `#[allow(clippy::expect_used)]` with a comment, not a silent fallback.

### 2. The change set is mechanically correct but doesn't audit upstream guards

For each `expect`/`unwrap` removed, the only thing that makes the new code safe is an invariant established a few lines above. None of those invariants are commented or asserted independently. If a future refactor ever loosens (e.g.) the `ops.len() == comparators.len()` guard, the dead branch becomes live and produces wrong output silently. Adding `unreachable!()` instead of a fallback both satisfies the lint and pins the invariant.

### 3. Lint enablement is partial

The lint is on, but ~1009 hits in test code remain unaddressed. Either pre-suppress them per-target/per-crate, or finish the burndown before flipping the lint. Leaving it half-on creates a tax on future test edits and a tripwire for `--all-targets`.

---

## Actionable recommendations

In rough priority order:

1. **Replace silent fallbacks with `unreachable!()`** at these six sites — they all assert invariants, none should silently degrade:
   - [class_method_emitter.rs:99-102](crates/sifr_codegen/src/class_method_emitter.rs:99-102)
   - [intrinsic_method_emitters.rs:629-632](crates/sifr_codegen/src/intrinsic_method_emitters.rs:629-632)
   - [stmt_support_emitter.rs:3803](crates/sifr_codegen/src/stmt_support_emitter.rs:3803)
   - [stmt_support_emitter.rs:5061](crates/sifr_codegen/src/stmt_support_emitter.rs:5061)
   - [stmt_support_emitter.rs:6340-6343](crates/sifr_codegen/src/stmt_support_emitter.rs:6340-6343)
   - [union.rs:30](crates/sifr_type_system/src/union.rs:30)

2. **Decide on the test-target lint policy** before this lands:
   - **Option A** (recommended for a small change): add `#![allow(clippy::unwrap_used, clippy::expect_used)]` at the top of each `*_tests.rs` / `tests/*.rs` module, scoped under `#[cfg(test)]`.
   - **Option B**: leave the lints at `warn` (current) but add a comment in `Cargo.toml` flagging that `--all-targets` is intentionally still dirty until follow-up cleanup.
   - **Option C**: revert this commit's `Cargo.toml` change until the test-side cleanup is staged.

3. **Restructure `make_union`** so the single-element arm doesn't have a dead `Type::Never` branch — either pop without the surrounding `match members.len()` arm, or keep the original `unwrap()` with `#[allow(clippy::unwrap_used)]` and a one-line justification comment.

4. **Keep** the four refactors that strictly improve the code with no risk:
   - [statements.rs:953-973](crates/sifr_hir/src/lower/statements.rs:953-973) (pattern field lookup `let-else`)
   - [statements.rs:1903-1924](crates/sifr_hir/src/lower/statements.rs:1903-1924) (narrowing single-element)
   - [nested_function_inference.rs:772-775](crates/sifr_hir/src/lower/nested_function_inference.rs:772-775) (state lookup `let-else`)
   - [discovery.rs:255-262](crates/sifr_driver/src/project/discovery.rs:255-262) (file_stem skip)
   - [orchestrator.rs:125-130](crates/sifr_driver/src/test_runner/orchestrator.rs:125-130) (comment fallback)

5. **No tests added.** The cleanup doesn't change observable behavior, but the new dead-branch fallbacks now exist in the binary. Consider adding focused unit tests that exercise `make_union(vec![Type::Int])`, `ensure_some_box_inner` with a typical `Some(x)`, and the numeric-truthiness condition lowering for `int | None`, so a future regression in the upstream guard is caught at the level of the helper rather than as a downstream e2e drift.

---

## Risk summary

| Concern | Severity | Triggered by |
|---|---|---|
| `--all-targets` clippy fails with ~1009 warnings | Medium | Any local clippy run with `--all-targets` |
| Codegen helpers silently emit invalid `Some()` if invariants drift | Low | Future refactor that loosens an outer guard |
| Compare/numeric-truthiness lowering silently bails to generic path | Low | Same — invariant drift |
| `make_union` dead `Type::Never` branch is misleading | Very Low | Code reading / future refactor |
| Behavior under current invariants | **None** | — |

No correctness regression in the current, in-tree state. The risk is in the future: the cleanup makes the codegen pipeline quieter when its invariants break, which is a step backward for a compiler that needs loud failures during development.
