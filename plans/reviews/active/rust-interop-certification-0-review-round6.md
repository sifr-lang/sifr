## Round-6 re-audit of `certification_0` (working tree vs `7554f89b5`)

Read-only. Nothing was written, staged, or committed in the repository. All
adversarial probes ran in memory against the real matrix, claims, and public
documentation.

### Round-5 blocker: resolved

The expanded `_prose_units` logic rejects all five round-5 qualifier-borrowing
shapes:

- G: separate Markdown bullets for a contract-only qualifier and runtime
  advertisement;
- H: comma-free `except`;
- I: `although`;
- J: separate Markdown bullets for a runtime deferral and runtime
  advertisement; and
- K: comma-free runtime-deferral `except`.

Numbered lists, alternate bullet markers, Markdown table rows, widened
adversative connectives, exact backticked row tokens, stale-promotion prose,
and secondary-document prose were also rechecked. The self-test carried the
five reproduced shapes and passed 26 cases; the complete Rust-interop area
passed all 10 variants.

### Blocking finding

**MEDIUM — the stable-claims marker region is a blind spot.**

`_outside_claim_table` removes everything between the stable-claims markers
before prose validation, while `_parse_public_claims` keeps only table lines
and silently discards other content. The canonical parser is applied only to
`docs/rust-interop.mdx`; secondary documents in the docs-wide sweep are not
allowed to establish another claim-table authority.

Two reproduced bypasses returned no failures:

1. Canonical shape L placed an unqualified runtime advertisement after the
   real claim rows but before the end marker. The table parser ignored that
   non-table line and the prose sweep never saw it.
2. Secondary shape M copied the marker pair into `docs/release-notes.md`, used
   a drifted `runtime-observed` table row for `zero_copy_bytes`, and added a
   runtime advertisement for a future-owned row. The secondary marker region
   was stripped before the docs-wide prose sweep.

Required correction:

- allow only nonblank table rows inside the canonical marker block;
- reject either stable-claims marker in every secondary public document;
- mutation-test L and M through the real docs-wide `main()` path; and
- make reversed canonical markers fail closed without a traceback.

### Optional findings

- The runner ordering seam is covered, but the real lane-step stdout line is
  still inferred rather than captured.
- The defense-in-depth prose sweep is necessarily keyword and Markdown-shape
  bound; the canonical structured-table comparison remains exhaustive.
- No cold-cache `cargo fetch --locked` measurement is recorded.
- The planned `zero_copy_runtime_matrix` example retains its pre-existing
  `/tmp` convention and must change when `certification_7` executes it.
- `check_stable_support_claims.py` and `profile_runner.py` retain limited
  maintainability headroom while remaining below the 900-line guardrail.

**Blocking: 1.** Optional: 5.

NOT SATISFIED
