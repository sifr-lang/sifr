# 12K-B50 — bounded in-process phase diagnostic

## Terminal disposition

**Diagnostic complete; emission owner localized; specific production repair remains
inconclusive inside that boundary. No execution blocker. STOP.**

Exactly one ordinary locked build and exactly four intended-success arithmetic
checks completed. No testsuite, Opus review, formatter, external profiler, full
gate, PR, merge, baseline change, production repair or qualification credit.
The latest concrete B50 authority superseded the generic review/merge workflow.
Both requested skills were read; their generic validation/review/merge steps did
not apply to this explicitly diagnostic-only item.

- Approved held production source: `30b25c551566bf0c146290c7ec38a55621c5d526` (B49).
- Temporary instrumented source: `eaac03390bce342fba77da5a73669cbc096dc584`.
- Diagnostic branch: `codex/item12k-b50-38g7V9`, pushed. **NEVER merge or use this
  instrumentation as a future production base.**
- This separate docs branch: `codex/item12k-b50-record-38g7V9`, rooted directly at
  approved B49. It contains no diagnostic compiler changes.
- Actual remote main observed: `4b4cc339964baeeb6641e57dc669fef700a5fa24`.
- Owned independent HTTPS clone: `/private/tmp/sifr-b50.38g7V9/sifr`; independent
  index, refs and objects, no alternates, own default target and TMP.
- Evidence: `/private/tmp/sifr-b50.38g7V9/evidence`.
- Parent and predecessor worktrees, binaries, records and caches remained read-only.
  The parent's two known dirty ledgers were intentionally foreign-owned.
- Owner: [issue 3776](https://github.com/sifr-lang/sifr/issues/3776).

## Actual observation

The same exclusive leading phase and module family appeared in both instrumented
runs: **public-stdlib Rust emission**. Every one of the 89 expected source modules
appeared in original inventory order. The largest individual emission module was
`sifr.process` in both runs (35.447 and 36.025 ms); `sifr.datetime` followed
(34.481 and 34.437 ms). Whole module container totals instead put `sifr.json` first
(63.472 and 65.647 ms). These are distinct rankings, not interchangeable costs.

| Exclusive phase, milliseconds | Instrumented 1 | Instrumented 2 |
| --- | ---: | ---: |
| Rust emission | 843.942078 | 838.335162 |
| Lowering, excluding nested externals copy | 464.242579 | 459.927160 |
| Command residual outside nested measured phases | 363.675501 | 368.858167 |
| Lowering externals copy, stdlib plus user | 71.876044 | 72.284917 |
| Sysroot inventory | 28.562791 | 27.710375 |
| Stdlib parsing | 28.455917 | 27.851833 |
| Emitted metadata/templates | 17.375420 | 17.249416 |
| HIR canonicalization | 9.601288 | 9.051667 |
| Export construction/publication | 7.792372 | 6.963619 |
| Bootstrap residual | 5.566135 | 5.199928 |
| Private plan, pending detection plus final batch | 2.397293 | 2.252381 |
| Cache access, excluding initialization children | 0.055374 | 0.011334 |

Emission comprises 57.10% / 57.22% of inclusive bootstrap time and 45.78% / 45.67%
of measured command time. Public modules account for 755.001456 / 751.208706 ms
of emission; private modules account for 88.940622 / 87.126456 ms. The full final
private contract batch alone is 2.348083 / 2.223958 ms, recorded globally because
it consumes the entire source-ordered pending inventory. Its cost was not assigned
artificially to individual modules. The B49 ownership repair is retained; these
observations do not support re-proposing its already-removed copies.

The measured source owner is the actual call from driver
`stdlib/bootstrap.rs:328-344` at B49 to
`sifr_codegen::generate_stdlib_module_body`, defined in
`crates/sifr_codegen/src/lib_modules_and_codegen.rs:136-147`. This calls the
structural/project-policy path in the same module. Its setup, union/field analysis,
named-module/structural emission, support demand and final result are inside the
measured emission family (`lib_modules_and_codegen.rs:150-306`). Final result
construction spans `lib_modules_and_codegen/deferred_codegen.rs:18-145`: Rust IR
passes, validation, rendering, inline support assembly and final syntax validation.
These source spans are owner traceability, not separately measured subphase costs.

**Precise remaining boundary:** the diagnostic does not distinguish which of those
emission-internal mechanisms is redundant or safely repairable. It supplies no
supported specific production optimization. The one next recommendation is for
the coordinator to scope attribution within this emission owner before selecting
a repair, using clean B49 and excluding the entire temporary diagnostic diff.
This worker did not begin that follow-up. Full required bootstrap/validation,
error timing, private contracts and source order remain binding; laziness,
validation elision and another speculative copy repair are not authorized.

## Exact four targets and perturbation limit

All commands directly launched `BINARY check crates/sifr/tests/e2e/pass/arithmetic.sifr`,
cwd `/private/tmp/sifr-b50.38g7V9/sifr`, explicit `SIFR_SYSROOT` equal to that clone,
and identical `TMPDIR=/private/tmp/sifr-b50.38g7V9/tmp`. The control binary was copied
read-only to `control/sifr-b49`; compilation could not overwrite it. Each invocation
has distinct stdout, stderr, process receipt and phase output path. The two controls
correctly produced no phase record.

| Order | Binary | Direct process seconds | Measured command seconds |
| --- | --- | ---: | ---: |
| 1 | Saved B49 control | 4.389716125 | uninstrumented |
| 2 | Temporary diagnostic | 3.863845459 | 1.843542792 |
| 3 | Temporary diagnostic | 1.851679375 | 1.835695959 |
| 4 | Saved B49 control | 1.888970125 | uninstrumented |

Every target exited 0, stdout was empty, and stderr was exactly
`no errors found\n`, matching the original CLI contract. All process groups were
released, all remaining-process lists were empty, and no cleanup signal or timeout
was required. The four-command window lasted **12.775430 seconds**, below 360;
every direct target was below 60 seconds and cleanup below 30 seconds. Police
was notified immediately at completion so Item42 could resume provisioning.

**Quantitative perturbation is inconclusive.** First/last control process times
differ by 2.324x. The instrumented/control mean ratio is 0.910306, a descriptive
comparison only; it cannot establish negative instrumentation overhead or a
speedup. Instrumented process time outside the measured command is 2.020303 s
and 0.015983 s respectively. That remainder includes pre-wrapper process startup,
post-command output and observation overhead, without attribution to one cause.
No extra warmup, retry or sample was taken. No exact subtraction correction was
applied. Phase durations are elapsed time, not retired instructions, heap, RSS or
evidence that any original budget passed.

## Instrumentation and integrity evidence

Source spans, precise commands, logic checks and capacity were registered before
instrumentation edits in `evidence/pre-edit-registration.md` (SHA256
`cc389cbaedcde2be1ca882678111ff08a410dd072c50b35c979238c4a2764964`).
The eight-path temporary patch is retained as `evidence/diagnostic.patch`, SHA256
`0ccc20633944a930ea05273ff8ad7172e970582a7f943a5c9bbe7aa1396de7c6`.

Exactly twelve named phase families use `Instant` and integer nanoseconds.
There are at most 128 module records (including global), 12 phase aggregates per
record and 32 nested scopes. Observed depth was five. Static phase labels are
borrowed; each module label is copied once into a fixed 96-byte record. There is
no per-node printing, instrumentation event, dependency, allocator substitution,
feature change or cache/semantic bypass. Structured output is flushed once only
after the normal command returns success. Rendering is outside the command timer.

Every scope records inclusive duration, direct-child inclusive duration and their
checked exclusive difference. Direct-child duration is charged to its parent
exactly once. The separate module totals are container coverage evidence and are
never added to exclusive phase totals. Nested lowering externals copies are
deducted from the parent lowering row; instrumentation bookkeeping remains in
parent exclusive/residual time.

Offline verification passed:

- Valid records, closed scope stacks, bounded sizes/counts, nanosecond units,
  original 89-module order and one parse/lower per module.
- Every inclusive value equals children plus exclusive; all exclusive phases
  sum exactly to measured command inclusive time in each run.
- Inclusive bootstrap 1.478030667 / 1.465225708 s; summed module container time
  1.446827250 / 1.435128418 s stays inside bootstrap.
- Module exclusive coverage 99.6355% / 99.6490%; uncovered gaps are retained.
- All 132 selected compiler/sysroot/stdlib/fixture/lock/config inputs match B49
  before and after acquisition; source and both binary hashes are unchanged.

Raw records and checks are attached under
[B50 artifacts](ad-hoc-emitted-rust-b50-artifacts/summary.json):

- [Instrumented 1 raw](ad-hoc-emitted-rust-b50-artifacts/check-02-instrumented.phases.json),
  SHA256 `d5d621ec903074c1eb81162499fc525c99ee462a655725aa4a6bb54145e5c9ae`.
- [Instrumented 2 raw](ad-hoc-emitted-rust-b50-artifacts/check-03-instrumented.phases.json),
  SHA256 `c153ac9a4bb6483e0768cc6560cd673d0e218b3c7410af6b3f670c698014ee51`.
- Both `.integrity.json` companions contain full module totals and ranked phases.
- [Freeze](ad-hoc-emitted-rust-b50-artifacts/freeze.json), SHA256
  `fc6252a5ba3e23cf2803628652646beef27be20a7b8a929d3f3b247320ad1de1`.
- [Summary](ad-hoc-emitted-rust-b50-artifacts/summary.json), SHA256
  `4722ea56aa280a9c964d52d6fa8bcf587f606c6c3ec45047d8adfe8f90b6f219`.

Control binary SHA256:
`41203eb21fcfe364925bdcc57309b0dbdcb8d5aef65bcbd2fb353e0aa6e26a12`.
Diagnostic binary SHA256:
`05510d0a437bdf54277cbd7ada7d6b36c672b31a7d735a26b519a90b7c3d1457`.
Arithmetic input SHA256:
`948ecd80ced4da3aab8f31f5f9712b2cd36a4c0186469d05c85dce335aefc448`.

## Bounded build, records and stop

Own pinned Ruff `f19957111640fdee8055bfe5b6aa854259344473` and WASI-Virt
`448f6df8f688cee5d6995e96b1ffc31f9bf00742` were provisioned without pin changes.
After Police released the ordinary build window, `cargo build --locked -p sifr`
passed in 107.425334 s (107.549108 s including release), below its 900-second cap.
One compile attempt; zero remediation attempts. The established B49 supervisor
was narrowly parameterized for separate stdout/stderr, direct launch timing and
30-second cleanup; no new custody framework or selftest suite was introduced.
Child-exit observation may add up to approximately 50 ms polling delay; durations
are process observations, not exact CPU timings.

Pre-build storage was about 0.994 GiB; after acquisition, total new owned storage
was 6,475,153,408 bytes (about 6.031 GiB), below 20 GiB, and free space was
135,909,146,624 bytes, above 8 GiB. No foreign cleanup. Build and all four targets
were supervised and reaped. Source-level diagnostic-logic verification, diff
whitespace check and file-size guard passed before build (3782 files, limit900);
the exact source evidence was reused after acquisition. Records receive only
documentation/integrity checks. No testsuite or additional gate was run.

The full original emitted-Rust phase remains open, including 12D/12E/12F,
full item12/12A closure, all nine existing budget failures, formatter/editor RSS
owners and the qualified builtin/list-repeat integration priority. B27's
pre-existing runtime interop gap and other owners were not absorbed. No f8
history hunt, B48 xctrace retry, broad corpus or unrelated work was started.
PR: none. Merge: none. Execution blocker: none. Mechanism selection remains the
coordinator's later scope. **STOP; native close after terminal callback.**
