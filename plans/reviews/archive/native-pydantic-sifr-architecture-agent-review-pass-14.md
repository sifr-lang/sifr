All threads complete. Writing the review.

---

# Pass 14 Closure Review — Native Pydantic-Sifr Architecture

**No files modified.**

**Method.** Read the issue in full (2,207 lines, +112 vs pass-13), pass-13, and the worktree diff. Re-derived both mandatory tables with an AST script against the pin. Read `string.rs`, `filter.rs`, `union.rs`, `validation_state.rs`, `decimal.rs`, `fields.rs` at the sole oracle (`../pydantic@f59e929c`, in-tree core **2.47.0** — confirmed `pydantic-core/Cargo.toml:3`; standalone `../pydantic-core` is 2.41.5, research-only). Extracted `CoreSchemaType` mechanically (53 kinds). Empirically reproduced pytest collection on a `/tmp` mirror of the pin's exact layout under the pin's own locked pytest 9.1.1 **and** 8.4.2. Ran every `rust_interop` validator, the area runner, `--emit-plan` for both gate profiles, and `code_coverage.py`.

**The mechanical layer is clean for a seventh consecutive pass**, and this revision closed **20 of pass-13's 25 items at the root** — including all four blockers' stated content. What remains is concentrated in one place: **the node algebra was never re-derived against the pin's full 53-kind `CoreSchemaType`**, which is the same failure mode pass-13 diagnosed (`prose not re-derived from the re-designated pin`) surviving in the one table pass-13 only sampled.

---

## Verified clean (concise)

| Item | Evidence |
| --- | --- |
| **Mechanical layer (7th pass)** | 55 rows, 90 path instances / 77 distinct — 90/90 exist and git-tracked; **322 anchor instances → 322 distinct `(path, selector)` pairs**; 0 unresolved, 0 ambiguous, 0 class-scoped, 0 `xfail`/`skip`-decorated, 0 multi-milestone-owned |
| **B-1 string pipeline** | **CLOSED.** `747-750` now reads strip → **ASCII** → length → pattern → case, matching `string.rs:122-124 → :125-127 → :129-155 → :157-167 → :169-178`. `751-755` certifies all four boundaries pass-13 flagged |
| **B-2 selection precedence** | **CLOSED.** `1093-1095` union / re-include / intersection all exact vs `filter.rs:210` (fall-through), `:208-209` (pre-`default_filter` return), `:251` (`include.contains && !exclude.contains`). Clauses 1-4 (`1082-1091`) exact vs `filter.rs:161-234`. Intersection now has a listed anchor — `test_filter` sets **both** schema include and exclude in `test_list_tuple.py:163-169` and `test_dict.py:58-65` |
| **B-4 two-process ledger** | **CLOSED.** Empirically: `pytest --collect-only tests --ignore=tests/pydantic_core` → 15 nodes; `pytest --collect-only pydantic-core/tests` → 23; **15+23 = 38 = the single-root universe, partitioned with no loss or duplication.** The naive two-root form aborts (`ImportPathMismatchError`, exit 4, 0 collected). Bonus: the `api::`/`core::` prefix (`1400-1402`) is proven **load-bearing** — 4 raw node-ID collisions exist between the rootdir-relative ledgers, and 2 are themselves mandatory anchors (`test_json_bytes_base64_round_trip` `1489`, `test_hide_input_in_errors` `1616`) |
| **MJ-1/2 smart union** | **CLOSED.** Clauses 1-7 (`989-1001`) exact vs `union.rs:122-131, 150-151, 160-164, 179-189`; clause 7's deferred `Omit` is 2.47.0-only (absent in 2.41.5). `1003-1005` non-transitive left fold correct — witness `(None,Lax)/(Some(1),Exact)/(Some(2),Lax)` flips outcome on reordering. `1007-1008` label fallback ↔ `union.rs:281, :86` |
| **MJ-4 `JsonLimitError`** | **CLOSED.** `1226-1228` `{ message: str, limit: int }` matches `crates/sifr_runtime/src/json.rs:104-107` exactly; `ResourceLimitError { kind, limit, location }` correctly separated as package-owned (`1229-1233`) |
| **MJ-5/6/7/8 governance** | **CLOSED.** `41:40` Quality Contract + `41:57` Exit Gate; **30/30** phase files 15-43 now have both. Phase 32 retargeted to the *released* `sifr.ipc` (`stdlib/sifr/ipc.sifr:8,22,84`; roadmap `:73` completed/audited) at 4 sites. Phase 42 gates `milestone_ps_11` (`42:9, :38-39`), **0** `ps_10` references, fallback forbidden `42:45`, Phase 27 invariants intact `42:40-42` |
| **MJ-9 closed args** | **CLOSED.** `527-534` reconciles with `registry.rs:619-634`'s closed `declared_args`: package template args go into a bounded structured note, never forwarded as registry args |
| **mn-1/2/3/9/10/12** | **CLOSED.** Euclidean modulo incl. positive-out-of-range (`1102-1105` ↔ `filter.rs:20-25`); empty-nested-inclusion carve-out (`1110-1113` ↔ `filter.rs:202-213, 230`); default-engine precompiled pattern (`756-758`); `hard` LSP + `rule` reserved (`516-518, 531-534` ↔ `conversion.rs:454-467`, `lsp_server.md:152-153`); atomic two-signature `re.compile` migration (`1914-1921` ↔ `stdlib/sifr/re.sifr:225,228`); decimal first-match-wins + allowance-in-context (`771-774` ↔ `decimal.rs:157-198`, `:192` `whole_digits: max_whole_digits`) |
| **Decimal arithmetic** | Hand-executed `max_digits=3, decimal_places=2`: `0.000`→accept, `0`→accept, `1.500`→accept, `0.5`→accept, `100`→reject `decimal_whole_digits` ctx=**1**. 5/5 reproduced by the doc's wording |
| **`certification_pkg_resource_core`** | Sequencing coherent: cert issue `:86-103` owns fixture creation, profile wiring, and inventory update, gated before `ps_3`; ad hoc `:1828` and `:1901-1906` agree. The premature fixture was correctly **reverted** (0 hits under `verification/`) |

---

## BLOCKER

### B-1 — Thirteen of the pin's 53 Core Schema node kinds are neither covered nor dispositioned; one is self-contradictory

`pydantic-core/python/pydantic_core/core_schema.py:4247-4301` defines exactly **53** kinds (mechanically extracted; kept in sync by `test_misc.py:178`). Pass-13's MJ-3 named seven; **five are now genuinely closed** — `fraction`/`complex` (`701`, `725-728`), `lax-or-strict`/`json-or-python`/`chain` (`706`, `717-723`). But the node algebra (`698-708`) was extended by hand rather than re-derived, and 13 kinds remain unhandled.

**Tier A — zero occurrences anywhere in 2,207 lines, and not Python-only** (verified by direct `grep -ci`):

| Kind | Doc hits | Upstream oracle at the pin |
| --- | --- | --- |
| `dataclass`, `dataclass-args` | **0**, **0** | `pydantic-core/tests/validators/test_dataclasses.py`, `serializers/test_dataclasses.py`, `tests/test_dataclasses.py`, `tests/test_validators_dataclass.py`. Also drags in `dataclass-field` from `CoreSchemaFieldType` (`:4303`) |
| `json` (the `Json[T]` node, `core_schema.py:3918-3926`) | **0** (`Json[` → 0) | `pydantic-core/tests/validators/test_json.py`. `core/json_foundation`/`core/json_values` are JSON *input-mode* families, not this node |
| `generator` | **0** | `pydantic-core/tests/validators/test_generator.py`. `704`'s "typed sequence policies supported by Sifr" is not a disposition |
| `invalid` (`core_schema.py:489-498`) | **0** | `800`'s "unknown schema versions or node kinds" is a different concept |
| `any` | **0 node-level** | `890` disclaims `Any` only inside the strings-profile input type; `1537-1543` adapts `any_schema()` *test harnesses*. Neither assigns the kind a compatibility class |

A Sifr record with declared fields is precisely what `dataclass` models, and `Json[T]` — a JSON-string-valued field nested inside a model — has no Python dependency whatsoever. These are capability gaps, not Python-only mechanics.

**Tier B — self-contradiction.** `multi-host-url` has **0 hits** as a node (`grep -ci 'multi-host'` → 0); the Specialized-scalars row `701` lists only "URL". Yet the doc mandates **two** anchors for it: `core/multi_host_url_serialization` (`1526`, gated at `2091`) and `api/networks::test_multihost_postgres_dsns` (`1624`). This directly violates the doc's own rule at **`1440-1442`**: "every `same` or `adapted` behavior maps to an explicit Core Schema node… an uncovered capability fails the audit."

**Tier C — dispositioned only by implication, kind never named:** `arguments`, `arguments-v3`, `call` (nearest hook: `1637-1640` "Python call signatures"; `validate_call` → 0 hits), `is-instance`, `is-subclass`, `callable` (nearest hook: `1368` "Python subclass and duck-typing behavior"). All seven of the doc's `arguments` hits and all three `callable` hits are unrelated (type arguments, diagnostic template arguments, test names). Defensible, but not the "explicit file/node-level classification" `1637-1640` promises.

This fails ps_0's exit gate on its own terms (`1863-1868`).

---

## MAJOR

**MJ-1 — The experimental-disposition sentence is factually wrong about one of its two subjects.** `733-736`: "Two pinned experimental/Python-state constructs deliberately do not survive **as Core Schema nodes**." Verified: `allow_partial` has **0 occurrences in `core_schema.py`** — it was never a Core Schema node. It is a validation-*call* keyword (`_pydantic_core.pyi:100, 161, 203`; Rust `PartialMode` in `validation_state.rs:29-30, 52, 60`; public `TypeAdapter.validate_*(experimental_allow_partial=...)`). Conversely `missing-sentinel` **is** a first-class node (`core_schema.py:1437-1450`, plus `src/validators/missing_sentinel.rs`, `src/serializers/type_serializers/missing_sentinel.rs`, `src/common/missing_sentinel.rs`); only its *public API* is experimental. The dispositions (`rejected` / `not-applicable`) are both correct and well-argued; the sentence that frames them mis-describes the pin in both directions.

**MJ-2 — `rust_interop` is still outside the authoritative gate (third consecutive pass), though now owned.** Proven by execution, not inference: `scripts/run_all_tests.sh --profile create-pr --emit-plan` → **`"execution_mode": "legacy-facade"`** (plan line 429) and **0 occurrences of `rust_interop`**; same for `--profile merge`. `profile_runner.py:198-200` defaults to `legacy-facade`; no profile sets `execution_mode`, so `selected_areas` (`create-pr.json:92`, `merge.json:71`, `nightly.json:73`, `release.json:72`) is never read as area selection and the hardcoded 20-step list at `:160-187` runs instead — `rust_interop` is in none of its 20 reachable areas. Only the matrix JSON is linted (`scripts/check_sysroot_stdlib_resource_certification_gate.py:17,19`). No meta-check catches it: `coverage_matrix/profile_assignment_matrix.py:145-155` only validates *declared* rows.
**Genuine progress:** the false README prose is gone (0 hits repo-wide), the premature fixture was reverted, and `rust-interop-runtime-ecosystem-certification.md:95-97` now mandates adding the area "to the authoritative legacy profile-runner path… rather than adding ignored `selected_areas` data," with a testable exit gate at `:100-103`. But no wiring exists today, and a `rust_interop` regression is still invisible to the merge gate.

**MJ-3 — `ps_5` implements fractions with no gate, and no engine oracle exists.** `1973` makes ps_5 responsible for "exact rational fractions," but its exit gate (`1984-1990`) lists `validators/complex` and **no fraction family**. Fraction's only anchors are `api/specialized_numeric` at **`ps_9`** (`1622`). Contributing cause: `pydantic-core/tests/validators/test_fraction.py` **does not exist** at the pin — there are no engine-level fraction validator tests, only `fraction_type` message text (`pydantic-core/tests/test_errors.py:397`) and a schema-construction row (`test_schema_functions.py:313`). So a Sifr-native contract is required and none is named, leaving four milestones (`ps_5`→`ps_9`) in which fraction ships ungated. This is the same class as `core/fixed_integer`/`core/pattern_value`, which the doc handles correctly.

**MJ-4 — `internal_docs/rust_interop_architecture.md` lists 31 of 34 fixtures, and the fix is parked behind `ps_2`.** The three real inventories agree exactly at **34** (`fixtures/` dirs, `data/rust_interop_fixture_matrix.json` ids, `data/rust_interop_tiers.toml`); the doc enumerates **31** at `:962-992`. Missing: `async_runtime_core`, **`opaque_resource_core`** (not named by pass-13), `panic_boundary_wrapper_emission`. The file is untouched on this branch, and `rust-interop-runtime-ecosystem-certification.md:98` assigns the update to `certification_pkg_resource_core`, which per `:88-89` cannot start until `ps_2` releases — so the durable inventory stays stale for the entire `ps_2` window, while `ps_3`'s prerequisite (`1828`) depends on two of the three missing rows.

---

## MINOR (edit-worthy)

- **mn-1** — `1403` "inert or changing upstream pytest `testpaths` configuration" is **factually false at the pin's own locked pytest 9.1.1**: `[tool.pytest]` (`pyproject.toml:182-183`) *is* honored there (header reports `testpaths: tests`), and is inert only at 8.4.2. The conclusion holds, but for a different reason — explicit path arguments bypass `testpaths` entirely. Related: `1397-1398` says "one pinned pytest/toolchain lock" but names **no version**; the pin's `uv.lock` has pytest 9.1.1 + 8 plugins, and the Core process loads `pydantic-core/pyproject.toml:115-130` whose `addopts` require `pytest-benchmark` and `timeout=30` requires `pytest-timeout`, or collection errors. Node identity is collector-derived; a concrete version belongs in the rule.
- **mn-2** — `test_filter_runtime` is **not listed** (0 hits) in either `1517` or `1518`, yet it is the sole upstream discriminator for "call-time inclusion re-includes an item removed only by a schema exclusion" (`test_list_tuple.py:172-178`, `test_dict.py:131-138`) — a clause the doc asserts at `1093-1094`. `1443-1445` requires a discriminating assertion per claimed precedence rule.
- **mn-3** — Two non-discriminating anchors survive: `test_positional_tuple` (`1517`) contains **no `include`/`exclude` at all** (`test_list_tuple.py:337-355`) and four of its assertions target the warning-and-passthrough mechanism `1125-1126` declares `not-applicable`; `test_error_type` (`1490`) runs **no validator** (`pydantic-core/tests/test_errors.py:426-431` constructs `PydanticKnownError` directly), so only `e.message()` is retainable and `.type`/`.context` are constructor round-trips excluded by `1450-1452`. Both need explicit `1453-1455` mixed-anchor treatment.
- **mn-4** — `1104-1105` "no index matches an empty sequence" is true of the pin but is a **call-site consequence, not a guard**: `filter.rs:25` swallows `ZeroDivisionError` via `unwrap_or_else`, and safety comes from every `index_filter` call site deriving `len` from the collection it enumerates (`list.rs:64,101`; `tuple.rs:174,183,232`). Also unstated: the pin normalizes only `dict`/`set` spellings (`filter.rs:44-58`), and the unsized-iterable path *rejects* negative indices (`filter.rs:26-35`).
- **mn-5** — `plans/phases/index.md:3` still claims "This index is generated from the flat phase files" with **no generator** (`grep -rn 'plans/phases' scripts/` → 0; the former `phase_contract_gate_check.py` was deleted in PR #2543 with no replacement guardrail). pass-13 mn-6's first half is fixed (`:50` title now matches `41:1`); this half is not. Two adjacent residues: `:4`'s "mirrored here" contradicts `41:3-6`'s "Superseded" status, and the index has no row linking the ad hoc issue that `roadmap.md:83` treats as canonical, though every sibling ad hoc track has one (`:40, :53-55`).
- **mn-6** — `internal_docs/integer_model.md` still lacks `x-sifr-integer-profile` (present only in `serialization_boundary_rules.md:43` and doc `1167`). `142-147` assigns ps_1 the `Reserved`→`Active` flip and ps_9 the boundary-artifact update; neither explicitly names this addition.
- **mn-7** — `roadmap.md:32` requires entry/exit gates and milestone quality checks *under* `## Quality Contract`; Phase 41 places entry criteria under a standalone `## Entry Criteria` (`41:18`) and delegates milestone checks by reference (`41:44-45, 54-55`). Repo-wide drift, not a 41-specific defect (`37_package_management.md` does the same), so either amend `roadmap.md:32` or normalize — but the rule as written is not met.

*Not attributable to this change set:* `verification/areas/diagnostics/checks/code_coverage.py:174` still checks `docs/errors/<CODE>.md` while the registry emits `.mdx` (`registry.rs:626`) and the tree holds 205 `.mdx` / 1 `.md` — **exit 1 with 204 errors today**. ps_1 must add `SIFR-META-*` pages onto that surface.

---

## Can `milestone_ps_0` be re-approved?

**No — but it is one finding away.**

Its exit gate (`1863-1868`) requires that "every required feature family with a meaningful Pydantic oracle has pinned selector anchors" and — via `1440-1442` — that "an uncovered capability fails the audit." Two things block it, both in the same table:

1. **B-1** — 13 of 53 upstream node kinds are unhandled, six with literally zero mentions and a live upstream oracle (`dataclass`, `dataclass-args`, `json`, `generator`, `invalid`, `any`), and `multi-host-url` carries two mandatory anchors with no node at all. Each needs either a node-algebra entry or a written disposition.
2. **MJ-1 / MJ-3** — the experimental-disposition sentence mis-describes the pin for both its subjects, and `ps_5` ships fractions with no gating family.

MJ-2 and MJ-4 do **not** block ps_0: both are now explicitly owned by `certification_pkg_resource_core` with a testable exit gate that blocks `ps_3` rather than `ps_0`, which is the correct sequencing. The minors are all editorial.

**Where the design stands.** This is the strongest revision in the series. The two-process ledger is now empirically collision-free and its `api::`/`core::` prefix is provably load-bearing rather than decorative — that closes the "mechanically detectable" clause that pass-13 rejected. The string pipeline, all four selection-precedence clauses, all seven smart-union clauses including 2.47.0's deferred `Omit`, the non-transitive-fold modeling note, the label fallback, and the entire decimal digit rule are now **exact against the sole oracle**, verified line by line and by hand-execution. `JsonLimitError` matches the implementation byte for byte. Every governance defect closed: 30/30 phase files gated, Phase 32 retargeted to shipped code, Phase 42 correctly gated on `ps_11`, typed model JSON unambiguously external. The premature `opaque_resource_package_core` fixture was correctly reverted rather than papered over, and the false README claim deleted.

The failure mode has narrowed to a single mechanism: **the node algebra is the one mandatory table that is still hand-maintained rather than derived.** Both selector tables are machine-checkable and have been clean for seven passes; the node table has now produced a blocker in two consecutive passes (`fraction` in pass-13, six more here) for exactly that reason. One change would retire this class permanently: **require the node algebra to be generated from the pin's `CoreSchemaType` literal, with every kind carrying either a family assignment or an explicit compatibility class** — the same total-set discipline `1393-1417` already imposes on files, nodes, and parameters. `test_misc.py:178` proves upstream keeps that literal authoritative, so the derivation is cheap and a re-pin would mechanically surface any new kind.

**VERDICT: NEEDS REVISION**
