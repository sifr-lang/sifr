---
name: sifr-demo-authoring
description: Create or refactor Sifr demos under demos/ when the task is to add a runnable demo, rename a demo, simplify an existing demo, preserve demo history, or keep demos feature-first instead of process-, planning-, or problem-solution driven. Use for demo naming, history headers, compact assert-driven examples, and moving non-demo artifacts out of demos/.
---

# Sifr Demo Authoring

Create demos that teach one visible language or compiler feature clearly and fail loudly with assertions when that feature regresses.

## Structure

- Keep runnable demos under `demos/<name>/main.sifr`.
- Keep the top-level `demos/` tree flat.
- Only allow nesting for explicit `negative_cases/`.
- If a demo needs helper files, keep them directly under `demos/<name>/`.
- Do not use `demo` in demo directory or file names.

## History

Preserve history in the file header, not in the path:

```text
# Reference: <historical tag>
# Source issue: <issue-file>.md
```

## Naming

- Name demos by the feature they show, not by delivery process, planning state, versioning, or issue workflow.
- Use short noun-phrase names that answer: "what feature does this show?"
- If traceability labels matter, keep them only in `# Reference:` or `# Source issue:`.

## Content

- One demo should teach one core feature, or one tight cluster of directly related features.
- Use `assert` as the primary signal. `print` is optional and secondary.
- Keep the demo compact. Prefer one small helper function; only add a second when it shows the same feature from another direct angle.
- Use short, neutral names for functions, variables, and fixtures.
- Prefer direct examples over clever ones: direct construction, direct mutation, direct indexing, direct recursion, direct stdlib calls.
- Use tiny fixtures unless the feature truly needs more.

Use this browsing test:

- A user should be able to tell what feature the demo shows from the path and first few lines.
- If the better summary is "this solves X", the demo is still wrong.

## Avoid

- Full problem solutions or interview-style examples.
- Demos that primarily teach an algorithm or trick instead of the Sifr feature.
- Large showcase files that bundle unrelated surfaces.
- Repeated helpers that demonstrate the same idea in multiple ways.
- Reporting artifacts, manifests, scorecards, backlogs, or summaries inside `demos/`.

## Refactoring Rule

When cleaning up an existing demo:

1. Identify the actual feature being demonstrated.
2. Remove process, phase, milestone, wave, or problem framing from names and comments.
3. Replace large fixtures with small assert-driven inputs.
4. Keep the history via `# Reference:` and `# Source issue:`.
5. Move non-demo artifacts to `verification/` or `internal_docs/`.
