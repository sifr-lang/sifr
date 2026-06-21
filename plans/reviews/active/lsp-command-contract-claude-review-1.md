Here are the findings.

## Blocking

### 1. `String(diagnostic.code)` is `"[object Object]"` for every Sifr diagnostic — explainDiagnostic silently always misses
`editor_integrations/vscode/src/commands.ts:107-110`

```ts
arguments: [String(diagnostic.code)],
```

Sifr's diagnostic renderer always emits `codeDescription.href` (`crates/sifr_diagnostics/src/codes/registry.rs:314` — `docs_url()` returns `format!("https://docs.sifr.sh/errors/{}", self.code())`, never empty). When `codeDescription` is present, `vscode-languageclient` rewrites `Diagnostic.code` into the object form `{ value, target: Uri }` (`node_modules/vscode-languageclient/lib/common/protocolConverter.js:73-78`). `String({value, target})` is `"[object Object]"`, so the server-side lookup at `crates/sifr_analysis/src/host/implementation.rs:591` (`rendered.code == diagnostic.code`) never matches and `crates/sifr_lsp/src/commands.rs:54-57` returns `"diagnostic code [object Object] is not present in the current workspace snapshot"`. The whole point of this PR is to make the round-trip work; today every real invocation breaks. The static `npm run test:extension` smoke didn't catch it because it only checks command registration.

Fix shape:

```ts
const rawCode = diagnostic.code;
const code =
  typeof rawCode === "object" && rawCode !== null
    ? String((rawCode as { value: string | number }).value)
    : String(rawCode);
```

### 2. No runtime coverage for `sifr.server.explainDiagnostic`
`verification/areas/developer_tooling/lsp_protocol_smoke.py:161-167`

The smoke removed the `sifr.runTests` call but only added `sifr.server.showGeneratedRust`. There is no `executeCommand` call that exercises explainDiagnostic against a real published diagnostic with `codeDescription` set. Adding one would have surfaced finding #1, and would lock the argument schema for future drift.

## Should fix

### 3. Doc/version bump beta.10 → beta.11 is out of scope for this PR
`docs/installation.mdx:54,65,118,126`, `docs/self_update.md:44,59`, `editor_integrations/vscode/README.md:33`.

The task is the LSP command contract fix; these bumps claim a `0.1.0-beta.11` release that does not exist yet (latest tag is beta.10 — commit `5cf47f086`). If beta.11 is being staged in this PR intentionally, fine, but the title and CHANGELOG don't say so. If not, separate the doc bumps so the command-contract fix can ship without coupling to an unreleased CLI version. The vscode `0.1.4` README also pre-binds itself to a CLI version that isn't yet shipped — fragile.

## Non-blocking observations / nits

### 4. The transcript-replay tightening is the right call
`verification/areas/developer_tooling/check_lsp_transcript_replay.py:128`. Flipping `expected.difference(commands)` to `set(commands) != expected_commands` catches drift in both directions — extras now fail too, which is exactly what we want now that the advertised set is supposed to be minimal. Worth keeping.

### 5. `executeCommand` `_ =>` arm vs. the contract
`crates/sifr_lsp/src/commands.rs:22-24`. Returning `method_not_found` for any unadvertised command is correct LSP behavior. Worth a one-line unit test that calling a no-longer-advertised name like `"sifr.runTests"` errors — cheap insurance against accidental re-introduction.

### 6. capabilities test pins one encoding only
`crates/sifr_lsp/src/capabilities.rs:138-148`. Since the command list doesn't vary by encoding, a single case is fine. Optional: parametrise over `(client_supports_pull_diagnostics, encoding)` so a future regression that adds an encoding-conditional command would be caught.

### 7. Argument schema dropped `range` for `showGeneratedRust`
The old call passed `{uri, range}`; the new call passes `[uri.toString()]`. `snapshot.generated_rust_preview(host, file, None)` already ignored the range (third arg hard-coded `None`), so no behavior loss — but worth a note in `internal_docs/lsp_server.md` that the argument schema is `[uri: string]` so future contributors don't reintroduce range plumbing.

### 8. Dead types in `sifr_analysis`
`TestItem`, `TestCommand`, `TestItemId`, and `TestCommandKind` are still exported from `crates/sifr_analysis` but no longer reached via LSP. If nothing else surfaces them, a follow-up can prune them. Out of scope for this PR — flagging only.

### 9. Namespacing choice is good
`sifr.server.*` keeps the LSP surface portable across editors and the invariant ("editor UI commands must not share IDs with advertised LSP commands") is documented in both `internal_docs/lsp_server.md:163-168` and `internal_docs/vscode_extension.md:99-102`. No collision risk for the current command set.

## Summary

The contract design (server owns `sifr.server.*`, editor owns `sifr.*`), the fixture/protocol-matrix sync, and the inventory/rules lock updates are all consistent and correct. The blocking issue is the VS Code-side argument conversion in finding #1, plus the missing runtime smoke that would have caught it. Recommend fixing #1, adding a real explainDiagnostic round-trip to `lsp_protocol_smoke.py`, and deciding whether the beta.11 doc bump belongs in this PR before landing.
