# Sifr Self Update

`sifr self update` updates official standalone installs created by the
Sifr installer:

```bash
curl -fsSL https://sifr.sh/install | sh
sifr self update
```

The command is intentionally narrow. The CLI verifies the local `install.json`
receipt, resolves an active governed version, downloads the immutable version installer,
and delegates installation to that installer. The CLI does not download release
archives, extract artifacts, rewrite shell profiles, or bypass installer-owned
checksum validation.

Channel resolution uses the `channels.json` asset on the `sifr-lang/sifr`
GitHub release tag `channels`. The resolved version's immutable installer is
downloaded from that version's GitHub release asset.
GitHub may redirect release asset downloads through its object storage hosts;
self-update allows only HTTPS downloads and HTTPS redirects.

The metadata is the canonical schema-v2 governed release index. Each channel
must point to an active release record with immutable installer and target
digests. Release mutation refuses publication when the existing v2 index is
unavailable or its expected generation/digest has changed.

## Commands

```bash
sifr self version [--short] [--format text|json]
sifr self update [--channel alpha|beta|stable] [--version <active-version>] [--dry-run] [--format text|json] [--force]
```

`sifr self version` reports the current executable, receipt version, install
directory, receipt binary path, target, channel, receipt match status, and the
current executable version. `--short` prints only the current executable version
in text mode.

`sifr self update` defaults to the channel recorded in the install receipt.
Use `--channel stable` for the governed stable channel. Alpha and beta remain
explicit preview choices. Use `--version` to pin an exact active release such
as stable `0.1.0`.

Use `--dry-run` to print the resolved plan without downloading an installer or
acquiring the install lock. `--format json` is available only with `--dry-run`.

## Channel and version rules

Self-update accepts `alpha`, `beta`, and `stable`. `rc` is not a public channel,
and `-rc.N` pins are rejected. Stable-looking pins resolve only when the exact
version is present and active in the governed release index. Withdrawn versions
are rejected.

Same-version reinstalls, downgrades, and channel switches require `--force`:

```bash
sifr self update --version 0.1.0 --force
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
self-update rules, rerun the standalone installer to enter the managed
rules:

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
intend to follow an approved downgrade or reinstall. If metadata or installer
download fails, retry after network recovery; the existing binary is not
replaced until the delegated installer validates and installs the target
artifact.

See [stable releases](/releases/stable) for the active version and
withdrawal rules.
