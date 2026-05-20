# Review: Cargo Failure Boundary — Pass 2

Pass 1 found 3 blockers. This review assesses whether the updated plan (lines 606–700) resolves them and whether it is production-grade for long-term maintainability.

---

## PASS 1 BLOCKER STATUS

### BLOCKER 1: Credential-Specific Codes Not Explicitly Retired

**Status: RESOLVED**

The plan now explicitly states (lines 616–618):

> "Credential-related Cargo failures, including `401`, `403`, auth-helper failures, `cargo login` failures, missing registry tokens, and Git credential failures, are still wrapped in `SIFR-PACKAGE-0101`.
> Existing or older credential-specific Cargo-failure codes such as `SIFR-PACKAGE-0105` must be retired, documented as superseded, or mapped to `SIFR-PACKAGE-0101` during this phase."

This satisfies the pass 1 requirement. The plan now explicitly prohibits credential-specific sub-codes and mandates retirement of `SIFR-PACKAGE-0105`.

---

### BLOCKER 2: Required Fields But No Machine-Readable Schema

**Status: RESOLVED**

The plan now includes a complete machine-readable field specification (lines 637–654) covering all pass 1 requirements:
- `action`: subcommand
- `current_dir`: absolute path
- `args_redacted`: redacted Cargo argument vector
- `exit_status`: process exit code
- `lock_mode`: unlocked | locked | offline | frozen
- `network_mode`: online | offline
- `package`: selected package name
- `package_instance`: resolved instance id
- `dependency_alias`: importing dependency alias
- `source_kind`: path | git | registry | unknown
- `stderr_redacted`: bounded redacted stderr
- `stdout_redacted`: bounded redacted stdout, included only when relevant
- `help`: Sifr-owned recovery hint

The schema aligns with the pass 1 requirement that diagnostic fields should include package context, dependency alias, source kind, lock/network mode, and recovery command.

---

### BLOCKER 3: Redaction Policy Underspecified

**Status: RESOLVED**

The plan now specifies (lines 670–680):

- **Credential patterns** (case-insensitive): `token=`, `bearer`, `gho_`, `ghp_`, `ghs_`, `ghr_`, `cargo:token`, `secret=`, `password=`, `api_key=`, `x-token:`
- **URL redaction with host preservation**: `https://user:token@private.example.com/pkg` → `https://[redacted host]/pkg`
- **Public registry visibility**: `https://crates.io/api/v1/crates` remains unredacted
- **Output streams**: stderr captured by default; stdout captured only when stderr is empty or operation is known to emit relevant failure text there
- **Bounding**: line count and byte count limits
- **Test requirements**: both overbroad and underinclusive cases

This addresses pass 1's concern about overbroad URL redaction (`://` matching) and underinclusive credential patterns.

---

## LONG-TERM MAINTAINABILITY ASSESSMENT

### Cargo Taxonomy Inheritance — AVOIDED

The plan explicitly prevents Cargo taxonomy inheritance:
- One stable wrapper (`0101`) for all Cargo process failures
- Specific Sifr codes reserved for Sifr-definable, Sifr-validatable failures (projection drift, privacy, trust policy, archive preflight, etc.)
- Explicit statement: "Sifr must not add a new Sifr diagnostic code for every Cargo resolver, registry, Git, credential, feature, or publish error"

**Assessment: Pass.** The boundary is clean.

### Security and Redaction — SOUND

The redaction spec (lines 670–680) is production-grade:
- Case-insensitive credential matching
- Host-preserving URL redaction (preserves scheme + path, redacts userinfo + host)
- Public registry visibility preserved
- Bounds prevent unbounded diagnostic growth

**One observation (non-blocking):** The credential pattern list is explicit but will require maintenance as Cargo/Git credential helpers evolve. The plan's test requirement ("include both overbroad and underinclusive cases") provides regression coverage but doesn't prevent pattern list rot. Consider adding a scheduled review note in the guardrails doc or an explicit maintenance note in the boundary section.

### Production-Grade DX — SOUND

Human-readable output (lines 656–661) and `sifr --explain` behavior (lines 662–669) are well-specified:
- Shows wrapper heading + stable context
- Shows redacted Cargo excerpt under "Underlying Cargo failure" label
- Explains that `0101` is a wrapper, not Cargo error reinterpretation
- Explains redaction behavior
- Gives generic next steps including `cargo login` for credential failures
- Explicitly does NOT list every possible Cargo failure mode

**Assessment: Pass.** The DX contract is clear and avoids the trap of trying to re-interpret Cargo diagnostics.

---

## MINOR OBSERVATIONS (Non-Blocking)

### 1. Credential Pattern Gap: `gh_` Prefix

The credential patterns (L673) include `gho_`, `ghp_`, `ghs_`, `ghr_`. These are GitHub OAuth, personal access, server, and refresh token prefixes respectively. However, GitHub also uses `gh_` as a generic prefix for some credential types. If `gh_` credentials appear in Cargo/Git error output, they would not be redacted by the current list.

**Recommendation:** Consider adding `gh_` as a prefix pattern or clarify that `ghs_`/`ghr_` cover the full GitHub token family. Non-blocking for pass 2 — this is a pattern list maintenance issue to address in implementation.

### 2. Credential Code Retirement: No Formal Process

The plan (L617–618) says credential codes "must be retired, documented as superseded, or mapped." The plan doesn't specify the mechanism:
- How is retirement documented?
- Where does the supersession mapping live?
- How are existing uses of `SIFR-PACKAGE-0105` handled in tests and docs?

**Assessment:** This is a pass 2 gap, but not a blocker. The plan's intent is explicit. The mechanism can be defined in `milestone_adhoc_pkg_3` implementation scope or in the guardrails extension. Non-blocking for design acceptance.

### 3. `source_kind: unknown` Underspecified

L650 defines `source_kind` as `path | git | registry | unknown`. The plan doesn't specify what causes `unknown` or how it should be handled in human-readable output.

**Assessment:** Non-blocking. `unknown` is a sensible default for cases where source kind cannot be determined before Cargo invocation. The field is optional ("when available") and doesn't affect the security of the redaction layer.

### 4. `stdout_redacted` Scope Is Narrower Than Redaction Spec

L679 (redaction section) says "stderr is captured by default" and stdout is captured only when stderr is empty or the operation emits relevant text there. But the machine-readable spec (L652) says `stdout_redacted` is included "only when relevant." These are consistent but the relationship could be clearer. Non-blocking.

---

## SUMMARY

| Requirement | Status |
|---|---|
| Pass 1 BLOCKER 1: Explicit credential code retirement | ✅ Resolved |
| Pass 1 BLOCKER 2: Machine-readable field schema | ✅ Resolved |
| Pass 1 BLOCKER 3: Redaction spec (overbroad + underinclusive) | ✅ Resolved |
| Cargo taxonomy inheritance avoided | ✅ Pass |
| Security and redaction sound | ✅ Pass (with pattern maintenance observation) |
| Production-grade DX | ✅ Pass |

---

## FINAL ASSESSMENT

**READY — No blockers remain.**

The Cargo Failure Boundary section is production-grade:

1. **One wrapper, explicit retirement**: `SIFR-PACKAGE-0101` is the sole wrapper for all Cargo process failures. Credential codes are explicitly mandated for retirement/supersession/mapping.

2. **Complete machine-readable schema**: All required fields are specified with types, meanings, and optionality indicators. JSON/machine output can be validated against this spec.

3. **Sound redaction policy**: Case-insensitive credential patterns, host-preserving URL redaction, public registry visibility preserved, bounds specified, test requirements explicit.

4. **Clean boundary**: Specific Sifr codes are reserved for Sifr-definable failures. Cargo error taxonomy is not inherited.

**Non-blocking observations** (implement in scope of `milestone_adhoc_pkg_3`):
- Consider adding `gh_` to credential pattern list
- Document credential code retirement mechanism in milestone scope
- Define `source_kind: unknown` fallback handling

The plan is ready for implementation.