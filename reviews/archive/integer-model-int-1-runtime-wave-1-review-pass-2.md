# Review: INT-1 Runtime Wave 1 — `sifr_runtime` substrate and codegen plumbing (Pass 2)

Reviewer: Claude Opus 4.7
Date: 2026-05-05
Phase: `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`, milestone INT-1
Pass-1 artifact: `reviews/integer-model-int-1-runtime-wave-1-review-pass-1.md`
Design source of truth: `internal_docs/integer_model.md`

## Verdict: SATISFIED

Both pass-1 blockers are resolved within the wave-1 substrate scope, and every follow-up the user enumerated is in place. No remaining correctness blockers for this wave.

---

## B1 — `sifr_runtime` Cargo dependency path resolution

Resolved in [crates/sifr_codegen/src/lib.rs:83-124](crates/sifr_codegen/src/lib.rs:83) and mirrored in [crates/sifr/tests/e2e.rs:1249-1290](crates/sifr/tests/e2e.rs:1249).

`sifr_runtime_dependency_spec()` now defers to `discover_sifr_runtime_path()`, which tries in order:

1. `SIFR_RUNTIME_PATH` env var, gated on `<path>/Cargo.toml` actually being a file ([lib.rs:92-98](crates/sifr_codegen/src/lib.rs:92)).
2. Walk up from `env::current_dir()` looking for `<ancestor>/crates/sifr_runtime/Cargo.toml` ([lib.rs:100-104](crates/sifr_codegen/src/lib.rs:100), [lib.rs:112-117](crates/sifr_codegen/src/lib.rs:112)).
3. Walk up from `env::current_exe()` with the same predicate ([lib.rs:106-110](crates/sifr_codegen/src/lib.rs:106)).
4. Compile-time in-tree fallback joining `CARGO_MANIFEST_DIR`'s parent with `sifr_runtime` ([lib.rs:119-124](crates/sifr_codegen/src/lib.rs:119)).

Each candidate is filtered against an actual `Cargo.toml` predicate before being accepted, so the absolute-path emission can no longer silently bake a broken pointer when SIFR_RUNTIME_PATH is set incorrectly. The compile-time fallback is explicitly scoped as the in-tree last resort, which is consistent with the user's framing of this resolution and acceptable for a substrate-only wave that does not yet emit `SifrInt` from any lowering path.

The same chain is mirrored verbatim in `crates/sifr/tests/e2e.rs` so the in-tree e2e harness picks up `SIFR_RUNTIME_PATH` and ancestor discovery without the codegen crate. Pass-1's note about logical duplication between the two copies still stands as a non-blocking maintenance concern (any later switch — e.g., to a published runtime crate — needs both edited together), but it is out of scope for this verification pass and was not on the user's follow-up list.

## B2 — `SifrInt::hash` no longer allocates for `Small`

Resolved in [crates/sifr_runtime/src/int.rs:337-353](crates/sifr_runtime/src/int.rs:337). The `Hash` impl now branches on the variant directly and feeds bytes into a shared `hash_normalized_integer_parts` helper ([int.rs:387-393](crates/sifr_runtime/src/int.rs:387)):

- `Small(v)` → `v.is_negative()` plus `v.unsigned_abs().to_be_bytes()` (stack 8-byte array, no `BigInt`, no `String`), then leading-zero-stripped + length-prefixed write.
- `Big(v)` → `BigInt::to_bytes_be()` (the unavoidable magnitude allocation that exists for any byte-form encoding of a `Big`), then the same canonical write.

Cross-form consistency holds: tracing `SifrInt::Small(1).hash(state)` and `NormalizedIntegerHash::from_signed(1).hash(state)` both reduce to `false.hash + write_usize(1) + state.write(&[1])` (the derived `Hash` on `NormalizedIntegerHash` ultimately calls `<[u8]>::hash` which is `write_length_prefix(len)` + `state.write(slice)`, identical to the explicit shape). The `equality_ordering_and_hashing_are_normalized` test at [int.rs:443-450](crates/sifr_runtime/src/int.rs:443) continues to lock the `Small`/`Big` agreement, and the `from_signed`/`from_unsigned` byte form at [int.rs:48-64](crates/sifr_runtime/src/int.rs:48) matches what `Hash for SifrInt` writes — so the `hash(int(1)) == hash(int8(1))` design contract from `internal_docs/integer_model.md:198-203` is preserved while the per-`Small` allocations are gone.

Nice-to-have detail in the helper: `normalized_negative = negative && !magnitude.is_empty()` ([int.rs:388-389](crates/sifr_runtime/src/int.rs:388)) collapses signed/unsigned zero to the same hash, which is correct and was not strictly required.

---

## Follow-up verifications

### PartialEq/Ord avoid BigInt clones where possible

[int.rs:300-310](crates/sifr_runtime/src/int.rs:300) and [int.rs:318-335](crates/sifr_runtime/src/int.rs:318):

- `(Big, Big)` compares the inner `Box<BigInt>` directly with no clone.
- `(Small, Big)` / `(Big, Small)` for `PartialEq` uses `right.to_i64() == Some(*left)` — no clone, no allocation.
- `Ord` for `(Small, Big)` / `(Big, Small)` uses `to_i64()` first and only falls back to a sign check (`right.is_negative()` / `left.is_negative()`) when the `Big` is genuinely outside i64 range. Still no clone.

This is the shape recommended in pass-1's N2/N3 (with the sign check substituting for the canonical-invariant short-circuit, which is fine because it gives the same answer without leaning on the public-variant invariant). The `as_bigint()` accessor is still owned-`BigInt`, but the previously-allocating callers no longer use it on the hot paths.

### `Small` overflow uses `i128` middle-ground

`Add`/`Sub`/`Mul`/`Neg` on `(Small, Small)` route through `checked_*` and only on overflow promote to `Self::from_i128(i128::from(l) ⊕ i128::from(r))` ([int.rs:160-168](crates/sifr_runtime/src/int.rs:160), [int.rs:198-206](crates/sifr_runtime/src/int.rs:198), [int.rs:236-244](crates/sifr_runtime/src/int.rs:236), [int.rs:274-282](crates/sifr_runtime/src/int.rs:274)). `from_i128` demotes back to `Small` when the result re-fits, so `i64::MAX + 1 - 1` no longer permanently allocates.

### Spill and overflow-then-cancel test coverage

[int.rs:469-489](crates/sifr_runtime/src/int.rs:469):

- `subtraction_multiplication_and_negation_spill_on_i64_overflow` exercises `i64::MIN - 1`, `i64::MAX * 2`, and `-i64::MIN` and asserts each produces a `Big` with the right textual value.
- `overflow_then_cancel_can_return_to_small` builds `(i64::MAX + 1) - i64::MAX` and asserts it demotes back to `Small(1)` — directly locking the `i128` middle-ground behavior in.

### `ir_imports` direct symbol coverage

[crates/sifr_codegen/src/ir_imports.rs:530-555](crates/sifr_codegen/src/ir_imports.rs:530) adds `collects_sifr_runtime_integer_symbols`, which builds a `RustItem::Fn` containing both `RustType::Named("SifrInt")` and a `RustExpr::Path(["sifr_runtime", "SifrInt", "from_i64"])` and asserts `needs.runtime.needs_sifr_int`. That covers both `mark_symbol` arms at [ir_imports.rs:436-437](crates/sifr_codegen/src/ir_imports.rs:436) directly, closing pass-1's N4.

### `num-bigint` / `num-traits` workspace dependencies

[Cargo.toml:55-56](Cargo.toml:55) declares both crates under `[workspace.dependencies]`, and [crates/sifr_runtime/Cargo.toml:9-11](crates/sifr_runtime/Cargo.toml:9) consumes them via `{ workspace = true }`. The user-emitted Cargo manifest sites in `sifr_codegen` and `sifr/tests/e2e.rs` still hard-code the version strings (those go into generated user projects, which can't reference our `[workspace.dependencies]`), but that is the intended split — the runtime crate itself is now in lock-step with the workspace.

---

## Observations (not blockers, not on the requested list)

- The duplicated `sifr_runtime_dependency_spec` / `discover_sifr_runtime_path*` block lives both in `crates/sifr_codegen/src/lib.rs:83-124` and `crates/sifr/tests/e2e.rs:1249-1290`. Any future change to the resolution chain has to be applied in both. Re-exporting the codegen helper would remove that drift surface.
- The cargo-toml-emission tests still assert only the `sifr_runtime = { path = ` prefix; they don't lock the resolved path's well-formedness. That is fine for substrate-only validation, but worth tightening once a real runtime-link mechanism (registry crate, vendored copy, or relocation logic) replaces the in-tree fallback.
- The `equality_ordering_and_hashing_are_normalized` test still hand-constructs `Big(Box::new(BigInt::from(1_i64)))`, exercising the cross-form invariant. Pass-1's N1 (canonicalization invariant) is unchanged. The `PartialEq`/`Ord` arms now handle this case allocation-free regardless of which side of N1 is eventually picked.

None of the above are blockers for this wave.

---

## Validation matched against the user's listed gates

The validations listed by the user (`cargo fmt`, `cargo test -p sifr_runtime`, `cargo clippy -p sifr_runtime -- -D warnings`, `cargo test -p sifr_codegen collects_sifr_runtime_integer_symbols`, `cargo test -p sifr_codegen test_generate_project_emits_sifr_runtime_path_dependency_when_required`, `cargo test -p sifr_driver test_generate_test_runner_cargo_toml_includes_required_crates`, `cargo test -p sifr test_generate_cargo_toml_required_sifr_runtime_uses_path_dependency`) all map to surfaces present in this diff:

- The new `sifr_runtime` tests at [int.rs:403-510](crates/sifr_runtime/src/int.rs:403) cover the ten unit cases — the original eight plus the two added for sub/mul/neg spills and overflow-then-cancel.
- The new `ir_imports` test at [ir_imports.rs:530-555](crates/sifr_codegen/src/ir_imports.rs:530) covers the symbol-detection arms.
- The three Cargo-emission tests cover the `sifr_runtime = { path = ` shape (prefix only — see Observation 2).

`scripts/run_all_tests.sh --profile quick` is the right authoritative gate; the user reports it running after these fixes.

---

## Recommended next step

Land the wave. Track the deferred items (path-resolution test tightening, codegen/e2e dedup, N1 invariant choice, optional `as_bigint()` → `Cow<'_, BigInt>`, and the parser/error-message cleanup from pass-1 N9/N13) under INT-1 wave 2 or the appropriate later milestone — none of them block lock as wave 1.
