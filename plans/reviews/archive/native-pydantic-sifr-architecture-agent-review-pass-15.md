All threads complete. Writing the review.

---

# Pass 15 Closure Review — Native Pydantic-Sifr Architecture

**No files modified.**

**Method.** Read the issue in full (2,331 lines, +124 vs pass-14), pass-14, and the complete worktree diff. Mechanically extracted `CoreSchemaType` (53) and `CoreSchemaFieldType` (4) by AST from the sole oracle (`../pydantic@f59e929c`, in-tree core 2.47.0) and set-diffed them against the new table. Re-derived both selector tables by AST. Read `filter.rs:155-260`, `validators/test_json.py`, `test_schema_functions.py`, `core_schema.py:489-498`. Ran `--emit-plan` for `create-pr`, the fixture-inventory triple, and the phase-gate sweep.

## Pass-14 items: 12 of 13 closed at the root

| Item | Evidence |
| --- | --- |
| **B-1 node algebra** | **CLOSED.** Table `725-777` + `783-786` is exact-set-equal to the pin: 53/53 and 4/4, **zero missing, zero extra, zero duplicates**, all 57 classes drawn from the four legal values. All 13 pass-14 gaps now have rows: `dataclass` `766`, `dataclass-args` `765`, `dataclass-field` `784`, `json` `771`, `generator` `749`, `invalid` `725`, `any` `726`, `multi-host-url` `773`, `arguments`/`arguments-v3`/`call` `767-769`, `is-instance`/`is-subclass`/`callable` `742-744`. `multi-host-url` is also added to the Specialized-scalars family (`702`), retiring the self-contradiction |
| **`invalid` mapping** | Exact. `725`'s "static schema construction fails; no executable invalid node exists" ↔ `test_schema_functions.py:385-387` (`SchemaValidator(invalid_schema())` → `SchemaError: Cannot construct schema with 'InvalidSchema' member.`) |
| **MJ-1 experimental sentence** | **CLOSED.** `808-815` now correct in both directions: `missing-sentinel` "is a real Core Schema node but is `not-applicable`"; `allow_partial` "is not a Core Schema node; it is a validation-call mode" |
| **MJ-3 fraction gate** | **CLOSED.** `core/fraction` native contract defined `1436-1439`, gated in ps_5 `2106`, row `732` owner `ps_5` |
| **MJ-4 durable inventory** | **CLOSED.** `rust_interop_architecture.md:962-995` now enumerates **34**, exactly matching all three inventories (fixture dirs 34, matrix ids 34) — zero in-fs-not-in-doc, zero in-doc-not-in-fs. `async_runtime_core`, `opaque_resource_core`, `panic_boundary_wrapper_emission` added |
| **mn-1/2/3/5/6/7** | **CLOSED.** pytest 9.1.1 + locked plugins named `1486`, `testpaths` reason corrected to "explicit path arguments bypass" `1491-1493`; `test_filter_runtime`/`_more` added `1610-1611`; mixed anchors `1646-1653`; `index.md:3-5` no longer claims generation + PS-1 row `:52`; ps_9 names `x-sifr-integer-profile` `2185-2188`; `roadmap.md:32-35` amended |
| **Mechanical layer (8th pass)** | 58 rows, 93 path instances / 80 distinct — 93/93 exist and git-tracked; **337 anchor instances → 337 distinct `(path, selector)` pairs**; 0 unresolved, 0 ambiguous, 0 multi-def, 0 class-scoped, 0 `xfail`/`skip`, 0 multi-milestone-owned |
| **Selection precedence** | Re-verified exact after reflow: clauses 1-4 (`1161-1170`) ↔ `filter.rs:164-186` (terminal exclude → `Ok(None)`), `:189-224` (call-include emits, forwards `next_exclude`, falls through on `explicit_include`), `:228-230`, `:232-234`+`:250-257` (`default_filter`). Union / re-include / intersection (`1172-1175`) all exact |
| **Governance** | 30/30 phase files 15-43 carry Quality Contract + Exit Gate. Phase 41 title matches `index.md:51`. Phase 42 gates `ps_11` (`:9, :38`), 0 `ps_10` refs. Phase 32 on released `sifr.ipc`. `serialization_boundary_rules.md:10-15` names the package a consumer, not a second authority |
| **MJ-2 `rust_interop`** | Still outside the gate — `create-pr --emit-plan` → `"execution_mode": "legacy-facade"`, **0** `rust_interop` occurrences. But correctly owned and sequenced: cert issue `:88-90` + `:95-97` mandate the authoritative legacy path, exit gate `:100-103` blocks `ps_3`, not `ps_0`. Unchanged disposition: **does not block ps_0** |

## BLOCKER

### B-1 — The exact-set audit rule is unsatisfiable against the table it must reproduce

`717-718` states the generator "fails exact-set equality unless every kind has **exactly one** row, compatibility class, normal form, **owner, and evidence family**." Acceptance criterion `2283-2284` repeats it ("**one** accepted family/disposition owner per kind"), and the maintainability contract `1904-1905` repeats it a third time ("**one** implementation owner, one specification table and **one** focused test family").

Seven of the 53 accepted rows violate it:

| Line | Kind | Violation |
| --- | --- | --- |
| `729` | `int` | 2 families — `validators/numeric`, `core/fixed_integer` |
| `731` | `decimal` | 2 — `validators/numeric`, `core/decimal_digit_counting` |
| `733` | `str` | 2 — `validators/text_bytes`, `core/string_pipeline_order` |
| `757` | `union` | 2 — `validators/unions`, `core/smart_union_ranking` |
| `764` | `model` | 2 — `core/json_models`, `api/base_model` |
| `773` | `multi-host-url` | 2 — `core/multi_host_url_serialization`, `api/networks` |
| `770` | `custom-error` | **2 owners** — `` `ps_4` `` and `` `ps_7` ``; second evidence is prose ("`ps_7` union anchors"), the only cell in either table with no machine-comparable family name |

Consequence: `ps_4`'s deliverable at `2069-2071` — "Generate `tests/provenance/core_schema_kinds.toml` … and **prove exact equality** with the accepted disposition table" — must fail on 7 of the 53 rows it is required to reproduce. And ps_0's own exit gate (`1977-1978`) demands "no unresolved **ownership** … ambiguity" while `custom-error` carries two owning milestones.

This is the mechanization pass-14 asked for, with its acceptance rule written strictly narrower than the artifact it governs. Fix is one sentence plus one cell: admit an ordered primary owner + supporting families (which is what the table actually means, and what ps_4 `2076-2077` / ps_7 `2136-2137` genuinely split), or collapse to one each.

## MAJOR

**MJ-1 — `any` is dispositioned three inconsistent ways, and its evidence provably cannot discriminate it.** Row `726` classes `any` as **`adapted`** with normal form "Typed `identity[T]` when `T` is known, or `JsonValue` for dynamic JSON."

1. **No node exists.** The required node algebra (`699-709`) contains no identity, dynamic-value, or `JsonValue` node in any of its nine families (verified: zero hits). The doc's only other `JsonValue` mentions are `150` (the compiler's general API, explicitly *not* the package's) and `974` (`jiter::JsonValue`, the *input* document). Rule `1530-1532` requires every `adapted` behavior to map to "an explicit Core Schema node … an uncovered capability fails the audit."
2. **Evidence is non-discriminating.** Its sole family `validators/embedded_json` anchors `test_any` at `pydantic-core/tests/validators/test_json.py:36-37`, which builds `core_schema.json_schema()` — the **`json`** kind. The same file's `test_any_schema_no_schema` (`:172-178`) proves the inner `any` is *erased*: `json_schema()` and `json_schema(any_schema())` both yield `validator:None`, only `json_schema(int_schema())` yields `validator:Some(`. No `any` validator ever executes. Rule `1533-1535` bars a non-discriminating anchor from certifying a claim. Row `771` (`json`) carries the **identical** owner/evidence with a different normal form, so one family is the sole evidence for two kinds.
3. **The harness rule says the opposite.** `1630-1636`: every upstream `any_schema()`/`Any` harness is adapted per-assertion "to the smallest concrete Sifr structural type," and "**Neither adaptation introduces `Any`**, an untyped callback, or a recursive dynamic value tree" — i.e. `any` is normalized away, not implemented.

**MJ-2 — ps_5's exit gate requires two families its checklist never schedules.** Gate `2101-2109` requires `validators/generator` and `validators/embedded_json` to pass. The ps_5 checklist (`2088-2099`) schedules scalars, integers/floats/decimals/fractions/complex/strings/bytes, temporal + pattern, constraints, "lists, tuples, mappings, sets and frozen sets," and the three input profiles — and nothing else. Neither newly-assigned capability appears: `ValidatedIterator` occurs only at `705, 749, 1187, 1856` and "embedded-JSON" only at `707, 726, 771, 1587, 2103` — **never in any milestone checklist**. `ValidatedIterator[T]` with `next() -> Result[Option[T], ValidationError]` (`749`) is a new public generic type, not a variation on a list; the embedded-JSON child decoder is its own Control node (`707`). This is pass-14 MJ-3 inverted — gate without work item instead of work item without gate — and it lands in ps_0's "no … sequencing ambiguity" clause (`1977-1978`).

## MINOR (edit-worthy)

- **mn-1** — Two of the four algorithms the doc claims as parity cite their pinned implementation source; two do not. The string pipeline cites `pydantic-core/src/validators/string.rs:110-178` (`829`) and decimal emission cites `decimal.rs:152-197` (`853`); the seven-clause smart-union algorithm (`1065-1098`, "follows the pinned Pydantic Core algorithm") and the four-clause selection precedence (`1156-1197`) cite **nothing** — `union.rs` and `filter.rs` have 0 occurrences doc-wide. Both have discriminating native families named, so the second half of `1533-1535` is met; the "records its pinned implementation source" half is met only where the precedent was set. These are the two most intricate algorithms in the design and, unlike the node/selector ledgers, prose does not regenerate on a re-pin — which is exactly the drift channel passes 13-14 diagnosed.
- **mn-2** — pass-14 mn-4's second half survives: `1183-1185` is a legitimate Sifr-owned rule, but the section still omits that the pin normalizes only `dict`/`set` include/exclude spellings before falling back to `__contains__` (`filter.rs:167-186, 191-224`) and that the unsized-iterable path rejects negative indices — the upstream facts `1186-1189`'s `ValidatedIterator` rejection is derived from.

*Not attributable to this change set (third pass):* `verification/areas/diagnostics/checks/code_coverage.py:174` still checks `docs/errors/<CODE>.md` while the registry emits `.mdx`. `ps_1` adds `SIFR-META-*` pages onto that surface.

*Housekeeping:* `plans/reviews/active/native-pydantic-sifr-architecture-agent-review-pass-15.md` exists but is 0 bytes, and the artifact list (`15-28`) has no pass-15 row; Status `9` still reads "Passes 4 through 14."

## Can `milestone_ps_0` be re-approved?

**No — two findings away, and both are narrow.**

B-1 blocks directly: ps_0's own deliverable `1970-1971` is "Approve the pin-derived 53-kind … table **plus its exact-set generated-manifest rule**," and the two are mutually inconsistent in 7 of 53 rows, with one row carrying two owning milestones against an exit gate that forbids ownership ambiguity. MJ-1 blocks on `1530-1532` + `1977-1982`: `any` is `adapted` without a node in the algebra, so an `adapted` behavior does not map to an explicit Core Schema node, which the doc says "fails the audit." MJ-2 bears on the same gate's sequencing clause. mn-1/mn-2 are editorial. MJ-2 (`rust_interop`) from pass 14 remains correctly deferred to `ps_3`.

**Where the design stands.** This revision closed the blocker class permanently. The node algebra is now generated from the pin's own `CoreSchemaType` literal with an exact-set audit, and it verified clean on the first mechanical attempt: 53/53, 4/4, no duplicates, no invalid classes, and every one of the 57 evidence families both exists and is gated in the milestone the row names — zero exceptions. That was the single structural weakness pass 14 identified, and it is fixed at the mechanism rather than the symptom. The `invalid` row's mapping to `test_err_on_invalid` is exact against the oracle's `SchemaError` text. The durable Rust-interop inventory now agrees at 34/34 with all three sources. The selector layer is clean for an eighth consecutive pass, and the reflowed selection-precedence block is still line-for-line exact against `filter.rs`. Governance is complete: 30/30 phase gates, index no longer claims a generator it doesn't have, roadmap rule amended to match reality, integer-schema ownership recorded in the locked boundary artifact.

What remains is one register lower than any prior pass: the three surviving findings are a singular/plural mismatch between a rule and its table, one row whose normal form was written before the node algebra caught up, and one checklist that gained two gates but not the two matching work items. None requires re-deriving anything. **Ask the next revision to make `any` say what `1630-1636` already says** — normalized away to a concrete type per assertion, which reads as `not-applicable` or as an `adapted` row pointing at a real node the algebra lists — and the disposition table will be internally closed.

**VERDICT: NEEDS REVISION**
