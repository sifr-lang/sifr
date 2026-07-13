//! Typed async Python declaration lowering.

mod conversions;

pub(crate) use conversions::{async_python_function_body, async_python_method_body};
