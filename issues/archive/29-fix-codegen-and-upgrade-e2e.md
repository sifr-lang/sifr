## Fix M3 Codegen Bugs and Upgrade E2E Test Harness

#### **Current Situation**

- E2E pass tests only verify that sifr emits Rust code containing `fn main()` -- they never compile the generated Rust with `rustc` or run the binary.
- Several M3 codegen bugs went undetected because of this gap:
  - Union types (e.g., `int | str`) reference an `IntOrStr` enum that is never generated.
  - `return None` in Option-returning functions emits `return ()` instead of `return None`.
  - `return "value"` in Option-returning functions is not wrapped in `Some()`.
  - `let x = find_user(...)` double-wraps in `Some()` when the function already returns `Option<T>`.
  - `if user is not None: print(user)` emits `if user.is_some() { println!("{}", user) }` which fails because `Option<String>` doesn't implement `Display`.
  - `isinstance(x, int)` emits `if true` instead of matching on the union enum variant.
  - Calling a function with an `Option<T>` param using a plain `T` value doesn't wrap in `Some()`.
  - `if x:` where `x` is `Option<T>` emits `if x` instead of `if let Some(x) = x`.
- The `tuple_unpack.sifr` test uses literal `\n` in `# expect-stdout:` instead of multiple lines.

#### **Desired Situation**

- E2E pass tests fully verify generated Rust by:
  1. Compiling the generated Rust with `cargo build`
  2. Running the binary and capturing stdout
  3. Comparing stdout against `# expect-stdout:` annotations
- All M3 codegen bugs are fixed so that generated Rust compiles and runs correctly.
- All 27 pass tests and 5 fail tests pass with the upgraded harness.

#### **Implementation Notes**

**Codegen fixes (`sifr_codegen/src/lib.rs`):**
- Generate actual Rust `enum` definitions with `Display` impl for non-Option union types
- Use `match` for isinstance narrowing on union enums
- Use `if let Some(var) = var` for `is not None` / `is None` / truthiness narrowing on Option types
- Wrap return values in `Some()` when function returns `Option<T>`
- Emit `return None` instead of `return ()` for Option return types
- Don't double-wrap `Some()` when RHS already returns `Option<T>`
- Wrap call arguments in `Some()` when parameter is `Option<T>` and argument is plain `T`
- Wrap call arguments in enum variants when parameter is a union type

**HIR fix (`sifr_hir/src/lower.rs`):**
- Pass the type name as a second argument in isinstance HIR calls so codegen can generate match arms

**E2E harness (`crates/sifr/tests/e2e.rs`):**
- Build generated Rust into a binary via a temp Cargo project
- Run the binary and verify stdout against `# expect-stdout:` annotations
- Report all failures together with generated Rust and rustc errors

**Test fix (`tuple_unpack.sifr`):**
- Use multiple `# expect-stdout:` lines instead of literal `\n`

#### **Acceptance Criteria**

- [ ] All 27 E2E pass tests compile their generated Rust with `rustc` and run successfully
- [ ] All stdout assertions match expected output
- [ ] All 5 E2E fail tests still pass
- [ ] All 3 milestone demos (M1, M2, M3) compile and run via `cargo run -- run`
