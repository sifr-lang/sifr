//! Runtime implementation of [`format!`](std::format).
//!
//! # `std::fmt` compatible formatting
//!
//! All options but the fill character for alignment is supported
//! (due to [rust-lang/rfcs#3394](https://github.com/rust-lang/rfcs/pull/3394)).
//!
//! Though the non [`Display`] traits need to be enabled through
//! [features](#features).
//!
//! ```
//! # use std::collections::HashMap;
//! use interpolator::{format, Formattable};
//!
//! let formatted = format(
//!     "{value:+05}", // could be dynamic
//!     &[("value", Formattable::display(&12))]
//!         .into_iter()
//!         .collect::<HashMap<_, _>>(),
//! )?;
//!
//! assert_eq!(formatted, format!("{:+05}", 12));
//! # return Ok::<(), interpolator::Error>(())
//! ```

#![cfg_attr(
    feature = "iter",
    doc = r#"
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

It is also possible to index a single value by only specifying an [`isize`]
`{list:i1}`.

A [`Formattable`] implementing iter is created using [`Formattable::iter`]:

```
// HashMap macro
use collection_literals::hash;
use interpolator::{format, Formattable};
// Needs to be a slice of references because `Formattable::display` expects a
// reference
let items = [&"hello", &"hi", &"hey"].map(Formattable::display);
let items = Formattable::iter(&items);
let format_str = "Greetings: {items:i..-1(`{}{outside}`)(, )} and `{items:i-1}{outside}`";
assert_eq!(
    format(format_str, &hash!("items" => items, "outside" => Formattable::display(&'!')))?,
    "Greetings: `hello!`, `hi!` and `hey!`"
);
# return Ok::<(), interpolator::Error>(())
```"#
)]

//! See [`format()`] and [`write()`] for details.
//!
//! # Macros
//!
//! To simplify creating contexts, some macros are provided.
//!
//! - [`context!`] creates a [`HashMap<&str, Formattable>`](HashMap) to be used
//!   with [`format()`].
#![cfg_attr(
    feature = "iter",
    doc = r"- [`list!`] creates a [`Formattable`] implementing supporting [iter](`i`-iter-format)."
)]
//! - [`iformat!`] and [`iwrite!`] macros matching the behaviour of [`format()`]
//!   and [`write()`] but allowing to specify context directly.
//! - Most of std's formatting macros are supported with an `i` prefix:
//!   - [`iwriteln!`]
//!   - [`iprint!`]
//!   - [`iprintln!`]
//!   - [`ieprint!`]
//!   - [`ieprintln!`]
//!
//! # Features
//! By default only [`Display`] is supported, the rest of the
//! [formatting traits](https://doc.rust-lang.org/std/fmt/index.html#formatting-traits)
//! can be enabled through the following features.
//!
//! - `debug` enables `?`, `x?` and `X?` trait specifiers
//! - `number` enables `x`, `X`, `b`, `o`, `e` and `E` trait specifiers
//! - `pointer` enables `p` trait specifiers
//! - `iter` enables [`i`](#i-iter-format) trait specifier
#![warn(clippy::pedantic, missing_docs)]
#![allow(
    clippy::wildcard_imports,
    clippy::implicit_hasher,
    clippy::enum_glob_use,
    clippy::module_name_repetitions
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
use std::borrow::Borrow;
use std::collections::HashMap;
use std::error::Error as StdError;
#[cfg(feature = "pointer")]
use std::fmt::Pointer;
#[cfg(feature = "number")]
use std::fmt::{Binary, LowerExp, LowerHex, Octal, UpperExp, UpperHex};
use std::fmt::{Debug, Display, Error as FmtError, Write};
use std::hash::Hash;
use std::num::ParseIntError;

#[macro_use]
mod error;
pub use error::*;
mod fmt;
use fmt::format_value;
mod formattable;
pub use formattable::*;
mod parser;
use parser::*;
mod macros;

type Result<T = (), E = Error> = std::result::Result<T, E>;

/// Runtime version of [`format!`].
///
/// Takes a string and a context, containing [`Formattable`] values, returns a
/// string.
///
/// ```
/// # use std::collections::HashMap;
/// use interpolator::{format, Formattable};
///
/// let formatted = format(
///     "{value:+05}", // could be dynamic
///     &[("value", Formattable::display(&12))]
///         .into_iter()
///         .collect::<HashMap<_, _>>(),
/// )
/// .unwrap();
///
/// assert_eq!(formatted, format!("{:+05}", 12));
/// ```
///
/// # Errors
///
/// It will return an error if the specified format string has invalid syntax,
/// the type doesn't implement the expected trait, or the formatting itself
/// failed.
///
/// For more details have a look at [`Error`] and [`ParseError`].
pub fn format(format: &str, context: &impl Context) -> Result<String> {
    let mut out = String::with_capacity(format.len());
    write(&mut out, format, context)?;
    Ok(out)
}

/// Context for `format` and `write`
pub trait Context {
    /// Returns the [`Formattable`] for the requested key
    fn get(&self, key: &str) -> Option<Formattable>;
}

impl<K: Borrow<str> + Eq + Hash> Context for HashMap<K, Formattable<'_>> {
    fn get(&self, key: &str) -> Option<Formattable> {
        HashMap::get(self, key).copied()
    }
}

#[cfg(feature = "iter")]
struct IterContext<'a> {
    outer: &'a dyn Context,
    inner: Formattable<'a>,
}

#[cfg(feature = "iter")]
impl<'a> IterContext<'a> {
    fn new(outer: &'a impl Context, inner: Formattable<'a>) -> Self {
        Self { outer, inner }
    }
}

#[cfg(feature = "iter")]
impl<'a> Context for IterContext<'a> {
    fn get(&self, key: &str) -> Option<Formattable> {
        if key.is_empty() {
            Some(self.inner)
        } else {
            self.outer.get(key)
        }
    }
}

/// Runtime version of [`write!`].
///
/// Takes a mutable [`Write`] e.g. `&mut String`, a format string and a context,
/// containing [`Formattable`] values.
///
/// ```
/// # use std::collections::HashMap;
/// use interpolator::{write, Formattable};
///
/// let mut buf = String::new();
/// write(
///     &mut buf,
///     "{value:+05}", // could be dynamic
///     &[("value", Formattable::display(&12))]
///         .into_iter()
///         .collect::<HashMap<_, _>>(),
/// )
/// .unwrap();
///
/// assert_eq!(buf, format!("{:+05}", 12));
/// ```
///
/// # Errors
///
/// It will return an error if the specified format string has invalid syntax,
/// the type doesn't implement the expected trait, or the formatting itself
/// failed.
///
/// For more details have a look at [`Error`] and [`ParseError`].
pub fn write(out: &mut impl Write, mut format: &str, context: &impl Context) -> Result {
    let format = &mut format;
    let idx = &mut 0;
    while !format.is_empty() {
        if format.starts_with("{{") || format.starts_with("}}") {
            out.write_str(&format[..1])
                .map_err(|e| Error::Fmt(e, *idx))?;
            step(2, format, idx);
            continue;
        }
        if format.starts_with('{') {
            step(1, format, idx);
            let start = *idx;
            let FormatArgument {
                ident,
                alignment,
                sign,
                hash,
                zero,
                width,
                precision,
                trait_,
            } = FormatArgument::from_str(format, idx)?;
            let value = context
                .get(ident)
                .ok_or(Error::MissingValue(ident.to_string(), start))?;
            format_value(
                out,
                value.borrow(),
                width,
                precision,
                alignment,
                sign,
                hash,
                zero,
                trait_,
                *idx,
                context,
            )?;
            ensure!(format.starts_with('}'), ParseError::Expected("}", *idx));
            step(1, format, idx);
            continue;
        }
        let next = format
            .chars()
            .next()
            .expect("should contain a char if not empty");
        out.write_char(next).map_err(|e| Error::Fmt(e, *idx))?;
        step(next.len_utf8(), format, idx);
    }
    Ok(())
}
