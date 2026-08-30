use crate::cache_fingerprint::stable_cache_fingerprint;
use crate::{CacheKeyContext, CacheKeyFingerprint};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmbeddedAnalysisKey {
    pub protocol: sifr_compiler_component::CacheKey,
    pub context: CacheKeyContext,
}

impl EmbeddedAnalysisKey {
    pub fn new(
        request: &sifr_compiler_component::EmbeddedAnalysisRequest,
        context: CacheKeyContext,
    ) -> Result<Self, sifr_compiler_component::ComponentError> {
        Ok(Self {
            protocol: sifr_compiler_component::CacheKey::for_request(request)?,
            context,
        })
    }

    #[must_use]
    pub fn fingerprint(&self) -> CacheKeyFingerprint {
        stable_cache_fingerprint(
            "embedded-analysis",
            [
                ("protocol", self.protocol.0.clone()),
                ("compiler", self.context.compiler.as_str().to_string()),
                ("workspace", self.context.workspace.as_str().to_string()),
                ("package", self.context.package.as_str().to_string()),
                (
                    "query_policy",
                    self.context.query_policy.as_str().to_string(),
                ),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::EmbeddedAnalysisKey;
    use crate::{
        CacheFamily, CacheKeyContext, CompilerFingerprint, FrontendMode, PackageContextFingerprint,
        QueryPolicyFingerprint, SourcePath, WorkspaceContextFingerprint,
        WorkspacePackageConfigIdentity,
    };

    fn context() -> CacheKeyContext {
        CacheKeyContext::new(
            CacheFamily::EmbeddedAnalysis,
            CompilerFingerprint::current(),
            WorkspaceContextFingerprint::single_file(
                &SourcePath::new("fixture.sifr"),
                FrontendMode::SingleFile,
            ),
            PackageContextFingerprint::from_identity(&WorkspacePackageConfigIdentity {
                workspace_root: Some(SourcePath::new("package")),
                entrypoint: Some(SourcePath::new("package/fixture.sifr")),
            }),
        )
    }

    #[test]
    fn embedded_analysis_fingerprint_includes_protocol_and_frontend_policy() {
        let base = EmbeddedAnalysisKey {
            protocol: sifr_compiler_component::CacheKey("a".repeat(64)),
            context: context(),
        };
        let changed_protocol = EmbeddedAnalysisKey {
            protocol: sifr_compiler_component::CacheKey("b".repeat(64)),
            context: base.context.clone(),
        };
        let changed_policy = EmbeddedAnalysisKey {
            protocol: base.protocol.clone(),
            context: context().with_query_policy(QueryPolicyFingerprint::new("changed")),
        };

        assert_ne!(base.fingerprint(), changed_protocol.fingerprint());
        assert_ne!(base.fingerprint(), changed_policy.fingerprint());
    }
}
