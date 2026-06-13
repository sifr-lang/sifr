## Phase 34 Wave 2 Round 3 Review — `codex/phase-34-emitted-audit-wave2`

**No blockers. Ready for PR/merge.**

---

### Change Inventory

| Change | File | Description |
|--------|------|-------------|
| IR rewrite: `while true` → `loop` | `ir_optimize.rs:633-641` | Converts `while True { ... break }` to native Rust `loop { ... break }` |
| IR rewrite: `.skip(0)` removal | `ir_optimize.rs:663-679, 772-781` | Eliminates no-op `.skip(0)` method calls from iterator chains |
| Empty string print lowering | `lower_stmt.rs:110-116` | Emits `MacroCall("println", [])` instead of a string-literal variant |
| Empty string print rendering | `render.rs:758-760` | Renders as `println!()` / `eprintln!()` without an empty `""` literal |
| Clippy allowlist reduction | `generated_code_quality.py` | Removed `while_true`, `iter_skip_zero`, `println_empty_string` |
| `pure_stdlib` variable rename | `demos/pure_stdlib/main.sifr:147-152` | Renamed `count0..count4` to avoid shadowing `c1..c8` Counter bindings |
| Bytes typed `Ok` | `bytes.rs:71-76` + all three fallible constructors | Emits `Ok::<Vec<u8>, ParseError/ValueError>(...)` for unambiguous standalone Result |

---

### Severity 1 — `while True` → `loop` (ir_optimize.rs)

**Correct and well-scoped.**

```rust
if matches!(cond, RustExpr::Literal(RustLiteral::Bool(true))) {
    *stmt = RustStmt::Loop { body: std::mem::take(body) };
}
```

- Only matches `RustExpr::Literal(RustLiteral::Bool(true))` — not boolean variables, comparisons, or function calls. Only Sifr's direct `while True:` literal condition triggers this.
- `std::mem::take(body)` transfers the body correctly — the old `While` statement is replaced in-place.
- Test `rewrites_true_while_to_loop` (line 939) verifies the transformation end-to-end.
- Pre-existing pattern from audit scan: `0287_find_the_duplicate_number.sifr` (1 occurrence). Now eliminated.

---

### Severity 1 — `.skip(0)` Removal (ir_optimize.rs)

**Correct and well-scoped.**

```rust
if method == "skip" && args.len() == 1 && is_zero_usize_expr(&args[0]) {
    let replacement = *std::mem::replace(receiver, ...);
    *expr = replacement;
    removed += 1;
}
```

- `is_zero_usize_expr` handles `0_i32`, `0_usize`, `(0)`, and transitively casts — matching exactly what Sifr's `.skip(0)` lowering produces.
- Replacing the outer method call with just the receiver is semantically correct: `xs.skip(0).take(n)` → `xs.take(n)`.
- Test `removes_zero_skip_method_call` (line 962) covers the `take(skip(0))` pattern.
- Pre-existing patterns from audit scan: LeetCode 0241 and 1849 (2 occurrences). Now eliminated.

---

### Severity 1 — Empty `println!` Lowering + Rendering

**Correct and minimal.**

Lowering (`lower_stmt.rs:110-116`):
```rust
[HirExpr::StringLiteral(value)] if value.is_empty() => {
    Some(RustStmt::Expr(RustExpr::MacroCall { name: "println".to_string(), args: vec![] }))
}
```

Rendering (`render.rs:758-760`):
```rust
if args.is_empty() && format_str.is_empty() && matches!(name.as_str(), "println" | "eprintln") {
    return format!("{name}!()");
}
```

- Exact match only: a format macro with no args and no format string becomes `println!()`. No broader changes.
- Handles both `println!` and `eprintln!` for symmetry.
- Tests: `renders_empty_println_format_macro_without_empty_string_literal` in `render.rs:1675` and `test_empty_string_print_emits_empty_println_macro` in `lib_codegen_tests.rs:903`. Both pass.
- Pre-existing patterns from audit scan: 4 demos (`data_structures`, `fibonacci`, `pure_stdlib`, `type_system`). Now eliminated.

---

### Severity 1 — Clippy Allowlist Reduction

**Verified against evidence.**

The reduced-allowlist clippy gate passed with all 71 manifest entries clean (`clippy-1778767148-95103.json`). The three removed allow entries directly correspond to the three IR rewrite optimizations above, and all three patterns are now eliminated from the passing corpus.

---

### Severity 2 — `pure_stdlib` Variable Rename

**Correct and confirmed.**

The original `main.sifr` used `c1`, `c2` as both `next()` output locals (typed `int | None`) and as `Counter[str]` variables. HIR kept the earlier `int | None` binding, so `c1` and `c2` in the Counter block resolved to `Option<i64>` instead of `Counter<String>`, producing a rustc E0277 error.

Renaming iterator locals to `count0..count4` (lines 147-152) frees `c1..c8` for Counter variables with correct types. The generated `from_list(...)` call returns `Counter<String>`, and `.get(&"a".to_string(), 0_i64)` produces i64 values — correct.

Evidence: `demos-wave2-failed-subset-after-pure-1778768309/report.jsonl` shows `pure_stdlib` passes all gates after this fix. Manual `cargo build` also confirms.

---

### Severity 2 — Bytes Typed `Ok` Constructors

**Correct and confirmed.**

Three fallible bytes constructors now emit `Ok::<Vec<u8>, ErrType>(...)` instead of bare `Ok(...)`:

- `lower_bytes_from_hex` → `Ok::<Vec<u8>, ParseError>(...)`
- `lower_bytes_with_size` → `Ok::<Vec<u8>, ValueError>(...)`
- `lower_bytes_from_ints` → `Ok::<Vec<u8>, ValueError>(...)`

The `typed_ok_expr` helper (`bytes.rs:71-76`) emits `Ok::<Vec<u8>, ErrorType>` as a path, which is valid Rust syntax. The `parse_map_err` path already uses `map_err` to convert to the correct error type, so the `Ok` is always compatible.

The `bytes_errors` demo now passes: `demos-wave2-failed-subset-after-bytes-1778769453/report.jsonl` shows 259 demos passing, 13 remaining pre-emitted-code failures.

Test `lowers_bytes_intrinsics_via_registry` passes, with assertions for `Ok::<Vec<u8>, ValueError>` and `Ok::<Vec<u8>, ParseError>` in the rendered output.

---

### Severity 3 — Pre-Existing Test Failures

The test suite has 60 pre-existing failures on main (e.g., `test_generate_rust_multi_assembles_single_rust_file` fails on both main and this branch). This is tracked separately and is not caused by wave 2 changes. All new tests introduced by this wave (5 total) pass.

---

### Summary

| Gate | Evidence | Result |
|------|----------|--------|
| `cargo fmt --check` | No output | Pass |
| `cargo check -p sifr_codegen` | Finished | Pass |
| `lowers_bytes_intrinsics_via_registry` test | ok | Pass |
| `renders_empty_println_format_macro_without_empty_string_literal` test | ok | Pass |
| `rewrites_true_while_to_loop` test | ok | Pass |
| `removes_zero_skip_method_call` test | ok | Pass |
| `test_empty_string_print_emits_empty_println_macro` test | ok | Pass |
| Reduced clippy allowlist gate | 71/71 clean | Pass |
| Demos sweep | 259/272 pass | Pass |
| Pre-emitted-code failures | 13 (all frontend/type/demo gaps) | Correctly classified |

All implementations are sound, targeted, and verified. No generated-code quality gaps remain in wave 2 scope.