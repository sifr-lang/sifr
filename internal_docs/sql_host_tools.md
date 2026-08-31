# SQL host tools

This document defines the implemented host-tool graph and command runner.

## Ownership

`sifr_package` owns tool discovery, identity, graph separation, and command
plans. The `sifr` CLI owns direct namespace dispatch and child-process results.
`sifr_sql_contract` owns the test connection manifest.

Host tools are packages. They are not part of the Sifr standard library. They
are not application dependencies.

## Workspace configuration

The root Cargo manifest selects one tools workspace member:

```toml
[workspace.metadata.sifr]
tools-package = "project-tools"
```

The selected Cargo package must be an exact workspace member. It must have a
Sifr discovery pointer:

```toml
[package.metadata.sifr]
manifest = "sifr.toml"
```

The tools manifest exports direct namespaces:

```toml
[tools.sql]
package = "sifr-sql-postgresql-tools"
entrypoint = "sql"
capabilities = ["network", "credentials", "project-write"]
```

The MySQL provider uses `sifr-sql-mysql-tools` with binary
`sifr-sql-mysql`. A project selects one provider tool package for the `sql`
namespace. Provider tool packages do not share or chain that namespace.

The package must be one direct, normal dependency of the tools member and an
exact workspace member. The entry point must be one binary target in that
package. A transitive, build-only, dev-only, registry-only, or Git-only package
is not valid. Add a small workspace wrapper when an external tool does not
already provide a workspace-member binary. This restriction gives Sifr a stable
artifact path and prevents Cargo package-name ambiguity.

## Locked identity

Sifr runs `cargo metadata --frozen`. Run this command after a reviewed tool
change:

```text
sifr tools lock
```

Commit the generated `sifr-tools.lock.json`. CI uses `sifr tools lock --check`.
The artifact records these inputs for each entry:

- Cargo package ID, name, version, and source
- Cargo package checksum, or a deterministic path-package identity; the path
  hash excludes only the workspace target directory, root Git metadata, and the
  two separately hashed lock artifacts
- binary entry-point name
- sorted capability grants
- tools-manifest SHA-256 fingerprint
- complete `Cargo.lock` SHA-256 fingerprint

Sifr verifies `Cargo.lock`, `sifr.toml`, every path-package byte, and the committed
tool lock before it builds and again before it starts the tool. A changed or
missing input is an error. The runner also hashes the produced executable and
passes that hash as `SIFR_TOOL_EXECUTABLE_SHA256`.

## Direct command dispatch

The CLI accepts a package namespace directly:

```text
sifr sql schema build --profile app
sifr sql test provision --profile app
```

The runner selects only the configured package and binary. It forwards all
arguments after the namespace. It resolves Cargo to an absolute path, removes
compiler-wrapper and Rust-flag environment overrides, invokes `cargo build
--locked` with an explicit host target and target directory selection,
and executes the resulting binary directly. It never uses `cargo run` and never
uses a Cargo runner. Thus, a workspace cross-compilation target cannot compile
or run a tool for the application target.

Built-in CLI names are reserved. The tools manifest also rejects an invalid
name, an unknown capability, a repeated capability, a missing binary, and an
unsupported field.

## Capability grants

The tools manifest is the project-owned grant record. Sifr enforces the grants
with the native operating-system sandbox. It fails closed if `sandbox-exec` is
not present on macOS or Bubblewrap is not present on Linux. There is no
unconfined fallback. Output is limited to 10 MiB across stdout and stderr.

The runner clears the child environment. It passes ordinary variables only for
`environment`, credential-shaped variables only for `credentials`, and `PATH`
only for `subprocess`. The sandbox independently controls project reads,
project writes, network access, home-directory credential reads, and subprocess
execution. The runner provides the sorted grants, package checksum, Cargo lock
fingerprint, and executable hash as audit variables.

The closed capability vocabulary is:

- `credentials`
- `environment`
- `network`
- `project-read`
- `project-write`
- `subprocess`

An unknown capability is an error. Package review remains the trust boundary
for native host code, in the same way as a trusted Rust backend package.

## Application isolation

The tools member is classified as host-only. Sifr does not load its tool
manifest as an application manifest. The resolver computes each application
dependency closure. It rejects a closure that reaches the tools member or a
selected tool entry package.

Application compilation continues to select the target application package.
Tool packages do not enter source discovery, HIR, generated Rust, linker input,
or application artifacts.

## Test provisioning

`sifr sql test provision --profile <name>` must return exactly one JSON test
connection manifest. The common contract contains:

- contract version
- provider and profile identities
- canonical schema fingerprint
- typed TCP or file connection data
- an environment reference or a structured helper executable plus argument
  array, never an inline credential or shell command string
- cleanup namespace and resource identity; cleanup always routes as
  `sifr <namespace> test cleanup --resource-id <id>`
- optional expiry time

The CLI validates the document. It also verifies the requested profile and the
cleanup namespace. It prints canonical JSON only after validation.

## Failure behavior

Tool resolution fails closed for an unknown namespace, reserved namespace,
missing direct package, missing binary, unknown capability, lock drift, or
application-graph contamination.

SQL editor initialization has a different failure boundary. A missing lockfile,
invalid schema, or provider initialization error disables SQL enrichment and
adds an explicit diagnostic. It does not disable ordinary Sifr analysis.
