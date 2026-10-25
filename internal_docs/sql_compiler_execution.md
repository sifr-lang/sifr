# SQL compiler execution boundary — implementation in progress

The LOCAL-GATE V1s phase owns this change. This document records the intended
call and lifetime contract; the production adapter is not yet qualified.

## Ownership and entrypoints

The canonical compiler owns profile preparation and migration-source compilation.
The native SQL libraries own provider validation, connections, credentials,
migration execution and artifact writes. Their V1q callable APIs are the seam.

Use one short-lived canonical compiler worker per tool invocation when semantic
inputs are needed. Reading an existing plan and database cleanup must not start
that worker. Do not introduce a daemon or persist correctness verdicts.

A direct native tool selects an explicitly paired source compiler or the compiler
in its selected immutable installed generation. An explicit development
selection is authority to execute that chosen build; a self-reported digest is
not. Installed selection reuses generation integrity and the existing
compiler-binary/metadata binding. No arbitrary PATH search or fallback to a
second embedded compiler implementation.

For confined namespace execution, the verified outer compiler owns worker
selection and launches the worker outside the native tool sandbox. The tool
receives only the private service channel under an explicit SQL compiler grant.
That channel must not grant arbitrary subprocess, network or filesystem access.
The native tool's existing sandbox remains responsible for its own operations.

## Source and executable identity

Source pairing needs a configuration-independent compiler source identity in
addition to the existing configured build identity. It must use the same declared
source roots and hash framing as the existing product identity, before feature,
profile, toolchain and dependency-token configuration is appended. Keep the
configured identity algorithm unchanged.

The source identity detects a mismatched compiler revision or edited inputs;
it is not executable authentication or an installation signature. Executable
selection, integrity, target/protocol compatibility and channel custody remain
separate checks. An installed generation's existing binary binding remains the
authority for its selected executable. Source execution must retain its explicit
build selection and reject replacement or drift.

## Channel and admission

An owned anonymous duplex socket inherited through stdin is feasible through the
unchanged Linux sandbox with only project-read. The V1s diagnostic confirmed
outside-file, project-write, subprocess and credential restrictions remain in
effect. Closed/silent peers fail, and ordinary pipe data is not a socket.
This transport proof used dummy bytes; it does not qualify canonical operations,
source/installed pairing, production cancellation or the macOS route.

Reuse sifr_ipc's bounded framing, schema/version checks and request tracking where
appropriate. Only the two named semantic operations are admitted. Requests do
not select arbitrary workspace roots or compiler executables. The worker is
bound to the caller's verified project and lock/input snapshot. Source compilation
must be bound to admitted migration inputs; arbitrary file reads are forbidden.
Returned profile identities and migration declarations retain their canonical
validation and every typed field. Preserve semantic diagnostic text.

Frame bounds, peer failure and request identity are protocol checks, not proof
that a payload came from the canonical compiler. The launcher owns that custody.
Direct execution must establish the same ownership through the selected worker.
Do not expose an unchecked packet-file shortcut as a normal command.

## Lifecycle and validation

The launcher owns the worker, channels and descendants until successful cleanup.
EOF, malformed traffic, oversized messages, wrong source/protocol identity,
input drift, worker death, cancellation and deadlines are explicit failures.
Reuse or extract existing process ownership primitives rather than weakening
cleanup or copying an unbounded subprocess pattern. Charge preparation,
identity checks, communication and cleanup to operation timing.

The default-feature transition follows source and installed command comparisons
and real sandbox qualification. Continue validating sandbox denial cases,
all affected SQL/compiler/editor/installation/runner assertions and the actual
three-executable independent clean-build protocol. The complete LOCAL-GATE
create-pr/merge and V5 requirements remain open.

## Current development evidence

The hidden canonical worker and its process session are implemented. Focused
checks compare real profiles and migration declarations against the existing
in-process semantics for all three providers and reject input drift. Native source and installed execution and the private sandbox capability now have
focused development qualification. Ordinary binaries use the worker adapter and
no longer select the optional embedded compiler feature. Explicit compiler-enabled
test commands preserve the existing in-process API coverage. Complete affected
coverage, clean executable qualification, independent review and the phase gate
remain open.

The worker fixes its frozen graph and profile inputs at initial preparation.
Migration input bytes are pinned at the first source request and checked before
and after compilation and on subsequent requests. This preserves lazy schema
commands while forbidding arbitrary source requests and between-request drift.
The shared decoder now rejects unused bytes inside declared IPC payloads.


## Explicit selection and current validation scope

Source-tree commands require an absolute SIFR_SQL_COMPILER pointing to the
paired build, with the source sysroot selected through the existing sysroot
mechanism. Installed commands use bin/sifr in the selected generation and
reject a source override. No executable is found through PATH. Selection
checks executable bytes before and after execution and checks installed
generation integrity and its compiler/metadata binding.

A confined invocation requires the sql-compiler capability. The outer compiler
attaches an anonymous socket and starts the selected worker only after a request
arrives. Direct and inherited sessions own cancellation and worker cleanup.
The capability grants neither subprocess access nor additional filesystem or
network access to the native tool.

The source comparison covered 24 native command cases with 33 identical artifact
files and actual SQLite provisioning/cleanup. The installed comparison repeated
those cases using a generation produced by the distribution metadata pipeline.
Five installed negative controls reject a source override, manifest drift,
manifest symlinks, executable substitution and executable symlinks without
project writes. Real sandbox tests cover all three schema providers, SQLite
provisioning/cleanup, missing grants, wrong workers and lazy plan reads. Existing
locked namespace admission tests remain separately selected; the real native
sandbox tests directly provide trusted executables and do not claim to repeat
Cargo/package admission in that same invocation. No Windows or macOS execution
qualification is inferred from these observations.
