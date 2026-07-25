No files were modified. Verification complete: all 77 path instances (73 distinct modules) and all 335 anchors resolve at the pinned revisions, with zero within-row ambiguity — but I read the anchor bodies, and that is where the problems are.

---

# VERDICT: **NEEDS REVISION**

**Method.** Re-parsed both tables, resolved every module through the new directory-inheritance rule against `git show <pinned-rev>:<path>`, AST-matched every anchor to a module-level `def`, then read the *bodies* (plus parametrize decorators, fixtures and helpers) of all 335 anchors. Results: 77/77 path instances exist; 335/335 anchors exist and are module-level; **0** ambiguous bare selectors; **0** duplicate `path::selector` owners; **51/51** fixture families named in an exit gate. Seven of the nine pass-4 items are genuinely closed. The two that are not, plus one new gap, are below.

---

## BLOCKER

### B-new — The pinned selector baseline `ps_0` must approve contains anchors that are vacuous, upstream-`xfail`, or depend on mechanisms this document explicitly rejects; four milestone gates are therefore unsatisfiable as written

Line 972–973 declares the selectors "mandatory portable anchors" and line 980–981 makes "a missing, renamed, ambiguously qualified, or unclassified selector" an automatic failure of "the upstream audit and the owning milestone gate". Only two adaptations are recorded anywhere: `@property`/descriptors (1034–1037) and `defer_build` (1061–1064). Everything below is unrecorded. All quotes verified at the pinned revisions.

**(a) Anchors with no behavioral assertion.**

| Doc line | Anchor | Body at pinned rev |
| --- | --- | --- |
| 1052 | `test_generic_recursive_models_parametrized` | `tests/test_generics.py:1744-1761` — two class defs then `Model1[str].model_rebuild()` / `Model2[str].model_rebuild()`. **No `assert`, no `pytest.raises`.** This is *verbatim* the pass-4 M4(a) defect ("the test body does nothing but call `model_rebuild` — it asserts no validation behavior at all"), reintroduced under a new name after the old one was deleted. Contradicts decision 9 (135–136), Non-Goal 1178, acceptance 1434. |
| 1049 | `test_config_inf_nan_enabled` | `tests/test_config.py:624-631` — `assert Model(value=inf_nan_capable_type(inf_nan_value))`. Asserts only that the instance is truthy; says nothing about inf/nan. (Its sibling `test_config_inf_nan_disabled` *is* behavioral.) |
| 1049 | `test_invalid_config_keys` | `tests/test_config.py:484-488` — body is `@validate_call(config={'alias_generator': lambda x: x})` on an empty function. No assertion, and `validate_call` is Python call-signature validation, classified not-applicable at 1069–1070. |
| 1056 | `test_any_url_success` | `tests/test_networks.py:115-119` — `assert Model(v=value).v, value`. The trailing comma makes it a 2-arg assert; 60+ parameters all funnel into a truthiness check. |
| 1051 | `test_recursive_discriminated_union` | `tests/test_discriminated_union.py:2376-2411` — declarations only, no `assert`/`raises`. It pins *schema construction*, not the "deterministic success/error behavior" `ps_7`'s gate (1347–1348) promises. |

**(b) An anchor that does not pass upstream.** Line 1054 mandates `test_validate_strings`' sibling `test_validate_json_strict`, which at `tests/test_type_adapter.py:225` carries `@pytest.mark.xfail(reason='Need to fix this in https://github.com/pydantic/pydantic/pull/5944')`. The differential oracle (1095–1102) runs the neutral corpus against pinned Pydantic — this fixture fails on the oracle side by construction.

**(c) Anchors requiring mechanisms on the "Do not port" list (928–939) or the not-applicable list (1121–1126), with no portable residue.**

- **`__pydantic_fields_set__` / `__dict__`** — forbidden at 934, and 768 says the field set "is not retained on the constructed Sifr model". Yet: `serializers/test_model.py:697` literally constructs `FieldsSetModel(foo=1, bar=2, spam=3, __pydantic_fields_set__={'bar','spam'})` for **`test_exclude_unset`** (line 1027); `validators/test_model.py` asserts `m.__dict__` and `m.__pydantic_fields_set__` in **all six** anchors of `validators/model` (line 1012); also `test_allow_extra` (1047, `tests/test_main.py:273`), `test_root_model_specialized` (1057, `:65`), `test_computed_fields_get` (1053, `:50`), `test_model_serializer_plain` (1053, `:433`). `ps_6`'s and `ps_8`'s gates (1332–1335, 1366) require these families to pass.
- **`from_attributes`** — "not applicable to fixed-layout Sifr values" (1124–1126). `test_model_class_extra_forbid` (1012) is built on `__dir__`/`__getattr__` probing plus `v.validate_python(Wrapper(m), from_attributes=True, ...)` (`validators/test_model.py:109-133`).
- **`revalidate_instances`/instance identity** — not applicable (1125). `test_model_class_strict` (1012) asserts `m is m2` and `m4 is m3` (`:638`, `:657`).
- **Runtime rebuild** — rejected (135–136, 1178). Besides (a): `test_recursive_models_union` (1052, `tests/test_forward_ref.py:715`) and `test_generic_recursive_models` (1052, `tests/test_generics.py:1708`) both call `model_rebuild()`. `test_recursive_models_union` was pass-4's own *recommended replacement*.
- **Serializer type-mismatch warn-and-passthrough** — the document has no warning channel (824–832) and 1127 forbids fallback; `T: StructuralProject` (776–779) makes the mismatch statically impossible. Yet ~18 `ps_8` anchors are built on it: `test_datetime`, `test_decimal` (1021), `test_timedelta`, `test_config_timedelta`, `test_union_timedelta_respects_instanceof_check` (1022), `test_positional_tuple` (1023), `test_plain_enum`/`test_int_enum`/`test_str_enum`, `test_int_literal`/`test_str_literal`, `test_none_fallback`, `test_nullable` (1028), `test_url`/`test_multi_host_url` (1031), `test_uuid` (1031), `test_function_wrap` (1029). Example: `serializers/test_timedelta.py:20-30` — `pytest.warns(...)` then `assert v.to_python(123, mode='json') == 123` against a `timedelta` schema.
- **Runtime reflection / Python `repr`** — Non-Goal 1176. `test_all_schema_functions_used` (997) is `get_type_hints(...)` over `core_schema.CoreSchema.__args__`; `test_all_errors` (1032) compares `core_schema.ErrorType.__args__`; `test_model_field_default_info` (1049) asserts an exact `str(Model.model_fields)` repr — a like-for-like replacement of the `test_config_defaults_match` that pass-4 M4(d) had removed; likewise `test_annotated`, `test_annotated_alias` (1058), `test_basic_alias` (1048), `test_self_forward_ref_collection` (1052), `test_field_order` (1047), `test_validate_multiple` (1050). Internal-`repr` assertions: `test_not_schema_definition_error` (996, `assert repr(v).count('TypedDictField') == 101`), `test_model` (1027), `test_int_literal`/`test_str_literal` (1028), `test_tagged_union`/`test_tagged_union_with_aliases` (1029), `test_discriminator_function`/`test_simple_tagged_union` (1017), `test_url_ok` (1006), `test_any_url_parts` (1056), `test_root_model_recursive` (1057), `test_generic`/`test_parse_generic_json` (1052).
- **Python exception construction** — 938. `test_raise_validation_error_hide_input` (1059) is `raise ValidationError.from_exception_data(...)`; its behavior already duplicates `test_hide_input_in_error` (999).
- **`hash()` on metadata objects** — 1120. `test_url_constraints_hash_equal` (1056).
- **Cyclic input data** — unrepresentable in the input abstraction (645–683) and in `ValidatedValue` (691–706). `test_cyclic_data` (1019, `cyclic_data['b'] = {'a': cyclic_data}`) and `test_cyclic_recursion` (1029, id-based "Circular reference detected").
- **Python subclass identity** — 932. `test_simple_serializers` (1021) asserts `type(v) == type(expected_python)` over `IntSubClass(42)`, `FloatSubClass(42)`, `MyIntEnum.one`.

**Correction (root cause, not another round of substitutions).** The baseline was selected by test *name*; nothing in the document prevents that. Add to the rules at 968–981: *a mandatory anchor must (i) contain a behavioral assertion, (ii) pass at its pinned revision, and (iii) not depend on any mechanism listed under "Do not port as Sifr behavior" (928–939) or declared not-applicable in the Public Compatibility Policy (1121–1126) unless its adaptation is recorded in this section like the two existing carve-outs.* Then re-select the affected anchors from bodies, and resolve the three semantic questions the current set exposes but the design never answers: (1) serializer behavior when value and schema types cannot match — the Sifr answer is "statically impossible", which makes the whole warn/fallback family `not-applicable` and should be stated; (2) `exclude_unset` — either declare a per-instance set field in the construction/projection contract or classify `exclude_unset` `not-applicable` alongside `from_attributes`; (3) the include/exclude *value language* (`{'bar': ...}` vs `{'bar': {}}` vs `True`, which changes semantics in `serializers/test_model.py:715-718`) — `inclusion/exclusion` at 785 names the capability but never defines its representation, and `test_include_exclude_args`, `test_filter_args_nested`, `test_advanced_exclude_nested_lists` all depend on it.

---

## MAJOR

### MJ-1 — `ps_4`'s exit gate still requires validators `ps_4` does not implement; pass-4 M3 is closed in form, not in substance

**Locations:** rows 996, 998, 999; `ps_4` checklist 1281–1296; exit gate 1298–1302.

`ps_4` builds the repository, schema format v1 + verifier, error/input/arena/plan foundations, `jiter`, licenses/fuzz/bench. It implements no bool, int, str, list, dict, tuple or model validator. Yet its gate requires `core/json_foundation` and `core/errors_foundation` to pass, and:

| Anchor | Doc line | What its body needs |
| --- | --- | --- |
| `test_schema_as_string` | 996 | `tests/test_build.py:9-11` — `v.validate_python('tRuE') is True`: string→bool lax coercion, i.e. `ps_5` (1306) |
| `test_error_json` | 999 | `str_schema(min_length=3)` — string validator + length constraint (`ps_5`) |
| `test_error_json_loc` | 999 | `dict[str, list[int]]` (`ps_5` collections) |
| `test_hide_input_in_error` | 999 | `int_schema` (`ps_5`) |
| `test_loc_with_dots` | 999 | `typed_dict` + `validation_alias='foo.bar'` + `tuple_positional` (`ps_6` aliases, 1324) |
| `test_input_types` | 998 | `validate_json` of `list[int]` (`ps_5`) |
| `test_error_loc` | 998 | `typed_dict` + `extras_schema` + `extra_behavior='allow'` (`ps_6`, 1325) |

Only `test_json_invalid`, `test_input_type_invalid`, `test_err_on_invalid`, `test_invalid_custom_error*` and `test_build_recursive_schema_from_defs` are executable with `ps_4`'s deliverables. Compounding it: `test_error_type` (999) is decorated `@pytest.mark.parametrize('error_type, message, context', all_errors)` at `tests/test_errors.py:423`, i.e. the **complete** ~160-entry error catalog (`:256-420`) — the exact reason pass-4 M3 moved `test_all_errors` to `ps_10` (1032). Two rows in the same table now disagree about when the catalog is complete.

**Correction.** Move `test_error_json`, `test_error_json_loc`, `test_hide_input_in_error`, `test_input_types` to `ps_5`; `test_loc_with_dots`, `test_error_loc` to `ps_6`; add the resulting families to those gates. Replace `test_schema_as_string` with a genuine build/verify case — `tests/test_build.py` contains only five tests and the other three are `pickle`/internal-`repr`, so the real schema-error corpus is the `SchemaError` cases in `tests/test_schema_functions.py`, `validators/test_typed_dict.py`, `validators/test_with_default.py` and `validators/test_definitions_recursive.py`. Scope `test_error_type` at `ps_4` to the foundation error codes and give the catalog-complete parameterization to `ps_10` with `test_all_errors`.

### MJ-2 — The `strings` input profile added to close M4(b) is implemented at `ps_5`, gated at no milestone before `ps_9`, missing from the canonical capability list, and has no defined Sifr input type

**Locations:** 530–538, 663–672, 1310–1311, 1314–1318, 1327–1328, 1054, 1061–1064, 1372–1373.

The design text itself is sound — three profiles over one abstraction, "not a third schema compiler, value representation, or validation engine" (670–672) — and the `defer_build` carve-out (1061–1064) is exactly the right mechanism. But:

1. **No engine-side row.** `tests/test_validate_strings.py` exists at the pinned revision (7 module-level tests: `test_bool`, `test_validate_strings`, `test_dict`, `test_model`, `test_dataclass`, `test_typed_dict`, `test_validate_strings_forbid_extra_fn_override`) and is the only upstream module dedicated to this profile. It appears in **neither** table.
2. **Gated nowhere until `ps_9`.** `ps_5` implements the profile (1310–1311) but its gate (1314–1318) names no strings family; the single strings anchor is `test_validate_strings` in `api/type_adapter` at `ps_9` (1054). This is precisely the pass-4 M3 shape ("ported at `ps_4` and gated at no milestone") recreated by the M4(b) fix. `ps_6` promises "strings-profile entry points" (1327–1328) with nothing gating them either.
3. **Absent from the public surface.** The canonical capabilities list (532–538) still enumerates only structural and JSON validation — the omission pass-4 M4(b) cited as evidence.
4. **Input type undefined.** 667–668 requires "a structural mapping/sequence whose scalar leaves are strings". Sifr is statically typed and has no `Any`; upstream `validate_strings` accepts nested string trees (`{'a': '1', 'b': {'c': '2'}}`). The document never names the Sifr type the profile accepts, so `Model.model_validate_strings(...)`'s signature is unspecified.

Also note the chosen anchor's fixture asserts `generate_schema_calls.count` via a monkeypatched internal (`tests/conftest.py:164`), which the `defer_build` note does not cover.

**Correction.** Add `| ps_5 | tests/test_validate_strings.py | test_bool, test_validate_strings, test_dict, test_typed_dict | core/strings_profile |`, add `core/strings_profile` to `ps_5`'s gate, add "validate a strings-leaf structural input as `T`" to 532–538, and name the accepted Sifr input type in 663–672.

---

## MINOR (edit-worthy)

- **mn-1 — Rule 970 contradicts rule 986–987, and the "Complete module audit scope" column leaves porting-vs-classification ambiguous.** 970 says "a named module is completely inventoried before **its** milestone closes" (singular) while 986–987 explicitly permits a module in several milestones — and five modules are in fact multi-owned (`tests/test_json.py` at 998/1000/1008, `tests/test_errors.py` at 999/1032, `tests/validators/test_nullable.py` at 1013/1018). Worse, 972–973 ("not a permission to ignore other relevant cases in the named module") plus the column title imply `ps_5` owns all of `tests/test_types.py` — 307 tests, 7291 lines, spanning `ps_7` enums/unions, `ps_9` JSON Schema and `ps_10` network types — while nothing directs the manifest to assign those nodes to a later milestone. Since `ps_4` already completes all classification (1291–1292, 1301–1302), restate 970 and the column as *the milestone that owns this module's anchors*, and add: *a non-anchor `same` node's owning milestone is the milestone that implements its feature.* `ps_0`'s gate forbids "sequencing ambiguity" (1230–1231).
- **mn-2 — `ps_11` cites a procedure the document never defines.** 1401–1402: "Re-audit the already-complete manifest against its pinned revisions and **the documented update-pin procedure**". No section defines it and no milestone deliverable creates it; the closest text (1104–1105) only describes detection. Either define the pin-update procedure or make it a `ps_0` deliverable alongside 1222–1224.
- **mn-3 — The `ps_0` gate clause is false by the document's own design.** 1232–1233 requires "every required feature family has pinned selector anchors", but 923–926 states that fixed-width integer schemas deliberately have no Pydantic oracle and are specified natively instead. Add the carve-out to the clause so the gate is checkable.

---

## Pass-4 closure matrix

| Item | Requirement | Status | Evidence |
| --- | --- | --- | --- |
| **B1** | Total upstream file/node/parameter equality; missing module ownership; no `ps_11` catch-up loophole | **CLOSED** | Ledger computed from the pinned Git trees, not the tables (953–965); exact-equality + content-hash rule; named modules 55→73; every pass-4-required row added (`test_build.py`→996, `validators/test_url.py`/`test_uuid.py`→1006/1007, `test_types.py`→1046, `validators/test_model.py`→1012, `serializers/test_timedelta.py`/`test_enum.py`/`test_literal.py`/`test_none.py`/`test_nullable.py`/`test_definitions.py`/`test_url.py`/`test_uuid.py`→1022–1031, `test_networks.py`/`test_root_model.py`/`test_annotated.py`/`test_errors.py`→1056–1059); gate re-quantified over upstream (1232–1233); `ps_11` catch-all replaced by re-audit only (1401–1403) and reinforced at 1458–1460 |
| **M1** | No within-row selector ambiguity; exact enforceable identity convention | **CLOSED** | Convention stated at 983–987; mechanically re-verified: **0 of 335** bare selectors resolve in >1 row module, 0 duplicate `path::selector` owners. All 17 pass-4 collisions split (962→1014/1015, 963→1009/1010, 964→1016/1017, 968→1023/1024, 969→1026/1027) |
| **M2** | Literal/enum/nullable/union rows aligned to milestones and gates | **CLOSED** | `test_none.py`→`ps_5` (1005), `test_nullable.py::test_nullable`→`ps_6` (1013), `test_literal.py`/`test_enums.py`→`ps_7` (1014/1015), `test_union_nullable_bool_int`→`ps_7` (1018); all gates restated as named families |
| **M3** | `ps_4` anchors limited to foundation; `test_build` added; fixture families gated | **PARTIAL** | `test_build.py` added, `test_all_errors`→`ps_10`, scalar/model anchors moved, **51/51 families now appear in an exit gate** — but `ps_4`'s own anchors still require `ps_5`/`ps_6` validators → **MJ-1** |
| **M4(a)** | No runtime-schema-rebuild anchor | **NOT CLOSED** | `test_rebuild_recursive_schema` removed, but `test_generic_recursive_models_parametrized` (1052) is a body of nothing but `model_rebuild()`; also `test_recursive_models_union`, `test_generic_recursive_models`, `test_min_length_field_info_not_lost` → **B-new(a)/(c)** |
| **M4(b)** | Strings input designed without a second engine | **PARTIAL** | Design is correct and explicit (663–672, 1310–1311); `defer_build` carve-out is exemplary (1061–1064) — but ungated, absent from 532–538, input type undefined → **MJ-2** |
| **M4(c)** | Callable discriminators in schema algebra and execution model | **CLOSED** | Sums row 578; verification 598; execution 752–757; indexed-dispatch claim qualified 1153–1154; `ps_7` 1339–1340 |
| **M4(d)** | Config introspection removed | **NOT CLOSED** | `test_config_defaults_match` gone, but replaced by `test_model_field_default_info` (`str(Model.model_fields)` repr) and `test_invalid_config_keys` (`validate_call`, no assertion) in the same row 1049 → **B-new(a)/(c)** |
| **M5** | Status distinguishes pre-manifest from current approval | **CLOSED** | 5–11 and pass-4 artifact link at 18 |
| **m1** | Bare-filename convention explicit | **CLOSED** | 983–984; re-verified against all 77 path instances |
| **m2** | Computed-field descriptor behavior adapted, not ported | **CLOSED** | Adaptation note 1034–1037; `test_property` replaced by `test_computed_field_exclude_none`; **no remaining anchor uses `round_trip=True`** (confirmed across all 20 anchored serializer modules) |

---

## Can `milestone_ps_0` be re-approved?

**No.** Its deliverable is "Approve the pinned module and selector baseline in this document" (1221) and its gate requires no sequencing ambiguity (1230–1231). The completeness machinery is now genuinely sound — B1, M1, M2, M5, m1, m2 and M4(c) are closed, and the ledger plus the exact-equality rule is the right answer. What is not approvable is the baseline's *content*: anchors that assert nothing, one that is `xfail` upstream, and ~60 that depend on `__pydantic_fields_set__`, `__dict__`, `from_attributes`, instance identity, runtime rebuild, `validate_call`, Python `repr`/`get_type_hints`, cyclic data, or serializer warn-and-fallback — every one of which this document rejects by name. By rule 980–981 each is an automatic gate failure, so `ps_4`, `ps_6`, `ps_8` and `ps_10` cannot close as written.

The architecture proper remains untouched and sound: bridge version 2, Core Schema as sole authority, the validated arena and no-third-tree invariant, crate-neutral specialized scalars, the certification prerequisites, and the `sifr-lang/sifr` ↔ `sifr-lang/pydantic-sifr` split (verified consistent across all 11 demo-ownership sites). Confine the rework to: the anchor-portability rule plus re-selection and the three missing semantic decisions (B-new), the `ps_4`/`ps_5`/`ps_6` row retargeting (MJ-1), one `tests/test_validate_strings.py` row with its `ps_5` gate entry and capability/type text (MJ-2), and three wording fixes (mn-1…3). The decisive change is B-new's rule: without it, pass 6 will be reviewing hand-picked names again.
