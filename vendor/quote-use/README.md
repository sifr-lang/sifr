# Use statements in `quote!`
[![Crates.io Version](https://img.shields.io/crates/v/quote-use.svg)](https://crates.io/crates/quote-use)
[![CI](https://github.com/ModProg/quote-use/actions/workflows/test.yml/badge.svg)](https://github.com/ModProg/quote-use/actions/workflows/test.yml)
[![Docs.rs Documentation](https://img.shields.io/docsrs/quote-use)](https://docs.rs/crate/quote-use)

## Description

Macro to simplify using Types in the [`quote!`](https://docs.rs/quote/latest/quote/macro.quote.html) macro.

## Usage

The [`quote_use!`](https://docs.rs/quote-use/latest/quote_use/macro.quote_use.html) macro can be used just like [`quote!`](https://docs.rs/quote/latest/quote/macro.quote.html), but with the added functionality of
adding use statements at the top:

```rust
quote_use!{
    use std::fs::read;
    
    read("src/main.rs")
}
```

This will expand to the equivalent statement using [`quote!`](https://docs.rs/quote/latest/quote/macro.quote.html):

```rust
quote!{
    ::std::fs::read::read("src/main.rs")
}
```

### Prelude

This also allows to use contents of the rust prelude directly:

```rust
quote_use!{
    Some("src/main.rs")
}
```

#### Overriding prelude
When you want to use your own type instead of the prelude type this can be achieved by simply
importing it like so

```rust
quote_use!{
    use anyhow::Result;
                                                                                                             
    Result
}
```

#### Different preludes

By default [`quote_use!`](https://docs.rs/quote-use/latest/quote_use/macro.quote_use.html) uses the [std prelude](std::prelude) for [2021 edition](std::prelude::rust_2021), 
but this can be configured via features, and also completely disabled.

- **`prelude_std`**: Enables [`std::prelude::v1`](https://doc.rust-lang.org/nightly/std/prelude/v1/index.html)  (incompatible with `prelude_core`)
- `prelude_core`: Enables [`core::prelude::v1`](https://doc.rust-lang.org/nightly/core/prelude/v1/index.html) (incompatible with `prelude_std`)
- **`prelude_2021`**: Enables [`core::prelude::rust_2021`](https://doc.rust-lang.org/nightly/core/prelude/rust_2021/index.html) (requires either `prelude_std` or `prelude_core`)

### Other quote macros

There are also variants for other quote macros from [syn](https://docs.rs/syn/latest/syn/) and [quote](https://docs.rs/quote/latest/quote/):

- [`quote_use!`](https://docs.rs/quote-use/latest/quote_use/macro.quote_use.html) and [`quote_spanned_use!`](https://docs.rs/quote-use/latest/quote_use/macro.quote_spanned_use.html) as replacement for [`quote!`](https://docs.rs/quote/latest/quote/macro.quote.html) and
[`quote_spanned!`](https://docs.rs/quote/latest/quote/macro.quote_spanned.html) respectively
- [`parse_quote_use!`](https://docs.rs/quote-use/latest/quote_use/macro.parse_quote_use.html) and [`parse_quote_spanned_use!`](https://docs.rs/quote-use/latest/quote_use/macro.parse_quote_spanned_use.html) for [`parse_quote!`](https://docs.rs/syn/latest/syn/macro.parse_quote.html)
and [`parse_quote_spanned!`](https://docs.rs/syn/latest/syn/macro.parse_quote_spanned.html)

## Auto namespacing idents

Until [`Span::def_site`](https://doc.rust-lang.org/stable/proc_macro/struct.Span.html#method.def_site) is stabilized, identifiers in e.g. let
bindings in proc-macro expansions can collide with e.g. constants.

To circumvent this you can enable the feature `namespace_idents` which will replace all
identifiers with autonamespaced ones using the pattern `"__{crate_name}_{ident}"`.
