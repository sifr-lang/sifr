I've inspected all the listed files. The user's round-4 changes are in place and consistent — the dedup of helpers, the real proc-macro stub with `#[proc_macro_derive(SifrGenerated)]` plus `[lib] proc-macro = true`, and the new structural assertions in `_scenario_checks.py` for proc-macro libs, build scripts, and native `links`. Below are remaining findings ordered by severity.

## Findings

### Medium — real gaps in checker rigor

1. `verification/areas/rust_interop/checks/_scenario_checks.py:213-216` (`_reject_generated_bridge_imports`) only rejects lines that **literally start with** `use crate::__sifr_bridge`. The following all slip through: `pub use crate::__sifr_bridge::X`, `pub(crate) use crate::__sifr_bridge::*`, an aliased re-export like `use crate::__sifr_bridge as bridge_internal`, or non-`use` references (`type X = crate::__sifr_bridge::Y;`, `impl crate::__sifr_bridge::Trait for ...`). The check is supposed to enforce the "shared bridge crate must not pull in package-generated bridge types" invariant; right now only the most obvious form is blocked.

2. `verification/areas/rust_interop/checks/_scenario_checks.py:374-388` (`_require_dependency_features`) accepts a feature list that is a **superset** of the required features — it tests `any(feature not in features for feature in expected_features)` but never asserts the actual feature set equals the expected one. The package-example side (`check_fixture_matrix.py:228-249`) uses exact-equality (`actual != expected`). A scenario can silently drift to `features = ["env-filter", "experimental", "registry"]` without failing the scenario checker.

3. `verification/areas/rust_interop/checks/_scenario_checks.py:174-202` (`_validate_scenario_sifr_source`) does not enforce the `# execution-kind:` and `# expected-result:` headers on scenario `.sifr` sources. Package-example sources require those headers explicitly (`check_fixture_matrix.py:374-383`), and every scenario `main.sifr` already includes them — so it's a consistency gap rather than a current failure, but a future scenario could omit them.

### Low — cosmetic / latent

4. `verification/areas/rust_interop/fixtures/shared_bridge_crate/examples/shared_hash_bridge/rust/sifr_shared_hash_bridge/src/lib.rs:18` — the token requirement `"crate::__sifr_bridge"` (`_scenario_checks.py:47`) is satisfied solely by a self-referential comment ("A shared bridge crate may mention `crate::__sifr_bridge` in comments…") added to clear the check. Combined with finding #1, the "must mention + must not import" pair is currently passed by metaprose about the rule itself rather than by exercising the contract.

5. `verification/areas/rust_interop/fixtures/cargo_locked_offline/examples/locked_offline_cache/Cargo.lock:1-3` — the token `"--frozen"` (`_scenario_checks.py:21`) is satisfied by free-form text in the `Cargo.lock` header comment. The user explicitly reports the runner invokes `cargo check --locked --offline` (no `--frozen`). Either drop `--frozen` from the required tokens or extend the runner to actually pass it.

6. `verification/areas/rust_interop/checks/_binding_helpers.py:45-46` (`decorated_function_return_type`) does `stripped.split("->", maxsplit=1)[1]` with no guard. A `@rust(...)` decorator over a `def foo(...): ...` declaration with no return annotation will raise `IndexError` instead of producing a structured failure. No current binding hits this, but a single future entry without `->` crashes `python3 verification/areas/rust_interop/checks/check_fixture_matrix.py`.

7. `verification/areas/rust_interop/checks/_binding_helpers.py:49-54` (`verifier_binds_call`) treats any `=` in the call prefix as a binding. A line like `if x == bound_function(y):` produces `before_call = "    if x == "`, which contains `=` and doesn't start with `return `, so the check returns True even though no value is bound. The intent of the check ("verifier must bind result before returning") is undermined by `==` and `!=` comparison forms. A safer test is a single trailing `=` not preceded by `=`/`<`/`>`/`!`.

If none of the above is worth chasing in this round, the examples + checker materially meet the completeness request: every scenario `cargo check` passes, every required scenario manifest property is asserted, and the shared helpers are no longer duplicated. The remaining items are bounds-tightening, not correctness failures.
