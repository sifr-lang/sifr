

Based on my thorough review comparing the plan against the existing Phase 37 implementation, I found **three blockers** where the plan correctly identifies the design goal but the implementation diverges in ways that would make the design principle unenforceable.

---

## NOT READY — 3 Blockers Required Changes

### BLOCKER 1: Plan Specifies ONE Code, Implementation Has TWO

**Plan (line 611):**
> "Cargo process failures receive a single stable wrapper diagnostic, `SIFR-PACKAGE-0101`."

**Plan (line 615):**
> "Sifr must not add a new Sifr diagnostic code for every Cargo resolver, registry, Git, credential, feature, or publish error."

**Implementation reality (`codes.rs:171,176`):**
```rust
pub const PACKAGE_CARGO_COMMAND_FAILED: Self = Self::new("SIFR-PACKAGE-0101", ...);
pub const PACKAGE_CREDENTIALS_UNAVAILABLE: Self = Self::new("SIFR-PACKAGE-0105", ...);
```

The implementation already splits Cargo process failures into two distinct codes — a generic wrapper (0101) and a credential-specific sub-code (0105). This is precisely the Cargo taxonomy inheritance the plan says to avoid.

**Required change:** Add explicit policy to the Cargo Failure Boundary section:

```
Diagnostic code policy for Cargo failures:
- SIFR-PACKAGE-0101 is the sole stable wrapper for all Cargo process failures.
- Credential-related Cargo failures (401, 403, auth errors) do NOT get a separate
  Sifr diagnostic code. They are wrapped in SIFR-PACKAGE-0101 with the redacted
  stderr excerpt and a Sifr-owned help message directing users to Cargo authentication.
- SIFR-PACKAGE-0105 (or any 01xx sub-code) must not be added for Cargo error variants.
- Rationale: credential failures are Cargo's domain. Sifr's role is redaction + routing,
  not classification of Cargo's own error taxonomy.
```

---

### BLOCKER 2: Plan Requires Fields the Implementation Doesn't Carry

**Plan (line 612):**
> "The wrapper includes the Cargo subcommand, current directory, redacted arguments, exit status, lock/network mode, and a redacted excerpt of Cargo stderr/stdout."

**Plan (lines 642-648):**
> "Diagnostics must include: package name and resolved package instance when available; dependency alias when relevant; source kind (`path`, `git`, `registry`); lock/network mode; exact recovery command when possible."

**Implementation reality (`diag/mod.rs:41-43`):**
```rust
pub enum PackageDiagnosticOrigin {
    CargoCommand {
        action: String,  // only this field
    },
    // ... other variants
}
```

The `PackageDiagnostic` struct only carries `code`, `message`, `origin`, and `help`. The plan's required fields (current directory, exit status, lock/network mode, source kind, dependency alias) are not present in the diagnostic data model. The diagnostic docs for 0101 don't document these fields either.

**Required change:** Add machine-readable diagnostic fields specification:

```
SIFR-PACKAGE-0101 machine-readable fields:
- action: the Cargo subcommand that failed (metadata | fetch | build | test | ...)
- current_dir: absolute path of the directory where Cargo was invoked
- exit_status: raw i32 exit code
- lock_mode: the Sifr lock mode in effect (unlocked | locked | offline | frozen)
- network_mode: online | offline
- redacted_excerpt: redacted Cargo stderr/stdout (first N lines or truncated)
- source_kind: registry | git | path (when determinable before invocation)

These fields are surfaced in JSON/machine-readable output. Human-readable output
shows the redacted excerpt. The package name, dependency alias, and source kind
come from PackageDiagnosticOrigin::CargoMetadata when available.
```

---

### BLOCKER 3: Redaction Policy Underspecified — Overbroad and Underinclusive Simultaneously

**Plan (line 612):**
> "The wrapper includes the Cargo subcommand, current directory, redacted arguments, exit status, lock/network mode, and a redacted excerpt of Cargo stderr/stdout."

**Implementation (`errors.rs:67-78`):**
```rust
fn redact_word(word: &str) -> &str {
    if word.starts_with("token=")
        || word.starts_with("Bearer")
        || word.starts_with("gho_")
        || word.starts_with("cargo:token")
        || word.contains("://")  // <-- overbroad: catches legitimate error context
    {
        "[redacted]"
    } else {
        word
    }
}
```

Issues with current redaction:

1. **Overbroad**: `.contains("://")` redacts the entire word if it contains any URL pattern — including the hostname, path, error descriptions that happen to mention URLs. A Cargo error like `"failed to fetch from https://crates.io"` becomes `"failed to fetch from [redacted]"` — losing the useful signal that this is a registry issue.

2. **Underinclusive**: The credential patterns don't cover `secret=`, `password=`, `api_key=`, `x-token:`, or GitHub App tokens (`ghs_`, `ghp_`). The current list is a hard-coded allowlist that will rot.

3. **No URL host redaction**: Redacting credentials but not hosts means `https://user:***@private-registry.example.com` leaks `private-registry.example.com` — a security issue if this appears in CI logs.

4. **No stdout redaction**: The plan says "Cargo stderr/stdout" but `map_cargo_failure` only accepts `stderr: &str`. If Cargo ever emits credentials on stdout, they leak.

**Required change:** Add explicit redaction specification:

```
Credential and sensitive data redaction for SIFR-PACKAGE-0101:
- Credential patterns (case-insensitive):
  token=, bearer, gho_, ghp_, ghs_, ghr_, cargo:token, secret=, password=, api_key=, x-token:
- URL host redaction: in any URL, redact the host portion if the URL also contains
  recognized credential patterns. Retain the scheme and path for error signal.
  Example: "https://user:token@private.example.com/pkg" → "https://[redacted host]/pkg"
  Example: "https://crates.io/api/v1/crates" → "https://crates.io/api/v1/crates" (no change)
- Stderr only by default. Stdout is captured only when stderr is empty and the command
  may emit credentials there (e.g., cargo login).
- Words matching credential patterns are replaced with [redacted].
- The full error context (file paths, line numbers, registry names) is preserved.
- Recovery help for credential failures: "authenticate with `cargo login` or configure
  CARGO_REGISTRIES_* credentials, then retry."
```

---

### Additional Observations (Non-Blocking but Worth Noting)

1. **Existing 0105 docs are orphaned**: `docs/errors/SIFR-PACKAGE-0105.md` exists but `codes.rs` shows no constant for it. The existing docs won't be consistent with either resolution of BLOCKER 1.

2. **Plan says "exit status" but `CargoAction` has no exit status field**: `cargo_command_failed` and `credentials_unavailable` don't receive an exit code. This needs to be added to the wrapper function signature.

3. **`sifr --explain` behavior is underspecified for 0101**: The plan says explain accepts any diagnostic code but doesn't specify whether explaining 0101 shows the underlying Cargo stderr excerpt or just the redaction-reminder message. Users reading `--explain` for a Cargo failure need to understand they're seeing a wrapper, not the full story.

---

### Summary

| Blocker | Issue | Impact |
|---|---|---|
| 1 | Plan: 1 code. Implementation: 2 codes. | If not fixed, the design principle "must not add a code for every Cargo error variant" is unenforceable. |
| 2 | Plan: 8+ required fields. Implementation: 1 field. | Machine-readable output can't support the plan's stated requirements without schema changes. |
| 3 | Redaction is overbroad (kills error signal) and underinclusive (misses real credentials). | Security/privacy risk AND degraded DX from lost context. |

Each blocker requires a concrete addition to the Cargo Failure Boundary section before this design is production-grade.
