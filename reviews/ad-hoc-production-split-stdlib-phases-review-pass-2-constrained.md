**FAIL** — four remaining gaps.

---

**1. asyncio core synchronization primitives are unowned**

Remediation designates "core asyncio scheduler/task helpers" as prior infrastructure but assigns ownership to no phase and no doc. `asyncio.gather`, `asyncio.wait`, `asyncio.sleep`, `asyncio.timeout`, `asyncio.TaskGroup`, `asyncio.Event`, `asyncio.Lock`, `asyncio.Semaphore`, and `asyncio.Condition` are neither network entries nor Queue — they fall into the gap between concurrency/runtime (Queue only) and network/web (open_connection/start_server only). No phase has a completion criterion that covers them. If any downstream phase (network/web's executor-backed serving, concurrency/runtime's async Queue) is allowed to depend on them as prior infrastructure, there must be a named milestone or doc section that owns their scope, even if that scope is "host-runtime, no parity target." Without it, the prior-infrastructure claim is unverifiable and blocks no gate.

**Action:** Assign asyncio synchronization/control-flow primitives explicitly — either to concurrency/runtime's scope, to a named prior milestone, or declare them host-runtime with intentional-diff status in each consuming doc.

---

**2. Binary file I/O as prior infrastructure is equally unowned**

Text/i18n correctly scopes itself to text-mode `open(encoding/errors)` and calls binary file I/O "prior infrastructure." But no phase doc owns binary open. This is the same structural defect as the asyncio core gap: text/i18n's own entry point has an unresolved hard dependency with no owning milestone. A consumer reading the text/i18n doc cannot determine which milestone to block on before text-mode open is deliverable.

**Action:** Name the milestone or crate that owns binary file I/O, even briefly, so text/i18n's dependency chain is complete.

---

**3. ThreadPoolExecutor "adapted/waived" is an unresolved decision, not a boundary**

Concurrency/runtime says ThreadPoolExecutor behaviors requiring threading objects are "adapted/waived" but does not enumerate which behaviors are adapted (and how), which are waived, and who decides. Network/web depends on concurrency/runtime for executor-backed serving. If network/web uses `loop.run_in_executor` with the default ThreadPoolExecutor, the adaptation/waiver decision directly affects what network/web can claim about its observable behavior. An unenumerated waiver at a phase boundary is equivalent to having no boundary.

**Action:** List at minimum the top two or three ThreadPoolExecutor behaviors that are adapted or waived (e.g., `initializer`/`initargs`, `cancel_futures` on shutdown, worker-thread naming), and state whether they become intentional-diffs or are simply deferred.

---

**4. urllib.parse non-ASCII URL behavior has no stated disposition**

Network/web constrains urllib.parse to byte/ASCII/UTF-8 and says it must not duplicate the codec registry. Python's `urllib.parse.quote`/`unquote` with non-UTF-8 encodings consumes the codec registry via `str.encode(encoding)`. The remediation correctly excludes codec duplication but does not state whether non-ASCII, non-UTF-8 URL encoding is (a) intentional-diff, (b) blocked on text/i18n, or (c) out of scope entirely. Without a disposition, any future work touching `quote(string, encoding='latin-1')` will relitigate the boundary.

**Action:** Add a one-line disposition for non-ASCII/non-UTF-8 URL encoding — either "intentional-diff: unsupported" or "blocked on text/i18n codec registry milestone."
