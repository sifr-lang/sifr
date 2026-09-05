# Review: INT-1 Runtime Wave 1 — `sifr_runtime` substrate and codegen plumbing (Pass 1)

Reviewer: agent
Date: 2026-05-05
Phase: `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`, milestone INT-1
Design source of truth: `internal_docs/integer_model.md`

## Verdict: NEEDS-CHANGES (one structural blocker, one performance blocker, several quality follow-ups)

The diff lands the right substrate for INT-1 wave 1 — workspace-member crate, exact-`SifrInt` enum with normalized comparison/ordering/hashing, `parse_decimal` with a digit-limit guard, and `sifr_runtime` plumbing into the generated-project Cargo manifest path. The shape is correct and the diff is appropriately scoped to "substrate, not lowering."

But two findings block lock as wave 1:

- **B1.** The `sifr_runtime` Cargo dependency is emitted as an absolute filesystem `path = "..."` baked in at codegen build time via `env!("CARGO_MANIFEST_DIR")`. This works for in-workspace tests but is non-portable for any installed/distributed `sifr` binary. Wave 1 ships the only mechanism that wave 2+ generated user code will rely on, so this needs an answer (or an explicit deferral plan with a tracked follow-up) before lowering starts emitting `SifrInt`.
- **B2.** `SifrInt::hash` allocates two `BigInt`s and a `String` per call for **every** `Small` value, directly contradicting INT-1's "Small integer construction and simple reuse do not allocate on the big-integer path" acceptance criterion. As soon as INT-3/INT-4 land hash/dict/set keys for `int`, this becomes a measurable regression vs. the design's no-allocation contract.

The remaining items are non-blocking but should be addressed before INT-2A starts depending on these surfaces.

---

## Files reviewed

- `Cargo.toml` (workspace member + workspace dependency entry).
- `crates/sifr_runtime/Cargo.toml`, `crates/sifr_runtime/src/lib.rs`, `crates/sifr_runtime/src/int.rs` (new crate).
- `crates/sifr_codegen/src/ir_imports.rs` (symbol → import-needs detection).
- `crates/sifr_codegen/src/lib.rs` (`generate_project_with_deps_and_crates`, `sifr_runtime_dependency_spec`, import emission).
- `crates/sifr_codegen/src/entrypoints.rs` (`generate_rust_test` mirror of import emission).
- `crates/sifr_codegen/src/lib_codegen_tests.rs` (`test_generate_project_emits_sifr_runtime_path_dependency_when_required`).
- `crates/sifr_driver/src/tests/test_runner.rs` (Cargo manifest test — added `sifr_runtime` to required crates).
- `crates/sifr/tests/e2e.rs` (dependency inference and Cargo emission for the e2e harness).

---

## Scope check against INT-1 wave 1

This wave correctly limits itself to the runtime substrate. Source-level `int` is not yet lowered to `SifrInt`; HIR/type-system are untouched; no e2e fixtures churn. That matches the user-stated framing and the milestone's "introduce the exact integer runtime representation without changing every integer operator at once" goal.

What this wave delivers against INT-1's scope (`issues/...md` lines 96-106):

| Scope item | Delivered |
| --- | --- |
| Create `crates/sifr_runtime` workspace crate | Yes |
| Generated Cargo manifest links the runtime crate | Yes (with caveat B1) |
| Canonical `SifrInt` with `Small(i64)` / `Big(Box<BigInt>)` | Yes |
| Construction from primitives + decimal strings with digit limits | Yes |
| Clone, equality, ordering, hashing, formatting, basic conversions | Yes (with caveat B2 on hashing performance) |
| Normalized integer hashing helpers for fixed-width keys | Yes |
| Source-level `int` value-semantic over non-`Copy` runtime | Deferred to wave 2 (HIR/codegen lowering not touched) |
| Generated-code panic-shape tests for runtime integer paths | Partial (parse_decimal covered; no broader sweep) |

Deferring the lowering and panic-shape sweep to wave 2 is reasonable. Calling out "wave 1 substrate" explicitly in the eventual PR description would help reviewers and the INT-1 closure check.

---

## Blocking findings

### B1. Generated `sifr_runtime` dependency uses a non-portable absolute path

`crates/sifr_codegen/src/lib.rs:82-93`:

```rust
fn sifr_runtime_dependency_spec() -> String {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let runtime_path = manifest_dir
        .parent()
        .map(|parent| parent.join("sifr_runtime"))
        .unwrap_or_else(|| manifest_dir.join("../sifr_runtime"));
    let escaped_path = runtime_path
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    format!("sifr_runtime = {{ path = \"{escaped_path}\" }}")
}
```

`env!("CARGO_MANIFEST_DIR")` evaluates at compile time of `sifr_codegen` to the *developer's* absolute crate-source path. That literal is then baked into the released `sifr` binary and stamped, verbatim, into every generated user Cargo.toml that needs `sifr_runtime`. The same function is duplicated in `crates/sifr/tests/e2e.rs:1249-1260`.

This is fine for in-tree validation because the embedded absolute path *happens* to point to a real `crates/sifr_runtime/` directory on the dev's machine. The four tests already on the green path (`generate_project_emits_sifr_runtime_path_dependency_when_required`, `generate_test_runner_cargo_toml_includes_required_crates`, `generate_cargo_toml_required_sifr_runtime_uses_path_dependency`) only assert the *prefix* `sifr_runtime = { path = ` and never validate that the path is portable, machine-relative, or installable. They will pass everywhere and silently allow the released compiler to ship broken Cargo manifests.

Failure modes:

- A `cargo install --path crates/sifr` build on machine A, copied to machine B, and run as `sifr build foo.sifr` would emit a Cargo.toml with a `path` that does not exist on B.
- Even on machine A, moving the workspace to a new location after a release build invalidates every produced manifest.
- The same applies to any tarball/Homebrew/CI distribution. The compiler has no general way to relocate the dependency.

This is not exercised end-to-end today because no codegen path emits the `SifrInt` symbol or `sifr_runtime::` use yet. The detection rule (`mark_symbol("SifrInt"|"sifr_runtime", ...)` in `ir_imports.rs:436-437`) cannot fire on output that was never produced. So the latent bug only becomes user-visible the moment INT-2A or INT-3 starts emitting `SifrInt` paths in generated code.

Recommended directions (any of these resolves it):

1. **Defer the runtime-link mechanism** explicitly. Document in the issue that `sifr_runtime` is not consumed by user-built artifacts in INT-1 wave 1; the absolute-path manifest exists only for in-tree integration tests, and a portable mechanism (registry publish, vendored sources, runtime-relative path discovery via `sifr` binary location) is a tracked INT-1 wave 2 prerequisite for any milestone that actually emits `SifrInt`. Then add a runtime guard so non-test calls error or skip the `path = "..."` emission with a clear "not yet supported" diagnostic.
2. **Resolve the path at runtime** from the running `sifr` binary's location (e.g., walk upward looking for `crates/sifr_runtime/Cargo.toml`, with a clear failure message if not found). This keeps the dev-shell story working without baking a build-time absolute path.
3. **Vendor `sifr_runtime` source into each generated project** — strictly worse for codegen size/duplication but it removes the host-dependency. The design doc explicitly favors a shared crate over per-file vendoring, so this is the worst option.
4. **Publish `sifr_runtime` to crates.io** (or an internal registry) and emit `sifr_runtime = "x.y.z"`. This matches how `num-bigint`, `rust_decimal`, etc. are handled today. Probably the eventual answer, but requires versioning, publishing, and a CI gate.

For wave 1, option 1 (deferral with a runtime guard so the broken path can never escape) is the smallest correct change. The validation tests should then assert the *current* contract — "in-tree only" — rather than just that the string starts with `sifr_runtime = { path = `.

Also worth noting: the dependency-spec function is now duplicated verbatim across `crates/sifr_codegen/src/lib.rs:82-93` and `crates/sifr/tests/e2e.rs:1249-1260`. Two copies, identical logic, two places to drift. The natural home is `sifr_codegen` with a `pub` re-export so `crates/sifr/tests/e2e.rs` shares it; that also gives a single place to apply whichever resolution above gets picked.

### B2. `SifrInt::hash` allocates twice per `Small` hash

`crates/sifr_runtime/src/int.rs:101-108, 318-322`:

```rust
pub fn normalized_hash_key(&self) -> NormalizedIntegerHash {
    let value = self.as_bigint();           // BigInt::from(*small) — allocates
    NormalizedIntegerHash {
        negative: value.is_negative(),
        magnitude_decimal: value.abs().to_str_radix(10),  // String — allocates
    }
}
// ...
impl Hash for SifrInt {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.normalized_hash_key().hash(state);
    }
}
```

For every `SifrInt::Small(_).hash(state)` call this:

1. allocates a fresh `BigInt` (`as_bigint` → `BigInt::from(*value)`),
2. allocates another `BigInt` (`value.abs()`),
3. allocates a decimal `String`,
4. drops all three.

INT-1's acceptance criteria explicitly include: "Small integer construction and simple reuse do not allocate on the big-integer path." Hashing a `Small` is the canonical "simple reuse" — every `dict[int, V]` lookup, every `set[int]` insert, every `int`-keyed cache. Three allocations per hash is not consistent with that contract.

The cross-family agreement (per `internal_docs/integer_model.md:198-203`: `hash(int(1)) == hash(int8(1))` whenever they compare equal) does require a canonical encoding. The current decimal-string normalization satisfies correctness but is unnecessarily expensive — a canonical magnitude byte form does the same work allocation-free for `Small`. Sketch:

```rust
impl Hash for SifrInt {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Small(v) => {
                v.is_negative().hash(state);
                // Normalize: hash big-endian magnitude bytes with leading-zero stripping,
                // matching the byte form used for Big and for the fixed-width helpers.
                hash_magnitude_be(state, &v.unsigned_abs().to_be_bytes());
            }
            Self::Big(v) => {
                v.is_negative().hash(state);
                let (_, mag_be) = v.to_bytes_be();
                hash_magnitude_be(state, &mag_be);
            }
        }
    }
}
```

`NormalizedIntegerHash::from_signed`/`from_unsigned` would adopt the same canonical form so that `int(1)` / `int8(1)` / `uint8(1)` agree without anyone visiting decimal text. This is a refactor in `int.rs` only; nothing else needs to change.

If this work is genuinely INT-8 territory and you want to keep it deferred, then INT-1 should at minimum carry a regression test that documents the current allocation count (e.g., a `#[ignore]`'d allocator-probe test) so that someone landing INT-3/INT-4 can't merge dict/set integration without explicitly addressing it. Right now there is no signal at all; the design's allocation guarantee is silently violated by a unit-tested-and-passing implementation.

---

## Non-blocking but worth fixing in this wave

### N1. `Big`-vs-`Small` canonicalization is enforced by constructors but not by the public type

All constructors in `int.rs` (`from_i64`, `from_i128`, `from_u128`, `from_bigint`, `parse_decimal`, the arithmetic spill paths) demote to `Small` whenever the value fits in `i64`. So in canonical use, `Big(_)` *only* holds values outside `i64`'s range, and `Small != Big` is therefore decidable in O(1) without reaching for `as_bigint()`.

But the variants are `pub`, so generated/external code can hand-construct `SifrInt::Big(Box::new(BigInt::from(1_i64)))` and break the invariant. The unit test at `int.rs:386-394` does exactly this on purpose to verify cross-form normalization. The design doc anticipated this (`internal_docs/integer_model.md:44-46`):

> If the implementation wants freedom to change the layout later, the variants should be hidden behind constructors and accessors from the first runtime slice.

There are two coherent options:

1. **Document the invariant and lean on it.** Make the variants `pub` (as today), document "constructed `Big(x)` for `x` fitting in i64 is malformed and behavior is unspecified," and replace the cross-form unit test with one that goes through `from_bigint`. Then `PartialEq` and `Hash` can short-circuit `(Small, Big)` to `false` / disjoint hash without an `as_bigint()` allocation.
2. **Hide the variants.** Make them `pub(crate)` and expose constructors / accessors (`SifrInt::small(i64) -> Self`, `SifrInt::try_as_small(&self) -> Option<i64>`, etc.). This preserves the option to change the layout later (e.g., add `Inline128(i128)` or pack the discriminant) without breaking generated projects that have been built against the old enum.

Either is a net improvement. Today's middle ground (variants public, equality routes everything through `as_bigint()`) costs allocations on every mixed-variant compare for no semantic benefit. Pick a side.

### N2. `PartialEq` and `Ord` allocate unnecessarily on the `Big`/`Big` path

`int.rs:294-316`:

```rust
impl PartialEq for SifrInt {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Small(left), Self::Small(right)) => left == right,
            _ => self.as_bigint() == other.as_bigint(),
        }
    }
}
// Ord identical pattern.
```

The `_` arm fires for `(Big, Big)`, `(Small, Big)`, and `(Big, Small)`. For `(Big, Big)`, `as_bigint()` clones both inner `BigInt`s for the comparison even though `BigInt` already implements `Eq` / `Ord` directly. Cleaner:

```rust
match (self, other) {
    (Self::Small(l), Self::Small(r)) => l == r,
    (Self::Big(l), Self::Big(r)) => l == r,                    // no clone
    (Self::Small(_), Self::Big(_)) | (Self::Big(_), Self::Small(_)) => false, // by N1 invariant
}
```

If you don't want to commit to N1's invariant, the `(Small, Big)` arm can compare via `BigInt::to_i64()` without allocation. Either way, no clone is needed. Same shape applies to `Ord::cmp`.

### N3. `as_bigint()` clones on `Big` even when borrowing would suffice

`int.rs:110-116`:

```rust
pub fn as_bigint(&self) -> BigInt {
    match self {
        Self::Small(value) => BigInt::from(*value),
        Self::Big(value) => value.as_ref().clone(),
    }
}
```

Every internal user (`PartialEq`, `Ord`, `Hash`) has to take that owned `BigInt`. A `Cow<'_, BigInt>` accessor would let the `Big` arm hand out a borrow:

```rust
pub fn as_bigint(&self) -> Cow<'_, BigInt> {
    match self {
        Self::Small(v) => Cow::Owned(BigInt::from(*v)),
        Self::Big(v) => Cow::Borrowed(v),
    }
}
```

Combined with N2, this drops the `Big`/`Big` comparison/hash from "two clones" to "no clones."

### N4. Symbol detection has no unit test

`crates/sifr_codegen/src/ir_imports.rs:436-437` adds two new `mark_symbol` arms (`"SifrInt"` and `"sifr_runtime"` → `needs.runtime.needs_sifr_int = true`), but the existing `tests` module at the bottom of the same file only asserts coverage for `HashMap`, `HashSet`, `VecDeque`, `Mutex`, `BigInt`. The new arms are tested only transitively, through Cargo-toml emission tests that bypass the symbol pipeline entirely (the project test plumbs `"sifr_runtime"` straight into `required_crates`).

A regression where someone renames `SifrInt`, drops one of the two arms, or breaks the syn-path visitor would not fail any test. Add an `ir_imports::tests` case that constructs an item containing `SifrInt` (as `RustExpr::Ident` or `RustType::Named("SifrInt")`) and one that contains `sifr_runtime::SifrInt` (as `RustExpr::Path` or a typed path), and asserts `needs.runtime.needs_sifr_int` is set in both.

### N5. The `generate_rust_test` import path mirrors the main path but is otherwise untested

`crates/sifr_codegen/src/entrypoints.rs:101-106, 141-143` adds the same `needs_sifr_int` → `use sifr_runtime::SifrInt;` + `required_crates` plumbing as `generate_rust_with_stdlib`. There is no unit test that exercises `generate_rust_test`'s SifrInt branch end-to-end. Given that the full project test only exercises `generate_project_with_deps_and_crates`, the test-codegen path is currently uncovered. A small input → output snapshot or assertion in `lib_codegen_tests.rs` for the test-mode path would close the gap.

### N6. `num-bigint` / `num-traits` versions are pinned in three places

The same `num-bigint = "0.4.6"` and `num-traits = "0.2.19"` strings now appear in:

- `crates/sifr_runtime/Cargo.toml:10-11` (the new crate)
- `crates/sifr_codegen/src/lib.rs:942-944, 1041-1048` (codegen-emitted user manifests)
- `crates/sifr/tests/e2e.rs:1211-1216` (e2e generator)

Pre-existing duplication (codegen + e2e) is not a wave-1 regression, but the new runtime crate introduces a third site that needs to stay in lock-step. Cargo's resolver will usually paper over a minor mismatch, but if any of these drifts to an incompatible version the generated user crate will see two `num_bigint::BigInt` types and the runtime helpers will silently stop being interchangeable with handwritten BigInt usage.

The natural fix is a single `[workspace.dependencies]` entry for `num-bigint` and `num-traits` in the root `Cargo.toml`, with all three sites referencing it. That's a small refactor and the wave already touches each of those files.

### N7. Arithmetic spill skips the `i128` middle ground

`int.rs:151-238` overflow-fallback paths reach for `BigInt` directly:

```rust
(Self::Small(left), Self::Small(right)) => left.checked_add(right).map_or_else(
    || Self::from_bigint(BigInt::from(left) + BigInt::from(right)),
    Self::Small,
),
```

For `i64 + i64` the mathematical result is always representable in `i128`. Routing through `i128` lets the result re-demote to `Small` when subsequent operations bring the magnitude back down, e.g. `i64::MAX + 1 - 1`. Today that sequence permanently allocates a `BigInt`.

Sketch:

```rust
(Self::Small(l), Self::Small(r)) => left.checked_add(r).map_or_else(
    || Self::from_i128(i128::from(l) + i128::from(r)),  // demotes when it fits
    Self::Small,
),
```

Same shape for `sub`, `mul`, `neg`. This stays in scope for INT-1 because `from_i128` already exists and demotes correctly. It avoids an allocation on a meaningful chunk of overflow-then-cancel sequences without changing observable semantics.

### N8. Missing runtime tests for spill on `Sub`, `Mul`, and `Neg`

`int.rs` has `arithmetic_spills_on_i64_overflow` for `Add` only. The other three operators have identical `checked_*` → `BigInt` patterns, but no test for `i64::MIN.neg() → Big`, `i64::MIN - 1 → Big`, or `i64::MAX * 2 → Big`. Each of those is a one-line addition and they're cheap insurance against a future refactor of the operator template.

### N9. `IntegerParseError` is debug-quality

`int.rs:11-31`:

```rust
pub enum IntegerParseError {
    Empty,
    InvalidDigit,
    DigitLimitExceeded { limit: usize, actual: usize },
}
```

`InvalidDigit` carries no position or character; `Empty` collapses three different inputs (`""`, `"+"`, `"-"`) into one variant. For wave 1 this is fine — the design's user-facing diagnostics will sit at the parser/HIR boundary, not at this runtime layer. But once `parse_decimal` is called from the JSON / CSV / env / URL boundaries in INT-5, the wrappers will need at least the position of the offending byte to produce useful error messages. Worth a TODO comment so the next milestone doesn't have to retrofit through public API churn.

### N10. `Path` fallback in `sifr_runtime_dependency_spec` is dead code

`crates/sifr_codegen/src/lib.rs:84-87`:

```rust
let runtime_path = manifest_dir
    .parent()
    .map(|parent| parent.join("sifr_runtime"))
    .unwrap_or_else(|| manifest_dir.join("../sifr_runtime"));
```

`manifest_dir` is a Cargo crate's source directory; `.parent()` returns `None` only when the path is `/`, which is impossible for a real crate. The fallback is unreachable. It's there to dodge the `expect_used` workspace lint. A clearer pattern is `manifest_dir.parent().unwrap_or(manifest_dir).join("sifr_runtime")` or, if you take B1's option (1) and gate this whole function behind a runtime test-only check, the fallback disappears entirely. Same comment applies to the duplicated copy in `crates/sifr/tests/e2e.rs:1249-1260`.

### N11. `sifr_runtime/Cargo.toml` does not use the workspace dependency aliases

The new manifest declares `num-bigint = "0.4.6"` directly. The rest of the workspace uses `{ workspace = true }` for shared deps. This compounds N6 — moving these to `[workspace.dependencies]` and switching `sifr_runtime/Cargo.toml` to `num-bigint = { workspace = true }` resolves both issues at once.

### N12. `sifr_runtime` lacks a `description`

Minor: `crates/sifr_runtime/Cargo.toml` has no `description` field. None of the other internal `crates/sifr_*/Cargo.toml`s do either, so this matches house style. Mention only because some future packaging step (e.g., publishing to crates.io to address B1's option 4) will need it.

### N13. `parse_decimal` walks the input twice

`count_decimal_digits` validates and counts, then `BigInt::from_str` re-parses. For a 4096-digit string this is two full passes. A combined loop that validates, counts, and parses in a single pass (or relies on `BigInt::from_str` to do the parsing while a precheck only looks at the byte length pre-stripping) would be cheaper. Not load-bearing for wave 1, but the parse path will be on the hot path for JSON ingestion in INT-5.

---

## Verification of provided commands

The user-listed validations all map to surfaces that exist in this diff:

- `cargo test -p sifr_runtime` — runs all 8 tests in `int.rs::tests` (positive: small stay, big spill, hash normalization, ordering, helper construction, arithmetic spill on add). Coverage gaps called out in N8.
- `cargo clippy -p sifr_runtime -- -D warnings` — the `#![cfg_attr(test, allow(clippy::expect_used))]` correctly silences `expect_err` use inside the test module under workspace `expect_used = "warn"`.
- `cargo test -p sifr_codegen test_generate_project_emits_sifr_runtime_path_dependency_when_required` — asserts only the `sifr_runtime = { path = ` prefix; does not validate path portability (B1) or the symbol-detection path (N4).
- `cargo test -p sifr_driver test_generate_test_runner_cargo_toml_includes_required_crates` — same prefix-only assertion.
- `cargo test -p sifr test_generate_cargo_toml_required_sifr_runtime_uses_path_dependency` — same prefix-only assertion.
- `cargo check -p sifr_codegen -p sifr --tests` — confirms the type-shape integrates cleanly.

I confirm none of these green commands would catch B1, B2, N4, or N5. They demonstrate "the wiring is in place" but not "the wiring produces correct, portable, performant output." That gap is normal for substrate-only waves; the recommendations above raise it from "implicit" to "tracked."

---

## Recommended path forward

1. Pick a resolution for **B1** — preferably explicit deferral with a runtime guard so the broken-on-distribution Cargo.toml can never escape the dev shell. Reflect the chosen approach in the INT-1 implementation checklist so wave 2 cannot start emitting `SifrInt` until the link mechanism is portable.
2. Fix **B2** by switching `Hash`/`NormalizedIntegerHash` to a magnitude-byte canonical form, with a regression test that hashing a `Small` does not allocate (allocator probe or counting allocator behind `cfg(test)`). If genuinely deferring to INT-8, add a `#[ignore]`'d regression test now so the deferral is materialized in the test suite, not in commit messages.
3. Tighten the public API per **N1** — pick "documented invariant" or "hidden variants" and follow through in `PartialEq` / `Ord` / `Hash`.
4. Close test gaps **N4**, **N5**, **N8**.
5. Resolve dependency-version duplication **N6** + **N11** by moving `num-bigint`/`num-traits` to `[workspace.dependencies]` and consuming via `{ workspace = true }`.
6. Apply optimization **N7** (i128 middle-ground), refactor cleanups **N2/N3/N10**, and queue **N9/N13** for the appropriate later milestone.

After these are addressed, this lands as a clean wave 1: the substrate, the dependency plumbing, and a defensible "no observable user behavior change yet, all the contracts the design promises are testable" story for wave 2 to build on.
