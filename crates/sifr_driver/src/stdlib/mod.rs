mod bootstrap;

pub(crate) use bootstrap::{compile_stdlib, StdlibCompiled};

#[cfg(test)]
pub(crate) use bootstrap::{compile_stdlib_uncached, get_or_init_stdlib_cache};
