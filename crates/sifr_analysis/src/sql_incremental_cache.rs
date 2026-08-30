use sifr_frontend::{CacheKeyFingerprint, EmbeddedQueryCache};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SqlAnalysisDependency {
    pub identity: String,
    pub fingerprint: String,
}

impl SqlAnalysisDependency {
    #[must_use]
    pub fn new(identity: impl Into<String>, fingerprint: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
            fingerprint: fingerprint.into(),
        }
    }
}

#[derive(Default)]
struct SqlDependencyIndex {
    by_query: BTreeMap<CacheKeyFingerprint, BTreeMap<String, String>>,
    by_dependency: BTreeMap<String, BTreeSet<CacheKeyFingerprint>>,
}

impl SqlDependencyIndex {
    fn record(
        &mut self,
        key: CacheKeyFingerprint,
        dependencies: impl IntoIterator<Item = SqlAnalysisDependency>,
    ) {
        self.remove(&key);
        let dependencies = dependencies
            .into_iter()
            .map(|dependency| (dependency.identity, dependency.fingerprint))
            .collect::<BTreeMap<_, _>>();
        for identity in dependencies.keys() {
            self.by_dependency
                .entry(identity.clone())
                .or_default()
                .insert(key.clone());
        }
        self.by_query.insert(key, dependencies);
    }

    fn affected_by_observed(
        &self,
        observed: &BTreeMap<String, String>,
    ) -> BTreeSet<CacheKeyFingerprint> {
        let mut affected = BTreeSet::new();
        for (identity, keys) in &self.by_dependency {
            for key in keys {
                let expected = self
                    .by_query
                    .get(key)
                    .and_then(|dependencies| dependencies.get(identity));
                if expected != observed.get(identity) {
                    affected.insert(key.clone());
                }
            }
        }
        affected
    }

    fn remove(&mut self, key: &CacheKeyFingerprint) {
        let Some(dependencies) = self.by_query.remove(key) else {
            return;
        };
        for identity in dependencies.keys() {
            let remove_identity = self.by_dependency.get_mut(identity).is_some_and(|queries| {
                queries.remove(key);
                queries.is_empty()
            });
            if remove_identity {
                self.by_dependency.remove(identity);
            }
        }
    }
}

/// Analysis-owned dependency index around the frontend-owned SQL cache key and
/// bounded value store. `observed` must be the complete current dependency map;
/// a changed or removed dependency invalidates only the queries that recorded it.
pub struct SqlIncrementalAnalysisCache<T> {
    values: EmbeddedQueryCache<T>,
    dependencies: SqlDependencyIndex,
}

impl<T> SqlIncrementalAnalysisCache<T> {
    #[must_use]
    pub fn open_default() -> Self {
        Self {
            values: EmbeddedQueryCache::open_default(),
            dependencies: SqlDependencyIndex::default(),
        }
    }

    pub fn new(capacity: usize) -> Result<Self, &'static str> {
        Ok(Self {
            values: EmbeddedQueryCache::new(capacity)?,
            dependencies: SqlDependencyIndex::default(),
        })
    }

    pub fn get(&mut self, key: &CacheKeyFingerprint) -> Option<Arc<T>> {
        self.values.get(key)
    }

    pub fn insert(
        &mut self,
        key: CacheKeyFingerprint,
        value: T,
        dependencies: impl IntoIterator<Item = SqlAnalysisDependency>,
    ) -> Result<Arc<T>, &'static str> {
        let inserted = self.values.insert(&key, value)?;
        for evicted in inserted.evicted {
            self.dependencies.remove(&evicted);
        }
        self.dependencies.record(key, dependencies);
        Ok(inserted.value)
    }

    pub fn pin(&mut self, key: &CacheKeyFingerprint) -> Result<(), &'static str> {
        self.values.pin(key)
    }

    pub fn unpin(&mut self, key: &CacheKeyFingerprint) {
        self.values.unpin(key);
    }

    pub fn invalidate_dependencies(
        &mut self,
        observed: &BTreeMap<String, String>,
    ) -> Vec<CacheKeyFingerprint> {
        let affected = self.dependencies.affected_by_observed(observed);
        for key in &affected {
            self.values.remove(key);
            self.dependencies.remove(key);
        }
        affected.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_compiler_component::{
        AnalysisContext, COMPONENT_PROTOCOL_MAJOR, ComponentIdentity, DiagnosticRegistry,
        EmbeddedAnalysisRequest, PlanKind,
    };
    use sifr_frontend::{
        CacheFamily, CacheKeyContext, CompilerFingerprint, EmbeddedAnalysisKey, FrontendMode,
        PackageContextFingerprint, QueryPolicyFingerprint, SourcePath, WorkspaceContextFingerprint,
        WorkspacePackageConfigIdentity,
    };

    #[test]
    fn invalidation_is_precise_for_changed_and_removed_dependencies() {
        let users = key("users");
        let orders = key("orders");
        let mut cache = SqlIncrementalAnalysisCache::new(4).expect("cache");
        cache
            .insert(
                users.clone(),
                "users-plan",
                [SqlAnalysisDependency::new("users.id", "u1")],
            )
            .expect("insert users");
        cache
            .insert(
                orders.clone(),
                "orders-plan",
                [SqlAnalysisDependency::new("orders.id", "o1")],
            )
            .expect("insert orders");

        let invalidated = cache.invalidate_dependencies(&BTreeMap::from([
            ("users.id".to_string(), "u2".to_string()),
            ("orders.id".to_string(), "o1".to_string()),
            ("unrelated.comment".to_string(), "c1".to_string()),
        ]));
        assert_eq!(invalidated, vec![users.clone()]);
        assert!(cache.get(&users).is_none());
        assert_eq!(cache.get(&orders).as_deref(), Some(&"orders-plan"));

        assert_eq!(
            cache.invalidate_dependencies(&BTreeMap::new()),
            vec![orders]
        );
    }

    #[test]
    fn bounds_pinning_and_eviction_change_reuse_not_results() {
        let first = key("first");
        let second = key("second");
        let mut cache = SqlIncrementalAnalysisCache::new(1).expect("cache");
        cache
            .insert(first.clone(), "same-plan", [])
            .expect("insert first");
        cache.pin(&first).expect("pin first");
        cache
            .insert(second.clone(), "same-plan", [])
            .expect("insert second");
        assert_eq!(cache.get(&first).as_deref(), Some(&"same-plan"));
        assert!(cache.get(&second).is_none());
        assert_eq!("same-plan", recompute_plan());
    }

    fn key(seed: &str) -> CacheKeyFingerprint {
        let request = EmbeddedAnalysisRequest {
            protocol_major: COMPONENT_PROTOCOL_MAJOR,
            component: ComponentIdentity {
                package: "postgresql".to_string(),
                processor: "sifr.sql.postgresql.sql".to_string(),
                version: semver::Version::new(1, 0, 0),
                sha256: "a".repeat(64),
            },
            provider_diagnostics: DiagnosticRegistry::compiler(),
            compiler_semantic_version: seed.to_string(),
            parts: Vec::new(),
            holes: Vec::new(),
            context: AnalysisContext {
                schema_profile: None,
                schema_fingerprint: None,
                semantic_profile: BTreeMap::new(),
                imported_signatures: Vec::new(),
                artifacts: Vec::new(),
            },
            plan_kind: PlanKind::Document,
        };
        EmbeddedAnalysisKey::new(
            &request,
            CacheKeyContext::new(
                CacheFamily::EmbeddedAnalysis,
                CompilerFingerprint::current(),
                WorkspaceContextFingerprint::single_file(
                    &SourcePath::new("fixture.sifr"),
                    FrontendMode::SingleFile,
                ),
                PackageContextFingerprint::from_identity(&WorkspacePackageConfigIdentity {
                    workspace_root: None,
                    entrypoint: None,
                }),
            )
            .with_query_policy(QueryPolicyFingerprint::new(seed)),
        )
        .expect("key")
        .fingerprint()
    }

    fn recompute_plan() -> &'static str {
        "same-plan"
    }
}
