## Verdict

**No blockers.** All three pass-1 non-blocking concerns that this revision targeted are correctly fixed. Two pass-1 items were explicitly deferred (entrypoint-vs-root breadth; per-root variant dedup); they remain as documented follow-ups.

**Another review round is not required.** Land as-is.

## Pass-1 fixes — verified

1. **`project_source_checksum` now tracked-file-only.** `oss_and_determinism.py:296-316` shells `git ls-files -- <project_root>`, sorts the result, and digests `path-relative-to-project-root \0 bytes \0` per tracked file. I re-implemented the exact algorithm against the working tree and got identical 64-char hex for all four roots — matching the manifests at `curated_manifest.json:8,32,56,80,104` and `ecosystem_broader_manifest.json:8,32,50,74`. `.git`, untracked drafts, and editor noise are excluded. Empty `git ls-files` output yields the SHA-256 of the empty input (`e3b0c…`), which will not match any real manifest entry — i.e. a non-existent or fully-untracked root surfaces as a checksum mismatch, not a false pass.
2. **SPDX allowlist enforced.** `ALLOWED_SPDX_LICENSES = {"MIT"}` at `oss_and_determinism.py:20`; the membership check at `:92` runs inside `required_missing`, so a missing or non-allowlisted value surfaces as a `license` mismatch in the metadata variant and `continue`s before commands execute. Matches the policy promise at `ecosystem_compatibility.md:33-37`.
3. **LICENSE artifacts present and locked.** `LICENSE` exists in all four project roots and is included in the tracked file set returned by `git ls-files`, so any future edit will invalidate the checksum. Confirmed via `git ls-files` listing.
4. **Manifest checksums refreshed.** All five curated and four broader entries now carry `source_checksum_sha256` matching disk.
5. **Pin durability across squash merge** (relevant to deferred concern 1): All four roots already had pre-Wave-9.2 tracked files (`main.sifr`, etc.), and `project_revision_history` returns `f6ababa5dd…` for every one of them. Because this revision adds entrypoints inside existing roots — not new roots — the pin contract survives squash merge.

## Non-blocking observations from this pass

1. **Policy/implementation wording drift.** `ecosystem_compatibility.md:28` still reads "the runner hashes every regular file under `project_root` in sorted repo-relative order." After the pass-1 fix the walker is `git ls-files`, so the doc should read "every git-tracked file" (or similar) to match runtime behavior. One-line wording fix; not blocking.
2. **LICENSE file contents are the SPDX identifier (`MIT\n`), not the MIT license text.** For first-party fixtures this is fine for the runner's contract (which validates the manifest field, not the file body), and the checksum locks whatever is there. If the LICENSE files are intended as real OSS legal artifacts they should carry the full MIT text + copyright/permission notice; if they're only intended as SPDX-marker files, calling that out in `ecosystem_limitations.md` would avoid future confusion. Either way: non-blocking, follow-up cleanup.
3. **`is_sha256_hex` only accepts lowercase hex** (`:293`). `hashlib.sha256().hexdigest()` always returns lowercase, so all in-tree manifests pass; uppercase manifest values would be rejected. Acceptable strict-form behavior.

## Deferred items (acknowledged, not addressed in this revision)

- New `project_root` breadth — explicit non-goal for this slice; framed as entrypoint expansion within pinned roots. Honest framing in the policy doc.
- Per-root variant de-duplication — variant inflation (pinned+checksum re-emitted per entry sharing a root) is known and non-blocking; deferred to a follow-up.

No additional review pass required. Merge.
