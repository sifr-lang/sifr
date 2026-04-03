## Review: wave2 batch11 pass1

### Severity: Low — API Clarity / Demo Polish

#### Finding 1: `subprocess/idiomatic.rs` — Dead constants mislead intent
**Severity:** Low  
**Location:** Lines 4–6 (`PIPE`, `STDOUT`, `DEVNULL`)

Three constants are defined but never referenced. They appear to mirror Python's `subprocess.PIPE/STDOUT/DEVNULL` sentinel values, but no code path uses them. The demo validates `run`, `check_call`, `check_output` only. These constants create a false impression of an API surface that doesn't exist — they suggest API parity where none is implemented.

**Impact:** Minor. No behavioral regression; purely misdirects a reader expecting the constants to participate in the demo.

---

#### Finding 2: `zipfile_io/idiomatic.rs` — `ZipFile::open` rejects `"rb"` but API inconsistency is deliberate-in-demo
**Severity:** Low  
**Location:** `ZipFile::open` (line ~152), assertion `open_rejected`

The demo explicitly validates that `reader.open("bin/raw.bin", "rb")` fails. This is intentional — the API is shaped to only accept `"r"` for text-mode reads. However, `read_bytes` returns `Vec<u8>` regardless, and the error message `"ZipFile.open supports only text read mode"` reveals internal mode distinctions rather than a user-facing constraint.

**Impact:** Low. Behavior is deliberate (the demo asserts it), but the error message leaks implementation framing.

---

#### Finding 3: `zipfile_io/idiomatic.rs` — `NamedTemporaryFile` lacks drop semantics
**Severity:** Low  
**Location:** `NamedTemporaryFile` struct and impl

`NamedTemporaryFile` requires explicit `.close()` then `.cleanup()` — no `Drop` impl guarantees cleanup on scope exit. The demo explicitly calls both, so behavior is demonstrated. But any consumer who forgets to call cleanup leaves a temp file on disk.

**Impact:** Low in demo context; moderate if this were a real API. The demo intentionally shows the explicit pattern.

---

### Accepted

All three files are **accepted**. Rationale:

- No behavioral regressions: all demos pass their self-contained assertions.
- Demo-visible behavior is correct and reproducible.
- The findings are cosmetic or API-shape notes that don't affect the demo's validation surface.
- Dead constants in `subprocess` and the `open` mode restriction in `zipfile_io` are consistent with "Rust-first companion" intent — they show idiomatic Rust shapes without Python-like generality.

The unused constants in `subprocess` and explicit-only cleanup in `zipfile_io` are acceptable within this batch's demonstrated scope.

---

### Final Verdict

**Accepted.** These are sound idiomatic Rust companions for the wave2 batch11 demo corpus. The low-severity notes above are on record but do not warrant a pass revision.
