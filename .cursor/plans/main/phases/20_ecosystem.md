# Ecosystem

**Why now:** With the language feature-complete, developer tools available, and packages manageable, the final step is building the ecosystem infrastructure that enables community growth: a package registry for sharing code, incremental compilation for fast iteration, a REPL for interactive exploration, and compile-time metaprogramming for advanced patterns. This is the capstone phase that turns Sifr from a language into a platform.

---

## milestone_metaprogramming: Compile-Time Decorators

status: pending

**Goal:** Support compile-time code generation and advanced decorators. This builds on Phase 13's `milestone_auto_init` which provides the baseline (`__init__`, `__eq__`, `__str__`); `@dataclass` adds the advanced features.

**Depends on:** milestone_dev_tooling (the full language, web stack, and developer tools should be complete)

### Work Items

- `@dataclass` decorator: explicit opt-in that adds `__hash__`, `__lt__`/`__le__`/`__gt__`/`__ge__` (ordering), `frozen=True` support, and `field()` configuration on top of the auto-init from Phase 13's milestone_auto_init
- Custom decorators: user-defined compile-time AST transforms
- Positional-only parameters (`def f(x, /, y)`)

### Definition of Done (milestone_metaprogramming)

- `@dataclass` adds ordering, hashing, frozen support, and field configuration
- Custom decorators can transform class definitions
- Positional-only parameters work
- All existing E2E tests still pass (no regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- E2E pass tests: dataclass_ordering, dataclass_frozen, custom_decorator, positional_only_params
- Milestone demo in `./demos/milestone_metaprogramming_demo.sifr`

---

## milestone_ecosystem: Package Ecosystem

status: pending

**Goal:** Build the infrastructure for sharing and reusing Sifr code, plus interactive development tools.

**Depends on:** milestone_metaprogramming (language features should be fully complete before the registry launches)

### Work Items

- Package registry (`sifr.dev`): publish, search, and install Sifr packages
- Incremental compilation: skip unchanged modules for faster iteration
- REPL (`sifr repl`): interactive expression evaluation with type display

### Definition of Done (milestone_ecosystem)

- `sifr publish` uploads packages to `sifr.dev`
- `sifr add <package>` resolves and installs from the registry
- Incremental compilation skips unchanged modules
- `sifr repl` provides interactive expression evaluation with type display
- All existing E2E tests still pass (no regressions)
- `cargo test` passes, `cargo clippy -- -D warnings` passes, no new `unsafe` without justification
- Fuzz testing for parser and type checker (cargo-fuzz or afl) — as required by architecture contract (CI Quality Gates, milestone_ecosystem)
- Milestone demo: a complete web application built entirely in Sifr, published as a package

---

## Milestone Ordering

- **milestone_metaprogramming first:** Compile-time decorators complete the language feature set before the ecosystem launches.
- **milestone_ecosystem last:** The registry and REPL are the final ecosystem pieces, building on everything before them.
