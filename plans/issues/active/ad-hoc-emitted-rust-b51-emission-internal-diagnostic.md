# 12K-B51 — bounded emission-internal diagnostic

## Terminal disposition

Diagnostic complete: the leading emission mechanism is final whole-file Rust
syntax parsing, including disposal of the successful parsed AST. A specific
behavior-preserving production repair remains unresolved inside that call.
Execution blocker: none. PR: none. Merge: none. STOP.

The concrete B51 authority explicitly replaced the generic implementation/review/
merge workflow with temporary diagnostics. Both requested skills were read.
Exactly one ordinary locked compiler build and four fixed U/I/I/U arithmetic
checks ran. No testsuite, Opus, profiler, representative, warmup, retry, full gate,
production repair, baseline change, PR, merge or qualification credit.

- Production base: `30b25c551566bf0c146290c7ec38a55621c5d526` (B49).
- TEMP diagnostic source: `0e7a1f37f49f070a494095284c1cf43aac6eb91b` on pushed
  `codex/item12k-b51-8B6YsH`. **NEVER merge or use as a production base.**
- This separate plans-only branch: `codex/item12k-b51-record-8B6YsH`, directly
  from B49. No instrumentation is in its ancestry or compiler tree.
- Independent HTTPS clone: `/private/tmp/sifr-b51.8B6YsH/sifr`, with own Git
  index/refs/objects, no alternates, own default target/TMP/control/evidence.
- Parent and predecessors remained read-only. The parent's two known dirty
  ledgers were foreign-owned and never treated as a blocker or edited here.
- Remote main observed before work: `4b4cc339964baeeb6641e57dc669fef700a5fa24`.
- Owning issue: [3776](https://github.com/sifr-lang/sifr/issues/3776).

## Measured source mechanism

At B49, driver `stdlib/bootstrap.rs:328-344` calls
`generate_stdlib_module_body`. The structural policy selects inline emission,
which assembles imports/support/body and executes `syn::parse_file(&assembled)`
in `crates/sifr_codegen/src/lib_modules_and_codegen/deferred_codegen.rs:134-136`.
The successful `syn::File` is discarded there. The timer brackets that original
statement, including its temporary's destruction; it does not separate parser
execution from successful AST disposal.

**This final syntax-validation call leads both instrumented runs**, with 89
calls over the complete original module inventory. Its share of inclusive emission
is 54.8471% / 54.9167%, and of measured command time 25.1927% / 23.1913%.
Public stdlib accounts for 407.208418 / 473.151791 ms of that final-parse family;
private declarations account for 59.006874 / 61.680291 ms.

| Exclusive emission subphase, ms | Instrumented 1 | Instrumented 2 |
| --- | ---: | ---: |
| Final `syn::parse_file` and AST disposal | 466.215292 | 534.832082 |
| HIR constants/body emission, excluding nested setup | 209.125040 | 240.718039 |
| Body IR preparation, optimization, import analysis, validation | 75.129167 | 80.732918 |
| Body Rust IR rendering | 35.457584 | 40.658419 |
| Emitter/import setup and named-module prescan | 19.316673 | 23.861998 |
| Union/recursive-field/layout passes | 15.194632 | 18.201244 |
| Support demand rendering/assembly | 12.717710 | 14.338284 |
| Emission residual outside narrower spans | 6.495850 | 8.230850 |
| Import/body/result assembly | 5.292713 | 6.268788 |
| Imported structural impls/post-enums/support demand | 5.081838 | 6.054748 |
| **Inclusive emission total** | **850.026499** | **973.897370** |

The individual leading final-parse module is **not stable**: `sifr.datetime`
19.704542 ms in the first run, `sifr.statistics` 33.186500 ms in the second.
Only the leading subphase and public-stdlib family agree. No single-module
optimization or workload-independent ranking follows from these observations.

**Precise unresolved production boundary:** token-stream construction, syn
grammar/AST construction, and returned AST destruction remain inside the same
measured final-validation call. The evidence identifies its owner but does not
select an error-equivalent lower-cost replacement or prove any portion redundant.
There is therefore **no supported production repair recommendation** from B51.
Required final syntax validation, checks, error timing, emitted semantics and
normal source order remain binding. No additional diagnostic or repair was begun.

## Fixed four targets and perturbation

Every invocation directly launched its binary with
`check crates/sifr/tests/e2e/pass/arithmetic.sifr`, cwd and `SIFR_SYSROOT`
`/private/tmp/sifr-b51.8B6YsH/sifr`, and identical
`TMPDIR=/private/tmp/sifr-b51.8B6YsH/tmp`. Saved original B49 was copied mode555
to `control/sifr-b49`, separate from the new compiler target. Each command has
distinct stdout/stderr/started/process receipts and a distinct phase output path.
Controls correctly produced no diagnostic record; instrumented runs produced two.

| Order | Binary | Observed process seconds | Measured command seconds |
| --- | --- | ---: | ---: |
| 1 | Saved uninstrumented B49 | 4.576740541 | — |
| 2 | TEMP instrumented B51 | 3.897435500 | 1.850594416 |
| 3 | Same TEMP instrumented B51 | 2.337899584 | 2.306179375 |
| 4 | Same saved uninstrumented B49 | 1.907278417 | — |

All four exited0, stdout empty, stderr exactly `no errors found\n`. No timeout,
cleanup signal or remaining owned process. Four-command window13.680223 seconds,
each target below60s and cleanup below30s. Immediate actual process release was
sent to Police and parent before offline analysis.

**Perturbation is inconclusive.** First/last controls differ by2.399618x.
Instrumented/control mean ratio0.961647 is descriptive only, not negative
overhead or a speedup. Observed process time outside measured command is
2.046841084 / 0.031720209 seconds; no particular startup/shutdown cause is proven.
No subtraction correction, extra sample, warmup or retry was taken. These are
actual monotonic elapsed nanoseconds, not retired instructions, heap, RSS,
historical-cause attribution or qualification evidence.

## Coverage and integrity

The authenticated B50 diagnostic patch and complete report were inspected before
reusing the bounded collector and unchanged established process supervisor.
B50 instrumentation was never the production base. New exact source spans and
commands were registered before editing in `evidence/pre-edit-registration.md`,
SHA256 `33787f5c5b5f4d09ae87103b6e6efe24143594b60098d2b25e4e2cebb564baeb`.
Final source/accounting verification is `evidence/source-verification.md`, SHA256
`396998f4d4bf1ecbfe91b23f094bf18c511e4d2cdea6c27c26eeaaa42f5bc308`.

Exactly12 total families: command/bootstrap/emission/setup/type_passes/hir_body/
structural_post/ir_passes/body_render/support/assembly/final_parse. All89 modules
appear in source order. The collector remains bounded128 module slots,32 scope
depth,12 aggregates per module,96-byte module names. Observed max depth6. Static
family labels are borrowed and module names copied once; no per-node events or
printing. One structured flush occurs after normal successful completion.

Each inclusive duration equals its exclusive duration plus direct-child time.
Children are charged exactly once; named-module setup is subtracted from HIR
body, emission from per-module bootstrap, and modules from global bootstrap.
Module totals are separate containers, never added to exclusive sums.

- All exclusive families sum exactly to command1.850594416 / 2.306179375s.
- Global bootstrap1.475648834 / 1.729926875s. Summed module containers
  1.444433294 / 1.695824162s remain inside those values.
- Module covered-exclusive time1.444399036 / 1.695765252s gives
  99.997628% / 99.996526% coverage; explicit gaps34,258 / 58,910ns retained.
- Emission partition coverage99.235806% / 99.154854%; residual6.495850 /
  8.230850ms retained, including call transitions and unpartitioned drops.
- Each module has one emission/IR/render/support/final-parse pass, two setup
  scopes and three assembly scopes. Full existing node traversal is preserved.
- All132 protected inputs and both binary hashes match their freeze after the
  four targets. No fixture, lockfile, sysroot, dependency pin or configuration edit.

Durable [raw/derived artifacts](ad-hoc-emitted-rust-b51-artifacts/summary.json):

- [Freeze](ad-hoc-emitted-rust-b51-artifacts/freeze.json):
  `566d0bb3748dd5828207e3ac75a11f271577342d7e502b752c24703438387044`.
- [Raw1](ad-hoc-emitted-rust-b51-artifacts/check-02-instrumented.phases.json):
  `7199ff2c224260127043da38d3867100c931eb9a57311be787cebff55a912cd7`.
- [Raw2](ad-hoc-emitted-rust-b51-artifacts/check-03-instrumented.phases.json):
  `37fe2f1e50e73569589dc2ecfbff497c78d968800cb900a427c7ad878fb86e36`.
- [Summary](ad-hoc-emitted-rust-b51-artifacts/summary.json):
  `81b639507bf51233142e5d61eea1702300bed7a64aa1d1ed2ef13ec7cd011d84`.
- [Attribution](ad-hoc-emitted-rust-b51-artifacts/attribution.json):
  `59376ef301735b4342d71024497672a220875d15d8a2e80de50804e230617900`.
- [Record checks](ad-hoc-emitted-rust-b51-artifacts/record-checks.json):
  `e2e45714ca6582dc615c7cd8fe1167b4769bbc5f762ee2967ecab4be164a25ca`.

TEMP eight-source-file patch: `evidence/diagnostic.patch`, SHA256
`7aa19623972bbad7f869e8fa884bc61b59a11e071c56079260612c1c81856225`.
Original control binary SHA256:
`41203eb21fcfe364925bdcc57309b0dbdcb8d5aef65bcbd2fb353e0aa6e26a12`.
TEMP binary SHA256:
`99ed89021a438f6249d9b56726eab66a80a5d496387b064b5f0eb7d387166c5b`.
Arithmetic input SHA256:
`948ecd80ced4da3aab8f31f5f9712b2cd36a4c0186469d05c85dce335aefc448`.

## Build, resource ownership and closure

Item42 held the native window throughout source work. Police separately released
the ordinary compiler build after Item42's actual process closure, then separately
authenticated the freeze and released the fixed four runtime targets. No competing
heavy-work assumption or routine user approval checkpoint was introduced.

One `cargo build --locked -p sifr` passed112.801440s (112.939120s including release),
below900s. Zero compile remediation. Own exact Rufff19957111640fdee8055bfe5b6aa854259344473
and WASI448f6df8f688cee5d6995e96b1ffc31f9bf00742; own default target/TMP, no shared
target. Pre-build root1,042,104KiB/free115,443,060KiB. Freeze root6,446,604,288bytes /
free113,076,953,088bytes; final runtime root6,429,089,792bytes/free113,311,973,376bytes.
All within20GiB/new-owned and8GiB/free bounds. No foreign cleanup.

Source-only accounting/order inspection, `git diff --check` and file-size guard
PASS3782 files/900-line cap before build. One extra trailing blank line was caught
and removed before committing the diagnostic candidate. No suite was run.
Final records receive documentation/integrity checks only, without more gates.

All nine existing budget failures remain binding. Full phase12D/12E/12F, full12/
12A closure, formatter/editor RSS owners and smallest complete qualified
builtin-registration/list-repeat delivery remain open. B49 ownership repairs
and B46 repairs are retained, not reproposed as defects. B27's existing runtime
interop gap and B49 nonblocking review suggestions remain with owner3776.
No baseline-f8 hunt, broad corpus/history audit or automatic next-item work.
Execution blocker none; production repair selection unresolved. **STOP.**
