# sifr

<img src="logo.webp" alt="Sifr Logo" height="200">

**Python syntax. Rust performance. If it compiles, it works.**

Sifr is a compiled language that looks like Python but compiles to Rust, producing native binaries. It enforces static typing, safe error handling, and ownership **all at compile time** so your programs never crash at runtime.

## Why sifr?

- **Write Python, run Rust.** Familiar syntax, native speed.
- **No runtime crashes.** Indexing returns `Option`, errors return `Result` - the compiler forces you to handle them.
- **No exceptions.** `try`/`except` is pattern matching on `Result`, not stack unwinding.
- **Ownership without the pain.** Function args borrow by default. No lifetime annotations. The compiler figures it out.
- **TypeScript-style types.** Union types, literal types, type narrowing - all first-class.

## Examples

**Union types and safe indexing** - no crashes, ever.

```python
def main():
    users: dict[str, int] = {"alice": 30, "bob": 25}

    age: int | None = users["charlie"]  # missing key returns None, not a crash
    if age is not None:
        print(f"age: {age}")
    else:
        print("user not found")

    # union types narrow automatically
    def show(val: int | str) -> str:
        if isinstance(val, int):
            return f"number: {val}"
        else:
            return f"text: {val}"

    print(show(42))       # number: 42
    print(show("hello"))  # text: hello
```

**Error handling** - `Result` instead of exceptions, compiler-enforced.

```python
class ParseError(Error):
    message: str

def parse_age(input: str) -> Result[int, ParseError]:
    if input == "":
        raise ParseError("empty input")  # maps to Err(...)
    return int(input)                     # auto-wrapped in Ok(...)

def main():
    try:
        age: int = parse_age("25")  # auto-unwrapped by the compiler
        print(f"age: {age}")
    except ParseError as e:
        print(e.message)
    # compiler error if you forget to handle ParseError ^
```

**Borrow by default** - use values after passing them to functions.

```python
def longest(items: list[str]) -> str:  # items is borrowed (&Vec), not moved
    best: str = ""
    for s in items:
        if len(s) > len(best):
            best = s.clone()
    return best

def main():
    names: list[str] = ["alice", "bob", "charlie"]
    print(longest(names))  # charlie
    print(names)           # ["alice", "bob", "charlie"] - still yours
```

## Installation

Install the current beta preview:

```bash
curl -fsSL https://sifr.sh/install | sh
```

The installer downloads a prebuilt preview binary for your platform, verifies
its SHA-256 checksum, installs it to `~/.sifr/bin/sifr`, and updates your shell
profile so new shells can find `sifr`.

Install the alpha preview instead:

```bash
curl -fsSL https://sifr.sh/install/alpha | sh
```

Pin an exact preview version:

```bash
curl -fsSL https://sifr.sh/install | sh -s -- --version 0.1.0-beta.1
```

Install to a custom directory:

```bash
curl -fsSL https://sifr.sh/install | SIFR_INSTALL_DIR="$HOME/.sifr/bin" sh
```

Disable shell profile changes:

```bash
curl -fsSL https://sifr.sh/install | SIFR_NO_MODIFY_PATH=1 sh
# or
curl -fsSL https://sifr.sh/install | sh -s -- --no-modify-path
```

Update an official standalone preview install:

```bash
sifr self update
sifr self update --dry-run
```

`sifr self update` works only for schema-versioned standalone installs created
by the official installer. Package-manager installs should use the package
manager's update command. See [`docs/self_update.md`](docs/self_update.md) for
preview channel limits, `--force` rules, and troubleshooting.

Supported preview targets:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`

Windows installer support is not available yet.

## Getting started

### Build the compiler from source

Source builds require the [Rust toolchain](https://rustup.rs/) with `rustc` and
`cargo`.

```bash
git clone https://github.com/sifr-lang/sifr.git
cd sifr
cargo build --release
```

The root `Cargo.lock` is tracked intentionally. Source builds, local
validation, and CI use the committed dependency graph; lockfile diffs are
reviewable dependency changes.

### Restore optional sub-repositories

External corpora, editor integration repositories, and package-management demo
repositories live in Git submodules and are restored to their expected owner
paths when needed. `scripts/clone_subrepos.sh` initializes all configured
submodules from `.gitmodules`, including:

- `third_party/ruff`
- `editor_integrations`
- `verification/areas/algorithmic_compatibility/corpora/leetcode`
- `verification/areas/developer_tooling/corpora/sifr-large-lsp-verification`
- `verification/areas/package_management/corpora/demo_repositories/*`

```bash
scripts/clone_subrepos.sh
```

The script is safe to run repeatedly. It initializes missing submodules and
fast-forwards existing clean checkouts when `--remote` is selected.

LeetCode fixtures live under
`verification/areas/algorithmic_compatibility/corpora/leetcode/src`. The large
LSP corpus and package demo repositories remain area-owned verification inputs,
not top-level demos or process history.

### Restore maintenance submodules

Sifr keeps a fork of Ruff at `third_party/ruff` for parser and AST maintenance.
The submodule tracks the `sifr/0.15.12-maintenance` branch. To restore only the
Ruff fork without touching verification corpora or editor integrations:

```bash
git submodule update --init --recursive third_party/ruff
```

### Compile and run a `.sifr` file

```bash
# Compile and run in one step
cargo run -- run hello.sifr

# Or build a native binary
cargo run -- build hello.sifr

# Type-check without compiling
cargo run -- check hello.sifr

# View the generated Rust code
cargo run -- emit hello.sifr
```

### Run the test suite

```bash
cargo test
```

## Architecture

For a deep dive into the compiler pipeline, type system, ownership model, and design decisions, see the [Architecture Document](internal_docs/architecture.md).

For stable command-mode behavior and edge-case guarantees, see the [CLI Command Semantics Rules](docs/cli_command_semantics.md).

For formatter command, configuration, and editor behavior, see the [Formatter Guide](docs/formatter.md).

### Interactive Compiler Pipeline

Want to see how a Sifr program travels from source text to a native binary?
The interactive pipeline visualizer walks through every compiler step with a live `factorial` example at each stage.

**[▶ Open the Compiler Pipeline Visualizer](https://htmlpreview.github.io/?https://github.com/sifr-lang/sifr/blob/main/internal_docs/compiler_pipeline.html)**

## License

[MIT](LICENSE.md)

## Sponsors

* [CDON](https://www.cdon.se/)/[Fyndiq](https://www.fyndiq.se/): Leading marketplaces in the Nordics (Sweden, Norway, Denmark, Finland).
