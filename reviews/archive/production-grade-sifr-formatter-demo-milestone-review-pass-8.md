## Review: Formatter Phase — Post-Showcase-Demo Assessment

### Criterion 1: Showcase demo concrete enough and correctly excluded from `.sifr` discovery

**Concrete enough: YES.** The `demos/formatter_showcase/main.sifr.input` file contains:
- All Sifr parameter conventions (`mut own scores:list[int]`, `bonus:int`) — unformatted spacing
- An embedded ` ```sifr ``` ` docstring code block for docstring-code-format coverage
- Multiple unformatted control flow/assignment lines (`if len(scores)>3:`, `scores[0]=scores[0]+bonus`)
- A real-world function structure (`normalize_scores`) that makes before/after visible

**Correctly excluded from `.sifr` discovery: YES.** I traced all three `.sifr` discovery paths:

| Discovery mechanism | Pattern | Catches `.sifr.input`? |
|---|---|---|
| E2E harness `discover_fixtures` | exact `ext == "sifr"` | **No** |
| Generated-code-quality `demos` mode | hardcoded manifest paths + explicit sets | **No** |
| E2E `tests/e2e/pass` fixture runner | `main.sifr`-style names via manifest | **No** |

The `.sifr.input` extension is not equal to `"sifr"` and is not referenced in any manifest or explicit set. The file is invisible to all discovery mechanisms.

### Criterion 2: Milestones 6 and 7 require using the demo

**Milestone 6 (execution.md line ~M6 scope):** Correctly requires adding `demos/formatter_showcase/main.sifr.input` to the formatter corpus with explicit "without treating it as a normal `.sifr` demo fixture" language.

**Milestone 7 (execution.md line ~M7 scope):** Correctly requires "copying `demos/formatter_showcase/main.sifr.input` to a temporary `.sifr` file, running `sifr fmt`, checking the formatted result, and recording the before/after output or diff in the execution tracker."

**Checklist items:** Items M6 and M7 in the execution tracker checklist map directly to these milestone scopes.

All three milestone/checklist references use the exact same path (`demos/formatter_showcase/main.sifr.input`), which matches the actual file on disk.

### Criterion 3: Demo validation/evidence requirement clear for implementation agents

**Yes.** The M7 scope language is unambiguous:
- "copying ... to a **temporary `.sifr` file**" — agent creates a temp `.sifr` copy
- "running **`sifr fmt`**" — invokes the production formatter
- "checking the formatted result" — validates correctness
- "recording the **before/after output or diff** in the execution tracker" — evidence artifact

No ambiguity about what "recorded" means — it requires a before/after diff or output pair in the execution tracker, not merely a pass/fail.

### Criterion 4: Any gaps or better requirements needed before implementation

**No blockers. No gaps.**

The plan is fully self-consistent:
- Showcase demo path is correct and verified on disk
- Milestone references use the same verified path
- Evidence requirement is actionable (copy → format → diff → record)
- Exclusion from `.sifr` discovery is structurally sound

---

**The demo/milestone plan is implementation-ready with no gaps.**
