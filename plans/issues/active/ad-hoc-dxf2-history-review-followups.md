# DXF.2 bounded history review follow-ups

Status: nonblocking follow-up work, separate from merged DXF.2.

Source: [scoped Opus review of #3877](https://github.com/sifr-lang/sifr/pull/3877#issuecomment-5749545936),
candidate `94d35655a065859acdecf1aa3ee33c31a3165305`.
Verdict: **SATISFIED**, no blocking findings. The review is preserved outside
the Git tree in the candidate's `dxf-evidence/opus-review.md`.

| ID | Classification / owner | Finding and disposition |
| --- | --- | --- |
| DXF2-F1 | Suggestion / storage documentation | Resolved by [DXF.3 / #3879](https://github.com/sifr-lang/sifr/pull/3879): `Store::prune` now describes bounded recent history, leased predecessors and safe misses for evicted records. |
| DXF2-F2 | Suggestion / capture efficiency | Cheap rejected-candidate resolver checks add true facts to the shared capture. Deduplication and overflow bound these facts and preserve correctness, but they can add validation work. Separate optimization work may isolate these facts only with unchanged resolver/absence authority. Not an authorized proof expansion. |
| DXF2-F3 | Suggestion / lookup cancellation and decoding | Lookup eagerly decodes the retained records before checking cancellation. The 128-record / 64 MiB cap bounds this cost; measured full retained lookup was about 21 ms. Separate work may restore finer cancellation or reduce retained decoding without weakening complete generation integrity. |
| DXF2-F4 | Suggestion / documented reuse limits | The one-proof budget tries the newest qualifying candidate; if it fails, older candidates are not proved. This intentional bounded-work policy can miss an older reusable result; ordinary computation remains authoritative. Retain this limit in future performance/reuse claims. |
| DXF2-F5 | Infrastructure / test scheduling | The non-ignored 4097-publication test took 95.27 seconds and adds that work to ordinary driver library runs. Future verification scheduling may separate this storage stress while preserving its exact named acceptance and nonzero selection guards; no test was silently disabled here. |

No finding reopens DXF.2, adds a current gate, or authorizes work outside the
canonical sequential follow-up plan.
