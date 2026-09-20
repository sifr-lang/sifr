# DXF.8 installation review follow-ups

Status: deferred; nonblocking installation naming suggestion.
Owner: distribution installer generation naming.
Source: [DXF.8 scoped review](https://github.com/sifr-lang/sifr/pull/3889#issuecomment-5751567671).
Canonical [completed handoff](../archive/ad-hoc-compiler-dx-followup-execution.md#dxf8-merged-record--2026-09-20).

The existing canonical installer derives each permanent generation name from
its temporary staging basename, leaving a cosmetic `.stage.<suffix>` segment
in the durable selected path. This does not weaken atomic selection, immutable
generation ownership or recovery, and the DXF.8 record faithfully reports the
actual path. A future installer item may consider a clearer durable suffix
while preserving those contracts. No renaming/migration of the current user
installation or already qualified archives is authorized by this note.

The other two review suggestions (post-install shell/fish evidence and the
legacy receipt body) were resolved through supplemental external evidence;
they are not open implementation work.
