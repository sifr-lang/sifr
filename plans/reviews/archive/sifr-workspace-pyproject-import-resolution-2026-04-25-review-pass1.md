# External Review Blocker

Note: future review prompts for this phase should write review output under `reviews/`, not `tmp/`.

Command attempted:

```bash
agent review
```

Result:

- Primary AX flow failed because `pyobjc` modules are not installed (`No module named 'AppKit'`).
- Geometry fallback also failed with `NameError: name 'kAXTrustedCheckOptionPrompt' is not defined`.

Review verdict: NOT AVAILABLE due to tool failure.
