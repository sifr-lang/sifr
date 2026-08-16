I have complete evidence. All 75 path instances and 303 anchors verified at the pinned revisions; I read bodies and parametrize tables for every anchor and traced the serializer filter semantics in upstream source.

---

# VERDICT: **NEEDS REVISION**

**Method.** Re-parsed both tables, resolved every module through the directory-inheritance rule against `git show <pinned-rev>:<path>`, AST-matched every anchor to a module-level `def`, then read all 303 bodies **plus their parametrize decorators** (pass 5 read bodies only; decorators are where several defects live). Results: **75/75** path instances exist; **303/303** anchors resolve to exactly one module-level `def` in their row; **0** ambiguous bare selectors; **0** duplicate `path::selector` owners; **0** anchors xfail/skipped by decorator, module `pytestmark`, `pytest.param(marks=…)`, fixture, or in-body call; **0** anchors with no assertion at all; **49/49** fixture families covered by an exit gate (11 `serializers/*` via the wildcard at line 1448). `exclude_unset`: 0 anchors. `round_trip=`: 0. `defer_build`: 0. `monkeypatch`/`generate_schema_calls`: 0.

The completeness machinery, the audit rules, ps_4's retargeting, and six of pass 5's nine items are genuinely closed. Two root causes are not.

---

## BLOCKER

### B-1 — The `Selection` language cannot express the two `ps_8` anchors it was introduced for, states no merge rule, and asserts an equivalence it does not have

**Locations:** design 800–814; anchors at 1070 (`serializers/models`) and 1066 (`serializers/sequences`); `ps_8` gate 1448–1449.

`Selection` (803–808) is a four-way sum with **no combination form, no default-plus-override form, and no merge rule**; `Indices[…]` and `Each[…]` are mutually exclusive at any node, and `Each` carries no index binding.

**Witness 1 — `tests/serializers/test_model.py::test_advanced_exclude_nested_lists` (mandatory, row 1070).** **14 of its 15 parameterizations** put `'__all__'` *and* an integer index in the same mapping, and upstream implements **two different** resolutions, both load-bearing and both absent from 810–812:

| Upstream param (`test_model.py`) | Behavior | Expressible? |
| --- | --- | --- |
| `{'subs': {'__all__': {'subsubs': {0}}, 0: {'subsubs': {1}}}}` (id `Merge sub sets 1`, :384) | **union**: index 0 loses subsubs {0,1}; index 1 loses {0} | No — two different effective selections for two indices of one list |
| `{'subs': {'__all__': {'subsubs'}, 0: {'subsubs': {'__all__': {'j'}}}}}` (id `Ignore __all__ for index with defined exclude 1`, :416) | **override**: index 0's nested selection replaces `__all__`'s whole-node exclusion (upstream comment records v1 gave the opposite answer) | No |
| `{'subs': {'__all__': ..., 0: {'subsubs'}}}` (id `…3`, :426) → `{'subs': [{'k': 1}]}` | index 0 kept-minus-`subsubs`; **every other index dropped** | No — `Indices` enumeration is length-dependent; `__all__` is not |

The last case also kills the only escape hatch (hand-merging into `Indices`), since `include`/`exclude` are call-time arguments reused across differently-sized data.

**Witness 2 — `tests/serializers/test_list_tuple.py::test_include` (mandatory, row 1066), upstream :96–116.** Requires (a) a **schema-declared** `filter_seq_schema(include={1,3,5})` **unioned** with a **call-time** `include={6}` — `# the two include lists are now combined via UNION!` — and (b) **negative indices**: `include={-1: None, -2: None}` resolved modulo length. `Selection` has neither a signed-index notion nor any composition rule between schema-level and call-time selections; 792 lists "inclusion/exclusion" as plan-owned without addressing it.

Both shapes are pure index/field selection over acyclic owned data — no Python-only mechanism — so 812–814's claim that the language "preserv[es] their portable selection behavior" is false, and rule 1017–1019 requires every retained parameterization to be included.

**Correction.** Replace the element alternative with a default-plus-override product and state the resolution rule, e.g.

```text
Selection =
    All
  | Fields[ordered map[field name, Selection]]
  | Elements{ default: Option[Selection], indices: ordered map[signed index, Selection] }
```

plus: how `default` and `indices` combine (union for leaf `All`, override for nested selections — matching upstream's two ids), how negative indices resolve, and how a schema-declared selection composes with a call-time one. Alternatively classify the mixed-`__all__`, negative-index, and schema∪call-time parameterizations `adapted`/`not-applicable` with reasons — but then 812–814 must stop claiming preservation.

### B-2 — Six mandatory anchors still have no portable behavioral residue, across five milestones

Rules 1005–1009 require a behavioral assertion relevant to Sifr and bar an "assertion solely about a rejected/not-applicable mechanism"; 1013–1015 bars a forbidden mechanism from being "the behavior being asserted"; 1022–1023 makes each an automatic gate failure. Verified at the pinned revisions:

| Row | Milestone / family | Anchor | Why nothing survives |
| --- | --- | --- | --- |
| 1104 | `ps_6` `api/aliases` | `tests/test_aliases.py::test_validation_alias_path` (:601–605) | Sole assertion is `Model.model_fields['x'].validation_alias == value`. **No input, output, or error** — pure metadata reflection, excluded by 1007–1009 and 1089–1093. Zero retained assertions. |
| 1062 | `ps_7` `validators/definitions_recursion` | `tests/validators/test_definitions_recursive.py::test_union_cycle` (:602–638) | Only assertion is a `recursion_loop` error from `data['foobar'].append(data)`. Cyclic input is not representable (1198–1200); upstream gates the guard on Python object identity, and a depth trip yields a different `loc` and is unreachable for non-Python input. Contrast its sibling `test_recursion_branch`, which *does* have residue. |
| 1108 | `ps_7` `api/generics_recursion` | `tests/test_forward_ref.py::test_forward_ref_in_generic` (:987–1003) | `x: dict['type[Bar]', type['Bar']]`; sole assertion `Foo(x={Bar: Bar}).x[Bar] is Bar` — runtime class objects as keys *and* values, checked by `is`. Barred by 1186 and Non-Goal 1255. |
| 1072 | `ps_8` `serializers/unions_callbacks_recursion` | `tests/serializers/test_functions.py::test_function_wrap` (:348–357) | All three assertions equal whole strings embedding the internal handler repr (`'result=3 repr=SerializationCallable(serializer=int)'`), so the repr *is* the expectation; and `len(value)` on `s.to_python('foo')` against an `int_schema` is statically impossible under 816–820. No correct-typed value anywhere, so 1082–1084 does not cover it. |
| 1040 | `ps_4` `core/json_foundation` | `tests/test_json.py::test_input_type_invalid` (:39–42) | Asserts a **runtime** `json_type` error from `v.validate_json([])`. Sifr's entry point is statically typed (the document's own example, 524, takes `payload: bytes`), so this is a compile error and the code is unreachable. This is **one of only two** anchors gating `core/json_foundation` (1380–1383). |
| 1111 | `ps_9` `api/json_schema` | `tests/test_json_schema.py::test_type_adapter_json_schemas_without_definitions` (:5439–5445) | Sole assertion `'definitions' not in json_schema` — a negative containment on a v1-era key, vacuously true for any implementation that emits `$defs`; no schema content asserted. |

I verified the **other 297** anchors do have residue, including all 38 my mechanical scan flagged as mixed: the 15 `pytest.warns` serializer anchors each retain correct-typed-value assertions (1082–1084 applies); `test_simple_serializers` and `test_none_fallback` are honestly carved out at 1084–1086 (`all_types` = 16 params, exactly one — `'none'` — retained); the `__dict__`/field-set/`repr`/`create_module` cases are covered by 1089–1093 and 1013–1015. I also examined and **cleared** `test_enum_exactness` (`is not MyEnum.a` observes union-variant selection, i.e. the exactness ranking at 758–772 — portable, like `isinstance(result, ModelB)` in `test_nested_unions_bubble_up_field_count`) and `test_union_timedelta_respects_instanceof_check` (pass 5's suspicion does not hold: `pyproject.toml:123-124` sets `filterwarnings = ['error', …]`, so it emits no warning, and `'foo'` legitimately inhabits the second branch's unconstrained plain validator — the residue is "a union member with a custom serializer serializes through that serializer", which is expressible; classify `adapted`).

**Correction.** Drop or replace these six. `test_validation_alias_path` → the row already has behavioral `test_validation_alias`; add a path-lookup case with an actual input. `test_union_cycle` → `test_recursion_branch` and `test_complex_recursive_type` already carry recursion behavior; if a depth-limit anchor is wanted, pick one that trips on acyclic depth. `test_forward_ref_in_generic` → `test_recursive_model`/`test_self_forward_ref_collection`/`test_complex_nesting` already cover forward refs. `test_function_wrap` → `test_function_general`/`test_custom_ser` in the same row. `test_input_type_invalid` → a second genuinely foundational malformed-JSON/limit case (note `tests/test_json.py`'s only other pure-parse anchor is `test_json_invalid`, already listed), or state that the statically-eliminated case is a Sifr-native compile-time contract like the fixed-width carve-out at 952–955. `test_type_adapter_json_schemas_without_definitions` → an anchor asserting the emitted schema positively.

---

## MAJOR

### MJ-1 — Five mandatory anchors require a **later** milestone's feature; pass-5 MJ-1 is closed for `ps_4` but recurs at `ps_5`–`ps_7`

`ps_0`'s gate (1310–1315) forbids sequencing ambiguity; 1022–1023 makes each an owning-gate failure. Using `model_dump()`/`TypeAdapter` merely to *observe* a validated value is acceptable harness and I did not count it — these five have a later milestone's feature as the **asserted behavior**:

| Row | Assigned | Anchor | Needs |
| --- | --- | --- | --- |
| 1104 | `ps_6` `api/aliases` | `tests/test_aliases.py::test_aliases_json_schema` (:585–590) | **`ps_9`.** Sole assertion is `Model.model_json_schema() == {…}`. No residue to salvage. |
| 1104 | `ps_6` `api/aliases` | `tests/test_aliases.py::test_serialization_alias` (:520–527) | **`ps_8`.** Behavior is `m.model_dump(by_alias=True) == {'foo': 'bar'}`; the field has only a *serialization* alias, so the `m.x == 'bar'` residue involves no alias at all. |
| 1050 | `ps_5` `core/strings_profile` | `tests/test_validate_strings.py::test_typed_dict` (:109–119) | **`ps_6`.** `typed_dict_schema` with two fields; `ps_5` (1385–1393) implements no record validator — `validators/typed_dict` is `ps_6`-owned (row 1054). |
| 1055 | `ps_6` `validators/defaults` | `tests/validators/test_with_default.py::test_validate_default` (:354) | **`ps_7`.** **All four** `inner_schema` params are `no_info_{after,before,wrap,wrap}_validator_function` (ids `after`, `before`, `wrap-before`, `wrap-after`); rule 1017–1019 requires all retained params. |
| 1107 | `ps_7` `api/discriminated_unions` | `tests/test_discriminated_union.py::test_tagged_union_no_fallback_on_matched_discriminator` (:2445) | **`ps_8`** — it is a serializer regression test (`ta.dump_python`); and its only discriminating assertions inspect **warning text** (`'ToolType1' not in warning_text`), a channel 1201–1202 declares not-applicable, with its premise (`tool.my_enum = 'disabled'` on an enum field) statically impossible. |

**Correction.** `test_aliases_json_schema` → `ps_9` `api/json_schema`; `test_serialization_alias` → `ps_8` (`api/serialization`, or a `ps_8` `api/aliases_serialization` family added to 1448); `test_typed_dict` → a `ps_6` strings family (see MJ-2); `test_validate_default` → `ps_7`, or record the four parameterizations as `ps_7`-owned per 1017–1019; drop `test_tagged_union_no_fallback_on_matched_discriminator`.

### MJ-2 — `ps_6`'s strings-profile entry points are gated nowhere; pass-5 MJ-2 item 2 moved rather than closed

**Locations:** 1409–1410, gate 1411–1416; `ps_5` row 1050 and gate 1398.

`ps_5` now correctly owns `core/strings_profile` and gates it (1398) — that part is closed. But `ps_6` line 1409–1410 promises "the first complete `BaseModel` validation API, **including** JSON, structural, and **strings-profile** entry points", and its gate (1411–1416) names no strings family. Combined with MJ-1, the *record-level* strings profile is implemented at `ps_6`, its only upstream anchor (`test_typed_dict`) is mis-assigned to `ps_5`, and nothing gates it. This is the exact "implemented at one milestone, gated at none" shape pass 4 M3 and pass 5 MJ-2 raised.

**Correction.** Move `test_typed_dict` (and, if desired, upstream `test_model`) to a `ps_6` row with family `core/strings_profile_models`, and add that family to `ps_6`'s gate.

---

## MINOR (edit-worthy)

- **mn-1 — The `strings` profile forbids the root shape two of its own four mandatory anchors use.** Line 669 says the profile "requires a structural mapping/sequence whose scalar leaves are strings"; 672 says "generic over a **structural** input `S`"; 535 says "strings-leaf **structural** input". But `tests/test_validate_strings.py::test_bool` (:12–17) is three bare-scalar-root assertions (`v.validate_strings('true') is True`), and `test_validate_strings` (:20–46) passes a bare string in **all 16** parameterizations. Lines 673–674 do not rescue this: "accepts `S` **only when** every terminal scalar … is `str`" states a necessary condition on leaves, granting no permission for a scalar root, and 674's "nested records, mappings, and sequences are allowed" is an additive permission for interior nodes. As written, both anchors retain zero assertions. *Fix:* widen 669/672/535 to admit a bare `str` root — which is the natural env-var/query-param case the profile wants anyway.
- **mn-2 — Three retained anchors have residue thin enough to be worth strengthening, though I do not count them as violations:** `tests/test_root_model.py::test_root_model_as_field` (row 1113; sole assertion `isinstance(m.root_model, MyRootModel)` — statically guaranteed in Sifr, leaving only "input shape `{'root_model': 1}` validates"), `tests/validators/test_literal.py::test_literal_none` (row 1057; two `isinstance_python` truthiness probes plus a `SchemaValidator` repr prefix), and `tests/test_errors.py::test_hide_input_in_error` (row 1042; sole assertion `'input' not in error`, which also implies the "safe input summary" at 838 must be **optional** in the error contract — it is currently the only unmarked field in a list whose neighbors say "optional").

---

## Pass-5 closure matrix

| Item | Requirement | Status | Evidence |
| --- | --- | --- | --- |
| **B-new rule** | Add an explicit anchor-portability rule (behavioral assertion, passes upstream, no rejected mechanism) | **CLOSED** | 1005–1009, 1010–1015, 1017–1019 added; carve-outs at 1076–1079, 1082–1087, 1089–1093 |
| **B-new(a)** | No vacuous anchors | **NOT CLOSED** | 0/303 have no assert at all, and every pass-5-named vacuous anchor is gone — but 6 survive with no Sifr-relevant residue → **B-2** |
| **B-new(b)** | No upstream-`xfail` anchor | **CLOSED** | `test_validate_json_strict` removed; mechanically re-verified 0/303 xfail/skip via decorator, module `pytestmark`, `param(marks=…)`, fixture, or in-body |
| **B-new(c)** | Remove rebuild-only, reflection-only, field-set/`__dict__`-only, cyclic-only, warning/fallback-only, exception-construction-only, hash-only anchors | **PARTIAL** | `model_rebuild` 0 anchors; `from_exception_data` 0; `hash()` 0; `exclude_unset` 0; `validate_call` 0; `__pydantic_fields_set__`/`__dict__` survive only as excluded material around real residue (1089–1093). Cyclic-only (`test_union_cycle`) and reflection-only (`test_validation_alias_path`, `test_forward_ref_in_generic`) remain → **B-2** |
| **B-new sem. 1** | Serializer type mismatches statically impossible | **CLOSED** | 816–820 states it plainly; 1082–1087 classifies the warn/passthrough family per-assertion; 1201–1202 in the policy. Verified against all 15 `pytest.warns` anchors — each retains correct-value assertions |
| **B-new sem. 2** | `exclude_unset` resolved without hidden field-set state | **CLOSED** | 1195–1197 classifies it not-applicable, retains `exclude_defaults`/`exclude_none`/typed selections; **0 anchors** reference `exclude_unset`; consistent with 776–779 and 786 |
| **B-new sem. 3** | One typed recursive `Selection` with deterministic semantics | **NOT CLOSED** | Language added (800–814) but no combination/merge/override form, no signed indices, no schema∪call-time rule; two mandatory `ps_8` anchors inexpressible; 812–814's preservation claim false → **B-1** |
| **MJ-1** | `ps_4` limited to executable foundation anchors | **CLOSED (for `ps_4`)** | All 5 `ps_4` anchors read: 3 `SchemaError`-at-construction + 2 pre-validation JSON failures; `test_error_type`, `test_all_errors`, `test_schema_as_string` gone; `test_error_json*`/`test_hide_input_in_error`/`test_input_types` → `ps_5` (1041–1042), `test_loc_with_dots`/`test_error_loc` → `ps_6` (1051–1052), with families in both gates. *But* `test_input_type_invalid` has no runtime residue (**B-2**) and the same defect class recurs at `ps_5`–`ps_7` (**MJ-1**) |
| **MJ-2** | Strings profile: engine row, `ps_5` gate, capability, typed input signature | **PARTIAL** | Row 1050 added; `core/strings_profile` in `ps_5`'s gate (1398); capability at 535; input `S` defined at 673–679 with no `Any` and no second tree; `generate_schema_calls`/`defer_build` anchors gone. *But* `ps_6`'s entry points ungated (**MJ-2**), `test_typed_dict` mis-assigned (**MJ-1**), scalar root forbidden (**mn-1**) |
| **mn-1** | Module column = anchor source, not whole-module ownership | **CLOSED** | 996–1000: "The module column identifies the source of that row's anchors… A non-anchor `same` or `adapted` node is owned by the milestone that implements its feature"; 1026–1030 permits repeat modules with one owning milestone per assertion |
| **mn-2** | Define the pin-update procedure | **CLOSED** | Dedicated section 1157–1173 (6 numbered steps, exact set equality, historical revisions retained); `ps_0` deliverable at 1304; referenced by `ps_11` at 1482–1483 |
| **mn-3** | Fixed-width carve-out in the `ps_0` gate | **CLOSED** | 1310–1315 now reads "every required feature family **with a meaningful Pydantic oracle**… Sifr-native families such as fixed-width integer overflow have explicit native contracts"; matches 952–955 |
| **Total-set equality** | Ledger computed from Git trees, exact equality, content hash | **CLOSED** | 984–994, 1168–1170, acceptance 1536–1538 |
| **No `ps_11` catch-up** | Re-audit only | **CLOSED** | 1482–1484 and 1540–1541 |
| **Demo ownership** | External repo only | **CLOSED** | Consistent across all 7 sites: 161–163, 213, 255–256, 1489–1491, 1496, 1563–1567 |
| **Status/review history** | Distinguish pre-manifest approval | **CLOSED** | 5–11 states passes 4 and 5 returned `NEEDS REVISION` and "`milestone_ps_0` is not re-approved until its closure pass is satisfied"; artifacts linked at 18–19 |

---

## Can `milestone_ps_0` be re-approved?

**No.** Its deliverable is "Approve the pinned module and selector baseline in this document" (1300) and its gate forbids unresolved sequencing ambiguity (1310–1315).

The baseline's *machinery* is now sound and materially better than pass 5's: the Git-tree-derived ledger, the exact-equality and content-hash rules, the anchor-portability rule, the per-assertion AST-hash requirement for mixed tests, and three honest carve-outs (descriptors, serializer wrong-type, `test_simple_serializers`/`test_none_fallback` parameters). 297 of 303 anchors survive scrutiny, `ps_4` is genuinely executable, and two of the three semantic decisions are complete and well-stated.

What blocks approval is narrow and specific: the `Selection` language is incomplete against two of the very anchors it was written for and claims an equivalence it lacks (**B-1**); six mandatory anchors are automatic gate failures under the document's own rule 1022–1023, spanning `ps_4`, `ps_6`, `ps_7`, `ps_8`, and `ps_9` (**B-2**); five anchors require a later milestone's feature (**MJ-1**); `ps_6`'s strings entry points are gated nowhere (**MJ-2**); and the strings profile forbids the root shape two of its own anchors use (**mn-1**).

The architecture proper remains untouched and sound — bridge version 2, Core Schema as sole authority, the validated arena and no-third-tree invariant, crate-neutral specialized scalars, the certification prerequisites, and the `sifr-lang/sifr` ↔ `sifr-lang/pydantic-sifr` split. The decisive item for pass 7 is **B-1**: it is a design gap, not a row substitution, and it is the one finding that cannot be fixed by editing a table cell.

*No files were modified. `plans/reviews/active/native-pydantic-sifr-architecture-opus5-review-pass-6.md` exists as a 0-byte placeholder; say the word and I'll write this review there.*
