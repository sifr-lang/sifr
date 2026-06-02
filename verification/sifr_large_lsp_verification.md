# Large LSP Verification Submodule

`verification/sifr-large-lsp-verification` is a git submodule pointing at the
public repository `https://github.com/sifr-lang/sifr-large-lsp-verification`.

The subrepo contains synthetic generated Sifr code for long-session LSP
verification. It is not a demo or idiomatic application corpus.

After a fresh checkout, initialize submodules with:

```bash
scripts/clone_subrepos.sh
```

or directly:

```bash
git submodule update --init verification/sifr-large-lsp-verification
```

## Checks

Smoke mode is part of local validation:

```bash
verification/tooling/lsp_large_session.py --mode smoke
```

Full mode is a manual qualification check for explicit large-session review:

```bash
verification/tooling/lsp_large_session.py --mode full
```

Both modes write JSON evidence to `target/lsp_large_session/` by default. The
evidence includes request/edit latency samples, peak RSS, and an RSS growth
slope over the second half of the session.

The verifier initializes the LSP with `diagnosticsMode=off`. This keeps the
long-session workload focused on editor responsiveness across opens, edits,
hover, completion, references, semantic tokens, inlay hints, and workspace
symbol requests without blocking every notification on synchronous diagnostic
publication. Full mode still issues periodic `textDocument/diagnostic` pull
requests over edited files. Diagnostic publication remains covered by the
existing LSP protocol smoke and stress checks.

Current local evidence:

- Smoke: 42 operations, p95 5.645 ms, peak RSS 2.1 MiB.
- Full: 1702 operations, including 480 change notifications and 30
  `textDocument/diagnostic` requests, p95 6.363 ms, peak RSS 19.0 MiB, RSS
  slope 29.34393 MiB/min.

## Updating The Corpus

Update the submodule repository first:

```bash
cd verification/sifr-large-lsp-verification
python3 tools/generate_corpus.py generate
python3 tools/generate_corpus.py check
git commit -am "Update large LSP corpus"
git push
```

Then update the submodule pointer in the main repo.
