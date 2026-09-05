# Wave 5.3 Name-Resolution Snapshots — Review Pass 1

Reviewer: agent (review pass 1)
Branch: `codex/wave-5-3-name-resolution-snapshots`
Scope under review:
- `crates/sifr_lowering/src/lib.rs`
- `crates/sifr_lowering/src/name_resolution_snapshot_tests.rs`
- `verification/areas/core_language/data/name_resolution_snapshot_matrix.json`
- `verification/areas/core_language/data/lowering_layer_inventory.json`
- `verification/areas/core_language/checks/lowering_layer_inventory.py`
- `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md` (Wave 5.3 notes)

## Verdict

**Acceptable for PR.** No blocking findings. The slice is genuinely executable evidence, the matrix matches the lowering output bit-for-bit on the three fixtures, the inventory checker enforces matrix-to-inventory claim coverage for the new `name_resolution_facts` kind, and tracker claims align with the reported validation runs. Optional follow-ups below are tightening, not gating.

## Executable evidence assessment

The slice is not aspirational inventory metadata:
- `name_resolution_snapshot_matrix_matches_lowered_name_facts` (`crates/sifr_lowering/src/name_resolution_snapshot_tests.rs:7-39`) reads the matrix JSON at test time, parses each `source`, lowers via `lower_module`, projects a normalized fact tree, and asserts byte-equality against `expected_name_resolution_snapshot`. A regression in name resolution, type assignment to a name reference, loop-target lowering, nested-function lowering, builtin lowering, or `Let`-binding lowering on any of the three fixtures fails the test.
- The matrix path is anchored at `CARGO_MANIFEST_DIR/../..` (`crates/sifr_lowering/src/name_resolution_snapshot_tests.rs:49-53`) so the test reads the same JSON the verification harness validates — no duplicated copy can drift.
- Inventory rows for the three fixtures (`verification/areas/core_language/data/lowering_layer_inventory.json:88-150`) are validated by `lowering_layer_inventory.py:54-58, 224-254` against the matrix file, both forward (row → fixture exists with `expected_name_resolution_snapshot`) and reverse (every matrix entry has a claiming row).
- The reported validation runs (cargo test, jq, py_compile, `sifr_verify areas run --area core_language`, file-size guardrail, fmt, `git diff --check`, `sifr_analysis`/`sifr_codegen`) are consistent with the artifacts in the working tree.

## Findings (ordered by severity)

### 1. Non-blocking, medium — Silent traversal gaps for several `HirStmt`/`HirExpr` variants

The projection at `name_resolution_snapshot_tests.rs:119-475` mirrors the HIR enums but drops binding/reference signal in several places. None of the three fixtures exercise these shapes today, so the matrix tells the truth for those fixtures; however, the moment a near-term fixture is added that touches these shapes, the evidence will silently miss facts that the matrix purports to assert.

Gaps observed:
- `HirStmt::Assign` and `HirStmt::AugAssign` (`name_resolution_snapshot_tests.rs:140-142`): the LHS `name` is never recorded. `AugAssign` is semantically a read-then-write of the target; the implicit read does not appear in `name_references`. In fixture 3, `total += value` records only the `value` reference and silently omits the implicit `total` read. `Assign` re-bindings produce no `local_bindings` entry and no `name_references` entry for the target.
- `HirStmt::FieldAssign`, `NestedFieldAssign`, `SubscriptAssign`, `NestedSubscriptAssign`, `AttributeNestedSubscriptAssign`, `SubscriptAugAssign`, `AttributeAugAssign`, `AttributeSubscriptAssign` (`name_resolution_snapshot_tests.rs:240-269`): each carries an `object: String` that names the receiver (typically `self`). That implicit name reference is never recorded, so any method-body fixture would silently miss `self` reads.
- `HirStmt::With { items }` (`name_resolution_snapshot_tests.rs:274-279`): destructures `(_, context, _)`, dropping the `as` binding name. A `with X() as y:` would have no `local_bindings` entry for `y`.
- `HirStmt::TupleUnpack` and `StarUnpack` (`name_resolution_snapshot_tests.rs:240-248`): only the RHS `value` is walked; the bound target names in `targets`/`before`/`star`/`after` are not recorded as local bindings.
- `HirStmt::AsyncWith { kind, target, .. }` (`name_resolution_snapshot_tests.rs:280`): `kind` is dropped, so `HirAsyncWithKind::TaskGroup { context }`, `TaskTimeout { duration }`, and `UserDefined { ... }` nested expressions are not walked; `target` binding is dropped.
- `HirStmt::Match` arms (`name_resolution_snapshot_tests.rs:212-220`): the `pattern` is never walked. `HirPattern::Capture { name, ty }` introduces a binding; `HirPattern::Literal { value: HirExpr }` carries an expression; `HirPattern::Value { path }` resolves a name. None of these appear.
- `HirExpr::Lambda` (`name_resolution_snapshot_tests.rs:438`): only `body` is walked; lambda parameters are dropped.
- `HirExpr::ListComp`/`SetComp`/`DictComp` (`name_resolution_snapshot_tests.rs:439-457`, `collect_generators` at 477-488): generators are typed `(String, HirExpr, Option<HirExpr>)`; the loop-target name (first field) is destructured `_` and never recorded as a `loop_target`. The target type is also dropped.
- `HirExpr::ConstructorCall` (`name_resolution_snapshot_tests.rs:433-437`): the class identity is not surfaced; only `args` are walked. A fixture like `Point(x, y)` would not claim the constructor was resolved to `Point`.
- `HirExpr::EnumVariant` (`name_resolution_snapshot_tests.rs:473`): swallowed silently — no record of e.g. `Color.RED`.
- `HirModule::classes` and `HirModule::imports`: not projected at all. Class-scoped methods, fields, and imported names are invisible.

Why this matters even though no current fixture hits these: the matrix kind is called `name_resolution_facts`, which reads as a complete projection. A reader extending the matrix may add a fixture that exercises (say) a class method, see the test pass, and incorrectly conclude that name resolution facts were checked for `self`-field reads. Recommend either:
1. Document explicit scope ("module-level constants, top-level functions, nested functions, simple `Let`/loop/aug-assign/return/expression statements; pattern bindings, with/unpack/async-with bindings, lambda/comprehension targets, classes, imports, enum variants, and field/subscript-assign object references are NOT projected") in a header comment, or
2. Mark unhandled binder-producing variants with `unreachable!` (or a structured "unsupported in matrix" panic) so future fixtures can't silently bypass the projection.

### 2. Non-blocking, low — Path label inconsistency for `Assign`

`HirStmt::Assign` and `HirStmt::AugAssign` share the same arm at `name_resolution_snapshot_tests.rs:140-142` and produce paths of the form `/augassign:{name}`. For a plain `Assign`, the `augassign:` label is misleading — paths are part of the matrix contract, and a future reader will read `augassign:total` and assume the source had `total += ...`. Recommend splitting the arms and using `/assign:{name}` for `Assign`. (No current fixture exposes `Assign` re-binding, so the label leak isn't user-visible yet.)

### 3. Non-blocking, low — Two records share one path key

In fixture 3, both the local binding `count_items/body[0]/let:total` and the `MethodCall` carry the same `path` value (`name_resolution_snapshot_matrix.json:108-156`). They live in different arrays, so disambiguation is fine and the test passes deterministically. Optional tightening: give the call a distinct suffix (e.g. `let:total/value`) so paths are primary-key-unique across the whole projection. Today's choice is internally consistent.

### 4. Non-blocking, low — Inventory checker accepts arbitrary normalizer strings

`lowering_layer_inventory.py:116, 176-189` enforces "non-empty list of non-empty strings" for `normalizers` but never compares against an allowed set per `snapshot_kind`. The four expected normalizers for `name_resolution_facts` (`name-resolution-facts`, `type-display-name`, `source-order`, `no-byte-spans`) are not validated, so a typo (e.g. `no-byte-span`) would not fail. Minor follow-up: add a `ALLOWED_NORMALIZERS_BY_SNAPSHOT_KIND` table.

## Items explicitly checked clean

- **Binding identity overclaim (concern 2):** The matrix does not claim definition IDs — it claims (name, ty) tuples at deterministic paths. Shadowing in `parameter_shadows_module_constant` is detectable via the `ty: int` on the `return VALUE` reference (versus the constant's `ty: str`); the equivalent of the binding-identity signal is carried by the type column. No overclaim observed.
- **`len(items)` → `MethodCall` (concern 3):** The Sifr lowering pipeline lowers `len(x)` to a HIR `MethodCall` with the receiver `x`, consistent with Sifr targeting Rust. `name_resolution_snapshot_matrix.json:144-156` records `kind: MethodCall, method: "len", receiver: { kind: Name, ty: "list[int]", name: "items" }, ty: "int", args: []`. The tracker note about realigning the matrix after the initial mismatch is accurate.
- **Stability (concern 4):** Paths are source-order indexed (`body[i]`, `arg[i]`, `comparator[i]`, etc.), types use `Type::display_name()`, no byte spans appear anywhere in the matrix, and function/nested-function scopes are explicit in the path (`outer/body[0]/nested:inner/body[0]/return`). The matrix should be stable across whitespace and span-only churn.
- **Inventory checker coverage for new kind (concern 6):** `EXPECTED_FIELD_BY_SNAPSHOT_KIND` (`lowering_layer_inventory.py:54-58`) is extended with `name_resolution_facts → expected_name_resolution_snapshot`. `validate_source_fixture` (`lowering_layer_inventory.py:192-221`) verifies the fixture exists and carries the expected field. `validate_matrix_fixtures_are_claimed` (`lowering_layer_inventory.py:224-254`) ranges over the name-resolution matrix and fails if any matrix row lacks an inventory claim — the symmetric coverage that Wave 5.2 review pass 1 asked for is now present for name-resolution too.
- **Tracker accuracy (concern 7):** The Wave 5.3 paragraph (`plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md:1151-1155`) matches the artifacts: matrix file, dedicated `sifr_lowering` test, three named fixtures, inventory-row addition, checker extension, and the `len(items)` realignment note. Status correctly reads "in progress on branch" (not "merged"). Validation commands and counts match the report set.
- **File size and maintainability (concern 8):** `name_resolution_snapshot_tests.rs` is 549 lines, well under the 900-line cap. `lowering_layer_inventory.py` is 258 lines, the matrix JSON is 164 lines, the inventory JSON is 152 lines. Helper duplication with `hir_snapshot_tests.rs` (which is 817 lines, near the cap) is the same deferred risk acknowledged at the end of Wave 5.2 review pass 2; not a Wave 5.3 blocker but worth pre-empting before Wave 5.4 lands a fourth snapshot kind.

## Required follow-up before PR

None.

## Optional follow-up

1. Document the explicit traversal scope at the top of `name_resolution_snapshot_tests.rs`, OR mark unhandled binder-producing variants (`Match` patterns, `With` `as`-targets, `TupleUnpack`/`StarUnpack` targets, `Lambda`/comprehension parameters, `AsyncWith.kind`/`target`, `*FieldAssign`/`*SubscriptAssign` `object`) with `unreachable!`/structured panics. Goal: a future fixture cannot silently land green with an incomplete projection.
2. Split `HirStmt::Assign` from `HirStmt::AugAssign` so the path label reads `/assign:{name}` for plain assignments, and at the same time record the implicit LHS read for `AugAssign`.
3. Add `ALLOWED_NORMALIZERS_BY_SNAPSHOT_KIND` to `lowering_layer_inventory.py` so typos in normalizer strings fail fast.
4. Extract `project_function`/`type_name`/`expr_kind` into a shared `snapshot_projection` module before Wave 5.4 adds a fourth projection, to avoid pushing `hir_snapshot_tests.rs` past the 900-line cap. (Deferred from Wave 5.2 review pass 2.)
5. Consider giving the `MethodCall` record a distinct path suffix (e.g., `let:total/value`) so paths uniquely key records across the full projection.

## Whether another agent review round is required after fixes

**No.** The slice is genuinely executable, the matrix and projection round-trip correctly, the inventory checker enforces claim coverage symmetrically, and tracker claims are accurate. The non-blocking findings can be folded into the same PR (or tracked explicitly in the Wave 5.3 notes as deferred risks before Wave 5.4 starts adding broader name/type rows) without another full review round.
