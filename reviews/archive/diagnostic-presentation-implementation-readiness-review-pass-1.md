

Let me write the final review directly since I've gathered sufficient evidence.

---

## Diagnostic Presentation Phase Implementation-Readiness Review

**SATISFIED**

### Rationale

Cross-referencing the phase docs against the actual code reveals the plan accurately describes implementation state, correctly identifies required work, and correctly identifies what does NOT need to change. There are no remaining implementation-time product decision holes. All required contracts are explicit and backed by concrete code.

### Precision Edits Required

No blockers, but four precision edits are needed before implementation:

**1. EDIT: Phase plan — M1 renderer contract lock (multiline fixtures)**

The plan says "Add fixture coverage for multiline diagnostics in all three modes." The existing fixture (`decimal_invalid_literal`) is single-line only. M1 must either create a new multiline fixture or annotate which existing fixture to update.

Line count in plan: The plan documents fixtures exist but does not call out the single-line gap.

**2. EDIT: Execution tracker — W-9 wording (command coverage tests)**

W-9 says "M1/M2/M3 add format-selection coverage for `check`, `build`, `run`, and `emit` diagnostics." All four commands already correctly pass `diagnostic_format` through the call chain. The actual scope is "add regression/contract tests proving format selection is not regressed," not "add format selection."

**3. EDIT: Phase plan — compact grouping key (stability note)**

M3 states "Replace grouped verbose compact output with stable summary plus one-line diagnostics." The plan's target shape key is `(severity abbreviation, code, location or<unknown>, message)`, but the canonical renderer in `presentation.rs` groups by `(severity_rank, code, message_template, primary_display_file)`. This produces different grouping than the target. Either:
- (a) M3 must document that grouping adopts the canonical key, or
- (b) update the target shape to reflect `(severity abbreviation, code, message template, file or<unknown>)`.

Current code (`presentation.rs:146-161`): `CompactKey { severity_rank, code, message_template, primary_display_file }` — does not include rendered message or `<unknown>` location.

Target shape in plan: "severity abbreviation, code, location or `<unknown>`, message" — includes rendered message and explicit location.

**4. EDIT: Phase plan — related spans clarification**

The plan says "Show related spans when present, without hiding the primary source location." The canonical `presentation.rs` human renderer does not currently render related spans (only primary spans and children). The plan correctly identifies this as a gap to be implemented. No change needed in the plan, but M2 must confirm: is rendering related spans in human mode also in scope, or is only primary span rendering in scope for M2?

### Blockers: None

All prior-review blockers are resolved:
1. `sifr_diagnostics` ownership is confirmed in both plan and code (`presentation.rs` routes all three modes)
2. Highlight rendering is correctly attributed to M2 as implementation work
3. JSON schema stability is protected via `deny_unknown_fields` and `#[schemars(required)]` annotations
4. `<WORKSPACE>` normalization is documented and operative in baselines
5. CRLF handling is documented: sources retain `\r` in serialized text, human renderers strip it

### Key Findings

**Renderer boundary** (`presentation.rs`): Confirmed correct. `render_human_envelope`, `render_compact_envelope`, `render_json_envelope` all operate on `RenderedDiagnostic` values. CLI routes through `render_diagnostics` which applies recovery limits then delegates.

**Missing highlight renderer**: Confirmed gap. `presentation.rs` prints line text but no caret-style highlights. `DiagnosticSpanLine.highlight_start` and `highlight_end` exist in schema but are not consumed by `render_human_envelope`.

**Suggested output**: Confirmed gap. `presentation.rs` human mode does not render suggestions. Schema stores suggestions; M2 likely needs to add suggestion rendering.

**CRLF handling**: Correctly documented — sources carry `\r` in serialized text, `render_line` trims only trailing `\n`, so `\r` persists to JSON. Human output is responsible for normalizing.

**Compact help-count change**: Correctly documented in W-7. Current `presentation.rs:164-176` counts only severity, but CLI `cli_model_and_entrypoint.rs:853-871` counts help items. Phase changes CLI to drop help counts.

**Format routing**: All four commands (`check`, `build`, `run`, `emit`) pass `diagnostic_format` through the call chain correctly.

### M1 Contract-Lock Prerequisites Before M2/M3

M1 must produce before M2/M3:
1. A multiline `RenderedDiagnostic` fixture locked for human/compact/json
2. A JSON schema-lock test enumerating required fields
3. A test proving `presentation.rs` canonical renderers do not drop span data4. Documentation of compact grouping key (current code vs. target, choose one)
5. A test proving CLI `check`, `build`, `run`, `emit` format selection worksThese are contract items that do not require M2 implementation work.
