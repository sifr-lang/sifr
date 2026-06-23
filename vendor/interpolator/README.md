# interpolator

[![CI Status](https://github.com/ModProg/interpolator/actions/workflows/test.yaml/badge.svg)](https://github.com/ModProg/interpolator/actions/workflows/test.yaml)
[![Documentation for `main`](https://img.shields.io/badge/docs-main-informational)](https://modprog.github.io/interpolator/interpolator/)
[![Crates.io](https://img.shields.io/crates/v/interpolator)](https://crates.io/crates/interpolator)
[![Docs.rs](https://img.shields.io/crates/v/interpolator?color=informational&label=docs.rs)](https://docs.rs/interpolator)

Runtime implementation of `format!`.

# `format`
Runtime version of `format!`.

Takes a string and a context, containing `Formattable` values, returns a
string.

```rs
use std::collections::HashMap;
use template::{format, Formattable};

let formatted = format(
    "{value:+05}", // could be dynamic
    &[("value", Formattable::display(&12))].into_iter().collect::<HashMap<_,_>>(),
)
.unwrap();

assert_eq!(formatted, format!("{:+05}", 12));
```

# `write`
Runtime version of `write!`.

Takes a mutable `Write` e.g. `&mut String`, a format string and a context,
containing `Formattable` values.
                                                                            
```rs
use std::collections::HashMap;
use template::{write, Formattable};
                                                                            
let mut buf = String::new();
write(
    &mut buf,
    "{value:+05}", // could be dynamic
    &[("value", Formattable::display(&12))].into_iter().collect::<HashMap<_,_>>(),
)
.unwrap();
                                                                            
assert_eq!(buf, format!("{:+05}", 12));
```

# `i` iter format

The feature `iter` enables an additional format trait `i`, it allows to
format a list of values with a format string and an optional join
expression.

The syntax is `{list:i(the format string, '{}' is the array element)(the
join)}`, an empty join can also be omitted `{list:i({})}`. If join is omitted
the format string `{}` can be omitted as well `{list:i}`.

Should you need to use `)` inside your format string or join, you can add `#`
similar to rust's [raw string](https://doc.rust-lang.org/reference/tokens.html#raw-string-literals)
(i.e. `#(({}))#`).

It is also possible to only iterate a sub-slice specified through a range
before the format string, i.e. `{list:i1..4}`. For open ranges range
bounds can also be omitted. To index from the end, you can use negative
range bounds.

It is also possible to index a single value by only specifying an `isize`
`{list:i1}`.


A `Formattable` implementing iter is created using `Formattable::iter`:

```rs
// HashMap macro
use collection_literals::hash;
use interpolator::{format, Formattable};
// Needs to be a slice of references because `Formattable::display` expects a
// reference
let items = [&"hello", &"hi", &"hey"].map(Formattable::display);
let items = Formattable::iter(&items);
let format_str = "Greetings: {items:i..-1(`{it}`)(, )} and {items:i-1..(`{it}`)}";
assert_eq!(
    format(format_str, &hash!("items" => items))?,
    "Greetings: `hello`, `hi` and `hey`"
);
# return Ok::<(), interpolator::Error>(())
```

# Features
By default only `Display` is supported, the rest of the
[formatting traits](https://doc.rust-lang.org/std/fmt/index.html#formatting-traits)
can be enabled through the following features.

- `debug` enables `?`, `x?` and `X?` trait specifiers
- `number` enables `x`, `X`, `b`, `o`, `e` and `E` trait specifiers
- `pointer` enables `p` trait specifiers
- `iter` enables `i` trait specifier
