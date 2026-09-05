# Rust interop certification 6 review — round 1

Date: 2026-07-28

Reviewer: agent (`--effort medium`)

Verdict: **NOT SATISFIED**

Scope: the complete certification-6 diff against `origin/main`, its issue-plan
acceptance criteria, and surrounding callback, panic, opaque-handle, runtime,
matrix, and stable-support code.

## Findings

1. **Blocker:** the negative generated-build test emitted two unrelated
   `SIFR-CLASS-0006` diagnostics because its placeholder `Subscription` class
   body was invalid, so its exact-one-diagnostic assertion failed.
2. **High:** retained callbacks did not reject packages or targets compiled with
   `panic = "abort"`, making the documented per-invocation panic containment
   unenforceable.
3. **Medium:** callback capture validation and the generated `Send + Sync +
   'static` backstop did not cover method-hosted retained callback declarations.
4. **Medium:** the runtime scenario asserted callback policy equality but used
   hard-coded queue capacity and shutdown output, decoupling behavior from the
   declared policy.
5. **Low:** cancellation evidence observed a synthetic sleeper rather than
   preventing an in-flight callback delivery from running.
6. **Low:** callback code generation silently fell back to the raw handler when
   policy rendering failed and duplicated a laxer policy parser.
7. **Low:** the non-provable callable-value and non-named-handler rejection
   branches lacked tests.
8. **Low:** `crates/sifr_runtime/src/interop.rs` reached 898 of the allowed 900
   lines and needed responsibility-based decomposition.

The reviewer also verified that the typed thread-safe bridge, owned convention,
mutable rejection, opaque subscription handle, real WebSocket/Redis/notify
runtime integrations, offline lock discipline, fixture inventories, and
scenario mutation self-tests were otherwise sound.

## Required outcome

Address every finding, run focused and authoritative validation, and submit the
exact updated head to another agent review round.
