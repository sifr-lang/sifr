use super::{
    FrontendDiagnosticStyle, FrontendMode, ProjectRoot, SourceHash, SourcePath,
    WorkspaceCompilerOptions, WorkspacePackageConfigIdentity, WorkspaceSessionTarget,
};

const CACHE_KEY_SCHEMA_VERSION: &str = "frontend-cache-key-v1";
const SOURCE_HASH_SCHEMA_VERSION: &str = "source-text-fnv1a64-v1";
const SOURCE_MAP_ALGORITHM_VERSION: &str = "source-map-line-index-v1";
const PARSER_OPTIONS_VERSION: &str = "ruff-0.16.4-sifr-parser-v1";
const LOWERING_OPTIONS_VERSION: &str = "sifr-hir-lowering-v1";
const DIAGNOSTIC_POLICY_VERSION: &str = "sifr-diagnostics-v1";
const LINT_POLICY_VERSION: &str = "sifr-lint-v1";
const FORMAT_POLICY_VERSION: &str = "sifr-format-v1";
const FORMAT_OPTIONS_VERSION: &str = "sifr-format-options-default-v1";
const PACKAGE_GRAPH_POLICY_VERSION: &str = "sifr-package-graph-v1";
const SYMBOL_BUCKET_POLICY_VERSION: &str = "sifr-symbol-buckets-v1";
const FLOW_GRAPH_POLICY_VERSION: &str = "sifr-flow-graph-v1";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompilerFingerprint(String);

impl CompilerFingerprint {
    #[must_use]
    pub fn current() -> Self {
        let mut builder = FingerprintBuilder::new("compiler");
        builder.field("cache_key_schema", CACHE_KEY_SCHEMA_VERSION);
        builder.field("frontend_crate_version", env!("CARGO_PKG_VERSION"));
        builder.field("parser_options", PARSER_OPTIONS_VERSION);
        builder.field("lowering_options", LOWERING_OPTIONS_VERSION);
        builder.field("source_map_algorithm", SOURCE_MAP_ALGORITHM_VERSION);
        builder.field("diagnostic_policy", DIAGNOSTIC_POLICY_VERSION);
        builder.field("lint_policy", LINT_POLICY_VERSION);
        builder.field("format_policy", FORMAT_POLICY_VERSION);
        builder.field("format_options", FORMAT_OPTIONS_VERSION);
        builder.field("package_graph_policy", PACKAGE_GRAPH_POLICY_VERSION);
        builder.field("symbol_bucket_policy", SYMBOL_BUCKET_POLICY_VERSION);
        builder.field("flow_graph_policy", FLOW_GRAPH_POLICY_VERSION);
        Self(builder.finish_hex())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CacheKeyFingerprint(String);

impl CacheKeyFingerprint {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub(crate) fn stable_cache_fingerprint(
    domain: &str,
    fields: impl IntoIterator<Item = (&'static str, String)>,
) -> CacheKeyFingerprint {
    let mut builder = FingerprintBuilder::new(domain);
    for (name, value) in fields {
        builder.field(name, value);
    }
    CacheKeyFingerprint(builder.finish_hex())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceContextFingerprint(String);

impl WorkspaceContextFingerprint {
    #[must_use]
    pub fn single_file(path: &SourcePath, mode: FrontendMode) -> Self {
        let mut builder = FingerprintBuilder::new("workspace-context");
        builder.field("kind", "single-file");
        builder.path_field("path", path);
        builder.field("mode", frontend_mode_label(mode));
        Self(builder.finish_hex())
    }

    #[must_use]
    pub fn project(root: &ProjectRoot) -> Self {
        let mut builder = FingerprintBuilder::new("workspace-context");
        builder.field("kind", "project");
        builder.path_field("root", &root.root);
        builder.path_field("entrypoint", &root.entrypoint);
        Self(builder.finish_hex())
    }

    #[must_use]
    pub fn from_target(target: &WorkspaceSessionTarget) -> Self {
        match target {
            WorkspaceSessionTarget::SingleFile(target) => {
                Self::single_file(&target.path, target.mode)
            }
            WorkspaceSessionTarget::Project(root) => Self::project(root),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageContextFingerprint(String);

impl PackageContextFingerprint {
    #[must_use]
    pub fn from_identity(identity: &WorkspacePackageConfigIdentity) -> Self {
        let WorkspacePackageConfigIdentity {
            workspace_root,
            entrypoint,
        } = identity;
        let mut builder = FingerprintBuilder::new("package-context");
        builder.optional_path_field("workspace_root", workspace_root.as_ref());
        builder.optional_path_field("entrypoint", entrypoint.as_ref());
        Self(builder.finish_hex())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryPolicyFingerprint(String);

impl QueryPolicyFingerprint {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn default_for_cache_family(family: CacheFamily) -> Self {
        Self(family.default_policy().to_string())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheFamily {
    Parse,
    SourceMap,
    HirLowering,
    Diagnostics,
    Lint,
    Format,
    PackageGraph,
    SymbolBuckets,
    FlowGraph,
}

impl CacheFamily {
    fn label(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::SourceMap => "source-map",
            Self::HirLowering => "hir-lowering",
            Self::Diagnostics => "diagnostics",
            Self::Lint => "lint",
            Self::Format => "format",
            Self::PackageGraph => "package-graph",
            Self::SymbolBuckets => "symbol-buckets",
            Self::FlowGraph => "flow-graph",
        }
    }

    fn default_policy(self) -> &'static str {
        match self {
            Self::Parse => PARSER_OPTIONS_VERSION,
            Self::SourceMap => SOURCE_MAP_ALGORITHM_VERSION,
            Self::HirLowering => LOWERING_OPTIONS_VERSION,
            Self::Diagnostics => DIAGNOSTIC_POLICY_VERSION,
            Self::Lint => LINT_POLICY_VERSION,
            Self::Format => FORMAT_POLICY_VERSION,
            Self::PackageGraph => PACKAGE_GRAPH_POLICY_VERSION,
            Self::SymbolBuckets => SYMBOL_BUCKET_POLICY_VERSION,
            Self::FlowGraph => FLOW_GRAPH_POLICY_VERSION,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheKeyContext {
    pub compiler: CompilerFingerprint,
    pub workspace: WorkspaceContextFingerprint,
    pub package: PackageContextFingerprint,
    pub query_policy: QueryPolicyFingerprint,
}

impl CacheKeyContext {
    #[must_use]
    pub fn new(
        family: CacheFamily,
        compiler: CompilerFingerprint,
        workspace: WorkspaceContextFingerprint,
        package: PackageContextFingerprint,
    ) -> Self {
        Self {
            compiler,
            workspace,
            package,
            query_policy: QueryPolicyFingerprint::default_for_cache_family(family),
        }
    }

    #[must_use]
    pub fn from_workspace(
        family: CacheFamily,
        target: &WorkspaceSessionTarget,
        package_identity: &WorkspacePackageConfigIdentity,
    ) -> Self {
        Self::new(
            family,
            CompilerFingerprint::current(),
            WorkspaceContextFingerprint::from_target(target),
            PackageContextFingerprint::from_identity(package_identity),
        )
    }

    #[must_use]
    pub fn with_query_policy(mut self, query_policy: QueryPolicyFingerprint) -> Self {
        self.query_policy = query_policy;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseCacheKey {
    pub source_hash: SourceHash,
    pub parser_options: QueryPolicyFingerprint,
    pub context: CacheKeyContext,
}

impl ParseCacheKey {
    #[must_use]
    pub fn new(source_hash: SourceHash, context: CacheKeyContext) -> Self {
        Self::with_parser_options(
            source_hash,
            QueryPolicyFingerprint::default_for_cache_family(CacheFamily::Parse),
            context,
        )
    }

    #[must_use]
    pub fn with_parser_options(
        source_hash: SourceHash,
        parser_options: QueryPolicyFingerprint,
        context: CacheKeyContext,
    ) -> Self {
        Self {
            source_hash,
            parser_options,
            context,
        }
    }

    #[must_use]
    pub fn fingerprint(&self) -> CacheKeyFingerprint {
        let mut builder = key_builder(CacheFamily::Parse, &self.source_hash, &self.context);
        builder.field("parser_options", self.parser_options.as_str());
        builder.finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceMapCacheKey {
    pub source_hash: SourceHash,
    pub line_map_algorithm: QueryPolicyFingerprint,
    pub context: CacheKeyContext,
}

impl SourceMapCacheKey {
    #[must_use]
    pub fn new(source_hash: SourceHash, context: CacheKeyContext) -> Self {
        Self {
            source_hash,
            line_map_algorithm: QueryPolicyFingerprint::default_for_cache_family(
                CacheFamily::SourceMap,
            ),
            context,
        }
    }

    #[must_use]
    pub fn fingerprint(&self) -> CacheKeyFingerprint {
        let mut builder = key_builder(CacheFamily::SourceMap, &self.source_hash, &self.context);
        builder.field("line_map_algorithm", self.line_map_algorithm.as_str());
        builder.finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HirLoweringCacheKey {
    pub source_hash: SourceHash,
    pub parse_fingerprint: CacheKeyFingerprint,
    pub compiler_options: WorkspaceCompilerOptions,
    pub context: CacheKeyContext,
}

impl HirLoweringCacheKey {
    #[must_use]
    pub fn fingerprint(&self) -> CacheKeyFingerprint {
        let mut builder = key_builder(CacheFamily::HirLowering, &self.source_hash, &self.context);
        builder.field("parse_fingerprint", self.parse_fingerprint.as_str());
        builder.field(
            "compiler_options",
            compiler_options_fingerprint(&self.compiler_options),
        );
        builder.finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticsCacheKey {
    pub source_hash: SourceHash,
    pub hir_fingerprint: CacheKeyFingerprint,
    pub diagnostic_style: FrontendDiagnosticStyle,
    pub context: CacheKeyContext,
}

impl DiagnosticsCacheKey {
    #[must_use]
    pub fn fingerprint(&self) -> CacheKeyFingerprint {
        let mut builder = key_builder(CacheFamily::Diagnostics, &self.source_hash, &self.context);
        builder.field("hir_fingerprint", self.hir_fingerprint.as_str());
        builder.field(
            "diagnostic_style",
            diagnostic_style_label(self.diagnostic_style),
        );
        builder.finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LintCacheKey {
    pub source_hash: SourceHash,
    pub hir_fingerprint: CacheKeyFingerprint,
    pub lint_policy: QueryPolicyFingerprint,
    pub context: CacheKeyContext,
}

impl LintCacheKey {
    #[must_use]
    pub fn fingerprint(&self) -> CacheKeyFingerprint {
        let mut builder = key_builder(CacheFamily::Lint, &self.source_hash, &self.context);
        builder.field("hir_fingerprint", self.hir_fingerprint.as_str());
        builder.field("lint_policy", self.lint_policy.as_str());
        builder.finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatCacheKey {
    pub source_hash: SourceHash,
    pub formatter_policy: QueryPolicyFingerprint,
    pub formatter_options: QueryPolicyFingerprint,
    pub context: CacheKeyContext,
}

impl FormatCacheKey {
    #[must_use]
    pub fn new(source_hash: SourceHash, context: CacheKeyContext) -> Self {
        Self {
            source_hash,
            formatter_policy: QueryPolicyFingerprint::default_for_cache_family(CacheFamily::Format),
            formatter_options: QueryPolicyFingerprint::new(FORMAT_OPTIONS_VERSION),
            context,
        }
    }

    #[must_use]
    pub fn fingerprint(&self) -> CacheKeyFingerprint {
        let mut builder = key_builder(CacheFamily::Format, &self.source_hash, &self.context);
        builder.field("formatter_policy", self.formatter_policy.as_str());
        builder.field("formatter_options", self.formatter_options.as_str());
        builder.finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageGraphCacheKey {
    pub source_hash: SourceHash,
    pub manifest_fingerprint: CacheKeyFingerprint,
    pub context: CacheKeyContext,
}

impl PackageGraphCacheKey {
    #[must_use]
    pub fn fingerprint(&self) -> CacheKeyFingerprint {
        let mut builder = key_builder(CacheFamily::PackageGraph, &self.source_hash, &self.context);
        builder.field("manifest_fingerprint", self.manifest_fingerprint.as_str());
        builder.finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolBucketsCacheKey {
    pub source_hash: SourceHash,
    pub module_graph_fingerprint: CacheKeyFingerprint,
    pub bucket_scope: SymbolBucketScope,
    pub context: CacheKeyContext,
}

impl SymbolBucketsCacheKey {
    #[must_use]
    pub fn fingerprint(&self) -> CacheKeyFingerprint {
        let mut builder = key_builder(CacheFamily::SymbolBuckets, &self.source_hash, &self.context);
        builder.field(
            "module_graph_fingerprint",
            self.module_graph_fingerprint.as_str(),
        );
        builder.field("bucket_scope", self.bucket_scope.label());
        builder.finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolBucketScope {
    Workspace,
    Package,
    Module,
    Stdlib,
}

impl SymbolBucketScope {
    fn label(self) -> &'static str {
        match self {
            Self::Workspace => "workspace",
            Self::Package => "package",
            Self::Module => "module",
            Self::Stdlib => "stdlib",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowGraphCacheKey {
    pub source_hash: SourceHash,
    pub hir_fingerprint: CacheKeyFingerprint,
    pub control_flow_fingerprint: CacheKeyFingerprint,
    pub context: CacheKeyContext,
}

impl FlowGraphCacheKey {
    #[must_use]
    pub fn fingerprint(&self) -> CacheKeyFingerprint {
        let mut builder = key_builder(CacheFamily::FlowGraph, &self.source_hash, &self.context);
        builder.field("hir_fingerprint", self.hir_fingerprint.as_str());
        builder.field(
            "control_flow_fingerprint",
            self.control_flow_fingerprint.as_str(),
        );
        builder.finish()
    }
}

pub(crate) fn stable_source_hash(source: &str) -> SourceHash {
    let mut builder = FingerprintBuilder::new("source-hash");
    builder.field("schema", SOURCE_HASH_SCHEMA_VERSION);
    builder.field("text", source);
    SourceHash(builder.finish_hex())
}

fn key_builder(
    family: CacheFamily,
    source_hash: &SourceHash,
    context: &CacheKeyContext,
) -> CacheKeyBuilder {
    let mut builder = CacheKeyBuilder::new(family);
    builder.field("source_hash", source_hash.as_str());
    builder.field("compiler", context.compiler.as_str());
    builder.field("workspace", context.workspace.as_str());
    builder.field("package", context.package.as_str());
    builder.field("query_policy", context.query_policy.as_str());
    builder
}

fn compiler_options_fingerprint(options: &WorkspaceCompilerOptions) -> String {
    let WorkspaceCompilerOptions { mode } = options;
    let mut builder = FingerprintBuilder::new("workspace-compiler-options");
    builder.field("mode", frontend_mode_label(*mode));
    builder.finish_hex()
}

fn frontend_mode_label(mode: FrontendMode) -> &'static str {
    match mode {
        FrontendMode::SingleFile => "single-file",
        FrontendMode::ProjectEntrypoint => "project-entrypoint",
    }
}

fn diagnostic_style_label(style: FrontendDiagnosticStyle) -> &'static str {
    match style {
        FrontendDiagnosticStyle::Bare => "bare",
        FrontendDiagnosticStyle::ModulePrefixed => "module-prefixed",
    }
}

struct CacheKeyBuilder {
    inner: FingerprintBuilder,
}

impl CacheKeyBuilder {
    fn new(family: CacheFamily) -> Self {
        let mut inner = FingerprintBuilder::new("cache-key");
        inner.field("schema", CACHE_KEY_SCHEMA_VERSION);
        inner.field("family", family.label());
        Self { inner }
    }

    fn field(&mut self, name: &str, value: impl AsRef<str>) {
        self.inner.field(name, value);
    }

    fn finish(self) -> CacheKeyFingerprint {
        CacheKeyFingerprint(self.inner.finish_hex())
    }
}

struct FingerprintBuilder {
    hash: u64,
}

impl FingerprintBuilder {
    fn new(domain: &str) -> Self {
        let mut builder = Self {
            hash: 0xcbf2_9ce4_8422_2325_u64,
        };
        builder.field("domain", domain);
        builder
    }

    fn field(&mut self, name: &str, value: impl AsRef<str>) {
        let value = value.as_ref();
        self.write(name.as_bytes());
        self.write(&[0]);
        self.write(value.len().to_string().as_bytes());
        self.write(&[0]);
        self.write(value.as_bytes());
        self.write(&[0xff]);
    }

    fn path_field(&mut self, name: &str, path: &SourcePath) {
        self.field(name, path.as_path().display().to_string());
    }

    fn optional_path_field(&mut self, name: &str, path: Option<&SourcePath>) {
        if let Some(path) = path {
            self.path_field(name, path);
        } else {
            self.field(name, "<none>");
        }
    }

    fn finish_hex(self) -> String {
        format!("{:016x}", self.hash)
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.hash ^= u64::from(*byte);
            self.hash = self.hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CacheFamily, CacheKeyContext, CacheKeyFingerprint, CompilerFingerprint,
        DiagnosticsCacheKey, FlowGraphCacheKey, FormatCacheKey, HirLoweringCacheKey, LintCacheKey,
        PackageContextFingerprint, PackageGraphCacheKey, ParseCacheKey, QueryPolicyFingerprint,
        SourceMapCacheKey, SymbolBucketScope, SymbolBucketsCacheKey, WorkspaceContextFingerprint,
    };
    use crate::{
        DocumentVersion, FileId, FrontendDiagnosticStyle, FrontendMode, SourceHash, SourcePath,
        WorkspaceCompilerOptions, WorkspacePackageConfigIdentity, WorkspaceSessionTarget,
        WorkspaceSingleFileTarget,
    };

    fn source(value: &str) -> SourceHash {
        SourceHash::from_source_text(value)
    }

    fn fingerprint(value: &str) -> CacheKeyFingerprint {
        CacheKeyFingerprint(value.to_string())
    }

    fn context(family: CacheFamily) -> CacheKeyContext {
        CacheKeyContext::new(
            family,
            CompilerFingerprint("compiler-a".to_string()),
            WorkspaceContextFingerprint("workspace-a".to_string()),
            PackageContextFingerprint("package-a".to_string()),
        )
    }

    #[test]
    fn source_hash_is_deterministic_and_content_sensitive() {
        assert_eq!(source("x = 1").as_str(), source("x = 1").as_str());
        assert_ne!(source("x = 1").as_str(), source("x = 2").as_str());
    }

    #[test]
    fn compiler_fingerprint_is_deterministic() {
        assert_eq!(
            CompilerFingerprint::current().as_str(),
            CompilerFingerprint::current().as_str()
        );
    }

    #[test]
    fn parse_key_includes_source_compiler_workspace_package_and_policy() {
        let base = ParseCacheKey::new(source("x = 1"), context(CacheFamily::Parse));
        let base_fingerprint = base.fingerprint();

        let changed_source = ParseCacheKey::new(source("x = 2"), base.context.clone());
        assert_ne!(base_fingerprint, changed_source.fingerprint());

        let mut changed_compiler = base.clone();
        changed_compiler.context.compiler = CompilerFingerprint("compiler-b".to_string());
        assert_ne!(base_fingerprint, changed_compiler.fingerprint());

        let mut changed_workspace = base.clone();
        changed_workspace.context.workspace =
            WorkspaceContextFingerprint("workspace-b".to_string());
        assert_ne!(base_fingerprint, changed_workspace.fingerprint());

        let mut changed_package = base.clone();
        changed_package.context.package = PackageContextFingerprint("package-b".to_string());
        assert_ne!(base_fingerprint, changed_package.fingerprint());

        let mut changed_policy = base;
        changed_policy.context.query_policy = QueryPolicyFingerprint::new("policy-b");
        assert_ne!(base_fingerprint, changed_policy.fingerprint());

        let custom_parser_options = ParseCacheKey::with_parser_options(
            source("x = 1"),
            QueryPolicyFingerprint::new("parser-options-b"),
            context(CacheFamily::Parse),
        );
        assert_ne!(base_fingerprint, custom_parser_options.fingerprint());
    }

    #[test]
    fn source_map_key_includes_line_map_algorithm() {
        let base = SourceMapCacheKey::new(source("x = 1"), context(CacheFamily::SourceMap));
        let mut changed = base.clone();
        changed.line_map_algorithm = QueryPolicyFingerprint::new("line-map-v2");
        assert_ne!(base.fingerprint(), changed.fingerprint());
    }

    #[test]
    fn hir_key_includes_parse_fingerprint_and_compiler_options() {
        let base = HirLoweringCacheKey {
            source_hash: source("x = 1"),
            parse_fingerprint: fingerprint("parse-a"),
            compiler_options: WorkspaceCompilerOptions {
                mode: FrontendMode::SingleFile,
            },
            context: context(CacheFamily::HirLowering),
        };
        let mut changed_parse = base.clone();
        changed_parse.parse_fingerprint = fingerprint("parse-b");
        assert_ne!(base.fingerprint(), changed_parse.fingerprint());

        let mut changed_options = base.clone();
        changed_options.compiler_options = WorkspaceCompilerOptions {
            mode: FrontendMode::ProjectEntrypoint,
        };
        assert_ne!(base.fingerprint(), changed_options.fingerprint());
    }

    #[test]
    fn diagnostics_key_includes_hir_and_rendering_policy() {
        let base = DiagnosticsCacheKey {
            source_hash: source("x = 1"),
            hir_fingerprint: fingerprint("hir-a"),
            diagnostic_style: FrontendDiagnosticStyle::Bare,
            context: context(CacheFamily::Diagnostics),
        };
        let mut changed_hir = base.clone();
        changed_hir.hir_fingerprint = fingerprint("hir-b");
        assert_ne!(base.fingerprint(), changed_hir.fingerprint());

        let mut changed_style = base.clone();
        changed_style.diagnostic_style = FrontendDiagnosticStyle::ModulePrefixed;
        assert_ne!(base.fingerprint(), changed_style.fingerprint());
    }

    #[test]
    fn lint_format_package_symbol_and_flow_keys_include_family_inputs() {
        let lint = LintCacheKey {
            source_hash: source("x = 1"),
            hir_fingerprint: fingerprint("hir-a"),
            lint_policy: QueryPolicyFingerprint::new("lint-a"),
            context: context(CacheFamily::Lint),
        };
        let mut lint_changed = lint.clone();
        lint_changed.lint_policy = QueryPolicyFingerprint::new("lint-b");
        assert_ne!(lint.fingerprint(), lint_changed.fingerprint());

        let format = FormatCacheKey {
            source_hash: source("x = 1"),
            formatter_policy: QueryPolicyFingerprint::new("format-a"),
            formatter_options: QueryPolicyFingerprint::new("line-width-88"),
            context: context(CacheFamily::Format),
        };
        let mut format_changed = format.clone();
        format_changed.formatter_policy = QueryPolicyFingerprint::new("format-b");
        assert_ne!(format.fingerprint(), format_changed.fingerprint());
        let mut format_options_changed = format.clone();
        format_options_changed.formatter_options = QueryPolicyFingerprint::new("line-width-100");
        assert_ne!(format.fingerprint(), format_options_changed.fingerprint());

        let package = PackageGraphCacheKey {
            source_hash: source("manifest-a"),
            manifest_fingerprint: fingerprint("manifest-a"),
            context: context(CacheFamily::PackageGraph),
        };
        let mut package_changed = package.clone();
        package_changed.manifest_fingerprint = fingerprint("manifest-b");
        assert_ne!(package.fingerprint(), package_changed.fingerprint());

        let symbols = SymbolBucketsCacheKey {
            source_hash: source("x = 1"),
            module_graph_fingerprint: fingerprint("graph-a"),
            bucket_scope: SymbolBucketScope::Workspace,
            context: context(CacheFamily::SymbolBuckets),
        };
        let mut symbols_changed = symbols.clone();
        symbols_changed.bucket_scope = SymbolBucketScope::Package;
        assert_ne!(symbols.fingerprint(), symbols_changed.fingerprint());

        let flow = FlowGraphCacheKey {
            source_hash: source("x = 1"),
            hir_fingerprint: fingerprint("hir-a"),
            control_flow_fingerprint: fingerprint("cfg-a"),
            context: context(CacheFamily::FlowGraph),
        };
        let mut flow_changed = flow.clone();
        flow_changed.control_flow_fingerprint = fingerprint("cfg-b");
        assert_ne!(flow.fingerprint(), flow_changed.fingerprint());
        let mut flow_hir_changed = flow.clone();
        flow_hir_changed.hir_fingerprint = fingerprint("hir-b");
        assert_ne!(flow.fingerprint(), flow_hir_changed.fingerprint());
    }

    #[test]
    fn workspace_and_package_contexts_are_path_sensitive() {
        let first = WorkspaceContextFingerprint::single_file(
            &SourcePath::new("a.sifr"),
            FrontendMode::SingleFile,
        );
        let second = WorkspaceContextFingerprint::single_file(
            &SourcePath::new("b.sifr"),
            FrontendMode::SingleFile,
        );
        assert_ne!(first.as_str(), second.as_str());

        let package = PackageContextFingerprint::from_identity(&WorkspacePackageConfigIdentity {
            workspace_root: Some(SourcePath::new("pkg")),
            entrypoint: Some(SourcePath::new("pkg/main.sifr")),
        });
        let package_changed =
            PackageContextFingerprint::from_identity(&WorkspacePackageConfigIdentity {
                workspace_root: Some(SourcePath::new("pkg")),
                entrypoint: Some(SourcePath::new("pkg/other.sifr")),
            });
        assert_ne!(package.as_str(), package_changed.as_str());
    }

    #[test]
    fn context_from_workspace_uses_target_and_package_identity() {
        let target = WorkspaceSessionTarget::SingleFile(WorkspaceSingleFileTarget {
            path: SourcePath::new("main.sifr"),
            mode: FrontendMode::SingleFile,
        });
        let package = WorkspacePackageConfigIdentity {
            workspace_root: Some(SourcePath::new("pkg")),
            entrypoint: Some(SourcePath::new("pkg/main.sifr")),
        };
        let context = CacheKeyContext::from_workspace(CacheFamily::Parse, &target, &package);
        let changed_context = CacheKeyContext::from_workspace(
            CacheFamily::Parse,
            &target,
            &WorkspacePackageConfigIdentity {
                workspace_root: Some(SourcePath::new("pkg")),
                entrypoint: Some(SourcePath::new("pkg/other.sifr")),
            },
        );

        let key = ParseCacheKey::new(source("x = 1"), context);
        let changed_key = ParseCacheKey::new(source("x = 1"), changed_context);
        assert_ne!(key.fingerprint(), changed_key.fingerprint());
    }

    #[test]
    fn document_identity_inputs_are_intentionally_omitted_from_content_keys() {
        fn parse_key_for_document(
            source_text: &str,
            _version: DocumentVersion,
            _file: FileId,
            _uri: Option<&str>,
        ) -> ParseCacheKey {
            ParseCacheKey::new(source(source_text), context(CacheFamily::Parse))
        }

        fn diagnostics_key_for_document(
            source_text: &str,
            _version: DocumentVersion,
            _file: FileId,
            _uri: Option<&str>,
        ) -> DiagnosticsCacheKey {
            DiagnosticsCacheKey {
                source_hash: source(source_text),
                hir_fingerprint: fingerprint("hir-a"),
                diagnostic_style: FrontendDiagnosticStyle::Bare,
                context: context(CacheFamily::Diagnostics),
            }
        }

        let first = parse_key_for_document(
            "x = 1",
            DocumentVersion::new(1),
            FileId::new(1),
            Some("file:///first.sifr"),
        );
        let second = parse_key_for_document(
            "x = 1",
            DocumentVersion::new(99),
            FileId::new(2),
            Some("file:///second.sifr"),
        );

        assert_eq!(first.fingerprint(), second.fingerprint());

        let diagnostics_first = diagnostics_key_for_document(
            "x = 1",
            DocumentVersion::new(1),
            FileId::new(1),
            Some("file:///first.sifr"),
        );
        let diagnostics_second = diagnostics_key_for_document(
            "x = 1",
            DocumentVersion::new(99),
            FileId::new(2),
            Some("file:///second.sifr"),
        );
        assert_eq!(
            diagnostics_first.fingerprint(),
            diagnostics_second.fingerprint()
        );
    }

    #[test]
    fn package_identity_optional_paths_are_distinct_and_exhaustive() {
        let none = PackageContextFingerprint::from_identity(&WorkspacePackageConfigIdentity {
            workspace_root: None,
            entrypoint: None,
        });
        let root_only = PackageContextFingerprint::from_identity(&WorkspacePackageConfigIdentity {
            workspace_root: Some(SourcePath::new("pkg")),
            entrypoint: None,
        });
        let entry_only =
            PackageContextFingerprint::from_identity(&WorkspacePackageConfigIdentity {
                workspace_root: None,
                entrypoint: Some(SourcePath::new("pkg/main.sifr")),
            });

        assert_ne!(none.as_str(), root_only.as_str());
        assert_ne!(none.as_str(), entry_only.as_str());
        assert_ne!(root_only.as_str(), entry_only.as_str());
    }

    #[test]
    fn project_workspace_context_is_path_sensitive() {
        let first = WorkspaceContextFingerprint::project(&crate::ProjectRoot {
            root: SourcePath::new("pkg"),
            entrypoint: SourcePath::new("pkg/main.sifr"),
        });
        let second = WorkspaceContextFingerprint::project(&crate::ProjectRoot {
            root: SourcePath::new("pkg"),
            entrypoint: SourcePath::new("pkg/other.sifr"),
        });

        assert_ne!(first.as_str(), second.as_str());
    }
}
