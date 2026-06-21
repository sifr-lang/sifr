mod bootstrap;
mod cache;
mod intrinsics;
mod re_exports;
mod types;

pub(crate) use bootstrap::compile_stdlib;
pub use bootstrap::external_defs;
pub(crate) use types::StdlibCompiled;

#[cfg(test)]
pub(crate) use bootstrap::compile_stdlib_uncached;
