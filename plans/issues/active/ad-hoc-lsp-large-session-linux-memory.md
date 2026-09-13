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
