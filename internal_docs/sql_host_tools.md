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

The package must be one direct dependency of the tools member. The entry point
must be one binary target in that package. A transitive package is not valid.

## Locked identity

Sifr runs `cargo metadata --frozen`. It records these inputs for each entry:

- Cargo package ID, name, version, and source
- Cargo package checksum, or a deterministic path-package identity
- binary entry-point name
- sorted capability grants
- tools-manifest SHA-256 fingerprint
- complete `Cargo.lock` SHA-256 fingerprint

Sifr verifies the lock fingerprint again before it starts the tool. A changed
lockfile is an error. A missing lockfile is an error.

## Direct command dispatch

The CLI accepts a package namespace directly:

```text
sifr sql schema build --profile app
sifr sql test provision --profile app
```

The runner selects only the configured package and binary. It forwards all
arguments after the namespace. It invokes Cargo with `--locked` and an explicit
host target. Thus, a workspace cross-compilation target cannot compile or run a
tool for the application target.

Built-in CLI names are reserved. The tools manifest also rejects an invalid
name, an unknown capability, a repeated capability, a missing binary, and an
unsupported field.

## Capability grants

The tools manifest is the project-owned grant record. A tool has only the named
grant set in the Sifr command contract. The runner provides the sorted set in
`SIFR_TOOL_CAPABILITIES`. It also provides the package checksum and lockfile
fingerprint.

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
- an environment or helper credential reference, never an inline credential
- cleanup namespace and resource identity
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
