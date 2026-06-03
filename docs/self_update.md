# Sifr Self Update

`sifr self update` updates official standalone preview installs created by the
Sifr installer:

```bash
curl -fsSL https://sifr.sh/install | sh
sifr self update
```

The command is intentionally narrow. The CLI verifies the local `install.json`
receipt, resolves a preview version, downloads the immutable version installer,
and delegates installation to that installer. The CLI does not download release
archives, extract artifacts, rewrite shell profiles, or bypass installer-owned
checksum validation.

## Commands

```bash
sifr self version [--short] [--format text|json]
sifr self update [--channel alpha|beta] [--version <preview-version>] [--dry-run] [--format text|json] [--force]
```

`sifr self version` reports the current executable, receipt version, install
directory, receipt binary path, target, channel, receipt match status, and the
current executable version. `--short` prints only the current executable version
in text mode.

`sifr self update` defaults to the channel recorded in the install receipt.
Use `--channel alpha|beta` to switch preview channels or `--version` to pin an
exact preview version such as `0.1.0-beta.2`.

Use `--dry-run` to print the resolved plan without downloading an installer or
acquiring the install lock. `--format json` is available only with `--dry-run`.

## Preview Limits

Self-update currently accepts only `alpha` and `beta` preview channels.
`stable` channels and stable-looking version pins remain gated until Phase 39
stable-channel promotion. Release-candidate channels and `-rc.N` pins are also
rejected before Phase 39.

Same-version reinstalls, downgrades, and channel switches require `--force`:

```bash
sifr self update --version 0.1.0-beta.2 --force
sifr self update --channel alpha --force
```

Regular newer-version updates within the receipt channel do not require
`--force`.

## Troubleshooting

`sifr self update` requires a schema-versioned receipt written by the official
standalone installer. If you installed Sifr with Cargo, Homebrew, a system
package manager, or a source build, update through that tool instead. Use
`sifr --version` for the raw binary version in unmanaged installs.

If the diagnostic says the receipt is missing, malformed, or predates the
self-update contract, rerun the standalone installer to enter the managed
contract:

```bash
curl -fsSL https://sifr.sh/install | sh
```

For custom install directories, rerun the installer with the same environment:

```bash
curl -fsSL https://sifr.sh/install | SIFR_INSTALL_DIR="$HOME/bin" sh
```

If the diagnostic says the receipt belongs to a different executable, your
`PATH` is likely finding another `sifr` binary. Check `command -v sifr`, run the
intended installed binary directly, or reinstall the standalone binary so the
receipt and executable match.

If the installer reports a newer installed version, use `--force` only when you
intend to downgrade. If metadata or installer download fails, retry after
network recovery; the existing binary is not replaced until the delegated
installer validates and installs the target artifact.
