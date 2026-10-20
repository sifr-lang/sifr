use crate::{StdlibEmissionCode, StdlibRustSource};
use std::collections::HashMap;

/// Synchronous emission borrows checked metadata. Bootstrap renders each
/// module separately, so its source view excludes previously emitted modules.
/// All signatures, templates and dependency facts still come from the same owner.
pub struct StdlibEmissionView<'a> {
    metadata: &'a StdlibEmissionCode,
    pub(crate) module_rust_code: &'a HashMap<String, StdlibRustSource>,
    pub(crate) syntax_session: Option<&'a crate::StdlibSyntaxSession>,
}

impl<'a> StdlibEmissionView<'a> {
    pub fn new(metadata: &'a StdlibEmissionCode) -> Self {
        Self {
            metadata,
            module_rust_code: &metadata.module_rust_code,
            syntax_session: None,
        }
    }

    pub(crate) fn bootstrap(
        metadata: &'a StdlibEmissionCode,
        module_rust_code: &'a HashMap<String, StdlibRustSource>,
    ) -> Self {
        Self {
            metadata,
            module_rust_code,
            syntax_session: None,
        }
    }
}

impl std::ops::Deref for StdlibEmissionView<'_> {
    type Target = StdlibEmissionCode;

    fn deref(&self) -> &Self::Target {
        self.metadata
    }
}
