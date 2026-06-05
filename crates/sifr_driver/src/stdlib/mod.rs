mod bootstrap;
mod cache;
mod intrinsics;
mod types;

pub(crate) use bootstrap::compile_stdlib;
pub(crate) use types::StdlibCompiled;

#[cfg(test)]
pub(crate) use bootstrap::compile_stdlib_uncached;
