# Sifr 0.1.0

Sifr 0.1.0 is the first governed stable release of the Sifr compiler,
installer, sysroot, self-update flow, public documentation, and VS Code
extension integration.

## Qualified targets

- `aarch64-apple-darwin` — macOS 15.0
- `x86_64-apple-darwin` — macOS 15.0
- `aarch64-unknown-linux-gnu` — glibc 2.39
- `x86_64-unknown-linux-gnu` — glibc 2.39

Qualification verifies the exact SHA-256 of every candidate target archive,
sysroot bundle, checksum file, aggregate installer, VSIX, and report before
publication. Install receipts are checked for version and channel agreement.

During installation, the dispatcher verifies the immutable installer's
SHA-256 from the governed release index. The installer then verifies the
selected target archive and sysroot before replacing a toolchain.

## Install and update

The default installer selects the stable channel:

```bash
curl -fsSL https://sifr.sh/install | sh
```

The explicit stable entrypoint selects the same release:

```bash
curl -fsSL https://sifr.sh/install/stable | sh
```

For a reproducible version pin:

```bash
curl -fsSL https://sifr.sh/install | sh -s -- --version 0.1.0
```

Update an official standalone installation with:

```bash
sifr self update
```

The governed release index uses `schema_version: 2`. Unknown, withdrawn, and
`X.Y.Z-rc.N` versions are rejected.

## Editor integration

The qualified VS Code extension is `sifr.sifr-vscode` 0.2.0 with compiler
compatibility `>=0.1.0,<0.2.0`. Protected publication reuses the exact
qualified VSIX without rebuilding it. Its compiler-backed actions include a
generated-Rust preview served by the native LSP.

## Recovery model

This first GA release has no eligible stable rollback predecessor. An incident
before a later normal stable release is recovered through the governed
incident roll-forward path. If an installed binary cannot run self-update, the
out-of-band recovery path is the explicit stable installer; `--force` is
reserved for an intentional reinstall, channel switch, or approved downgrade.

## Current scope

Windows packages, package-manager distribution, binary signing, and
notarization are not part of this release.

Stable Rust interop support is limited to the categories and evidence scopes
in the [stable claims table](https://sifr.sh/rust-interop#stable-support-claims).
A `contract-only` row does not claim runtime-observed support, and future-owned
runtime rows are not advertised.

Alpha and beta remain explicit preview channels. `rc` is not a public channel.
