use crate::cache_fingerprint::stable_cache_fingerprint;
use crate::{CacheKeyContext, CacheKeyFingerprint};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmbeddedAnalysisKey {
    pub protocol: sifr_compiler_component::CacheKey,
    pub context: CacheKeyContext,
}

impl EmbeddedAnalysisKey {
    /// Build the frontend cache identity from the complete component request.
    /// The protocol key includes template parts, hole types and fragment
    /// identities, the schema profile/fingerprint and artifacts, provider
    /// compatibility settings, component identity/protocol, imported
    /// signatures, plan kind, and compiler semantic version. The outer context
    /// adds compiler, workspace, package, target-mode, source, and policy
    /// identities.
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
    use semver::Version;
    use sifr_compiler_component::{
        AnalysisContext, COMPONENT_PROTOCOL_MAJOR, ClosedType, ComponentIdentity,
        DiagnosticRegistry, EmbeddedAnalysisRequest, HoleDescriptor, PlanKind, SourceSpan,
        TemplatePart,
    };
    use std::collections::BTreeMap;

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

    #[test]
    fn embedded_analysis_request_key_covers_every_sql_semantic_input() {
        let base = request();
        let base_key = EmbeddedAnalysisKey::new(&base, context())
            .expect("key")
            .fingerprint();
        let mut variants = Vec::new();

        let mut changed = base.clone();
        changed.parts[0] = TemplatePart::Static {
            text: "SELECT changed".to_string(),
            span: span(0, 14),
        };
        variants.push(changed);
        let mut changed = base.clone();
        changed.holes[0].ty = ClosedType::Str;
        variants.push(changed);
        let mut changed = base.clone();
        changed.holes[0].fragment_identity = Some("fragment.other".to_string());
        variants.push(changed);
        let mut changed = base.clone();
        changed.context.schema_fingerprint = Some("b".repeat(64));
        variants.push(changed);
        let mut changed = base.clone();
        changed
            .context
            .semantic_profile
            .insert("compatibility".to_string(), "strict".to_string());
        variants.push(changed);
        let mut changed = base.clone();
        changed.protocol_major = COMPONENT_PROTOCOL_MAJOR + 1;
        variants.push(changed);
        let mut changed = base.clone();
        changed.component.version = Version::new(2, 0, 0);
        variants.push(changed);
        let mut changed = base.clone();
        changed.compiler_semantic_version = "semantic-v2".to_string();
        variants.push(changed);

        assert!(variants.into_iter().all(|variant| {
            EmbeddedAnalysisKey::new(&variant, context())
                .expect("variant key")
                .fingerprint()
                != base_key
        }));
    }

    fn request() -> EmbeddedAnalysisRequest {
        EmbeddedAnalysisRequest {
            protocol_major: COMPONENT_PROTOCOL_MAJOR,
            component: ComponentIdentity {
                package: "postgresql".to_string(),
                processor: "sifr.sql.postgresql.sql".to_string(),
                version: Version::new(1, 0, 0),
                sha256: "a".repeat(64),
            },
            provider_diagnostics: DiagnosticRegistry::compiler(),
            compiler_semantic_version: "semantic-v1".to_string(),
            parts: vec![TemplatePart::Static {
                text: "SELECT ".to_string(),
                span: span(0, 7),
            }],
            holes: vec![HoleDescriptor {
                index: 0,
                ty: ClosedType::Int,
                fragment_identity: Some("fragment.users".to_string()),
            }],
            context: AnalysisContext {
                schema_profile: Some("main".to_string()),
                schema_fingerprint: Some("a".repeat(64)),
                semantic_profile: BTreeMap::from([(
                    "compatibility".to_string(),
                    "postgresql-18".to_string(),
                )]),
                imported_signatures: vec!["fragment.users".to_string()],
                artifacts: Vec::new(),
            },
            plan_kind: PlanKind::Document,
        }
    }

    fn span(start: u32, end: u32) -> SourceSpan {
        SourceSpan {
            document: "query.sifr".to_string(),
            start,
            end,
        }
    }
}
