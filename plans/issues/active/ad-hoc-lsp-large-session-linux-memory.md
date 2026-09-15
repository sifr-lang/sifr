# Large LSP session Linux memory qualification

Owner: compiler/performance and developer-tooling stress verification.

The optional lsp-stress large-session smoke on the remote continuation observed
139534336-byte peak RSS against its fixed134217728-byte cap. The required62D
lsp-smoke and all82 LSP Rust tests passed. Neither create-pr nor merge selects
lsp-stress; this is a separate nightly stress follow-up, not a new blocker for
the frozen latest-stable implementation batches.

Independent main compiler ca7a6d60e266beccce21d4c16e08667086578277 with its own
sysroot passed the same smoke at104.0MiB and p95=56.335ms. The first comparison
accidentally paired it with the candidate sysroot and failed analysis; that
attempt is preserved and is not a performance result. No cap was increased and
no failed stress result was reclassified. A named reference for other benchmark
cases does not automatically authorize reusing their workload numbers here.

Evidence: /home/yaser5/projects/sifr/continuation-evidence/20260913-lsp-memory
and ../20260913-lsp62d/remote-lsp62d-settled.json. Assess the candidate's additional
resident memory and qualify the stress workload under the approved named-host
policy in this owner. Preserve the original fixed-cap failure.


## Release-profile dependency — 2026-09-13

Full release developer-tooling includes lsp-stress. Therefore this issue must
be resolved before fresh release-profile/phase closure even though create-pr
and merge do not select it. It is not an external dependency or an upstream wait.

After fixing Cargo target selection, the current compiler still recorded
138489856-byte peak RSS. A separate smaps diagnostic measured heap98444KiB
and executable mappings35012KiB. Both runs preserve the fixed128MiB cap.
Compiler SHA256:2a5e635f13d52abfef269c46b9a86e7dc89994df7e30cdb0ae08410c748e644d.
Evidence includes corrected-target.json, corrected-target-execution.log,
smaps-diagnostic.json and peak-smaps.txt in the existing evidence directory.
An initial unsupported --output-root invocation did not execute the workload;
it is preserved separately. No failed performance result is relabeled.


## Remote LSP memory follow-up qualified — 2026-09-13

The required release stress dependency is now implemented and locally qualified
within the existing low-resource performance work. No stress cap or workload was changed.

A focused negative test found2635776 bytes retained by empty structural-template
body vectors. Signature templates now project declaration fields directly, avoiding
full-body cloning and retained empty buffers. A full-inventory oracle compares
every projected class with the previous clone/clear semantics; complete checked
HIR bodies remain present. Buffer-only and direct-projection-only RSS attempts
still failed and remain recorded.

The dominant HIR statement layout was1584 bytes because rare nested writes kept
three expressions inline. Boxing their inner index in both nested-write variants
reduces every statement to1232 bytes on the measured64-bit host. All typed content,
visitation and evaluation order are retained. Four lowering constructors change;
a64-bit layout guard bounds future statement inflation.

Validation: IR5, lowering96, codegen138 and stdlib97 tests pass (two explicitly
ignored stdlib tests stay ignored). The new nested_assignment_storage_regression
and existing checked_place_collection_semantics programs build and execute;
they cover plain/augmented and attribute-backed writes, evaluation order,
outer/inner failures, negative indexes and unchanged contents after errors.
A first outside-package fixture invocation was rejected before execution;
the subsequent owned e2e fixture run passes and is the retained execution proof.
Format, file-size3874, lowering maintainability and diff guards pass.

After the required idle private-target cleanup and cold build, an explicit
large-session warmup is kept separate. The controlled qualifying smoke passes:
42operations, p95=21.895ms, peakRSS132091904 bytes against134217728,
zero measured RSS slope. It uses the complete unchanged stress corpus.
This is targeted qualification, not a create-PR/full merge/release-profile pass.
Evidence: /home/yaser5/projects/sifr/continuation-evidence/20260913-lsp-memory/.
The source/binary receipt is retained outside Git after this commit.


## Integration delivery — 2026-09-15

The implementation is merged through PR 3827, merge
`33639f4ee3b7079ec4da889834cae0d55d763786`. Full merge gate 3 passes at
`d2bc1e0c3ca2c1fd454dc5eb4bc307abfcfa66ea`, including both nested-write E2E
regressions and the representative performance selection. The actual release
profile, including large-session stress, is running at that same unchanged
candidate. This issue remains open for that final qualification result; the
earlier focused PASS is not relabeled as a release-profile result.
