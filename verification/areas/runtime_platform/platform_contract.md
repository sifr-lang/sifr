# Production Stdlib Platform Contract

Status: approved shared baseline for split production-stdlib substrate phases.

This contract is shared by the text/i18n, concurrency/runtime, and network/HTTP production-stdlib substrate phases. It is not a CPython compatibility layer. It defines native Sifr platform rules for state vocabulary, ownership, cancellation, backpressure, typed error nesting, observability, host support, security/resource ownership, and executable cross-phase golden fixtures.

## Terminal States

Every phase inventory entry must use one terminal state:

| State | Meaning |
| --- | --- |
| `production-public` | Stable recommended Sifr API for user code. |
| `production-substrate` | Stable substrate consumed by public APIs or later phases. |
| `compat-adapter` | Thin adapter over production substrate with proven migration value. |
| `internal-only` | Implementation detail hidden from stable imports. |
| `test-only-harness` | Fixture or harness code unavailable as user API. |
| `deferred-to-phase-X` | Explicitly deferred to a named phase or issue. |
| `blocked-on-phase-X` | Blocked until a named provider milestone closes. |
| `host-limited` | Supported only on named hosts with documented behavior and tests. |
| `unsupported-with-diagnostic` | Unsupported and must produce a stable diagnostic when referenced. |
| `rejected` | Intentionally out of product scope with rationale and revisit rule if any. |

Reference evidence and test families use `mined-as-substrate-fixture`, `adapted-for-sifr-api`, `compat-adapter-deferred`, `blocked-on-phase-X`, `external-signal`, `waived-with-rationale`, or `rejected`.

Stable or semi-stable surfaces must also declare one stability level: `stable-public-api`, `stable-production-substrate`, `unstable-internal-substrate`, `compiler-known-intrinsic`, `compatibility-adapter`, or `test-only-harness`.

## Ownership And Lifetime

| Resource family | Contract |
| --- | --- |
| Text values | Always valid Unicode scalar text; arbitrary bytes never become text without explicit decoding. |
| Bytes, HTTP bodies, subprocess pipes, and IPC payloads | Owned by default. Borrowing across async/task/process boundaries requires compiler proof. |
| TCP streams, TLS streams, HTTP body streams, subprocess pipes, and text streams | Linear resources with explicit close/shutdown/drop behavior; double-close and use-after-close are diagnostics or typed errors. TCP/TLS full-duplex use requires owned split halves unless a future phase proves a borrowed lifetime-safe design. |
| Incremental codecs | Unique mutable state; no concurrent aliasing or hidden shared mutation. |
| Executor futures and task handles | Must be observed, joined, cancelled, or explicitly consumed; unobserved failure is diagnosed. |
| Task, thread, process, and IPC boundary captures | Must satisfy the owning phase's sendability/shareability or IPC-serializability rules before codegen. |

## No Global State

Production platform APIs must not introduce unsynchronized process-global mutation. Text/i18n enforces a static codec registry, rejects `codecs.register` / `codecs.unregister` / `codecs.register_error`, rejects process-global `locale.setlocale`, rejects `gettext.install` and global `_` injection, and forbids locale-derived implicit text I/O defaults. Locale-sensitive behavior is object-scoped through explicit `LocaleId` and formatter values.

Concurrency/runtime rejects public event-loop policy mutation, raw thread/process-pool globals, Python `warnings` global filter parity, and mutable global Rayon/Tokio runtime configuration. Runtime work is scoped, observed, and typed through Sifr-owned handles.

## Cancellation, Backpressure, And Errors

Cancellation is typed evidence, not an invisible drop path. Streaming decoders preserve linear state and report exhausted or partial-state errors explicitly. Bounded buffers have explicit capacity and typed full/closed outcomes. Producers cannot hide unbounded buffering inside adapters.

Higher-level errors preserve lower-level evidence, for example `ProcessError::Pipe(PipeError::TextDecode(DecodeError))` and `HttpError::Text(DecodeError)`. Exception-only control flow is rejected.

Network/HTTP cancellation, timeout, shutdown, backpressure, offload, and diagnostics consume the concurrency/runtime provider model. Network/HTTP must not introduce a parallel cancellation token, deadline coordinator, shutdown manager, queue/channel primitive, executor, process-worker model, or diagnostics bus.

## Observability

Structured platform events use shared fields: `sifr.phase`, `sifr.operation`, `sifr.correlation_id`, `sifr.task_id`, `sifr.resource_id`, `sifr.host`, `sifr.error.kind`, `sifr.error.cause`, `sifr.blocked_on`, and `sifr.duration_ms`.

Redaction is required for URLs with credentials, headers, cookies, request/response bodies, certificate fields, subprocess command lines, environment variables, IPC payloads, locale data paths, and translation catalog contents.

## Security And Resource Ownership

| Concern | Owning phase |
| --- | --- |
| Codec amplification and malformed byte sequences | text/i18n |
| Malicious `.mo` catalog, plural expression, and translation payload handling | text/i18n |
| Locale data discovery and host-limited formatting | text/i18n |
| Subprocess resource limits and `@shell_exec` security surface | concurrency/runtime |
| IPC payload size and panic-free malformed-message handling | concurrency/runtime |
| Cancellation storms, task explosion, queue bounds, and offload capacity | concurrency/runtime |
| Sendability/shareability across task, thread, process, and IPC boundaries | concurrency/runtime |
| Structured signal shutdown and process termination escalation | concurrency/runtime |
| Buffer/body size limits, parser DoS input, HTTP flow control, TLS verification, headers, and smuggling | network/HTTP |
| Log redaction for URLs, headers, bodies, cookies, certificates, subprocess commands, env, and catalogs | shared contract with owning phase-specific fields |

## Golden Fixtures

Executable platform fixtures live under `verification/areas/runtime_platform/golden` and are declared in `verification/areas/runtime_platform/golden/manifest.json`. `uv run --project verification --locked python -m sifr_verify areas run --area runtime_platform --suite platform-golden` skips entries whose `blocked_until` milestones are not listed in `SIFR_PLATFORM_CLOSED_MILESTONES`; unblocked entries must satisfy their exit code and output expectations.

Text/i18n owns the binary file I/O prerequisite fixture, unsupported CPython import diagnostic fixture, and text-dependent blocked entries. Later phases must consume this substrate instead of adding local encoding, Unicode, locale, or fallback decoder behavior.

Network/HTTP owns the unsupported CPython network import fixture, loopback-only transport fixtures, and HTTP body text fixtures that remain blocked until the required text/i18n and network milestones close.

## Sanitizer And Model Lanes

Sanitizer, Miri, and deterministic concurrency model lanes are declared in `verification/areas/runtime_platform/sanitizer_manifest.json`. The merge profile selects `sanitizer-smoke`; nightly and release select `sanitizer-full`.

Each sanitizer case records:

- a target suite (`sanitizer-smoke`, `sanitizer-full`, or both)
- the runtime/platform scope under test
- the sanitizer or model tool
- supported host triples
- required local tools, Rust toolchains, and Rustup components
- the exact reproduction command and environment
- timeout and finding-promotion policy
- a structured skip reason when local support is unavailable

The runtime-platform runner reports unsupported cases as `skip`, not `pass`, and keeps those skips machine-readable in the area result JSON. Supported hosts execute the checked-in command with `CARGO_NET_OFFLINE=true` unless the case explicitly overrides the environment. A sanitizer/Miri/model finding must be minimized and promoted to `regression:fixedbugs` or an owning runtime/platform golden fixture before the finding can close.
