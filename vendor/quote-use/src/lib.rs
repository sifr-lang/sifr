//! # Description
//!
//! Macro to simplify using Types in the [`quote!`] macro.
//!
//! # Usage
//!
//! The [`quote_use!`] macro can be used just like [`quote!`], but with the
//! added functionality of adding use statements at the top:
//!
//! ```
//! # use quote_use::quote_use;
//! quote_use! {
//!     ## use std::fs::read;
//!
//!     read("src/main.rs")
//! }
//! # ;
//! ```
//!
//! This will expand to the equivalent statement using [`quote!`]:
//!
//! ```
//! # use quote::quote;
//! quote! {
//!     ::std::fs::read::read("src/main.rs")
//! }
//! # ;
//! ```
//!
//! ## Prelude
//!
//! This also allows using contents of the rust prelude directly:
//!
//! ```
//! # use quote_use::quote_use;
//! quote_use! {
//!     Some("src/main.rs")
//! }
//! # ;
//! ```
//! ### Overriding prelude
//! When you want to use your own type instead of the prelude type this can be
//! achieved by simply importing it like so
//!
//! ```
//! # use quote_use::quote_use;
//! quote_use! {
//!     ## use anyhow::Result;
//!
//!     Result
//! }
//! # ;
//! ```
//! ### Different preludes
//!
//! By default [`quote_use!`] uses the [core prelude](core::prelude), [std
//! prelude](std::prelude) and [2021 edition prelude](std::prelude::rust_2021).
//! Preferring `core` where available.
//!
//! All preludes can be disabled by adding `# use no_prelude;` at the top of the
//! macro input. The `std` prelude can be disabled with `# use no_std_prelude;`.
//!
//! ## Other quote macros
//!
//! There are also variants for other quote macros from [syn] and [mod@quote]:
//!
//! - [`quote_use!`] and [`quote_spanned_use!`] as replacement for [`quote!`]
//!   and
//! [`quote_spanned!`](quote::quote_spanned!) respectively
//! - [`parse_quote_use!`] and [`parse_quote_spanned_use!`] for
//!   [`parse_quote!`](syn::parse_quote!)
//! and [`parse_quote_spanned!`](syn::parse_quote_spanned!)
#[cfg(doc)]
use quote::quote;
// Reexport
pub use quote::{format_ident, IdentFragment, ToTokens, TokenStreamExt};

#[doc(hidden)]
pub mod __private {
    pub use quote;
    pub use quote_use_macros::quote_use_impl;
    #[cfg(feature = "syn")]
    pub use syn;
}

#[macro_export]
macro_rules! quote_use {
    ($($tokens:tt)*) => {
        $crate::__private::quote_use_impl!{($crate::__private::quote::quote) () ($($tokens)*)}
    };
}

#[macro_export]
macro_rules! quote_spanned_use {
    ($span:expr => $($tokens:tt)*) => {
        $crate::__private::quote_use_impl!{($crate::__private::quote::quote_spanned) ($span =>) ($($tokens)*)}
    };
}

#[cfg(feature = "syn")]
#[macro_export]
macro_rules! parse_quote_use {
    ($($tokens:tt)*) => {
        $crate::__private::quote_use_impl!{($crate::__private::syn::parse_quote) () ($($tokens)*)}
    };
}

#[cfg(feature = "syn")]
#[macro_export]
macro_rules! parse_quote_spanned_use {
    ($span:expr => $($tokens:tt)*) => {
        $crate::__private::quote_use_impl!{($crate::__private::syn::parse_quote_spanned) ($span =>) ($($tokens)*)}
    };
}

#[macro_export]
macro_rules! quote_use_no_prelude {
    ($($tokens:tt)*) => {
        $crate::__private::quote_use_impl!{($crate::__private::quote::quote) () (#use no_prelude; $($tokens)*)}
    };
}

#[macro_export]
macro_rules! quote_spanned_use_no_prelude {
    ($span:expr => $($tokens:tt)*) => {
        $crate::__private::quote_use_impl!{($crate::__private::quote::quote_spanned) ($span =>) (#use no_prelude; $($tokens)*)}
    };
}

#[cfg(feature = "syn")]
#[macro_export]
macro_rules! parse_quote_use_no_prelude {
    ($($tokens:tt)*) => {
        $crate::__private::quote_use_impl!{($crate::__private::syn::parse_quote) () (#use no_prelude; $($tokens)*)}
    };
}

#[cfg(feature = "syn")]
#[macro_export]
macro_rules! parse_quote_spanned_use_no_prelude {
    ($span:expr => $($tokens:tt)*) => {
        $crate::__private::quote_use_impl!{($crate::__private::syn::parse_quote_spanned) ($span =>) (#use no_prelude; $($tokens)*)}
    };
}
