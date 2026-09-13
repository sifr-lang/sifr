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
