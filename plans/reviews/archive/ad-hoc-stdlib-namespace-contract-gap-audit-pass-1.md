# CHANGES_REQUESTED

The previous READY was premature. The plan has a load‑bearing API impossibility and several unmade cross‑layer decisions. Below is what must be locked in before this becomes implementable.

---

## 1. Machine-readable diagnostic args — currently impossible from lowering

The plan says SIFR-IMPORT-0008 "must include machine-readable diagnostic data: `bare_module`, `suggested_module`, and `imported_names`". The lowering pipeline cannot do this today:

- `HirDiagnostic` (`crates/sifr_ir/src/diagnostic_types.rs:6`) is `{ code, message, primary_range, line, col }` — **no args/data field**.
- `LowerCtx::error_with_code_at` (`crates/sifr_lowering/src/lower/mod_context.rs:180`) takes only `(code: DiagnosticCode, message: String, range: TextRange)`. There is no template/args plumbing.
- Driver-level callers (`discovery.rs:268-337`, `package_discovery.rs:152-275`) emit through `DiagnosticBuilder::source(...).arg(name, DiagnosticArg::String(...))` because they construct rendered diagnostics directly. Lowering does not.

Lock one of these in the plan, with the API change spelled out:

- **(A) Extend the HIR diagnostic transport** — add `args: Vec<(String, DiagnosticArg)>` (or equivalent) to `HirDiagnostic` and a sibling `error_with_code_args_at` (or expand `error_with_code_at`) on `LowerCtx`. Then thread args through `hir_diagnostic_to_rendered` in `query_diagnostics.rs:117-128`+ to land on the rendered diagnostic. List the touched files; this is not a trivial extension and currently has no decision in the plan.
- **(B) Emit SIFR-IMPORT-0008 only from the driver layer** — keep lowering's diagnostic message-only (or drop it entirely) and centralize structured emission in `discovery.rs` / `package_discovery.rs` where `DiagnosticBuilder` is already in use. This implies single-file lowering can't carry the structured args; you must decide whether that's acceptable.
- **(C) Drop the machine-readable args requirement** and update the contract harness expectations.

The current plan paper-shuffles past this. As written, M1 is unimplementable.

---

## 2. Layer ownership of SIFR-IMPORT-0008 is undecided

The plan says "top-level user/package resolution comes before bare-stdlib diagnostics" but never says **which layer** emits the diagnostic in each mode. The real layer matrix today:

| Mode | `from math import sqrt` | `import math` |
|---|---|---|
| Single-file | `mod_impl.rs:468` → `unknown_import_target` (`IMPORT_UNKNOWN_SOURCE_MODULE`) | `mod_impl.rs:611-620` → `unsupported_form` (`IMPORT_UNSUPPORTED_FORM`) |
| Project | `discovery.rs:268-285` → `IMPORT_UNKNOWN_SOURCE_MODULE` with `tried_paths` (before lowering ever runs) | discovery silently ignores `Stmt::Import`; lowering then emits `IMPORT_UNSUPPORTED_FORM` |
| Package | `package_discovery.rs:152-207` → `IMPORT_UNKNOWN_SOURCE_MODULE` | silently dropped (package code only walks `Stmt::ImportFrom`) |

Pick the ownership rule explicitly. Recommended:

- Project mode: discovery emits SIFR-IMPORT-0008 **after** workspace resolution returns `Unresolved` AND the unresolved module name matches a bare stdlib tail. Mention `discovery.rs:159-164` (Unresolved arm) as the patch site and `discovery.rs:268-285` (`to_source_diagnostic` unresolved arm) as where the args/template change.
- Package mode: same as project — `package_discovery.rs:152-207` checks tail set before falling through to generic `IMPORT_UNKNOWN_SOURCE_MODULE`.
- Single-file: lowering owns it (`mod_impl.rs:460-468` for `Stmt::ImportFrom`, `mod_impl.rs:611-620` for `Stmt::Import`). Specify whether structured args land here (depends on decision in §1).

Without naming the patch sites, M1 has no scope.

---

## 3. `Stmt::Import` is invisible to four out of five collectors

You correctly flagged five collectors today, but the plan never says how to fix the `Stmt::Import` asymmetry:

- `crates/sifr_driver/src/project/discovery.rs:528-558` — `collect_import_closure_module_dependencies` destructures `Stmt::ImportFrom` only.
- `crates/sifr_driver/src/project/compile_order.rs:21-83` — same.
- `crates/sifr_driver/src/project/package_discovery.rs:432-505` — same.
- `crates/sifr_frontend/src/query_diagnostics.rs:48-78` — same.
- `crates/sifr_frontend/src/module_signatures.rs:91-120` — same.

So in project/package mode, `import math` is invisible to discovery, **never gets a tried_paths diagnostic, and currently falls through to lowering's `IMPORT_UNSUPPORTED_FORM`**. The plan's stated intent ("SIFR-IMPORT-0008 covers both `Stmt::Import` and `Stmt::ImportFrom`") cannot be honored in project mode unless either:

- discovery starts walking `Stmt::Import` (at least for the bare-stdlib detection — not for dependency edges, since `import math` doesn't import a module-object today), OR
- lowering's `Stmt::Import` branch at `mod_impl.rs:611-620` is explicitly rewritten to detect bare stdlib tails and emit SIFR-IMPORT-0008 instead of (or in addition to) `IMPORT_UNSUPPORTED_FORM`, **and** discovery is documented as intentionally silent for `Stmt::Import`.

Pick one and name the file. The plan does not currently mention `mod_impl.rs` at all even though that's where the existing unsupported-form text lives.

For the frontend collectors (`query_diagnostics.rs`, `module_signatures.rs`), state explicitly that no change is needed because they don't emit user-facing diagnostics and bare-stdlib imports legitimately do not contribute to local dependency edges or signature hashes. Otherwise reviewers will keep flagging them.

---

## 4. Duplicate-diagnostic prevention rule is missing

In project mode, `from math import sqrt` hits both:

1. `discovery.rs` (resolver fails → emits some import diagnostic), and
2. lowering's `resolve_imports_early` + `mod_impl.rs` ImportFrom path (which today emits `IMPORT_UNKNOWN_SOURCE_MODULE` again via `external_module_exists` returning false).

If both layers add bare-stdlib detection, both will emit SIFR-IMPORT-0008 for the same statement. Lock down a single-emitter rule, e.g.:

> In project/package mode, discovery emits SIFR-IMPORT-0008 and short-circuits closure traversal for that module. Lowering's `resolve_imports_early` and `mod_impl.rs` ImportFrom path treat a known bare stdlib tail as a recognized import that has already been diagnosed at the discovery layer — they skip the `external_module_exists` check, do not register any bindings, and do not push to `imports`, leaving subsequent name resolution to fail at the use site under the existing missing-name diagnostics.

Either that, or the inverse (lowering owns it; discovery filters bare-stdlib tails the same way it filters `sifr.*` / `_sifr.*` and never tries to resolve them). The plan needs the rule written down.

---

## 5. Probe-then-diagnose order vs. tail short-circuit

You explicitly asked about this. The phase doc already says "User or package modules named `math`, `json`, or similar therefore keep priority once they are real import targets" — that resolution-then-diagnose order is correct and should stay. Re-state it as a concrete rule for `discovery.rs`:

> `ResolutionFailureKind::Unresolved` for `module_name` shall be reclassified into `BareStdlib` (a new `ResolutionFailureKind` variant) if `module_name`, or its leading dotted root, is a bare stdlib tail and no workspace candidate file matched. The reclassification happens **after** all workspace candidates have been probed, preserving today's behavior when a real `math.sifr` exists. The reclassification site is `resolve_with_provider` at `discovery.rs:159-164`, and `to_source_diagnostic` gains a corresponding arm emitting `IMPORT_BARE_STDLIB` with `bare_module`, `suggested_module`, `imported_names`.

Package mode needs the symmetrical reclassification site in `package_discovery.rs`.

---

## 6. Tail-set construction and matching semantics

The phase doc gestures at "stripping the leading `sifr.` from each embedded stdlib module name" and "dotted tails such as `collections.abc` are matched as full tails and by their root". This is not implementable as written. Lock down:

- The tail set is derived from `sifr_stdlib::STDLIB_SOURCES` at compile time. Add a helper, e.g. `sifr_stdlib::bare_stdlib_tails() -> &'static BTreeSet<&'static str>` (or a `is_bare_stdlib_tail(&str) -> bool` predicate), so discovery and lowering share the same source of truth.
- Matching is **exact full-tail first**, then **leading-root fallback**. Concretely, for input `m`:
  - if `m` ∈ tail_set → match `m`, suggest `sifr.m`;
  - else if any prefix `r` of `m.split('.')` with `r` ∈ tail_set → match `r`, suggest `sifr.r`;
  - else not a bare stdlib import.
- For `from collections.abc import Iterable`, the `module_name` is `collections.abc`. State explicitly which behavior the plan wants: (a) if `collections.abc` is in `STDLIB_SOURCES`, suggest `from sifr.collections.abc import Iterable`; (b) otherwise fall back to root match and suggest `from sifr.collections.abc import Iterable` with `bare_module = "collections.abc"`, `suggested_module = "sifr.collections.abc"` — **do not** rewrite to `from sifr.collections import abc`, which would be wrong. Today `STDLIB_SOURCES` likely does not have `sifr.collections.abc`, so name the current behavior explicitly so M1 tests can assert it.
- For `import collections.abc`, the suggestion text in the phase doc says "suggest the corresponding `sifr.collections.abc` path only if that embedded stdlib module exists; otherwise report `collections` with `sifr.collections`". Reconcile this with the `from collections.abc import Iterable` form so the two forms agree.

---

## 7. The plan's framing of today's behavior is wrong in places

The phase doc says bare stdlib imports "either silently no-op and fail later at the use site or hit generic unsupported-import diagnostics". In reality:

- Single-file `from math import sqrt` does **not** silently no-op — `mod_impl.rs:468` emits `IMPORT_UNKNOWN_SOURCE_MODULE` "unknown import target: 'math'".
- Single-file `import math` emits `IMPORT_UNSUPPORTED_FORM`, not "unknown".
- Project `from math import sqrt` emits `IMPORT_UNKNOWN_SOURCE_MODULE` with a workspace-path tried list (not at the use site, and not unsupported-form).

Fix the framing so subsequent reviewers don't have to re-derive it. This also affects M1's test matrix — you need an explicit "before/after" pair for each of (single-file × project × package) × (`Stmt::Import` × `Stmt::ImportFrom`), with the existing diagnostic code (`IMPORT_UNKNOWN_SOURCE_MODULE` / `IMPORT_UNSUPPORTED_FORM`) named in the "before" column.

---

## 8. M1 test scope under-specifies the layer coverage

M1 lists test cases at the diagnostic level but not at the layer level. Required additions:

- Project-mode test: place a `main.sifr` with `from math import sqrt` under a workspace that does **not** contain `math.sifr` → expect `SIFR-IMPORT-0008`, not `IMPORT_UNKNOWN_SOURCE_MODULE`. Pair with a positive test that places `math.sifr` in a source root and asserts that `from math import sqrt` resolves to the user file (proves the resolution-priority decision).
- Package-mode test: same shape, under `package_discovery.rs`.
- Single-file test: covers both `Stmt::Import` and `Stmt::ImportFrom`.
- A `cargo run -- check` CLI fixture under the verification harness producing the human/json/compact tri-output, since the existing diagnostic-contract harness (`crates/sifr_driver/src/bin/diagnostic_rendering_harness.rs:34-60`) enforces the args set. New args (`bare_module`, `suggested_module`, `imported_names`) need to be registered there explicitly.

M1 must also state whether `SIFR-IMPORT-0008` shares args with `IMPORT_UNKNOWN_SOURCE_MODULE` or defines its own arg set — the harness contract is structural.

---

## 9. M1/M2/M3 sequencing has a hidden cross-layer dependency

M1 is "policy and diagnostics" only, but if the diagnostic is partly owned by `discovery.rs`/`package_discovery.rs`, M1 must also patch:

- `collect_import_closure_module_dependencies` (or a sibling pass) to detect the bare-stdlib tail.
- A new `ResolutionFailureKind::BareStdlib` variant and its `to_source_diagnostic` arm.
- `package_import_source_diagnostic` reclassification.
- Frontend layer documentation that explicitly says "no change needed here".

If you keep M1 strictly lowering-side and defer the discovery/package patches, then in project mode the new diagnostic literally cannot fire for `from math import sqrt`, since discovery will short-circuit with `IMPORT_UNKNOWN_SOURCE_MODULE` before lowering runs. Decide:

- **(i)** Expand M1 to include the discovery+package patches, OR
- **(ii)** Split off a new milestone M1.5 (or fold into M1) that explicitly does the discovery+package work; keep M2 (synthetic-import removal) and M3 (defaultdict) where they are; renumber M4 → M5.

Option (ii) keeps M1 focused but the phase doc currently does neither — it leaves the discovery layer unaddressed.

---

## 10. Smaller things still worth pinning down

- M2 says "Remove the consuming site in `crates/sifr_lowering/src/lower/mod_impl.rs` that extends final `imports` with `ctx.synthetic_imports`". Reference the exact line (`mod_impl.rs:647`) and note that `imports::report_unknown_stdlib_module` (line 460) is the related path that emits the deferred-compat module diagnostic — confirm whether `deferred_compat_module` paths are retained or also rewritten under the bare-stdlib contract for `sifr.*` deferred modules (e.g., `sifr.selectors`, `sifr.contextvars`).
- M3's "record explicit imported special constructors in lowering state" needs to name the state field (`ctx.defaultdict_bindings: HashMap<String, ()>` or similar) and the lookup point in `lower_call` / `call_builtins.rs`. The call site is in `call_builtins.rs` based on the phase doc, but the field is unspecified.
- The plan should clarify whether `from sifr.collections import defaultdict, deque` (mixed factory + non-factory) is allowed in M3 or kept as today's behavior. Currently `deque(...)` works through synthetic-import compat (which M2 removes), so the M2 → M3 transition needs an explicit "after M2 lands, `deque(...)` requires `from sifr.collections import deque`; this is exercised by the explicit-import demos updated in M2".
- The M4 guardrail grep `rg "__compat_sifr_(math|heapq|collections)_"` should also cover `crates/sifr_type_system/src` since type-rendering code might still mention old aliases.
- Decision 7 says the `__compat_defaultdict_*` → `__sifr_defaultdict_*` rename happens in M3, but the M4 grep `rg "__compat_defaultdict_"` will only pass if M3 finishes the rename across `sifr_codegen` type rendering too. State the file set: lowering, type rendering, codegen, snapshot fixtures.

---

## Required edits to mark the phase READY

In `issues/ad-hoc-stdlib-namespace-contract-and-compat-cleanup.md`:

1. **Diagnostics Contract** section: add an explicit "Layer ownership" subsection naming the patch sites for single-file / project / package modes and the duplicate-emitter rule.
2. **Diagnostics Contract** section: replace "include machine-readable diagnostic data" with the chosen option from §1 above, including any required `HirDiagnostic` / `LowerCtx` API extension (with the touched files listed) or the explicit "structured args only emitted from the driver layer" carveout.
3. **Namespace Policy** section: replace the prose tail-set description with the exact-match-then-leading-root rule and reference the new `sifr_stdlib` helper.
4. **Context** section: correct the "silently no-op or generic unsupported-import" framing using the today-state table above.
5. **Milestones / M1**: expand to include the discovery+package work (or add a dedicated milestone) with file-level patch sites; expand the test list to cover all three modes for both `Stmt::Import` and `Stmt::ImportFrom`; add the diagnostic-contract-harness arg registration.
6. **Milestones / M3**: name the lowering state field and the call lookup site for explicit defaultdict bindings; declare the M2 → M3 transition state for `deque`/`Counter` post-synthetic-removal.
7. **Exit Gates** / **M4 guardrails**: extend grep scope to `sifr_type_system` and confirm the renamed-name guardrail.

After those edits I expect the phase to be implementable without further cross-layer surprises. Until they're in, this is **CHANGES_REQUESTED**.
