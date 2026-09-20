# DXF.6 contract review follow-ups

Status: open; nonblocking maintenance observations, not new authorized scope.

Origin: [scoped Opus review of a7e45101600f89179fa1c5b2a80cb5beaa3e7d53](https://github.com/sifr-lang/sifr/pull/3885#issuecomment-5750006086).
DXF.6 is satisfied; these observations do not reopen it.

- **Documentation history owner:** historical `plans/reviews/active/*` prose and
  the archived interop issue's self-reference still mention moved active paths.
  These are outside the five audited roadmap/index links. Consider only in a
  separately scoped historical-link audit.
- **Documentation verification owner:** the local link/anchor evidence comes
  from external `dxf-evidence/check_dxf6_docs.py`; the in-tree structure checker
  does not enforce links. A durable documentation-area link check is optional
  future work, not a missing DXF.6 check or authorization to expand DXF.7.
- **Cache CLI test owner:** exact four-examined/three-eligible counts encode two
  generations plus two publication staging locks. The harness comments explain
  this source; a future storage-layout change should update the contract with
  diagnostic assertions or observed-layout evidence. No current defect found.
