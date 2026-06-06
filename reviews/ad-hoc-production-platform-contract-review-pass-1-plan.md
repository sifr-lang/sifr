**FAIL**

The plan has the right shape — the contract doc, shared host matrix, golden fixtures, and three-phase patch are all necessary. But six gaps prevent it from being implementation-ready. Exact changes required:

---

### 1. Terminal state vocabulary must be defined in the contract doc, not just named as a section

The three phases use incompatible state vocabularies today:

| Concept | text/i18n | concurrency | network |
|---|---|---|---|
| CPython test result | `adopted-as-substrate-fixture`, `adapted-for-sifr-api`, `adapter-deferred`, `waived`, `rejected` | `adopted`, `adapted`, `waived` | `mined`, `blocked`, `rejected`, `external-signal` |
| Surface stability | `done`, `intentional-diff`, `unsupported`, `host-limited`, `deferred-adapter` | `done`, `intentional-diff`, `unsupported`, `host-limited`, `adapter-later`, `deferred`, `rejected` | `production-public`, `production-substrate`, `internal-test`, `deferred`, `rejected`, `blocked-on-*`, `host-limited` |
| Support tier | `production-substrate`, `future-adapter`, `host-limited`, `rejected`, `deferred` | `production-substrate`, `production-public`, `internal-runtime`, `adapter-later`, `deferred`, `rejected` | (same field, different terms) |

The contract doc must include the **authoritative unified tables**, and the patch to each phase doc must replace phase-local definitions with a single reference to the contract. The plan currently says "include a section for standardized terminal states" but doesn't resolve the vocabulary. Without the resolved vocabulary, phase owners will diverge again at M0.

---

### 2. Per-phase host matrix paths create a naming conflict

The concurrency phase doc already mandates creating `verification/stdlib/concurrency_runtime_supported_host_matrix.md` at M0 (line 674 of that doc). The proposed shared contract would produce `verification/platform/supported_host_matrix.md`. The patch to the concurrency doc must **explicitly supersede** the per-phase path and redirect M0 to populate the shared matrix instead. Without this, two matrices will diverge during implementation.

---

### 3. Golden fixture execution mechanism is unspecified

"Executable golden fixtures" must state:
- **How they run**: `cargo run -q -p sifr -- run verification/platform/golden/<name>.sifr`
- **What "checks" means**: minimally, exit code 0 plus optional stdout regex match; the manifest fields must be concrete (`command`, `expected_exit`, `expected_stdout_contains`)
- **When they must pass**: fixtures gated by `blocked_until: concurrency-runtime-m1` must be skipped (not failed) until that milestone closes; all non-blocked fixtures must pass at each phase's final milestone
- **Integration**: either added to `scripts/run_e2e_pass.sh` as a named fixture group, or a new `scripts/run_platform_golden.sh` called from `scripts/run_all_tests.sh`

Without this, "cross-phase executable golden fixtures" will be treated as docs fixtures or demos, not as regression gates.

---

### 4. `platform_contract.*` is an ambiguous wildcard

Every other inventory artifact in the codebase uses `.md` + `.json` pairs (e.g., `text_i18n_substrate_inventory.md`, `concurrency_runtime_substrate_inventory.json`). The plan must specify `platform_contract.md` (human-readable) + `platform_contract.json` (machine-readable schema, so CI can validate state completeness), matching the established pattern.

---

### 5. Security/resource-exhaustion table needs the concern categories listed

The plan says "include a security/resource-exhaustion ownership table" but leaves it empty. For the table to be implementable at M0, the plan must enumerate the concerns and their owning phase:

| Concern | Owner phase |
|---|---|
| Buffer/body size limits, DoS parser input | network |
| Connection and task count limits | network + concurrency |
| TLS certificate verification authority | network |
| Codec amplification attacks | text/i18n |
| Malicious `.mo` catalog / plural expression | text/i18n |
| Subprocess resource limits and `@shell_exec` security surface | concurrency |
| IPC payload size and panic-free malformed-message handling | concurrency |
| Cancellation storm / task explosion | concurrency |

Without this seed table, M0s will independently define their scope and leave gaps in cross-phase resource limits.

---

### 6. Ordering of the shared contract relative to individual M0s is unaddressed

The three phases have a strict execution order (text/i18n → concurrency → network). The shared contract must be created before any **M1** starts, but the plan does not say:
- Is the shared contract a prerequisite for text/i18n M0, or for all three M0s simultaneously?
- Does the shared contract require its own external review PASS before text/i18n M1 opens?

Recommended fix: the shared contract is created as a new step **before** text/i18n M0 closes (i.e., text/i18n M0 definition-of-done gains an entry: "shared platform contract is checked in and passes external review"). The concurrency and network M0s then only need to verify their sections against the already-approved contract.

---

Implement those six fixes and the plan becomes implementation-ready.
