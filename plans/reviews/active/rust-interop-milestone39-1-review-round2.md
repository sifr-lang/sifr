I've inspected the substantive new files (lowering pass, IR types, codegen plan, diagnostics) plus the call sites, registry tests, and parser carve-out. Round 1 blockers are addressed; remaining notes are minor.

# Phase 39 milestone_39_1 review (round 2)

## Round 1 follow-ups — resolved

1. **Cache key determinism** — `crates/sifr_codegen/src/rust_interop_plan.rs:12-29`. Replaced `{:?}` with `InteropBuildPlan::cache_key_fragment()`, which walks declarations in `Vec` order (driven by `compile_order` in `sifr_driver/src/build/project_codegen.rs:65`), encodes each field with a fixed delimiter scheme, serializes `TextRange` as `start..end`, and uses no `HashMap`/`HashSet`. Content-derived and deterministic. Good.
2. **Hyphenated family grammar** — `crates/sifr_diagnostics/src/codes/registry_tests.rs:159-163`. New assertion pins the grammar to either one segment or `RUST-<SEGMENT>` exactly. Round 1 concern about unbounded `A-B-C-D` chains is closed.

## Findings (minor)

### Low

**L1. `Self.X` rejected as an argument *value* even inside methods — `rust_interop.rs:212-213`**

`parse_value` hard-codes `RustInteropOwner::Function` when descending into `Expr::Attribute`, so `view=Self.DigestView` on a method's decorator would fail the `Self`-on-non-method check at `parse_target_path:301-309`. Argument values may not realistically use `Self.X` per the design today, but the inconsistency (positional `Self.poll` is allowed on a method while value `Self.X` is not) is silent and span-correct. Threading the actual owner through, or whitelisting `Self` only outside the owner-check, would make this uniform when later milestones add argument-position `Self` references. Not a blocker for M39.1 scope.

**L2. Negative integers can't be represented in the value grammar — `rust_interop.rs:197-211`**

`Expr::UnaryOp { USub, NumberLiteral(_) }` isn't matched, so `-1` would fall through to the generic "unsupported Rust interop decorator value" arm. No current grammar slot needs negatives, so this is documentation-quality only; worth a comment if signed integers are ever introduced.

**L3. Test coverage gaps for owner/kind cross-rejections — `crates/sifr_lowering/src/lower/rust_interop_tests.rs`**

Tests cover string targets, legacy `crate=/path=`, and `Self.X` outside methods. Not covered:
- `@rust.opaque` on a function (should hit `kind_allowed_on_owner` rejection)
- `@rust.async` / `@rust.zero_copy` / `@rust.view` on a class
- Unknown `@rust.unknown(...)` attribute (rejection path at `rust_interop.rs:90-97`)
- `**kwargs` rejection (`rust_interop.rs:167-174`)
- `@rust` used without parens (`classify_rust_decorator:60-66` — should raise "must be call expressions")

These are the unique rejection branches that don't currently have a fixture. Each is one short test like the existing ones.

**L4. Materialize unit test always uses empty interop plan — `crates/sifr_driver/src/build/materialize.rs:280` and `project_codegen.rs:132`**

Both touched tests construct `interop: InteropBuildPlan::default()`, so nothing in the test suite proves a non-empty plan participates in the cache key. The codegen unit test in `rust_interop_plan.rs:331-334` checks fragment substrings, but not the full cache-key path through `binary_project_cache_key`. A regression where the fragment is silently dropped from the cache string would still pass these tests. One additional assertion — same `GeneratedBinaryProject` with vs. without a declaration → distinct cache keys — would close this.

**L5. `RustInteropOwner` is reachable through public API but not re-exported — `crates/sifr_codegen/src/lib.rs:85-86`**

`pub use rust_interop_plan::{InteropBuildPlan, RustInteropPlan, RustInteropPlanDeclaration};` re-exports the struct that carries `owner: RustInteropOwner`, but the enum itself isn't in the export list. External consumers can read `decl.owner` but can't name the type to match on it. Adding `RustInteropOwner` to the `pub use` makes the API usable without leaking module paths.

### Informational

**I1. Span-based cache key is conservative — `rust_interop_plan.rs:260-264`**

Spans (`TextRange::start..end`) flow into the cache key fragment, so whitespace-only edits above a `@rust(...)` decorator invalidate the project cache even though the declaration's semantics are unchanged. This is consistent with how the rest of the generated Rust source already participates in the key, so it's not a regression. Worth flagging because Phase 39 design emphasizes "cache invalidation changes when any bridge declaration... changes" — the current implementation is stricter than that contract (invalidates on whitespace too).

**I2. Parser carve-out scope — `third_party/ruff/.../expression.rs:1129-1132`**

`rust_async_attribute_is_allowed` is gated on `Expr::Name("rust")` LHS and current token `Async`, which is tight enough that it can't leak. It does, however, accept `rust.async` outside decorator context (e.g. inside an expression), since the parser doesn't carry decorator context. That has no semantic meaning today and will be a downstream name-resolution error, which is fine — calling it out so it's not a surprise later.

## Phase-scope fit

- Reserves all ten `SIFR-RUST-*` families with consistent summaries (`registry.rs:451-501`). Only `RUST-CONFIG-0001` is active, matching the M39.1 scope of "stable malformed-decorator diagnostics". The remaining reserved family names line up with the architecture document's subsystems.
- HIR carries `rust_interop` on both `HirFunction` and `HirClass` with spans, effect classification, and ABI requirements — that satisfies the "every parsed Rust interop declaration is visible to check/build/run paths before Cargo execution" DoD.
- `InteropBuildPlan` is plumbed through both single-file (`entrypoints.rs:195`) and multi-module (`lib_project_codegen.rs:80-82`) codegen, and through both materialization paths. No emitted-Rust scanning fallback was added — declarations flow as structured metadata, which is the milestone's core contract.
- `MultiModuleCodegenResult.interop` is populated from `compile_order`, which is the deterministic ordering already trusted for the rest of the cache key. Good.

## Determinism

Audited for nondeterministic sources in the cache-key path:
- `RustInteropPlan.declarations`: `Vec`, populated in deterministic module order, then function order, then class order, then method order, then `operator_impls` order — all `Vec` iteration. No `HashMap`/`HashSet`.
- `RustInteropDeclaration.arguments`: `Vec` preserving decorator-call keyword order from the AST.
- `cache_key_fragment` only uses `push_str`, `to_string`, and literal delimiters. No floats, no clock values.

No issues.

## Diagnostics quality

`SIFR-RUST-CONFIG-0001` is the single active code. Pattern: registry template `"malformed Rust interop decorator: {reason}"` plus call sites that pre-format the full message with `format!`. This matches the existing convention in `workload_annotations.rs:85-93` — the registry template documents the message shape but isn't applied at runtime. Consistent with the codebase, no action.

Messages are span-correct: each `malformed(...)` call targets the offending sub-expression's range (decorator expression, positional arg, keyword arg, value expression, policy call name/args), so reported errors point at the exact source the user typed. Good.

## Residual risks (none blocking)

- **Future M39.2+ cache shape may need to be bumped.** When probe plans, trust evidence, Cargo digests, etc. start participating in `InteropBuildPlan`, the fragment format will gain new key=value tokens. Reviewers of those milestones should re-check that the fragment's parser-free design holds up — adding fields is safe; reordering existing ones would silently invalidate old caches, which is generally acceptable but worth flagging at the time.
- **Argument-value `Self.X` (L1) and negative integers (L2)** will likely be revisited when later decorators need them. Acceptable to defer.

## Bottom line

Review-satisfied for M39.1. Round 1 blockers are correctly resolved; the lowering, IR shape, codegen plan, parser carve-out, diagnostic family allocations, and registry test grammar all line up with the milestone scope. L3 (a handful of negative-fixture tests) and L4 (one cache-key regression test) are the only items I'd ask for before merge if you want belt-and-braces coverage; L1, L2, L5, I1, I2 are notes for future milestones, not gates on this one.
