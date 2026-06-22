I have enough context. Here is the review.

## Review of `plans/issues/ad-hoc-lsp-semantic-hover-signature-plan.md`

### 1. Root cause: partially correct, missing the layer underneath

You've correctly identified that hover/signature‑help render lexer metadata. But the deeper defect the plan glosses over is that `sifr_analysis` has **no name‑resolution surface at all** today — all "semantic" queries (`definition`, `declaration`, `references`, `rename`, `document_highlights`) currently work by string‑matching token text across every file. Hover/signature is just the most visible symptom. If the plan only renames the symptom (token‑kind → semantic text) without surfacing real position‑to‑symbol resolution, those other queries stay defective and "split brain" will reappear the moment two locals share a name.

Architectural observation that should drive the plan:

- `crates/sifr_ir/src/hir_nodes.rs` `HirFunction`/`HirParam`/`HirStmt`/`HirExpr` carry **no `TextRange`** today. Spans live only on diagnostic types and `RustInteropDeclaration`. There is no existing offset→HIR map.
- `sifr_frontend::SymbolView` is `{ name, kind }` only — no params, no types, no spans.
- `crates/sifr_analysis/src/host/implementation.rs:681` `editor_facts` is the only "lookup at position" primitive and it's pure parser tokens.

So "build callable signatures from `FunctionType`/HIR data already owned by analysis" is misleading — analysis doesn't own HIR (it's explicitly forbidden by `verification/areas/developer_tooling/check_analysis_split_brain.py:23` which bans `HirModule` references). The plan must say: **`sifr_frontend` exposes new typed views (callable signatures, binding types, position→binding map); `sifr_analysis` only consumes them.**

### 2. Architecture risk: plan wording invites a split‑brain re‑introduction

The plan says "Add semantic editor query data in `sifr_analysis`" and "Build callable signatures from Sifr `FunctionType`/HIR data already owned by analysis". As written, an implementer would reach for `HirModule` inside `sifr_analysis` and trip the existing guard, then might "fix it" by importing `sifr_type_system::format_type` or by re‑deriving signatures from `analysis_for_project()` blobs — both options would still leave analysis as the semantic authority, exactly what the principle forbids.

Recommended rewording:
- New frontend queries (e.g. `callable_signature_for(module, name)`, `binding_at_position(file, offset)`, `type_at_position(file, offset)`) returning fully‑rendered, span‑annotated views.
- Analysis is a passthrough that maps `TextPosition` → frontend offset, calls the new queries, and shapes them into `HoverInfo`/`SignatureHelp`.
- Update `check_analysis_split_brain.py` to also forbid `FunctionType`, `sifr_type_system::format_type`, and direct stdlib `external_defs` reads inside `sifr_analysis`.

### 3. Missing verification cases

The corpus covers function/local hover. Add (or explicitly defer with reason):
- Method call hover/signature (`obj.do_thing(|)`).
- Class instantiation (`Foo(|)`).
- Imported‑name call (`from utils import helper; helper(|)`).
- Keyword argument signature help (`randint(stop=10, |)`) — `HirParam::keyword_only` matters.
- Generic function (`def f[T](x: T) -> T:`) — verifies type‑param rendering.
- `Result`/`Option` return types — the Sifr‑specific shape that motivates this work.
- Shadowed binding: hover on the later `x` must show its type, not the first.
- Closure capture and parameter inside nested `def`.
- Multi‑line call cursor (`f(\n  1,\n  |\n)`) for `activeParameter`.
- UTF‑16 position negotiation: the existing smoke test for hover only checks "contains `helper`"; the new semantic corpus should include at least one non‑ASCII case so the encoding path isn't bypassed.
- Stale‑then‑refresh: assert that after a stale notification the hover returns `None` or an `AnalysisError::StaleSnapshot`, then post‑refresh returns the new symbol — same for `activeParameter` after arguments are inserted.
- Negative case: hover over a keyword, whitespace, comment, or string literal returns `null`, not an empty markdown block.

### 4. Related defects the plan should at least name

- `prepare_type_hierarchy` (`implementation.rs:389-404`) uses "first char is uppercase → type" — the same lexer‑shaped heuristic the plan deprecates. Either bring it into scope or list it as known dead‑weight to remove next.
- `inlay_hints` (`editor.rs:116-152`) reads `:` suffixes out of raw source and emits parameter annotations — these will diverge from the new semantic hover. Add at least one corpus assertion that inlay text equals the semantic type or remove inlay heuristic in the same pass.
- `definition` / `declaration` / `type_definition` are all `locations_for_identifier_at` with `first_only = true` — three semantically distinct queries collapsed into a token‑name match. Same root cause, worth listing.
- `signature_help` always returns `active_parameter: Some(0)` and the conversion (`conversion.rs:210-214`) hard‑codes `"parameters": []`. The plan calls out the label fix but the JSON shape needs to start emitting real `parameters` entries with offset pairs.

### 5. Over‑specified tests to soften

- `def generate_random() -> int | None` as a literal hover assertion is brittle against any change to type formatting (e.g. `Optional[int]`, ownership prefixes like `own`/`mut`, default rendering). Bind the assertion to a single canonical `format_type` from `sifr_type_system` and assert structural fragments (name, `->`, the canonical return‑type string from the same renderer) rather than a frozen literal.
- "Hover renders semantic contents as a `sifr` markdown code block" pins `HoverInfo { contents: String }`. That shape can't carry docstrings or multiple sections. Make `HoverInfo` carry `{ code: String, documentation: Option<String> }` (or a `Vec<Section>`) up front; the conversion test should assert at least one fenced code block plus an optional documentation section, not exact wrap text.
- Signature corpus that asserts "active parameter follows cursor before and after comma" is fine — but add the trailing‑whitespace and multi‑line cases above so a token‑based shortcut can't satisfy the test.

### 6. Static‑guard precision

Extending `check_analysis_split_brain.py` is the right hook, but:
- Scope the new forbidden patterns to production paths only (the script today checks every `.rs`; tests need to seed). Add an exclude on `#[cfg(test)]` blocks or scope by directory.
- Forbidden patterns should target the failure mode, not the type names — e.g. forbid the format string fragment `({})` paired with a `kind` arg inside hover/signature constructors, and forbid `format!("{}(...)")`‑style heuristic signature labels. Substring `token.kind` is too narrow; the current bug formats `token.kind` via `EditorToken.kind: String`, not the field name.
- Keep the seed‑and‑prove self‑test pattern from the existing guard.

### 7. Concrete edits to the plan before implementation

1. Rewrite "Implementation Shape" so the new semantic surface lives in `sifr_frontend` (`CallableSignatureView`, `BindingView`, `PositionResolution`), with `sifr_analysis` reduced to mapping + conversion. Explicitly note `sifr_ir::HirFunction`/`HirParam` need source spans threaded through lowering — that's the real prerequisite and likely the longest sub‑task.
2. Add a "Related cleanup" section listing `prepare_type_hierarchy`, `inlay_hints`, and the `definition/declaration/type_definition` collapse so reviewers know they're out of scope but tracked.
3. Add the missing verification cases above; pick one canonical type renderer and assert against it; switch literal expected strings to structural assertions.
4. Specify `HoverInfo` and `SignatureHelp` shape changes (multi‑section hover, real `parameters` with offset pairs and a derived `activeParameter`) so the conversion layer isn't pinned to today's flat string.
5. Tighten the static guard: production scope, pattern‑level checks (format strings + heuristic labels), and add `FunctionType`/`sifr_type_system::format_type` to the analysis‑forbidden list once the frontend exposes a wrapper.
6. Anchor the "scribbles" acceptance fixture to a real file under `verification/areas/developer_tooling/fixtures/` and name the new corpus file (e.g. `lsp_semantic_corpus.py`) so the e2e discovery harness picks it up.

Net: the plan correctly removes the placeholder, but unless it elevates the semantic surface into `sifr_frontend` first and rewrites the position‑to‑symbol primitive, the same defect will resurface in `definition`, `references`, and `rename`.
