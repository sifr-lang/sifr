All six audits complete and cross-verified. Here is the review.

---

# VERDICT: **NEEDS REVISION**

**Method.** Re-parsed both tables mechanically: **79 path instances (71 distinct modules)** and **296 anchor instances** all resolve at the pinned revisions — 0 unresolved, 0 ambiguous bare selectors, 0 duplicate `path::selector` owners, 0 anchors without an assertion, **50/50 fixture families gated exactly once** (11 `serializers/*` via the ps_8 wildcard at 1469). All 23 anchors named for removal by passes 4–6 are gone. I then read anchor bodies, parametrize decorators, and consumed fixtures, and traced `src/serializers/filter.rs` at the pin. Both repos confirmed at `f59e929c…` / `383eb95a…`.

The completeness machinery, the six pass-6 `B-2` substitutions, all five pass-6 `MJ-1` retargetings, `MJ-2`, and both minors are genuinely closed. Three things are not.

---

## BLOCKER

### B-1 — `Selection` has no form for a typed mapping's keys; 832–834 claims an equivalence it lacks

`Fields` (809) is keyed by *field name* and "recurses by field name" (816). `Elements` (810–813) is keyed by *signed index* "normalized against the current **sequence length**" (818–819). The document separates field names from mapping keys everywhere else (865 vs. 868) and records from typed mappings (580). So for a typed-mapping node **only `All` applies** — and `All` "removes the current node" (816), not each entry. Upstream confirms these are distinct operations: `key_filter` addresses entries by key hash with no index normalization, while `index_filter` applies `map_negative_indices` (`filter.rs:129-140` vs. `93-101`), yet `merge_all_value` (`filter.rs:326-348`) makes `__all__` **container-generic**.

Affected mandatory anchors — every filtering assertion in `serializers/mappings` (row **1088**, gated at 1467–1470):

| Anchor | Upstream | Selector | Why inexpressible |
| --- | --- | --- | --- |
| `test_dict.py::test_exclude` | `:51` | `exclude={'__all__'}` → `{}` | Needs a per-entry default on a mapping. `Fields` has no `default`; `All` would drop the whole map. |
| `test_dict.py::test_filter_args_nested` p4/p6/p7 | `:102,:106,:107` | `include={'__all__': {0}}`, `exclude={'__all__': {'__all__'}}`, `exclude={'__all__': {0}}` | Mapping-key default over statically-unknown keys. |
| same, **p8** | `:108` | `exclude={'__all__': {0}, '3': {1}}` | **Decisive**: one node must carry a default *and* a merging per-key override. `Fields` gives keys without a default; `Elements` gives a default with index addressing. No combination works. |
| `test_dict.py::test_include` | `:39-41` | `include={5}` on `{'a':1,…,5:6}` | No key-typed selector map. An integer would land in `Elements.indices` and be reinterpreted as a position (negatives wrapped mod length). Row **1095** independently anchors URL- and UUID-keyed mappings (`test_url_dict_keys`, `test_uuid_key`). |

Line 832–834 — "preserving their portable **default**, signed-index, override, and composition behavior" — is false for mappings. Rules 1036–1037 (every retained parameterization) and 1030–1032 (a normalized Sifr expectation per retained assertion) cannot be discharged.

*Note:* the record case (`test_model.py::test_include_exclude_args` p12, `:176`, `exclude={'__all__'}`) **is** rescuable, since a record's field list is static and `__all__` desugars to the enumerated `Fields` map — but the document never says so.

**Correction.** Add a third keyed alternative and one rule sentence:
```text
  | Entries { default: Option[Selection], keys: ordered map[declared key type, Selection] }
```
"`Entries` applies to typed mappings; entries match by validated key value, not position, with no index normalization; overlay follows the `Elements` rule." State that `__all__` over a record desugars to the static field enumeration. Otherwise classify mapping-key selection `not-applicable`, reclassify the three anchors, and delete "default" from 833.

### B-2 — Line 830's composition rule is wrong, and produces incorrect output for a mandatory anchor

> 830: "Inclusion then narrows the value, and exclusion removes from that result."

Combined with 818–819 ("normalized against the **current** sequence length"), exclusion indices resolve against the *post-inclusion* sequence. Upstream evaluates **both polarities in one pass against the original index** — `self.filter(index, index, include, exclude)` (`filter.rs:99`), exclude-then-include inside one call (`filter.rs:152-233`).

Verified against `test_filter_runtime_more` (row **1087**, gated at ps_8), on `list('abcdefgh')`:

| Params (`:198-204`) | Upstream expects | Doc 830 yields |
| --- | --- | --- |
| `include={1,3,5}, exclude={5}` | `['b','d']` | `['b','d','f']` ✗ |
| `include={2,3,5}.keys(), exclude={5}` | `['c','d']` | `['c','d','f']` ✗ |
| `include=[1,2,3], exclude=[2,3]` | `['b']` | `['b','c']` ✗ |

(Field-keyed nodes are unaffected — keys are stable — so `test_include_exclude_args` p7 in both `test_model.py` and `test_typed_dict.py` still agrees.)

**Correction.** Replace 830 with: "Both polarities are resolved in one pass against each element's original index (or key): an entry is emitted only if it is not excluded and, when an inclusion exists, is included. Signed-index normalization uses the pre-filter sequence length."

### B-3 — Three mandatory anchors have no portable behavioral residue

Rules 1025–1026 and 1027–1029 bar a truthiness-only assertion, a Python `repr`, a reflection invariant, or an assertion solely about a not-applicable mechanism. Each verified by direct reading:

| Row | Anchor | Evidence |
| --- | --- | --- |
| **1076** ps_6 `validators/defaults` | `tests/validators/test_with_default.py::test_default_value` (`:493-500`) | **No `validate_*` call at all.** `r = v.get_default_value()`; `assert r is not None`; `assert r.value == [1,2,3]`. `validate_default` is unset, so `r.value` is a verbatim echo of the `default=` construction argument — introspection plus a truthiness check. |
| **1127** ps_7 `api/validators` | `tests/test_model_validator.py::test_nested_models` (`:121-144`) | Both `Model.model_validate(...)` results are **discarded**; the only two assertions are `calls == ['before','after']` and `calls == ['before']*3 + ['after']*3` — a `nonlocal` list side-channel. Not a declared field value, serialized output, or normalized error, so 1110–1114 retains nothing. |
| **1093** ps_8 `serializers/unions_callbacks_recursion` | `tests/serializers/test_definitions_recursive.py::test_custom_ser` (`:60-84`) | Sole assertion's expected value is a Python container `repr`: `'sub_branch': "{'name': 'branch', 'sub_branch': None}"`, produced by `to_string_ser_schema` over a dict. Barred by 1027. |

**Correction.** `test_default_value` → `test_typed_dict_default` (`:25-38`, a missing field takes its default) or `test_default_value_validate_default` (`:503-510`, coerces `['1','2','3']`→`[1,2,3]`). `test_nested_models` → drop; the same row's `test_annotated_validator_runs_before_field_validators` already asserts ordering through a value. `test_custom_ser` → the sibling `tests/serializers/test_definitions.py::test_custom_ser` (`:6-13`, `[1,2,3]`→`['1','2','3']`) is portable, but the row's module list would need it.

---

## MAJOR

**MJ-1 — `api/constraints` is gated at ps_5 but no anchor can run there.** Row **1123**; gate **1419**. All 10 model-based anchors need a `BaseModel` with a named field (`ConBytesModel` `:136-141`, `ConStringModel` `:743-748`, `StrModel` `:1495-1500`, in-test models at `:241,:257,:448,:490`), and 6 assert a **field-scoped** `loc` (`('v',)` at `:185,:266,:499,:776`; `('str_check',)` at `:1510,:1525`) — so the record is part of the expectation, not harness. `test_decimal_precision` (`:1145-1152`) has no model but its sole entry point is `TypeAdapter(Decimal)`. ps_6 is "the **first** complete `BaseModel` validation API" (1429–1430); `TypeAdapter[T]` is a ps_9 deliverable (1474); ps_5's only porting deliverable is scoped to the "Pydantic **Core** corpus" (1414). This is the pass-6 `MJ-1` defect class in a family pass 6 did not enumerate. **Correction:** move `api/constraints` to ps_6's gate (1434–1437) and `test_decimal_precision` to ps_9 (row 1131).

**MJ-2 — Validation/serialization *context* is anchored seven times but never specified.** "context" appears only at **758** ("context values"), **859** ("optional context"), and **1446** (ps_7 "…ordering and context"). It is absent from the serializer-plan ownership list (792–802), from ps_8's deliverables (1459–1465), and from the typed-callback contract (482–490). Yet these mandatory anchors assert nothing else: `test_after`, `test_mutable_context`, `test_typed_dict`, `test_wrap` (row **1084**), `test_serialize_json_context` (**1130**), `test_validate_python_context`, `test_validate_json_context` (**1131**). Their content requires exactly what is unspecified — `test_validation_context.py:14-16` and `:60-62` pass `None`, `{1: 10}`, and `'frogspawn'` to the *same* validator (an untyped heterogeneous context, against 1205 and the no-`Any` rule at 679–681); `:21,:27` assert caller-visible mutation through the callback. The three API-table anchors discard every return value, so 1110–1114's whitelist retains nothing. **Correction:** specify the context's Sifr type, ownership, and mutability in the validation-state and serializer-plan contracts, add it to ps_8's deliverables, and extend 1111 to admit callback-observable context.

**MJ-3 — No rule for a non-empty nested selection under a scalar leaf.** 831 covers only "An empty nested selection changes nothing"; `{1}` is not empty. Upstream keeps the leaf. Affects 10 retained parameterizations: `test_model.py::test_include_exclude_args` p4/p8/p10 (`:168,:172,:174`), `test_typed_dict.py::test_include_exclude_args` p4/p8/p9/p10 (`:59,:63,:64,:65`), `test_dict.py::test_include` `:37`, `test_exclude` `:53`. `test_typed_dict` p10 is load-bearing: `include={'0','1'}, exclude={'1':{1}}` → `{'0':0,'1':1}` requires `'1'` **retained** despite appearing in `exclude`. Under 1461's "**typed** recursive include/exclude selections", `Fields{d: Elements{…}}` where `d: int` is ill-typed and thus unconstructible. **Correction:** state that a nested selection under a scalar leaf is inert, or record non-`same` dispositions for all 10.

**MJ-4 — `test_filter_runtime_more`'s selector spellings are Python-only and unclassified.** 831–832 enumerates only "set/dict/`True`/ellipsis/`__all__`". 5 of 7 params use `list`, `dict_keys`, or duck-typed `__contains__` (`filter.rs:288-301`). Params 4–6 assert `__contains__` **overrides** `__iter__`: `ExplicitContains.__contains__` → `{2,5}` beats inherited `__iter__` → `[1,2,5]`, and the expected `['c','f']` proves it (`:181-188,:201-203`). Doc 980 forbids porting duck-typing behavior. No disposition recorded.

**MJ-5 — `tests/test_config.py::test_invalid_extra` (row 1126, ps_6) has no validation residue.** `:464-481`: a bare `ConfigDict(extra='invalid-value')` with no assertion, then three `SchemaError` routes on a stringly-typed config value — via `model_config`, via `create_model` (runtime model creation, Non-Goal 1278), and via `@pydantic_dataclass`. Sifr's extra policy is a typed declaration (1210–1212), so the specific error is impossible; a build-time-diagnostic normalization is defensible but none is recorded. **Correction:** drop it, or reclassify as an `adapted` build-time entry outside `api/config_fields`.

**MJ-6 — Two required Core Schema nodes are never implemented, gated, or carved out.** Line **579** lists "**uniqueness** and **typed refinement**" as required Constraints nodes. Grep confirms each appears exactly once in 1,606 lines. No deliverable in 1408–1499 names either; no fixture family anchors either (`validators/collections` and `api/constraints` cover set *length*, not uniqueness; `test_new_type_schema` at 1132 is ps_9 JSON-Schema description, not validation); and neither gets the oracle-less native carve-out that 972–975 gives fixed-width integers. This is what makes ps_0's gate clause "every required feature family with a meaningful Pydantic oracle has pinned selector anchors" (1332–1334) unmet.

---

## MINOR (edit-worthy)

- **An anchor parameterization xfails at the pin — 1025–1026 violated, and pass-6's closure claim is falsified.** `tests/validators/test_with_default.py::test_default_factory_not_called_if_existing_error` (row 1076) consumes fixture `container_schema_builder`, `@pytest.fixture(params=['model','typed_dict','dataclass','arguments_v3'])`, whose `arguments_v3` branch is `raise pytest.xfail(...)` at `:853`. Pass 6 asserted "0/303 xfail/skip via decorator, module `pytestmark`, `param(marks=…)`, **fixture**, or in-body"; that is wrong. *Fix:* disposition `[arguments_v3]` as not-applicable and make 1025–1026 per-retained-parameterization.
- **`validators/definitions_recursion` has no recursion guard Sifr can hit.** `test_recursion_branch`'s distinguishing assertion is cycle-based (`b['branch'] = b`, `:318-319`, `recursion_loop`), not representable per 1219–1221; its remaining residue (`:312-316`) duplicates `test_branch_nullable` (`:20-53`). ps_7's gate promises "bounded execution" (1450) and 599 requires rejecting "unbounded recursive entry". *Fix:* add a depth-limit anchor or declare the guard a Sifr-native contract like 972–975.
- **`tests/serializers/test_definitions_recursive.py::test_recursive_function`** (row 1093, `:87-100`): schema is a **non-nullable** self-recursive record (`my_ref = TypedDict{root: my_ref}`) and the asserted input `{'root': {'root': {}}}` omits the required field at the innermost level — unrepresentable as an owned Sifr value. 1219–1221 covers only cycles.
- **`tests/test_schema_functions.py::test_err_on_invalid`** (row 1059, ps_4): sole assertion rejects `core_schema.invalid_schema()`, a placeholder whose upstream purpose is Python's deferred model build (`core_schema.py:475-495`: "we never plan to use this"; `validators/mod.rs:527`), i.e. the runtime-schema-construction path rejected by decision 9 (138–139) and Non-Goal 1278. No adaptation recorded.
- **`tests/validators/test_string.py::test_unicode_error`** (row 1064): the named behavior — unpaired surrogates in a `str` (`:124-140`, the test's own comment attributes it to the PyO3 boundary) — is unreachable in Sifr; the surviving assertion duplicates `test_constrained_str`. The portable byte-source form already lives in `test_lax_bytes_validator` in the same row.
- **"the error-disclosure policy" (858) is referenced once and defined nowhere**, yet it governs `test_hide_input_in_error` in a gated family (1062/1419).
- **Parameter identities sit outside the equality rule.** 1010–1012 and the ps_0 gate (1335–1336) quantify over "paths and collected node identities"; acceptance at 1557–1559 promises "no upstream path, collected selector, **or parameter** can disappear". Add "and parameter identities" / "or parameter".
- **1036–1037 licenses dropping `adapted` parameterizations** ("unless … a non-`same` disposition"), contradicting 942 and 946. Restrict to `not-applicable`/`rejected`.
- **Node-classification quantifier mismatch:** 1008 classifies nodes only "in a conformance file" while 1011–1012 fails on any "unclassified … node" — a file misclassified as infrastructure silently removes its nodes.
- **997–999 is unsatisfiable as written** ("Before `ps_4` implementation begins, the external repository stores …") — the repository is created *by* ps_4 (1388–1390). 1394–1395 says it correctly.
- **Prerequisites table 1289–1297 is self-referential** against ps_1/ps_2/ps_3's own deliverables (1340–1342, 1357–1359, 1375–1376); only the `ps_4` row is a true precondition.
- **Overlay (822–824, "an explicit branch replaces a base `All`") and union (828–829, "`All` dominates") answer the same pair oppositely** and are never declared distinct operations. Not a contradiction — they are scoped to different operands — but say so.

I examined and **cleared**: `test_type_adapter_dump_json` (row 1130 — the asserted behavior is a plain model serializer's JSON output, `:962-973`; `TypeAdapter` is harness for a `TypedDict` root, matching pass 6's own standard), `test_url_ok`, `test_any_url_parts`, `test_basic_alias`, `test_validate_multiple`, `test_self_forward_ref_collection`, `test_enum_exactness`, `test_union_timedelta_respects_instanceof_check`, `test_recursive_model`, `test_callable_discriminated_union_recursive`, `test_model_validate_strict`/`_json_strict` (the pass-5 `xfail` is a different function in a different module), and all 15 parameterizations of `test_advanced_exclude_nested_lists` — the strongest positive result of this pass.

---

## Pass-6 closure matrix

| Item | Requirement | Status | Evidence |
| --- | --- | --- | --- |
| **B-1** | `Selection`: default/`__all__` + signed-index overrides, recursive overlay, negative-index normalization, schema∪call-time composition, deterministic and non-contradictory | **PARTIAL** | `Elements{default, indices}` (810–813), overlay (822–826) and union (828–829) added. **All 15** `test_advanced_exclude_nested_lists` params now reproduce upstream, and overlay is a faithful statement of `merge_all_value`/`merge_dicts`; `test_include`'s negative indices and schema∪call-time union both check out. **But** mappings have no selector form (**B-1**), 830's composition rule is wrong (**B-2**), and scalar-leaf nesting is unstated (**MJ-3**) |
| **B-2** | Six residue-free anchors removed/replaced | **CLOSED** | All six gone. `test_validation_alias_parse_data` (`:622-639`) + `test_validation_alias_priority_json` (`:660-675`, 8 behavioral assertions) replace the reflection-only path anchor; `test_type_adapter_json_schemas_title_description` (`:5422-5436`) asserts positive emitted content; `test_input_type_invalid` dropped, leaving `test_json_invalid` (a pure pre-validation parse failure, ps_4-executable). *Three new residue-free anchors found → **B-3*** |
| **MJ-1** | Cross-milestone selectors correctly owned | **CLOSED** for all five | `test_aliases_json_schema`→ps_9 (1132); `test_serialization_alias`→ps_8 (1130); `test_typed_dict`→ps_6 (1073); `test_validate_default`→ps_7 (1084, all 4 `inner_schema` params are `no_info_*_validator_function`); tagged-union warning case removed. *Defect class recurs at `api/constraints` → **MJ-1(new)*** |
| **MJ-2** | ps_6 strings-profile engine row + gate; bare `str` and structural roots both expressible | **CLOSED** | Row 1073 `core/strings_profile_models` + ps_6 gate 1435. `test_bool`/`test_validate_strings` are bare-`str` roots, `test_dict` is a mapping (ps_5-owned), `test_typed_dict` is the record case (ps_6). None uses `generate_schema_calls` or monkeypatching |
| **mn-1** | Bare `str` root admitted; no `Any`, no second tree | **CLOSED** | 536–537, 671–673, 676–678 agree; 679–681 "there is no `Any` or package-owned recursive value tree" |
| **mn-2** | Safe input summary optional; thin root/literal anchors removed | **CLOSED** | 858 "optional safe input summary, controlled by the error-disclosure policy"; `test_root_model_as_field` and `test_literal_none` gone |
| **Total-set equality** | Git-tree ledger, exact equality, content hash, pin procedure | **CLOSED** (2 minor loopholes) | 1002–1014, 1178–1194, 1557–1559; parameters outside the equality clause and the file/node quantifier mismatch are the residual gaps |
| **Milestone gates** | Every family gated exactly once | **CLOSED** mechanically | 50/50 families, each in exactly one gate; ps_8's `serializers/*` wildcard covers exactly the 11 ps_8-owned families. *Content defects at ps_5 → **MJ-1**, ps_8 → **MJ-2*** |
| **No ps_11 catch-up** | Re-audit only | **CLOSED** | 1503–1505, 1560–1562; no family assigned to ps_11 |
| **External package/demo ownership** | External repo only | **CLOSED** | All 11 sites consistent (118–122, 162–164, 171–182, 203–216, 256–257, 1388–1393, 1511–1512, 1526–1532, 1584–1588) |
| **Status/history** | Accurate, artifacts linked | **CLOSED** | 5–11 matches every artifact's verdict; all six links resolve and are non-empty; the 0-byte pass-7 file is correctly unlinked |

---

## Can `milestone_ps_0` be re-approved?

**No.** Its deliverable is "Approve the pinned module and selector baseline" (1321), and its gate forbids unresolved **sequencing ambiguity** and requires "every required feature family with a meaningful Pydantic oracle [to have] pinned selector anchors" (1331–1336). Both clauses are currently unmet: `api/constraints` at ps_5 and the unspecified serialization context are sequencing ambiguity (**MJ-1**, **MJ-2**), and `uniqueness`/`typed refinement` are required nodes with no anchors (**MJ-6**).

The baseline is materially better than pass 6's. The mechanics are now clean on every dimension I can check automatically, the six residue-free anchors are genuinely replaced with behavioral substitutes, all five cross-milestone retargetings landed correctly, and the `Selection` overlay rules are a faithful and complete statement of upstream's `__all__` merge for sequences — all 15 parameterizations of the anchor that drove pass-6 B-1 now check out. That was the hard part and it is done.

What remains is narrower but real: the same language is silent on typed mappings (**B-1**), states one composition rule that is demonstrably wrong (**B-2**), and three more anchors assert nothing portable (**B-3**). Of these, **B-2 is the one to fix first** — it is a false statement of semantics rather than a missing case, and unlike **B-1** it cannot be found by reading the grammar; it only surfaces when you evaluate the rule against `test_filter_runtime_more`. **B-1** needs one grammar alternative plus one sentence; **B-3** and every MAJOR are row substitutions or added text.

The architecture proper remains untouched and sound: bridge version 2, Core Schema as sole authority, the validated arena and no-third-tree invariant, crate-neutral specialized scalars, the certification prerequisites, and the `sifr-lang/sifr` ↔ `sifr-lang/pydantic-sifr` split.

*No files were modified. `plans/reviews/active/native-pydantic-sifr-architecture-agent-review-pass-7.md` is still a 0-byte placeholder — say the word and I'll write this review there.*
