All eight files inspected. Verdict below.

## Review pass 2: reconstructed Sifr 0.1.0 stable release notes

**Inspected**
- `/private/tmp/sifr-phase40-candidate-rebuild.s49VOp/work/release-notes.md` (79 lines, up from 36)
- `plans/reviews/active/phase-40-candidate-release-notes-review-pass-1.md`
- `plans/phases/40_stable_channel_ga_promotion_and_release_governance.md` (§Supported release targets L110–121, §Canonical governed release index, plan bindings L250–283, integrity boundary L343–347, retention floor L884–890)
- `work/qualification-artifact-index.json` (run `30416219284`, source `c9d611fb…cae9`, `schema_version: 2`, expiry `2026-08-28`)
- `qualification/…-editor/qualification-editor.json`
- `docs/releases/0.1.0.mdx`, `docs/releases/stable.mdx`, `docs/releases/compatibility.mdx`, `docs/rust-interop.mdx`

### Pass-1 defect resolution

| # | Pass-1 defect | Status |
|---|---|---|
| 1 | Receipt claimed as SHA-256-bound | **Resolved.** L14–16 now digest-binds archive, sysroot, checksum, installer, VSIX, report; receipts are separately described as "checked for version and channel agreement," matching plan L260–261. VSIX added, consistent with `vsix` / `editor-qualification-report` entries in the index. |
| 2 | Missing platform floors | **Resolved.** L9–12 state macOS 15.0 for both `*-apple-darwin` and glibc 2.39 for both `*-unknown-linux-gnu`, matching `compatibility.mdx:11–16` and satisfying the L120–121 contract requirement verbatim in scope. |
| 3 | Integrity described at wrong boundary | **Resolved.** L14–20 now cover both boundaries: qualification-time digest verification of every candidate artifact, and install-time verification (dispatcher verifies the immutable installer's SHA-256 from the governed index; installer verifies target archive and sysroot before replacing a toolchain) — matches `stable.mdx:40–42` and `0.1.0.mdx:17–18`. |
| 4 | No install/update information | **Resolved.** L22–49 give the default entrypoint (noted as selecting stable), the explicit `/install/stable` form, the `--version 0.1.0` pin in the exact form used at `stable.mdx:37`, `sifr self update`, `schema_version: 2`, and rejection of unknown, withdrawn, and `X.Y.Z-rc.N` versions. |
| 5 | Generated-Rust exclusion too narrow | **Resolved.** L69–71 reproduce the recorded exclusion — `sifr emit` **and** the VS Code generated-Rust preview action, outside the packaged `0.1.0` GA-qualified surface, with the recorded reason (cold first-run qualification exceeded its deterministic bound) — matching `0.1.0.mdx:54–56`. The prior affirmative list of governed surfaces that implied `sifr emit` was qualified is gone. |
| 6 | Rust interop limits omitted | **Resolved.** L73–76 scope stable interop to the claims table, state that `contract-only` rows do not claim runtime-observed support, and that future-owned runtime rows are unadvertised — matching `compatibility.mdx:32–38` and `rust-interop.mdx:72–83`. Anchor `#stable-support-claims` resolves to `rust-interop.mdx:75`. |
| 7 | Channel context missing | **Resolved.** L78 states alpha and beta remain explicit preview channels and `rc` is not a public channel. |

Non-blocking improvement 2 from pass 1 was also adopted (L60–63 names the out-of-band recovery path).

### Overclaim check on the new text

- Targets/floors, extension identity `sifr.sifr-vscode` 0.2.0, range `>=0.1.0,<0.2.0`, and "reuses the exact qualified VSIX without rebuilding" all match `qualification-editor.json` (`publisher: sifr`, `rebuild: false`, identical `vsix_sha256` in both files).
- Recovery model matches `rollback_version: "none"`: no eligible predecessor, incident roll-forward. No rollback target is implied as available.
- Signing and notarization are disclaimed, not claimed — compliant with plan L346–347. No signing/provenance-attestation language crept in with the new integrity paragraph.
- Every digest kind enumerated at L14–15 has a corresponding entry in the artifact index; none is invented.
- Windows, package-manager distribution, and `rc` remain unclaimed.

No new factual overclaim. Two cosmetic points, neither blocking and neither requiring a change before evidence capture:
- "aggregate installer" (L15) is loose: the index binds `sifr-installer-0.1.0` (`kind: installer`) plus `checksums.txt` (`kind: checksums`) as the aggregate manifest. The claim is true either way; "installer and aggregate checksum manifest" would read more precisely.
- L63 says `--force` is reserved for "approved downgrade" where `stable.mdx:55–56` says "approved rollback." Same policy, and it is stated as reserved purpose rather than as an available path for 0.1.0, so it does not contradict L59–60.

Scope respected: no Rust interop implementation change, demo, or algorithm work is requested.

SATISFIED
