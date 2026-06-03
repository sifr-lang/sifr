Got enough to file the review. Here it is.

---

## Verdict: CHANGES_REQUESTED

The product shape — receipt-eligibility → resolve → delegate to immutable installer — is right and matches the "least new Rust code" goal stated in the Quality Bar section. But the metadata trust model, receipt-schema ownership, and a few safety-critical specs are underspecified or wrong in ways that compromise the update-safety bar before any Rust line is written.

---

## Blocking findings

### B1. Metadata-URL injection: `installer_url` in `channels.json` defeats checksum delegation
`ad-hoc-sifr-self-update.md:174-196` ("Update Metadata Contract") puts `installer_url` and `artifact_base_url` *inside* the unauthenticated metadata files. Combined with `self_update_runner.rs` "download the resolved immutable installer" (line 213), this means an attacker who can modify metadata (compromised CDN, MITM where TLS verification is weak, accidentally checked-in test fixture published) can redirect users to an attacker-supplied installer. The spec's mitigation — "Checksums remain embedded in the immutable installer and verified by the installer" — does not apply, because the attacker now controls *which* installer runs and which checksums are checked against.

**Required fix:** the CLI must derive `installer_url` from a hardcoded constant base + the resolved version string, e.g. `https://sifr.sh/install/versions/<version>`. Metadata is allowed to record only channel→version *strings*. Optionally add a `sha256` over the immutable installer in `versions/<version>.json` and verify post-download — but the constant URL is the load-bearing defense.

### B2. Backward-compat default of `channel = beta` silently migrates alpha users
`ad-hoc-sifr-self-update.md:139-142` and Locked Decision #6 (line 75) both fall back to `"beta"` when the receipt lacks `channel`. The existing receipt schema written by `scripts/distribution/generate_version_installer.sh:302-313` does not record `channel`, so every currently installed alpha user qualifies for that fallback. On their next `sifr self update` they would silently switch update train from alpha to beta. The correct default is to parse the channel from the receipt's `version` prerelease label (`-alpha.N` → alpha, `-beta.N` → beta, `-rc.N` → rc) and only fail closed when even that doesn't disambiguate. No global "default beta" fallback.

### B3. Receipt schema has two owners with no binding contract
`scripts/distribution/generate_version_installer.sh:302-313` writes the receipt. `ad-hoc-sifr-self-update.md:116-135` defines a Rust-side schema with six new fields. milestone_self_update_1 says "Extend generated installer receipts" but doesn't lock the schema to a single source of truth. With two implementations (bash template, Rust struct), the next installer regeneration will drift one or both fields. Required: the schema lives as a JSON Schema (or equivalently strict spec) under `verification/distribution/`, with milestone_1 producing a generator-output snapshot test AND a Rust round-trip test that both reject any deviation.

### B4. Eligibility comparison is underspecified — symlinks will break it
`ad-hoc-sifr-self-update.md:153-159` requires `canonicalize(current_exe) == canonicalize(receipt.binary_path)` and then "refer to the same filesystem entry." Path canonicalization on Linux/macOS does *not* always survive bind mounts, hardlinks across volumes, or sandboxes; and `current_exe()` on macOS sometimes returns the unresolved invocation path. The realistic install layouts include `~/.local/bin/sifr → ~/.sifr/bin/sifr` symlinks created by the user. Spec needs to commit to one of: (a) inode+dev equality via `stat`, with `fs::canonicalize` as the precondition only; (b) the contract that the receipt's `binary_path` *is itself* canonicalized at install time and that the CLI canonicalizes only `current_exe`. Without this lock, the eligibility check will reject legitimate installs.

### B5. Stable-channel gating is not robust to forward-dated metadata
`ad-hoc-sifr-self-update.md:100-101, 281-282` reject `stable` on input and on generation, but the CLI is permanent — a Phase-39 metadata file containing a `stable` channel could reach a pre-39 client. The CLI must refuse to act on any metadata that exposes a `stable` channel or a stable-looking version, regardless of whether the user invoked the stable path. As written, an alpha user with the new CLI fetching a post-39 `channels.json` could plausibly still parse a `stable` entry into memory; the spec must explicitly require rejecting the whole document or the offending channel.

### B6. Drift checks miss the immutable installer's embedded version
`ad-hoc-sifr-self-update.md:278-283` mandates dispatcher↔metadata agreement, but not metadata↔`APP_VERSION`-embedded-in-installer↔GitHub release tag. `generate_version_installer.sh:120` writes `APP_VERSION="${VERSION}"` into each immutable installer file. The drift checker must extract that variable from the installer file and confirm it matches metadata for the same URL/version. Otherwise a regenerated dispatcher could point at a regenerated installer with stale `APP_VERSION` and no test catches it.

### B7. No mutual exclusion around the running update
The runner section (`ad-hoc-sifr-self-update.md:215-225`) describes download + run, but two concurrent `sifr self update` invocations can race on `install_dir/sifr`. The bash installer's `mv tmp → sifr` is atomic, but `install.json` is rewritten non-atomically (`generate_version_installer.sh:302-313`), and two installers in parallel can interleave to produce a binary from one version with a manifest from another. Spec must require a `flock` (or `O_EXCL` lock file) on the install directory before the runner invokes the installer.

---

## Non-blocking polish

- **N1. Channel-switching should require `--force`.** `--channel alpha` against a beta receipt updates the receipt's `channel` post-install and changes the user's future no-arg `sifr self update` train. Treat as a downgrade-class mutation.
- **N2. Drop `installer_url` and `artifact_base_url` from the receipt.** Receipts should record state-of-install (version, target, channel, install_dir, binary_path, modify_path), not how-to-update. Self-update derives URLs from current metadata + constants. Removes a drift surface and rhymes with the B1 fix.
- **N3. Freeze the `sifr self version --format json` schema.** The spec says "deterministic"; pin it: `schema_version`, exhaustive field set, field types, key ordering, and a snapshot test. Otherwise the JSON shape becomes whatever serde happens to emit per release.
- **N4. Receipt discovery rule 3 (`ad-hoc-sifr-self-update.md:148`) needs canonicalization before equality** against `~/.sifr/bin/sifr`, or it falls over on symlinked invocation paths. Or just drop rule 3 — rule 2 (`<current_exe_parent>/install.json`) is the strong gate.
- **N5. Specify HTTP client / TLS policy.** No insecure-skip-verify; commit to rustls + webpki-roots or system roots. For a self-updater this is not implementation detail.
- **N6. Specify download-then-rename for the immutable installer.** A truncated download piped to `sh` runs a partial script with unpredictable effects. Atomic temp-file write + size sanity check + rename, then exec.
- **N7. `modify_path` backward compat is asymmetric and brittle** (`ad-hoc-sifr-self-update.md:142`). Cleaner: missing `modify_path` is always a `self version` warning, and on update simply *omit* `SIFR_NO_MODIFY_PATH` rather than erroring — the installer's default kicks in and existing preview users are not locked out.
- **N8. `cli_model_and_entrypoint.rs` is 849 lines today** (AGENTS.md guardrail: 900). The proposal correctly says "without expanding `cli_model_and_entrypoint.rs` into a monolith" but the milestone_2 PR will breach if the `Self` subcommand structs land in that file. Require structs to live entirely in `self_update_cli.rs`; entry file gets only registration + dispatch.
- **N9. Update `internal_docs/distribution_pipeline.md` in milestone_1, not milestone_5.** That doc is the existing Phase-33 spec for the substrate being extended; deferring it to the docs milestone leaves the schema-of-record floating across two issues for three milestones.
- **N10. Validation contract should name negative receipt-parsing cases explicitly:** empty file, invalid JSON, wrong types per field, extra unknown fields (forward-compat preserve-vs-reject decision must be stated). Currently captured loosely as "receipt parsing for Phase 33 and new schema shapes."
- **N11. Roadmap row 37.1** correctly links both issue files and is appropriately positioned between Phase 37 and Phase 38; no change needed there beyond updating its status as milestones land.

---

## Recommended simpler architecture

Collapse the two metadata files into one, and remove all URL fields from both metadata and receipts:

1. **One metadata file:** `https://sifr.sh/install/metadata/channels.json` — schema is just `{ schema_version, channels: { alpha: "0.1.0-alpha.N", beta: "0.1.0-beta.N" } }`. No URLs, no targets, no installer hashes. (If hash pinning is added later for B1, it lives in a separate, server-signed `versions/<version>.json` whose only job is to publish the SHA — never the URL.)
2. **Constants in the Rust binary:** `INSTALL_BASE_URL = "https://sifr.sh/install"` and `INSTALLER_URL_TEMPLATE = "{base}/versions/{version}"`. Both are compile-time strings, overridable only by a build-time env var used by integration tests.
3. **Receipt records install state only:** `schema_version`, `name`, `version`, `channel` (derived from version prerelease at install time so it's always present), `target`, `install_dir`, `binary_path` (canonicalized), `modify_path`. No `installer_url`, no `artifact_base_url`.
4. **Self-update flow:**
   - Parse args → discover receipt → eligibility check (no network).
   - If `--version`, that's the target. Else fetch `channels.json`, resolve `channel → version`.
   - Same-version: no-op unless `--force`. Downgrade or channel-switch: require `--force`.
   - Dry-run prints the plan and exits.
   - Real: `flock` on `install_dir`, download `INSTALLER_URL_TEMPLATE.format(version)` to temp with atomic rename, `chmod 755`, `Command::status()` it with `SIFR_INSTALL_DIR`, `SIFR_INSTALL_MANIFEST_DIR`, `SIFR_NO_MODIFY_PATH` (when applicable), `--force` (when applicable).
   - The immutable installer continues to own target detection, archive checksum, extraction, and atomic `mv`.

This keeps the Rust crate to four small modules (`self_update_cli`, `self_update_receipt`, `self_update_metadata`, `self_update_runner`) as proposed, but removes the URL-injection surface (B1), the dead receipt fields (N2), and an entire metadata file class.

---

## What looks good

- Receipt-eligibility-before-network ordering is correct.
- Delegating extraction/checksum/PATH-edits to the existing installer instead of duplicating in Rust is the right call and is consistent with the Quality Bar.
- Stable-gating discipline (including rejecting stable metadata generation) is appropriately strict for pre-Phase-39.
- The non-goals list is well-bounded and explicitly defers Windows, telemetry, daemons, and rollback policy.
- Milestone sequencing puts the receipt/metadata contract before any CLI code, which is the only order that lets backward-compat be tested rather than retrofitted.
