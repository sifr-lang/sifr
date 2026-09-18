//! Immutable outer compilation identity and its process-local source-stdlib owner.
use crate::diagnostics::RenderedDiagnostic;
use sifr_identity::CompilerIdentity;
use std::sync::{Arc, Mutex};
type StdlibCache = Mutex<Option<Arc<crate::metadata_reader::Provider>>>;

#[derive(Clone)]
pub struct CompilerContext {
    identity: CompilerIdentity,
    application_profile: crate::ApplicationProfile,
    metadata_override: Option<std::path::PathBuf>,
    sysroot: Result<sifr_sysroot::ResolvedSysroot, sifr_sysroot::SysrootError>,
    pub(crate) stdlib_cache: Arc<StdlibCache>,
}
impl CompilerContext {
    pub fn new(identity: CompilerIdentity) -> Self {
        Self::from_resolved(identity, sifr_sysroot::resolve_sysroot(None))
    }
    pub fn with_sysroot(
        identity: CompilerIdentity,
        sysroot: sifr_sysroot::ResolvedSysroot,
    ) -> Self {
        Self::from_resolved(identity, Ok(sysroot))
    }
    #[must_use]
    pub fn with_metadata_override(mut self, path: std::path::PathBuf) -> Self {
        self.metadata_override = Some(path);
        self.stdlib_cache = Arc::new(Mutex::new(None));
        self
    }
    fn from_resolved(
        identity: CompilerIdentity,
        sysroot: Result<sifr_sysroot::ResolvedSysroot, sifr_sysroot::SysrootError>,
    ) -> Self {
        let cache = Arc::new(Mutex::new(None));
        Self {
            identity,
            application_profile: crate::ApplicationProfile::Development,
            metadata_override: None,
            sysroot,
            stdlib_cache: cache,
        }
    }
    /// Start a new editor generation. Existing hosts and snapshots keep their
    /// original immutable metadata owner alive until their last reference drops.
    pub fn shares_metadata_generation(&self, other: &Self) -> bool {
        self.identity == other.identity && Arc::ptr_eq(&self.stdlib_cache, &other.stdlib_cache)
    }

    pub fn refreshed_toolchain(&self) -> Self {
        Self::new(self.identity.clone()).with_application_profile(self.application_profile)
    }

    /// Drop this idle editor's cache ownership without changing its pinned
    /// toolchain selection or invalidating another active generation.
    pub fn without_cached_metadata(&self) -> Self {
        let mut next = self.clone();
        next.stdlib_cache = Arc::new(Mutex::new(None));
        next
    }

    pub fn for_test_tokens(mut tokens: Vec<(&str, &str)>, configuration: &str) -> Self {
        tokens.extend(crate::compiled_input_tokens());
        Self::new(CompilerIdentity::for_test(tokens, configuration))
            .with_application_profile(crate::ApplicationProfile::Release)
    }
    pub fn for_test() -> Self {
        Self::new(CompilerIdentity::for_test(
            crate::compiled_input_tokens(),
            "driver-producer",
        ))
        .with_application_profile(crate::ApplicationProfile::Release)
    }
    pub fn ensure_development_metadata(&self) -> Result<(), Vec<RenderedDiagnostic>> {
        let sysroot = self.sysroot()?;
        if sysroot.mode() == sifr_sysroot::SysrootMode::InstalledToolchain {
            return Ok(());
        }
        let target = if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                "aarch64-apple-darwin"
            } else {
                "x86_64-apple-darwin"
            }
        } else if cfg!(target_arch = "aarch64") {
            "aarch64-unknown-linux-gnu"
        } else {
            "x86_64-unknown-linux-gnu"
        };
        match &self.metadata_override {
            Some(path) => crate::metadata_producer::validate_development_metadata(
                self.identity(),
                &sysroot.root,
                target,
                path,
            ),
            None => crate::metadata_producer::ensure_development_metadata(
                self.identity(),
                &sysroot.root,
                target,
                &crate::cache_storage::root(),
                &std::sync::atomic::AtomicBool::new(false),
            ),
        }
        .map(|_| ())
        .map_err(|error| {
            vec![crate::diagnostics::diagnostic_with_code(
                error.to_string(),
                sifr_diagnostics::DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
            )]
        })
    }
    pub(crate) fn metadata_provider(
        &self,
    ) -> Result<Arc<crate::metadata_reader::Provider>, Vec<RenderedDiagnostic>> {
        let mut cache = self.stdlib_cache.lock().map_err(|_| {
            vec![crate::diagnostics::diagnostic_with_code(
                "metadata provider owner poisoned",
                sifr_diagnostics::DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
            )]
        })?;
        if let Some(provider) = &*cache {
            return Ok(provider.clone());
        }
        let provider = crate::metadata_reader::select(
            self.identity(),
            self.sysroot()?,
            self.metadata_override.as_deref(),
        )
        .map_err(|error| {
            vec![crate::diagnostics::diagnostic_with_code(
                error.to_string(),
                sifr_diagnostics::DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
            )]
        })?;
        *cache = Some(provider.clone());
        Ok(provider)
    }
    pub fn stdlib_navigation(
        &self,
    ) -> Result<Arc<crate::StdlibNavigation>, Vec<RenderedDiagnostic>> {
        self.metadata_provider()?
            .navigation(&self.sysroot()?.paths.stdlib_root)
            .map_err(|e| {
                vec![crate::diagnostics::diagnostic_with_code(
                    e.to_string(),
                    sifr_diagnostics::DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
                )]
            })
    }
    pub fn metadata_stats(&self) -> Option<serde_json::Value> {
        let cache = self.stdlib_cache.lock().ok()?;
        let provider = cache.as_ref()?;
        let store = &provider.metadata.store;
        use sifr_sysroot::metadata as wire;
        Some(serde_json::json!({
            "metadata_id":provider.metadata.metadata_id,
            "semantic_modules":provider.loaded_semantic_modules(),
            "decoded_semantic_records":store.decoded_count::<wire::SemanticExports>(),
            "decoded_hir_modules":store.decoded_count::<wire::HirModule>(),
            "decoded_types":store.decoded_count::<wire::Type>(),
            "decoded_nominal_views":store.decoded_count::<wire::NominalView>(),
            "projected_nominal_views":provider.projected_nominal_views().ok(),
            "decoded_hir_functions":store.decoded_count::<wire::HirFunction>(),
            "decoded_hir_classes":store.decoded_count::<wire::HirClass>(),
            "decoded_rust_payloads":store.decoded_count::<wire::RustPayload>(),
            "decoded_templates":store.decoded_count::<wire::TemplatePayload>(),
            "rust_payload_reads":store.payload_read_count::<wire::RustPayload>(),
            "hir_module_reads":store.payload_read_count::<wire::HirModule>(),
            "retained_decode_bound_bytes":store.retained_bound().ok()
        }))
    }
    #[must_use]
    pub fn with_application_profile(mut self, profile: crate::ApplicationProfile) -> Self {
        self.application_profile = profile;
        self
    }
    pub fn application_profile(&self) -> crate::ApplicationProfile {
        self.application_profile
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
        let same = a.clone();
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
