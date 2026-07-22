All evidence is gathered, the scratch package is removed, and the repository is untouched. Here is the review.

---

# M14 Binding And Certification Authoring — Independent Full Review, Pass 4 (PR #2994, `codex/m14-python-binding-certification`)

Single-agent direct review of the complete committed diff `origin/main...HEAD` at candidate HEAD `f3a768c75` (62 files, +4,646/−346; commits `dfbf31532` feat, `dd7432d35` pass-1 remediation, `c3982bba1` ledger, `f3a768c75` pass-3 remediation), against `AGENTS.md`, `plans/issues/active/ad-hoc-declaration-first-python-interop.md:1948-2045`, both durable architecture contracts, and all three prior M14 review artifacts (pass 1 direct, pass 2, pass 3 frozen). No repository files were modified; live experiments ran in a scratch package under `/tmp` (removed afterward). The untracked pass-4 file at `plans/reviews/active/…-pass-4.md` is an empty pre-existing placeholder and was left untouched; this message is the review. The dirty `third_party/ruff` submodule pointer predates this review and was ignored, consistent with passes 1–3.

## Validation performed this session

- Rebuilt `target/release/sifr` from HEAD sources (no-op confirmed the binary was already fresh).
- Ran the blocking `binding-authoring` suite end-to-end: **passed** — `python interop binding authoring ok: sources=5 generated=4 untyped_failures=3 drift_checks=2 mutations=0` (31.6 s), matching the authoritative gate evidence and exercising all prior remediation regression cases plus the three new container-rejection cases.
- Focused tests: `sifr_driver` `python_binding` **7/7** (including the four new grammar tests), `sifr_package` python **63/63**.
- `cargo fmt --check` clean; `cargo clippy -p sifr_driver -- -D warnings` clean; `check_hir_maintainability_guardrails.py` **PASS**; `check_file_size_guardrails.py` **PASS** (2,782 files, 900-line cap).
- Accepted as known-good the stated fresh authoritative `scripts/run_all_tests.sh --profile create-pr` at this exact HEAD (1205.40 s; Python interop 18/18, E2E 131/131 signature `7c39b8c1dd4fec7c`, runtime platform 28/28, hardening 6/6, blocking budgets passed) — consistent with everything I re-verified independently.
- Live adversarial experiments in a scratch package against the frozen binary (details below).

## Pass-3 actionable Minor 1 — explicitly verified fully remediated

The remediation commit `f3a768c75` touches exactly the two implicated layers plus tests and honest ledger updates; no other code changed since the pass-3-verified `c3982bba1`.

**Code.** The probe now rejects bare `list`/`List`/`dict`/`Dict`/`tuple`/`Tuple` ("bare container annotation … requires type arguments", `crates/sifr/src/python_binding_probe.py:59-60`), all `set`/`Set` forms bare and subscripted (`:61-62,81-82`), and non-`str` dict keys ("direct-conversion dict keys must use str", `:83-87`). The Rust scaffold validator's token allowlist is replaced by a recursive grammar, `is_supported_direct_type` (`crates/sifr_driver/src/python_binding.rs:235-275`): scalars `None|bool|bytes|float|int|str`, bound class names, two-part unions with exactly one `None` and a non-optional direct other part, `list[T]` (one argument), `tuple[…]` (non-empty), and `dict[str, T]` — with `split_top_level` (`:277-305`) rejecting unbalanced brackets and empty parts.

**Grammar match with the compiler.** I compared this directly against the compiler's authority, `is_direct_type` (`crates/sifr_lowering/src/lower/python_interop.rs:687-714`). The productions reachable from probe output are isomorphic, including the `allow_option` threading (option allowed at top level and inside list/tuple/dict values; forbidden inside the non-`None` union arm). Where they differ, the bind-side is strictly *stricter* (empty tuple, degenerate `None | None`, record-classes and the `py.Object` contract that the probe can never emit) — every divergence fails closed at bind, never open.

**Live proof, acceptance direction** (the property pass 3 demanded: nothing `bind` accepts may be rejected by ordinary `check`): in a scratch package, an override with `dict[str, list[float]] | None`, `Union[None, int]` (reversed ordering), `Optional[list[tuple[int, str | None]]]`, and `dict[str, tuple[bool, bytes]] | None` → `sifr python bind math …` exits 0, and `sifr check --frozen` on the checked-in result reports **no errors found**; `sifr python bind --check` reports ok. The unit test `accepts_the_recursive_direct_conversion_grammar` (`python_binding.rs:524-538`) covers the same recursive shapes including bound classes.

**Live proof, rejection direction**: 12 adversarial overrides — bare `set`, bare `frozenset`, `frozenset[int]`, `dict[bytes, str]`, `int | str`, `Optional[Optional[int]]` (→ `int | None | None`), variadic `tuple[int, ...]`, `Sequence[int]`, bare `dict`, nested `list[set[int]]`, empty `tuple[()]`, bare `tuple` — all made `bind` exit 1 with an explicit unresolved/unsupported diagnostic, and a full before/after package hash confirmed **zero mutation**. Notably, cases that slip past the probe's identifier fallthrough (bare `frozenset`) or its union join (`int | str`, `int | None | None`) are caught by the independent Rust validator — the two layers genuinely back each other up. Exactly the pass-3 reproductions (`list` return, `set[int]` parameter, `dict[int, str]` return) are now permanent regression fixtures: unit tests `rejects_bare_list_direct_conversion_type` / `rejects_set_direct_conversion_type` / `rejects_non_string_dict_key_direct_conversion_type` (`python_binding.rs:508-521`) plus the suite's three `unsupported_containers.pyi` bind attempts with a snapshot-asserted non-mutation guard (`verification/areas/python_interop/runner/binding_authoring.py:196-240,318-323`) — both test artifacts pass 3 required. The contract sentence (`internal_docs/python_interop_declaration_architecture.md:577-581`) and the public claim (`docs/python-interop.mdx:196-199`) are now true as written.

**Remediated in full.**

## All earlier findings and dimensions — rechecked for regressions

Since `f3a768c75` changes only the probe, the scaffold validator, the suite, and planning Markdown, all machinery pass 3 verified at `c3982bba1` is byte-identical; I additionally spot re-verified live:

- **Integrity/mutation ordering**: validate-before-write and overwrite refusal unchanged (`python_binding_cli.rs:189-206,311-350`); the suite's environment-drift, output-collision, user-owned-output, and positional-only non-mutation cases all re-ran and passed this session, and my 12 failure probes confirmed zero mutation independently.
- **Confinement**: unchanged (`binding_authoring.rs:205-225`, `binding_validation.rs:212-241`, output-ancestor symlink checks); nothing in this commit touches path handling.
- **Freshness/certification**: environment-digest revalidation, fingerprint/digest validation on every load, build-cache identity perturbation, and DLPack within-run-evidence enforcement are all in untouched files; the suite's tampered-digest case re-ran and passed; `sifr_package` python 63/63 covers the drift, symlink, DLPack-evidence, and v2-schema tests.
- **Codegen/runtime**: cross-module `PythonError` conversion untouched; the suite's compiled `sifr run --frozen` case executed live this session (`binding runtime ok`).
- **Profiles/tests**: `binding-authoring` remains committed in all four profiles; the suite's expected banner was correctly bumped to `untyped_failures=3` (`binding_authoring.py:288-291`) and matches observed output.
- **Maintainability**: guardrails, fmt, clippy all clean; largest touched file `python_binding_cli.rs` remains 514 lines.
- **Documentation/ledger honesty**: `f3a768c75` correctly *re-opened* the M14 checkbox and Wave 5 (`ad-hoc-declaration-first-python-interop.md:214,1969`), reworded the status and roadmap to "in final remediation and review," recorded pass 3's NEEDS CHANGES verbatim (`:2020-2031`), and checked in the pass-3 frozen artifact. The record is accurate for the pre-merge state.

Pass-2 Notes 1–4 (error branch never executed by a gate, orphaned previous output, string-based duplicate detection, non-atomic dual write) and pass-3 Notes 2–3 (probe `SyntaxError` diagnostics on unparseable runtime annotations, stub-file suffix matching) remain accurate, non-blocking, and unchanged; they are not re-reported as findings.

## Findings

### Note 1 — probe and validator reject with different diagnostics for the same defect family

Bare `frozenset` (any unknown bare identifier) passes the probe's identifier fallthrough (`python_binding_probe.py:63-64`, needed for bound class names) and is rejected only by the Rust validator with the generic "unresolved or unsupported direct-conversion type" message, while `frozenset[int]` gets the probe's more specific "unsupported generic annotation". Both fail closed before any write; purely diagnostics polish, no action needed for M14.

### Note 2 — bind-side grammar is intentionally stricter than the compiler in degenerate corners

`is_supported_direct_type` rejects empty `tuple[]`/`None | None` shapes the compiler's `is_direct_type` would technically admit (`python_interop.rs:700,704-711`). The probe can never emit them, and strictness here is the safe direction; recorded only so a future compiler-grammar change knows both sites must move together.

## Verdict rationale

The single actionable finding from pass 3 is remediated at the root — both the probe and the scaffold validator now implement the compiler's recursive direct-conversion grammar, verified live in both the acceptance and rejection directions with zero-mutation failure behavior, and locked in by unit tests plus blocking suite fixtures exactly as the pass-3 remediation demanded. Every earlier finding remains fixed with no regression, all three M14 acceptance criteria continue to hold, the authoritative gate passed fresh at this exact HEAD, and the planning ledger honestly reflects the re-opened-pending-review state. Nothing at blocker, major, or minor severity remains.

VERDICT: SATISFIED
