## Review: reconstructed Sifr 0.1.0 stable release notes

**Inspected**

- `/private/tmp/sifr-phase40-candidate-rebuild.s49VOp/work/release-notes.md` (36 lines)
- `plans/phases/40_stable_channel_ga_promotion_and_release_governance.md`
- `work/qualification-artifact-index.json` (run `30416219284`, source `c9d611fb…cae9`, `schema_version: 2`, expiry `2026-08-28`)
- `qualification/…-editor/qualification-editor.json` (package `0.2.0`, range `>=0.1.0,<0.2.0`, `rebuild: false`, `rollback_version: "none"`)
- `docs/release-scope-and-limitations.mdx` **does not exist**. The canonical public release-scope/limitations surface is `docs/releases/0.1.0.mdx` ("Known boundaries") plus `docs/releases/compatibility.mdx`, `docs/releases/stable.mdx`, and `docs/support.mdx`. Reviewed all four.
- `docs/rust-interop.mdx` (§Stable Support Claims + generated claims table)

### What is correct

- Four qualified targets match the contract and the qualification index exactly.
- Extension identity `sifr.sifr-vscode` 0.2.0, range `>=0.1.0,<0.2.0`, and "reuses the exact qualified VSIX without rebuilding" match `qualification-editor.json` (`publisher: sifr`, `extension: sifr-vscode`, `rebuild: false`) and `milestone_40_5`.
- Recovery model is accurate: no stable predecessor, `rollback_target: none`, incident roll-forward as the recovery path.
- Correctly disclaims binary signing and notarization, as the phase contract requires ("Public docs must not claim cryptographic signing or notarization").
- No overclaim of a rollback target, no Windows/package-manager claim, no `rc` claim.

### Blocking defects

**1. Unsupported claim: install receipts are not SHA-256-bound.** Line 14–15 says "Every target archive, sysroot, checksum, installer **receipt**, and aggregate installer is verified by exact SHA-256 before publication." The release plan binds "binary, sysroot, archive, checksum, and installer digests" and separately requires *receipt version/channel agreement* — a field-agreement check, not a digest. No receipt digest exists in the qualification index. The claim asserts integrity coverage that no evidence supports.

**2. Missing platform floors — direct contract violation.** Phase 40 §Supported release targets: "Public documentation must state the supported targets **and the minimum macOS/Linux ABI or OS floor** established by the release builders." The notes list bare target triples. `docs/releases/compatibility.mdx` and `docs/releases/0.1.0.mdx` both state macOS 15.0 and glibc 2.39 (consistent with `macos-15` / `macos-15-intel` / `ubuntu-24.04*` builders). As written, a macOS 13 or glibc 2.35 user reads the notes as supported.

**3. Checksum enforcement is described at the wrong boundary.** "verified by exact SHA-256 **before publication**" describes only pre-publication qualification and omits the user-facing enforcement that is Phase 40's actual GA integrity boundary: the dispatcher verifies the immutable installer's SHA-256 from the governed release index before execution, and the installer verifies the selected target archive and sysroot before replacement (contract §Canonical governed release index; `stable.mdx:40–42`; `0.1.0.mdx:17–18`). Since the notes simultaneously disclaim signing, this omission leaves readers with no stated integrity guarantee for their own download. The VSIX is also digest-bound in the qualification index but absent from the enumerated list.

**4. No installation or update information at all.** For release notes checked in as candidate evidence and mirrored publicly, the notes never mention `https://sifr.sh/install` (which defaults to stable), `/install/stable`, version pinning, `sifr self update`, the governed index at `schema_version: 2`, or that unknown, withdrawn, and `X.Y.Z-rc.N` versions are rejected. Every one of these is in `0.1.0.mdx` and `stable.mdx`. Installation clarity is absent, not merely thin.

**5. Generated-Rust exclusion is narrower than the public doc.** The notes exclude only "Packaged generated-Rust preview." `docs/releases/0.1.0.mdx:54–56` excludes "Generated-Rust output through **`sifr emit`** and the VS Code preview action … because cold first-run qualification exceeded its deterministic bound." The notes then affirmatively list governed surfaces ("initialization, diagnostics, formatting, building, running, testing, installation, self-update"), so a reader plausibly concludes `sifr emit` is qualified. The notes must not be less restrictive than the checked-in GA doc.

**6. Rust interop limits are omitted entirely while the notes assert blanket governance.** `0.1.0.mdx` records two Rust boundaries (claims limited to the stable claims table; future-owned runtime work unadvertised). Per `docs/rust-interop.mdx`, `zero_copy_bytes`, `zero_copy_view_matrix`, `arrow_record_batch`, `tensor_dlpack_bridge`, `advanced_data_matrix`, `panic_boundary`, `panic_abort_profile`, `async_runtime_core`, `callbacks_threadsafe`, and `callback_subscription_core` are `contract-only`, and `advanced_data_runtime_matrix` / `zero_copy_runtime_matrix` are future-owned. Saying "building, running, testing … remain governed by the stable qualification evidence" with no interop qualifier reads as full-surface runtime coverage — exactly what the Quality Contract ("Public Rust interop claims never exceed the compatibility matrix") forbids by omission.

**7. Channel context missing.** Neither "alpha and beta remain explicit preview channels" nor "`rc` is not a public channel" appears, though both are contract invariants and are stated in `0.1.0.mdx:51`, `stable.mdx:63`, and `support.mdx`.

### Required corrections

1. Delete `installer receipt` from the SHA-256 list; if receipt integrity is mentioned, describe it as receipt version/channel agreement. Add the VSIX to the digest-bound list.
2. State the platform floors: macOS 15.0 for both `*-apple-darwin` targets, glibc 2.39 for both `*-unknown-linux-gnu` targets (or link `/releases/compatibility` and state them there-and-here).
3. Rewrite the integrity sentence to cover both boundaries: qualification-time digest verification of every candidate artifact, **and** install-time verification — dispatcher verifies the immutable installer's SHA-256 from the governed release index; the installer verifies the target archive and sysroot before replacing a toolchain.
4. Add an installation/update section with `curl -fsSL https://sifr.sh/install | sh` (defaults to stable), the explicit `/install/stable` form, the `--version 0.1.0` pin, `sifr self update`, and the statement that the governed index uses `schema_version: 2` and rejects unknown, withdrawn, and `X.Y.Z-rc.N` versions.
5. Replace "Packaged generated-Rust preview" with the exact recorded exclusion: generated-Rust output through `sifr emit` and the VS Code generated-Rust preview action is outside the packaged `0.1.0` GA-qualified surface, with the recorded reason (cold first-run qualification exceeded its deterministic bound).
6. Add a Rust interop boundary: stable Rust interop support is limited to the categories and evidence scopes in the stable claims table (`/rust-interop#stable-support-claims`); `contract-only` rows do not claim runtime-observed support, and future-owned runtime rows are not advertised.
7. Add: alpha and beta remain explicit preview channels; `rc` is not a public channel.

### Non-blocking improvements

- Note that the extension range `>=0.1.0,<0.2.0` also covers any eligible `0.1.x` rollback target (`compatibility.mdx:25–26`), which is why it is a range rather than an exact version.
- Under Recovery model, name the out-of-band recovery path (`/install/stable` with `--force`) for a binary that cannot run self-update, and that a forced downgrade requires explicit consent.

NOT SATISFIED
