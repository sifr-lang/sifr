Reviewing the three layers for both conditionals:

**futures-util:**
- **Policy** (line 33): "only where generated-runtime async combinators would otherwise require substantial custom poll-level helper code" ✓
- **Phase table** (line 261): "conditional `futures-util`" ✓
- **Phase locked row** (line 273): "only if M1 proves `join_all`, `race`, `select`, or stream adapters would otherwise require substantial custom `Future`/`poll` code … If M1 can implement the accepted combinators cleanly with Tokio and generated helpers, `futures-util` is not added." ✓
- **Resolved Decision** (line 905): "conditional `futures-util 0.3.32`" ✓
- **Ledger** (line 158–159): "`futures-util` is now conditional and added only if M1 proves … substantial custom `Future`/`poll` code." ✓

**rustix:**
- **Policy** (line 46): "for documented host-limited process/signal/file-descriptor gaps not covered by `std` or Tokio" ✓
- **Phase table** (line 262): "targeted `rustix`" ✓
- **Phase locked row** (line 279): "only … after M4/M5 records that `std`/Tokio cannot provide the required behavior … Every use must have a supported-host matrix row and a deterministic host-specific fixture." ✓
- **Resolved Decision** (line 905): "Rustix only after a documented std/Tokio gap with host-matrix fixtures" ✓
- **Ledger** (line 159): "`rustix` now requires a documented `std`/Tokio capability gap plus supported-host matrix rows and deterministic host-specific fixtures before use." ✓

**Contradiction scan:** No drift between policy → phase ring summary → phase locked table → resolved decision → ledger for either crate. All three documents express the same conditionality with consistent language.

---

**PASS** — Both conditionals are correctly expressed across policy, phase, and ledger with no contradictions.
