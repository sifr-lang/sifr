//! Immutable outer compilation identity and its process-local source-stdlib owner.
use crate::{diagnostics::RenderedDiagnostic, stdlib::StdlibCompiled};
use sifr_identity::CompilerIdentity;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, OnceLock},
};

type StdlibCache = OnceLock<Result<Arc<StdlibCompiled>, Vec<RenderedDiagnostic>>>;
type ContextCaches = BTreeMap<(CompilerIdentity, String), Arc<StdlibCache>>;
static CACHES: OnceLock<Mutex<ContextCaches>> = OnceLock::new();

#[derive(Clone)]
pub struct CompilerContext {
    identity: CompilerIdentity,
    sysroot: Result<sifr_sysroot::ResolvedSysroot, sifr_sysroot::SysrootError>,
    pub(crate) stdlib_cache: Arc<StdlibCache>,
}
impl CompilerContext {
    pub fn new(identity: CompilerIdentity) -> Self {
        let sysroot = sifr_sysroot::resolve_sysroot(None);
        let key = sysroot.as_ref().map_or_else(
            |_| "<unresolved>".to_owned(),
            |root| format!("{}:{}", root.root.display(), root.toolchain_id()),
        );
        let cache = CACHES
            .get_or_init(Mutex::default)
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry((identity.clone(), key))
            .or_default()
            .clone();
        Self {
            identity,
            sysroot,
            stdlib_cache: cache,
        }
    }
    pub fn for_test_tokens(mut tokens: Vec<(&str, &str)>, configuration: &str) -> Self {
        tokens.extend(crate::compiled_input_tokens());
        Self::new(CompilerIdentity::for_test(tokens, configuration))
    }
    pub fn for_test() -> Self {
        Self::new(CompilerIdentity::for_test(
            crate::compiled_input_tokens(),
            "driver-producer",
        ))
    }
    pub fn identity(&self) -> &CompilerIdentity {
        &self.identity
    }
    pub fn resolved_sysroot(
        &self,
    ) -> &Result<sifr_sysroot::ResolvedSysroot, sifr_sysroot::SysrootError> {
        &self.sysroot
    }
    pub fn sysroot(&self) -> Result<&sifr_sysroot::ResolvedSysroot, Vec<RenderedDiagnostic>> {
        self.sysroot.as_ref().map_err(|error| {
            vec![crate::diagnostics::diagnostic_with_code(
                error.boundary_message(),
                sifr_diagnostics::DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
            )]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dx_identity_contexts_do_not_share_incompatible_stdlib_owners() {
        let a = CompilerContext::new(CompilerIdentity::product(&"a".repeat(64)).unwrap());
        let b = CompilerContext::new(CompilerIdentity::product(&"b".repeat(64)).unwrap());
        assert!(!Arc::ptr_eq(&a.stdlib_cache, &b.stdlib_cache));
        let same = CompilerContext::new(a.identity().clone());
        assert!(Arc::ptr_eq(&a.stdlib_cache, &same.stdlib_cache));
        assert!(a.identity().validate_override(b.identity()).is_err());
    }
    #[test]
    #[allow(clippy::print_stdout)]
    fn dx_identity_bare_test_uses_compiled_dependency_configuration() {
        let context = CompilerContext::for_test();
        assert!(context.identity().is_test());
        assert_eq!(context.identity(), CompilerContext::for_test().identity());
        assert!(
            crate::compiled_input_tokens()
                .iter()
                .any(|(name, _)| *name == "sifr_frontend")
        );
        println!("DX_TEST_COMPILER_ID={}", context.identity().as_str());
    }
}
