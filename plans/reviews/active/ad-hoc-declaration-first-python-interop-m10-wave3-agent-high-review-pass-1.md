# M10 Wave 3 agent Review Pass 1

- Reviewer: agent
- Reasoning: high
- Service tier: fast
- Scope: complete committed `main...b3616a7ab` Wave 3 diff for PR #2989
- Verdict: changes requested

## Findings

1. **High — exact release and pointer evidence was incomplete.** Logical
   resource diagnostics did not independently prove exact `PyBuffer_Release`,
   failure rollback, or producer/view pointer identity.
2. **Medium — the evidence validator accepted fabricated matrices.** It did
   not lock the schema version, exact/unique rows, resolvable owners,
   cancellation reason, or duplicate live rows.
3. **Medium — generated packages overgranted trust roots.** A shared bridge
   directory caused unrelated `builtins`, `mmap`, and NumPy cases to authorize
   roots they did not use.
4. **Low — the architecture evidence paragraph contained a broken sentence.**

## Reviewer validation

- Five compiled demo binaries passed.
- Focused lowering passed `34/34`.
- Focused code generation passed `10/10`.
- Focused runtime buffer operations passed `18/18` before the new release
  evidence tests were added.
- Runner self-test, diff check, and HIR maintainability guardrails passed.

## Remediation

- Added an instrumented Python buffer exporter and five runtime tests that
  independently assert real data-pointer identity and exact release counts for
  explicit release, automatic drop, validation failure, admission conflict,
  and store rollback.
- Added compiled bridge and aggregate exporter counters plus retained-producer
  NumPy mutation/identity checks.
- Replaced the permissive evidence parser with an exact schema, coverage,
  owner-resolution, live-case, manifest, and delivery-profile validator plus
  mutation self-tests.
- Split buffer bridge modules by responsibility and copy only the declared
  bridge file, yielding exact per-case import/native trust roots.
- Repaired the architecture paragraph.

The complete remediated diff requires a fresh whole-diff review pass.
