All audits complete and cross-verified. I confirmed every reported finding directly against the pins.

# VERDICT: **NEEDS REVISION**

**Method.** Re-parsed both manifest tables mechanically: **50 rows, 80 path instances (71 distinct modules), 291 anchor instances, 50 fixture families**. All 80 paths resolve; all 291 selectors resolve in exactly one module of their row; **0 ambiguous bare selectors, 0 duplicate `path::selector` owners**; **50/50 families gated exactly once in their owning milestone** (11 `serializers/*` via the ps_8 wildcard). AST-walked all 291 anchor bodies for `xfail`/`skip` via decorator, aliased-mark indirection, module `pytestmark`, `param(marks=…)`, in-body, and transitive fixture chains including `name=`-renamed and autouse conftest fixtures: **0 flags**. Then read anchor bodies, parametrize cases, and `src/serializers/filter.rs` in full. Both pins confirmed at `f59e929c…` / `383eb95a…`.

**The mechanical layer is now completely clean, and most of pass 7 is genuinely closed at the root.** Two things are not, and both are in newly written text.

---

## BLOCKER

### B-1 — Line 857–858's composition rule is refuted: upstream gives call-time selections precedence over schema-declared ones

> 857–858: "An entry is emitted only when it is **not excluded** and, when an inclusion exists, is included."

In `filter.rs`, a call-time include hit **returns immediately** (`:196-200`, `:206-208`, `:214-216`), and a call-time nested exclude emits at `:227-228` — both bypass `default_filter` (`:248-255`), which is the *only* consumer of the schema-declared sets. Upstream asserts this explicitly and even comments it:

| Upstream | Schema filter | Call-time | Upstream expects | Doc 857–858 yields |
| --- | --- | --- | --- | --- |
| `test_list_tuple.py:176-178` (`# `include` as a call argument trumps schema `exclude``) | `exclude={0,1}` | `include={1,2}` | `[1,2]` | `[2]` ✗ |
| `test_dict.py:137` | `exclude={'0','1'}` | `include={'1','2'}` | `{'1':1,'2':2}` | `{'2':2}` ✗ |
| `test_dict.py:138` | same | `include={'1','2'}, exclude={'2','3'}` | `{'1':1}` | `{}` ✗ |
| `test_dict.py:146` | `exclude={0,1}` | `include={1,2}` | `{1:1,2:2}` | `{2:2}` ✗ |

851–854's "`All` dominates" is wrong for the same reason (schema `exclude={'a'}` + call-time `exclude={'a':{1}}` emits `'a'` upstream, per `:172` then `:227-228`). These are `same`-classified collected nodes in two anchor source modules, which rules 1054–1060 and 1067 require ps_8 to reproduce. This is a regression introduced by the edit that fixed pass-7 B-2: the old rule was wrong about *ordering*, the new one is wrong about *precedence*.

**Correction.** Replace 856–860 with a precedence-ordered rule:

```text
Selections are resolved per node against every original field, mapping key, or
pre-filter sequence index, in this order:
  1. a call-time exclusion that terminally selects the entry removes it;
  2. otherwise, when a call-time inclusion exists, the entry is emitted when that
     inclusion selects it (terminally or with a nested selection), and is otherwise
     removed unless the schema-declared inclusion selects it;
  3. otherwise, a call-time exclusion that selects the entry only with a nested
     selection emits it and forwards that nested selection; and
  4. otherwise the schema-declared filter decides: emitted when its inclusion is
     absent or selects the entry and its exclusion does not.
Call-time selections dominate schema-declared ones at the same node; schema-declared
inclusion and exclusion combine as intersection. Signed indices normalize against
the pre-filter sequence length.
```

I re-derived every anchor assertion against this replacement — all 7 `test_filter_runtime_more` params, both `test_include`/`test_exclude` anchors in `test_list_tuple.py` and `test_dict.py`, all 12 `test_model` and 11 `test_typed_dict` `test_include_exclude_args` params, all 15 `test_advanced_exclude_nested_lists` params, both `test_filter_args_nested` anchors — plus the four rows above. It reproduces all of them.

### B-2 — Four mandatory anchors have no portable behavioral residue

Rules 1073–1077 and 1081–1083 bar a `repr`, a reflection invariant, and an assertion solely about a rejected/not-applicable mechanism. Each verified by direct reading:

| Row | Anchor | Evidence |
| --- | --- | --- |
| **1142** ps_8 `serializers/unions_callbacks_recursion` | `serializers/test_functions.py::test_function_general` (`:24-36`) | The serializer **is** `repr`: `def repr_function(value, _info): return repr(value)` (`:20-21`). All 3 params' expected values are repr output — `(None,'None',b'"None"')`, `(1,'1',b'"1"')`, `([1,2,3],'[1, 2, 3]',b'"[1, 2, 3]"')` — the last carrying Python's comma-space list repr. Barred by 1075; it is the behavior asserted, not harness. |
| **1135** ps_8 `serializers/duration` | `serializers/test_timedelta.py::test_union_timedelta_respects_instanceof_check` (`:361-373`) | Sole assertion `s.to_python('foo') is None`. `'foo'` matches neither arm of `union_schema([timedelta_schema(), …])` — the runtime `isinstance` dispatch the name describes, `not-applicable` per 869–871 and 1151–1153. The `None` comes from the constant `lambda v: None` (`:362`), so **no duration behavior is pinned at all**. *Pass 7 explicitly cleared this anchor; that clearance was wrong.* |
| **1129** ps_7 `validators/unions` | `validators/test_union.py::test_nested_unions_bubble_up_field_count` (`:1194-1296`) | A 103-line body containing exactly two assertions, both `isinstance` (`:1295-1296`). Excluded as subclass identity (1024–1025, 1163–1164) → zero observable residue. Aggravating: the arms `SubModelX`/`SubModelY` are structurally identical, so only Python class identity distinguishes them. |
| **1124** ps_6 `validators/typed_dict` | `validators/test_typed_dict.py::test_field_required_and_default` (`:334-346`) | Sole assertion is a build-time `SchemaError` for `required=True` **plus** a default — unconstructible given first-class required/default metadata (316–317), so 1075–1077 bars it. It is also the *only* anchor distinguishing this row from row 1123, and it asserts nothing runtime in a runtime-validation family. |

**Corrections.** `test_function_general` → `test_function_known_type` (`:199-224`, `[1,2,3]`→`[1,2,3,42]` with a declared `return_schema`). `test_union_timedelta_respects_instanceof_check` → `test_timedelta_key` (`:41-45`, three portable assertions incl. `b'{"P2DT3H4M":1}'`). `test_nested_unions_bubble_up_field_count` → `test_smart_union_does_nested_typed_dict_field_counting` (`:1170-1191`, same behavior asserted on returned values over both choice orderings). `test_field_required_and_default` → reclassify `not-applicable` (or move to `core/schema_contract`/ps_4 as an `adapted` build-time diagnostic); use `test_fields_required_by_default_with_optional` (`:258-269`) if a distinguishing typed-dict anchor is wanted.

---

## MAJOR

**MJ-1 — Overlay clauses 845–846 have no `default`×`default` rule, and a mandatory anchor parameterization needs one.** `serializers/test_model.py::test_advanced_exclude_nested_lists` id `'Merge sub dicts 1'` (`:367-371`): `exclude={'subs': {'__all__': {'subsubs': {'__all__': {'i'}}}, 0: {'subsubs': {'__all__': {'j'}}}}}` → `subs[0].subsubs == [{}, {}]`. Both operands lift to `Elements{default: Fields{…}, indices: {}}` with **empty** branch maps, so none of the four clauses ("missing entries inherit the base; branch maps merge by key; explicit branch replaces base `All`; explicit `All` replaces base branch") applies — `default` is not a branch-map key. A reader who overwrites `default` gets `[{'i':1},{'i':2}]`; upstream recurses on `'__all__'` like any other key (`filter.rs:379`) and gets `[{},{}]`. **Correction:** append to 845 — "a `default` present on both sides overlays recursively, and a `default` present on only one side is inherited". (The other three clauses *are* an exact restatement of `merge_all_value`/`merge_dicts` — I verified each against `filter.rs:336-338, 373-375, 378, 381-390`.)

**MJ-2 — `test_validate_default`'s 36 parameterizations all rest on a declaration Sifr cannot make.** Row **1133**, ps_7. `default='42'` (a `str`) on a field whose inner schema is `int` (`:366-368`). `expected = 84 if (config_validate_default is True and schema_validate_default is not False or schema_validate_default is True) else '42'` (`:376-381`) → **5 of 9 config×schema combos × 4 inner schemas = 20 of 36 params assert `{'x': '42'}`**, i.e. an unvalidated default bypassing the declared field type. The doc never states whether a default may differ in type from its field; 616–617 mentions only "defaults that do not validate under their declared policy". Rule 1084–1086 demands every `same`/`adapted` param be retained or individually justified. Pass 7 verified this anchor's `inner_schema` axis when it retargeted it here, but not the default-typing axis. **Correction:** classify the 20 `'42'` params `not-applicable` and state the retained-default typing rule, or substitute `test_validate_default_factory` (`:384-392`) plus `test_default_value_validate_default_fail` (`:513-527`).

**MJ-3 — `pattern` is a required node with no anchor, no reconstruction type, and no native carve-out.** Line **590** lists `pattern` under Specialized scalars beside `date, time, datetime, duration, UUID, URL` — all of which have their own anchor family — while 591 separately lists `pattern` as a Constraints node (anchored by `test_constrained_str`). The scalar form has: no fixture family; no entry in the `StructuralConstruct` reconstruction list (753–757), so its Sifr-facing type is unstated; and no oracle-less carve-out like 1011–1014 / 1016–1021. A meaningful oracle exists: `pydantic/tests/test_types.py::test_pattern` (`:4240-4269`) asserts match/non-match over `re.Pattern`, `Pattern`, `Pattern[str]`, `Pattern[bytes]` plus `{'type':'string','format':'regex'}` JSON Schema. This leaves the ps_0 gate clause at 1386–1387 unmet — the same defect class as pass-7 MJ-6. **Correction:** pin `test_pattern` into a family (its `__class__.__name__` and `is p` assertions excluded) and add pattern to 753–757; **or** state that 590's `pattern` denotes the compiled-pattern mechanism behind the pattern constraint and that a pattern-valued field is out of scope.

**MJ-4 — Parameter identity is undefined for the ledger, and the one definition given provably collides at the pin.** 1045–1048 lists "parameter identity" as a manifest field; the only definition is 1079's "stable AST-content hash". Two mandatory anchors contain byte-identical parameters: `validators/test_time.py::test_time_json` lines **59 and 60** are character-identical `pytest.param('12:13:14.123456', time(12,13,14,123456), id='str-micro-6dig')` entries; `validators/test_uuid.py::test_uuid_version` line 126 duplicates 128 and 127 duplicates 129. Two collected parameters therefore collapse to one ledger entry, so one can be deleted upstream without failing the audit — directly falsifying acceptance criterion 1613–1615 ("no upstream path, collected selector, **or parameter** can disappear without failing the audit"). The alternative (pytest node ids) is no better: the many `ids=`-less parametrizations use positional `input_valueN-expectedN`, so any insertion renumbers everything and a pure reordering is invisible. **Correction:** define parameter identity as (AST-content hash, occurrence index within its parametrize list) and require the ledger to record multiplicity.

**MJ-5 — No adaptation rule for `Any`-typed upstream containers or heterogeneous mapping keys, and `Entries` is keyed by "declared key type".** `serializers/test_dict.py::test_include` (`:39-41`, row 1137) asserts `s.to_python({'a':1,'b':2,'d':4, 5:6}, include={5}) == {'a':1, 5:6}` over `dict_schema()` — a mapping with **mixed `str`/`int` keys**. Line 832 keys `Entries` by "declared key type"; line 695 states flatly "there is no `Any`"; Non-Goal 1329 rejects dynamic introspection. Nothing says whether union-typed mapping keys exist, so 1078–1080's "records its normalized Sifr expectation" cannot be discharged for these three retained assertions. Eight further serializer anchors construct `any_schema()` elements (`test_list_any`, `test_set_any`, `test_frozenset_any`, both `test_filter_args_nested`, `test_include`/`test_exclude` ×2) and ~15 pydantic anchors carry `Any` annotations. The doc has explicit adaptation rules for descriptors (1146–1149), wrong-type warnings (1151–1153), subclass params (1154), `py_and_json` (1087–1088) and heterogeneous *contexts* (1165–1167) — this class has none. **Correction:** one sentence — upstream `any_schema()`/untyped-container/`Any`-annotated harness normalizes to a concrete typed Sifr element type per retained assertion, and a heterogeneous upstream mapping is normalized to a declared union key type or classified `not-applicable`, with the reason recorded.

---

## MINOR (edit-worthy)

- **839 "current sequence length" vs 859 "pre-filter sequence length".** Upstream normalizes only call-time selections, against the **input** length captured before filtering (`filter.rs:101-102`, `list.rs:68`, `tuple.rs:177`). One live reading of "current" — the length after the schema-declared filter — breaks retained assertions of the mandatory anchor `test_exclude`: `:158`/`:160` (`schema exclude={1,3,5}`, call `exclude={-1,-2}`, `list('abcdefgh')`) give `['a','c','e']` under "pre-filter" but `['a','c','g','h']` under that reading. *Fix:* "this node's pre-filter sequence length".
- **863's spelling enumeration omits the `None`-valued entry**, used by 4 retained anchor assertions (`test_list_tuple.py:108,109`, `test_dict.py:36`). `is_ellipsis_like(None)` is false (`filter.rs:319-325`), so under an inclusion `{k: None}` desugars to `All`; under an exclusion it means absent. *Fix:* add "`None`-valued entry" and note it desugars to `All` under an inclusion.
- **860 states the behavior (inert) but not the typing.** Nine retained anchor assertions require `Elements{indices:{1:All}}` to be *constructible* beneath an `int` leaf, while 872–873 makes incompatible declared types a compile-time diagnostic and 1516 promises "**typed** recursive include/exclude selections". *Fix:* "…is accepted and ignored rather than rejected; shape checking applies only where the declared type has fields, elements, or entries."
- **`test_default_factory` (row 1177, ps_6) has no recordable residue.** `tests/test_main.py:1732-1766`: `m1.uid == m2.uid` on a non-const class-body default `uuid4()`; `isinstance(m1.uid, UUID)` (reflection); `m.uid is uuid4` and the singleton `is` check (identity + `arbitrary_types_allowed`, 1266–1268). The sole survivor is `m1.uid != m2.uid` (`:1746`) — a nondeterministic inequality, which the neutral-fixture format's "expected normalized value or error list" (1209) cannot express. *Fix:* substitute `test_default_factory_called_once_2` (`:1790-1806`, `m1.id == 1` / `m2.id == 2` via a counter factory).
- **`test_computed_field_exclude_none` (row 1140, ps_8) is voided by the doc's own carve-out.** `volume` is declared `computed_field('volume', int_schema())` (`:803`) while the `@property` returns `None` (`:789-791`). Applying 1151–1153 literally deletes every `'volume': None` observation (`:812, :819, :827`), after which `exclude_none=True` and `False` produce identical output and the anchor no longer demonstrates its named behavior. *Fix:* record the adaptation `computed_field('volume', nullable_schema(int_schema()))`; all six assertions then retain under 1160 plus the 1146–1149 `@property` rule.
- **The field-set exclusion at 1163 is scoped to "Pydantic public-API anchors" only.** Seven `validators/model_fields` anchors (row 1123, ps_6) assert a single compound 3-tuple whose third component is field-set state, forbidden by 1029 and 793–795: `test_simple` (`:54-58`), `test_with_default` (`:99-104`), `test_ignore_extra` (`:187-191`), `test_fields_required_by_default` (`:451`), `test_alias` (`:484`), `test_alias_path` (`:580`), `test_alias_error_loc_alias` (`:775-777`). 1078–1080 keys retention at assertion granularity and cannot split a tuple. *Fix:* add a sentence to 1146–1157 retaining tuple element 0 (and element 1 under `extra_behavior='allow'`), with element 2 `not-applicable`.

---

## Cleared (notable positives)

The **entire mechanical layer** is clean — see Method. Beyond that, I verified and cleared: the `Entries` alternative expresses **every** `__all__` parameterization exactly, including the decisive `exclude={'__all__': {0}, '3': {1}}` (`test_dict.py:108`); the record-wide desugar at 842–843 matches its one anchor param (`test_model.py:176`); the three non-`default` overlay clauses are an exact restatement of `merge_all_value`/`merge_dicts`; all 7 `test_filter_runtime_more` params reproduce (pass-7 B-2's cases now correct) and MJ-4's `__contains__` precedence is dispositioned; the inert-scalar-leaf rule yields the right answer for all 9 affected assertions including the load-bearing `test_typed_dict` p10; sets correctly need only `All` (`set_frozenset.rs` holds no filter); `Entries` correctly omits positional normalization and matches on the **validated** key even in JSON mode (`dict.rs:89-92`); `uniqueness`/`typed refinement` are cleanly eliminated at 602–604 **and** set dedup has a real pinned oracle (`test_set_ints_both`/`test_frozenset_ints_both` param `[1,2,3,2,3]→{1,2,3}`); `ErrorDisclosure` is fully defined (896–900); the typed context is specified across all seven required dimensions (490, 496–503, 553–554, 773, 816, 1161–1167, ps_7 1500–1501, ps_8 1519); `test_definitions.py::test_custom_ser` (`:6-13`) is the correct portable sibling and resolves unambiguously; `test_recursion_branch` retains genuine non-duplicate residue (`:313-316` supplies the recursive nullable field *explicitly* as `None`, which `test_branch_nullable` never does); the ps_4 manifest ordering, the prerequisites table, and ps_11-as-re-audit-only are coherent; external repo/demo ownership is intact across all 11 sites; and the status block, all seven artifact links, and the 0-byte unlinked pass-8 placeholder are accurate. `test_type_adapter_dump_json` **is** correctly owned by ps_8 — pass 7's clearance holds, because the neutral-fixture record (1200–1212) is schema+mode+expectation and names no Python entry point, and a Python `TypedDict`'s inability to carry methods is exactly the Python-object-model artifact that forces `TypeAdapter` upstream. Likewise the "stranded `model_dump()`" class across ~9 pre-ps_8 anchors is already discharged by 1160's "declared field values" plus per-assertion milestone ownership (1067, 1078–1080, 1096–1099) — a real improvement over pass 6.

---

## Pass-7 closure matrix

| Item | Requirement | Status | Evidence |
| --- | --- | --- | --- |
| **B-1** | `Selection` gains typed-key `Entries`; record `__all__` desugars | **CLOSED** | 830–833, 840–843. Every `__all__` param maps exactly, incl. the decisive default+override merge. *Residual: heterogeneous key types → **MJ-5*** |
| **B-2** | One-pass, original-index/key inclusion/exclusion | **NOT CLOSED** | 856–860 fixes the ordering error but introduces a precedence error → **B-1 (new)** |
| **B-3** | Three residue-free anchors replaced/removed | **CLOSED**, defect class recurs | `test_default_value`→`test_typed_dict_default` (1125); `test_nested_models` dropped (1180); `test_custom_ser` moved to the portable `test_definitions.py` sibling (1143). *Four new instances → **B-2 (new)*** |
| **MJ-1** | `api/constraints`→ps_6; `test_decimal_precision`→ps_9 | **CLOSED** | Row 1176 = ps_6; ps_6 gate 1490 includes it; ps_5 gate 1469–1473 does not; `test_decimal_precision` at row 1184 |
| **MJ-2** | Typed context: type, ownership, borrow/mutation, ABI, capability, milestones, fixtures | **CLOSED** | All seven dimensions present (see Cleared) |
| **MJ-3** | Inert nested scalar selection | **CLOSED** | 860; verified against all 9 affected assertions. *Typing precision → minor* |
| **MJ-4** | Python spellings enumerated and dispositioned | **CLOSED** | 862–867 adds `list`, dict-view, and the `__contains__` disposition. *`None`-valued entry → minor* |
| **MJ-5** | `invalid_extra` removed | **CLOSED** | Row 1179 |
| **MJ-6** | uniqueness/refinement redundancy eliminated cleanly | **CLOSED** | 591, 602–604; dedup oracle exists. *Same class recurs at `pattern` → **MJ-3 (new)*** |
| **mn** | Fixture-level `xfail` anchor removed | **CLOSED** | Anchor removed from row 1125; 0 flags across 291 anchors and all transitive fixture chains |
| **mn** | Recursion limit = Sifr-native acyclic resource contract | **CLOSED** | 1016–1021; cyclic sibling `test_cyclic_recursion` correctly unselected |
| **mn** | Unrepresentable recursive serializer / invalid-schema / PyO3-unicode anchors removed | **CLOSED** | `test_recursive_function`, `test_err_on_invalid`, `test_unicode_error` all gone |
| **mn** | `ErrorDisclosure` defined | **CLOSED** | 896–900 |
| **mn** | Parameter identities inside exact-set equality | **PARTIAL** | 1056–1060 and 1613–1615 add "parameter", but identity is undefined and the 1079 definition collides → **MJ-4 (new)** |
| **mn** | `adapted` params cannot be silently dropped | **CLOSED** | 1084–1086 restricts omission to `not-applicable`/`rejected` |
| **mn** | Node-classification quantifier | **CLOSED** | 1056–1057 "in any file" |
| **mn** | ps_4 manifest ordering; sequential prerequisites | **CLOSED** | 1043–1048; 1345–1350 each name the prior milestone's released output |
| **mn** | Overlay vs union explicitly distinct | **CLOSED** as a statement | 851–854. *But the union half is factually wrong → **B-1*** |
| **Milestone gates** | Every family gated exactly once | **CLOSED** | 50/50, verified programmatically |
| **No ps_11 catch-up** | Re-audit only | **CLOSED** | 1559–1561, 1616–1618; no family assigned to ps_11 |
| **External ownership** | External repo only | **CLOSED** | All 11 sites consistent |
| **Status/history** | Accurate, artifacts linked | **CLOSED** | Matches every artifact's verdict; 7 links non-empty; pass-8 placeholder correctly unlinked |

---

## Can `milestone_ps_0` be re-approved?

**No.** Its deliverable is "Approve the pinned module and selector baseline" (1374) and its gate (1384–1389) forbids unresolved sequencing ambiguity and requires every required feature family with a meaningful Pydantic oracle to have pinned selector anchors. Four mandatory anchors currently pin no portable behavior (**B-2**), the `pattern` node has an unpinned oracle (**MJ-3**), and the normative selection semantics contains a demonstrably false statement (**B-1**).

The trajectory is strongly positive and the remaining work is narrow. Everything mechanically checkable is now clean, and it is clean for the first time: no unresolved path, no ambiguous selector, no duplicate owner, no `xfail` reachable by any route including the fixture chain that pass 6 missed, and every family gated exactly once in the right milestone. Pass-7's B-1, B-3, all six MJ items, and every minor are closed at the root; the `Entries` alternative and the three non-`default` overlay clauses are a faithful and now near-complete statement of upstream's `__all__` merge.

What remains splits cleanly. **B-1 is the one to fix first** — it is a false statement of semantics, it was introduced by the edit that fixed pass-7 B-2, and it is invisible from the grammar; it surfaces only when you evaluate the rule against `test_filter_runtime` in either module, where upstream's own comment states the opposite. A verified replacement is above. **B-2** and MJ-1 through MJ-3 are row substitutions or one added sentence each, all with named portable siblings identified. **MJ-4** and **MJ-5** are single clauses in the manifest rules. The architecture proper remains untouched and sound: bridge version 2, Core Schema as sole authority, the validated arena and no-third-tree invariant, crate-neutral specialized scalars, the certification prerequisites, and the `sifr-lang/sifr` ↔ `sifr-lang/pydantic-sifr` split.

*No files were modified. `plans/reviews/active/native-pydantic-sifr-architecture-agent-review-pass-8.md` is still a 0-byte placeholder — say the word and I'll write this review there.*
