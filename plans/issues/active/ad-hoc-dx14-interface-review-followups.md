# DX.14 interface and editor persistence review followups

Status: open follow-up work; not a blocker for scoped DX.14 closure.

Source: initial and final [scoped Opus reviews of PR #3866](https://github.com/sifr-lang/sifr/pull/3866#issuecomment-5729004460). Final verdict: **SATISFIED**, no blocking findings. The initial review's sole blocking
AST-string residency/cache-key cost regression was corrected with full SHA-256
structural fingerprints and a bounded-key regression. These nonblocking
observations are separate bounded work; do not expand the merged DX.14 item.

| ID | Owning boundary | Observation and next bounded action |
| --- | --- | --- |
| DX14-F1 | Captured observation storage | Repeated interface-restored edits append failed-validation prefixes, source reads and replayed observations. Bound redundant observations while preserving resolver order and absence semantics; prove unchanged read/write validity before changing the canonical capture contract. |
| DX14-F2 | Historical project-record lookup | Every compatible historical record can trigger interface proof parsing. Add a cheap, correctness-preserving candidate filter and measure a bounded long edit history. Coordinate with DX13-F4 record-retention work. |
| DX14-F3 | Editor semantic context | Explicitly pin frontend target/mode with saved-record restoration as the context model evolves; keep the current path/source/context checks intact. Add a single-module project-versus-document boundary test before expanding eligibility. |
| DX14-F4 | Phase-end lint qualification | Review noted a possible clippy::let_and_return shape in manifestless_inputs. This is not a measured lint failure or a waived gate; qualify and fix it with the final implementation's full gate under the existing lint ownership. |
| DX14-F5 | Reuse effectiveness boundaries | The original DX.14 qualification covered flat legacy source graphs reached from a manifestless CLI invocation; ordinary package graphs initially retained only exact-context reuse. Editor restoration still requires the same process cwd as the saved CLI semantic context. Package importer reuse is now qualified by [DXF.1 / #3875](https://github.com/sifr-lang/sifr/pull/3875), retaining package, trust, toolchain and resolver authority. The editor cwd limitation is unchanged. |
| DX14-F6 | Structural hash adapter invariant | Hasher::finish is unused by the current derived comparable-AST Hash implementations. Guard that invariant before accepting future manual nested Hash implementations, so a shortened intermediate hash cannot become semantic authority. |
| DX14-F7 | Interface fingerprint constant cost | IdentityEncoder frames every primitive write with a repeated chunk label. Measure whether a smaller label is worthwhile without changing domain separation or unambiguous framing. |
| DX14-F8 | Identity documentation precision | Clarify the InterfaceHasher comment: its digest is recomputed for disk-record comparisons and used in compiler-namespaced in-process cache keys; the digest itself is not serialized into project records. |
| DX14-F9 | Pre-existing declaration signature residency | Large module-level literal initializers still use expanded Debug text in ExportSignatureEntry::shape at the base. Bound this declaration-level cost in separate work, preserving constant/default discrimination. |
| DX14-F10 | Future interface-proof expansion | Document the transitivity obligation of chained restored-success records before widening the body-erasure class. Each current hop must continue to prove semantic stability and check changed code successfully. [DXF.1](ad-hoc-compiler-dx-followup-execution.md#dxf1-merged-record--2026-09-20) documents and exercises this chained-success obligation without widening the body-erasure class. |

The initial completed review remains external at
/home/yaser5/projects/sifr/dx14-evidence/86cef8b58945d7f26366e25328b4beb8e49b5754/opus-review-initial.md.
The approved candidate and final review are keyed by
2c3f23127966850e7ed961155bcee817514d56f1 in the same evidence root.
Existing [DX.13 followups](ad-hoc-dx13-project-review-followups.md) and
[metadata/profile lint ownership](ad-hoc-dx-metadata-review-followups.md)
remain separate.
