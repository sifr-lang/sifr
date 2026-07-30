## Scope of the delta

Files touched after the pass-6 artifact (06:48:48) are exactly three: `state_collection.rs` (06:49:32), `expressions_tests/empty_plain_dict_inference.rs` (06:49:32), and the ledger (06:52:16). Every other Wave 3 file — codegen `local_binding_registry.rs`/`scope_and_function_types.rs` (03:44–05:07), `container_literal_specialization.rs` (03:38), `mod_context.rs` (05:09), `statement_dispatch.rs` (05:41), `empty_plain_dict_inference.rs` (05:42), `expression_inference.rs` (05:43), the e2e fixture (05:09) — predates pass 6 and is byte-identical to what pass 6 approved. So the correction cannot disturb the approved mechanisms, including the pass-4 widening fix.

## Verification

**Disqualification is total and sticky.** `state_collection.rs:570-574` routes any `Expr::Subscript` augassign target whose base is a `Name` to `disqualify_exact_dict_writes`, which pins `Some(shape) → None` (`state_collection.rs:137-139`). `record_dict_write`'s `and_modify` compares against `Some(&shape)`, so a `None` entry can never be lifted back (`state_collection.rs:125-135`) — later normal writes cannot restore eligibility. `exact_dict_write_hints()` drops `None` entries (`state_collection.rs:156-168`), and the adoption gate requires `binding_hints.get(name) == exact_dict_write_hints.get(name)` (`statement_dispatch.rs:128-131`); a disqualified name yields `Some(ty) != None` and leaves the candidate set. Where the name is absent from both maps the comparison trivially holds, but `inferred_binding_hint` then returns `None`, so nothing is adopted — safe.

**Nested control flow propagates.** All compound forms analyze a clone and merge back through `merge_env_types`, which now calls `merge_exact_dict_writes` (`expression_inference.rs:81-90`): if/elif/else and while/for at `state_collection.rs:596,604,620,633,643,673,683`; match/try/handlers/finally/with at `compound_statement_inference.rs:164,180,194,205,242`. `merge_exact_dict_writes` (`state_collection.rs:141-154`) `or_insert`s a branch-only disqualification into a parent that lacks the key, so an augassign discovered only inside a loop or handler still poisons the outer env. Its documented invariant holds: each branch env is cloned from the current merged parent, and the parent is not mutated between clone and merge.

Confirmed empirically with throwaway files under `/tmp` (since removed; repo state byte-identical to session start, verified by `git status`): augassign in a `for` body, in `if`-inside-`while`, and in a `try` body all keep `SIFR-TYPE-0005 unsupported operand type(s) for +: 'Any' and 'int'`, including when a concrete write follows. A read-before-write candidate (`if word in counts: ... ; counts[word] = len(word)`) still adopts and checks clean, so the fix does not over-reject.

**Direct rebinding starts fresh.** `bind_var` and `bind_call_result` both `remove` the shape entry (`state_collection.rs:107,121`). The stale-parent-shape hazard after a *branch-local* rebinding is unreachable, because `safe_hint_names_for_block` requires exactly one direct binding and no nested-block binding of the name (`empty_plain_dict_inference.rs:23-26,71-109`).

**Regression test.** `expressions_tests/empty_plain_dict_inference.rs:92-99` uses the realistic loop form (`for word in words: counts[word] += 1`) with a *following* concrete write, and asserts both `DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR` and the exact message — that code is `SIFR-TYPE-0005` (`sifr_diagnostics/src/codes/registry.rs:42`), matching the CLI output exactly. All 13 focused tests pass locally (`cargo test -p sifr_lowering empty_plain_dict`: 13 passed, 0 failed).

**defaultdict.** Candidacy requires `empty_collection_literal_kind(&assign.value) == Some("dict")`, i.e. a literal `{}` (`empty_plain_dict_inference.rs:12-20`, `statement_dispatch.rs:49-54`). `defaultdict(int)` / aliased `dd(set)` are `Expr::Call`, so they never enter the candidate set nor the new `empty_dict_specializations` insert (`control_flow.rs:438-443`); the `__sifr_defaultdict_` alias path at `container_literal_specialization.rs:247` is untouched. A `defaultdict(int)` counter loop still checks clean.

**Ledger.** The Wave 3 row records the augassign disqualification and the looped regression coverage as pass 6's response, marks status `remediation validated; re-review pending`, and Wave 2 is correctly flipped to `merged` (matches `b3f663a17`). The concrete-write-before-augassign wrong result remains separately tracked in `ad-hoc-dict-missing-key-augassign-semantics.md`, which describes exactly that shape with a 2026-10-31 fail-closed deadline. I confirmed that shape's acceptance is pre-existing and independent of Wave 3: the same source with a second `counts = {}` (which removes the name from the candidate set entirely) still checks clean, so it comes from the in-order refinement path, not the new adoption.

## Non-blocking observations (no action required)

- The ledger says "any subscript augassign"; the code only disqualifies when the subscript base is a `Name`, so chained `d[k1][k2] += 1` is not disqualified. Not exploitable: a typed dict read yields `int | None`, so that form is still rejected (`SIFR-TYPE-0005 ... 'int | None' and 'int'`).
- Augassign inside a *nested function* body is not merged back (`state_collection.rs:709-731` unifies only `vars`). Also not an expansion: blocks containing a nested function already take the pre-existing general empty-collection hint path (`statement_dispatch.rs:125`), which the ledger explicitly documents as unchanged.
- `plans/reviews/active/...wave-3-claude-opus-review-pass-8.md` is present as a zero-byte untracked file; consistent with prior practice it should not be committed empty.

Zero actionable or blocking findings.

**APPROVED**
