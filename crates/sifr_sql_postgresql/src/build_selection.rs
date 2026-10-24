//! Native parser selection is part of Cargo's feature/artifact identity.

use std::io;

pub(super) fn select_major(features: &[u16], legacy_override: bool) -> io::Result<u16> {
    if legacy_override {
        return Err(io::Error::other(
            "SIFR_POSTGRESQL_MAJOR is no longer a build selector; use --features postgresql-N",
        ));
    }
    match features {
        [] => Ok(18),
        [major @ 13..=18] => Ok(*major),
        [_] => Err(io::Error::other("unsupported PostgreSQL version feature")),
        _ => Err(io::Error::other(
            "select only one PostgreSQL version feature",
        )),
    }
}
