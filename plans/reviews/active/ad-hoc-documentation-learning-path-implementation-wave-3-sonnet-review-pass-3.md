Wave 3 is fully PR-ready. The patch resolves the only soft flag from Pass 2:

- **"Why It Happens"** now correctly explains that `@blocking_io`/`@cpu_heavy` describe synchronous workload classes and cannot annotate `async def` because an async function already has its own scheduling contract — no longer generic.
- The erroneous/fixed code pair is well-chosen: the bad example shows `@blocking_io` on an `async def` with an `await`, the fixed example shows the annotation correctly on a plain `def` with a synchronous I/O call.
- Structure, frontmatter, Details table, and cross-links all look clean.

No new blockers introduced by this patch.
