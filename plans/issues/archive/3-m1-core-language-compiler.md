## M1: Core Language Compiler

---

### 1. Product Requirements

#### **Title**

M1: Core Language -- First Working Sifr Compiler

---

#### **Objective / Problem Statement**

Sifr is a new compiled programming language with Python syntax and enforced static typing that emits Rust source code, compiled via `rustc` into native binaries. There is currently no compiler implementation. We need to build the foundational compiler pipeline that can take a simple `.sifr` source file and produce a working native binary. This is the first milestone (M1) that proves the entire pipeline works end-to-end.

---

#### **Constraints**

| Constraint | Rationale |
| --- | --- |
| Must emit valid Rust source code (not LLVM IR or machine code) | Leverages Rust's mature compiler for optimization and safety |
| Parser must be forked from ruff (MIT licensed) | Battle-tested Python parser, avoids building from scratch |
| Types are enforced (strict mode with opt-in Any) | Core language design decision |
| Move-by-default ownership for non-primitive types | Core language design decision aligned with Rust semantics |
| Built entirely by AI agents | Test suite must be comprehensive and self-documenting |

---

#### **Business Goals & Success Criteria (KPIs)**

| Metric | Baseline (Today) | Target (Post-launch) |
| --- | --- | --- |
| Compilable programs | 0 | Simple programs with functions, if/else, primitives compile to native binaries |
| Test coverage | 0 | 4-layer test suite (unit, snapshot, e2e, corpus) all passing |
| CLI commands working | 0 | `sifr build`, `sifr run`, `sifr check`, `sifr emit` all functional |

---

#### **Scope**

##### Features In

1. Fork and adapt ruff Python parser crates (lexer, parser, AST, trivia, text_size, source_file, literal)
2. Type system with primitives (int, float, bool, str, None), type inference, and type checking
3. HIR (High-level IR) with name resolution and ownership tracking
4. Rust code generation from HIR
5. Driver to orchestrate the full pipeline with error diagnostics
6. CLI binary with build/run/check/emit commands
7. Comprehensive test suite (unit, snapshot, mdtest, e2e)

##### Features Out

| Feature | Reason for Exclusion |
| --- | --- |
| Loops (for, while) | Deferred to M2 |
| Data structures (list, dict, tuple) | Deferred to M2 |
| Error handling (Result/Option) | Deferred to M3 |
| Classes and methods | Deferred to M4 |
| Module system (import) | Deferred to M5 |
| Generics | Deferred to M6 |
| Standard library | Deferred to M7 |
| Async/await | Deferred to M8 |

---

#### **Users / Stakeholders, Use-Cases & Dependencies**

| Persona | Use-Case / Benefit | Dependencies | AC-ID |
| --- | --- | --- | --- |
| Sifr developer | Write a simple program in Python syntax and compile to native binary | Rust toolchain installed | AC-1 |
| Sifr developer | Get type errors at compile time for type mismatches | Type system implemented | AC-2 |
| Sifr developer | Get move/ownership errors at compile time | Ownership tracking in HIR | AC-3 |
| AI agent | Run `cargo test` to verify compiler correctness | Test suite implemented | AC-4 |

---

### **Acceptance Criteria**

| AC-ID | Persona | Criterion |
| --- | --- | --- |
| AC-1 | Developer | **Given** a valid `.sifr` file with functions, if/else, and primitives **When** running `sifr build` **Then** a native binary is produced that runs correctly |
| AC-2 | Developer | **Given** a `.sifr` file with type mismatches **When** running `sifr check` **Then** clear error diagnostics with source locations are shown |
| AC-3 | Developer | **Given** a `.sifr` file that uses a moved variable **When** running `sifr check` **Then** a use-after-move error is reported |
| AC-4 | AI Agent | **Given** the full codebase **When** running `cargo test` **Then** all unit, snapshot, mdtest, and e2e tests pass |

---

## 2. Solution Design

### **2.1 Functional Requirements**

* Parse `.sifr` source files using Python syntax (forked from ruff)
* Type-check all expressions and statements with enforced typing
* Infer types from initializers (e.g. `x = 42` infers `int`)
* Track ownership: move on assignment for `str`, copy for `int`/`float`/`bool`
* Generate valid Rust source code from typed HIR
* Invoke `cargo build` on generated Rust project to produce native binary
* Report errors with source file, line, and column information

---

### **2.2 Non-Functional Requirements**

| ID | Requirement |
| --- | --- |
| NFR-1 | Compilation of small programs (< 100 lines) completes in under 5 seconds |
| NFR-2 | Error messages include source location (file:line:col) and code snippets |
| NFR-3 | Test suite runs in under 60 seconds via `cargo test` |
| NFR-4 | All crates compile with no warnings on stable Rust |

---

### **2.3 High-Level Architecture**

```
Source (.sifr)
    |
    v
[sifr_python_parser] -- Lexer + Parser
    |
    v
[sifr_python_ast] -- Untyped AST
    |
    v
[sifr_hir] -- Name Resolution + Type Checking (uses sifr_type_system)
    |
    v
Typed HIR
    |
    v
[sifr_codegen] -- Rust Source Generation
    |
    v
Generated .rs files + Cargo.toml
    |
    v
[rustc via cargo build] -- Native Binary
```

Orchestrated by `sifr_driver`, exposed via `sifr` CLI.

---

### **2.4 Detailed Component Design**

**sifr_python_parser (forked from ruff)**
Lexer tokenizes `.sifr` source. Parser produces untyped AST. Stripped to M1-relevant nodes only.

**sifr_type_system**
Type enum (Int, Float, Bool, Str, None, Function, Any, Never). Type inference from initializers. Type checking for binary ops, comparisons, function calls. Subtyping rules.

**sifr_hir**
Typed intermediate representation. Name resolution with scope tracking. Every expression carries its resolved Type. Ownership tracking (move vs copy).

**sifr_codegen**
Walks typed HIR and emits Rust source code. Type mapping: int->i64, float->f64, bool->bool, str->String, None->(). Generates Cargo.toml + src/main.rs. Maps print() to println!.

**sifr_driver**
Orchestrates: parse -> type-check -> HIR -> codegen. Error collection and reporting with source spans. Uses miette or ariadne for pretty diagnostics.

**sifr (CLI)**
Commands: build, run, check, emit. Uses clap for argument parsing. Invokes cargo build on generated project.

---

### **2.5 Data Model**

Not applicable (compiler, no database).

---

### **2.6 API Integration**

Not applicable for M1.

---

### **2.7 Error Handling & Monitoring**

* Parse errors: reported with source location and code snippet
* Type errors: reported with expected vs actual type and source location
* Ownership errors: reported with move location and use-after-move location
* Codegen errors: reported if Rust compilation fails (with rustc output)

---

### **2.8 Deployment Plan**

* Distributed as a Rust binary via `cargo install`
* No external dependencies beyond Rust toolchain

---

### **2.9 Trade-offs & Alternatives**

| Option Considered | Pros | Cons | Rationale for Final Choice |
| --- | --- | --- | --- |
| Build parser from scratch | Full control | Massive effort, error-prone | Fork ruff -- battle-tested, MIT licensed |
| Emit LLVM IR | More control over codegen | Much more complex | Emit Rust -- leverages rustc optimizations |
| Use ruff as dependency (not fork) | Simpler setup | Can't modify AST/parser for Sifr needs | Fork -- need to strip and extend AST |

---

### **2.10 Testing Strategy**

| AC-ID | Test Layer | Happy-Path Check | Non-Happy / Edge Check | Tooling | Pass/Fail Gate |
| --- | --- | --- | --- | --- | --- |
| AC-1 | E2E (Layer 3) | factorial.sifr compiles and outputs 120 | Invalid syntax produces parse error | cargo test --test e2e | All pass |
| AC-2 | Snapshot (Layer 2) | Type inference resolves x=42 as int | Type mismatch int="hello" produces error | insta snapshots + mdtest | All snapshots match |
| AC-3 | Snapshot (Layer 2) | Copy types (int) allow reuse after assign | str move produces use-after-move error | mdtest assertions | All assertions pass |
| AC-4 | All layers | cargo test passes | No panics on corpus tests | cargo test | Exit code 0 |
