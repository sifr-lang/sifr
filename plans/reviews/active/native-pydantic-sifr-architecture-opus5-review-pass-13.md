All verification complete. No files were modified.

---

# VERDICT: **NEEDS REVISION**

**Method.** Read the issue (2,095 lines), pass-12, the full worktree diff, and every listed document. Re-derived both mandatory tables with an AST script: **52 rows, 83 path instances (71 distinct), 298 anchor instances, 52 table families + 6 Sifr-native**. Read `string.rs`, `union.rs`, `decimal.rs`, `filter.rs`, `fields.rs`, `bytes.rs`, `validation_state.rs` in full at the newly-designated oracle; diffed each against the de-designated 2.41.5 tree; executed the decimal formula by hand; ran every Rust-interop validator, the area runner, and the profile runner; and traced `scripts/run_all_tests.sh` to its dispatch. Empirically reproduced pytest collection behavior against the pin's real directory layout.

**Nine of pass-12's fourteen items are closed at the root**, including all three blockers' stated content, and the mechanical layer is clean for a sixth consecutive pass. What this pass finds is concentrated in one place: **the pin re-designation that closed pass-12 B-3 was applied to the tables and captions but not to the prose algorithms derived from the old tree**, plus two gate/ownership defects that the change set's own fixes created.

---

## BLOCKER

### B-1 — The string pipeline order contradicts the sole oracle, and the error was introduced by this revision

> **705-707:** "Its fixed pipeline, matching the pinned `string.rs` validator, is input conversion, whitespace stripping, Unicode-scalar length checks, pattern check, ASCII restriction, then case conversion."

The pinned validator (`pydantic/pydantic-core/src/validators/string.rs:110-178`) runs:

| step | line |
| --- | --- |
| input conversion (`coerce_numbers_to_str` is an argument, not a stage) | `:117-119` |
| `strip_whitespace` | `:122-124` |
| **`ascii_only` → `ErrorType::StringNotAscii`** | **`:125-127`** |
| `str.chars().count()` → `min_length` / `max_length` | `:129-155` |
| `pattern` | `:157-167` |
| `to_lower` / `to_upper` | `:169-172` |

ASCII is step 3, **before** length and pattern. The doc puts it after pattern — two boundaries inverted.

The root cause is mechanical and traceable: the de-designated standalone checkout has **zero** occurrences of `ascii_only` (`pydantic-core/src/validators/string.rs`, `grep -c` → `0`), and its pipeline is `strip :118 → length :123-148 → pattern :150 → case :162` — *exactly the doc's sentence with ASCII removed*. The sentence is 2.41.5's order with the new policy appended at a guessed slot, while asserting it matches the pin.

Observable: `ascii_only=True, max_length=1` on `"éé"` returns `string_not_ascii` upstream and `string_too_long` under the doc's rule. The named certifier cannot catch it — `core/string_pipeline_order` is scoped at **708-710** and **1279** to "one value that fails both length and pattern" plus length units. **No test in either core tree uses `ascii_only` at all** (5 hits, all in `string.rs`); the only observer anywhere is `pydantic/tests/test_types.py:7279 test_string_constraints_ascii_only`, which uses it alone and certifies the code, not the position. Four boundaries — strip↔ascii, ascii↔length, ascii↔case, length↔case — have no certifier in any table or native family. This fails the doc's own rule at **1366-1368**.

### B-2 — A serialization precedence rule is refuted by two of the doc's own mandatory anchors

> **1039:** "Call-time selections dominate sequence/mapping schema filters at the same node."

Call-time and schema **includes union**; they do not dominate. `pydantic-core/tests/serializers/test_list_tuple.py:104-105` — a listed anchor for `serializers/sequences` (row **1438**) — states it in an upstream comment and asserts it:

```python
# the two include lists are now combined via UNION! unlike in pydantic v1
assert v.to_python(seq_f('a',...,'h'), include={6}) == seq_f('b', 'd', 'f', 'g')
```

Schema `include={1,3,5}` plus call-time `include={6}` yields 1,3,5,**6**. Same at `tests/serializers/test_dict.py:35` (row **1439**): schema `include={'a','c'}` + call `include={'d'}` → `{'a':1,'d':4}`. Only call-time-include-over-schema-*exclude* dominates.

**1039 also contradicts the doc's own clause 2** at **1032-1033** ("…and is otherwise removed *unless the schema-declared inclusion selects it*, in which case clauses 3 and 4 decide"), which transcribes `filter.rs:202-206` exactly. Clauses 1-4 (**1028-1037**) are otherwise a faithful, line-for-line transcription of `filter.rs:161-233`; the summary sentence beneath them is the defect.

Compounding: **1040**'s "Schema-filter inclusion and exclusion combine as intersection" is semantically right but has **no anchor** — the discriminating tests (`test_list_tuple.py::test_filter`, `test_dict.py::test_filter`, `::test_filter_int`) are not in rows 1438-1442, and no listed anchor sets both a schema include and a schema exclude.

### B-3 — `rust_interop` is still outside the authoritative gate; the profile fix is data no code reads

`check_tiers.py` is genuinely fixed (`rust_interop_tiers.toml:29`; exit 0, `tiers=5 fixtures=35`), and the area passes standalone (`variants=4, failures=0`). The second half of pass-12 B-1 is not fixed.

All four profiles gained an identical `selected_areas` entry, but:

- `verification/profiles/{create-pr,merge,nightly,release}.json` — `execution_mode` is **UNSET** in all four.
- `profile_runner.py:200` — `return str(self.profile.get("execution_mode", "legacy-facade"))`.
- `profile_runner.py:157-158` — `selected_areas` is reached only via `run_selected_areas_only()` when `execution_mode == "selected-areas-only"`. The only profile that sets it is `python-interop-live.json:5`.
- The `legacy-facade` step list (`:160-187`) is hardcoded; every `selected_suites_for_area(...)` call site names a fixed area (`:324, 383, 394, 413, 424, 470, 483, 504, 594`) and **none is `rust_interop`**.
- `grep -rn rust_interop verification/runner/ scripts/run_all_tests.sh` → **0 hits**.

`scripts/run_all_tests.sh:108-110` execs `profiles run`, so the AGENTS.md-designated authoritative gate still never runs the area. Worse, the new fixture asserts the opposite in prose: `verification/areas/rust_interop/fixtures/opaque_resource_package_core/README.md:18-19` — "The repository verification profiles select that area runner; README prose alone is never passing evidence." That sentence is false as written.

### B-4 — The two-root ledger rule is defeated by a tracked symlink inside the pin

**1322-1326** is pass-12 MJ-7's fix: "every tracked file under each upstream `tests/` tree is enumerated; pytest collection is invoked with the two explicit roots `<pydantic-pin>/tests` and `<pydantic-pin>/pydantic-core/tests`, never the repository root."

`<pin>/tests/pydantic_core` is a git-tracked symlink — `git ls-files -s` → `120000 3cc4ac11…`, blob content `../pydantic-core/tests`. Reproduced against a tree mirroring the pin's exact layout (both `__init__.py` present, same basenames):

```
A) pytest --collect-only tests
   tests/pydantic_core/test_errors.py::test_core_only
   tests/pydantic_core/test_json.py::test_bool
   tests/pydantic_core/validators/test_union.py::test_union_bool_int
   tests/test_json.py::test_bool  ·  tests/test_main.py::test_api_only     → 5 collected

B) pytest --collect-only tests pydantic-core/tests        ← what 1324-1326 mandates
   ERROR pydantic-core/tests/test_json.py — import file mismatch:
     imported module 'tests.test_json' has this __file__ attribute: .../tests/test_json.py
   ERROR pydantic-core/tests/test_errors.py — ModuleNotFoundError: No module named 'tests.test_errors'
   !!!! Interrupted: 3 errors during collection !!!!
```

Against the real pin: 5 colliding basenames (`test_config.py`, `test_errors.py`, `test_json.py`, `test_strict.py`, `test_typing.py`); the symlink adds **119** `.py` files to a filesystem walk of `<pin>/tests` while `git ls-files` reports **one** entry; no `collect_ignore`, `norecursedirs`, or symlink exclusion exists in either `conftest.py` or either `pyproject.toml`; and `[tool.pytest]` (`pyproject.toml:182`) is inert so nothing mitigates it.

So the API root silently re-collects the entire engine suite under a second path spelling — every core anchor gains a duplicate node identity (`tests/pydantic_core/validators/test_union.py::test_left_to_right_union`), which **1398-1400** forbids ("each retained assertion/parameter has exactly one owning milestone") and **1336-1338** cannot reconcile. **113-118** dispositions the *other* duplicate (the standalone checkout) but not this one, and no pytest version is pinned even though collected node identity depends on the collector.

---

## MAJOR

**MJ-1 — Smart-union clauses 3-5 describe a total order; the pin is an intransitive fold.** Clauses 1, 3, 4, 5, 6, 7 are each individually **exact** against `union.rs:122-131, 145-153, 160-164, 176-189` and `validation_state.rs:14-19` — pass-12's MJ-2 is closed on content, and clause 7 now matches 2.47.0's deferred `should_omit` rather than 2.41.5's immediate abort. But `:145-153` is a **left fold with a non-transitive comparator**: with `X=(Strict,None)`, `Y=(Exact,Some(1))`, `Z=(Lax,Some(2))` — `X>Z` (mixed → exactness), `Z>Y` (2>1), `Y>X` (mixed → exactness) — a 3-cycle. `[X,Y,Z]→Z`, `[X,Z,Y]→Y`, `[Y,Z,X]→X`. An implementer following "counts decide… otherwise exactness ranks… a remaining tie selects the earliest declared candidate" builds an order-independent key and gets different answers. Clause 5's "earliest declared" is a property of the fold's local comparison, not of a ranking. The doc must say the comparison is sequential against the current best. (`any_schema` is the easiest cycle witness and Sifr rejects `Any`, but the modeling defect is independent of that witness.)

**MJ-2 — The union choice-label rule omits the fallback its own anchor asserts.** **953-954**: "the aggregate retains declaration order and each candidate's *declared* choice label." Upstream is `union.rs:281` — `let case_label = label.unwrap_or(choice.get_name());`. The doc's anchor `test_case_labels` (`test_union.py:428-447`, row **1431**) asserts two of three locs are validator *names* (`('none',)`, `('str',)`) and one is a declared label (`('my_label',)`), plus a union name of `union[none,my_label,str]`. Error locations are wire-visible.

**MJ-3 — Upstream Core Schema node kinds with Sifr meaning have no node, no anchor, and no disposition.** The 2.47.0 `CoreSchemaType` union has 53 kinds. Uncovered by the node algebra (**677-687**) and by both not-applicable enumerations (**1294-1305**, **1552-1556**): `fraction`, `complex`, `lax-or-strict`, `json-or-python`, `chain`, plus `allow_partial` and `missing-sentinel`. These are load-bearing, not exotic — pydantic's own generator emits `chain_schema` 16×, `json_or_python_schema` 12×, `lax_or_strict_schema` 6×, and each has a dedicated upstream test module (`tests/validators/test_{lax_or_strict,json_or_python,chain,complex,allow_partial}.py`). **`fraction` is attributable to this change set**: it is the single node kind present in 2.47.0 and absent from 2.41.5. Under **1363-1365** ("an uncovered capability fails the audit") and ps_0's gate ("every required feature family with a meaningful Pydantic oracle has pinned selector anchors"), these need a node or a written disposition.

**MJ-4 — `JsonLimitError`'s stated shape contradicts the implementation, four of its five "locked" budgets do not exist, and no milestone owns the reconciliation.** **1163-1166**: "the locked byte, depth, collection, string, and integer-digit budgets… returns typed `JsonLimitError { kind, limit, position }`." Reality: `crates/sifr_runtime/src/json.rs:104-107` is `{ message: String, limit: usize }` — no `kind`, no `position` — and that module implements **only** the integer-digit budget (`:5`, `:147-161`). The locked artifact mentions only decoder digit limits (`serialization_boundary_rules.md:79-80, 100-105`); the other four budgets are locked nowhere. Decision 17 (**189-191**) forbids the package from redefining these errors, ps_5 (**1884**) only *asserts* coverage, ps_9 (**1953-1957**) is a documentation PR, and ps_1-ps_3 never mention `sifr_runtime::json`. Pass-12 mn-2 landed the reference; the shape and the owner did not.

**MJ-5 — Phase 41's Quality Contract is restored in name only, and it is now the only phase file 15-43 with no Exit Gate.** The heading is back (`41_typed_data_model_and_validation.md:30`), closing the literal reading of `plans/roadmap.md:32`. But that rule requires "Phase entry/exit gates, milestone quality checks, and mandatory local validation commands," and the rewrite deleted all three: no entry criteria, no exit criteria, no milestone quality checks. Mechanically swept, **29/30 phase files 15-43 have `## Exit Gate`; only 41 does not** — the diff deleted it and nothing restores it. `roadmap.md:32` was not amended to exempt 41. Related: the ad hoc issue — the document that actually carries the gates — never names Phase 27 (0 hits) and never names `scripts/run_all_tests.sh` (0 hits), so Phase 27's diagnostic-stability invariant, which Phase 42 retains verbatim at `:41` and which ps_1's new `SIFR-META-*` channel touches, binds no ad hoc milestone gate.

**MJ-6 — Phase 32's deferral was redirected to a successor that is both unowned and factually wrong.** `32_async_ecosystem.md:60-62, :93-95, :1050-1051` now read "blocked on a future general process/IPC serialization contract." That phrase names no phase, issue, or document and exists nowhere else in the repo — the prior pointer ("Phase 41") at least resolved. And it is stale: `plans/roadmap.md:73` marks 36.4 `completed, audited` with `sifr.ipc` shipped 2026-06-09, and `stdlib/sifr/ipc.sifr` exists with `SchemaId:8`, `ProtocolVersion:22`, `require_serializable[T]:84`. The archived closeout is explicit that the residual blocker is a *public process-worker pool API*, not a serialization contract. A `completed` phase now carries an untrackable, incorrect forward dependency.

**MJ-7 — Typed JSON serialization is now ownerless inside `sifr-lang/sifr`, and no document accepts that.** **140-143** dispositions the deleted `milestone_41_1` deliverable ("intentionally subsumed by the one `TypeAdapter[T]`/`BaseModel` Core Schema path"). The single-authority argument is sound; the consequence is unstated. `stdlib/sifr/json.sifr:369, 411` remain `JsonValue`-only (`loads(s: str) -> Result[JsonValue, JSONDecodeError]`, `dumps(value: JsonValue)`); no typed derive exists anywhere; **2011-2013** bars an in-repo successor and non-goal **1712** bars a compiler one. Downstream, `42_web_framework_and_platform_expansion.md:9, 38-39` make the **in-repo** web framework's entry gate an **external-org** package release, while `41:22` forbids a second contract and `42:45` forbids fallback code — no schedule or abandonment risk is addressed.

**MJ-8 — Phase 42's entry gate cites a milestone that predates release certification.** `42:9, :38-39` require "released `milestone_ps_10`". But ps_10 (**1966-1980**) is the API-completeness milestone; **ps_11** (**1982-1999**) owns "Certify supported compiler/core/package version combinations" (**1990**), differential validation (**1987**), and adversarial/panic testing (**1988**), and its gate (**1996-1998**) is what requires released artifacts. **130-134** propagates the same `ps_10` framing.

**MJ-9 — `ConstPackageIssue`'s open template arguments cannot fit the registry's closed per-code contract.** Pass-12 MJ-1 is otherwise closed at the root: `ConstSpecializationOutcome`/`ConstPackageIssue` collide with nothing (0 hits in `crates/`), **498-501** correctly disclaims `sifr_package::PackageDiagnostic` (`crates/sifr_package/src/diag/mod.rs:12-17`) and `sifr_driver::CompileResult` (`crates/sifr_driver/src/diagnostics.rs:13-16`), family `META` is legal under every validator (`code_coverage.py:14`, `code_baseline_coverage.py:27`, `registry_tests.rs:149-181`), URLs stay registry-derived (`registry.rs:296-299`), and "a warning does not make checking fail" matches `cli_model_and_entrypoint.rs:857-868`. What is unaddressed: `active_entry!` (`registry.rs:619-634`) fixes one `message_template` and a **closed `declared_args`** list per code, and `registry_tests.rs:203-225` asserts every `{placeholder}` is declared. **503-505** routes a package-supplied, open argument set into three fixed codes. The doc never reconciles the two.

---

## MINOR (edit-worthy)

- **mn-1** — **1008-1009** "Signed indices are normalized against the node's pre-filter sequence length": `filter.rs:23-25` applies `__mod__` *unconditionally*, so a positive out-of-range index also wraps (`exclude={8}` on an 8-element list removes index 0). No upstream test exercises it.
- **mn-2** — **1049-1051** "An empty nested inclusion on a composite… empties that subtree; it is inert only below a scalar leaf" is right for the plain case (pass-12 mn-6 closed) but wrong when the node carries a schema-declared include: `explicit_include` is true, `filter.rs:202-206` is skipped, and `default_filter` emits the entry — the same carve-out the doc states correctly at **1032-1033**.
- **mn-3** — Unrecorded divergence: a precompiled `re.Pattern` forces the Python engine *under the default engine*, not only under `regex_engine='python-re'` (`string.rs:299-305`, `// we default to using the python re engine so that any flags are preserved`; asserted by `test_compiled_regex` with `engine=None`). **711-712**/**1651-1653** cover only the explicit `python-re` mode. Flag translation and Python-only regex syntax becoming build-time rejections are unstated.
- **mn-4** — `internal_docs/rust_interop_architecture.md:961-994` lists 33 of 35 fixtures; the +2 lines fixed the new row and a neighbour but `async_runtime_core` and `panic_boundary_wrapper_emission` remain omitted — the latter being a row this change set's own handoff depends on.
- **mn-5** — `opaque_resource_package_core/README.md:10-19` cites the area's four schema validators as its "owning checks," which only validate the row's own metadata. `verification/areas/rust_interop/README.md:21-22` requires "links to the owning implementation checks"; the sibling `opaque_resource_core/README.md` cites `cargo test -p sifr_runtime interop`. Related: `rust-interop-runtime-ecosystem-certification.md:91-92` mandates "runner wiring that executes the fixture rather than relying on README-only evidence," and none exists — the fixture is text-linted only (`check_fixture_matrix.py:481-540`, no `subprocess`).
- **mn-6** — `plans/phases/index.md:50` still reads "Phase 41: Typed Data Model and Validation (Pydantic-Parity Track) | unspecified" against the renamed `41:1`. The file claims to be generated (`index.md:3`); no generator exists.
- **mn-7** — Ownership of `SIFR-INT-0009` documentation is claimed twice: ps_1 must activate it "with its documentation and tests" (**1782**) while **138-140** and **1953-1957** assign the durable-doc update to ps_9. ps_9's exit gate (**1962-1964**) omits the doc PR entirely, and its three named contents already exist in the artifact (`:10-14`, `:39-43`, `:79-80`) — so it is close to a no-op while the real updates (flipping `internal_docs/integer_model.md:476` from Reserved, adding `x-sifr-integer-profile` which `integer_model.md` lacks) go unnamed.
- **mn-8** — **1109-1110**'s `SIFR-INT-0009` payload omits the artifact's required "selected or missing profile" (`serialization_boundary_rules.md:111`) and drops "when known" from static range (`:112`). Separately, the artifact's conditional generated-client warning (`:43`) has no owning code, milestone, or repo, and cannot be `SIFR-INT-0009` because a `DiagnosticCode` carries exactly one declared severity.
- **mn-9** — A `SIFR-META-0002` warning takes LSP class `hard` (`crates/sifr_lsp/src/conversion.rs:454-467`), which `internal_docs/lsp_server.md:152-153` defines as unsuppressible and unfixable — a package-authored, unsuppressible editor warning, unstated. `"rule"` is a load-bearing reserved arg name (`conversion.rs:463`); **516-518**'s validation list does not reserve it.
- **mn-10** — ps_2 (**1812-1816**) changes released stdlib `re.compile` from `compile(pattern: str) -> Pattern` (`stdlib/sifr/re.sifr:225`) to a two-arg `Result`-returning form, and adds a "thin `compile_flags` compatibility view" that sits awkwardly against non-goal **1717-1718**. No document classifies it as breaking or names a migration path.
- **mn-11** — Anchor gaps against **1366-1368**: union→union count bubbling and exactness-floor bubbling are certified by `test_nested_unions_bubble_up_field_count` (`test_union.py:1191`) and `test_smart_union_validator_function` (`:718`), both clean and neither listed; the listed `test_smart_union_does_nested_model_field_counting` (`:1021`) discriminates additive accumulation *within* one candidate, not clause 6's "enclosing validation state." Likewise `test_error_type` (row **1413**) constructs `PydanticKnownError` directly and never runs a validator, so it anchors the code registry, not decimal semantics. And `test_positional_tuple` (row **1438**) contains no `include`/`exclude` at all.
- **mn-12** — Decimal wording: **1648-1650**'s "sequential resolution" for `to_lower`+`to_upper` should be *precedence* resolution (`string.rs:169-172`, `to_upper` silently dropped); **715-724** omits that the error `ctx` carries the *allowance*, not the observed count (`decimal.rs:192`), and that emission is first-match-wins across the three codes.

*Not attributable to this change set, but adjacent:* `verification/areas/diagnostics/checks/code_coverage.py:174` still checks `docs/errors/<CODE>.md` after the `.mdx` migration and exits 1 with 204 errors today — ps_1 must add `SIFR-META-*` pages onto that surface.

---

## Pass-12 closure matrix

| Item | Status | Evidence |
| --- | --- | --- |
| **B-1** tier gate + profile wiring | **HALF-CLOSED** | `check_tiers.py` exit 0, `tiers=5 fixtures=35`; area runner `variants=4 failures=0`. Profile half unfixed → **B-3** |
| **B-2** `SIFR-INT-0009` owner | **CLOSED at the root** | ps_1 owns registry entry + verifier + gate (**1781-1782, 1794-1795**); rejection condition present at **742**; prerequisite **1728**; artifact updated in-tree (`serialization_boundary_rules.md:10-14`). Residues → mn-7, mn-8 |
| **B-3** pin pair | **CLOSED** | Single parity pin (**108-118**); captions explicit (**1404-1406, 1524-1525**) and load-bearing — 15/197 core anchors are AST-different between trees; `ascii_only` dispositioned as a policy (**704**) and anchored (**1529**); `Omit` clause now matches 2.47.0 `should_omit` and contradicts 2.41.5. **But the prose algorithms were not re-derived → B-1, MJ-3** |
| **MJ-1** diagnostic channel | **CLOSED at the root** | New frontend contracts, 0 collisions, registry-owned `SIFR-META-*`, `META` legal by form, URLs preserved, warning semantics match `diagnostic_exit_code`. Residues → **MJ-9**, mn-9 |
| **MJ-2** union clauses 2-4 | **CLOSED on content** | Clauses 1/3/4/5/6/7 verified exact against `union.rs:122-131, 145-153, 160-164, 176-189`; discriminating anchors added. New defects → **MJ-1**, **MJ-2** |
| **MJ-3** `left_to_right` | **CLOSED** | Node family **684**, algorithm **955-957**, ps_7 deliverable **1909**, anchor **1431** verified discriminating (`test_union.py:458-485`), auto-collapse **929-930** matches `union.rs:76-82`, labels **953** |
| **MJ-4** decimal counting basis | **CLOSED at the root** | Both-set precondition, `saturating_sub` on both sides, both-forms conjunction all exact vs `decimal.rs:152-197`. Hand-executed: `0.000`→accept, `0`→reject, `1.500`→reject, `0.5`→accept, `100`→reject; the doc's wording reproduces every case including the zero asymmetry. Anchors added and verified |
| **MJ-5** policy entry + artifact owner | **CLOSED** | **1621-1623** covers both the JSON-number and JSON-Schema divergences; ps_9 coordinated PR **1953-1957**; artifact updated. Residue → mn-7 |
| **MJ-6** bounded-integer schema row | **CLOSED** | **1098-1107** matches `serialization_boundary_rules.md:39-43` including the conditional client warning and the unbounded-schema prohibition |
| **MJ-7** Phase 41 supersession | **PARTIAL** | Quality Contract heading restored; Phase 40 dependency restored (`41:16`); Phase 42 consistent on `ps_10`. But no entry/exit criteria, only phase 15-43 with no Exit Gate, Phase 32 redirect regressed, deliverable ownerless → **MJ-5, MJ-6, MJ-7, MJ-8** |
| **MJ-8** sysroot row counts | **CLOSED** | `sifr_sysroot_and_stdlib_architecture.md:157-164` now 12, enumeration set-matches the matrix exactly; gate `future_runtime_rows=12` PASS; `opaque_resource_ecosystem` gone from all live docs. Residue → mn-4 |
| **mn-1** conditional client warning | **CLOSED** | **1105-1106** ↔ artifact `:43` |
| **mn-2** `JsonLimitError` | **PARTIAL** | Present at **1163-1166, 1671, 1884**; shape and budget provenance wrong, owner missing → **MJ-4** |
| **mn-3 / mn-4 / mn-5 / mn-7** | **CLOSED** | Case policy **1648-1650**; `coerce_numbers_to_str` **704** + `regex_engine` **711-712, 1651-1653**; `core/string_pipeline_order` **708-710**; units **706, 708** (`string.rs:130` scalars, `bytes.rs:85` bytes). New residues → mn-3, mn-12, and B-1 |
| **mn-6** empty nested selection | **PARTIAL** | Plain case fixed; schema-declared-include exception remains → mn-2 |
| **mn-8** cert Scope + README | **PARTIAL** | Scope bullet added (`:19-20`), ownership bidirectional and consistent. README circular; still no milestone or row gate → mn-5 |
| **Mechanical layer** | **CLOSED (6th pass)** | 52 rows, 83 path instances / 71 distinct (83/83 exist and git-tracked), 298 anchors → 298 distinct `(path, selector)` pairs, 0 unresolved, 0 ambiguous, 0 class-only, 0 duplicate owners, 0 shadowed. 0/298 trip any of eight xfail/skip mechanisms; detector validated by 184 non-anchor hits across 4,267 functions. 52/52 table families + 6/6 native families gated exactly once in the owning milestone; `serializers/*` expands to exactly 11; 0 orphans |

---

## Can `milestone_ps_0` be re-approved?

**No.** Its exit gate (**1764-1769**) requires no unresolved ownership/semantic-authority/sequencing ambiguity, anchors for every family with a meaningful oracle, and that an omitted upstream file or node be **mechanically detectable**. Four classes block it:

1. **Two stated algorithms are refuted by the sole oracle** — the string pipeline by `string.rs:125-127`, and the serialization precedence summary by two of the doc's own mandatory anchors (**B-1**, **B-2**). Neither has a fixture that would catch it.
2. **The detection mechanism itself does not run as specified.** The two-root collection rule aborts on the pin's real layout and, single-rooted, silently duplicates 119 files (**B-4**) — so "mechanically detectable" is not yet true. And the change set's own certification row remains outside `scripts/run_all_tests.sh` (**B-3**), with the fixture's README asserting the contrary.
3. **Coverage and ownership gaps** — seven upstream node kinds with Sifr meaning and no disposition, one of them introduced by the pin re-designation (**MJ-3**); `JsonLimitError`'s shape and four "locked" budgets with no owning milestone in either repo (**MJ-4**); the registry's closed declared-args contract versus open package template arguments (**MJ-9**).
4. **Governance and sequencing** — Phase 41 is the only phase file 15-43 without an Exit Gate and its restored Quality Contract carries none of the three mandated elements (**MJ-5**); Phase 32's deferral now points at an unowned, factually superseded successor (**MJ-6**); typed JSON serialization is ownerless in-repo while Phase 42 hard-depends on an external-org release (**MJ-7**); Phase 42 gates on `ps_10` when certification is `ps_11` (**MJ-8**).

**Where the design stands.** The conformance machinery is finished and clean for a sixth consecutive pass, and this revision genuinely closed the hardest items: the pin is now a single unambiguous parity oracle with load-bearing captions, `SIFR-INT-0009` has an in-repo owner and a gate, the diagnostic channel no longer collides with anything and keeps codes, severity, and URLs registry-owned, the decimal digit rule is arithmetically exact including the counter-intuitive zero cases, the smart-union clauses match 2.47.0 clause by clause, the integer JSON/JSON-Schema contract now agrees with the locked artifact on every row, and the sysroot enumeration is a set-exact 12/12. The architecture proper remains sound.

The failure mode has shifted again — from "the fix landed on the contract but not on the owner" to **"the pin was re-designated in the tables but the prose was not re-derived from it."** Both B-1 and MJ-3 are direct products of that: `ascii_only` does not exist in the tree the pipeline sentence was written against, and `fraction` does not exist there either. Three changes would make this class checkable: **(a)** require every prose algorithm to cite the pinned file *and line range* it was read from, so a re-pin mechanically invalidates it — **1366-1368** already demands the source, but not a citation the audit can re-verify; **(b)** require every claimed pipeline or precedence *boundary* — not merely each step — to name its certifying assertion, which would have caught B-1, B-2, and the schema include∧exclude gap; and **(c)** disposition the pin's own directory layout (symlinks, duplicate basenames, collector version) in the ledger rules, since the ledger's guarantee currently rests on an untested assumption about how pytest walks the tree.
