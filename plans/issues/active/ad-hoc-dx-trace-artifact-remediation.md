# Required DX trace-artifact remediation

status: implementation candidate; validation/review/merge pending; blocks DX.16
owner: CLI / driver observability; existing frontend trace and build-report owners

[Whole-phase review](https://github.com/sifr-lang/sifr/pull/3870#issuecomment-5748613670)
reviewed implementation `9ee360474655fde979dc5ddfdf33eb4dc39c9968` and docs
`1901074b62e2aad614d285f1c35c95aa16218490`; verdict NOT SATISFIED.
[Phase status](ad-hoc-compiler-dx-and-toolchain-reuse.md#execution-status) is blocked.

## Required scope

[Architecture §11.1](../../../internal_docs/compiler_dx_architecture.md#111-target-commands)
requires `--trace-dir <dir>`: "Opt-in versioned trace/artifact diagnostics with
redaction and controlled size". §11.3 requires existing timing/report owners,
identity/cache attribution, no credentials/complete environments/user source by
default, and separation from emitted source, program stdout and LSP transport.
DX.15 scope requires completion of integrated timings/trace/doctor/cache surfaces.
The phase owner explicitly retained this requirement after review; no waiver or
scope reduction was granted. Implement only this missing surface, not optional
whole-phase observations or another observability framework.

## Existing owners

- `crates/sifr/src/cli_model_and_entrypoint.rs`: global CLI schema and dispatch.
- `crates/sifr/src/main.rs`: immutable invocation context and command lifetime.
- `crates/sifr/src/trace_cli.rs`: existing text trace command, not a directory sink.
- `crates/sifr/src/eager_cli_contract_tests.rs` and `deferred_cli_args_tests.rs`:
  help/argument parity contracts.
- Existing driver BuildReport/frontend trace and process-reporting owners retain
  event authority. Extend them rather than introduce a second tracing engine.

## Named minimal acceptance for the implementation owner

These are proposed focused test names/behaviours, not tests already present or run:

- `trace_dir_cli_contract`: help and global argument parsing, including deferred
  command parity and missing directory argument rejection.
- `trace_dir_versioned_artifacts`: opt-in artifact directory contains a bounded,
  versioned report with command/compiler identity, stages and cache outcomes;
  ordinary invocation does not create trace artifacts.
- `trace_dir_redaction_and_size_bound`: secret-bearing environment/source sentinel
  data is absent by default, output stays within explicit size bounds, and any
  truncation is represented truthfully.
- `trace_dir_output_and_failure_contract`: emitted Rust/program stdout and JSON
  diagnostics remain unchanged; missing/unwritable destinations and compilation
  failure preserve actionable outcomes without false success.
- `trace_dir_timing_attribution`: trace-enabled and disabled invocations preserve
  semantic results and report tracing overhead without hiding preparation work.

Use actual installed CLI acceptance for artifact/output behaviour; reuse current
artifact inputs where valid. The next owner selects exact commands around these
existing contracts and records new evidence on its candidate. Do not claim these
checks cover an unexecuted full matrix or rerun unchanged broad qualification.

## Handoff

The docs closer did not implement this capability and did not merge or close
DX.16. Draft #3870 retains the blocking review. After scoped implementation and
validation, return to documentation closure; existing qualification remains valid
only for unchanged relevant inputs. Preserve the original NOT SATISFIED review.

## Bounded implementation and validation policy

This branch implements only the five named trace-directory contracts. The sink
projects existing owner reports; it does not add another tracing engine. The
CLI requires a fresh directory with an existing parent, writes one versioned
32 KiB maximum report, caps records at 64, counts omissions, and excludes free
text, paths, source, environment and child output. Successful finalization records
the original exit status; sink failure cannot report false success.

For this user-authorized remediation, run the named focused tests and scoped
Opus review, reuse unchanged DX.15 broad qualification, and do not restart the
full merge/create-PR gates. A new installed optimized compiler receipt and
package preparation bind affected CLI/output/timing evidence to this candidate.
Prior binary-specific measurements are not relabeled as this binary's evidence.
The original whole-phase NOT SATISFIED review remains preserved.

Commands:
- `cargo test -p sifr --bin sifr trace_dir_` (argument/parity and truncation units)
- `python3 verification/areas/performance/prepare_compiler_lane.py --lane product-installed-optimized --output <candidate>/product` (artifact preparation, not a gate)
- `python3 verification/areas/developer_tooling/trace_dir_acceptance.py --receipt <candidate>/product/receipt.json --evidence <candidate>/timing.json` (all five installed CLI contracts)
- File-size guardrail and relevant formatting/documentation checks.

This remediation does not close DX.16. The docs closer must reconcile its own
unmerged closure draft after this implementation is merged.
