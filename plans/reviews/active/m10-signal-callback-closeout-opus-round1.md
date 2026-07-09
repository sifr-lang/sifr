## M10 Closeout Review — Verdict: **Findings**

The two engineering slices themselves look sound, but M10 cannot be marked complete: the milestone tracker was never updated to reflect the merged M10b slice, and there is a manifest-reference item worth verifying. Details below, mapped to the four review axes.

### What checks out

- **Signal native-boundary closure (M10a).** `_sifr.signal` is `retained → closing` with a concrete `removal_criteria` ("delete after migrated signal registry names are proven absent by closure guards"), retained compiler signal registry/source removed, routing through `_sifr.signal` private Rust interop + `sifr_stdlib::signals`. This matches the established M4–M9 closing pattern.
- **Future-owned ecosystem separation (M10b).** `callback_subscription_core` carries `required_crates: []` and `callback_subscription_ecosystem` retains `tokio-tungstenite`/`redis`/`notify` with `future_owner` pointing at the certification issue and evidence `planned`. Ecosystem crates are correctly kept out of the supported core row — exactly what the handoff table requires.
- **Certification ownership.** `_sifr.signal` claims only `callback_subscription_core`; negative evidence (`invalid_subscription_callback_policy_rejected`, `missing_backpressure_rejected`) is passing, so unsafe migration is blocked. Consistent with the M3/M6 split precedent.

### Findings

**1. (Blocking for closeout) Tracker not updated for merged M10b.**
The tracker still shows M10 as **`in progress`** and its evidence column records only M10a (PR #2892, "Opus review satisfied in round 6"). Merged M10b (PR #2894 · sha=330e277) has no evidence cell. Per the milestone format, it needs a certification-style entry, e.g.:
> `PR #2894 · sha=330e277 · certification: callback_subscription_matrix split into supported callback_subscription_core and future-owned callback_subscription_ecosystem`

and the state must flip `in progress → merged`. There is a "Record M10a signal closeout" commit (f838c15db) but no corresponding M10b closeout recording. AGENTS.md requires docs be updated with status/PR links after each item — until that lands, M10 is not complete.

**2. (Verify) Possible stale `opaque_resource_matrix` reference in retained `_sifr.python`.**
The `_sifr.python` manifest row lists `opaque_resource_matrix` in `certification_rows`, but the handoff table states M3 split that broad row into `opaque_resource_core` + `opaque_resource_ecosystem`. If the broad row no longer exists, this is a dangling reference. It is **pre-existing (M3-era), M11-scoped, and not introduced by M10**, so it is not an M10 blocker — but since it touches "retained manifest state" it is worth confirming during this closeout rather than carrying it silently into M11. (I could not confirm against the actual opaque-resource rows, which weren't in the provided evidence.)

### Bottom line
The signal boundary and the callback core/ecosystem split are correctly executed and internally consistent. **The only gap holding M10 back is the closeout record itself** — flip the tracker to `merged` and add the M10b certification-split evidence cell. Once that lands (and finding #2 is confirmed benign), M10 is PR-ready.
