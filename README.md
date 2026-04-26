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

## Getting started

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (rustc + cargo)

### Build the compiler

```bash
git clone https://github.com/sifr-lang/sifr.git
cd sifr
cargo build --release
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

For stable command-mode behavior and edge-case guarantees, see the [CLI Command Semantics Contract](docs/cli_command_semantics.md).

### Interactive Compiler Pipeline

Want to see how a Sifr program travels from source text to a native binary?
The interactive pipeline visualizer walks through every compiler step with a live `factorial` example at each stage.

**[▶ Open the Compiler Pipeline Visualizer](https://htmlpreview.github.io/?https://github.com/sifr-lang/sifr/blob/main/internal_docs/compiler_pipeline.html)**

## License

[MIT](LICENSE.md)

## Sponsors

* [CDON](https://www.cdon.se/)/[Fyndiq](https://www.fyndiq.se/): Leading marketplaces in the Nordics (Sweden, Norway, Denmark, Finland).
