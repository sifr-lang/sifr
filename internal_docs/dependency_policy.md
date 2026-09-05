# Dependency Policy

Sifr is a programming language and compiler, not a single Rust application. Dependencies must be classified by where they are allowed to appear, which Sifr feature activates them, and whether they can affect generated user projects.

The default rule is dependency-minimal and ecosystem-first: use mature Rust crates where they are the right production substrate, but keep crate types behind Sifr APIs and do not add adjacent crates for convenience. If a dependency cannot satisfy Sifr's ownership, typed-error, cancellation, panic, host, and testing rules, defer the Sifr surface instead of adding a bespoke replacement or fallback path.

## Workspace Lockfile

Sifr tracks the root `Cargo.lock`. This repository builds a compiler, CLI, runtime crates, release tools, and verification harnesses as one Rust workspace, so the lockfile is part of the build and validation rules rather than an application-local convenience.

Local validation and CI both start from the committed lockfile. The main validation profile runs the same `scripts/run_all_tests.sh` facade locally and in CI, keeping contributor machines and GitHub Actions on the same dependency graph unless a PR intentionally changes it.

Any `Cargo.lock` diff is a contributor-visible dependency change. Validate it with the same care as a manifest edit: confirm why the graph changed, whether transitive updates are expected, and whether generated runtime, release, or verification behavior can be affected.

## Rings

### Ring 1: Compiler And Tooling Only

Compiler/tooling dependencies may be used by the compiler, CLI, diagnostics implementation, local verification tools, test harnesses, docs generators, or build tooling. They must not automatically leak into generated user projects, generated runtime preambles, public stdlib APIs, or Sifr language semantics.

Examples:

- `anyhow`
- `eyre`
- `tracing-subscriber`
- `insta`
- `tempfile`
- `proptest`
- local verification helpers

`anyhow` and `eyre` are acceptable only when contained to compiler/tooling implementation paths that do not replace structured diagnostics, typed compiler errors, generated runtime errors, or Sifr language-level `Result`/diagnostic rules.

### Ring 2: Generated Runtime Core

Generated runtime core dependencies may appear in generated Sifr projects only when the program uses the runtime feature that requires them. They provide substrate for Sifr-owned runtime semantics and must be feature-minimal.

Examples:

- `tokio` for accepted async runtime, timers, process I/O, signal, sync, and blocking-pool substrate
- `tokio-util` only when internal cancellation machinery or Tokio I/O helpers require it
- `futures-util` only where generated-runtime async combinators would otherwise require substantial custom poll-level helper code
- `tracing` for structured runtime events

Crate types, runtime handles, subscribers, recorders, and global runtime configuration must not appear in public Sifr APIs.

### Ring 3: Stdlib Feature-Gated Substrate

Stdlib substrate dependencies are enabled only by the Sifr stdlib feature that needs them. They are not a baseline generated-project dependency.

Examples:

- `crossbeam-channel` for sync cross-thread channels, if those channels remain production-public
- `rayon` for CPU-heavy parallel work and `sifr.parallel`
- `rustix` for documented host-limited process/signal/file-descriptor gaps not covered by `std` or Tokio
- `metrics` only after a concrete metric schema, label/cardinality policy, and test strategy exist
- `thiserror` only as an internal Rust implementation aid for first-party runtime/compiler error enums

Every Ring 3 dependency needs a feature gate, host-matrix story where host behavior differs, panic story, typed error mapping, and deterministic local tests.

### Ring 4: Feature-Specific Protocol Or Data Substrate

Protocol/data dependencies are scoped to one accepted feature family. They must not become general "serialize anything" or compatibility facilities.

Examples:

- `serde` for generated typed schemas where the Sifr compiler owns eligibility rules
- `postcard` for typed IPC typed local IPC frames

For typed IPC, Sifr owns `IpcSerializable`, schema identity, version negotiation, compatibility policy, cancellation frames, and malformed-frame diagnostics. `postcard` is only the binary encoding backend. `serde`/`postcard` must appear in generated projects only when the user uses an accepted IPC or serialization feature.

### Ring 5: Dev/Test/Demo Only

Dev/test/demo dependencies may support fixtures, golden tests, local demos, or documentation verification. They must not be required by production generated projects or runtime semantics.

Examples:

- `tracing-subscriber` for tests and demos that install a subscriber
- `insta` snapshots
- `tempfile`
- `serde_json` for inventories, debug artifacts, or golden files

### Ring 6: Rejected Direct Dependencies

Rejected direct dependencies are not used directly by first-party Sifr code for the given feature surface. They may appear transitively through accepted dependencies, but Sifr must not build public/runtime semantics around them.

Examples for the current runtime/platform scope:

- extra channel stacks such as `flume`, `async-channel`, and `futures-channel`
- direct `parking_lot` for runtime locks
- new runtime use of `once_cell` where `std::sync::OnceLock` is enough
- `scopeguard` for Sifr cleanup semantics
- `serde_json` as a production IPC frame format
- `bincode` where `postcard` is the selected typed IPC codec
- Unix abstraction stacks such as `signal-hook` and `nix` when Tokio plus targeted Rustix is sufficient
- direct `mio`, `bytes`, or `dashmap` unless a later design proves a narrow production need
- `anyhow`/`eyre` for runtime or language-facing errors

`bincode` is not rejected because it is pickle-like. It is rejected for the typed IPC work because Sifr chooses one compact binary Serde codec, and multiple production IPC codecs would complicate schema/version compatibility. Pickle-like arbitrary object transport is rejected separately.

## Acceptance Checklist

Before a dependency is accepted outside Ring 1 or Ring 5, the surface or issue must record:

- dependency ring
- accepted crate, version, and exact feature flags
- Sifr feature or `StdlibFeature` that activates it
- whether it can appear in generated user projects
- public API boundary proving no crate types leak
- ownership/lifetime mapping
- cancellation/drop semantics where relevant
- sendability/shareability or serialization eligibility mapping
- typed error mapping into Sifr variants or diagnostics
- panic/unsafe audit for user-controlled paths
- host-matrix impact
- binary-size/MSRV/license/supply-chain notes
- deterministic local tests and golden fixtures
- observability and redaction policy where events/metrics/logging are involved

## Generated Project Hygiene

Generated user projects must include only the dependencies required by the Sifr features the program uses. Compiler/tooling dependencies do not imply generated runtime dependencies.

Every generated-project dependency must be represented by a stable Sifr feature decision, not by incidental compiler implementation. Feature selection must be narrow; broad feature flags such as `full` are rejected unless a feature decision explicitly justifies them.

## Public API Boundary

Sifr owns public semantics. Rust crates provide substrate.

Public Sifr APIs must not expose Rust implementation crate types, configuration objects, global recorders, runtime handles, task handles, serializers, subscribers, or error bags. Public APIs expose Sifr-owned types, typed `Result`/sum errors, diagnostics, cancellation evidence, ownership rules, and capability-specific configuration.

## Change Control

Accepted or rejected dependency decisions are implementation inputs. Implementation PRs must not perform crate-family discovery or swap adjacent crates. Changing a dependency decision requires a new issue or explicit policy amendment before implementation starts.
