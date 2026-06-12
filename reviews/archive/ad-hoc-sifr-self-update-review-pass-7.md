# Pass-7 Review: Ad Hoc Sifr Self Update

Scope: verify pass-6 `CHANGES_REQUESTED` findings are closed against the current `issues/ad-hoc-sifr-self-update.md` and `internal_docs/phases/39_stable_channel_ga_promotion_and_release_governance.md`. Pass-6 review is `reviews/ad-hoc-sifr-self-update-review-pass-6.md`.

## Verdict: READY

## Pass-6 findings — verification

### 1. MED — Dry-run JSON output specification → FIXED

- `issues/ad-hoc-sifr-self-update.md:88` now lists `[--format text|json]` directly on the `self update` command signature.
- `:97` constrains `--format` to dry-run only: real updates always preserve installer stdout/stderr as human output.
- `:104` rejects `--format` without `--dry-run` as an invalid combination.
- `:110` makes dry-run obey the same `--force` rules as a real update (no same-version reinstall, downgrade, or channel-switch plan is printable without `--force`).
- `:112–128` locks the complete JSON shape with `schema_version: 1`, `current_version`, `target_version`, `receipt_channel`, `requested_channel`, `resolved_channel`, `install_dir`, `binary_path`, `installer_url`, `action`, `force`, `would_run_installer`, and `warnings`.
- `:130–136` enumerates field requirements: schema_version pinned to 1 until a reviewed bump, `requested_channel` null-vs-absent behavior, the `action` enum (`no_op`, `update`, `reinstall`, `downgrade`, `channel_switch`), `would_run_installer` only false for `no_op`, and snapshot tests cover names, ordering, types, warning ordering, and absent-vs-null.
- M2 unit tests at `:343–345` cover dry-run output in text and JSON formats, and explicit rejection of `--format` without `--dry-run`.

No implementation discretion remains; this is the contract.

### 2. MED — Installer download minimum-size threshold → FIXED

`issues/ad-hoc-sifr-self-update.md:284` reads: "reject downloads smaller than 1024 bytes and files whose first line does not start with `#!` before execution." 1024 bytes is the contract; the validation test has a concrete oracle.

### 3. MED — Immutable installer `--force` sequencing → FIXED

- M1 scope `:418` explicitly adds `--force` flag handling to the immutable installer template so the runner can delegate force semantics.
- M1 definition of done `:433` requires immutable installer tests that prove `--force` is accepted and preserves existing force semantics.
- M3 scope `:464` is now downstream of M1 — "Pass `--force` through to the immutable installer when requested, relying on the immutable installer template support added in milestone 1." Sequencing seam closed.

### 4. LOW — `rc` rejection in invalid combinations → FIXED

`issues/ad-hoc-sifr-self-update.md:106` now lists "`rc` channels and `-rc.N` version pins are rejected before Phase 39" alongside the stable rejection bullets, restoring symmetry with diagnostics (`:308`), human remediation (`:325`), unit tests (`:341`), and M2 DoD (`:454`).

### 5. LOW — Diagnostic family carve-out → FIXED

`issues/ad-hoc-sifr-self-update.md:297` now reads: "Self-update diagnostics use `SIFR-BUILD-09xx` in this phase. A dedicated CLI diagnostic family is out of scope for this ad hoc phase and requires a later reviewed planning change." The range is locked for the phase; any family change is out of scope rather than implicitly tolerated.

### 6. LOW — Phase 39 schema-bump symmetry → FIXED

`internal_docs/phases/39_stable_channel_ga_promotion_and_release_governance.md:54` now states: "Keep the ad hoc self-update receipt schema, `self version` JSON schema, and `channels.json` schema at `schema_version: 1`; stable activation changes the governed allowlist and accepted version classes, not field shapes." The stable activation tests requirement at `:62` reinforces that schema-version `1` metadata and receipts are what accept `stable` after the governed allowlist is updated. Phase 39 readiness no longer leaves a schema-bump question open.

## Rationale

The phase contract is implementation-ready under the "everything is unstable; decide now" directive:

- Every flag, rejection, schema field, byte threshold, and lock path is pinned to a concrete value with snapshot or integration-test coverage attached.
- No backward-compatibility shims survive: pre-schema receipts fail closed, the receipt schema is binary (required fields, unknown-field rejection), and unstable preview installs are explicitly allowed to break.
- Trust boundaries are tight: installer URLs derive from compile-time constants only; metadata carries no URLs and no checksums; channel allowlist is exactly `alpha` and `beta` until Phase 39 governs `stable`.
- Sequencing is unambiguous: M1 ships the installer `--force` capability, the receipt schema, the metadata generator, and the manifest atomic-write/lock changes; M2 layers eligibility and dry-run; M3 delegates to the installer; M4 wires drift checks; M5 closes docs and release readiness.
- Phase 39 inherits the same safety model and the same `schema_version: 1` surface, so stable activation is a governed allowlist flip rather than a schema renegotiation.

Recommend promoting to implementation. The execution checklist at `issues/ad-hoc-sifr-self-update-execution.md` should be updated to record pass-7 as the final planning review before milestone work begins.
