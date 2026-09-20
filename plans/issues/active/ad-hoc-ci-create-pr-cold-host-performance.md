# Automatic create-pr cold-host performance failure

Status: open; not repaired or waived by workflow admission closure
Owner: verification Cargo preparation / Rust interop area performance

The first admitted CI candidate `dde99fdc32153afdcf93c448db9e709c8d32828d`
reaches and passes all 13 Rust interop variants, then the unchanged profile
correctly exits 124 because `area_rust_interop` takes 47,478 ms against its
20,000 ms blocking performance budget.
[Run 35532393023 / job 106135185833](https://github.com/sifr-lang/sifr/actions/runs/35532393023/job/106135185833)
reports `maintained-rust-demo-compile/area-check` at 40,202 ms.
Its cold Cargo setup passes functionally but takes 8,692,108 ms; total lane time
is 8,947.39 s and maximum RSS is 4.1 GiB. The lane stops at its first blocking
budget failure, so subsequent profile checks are not claimed executed/passing.
No compiler assertion failure is identified in this result.

The original log is preserved outside Git as
`/home/yaser5/projects/sifr/ci-admission-evidence/dde99fdc32153afdcf93c448db9e709c8d32828d/create-pr.log`.
This first cold hosted run is not host-sensitive performance qualification.
The admission repair changes no profile budget, Rust interop assertion, fixture,
compiler source, or demo preparation/execution owner. It neither inflates the
budget nor reruns the full cold profile to seek a lucky pass.

Minimal next action: inspect the preserved preparation and area command evidence,
then use the existing prepared remote artifacts for a named
`maintained-rust-demo-compile` preparation/execution comparison. Establish whether
preparation and execution actually select the same Cargo input/configuration and
reuse its artifacts; fix an observed bounded invalidation defect if present.
If they match, collect a correctly labeled warm-host measurement of this exact
area before making a budget/platform policy decision. Preserve every assertion
and the original failed receipt. Do not restart the monolithic create-pr gate as
a substitute for this diagnosis.


## Final admission candidate receipt

Candidate `05c72e32a2b66fc1414929e40b59f6c6e619e454` reproduces the performance
failure in [run 35533124294 / job 106137284268](https://github.com/sifr-lang/sifr/actions/runs/35533124294/job/106137284268):
all 45 preparation-policy tests and 13 Rust interop variants pass; the area takes
57,170 ms against 20,000 ms and exits 124. The maintained-demo case takes
46,250 ms. Cold setup passes at 10,287,999 ms; total lane time is 10,603.50 s,
maximum RSS 4.1 GiB. Later lane checks are not executed after this fail-fast stop.
The final log is `create-pr.log` in the matching candidate evidence directory.
This remains unresolved after admission/provisioning PR #3891; its merge does
not waive this budget or establish a warm-host performance pass.
