# Wave 5.1 Parsed-Source Shape Inventory — Review Pass 1

Branch: `codex/wave-5-1-parsed-source-shape-inventory`
Reviewer scope: blocking review of working-tree diff for Wave 5.1 slice closure.
Date: 2026-06-15

## Verdict

No blocking findings. The slice is schema-valid, the shape snapshots are Sifr-owned (variant-name projections, not raw Ruff debug output), the test refactor preserves coverage and keeps both files under the 900-line guardrail (`lib.rs` 618 lines, `syntax_matrix_tests.rs` 281 lines), and the tracker update correctly states "in progress on branch …" without claiming a merged PR or unrun gates.

Another full review round is NOT required for this slice. A second pass is only warranted if the residual-risk items below are addressed in-slice rather than deferred to Wave 5.2+.

## Findings

### Non-blocking — residual risks worth tracking before Wave 5.2

1. **Statement-shape projection drops non-`body` children, including Match arm boundaries** — `crates/sifr_syntax/src/syntax_matrix_tests.rs:87-119`. `statement_shape` only descends into the primary `body` of container statements:
   - `If.elif_else_clauses` is omitted.
   - `For.orelse` / `While.orelse` are omitted.
   - `Try.handlers`, `Try.orelse`, `Try.finalbody` are omitted (only `Try.body`).
   - `Match.cases` are flattened: `match_stmt.cases.iter().flat_map(|case| statement_tree(&case.body))` collapses all case arm bodies into a single linear list under `Match.body`, losing arm boundaries, patterns, and guards. The first fixture's snapshot (`shape_snapshots[0]`) shows two `Return` entries directly under `Match.body` — that is *not* the actual parsed shape; the parsed shape has two `MatchCase` arms each containing one `Return`. A refactor that swaps the order of the two case arms, drops one arm into a guard, or merges two arms into one would still pass this snapshot.

   This is acceptable as an *initial* shape projection if Wave 5 explicitly chooses "primary-body only" as the normalizer. Recommend documenting that decision either as an additional entry in `normalizers` (e.g. `"primary-body-only"`) on each inventory row, or in the inventory file's preamble. As written, the normalizer list (`statement-kind-only`, `source-order`, `no-byte-spans`) doesn't disclose the body-only descent.

2. **Inventory check is purely static — does not transitively invoke the Rust shape test** — `verification/areas/core_language/checks/lowering_layer_inventory.py:147-171`. The check validates row structure, fixture existence, presence of `expected_statement_tree`, and `snapshot_id`-to-fragment naming. It never runs the parser or compares the inventory's claimed shape against actual parser output. The executable evidence lives in `parsed_source_shape_snapshots_match_sifr_owned_boundary` (`syntax_matrix_tests.rs:230-247`), which only runs under `cargo test -p sifr_syntax` (not under the `core_language` verification area). A divergence between `expected_statement_tree` and parser output would be caught by the Rust gate but not by the verification area. This is adequate ("tied to executable evidence" via shared fixture data + `snapshot_id` naming), but the binding is documentary, not direct. Consider, in a future slice, having `lowering_layer_inventory.py` shell out to a small Rust helper or consume a serialized Rust-side snapshot artifact so the verification area itself fails on shape drift.

3. **`snapshot_id` format is hard-coded to `syntax_parser_lexer_matrix.<collection>.<id>`** — `lowering_layer_inventory.py:169`. Later waves (5.2+) adding HIR / name-resolution / type-ownership / CFG snapshots will reference different fixture files; the equality check above will block all such rows until the format is generalized (e.g., derive from `Path(path_text).stem`). Not a Wave 5.1 blocker — only one fixture file is referenced today — but the constraint should be relaxed before any non-`parsed_source` row is added.

4. **`status="mapped"` does not require `replacement` to be non-null** — `lowering_layer_inventory.py:78,69-71`. `replacement` is in `REQUIRED_FIELDS` (key must exist), but `null` is accepted regardless of `status`. Today no rows are `mapped`, so this has no live impact, but the schema should enforce `status == "mapped" => isinstance(replacement, str) and replacement` before that status is used.

### Wave 5 scoping — no overclaim observed

- `ALLOWED_LAYERS` (`lowering_layer_inventory.py:14-20`) lists all five future layers, but only `parsed_source` rows exist in `lowering_layer_inventory.json`. The allowed-enum is forward-compatible vocabulary; no claim is made about HIR/name/type/CFG coverage. Consistent with "parsed-source shape only, with inventory-backed validation before later slices."
- Both inventory rows have `compiler_layer: "parsed_source"`, `owner: "compiler/syntax"`, and `profile_assignment: ["create-pr", "merge"]`. No mention of HIR/name/type/CFG anywhere in the rows.
- Tracker delta (`plans/issues/active/…-gate-closure.md:4, 1135-1141`) reads "in progress on branch …" and lists only the focused gates the user actually ran. It does NOT claim `scripts/run_all_tests.sh --profile create-pr` was run or that a PR is open/merged. Accurate.

### Test refactor — coverage preserved

- The four pre-existing matrix tests (`positive_parser_matrix_cases_parse_and_expose_required_tokens`, `negative_parser_matrix_cases_emit_stable_diagnostics`, `lexer_token_matrix_preserves_kinds_and_byte_spans`, `syntax_matrix_has_no_positive_negative_source_contradictions`) are moved byte-for-byte into `syntax_matrix_tests.rs:121-281`; assertions, panic messages, and fixture wiring all match the prior versions in `lib.rs`.
- The fifth test (`parsed_source_shape_snapshots_match_sifr_owned_boundary`, `syntax_matrix_tests.rs:230-247`) is additive.
- `use super::*` is replaced with explicit imports: `crate::{parse_module, parse_module_raw}`, `sifr_diagnostics::DiagnosticArg`, `sifr_python_ast::Stmt`. The corresponding `DiagnosticArg`, `Value`, `BTreeSet`, `PathBuf` imports in `lib.rs`'s `mod tests` are correctly removed (`lib.rs:480-487` diff). `DiagnosticArg` is still imported at module scope in `lib.rs:10` for the `unsupported_details` path, which still uses it.
- `cargo test -p sifr_syntax` is reported passing by the user.

### Inventory ↔ snapshot binding — sound for parsed_source layer

- `validate_source_fixture` (`lowering_layer_inventory.py:147-171`) correctly detects misrouting: pointing a row at `positive_parse_cases/sifr_owned_function_class_match` (which has no `expected_statement_tree`) would fail with "lacks expected_statement_tree". Pointing at a non-existent fragment fails with "does not exist". Pointing at a malformed file fails with the OSError/JSONDecodeError branch. Uniqueness of `id`, `contract_id`, `snapshot_id` is enforced. Active rows are forced into `create-pr`/`merge` (`lowering_layer_inventory.py:84-85`).
- I ran `python3 verification/areas/core_language/checks/lowering_layer_inventory.py` against the current tree: exit 0, "lowering layer inventory ok". Consistent with the user-reported area-run results.

### Snapshot stability — Sifr-owned, not Ruff-leaking

- `statement_kind` (`syntax_matrix_tests.rs:57-85`) is an exhaustive match over `sifr_python_ast::Stmt` variants — no wildcard arm — so a Ruff/`sifr_python_ast` variant addition would fail to compile rather than silently widen the projection.
- The projected JSON is `{kind, body?}` only; no `TextRange`, no Ruff `Debug` output, no `is_async`, no decorators, no identifiers. Snapshots are insensitive to whitespace, span shifts, and decorator/attribute reshuffling — exactly the stability property called out for "Sifr-owned boundary."

## Residual risks (carry into Wave 5.2+ planning)

- The "primary-body-only" projection is the most impactful gap: changes to `Try` handlers, `Match` arm structure, `If/While/For` else-clauses are invisible to the shape snapshots. If Wave 5.x is going to add CFG/control-flow inventory layers, they will catch the structural piece — but until they exist, parsed-source shape coverage is partial.
- Static inventory ↔ Rust-test binding is documentary; future drift would only be caught in the Rust test gate, not in `sifr_verify`.
- `snapshot_id` format and `status="mapped"`/`replacement` invariants will need to be generalized before Wave 5.2 adds HIR rows or any mapped row.
- AGENTS.md mandates `scripts/run_all_tests.sh` (or `--profile create-pr`) as the local authoritative gate before PR. The Focused-validation block in the tracker honestly omits this — confirming the slice is "in progress," not "ready for PR." The full local gate must be run before this slice is opened as a PR.

## Sign-off

Approved as-is for the Wave 5.1 closure scope, with the residual-risk items above flagged for explicit decision in Wave 5.2 planning (specifically: document the primary-body-only normalizer on inventory rows, and decide whether `lowering_layer_inventory` should consume a Rust-side snapshot artifact). No second review round required at this layer.
