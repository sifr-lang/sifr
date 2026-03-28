---
name: sifr-demo-authoring
description: Create or refactor Sifr demos under demos/ when the task is to add a new demo, rename a demo, simplify an existing demo, preserve demo history, or keep demos feature-first instead of milestone/planning/problem-solution driven. Use for runnable demo authoring, demo naming, history headers, and moving non-demo reporting artifacts out of demos/.
---

# Sifr Demo Authoring

Create demos that teach one language/compiler feature clearly and fail loudly with assertions when that feature regresses.

## Core Rules

- Keep runnable demos under `demos/<name>/main.sifr`.
- Keep the top-level demos tree flat. Do not create nested demo folders except `negative_cases/` when a demo needs explicit negative coverage.
- If a demo needs helper files, keep them directly under `demos/<name>/`.
- Do not use `demo` in demo directory or file names.
- Keep historical phase/milestone/wave names out of the path name.
- Preserve history in the file header, not in the path:

```text
# Reference: <historical tag>
# Source issue: <issue-file>.md
```

## Naming

Prefer names that answer "what feature does this show?"

Good patterns:

- `imports`
- `guarded_sequence_index`
- `tuple_attributes`
- `owned_mutation`
- `priority_queue`
- `structured_parsing_serialization`

Avoid planner/process naming in demo paths:

- `phase31`, `milestone`, `wave`
- `v2`, `v3`
- `analysis`, `audit`, `closure`, `contract`
- `hardening`, `remediation`, `governance`, `alignment`
- `regression_matrix`, `gate`, `fixes`, `cleanup`

If those labels matter for traceability, keep them only in `# Reference:` or `# Source issue:`.

## Demo Content

- Focus on one core feature or one tight cluster of related features.
- Prefer short, neutral helper names over problem-specific names.
- Use `assert` as the primary signal. `print` is optional and should be secondary.
- Keep examples compact and readable.
- Explain the feature with one or two short header comments at the top when needed.
- Prefer direct, concrete examples over broad showcase files.

## What To Avoid

- Full LeetCode or interview-problem solutions.
- Demos that primarily teach algorithm tricks instead of the Sifr feature.
- Reporting artifacts, scorecards, taxonomies, backlogs, or corpus manifests inside `demos/`.
- "Kitchen sink" demos that bundle many unrelated surfaces.

If a current demo is problem-shaped, keep the feature and shrink the example:

- `word ladder BFS` -> tiny queue-based BFS over a small typed graph
- `coin change DP` -> bounded local-table recurrence with neutral naming
- `trapping rain water` -> small same-sequence two-pointer example
- `KthLargest`/`MedianFinder` bundles -> tuple assignment or running-state examples

## Refactoring Existing Demos

When cleaning up an existing demo:

1. Identify the actual feature being demonstrated.
2. Remove milestone/planning/problem framing from the code and comments.
3. Rename functions and variables to neutral feature-oriented names.
4. Replace large problem fixtures with small assert-driven inputs.
5. Preserve the original history via `# Reference:` and `# Source issue:`.
6. Keep any non-demo artifacts in `verification/` or `internal_docs/`, not `demos/`.

## Non-Demo Artifacts

Do not keep these under `demos/`:

- corpus manifests
- runner outputs
- scorecards
- remediation backlogs
- failure taxonomies
- verification summaries

Place them in the canonical verification/docs location instead, usually `verification/leetcode/` or `internal_docs/verification/`.
