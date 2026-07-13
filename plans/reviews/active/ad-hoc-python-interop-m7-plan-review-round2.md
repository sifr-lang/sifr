## Round 2 Verification

Checked the revised M7 wave plan (7 waves) against every Round 1 blocking finding and every prior refinement listed by the user.

### Blocking findings — status

**§1 wave ordering vs. Delivery Rule.** Wave 1 is reworded from "activate" to "Prepare coroutine and async-close frontend contracts behind the existing `SIFR-PYRES-0002` gate"; Waves 5–6 explicitly state "behind the public gate"; Wave 7 is the single atomic activation ("Lift the `@python.coroutine` and `cleanup=async_close` gates only after the owned loop, typed wrappers, cooperative cancellation, terminal shutdown, and consuming lifecycle are present in the same production path"). This now mirrors M6 Wave 3's substrate-first + atomic-activation pattern. Resolved.

**§2 cancellation carrier vs. Tokio-abort supervisor.** A concrete task-local carrier/claim design is chosen: at Python-await entry the submission atomically claims the carrier and registers its exact-task hook; a claimed cancellation signals the exact asyncio task and terminally waits on the child + Python terminal latch; an unclaimed cancellation retains Tokio abort; the registration race is explicitly closed ("aborts before Python submission or is observed by the newly registered submission, never leaving untracked Python work"). The supervisor cutover is split as requested: Wave 3 covers direct task paths (`task.cancel`, cancel-and-join, timeout); Wave 4 covers scope/group fail-fast, race/select losers, and join-set. Resolved.

**§3 async-close split.** Wave 1 explicitly keeps `cleanup=async_close` gated ("Keep `cleanup=async_close` gated until its runtime lifecycle is complete"); Wave 6 completes the consuming runtime lifecycle (transfer, close-once, poison, reject reuse/duplicate/abandonment, cancellation, shutdown interaction) behind the gate; Wave 7 lifts the gate atomically alongside `@python.coroutine`. Runtime completion precedes activation. Resolved.

### Prior refinements — status

- **§4 Conditional loop bootstrap** — Wave 2: "Wire loop bootstrap only when the resolved target uses an async Python declaration or the raw coroutine intrinsic." Addressed.
- **§5 Explicit Bodyless-as-Suspends mechanism** — Wave 1 names option (ii): "Mark bodyless async interop declarations as `Suspends` in the existing suspension summary so they bypass normal body lowering and the `NoSuspend` fake-async diagnostic without adding a summary variant or removing their ordinary async function identity." Addressed.
- **§6 Typed-wrapper loop identity proof** — Wave 5: "Prove two concurrent typed wrappers observe the same loop/thread identity, while the syntax remains gated until cancellation and cleanup are complete." Addressed.
- **§7 M9 no-op shutdown slot** — Wave 4: "stop external admissions; invoke the M9 callback shutdown hook (a no-op ordered slot until M9); run registered async cleanup while the loop is live; cancel and terminally drain remaining submissions; stop the loop; join its thread." Ordering fixed now so M9 does not later reshape it. Addressed.
- **§8 Preserved `blocking_io` raw API** — Wave 2: "Replace raw `asyncio.run` with owned-loop submission while preserving the raw API's synchronous `blocking_io` classification and explicit-offload requirement." Addressed.

### Remaining actionable defects

None found. The plan is internally consistent, the seven-wave decomposition matches the Delivery Rule, the cancellation design is concrete, and the runtime substrate for both coroutines and `async_close` lands before the single activation wave.

SATISFIED
