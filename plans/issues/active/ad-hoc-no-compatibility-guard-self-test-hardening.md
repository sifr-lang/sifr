# Ad Hoc Follow-up: No-Compatibility Guard Self-Test Hardening

Status: queued after the pre-v1 compatibility-removal phase closes.

## Objective

Strengthen mutation coverage for the final no-compatibility guard without
changing its canonical contracts or reopening the completed removal phase.

## Source

The sole Item 16 remediation review for
`ad-hoc-pre-v1-compatibility-removal.md` found these new mechanisms. The phase
review limit permits no third review, so this issue owns them as later work.

## Item 0: Prove Excluded Mutations Are Reachable

The guard self-test must prove that each archive and generated-file mutation
is below an active scan root before `should_skip` excludes it. Removing an
archive root from `SCAN_ROOTS` must fail the self-test.

Acceptance criteria:

- Each excluded mutation records that its path was visited.
- Removing either archive scan root fails the self-test.
- Removing the generated `emitted.rs` exclusion fails the self-test.
- The production scan still excludes archives and generated companions.

## Item 1: Cover Both Source-Default Reader Shapes

The `legacy-source-default` rule has separate patterns for the driver reader
and the package reader. Its self-test must mutate both shapes independently.

Acceptance criteria:

- One mutation covers the driver `unwrap_or_else` shape.
- One mutation covers the package `None => PackageSourceRoot` shape.
- Removing either pattern fails the self-test.
- Both canonical readers keep the `src` default and pass the production scan.

## Validation

- `python3 verification/areas/developer_tooling/check_no_pre_v1_compatibility.py --self-test`
- `python3 verification/areas/developer_tooling/check_no_pre_v1_compatibility.py`
- `python3 verification/areas/developer_tooling/runner.py --suite static`

## Next Action

Start Item 0 in a separate phase-closure session after the pre-v1 phase record
is closed.
