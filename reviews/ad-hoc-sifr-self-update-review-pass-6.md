I've completed the review. Here are my findings.

## Verdict: CHANGES_REQUESTED

The contract is dramatically tighter than pass-5 left it, but one genuine punt-to-implementation remains, plus a few smaller seams that the "ad hoc phase must decide the final contract" rule should close.

## Findings (severity-ordered)

### 1. MED — `sifr self update --dry-run` JSON output is undecided (real gap)

`issues/ad-hoc-sifr-self-update.md:312` lists as a unit test:
> dry-run output in text and JSON formats

But:
- The command contract at `:88` lists `[--channel ...] [--version ...] [--dry-run] [--force]` — **no `--format` flag** on `self update`.
- `--dry-run` semantics at `:96` describe only a text-shaped list of fields.
- No JSON schema is provided for dry-run output (only `self version` JSON is locked at `:120-133`).
- M2 scope at `:411` says "Implement dry-run output" without specifying JSON.

Implementation will be forced to (a) invent a `--format text|json` flag on `self update`, (b) invent a JSON schema, or (c) drop JSON dry-run. This is exactly the "engineering discretion where the phase should decide" the user flagged.

Required edit: either add `[--format text|json]` to the `self update` command contract at `:88` and lock a dry-run JSON schema (with `schema_version`, planned source/target/channel/installer URL/install dir, deterministic ordering) **or** delete "and JSON formats" from `:312` and clarify that dry-run text output is deterministic via snapshot.

### 2. MED — Minimum installer-download size threshold is undecided

`issues/ad-hoc-sifr-self-update.md:253`:
> reject empty downloads, downloads smaller than a documented minimum size, and files whose first line does not start with `#!` before execution

"a documented minimum size" — no value pinned, no location named (M1 says "Update `internal_docs/distribution_pipeline.md` with the … contract"). Implementation could land 256 bytes, 1 KiB, or 4 KiB and all would be defensible. This is the contract; pick a number.

Required edit: pin an explicit byte threshold (e.g. "≥ 1024 bytes") in the Rust CLI Architecture runner rules at `:253`, or pin where in `distribution_pipeline.md` it will be set so the validation contract test has a concrete oracle.

### 3. MED — `--force` pass-through still depends on the immutable installer accepting `--force`

`issues/ad-hoc-sifr-self-update.md:430` (M3 scope):
> Ensure the immutable installer accepts `--force` before the runner depends on `--force` pass-through.

Pass-5 already flagged this as a sequencing seam (non-blocking note). The phrasing still doesn't decide whether (a) Phase 33's installer template already accepts `--force` and M3 only verifies, or (b) M3 must add it. Since same-version-reinstall, downgrade, and channel-switch all require `--force` (decisions #9/#10/#13), the runner's correctness hinges on installer support.

Required edit: either (a) state "the Phase 33 immutable installer already accepts `--force`; M3 verifies pass-through" or (b) add an explicit M1 (or pre-M3) scope item: "Add `--force` flag handling to the immutable installer template." Don't leave both possibilities open.

### 4. LOW — `rc` rejection asymmetry in the command contract

`rc` channels and `-rc.N` version pins are explicitly mentioned in Diagnostics (`:277`), human remediation (`:294`), unit tests (`:310`), and M2 DoD (`:420`) — but the "Invalid combinations" block at `:100-105` enumerates only `stable`/stable-looking/unknown-channel. A reader checking command semantics for `rc` won't find it next to the stable rejection.

Required edit: add an "rc channels and `-rc.N` version pins are rejected before Phase 39" bullet to the Invalid combinations list at `:103` for symmetry with the diagnostics taxonomy.

### 5. LOW — Diagnostic family ownership is a deferred decision (acceptable but flag-worthy)

`:266`:
> Self-update diagnostics reserve `SIFR-BUILD-09xx` … If implementation introduces a dedicated CLI diagnostic family, that taxonomy change must be reviewed before replacing this reserved range.

The carve-out is bounded and pass-5 accepted it. Worth noting because the user said decide the final contract — pre-allocating `SIFR-BUILD-09xx` while leaving the door open to "actually use a new CLI family" is the one place the contract explicitly tolerates a later structural change. Not blocking, but if you want zero punts, lock the family name now.

### 6. LOW — Phase 39 / ad-hoc schema-bump symmetry is not described

Phase 39 milestone_39_4 says it will "Update `sifr self update` to accept stable channel metadata and stable-looking version pins." The ad-hoc phase locks `schema_version: 1` on the receipt and on `self version` JSON output. When stable is added, will:
- `channels.json` schema_version bump? Locked at 1 in `:204`.
- `self version` JSON schema_version bump (it newly emits `channel: "stable"`)? Locked at 1 in `:122`.
- Install receipt schema_version bump? Locked at 1 in `:144`.

Phase 39 doesn't say. Either is fine — but the ad-hoc phase's "until a reviewed schema bump changes the contract" language at `:159` implies Phase 39 will be that reviewed change, and Phase 39 should explicitly say whether stable activation is or isn't a schema_version bump for each surface. Not blocking ad-hoc implementation; will be a Phase 39 readiness gap.

## What's right (no changes needed)

- No `may`/`should`/`either`/`TBD` wording survives in either file (greps clean).
- Phase 39 ↔ ad-hoc relationship is consistent: ad-hoc rejects stable until Phase 39 lifts the gate; Phase 39 explicitly inherits the preview safety model in its exit gate.
- Receipt validity is binary (no compat shims); `channel` and `modify_path` are required; pre-schema receipts fail closed.
- Installer URL derivation is locked to compile-time/`cfg(test)` only; runtime env vars cannot redirect.
- Manual installer and self-update share the named lock path `<install_dir>/.sifr-update.lock`.
- Architecture stays in the "eligibility + resolution + delegation" envelope; no Rust-side extraction/checksum/replacement re-implementation.

## Rationale

The contract is implementation-grade in 90% of its surface. Finding #1 is the only one I'd call a true "engineering discretion left open" — dry-run JSON is named as a test target with no schema and no flag in the command contract, which forces implementation to invent. Findings #2 and #3 are concrete numbers/sequencing that the contract should pin now, given the user's "decide now" directive. #4–#6 are polish.

Recommend a pass-6 edit that closes #1–#3 (and ideally #4), then promote to READY.
