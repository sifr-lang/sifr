use super::{
    CompilerFingerprint, ModuleGraphView, OverlayDocument, SourceDependency, SourceDependencyKind,
    SourceHash, SourceMapView, SourcePath, WorkspacePackageConfigIdentity, WorkspaceSessionTarget,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProjectResidencyKind {
    OpenFileOwner,
    AncestorSolution,
    ReferencedByOpenProject,
    ExplicitApiOpen,
    Evictable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectResidencyEntry {
    pub root: Option<SourcePath>,
    pub kind: ProjectResidencyKind,
    pub ref_count: usize,
    pub loaded: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigRegistryEntry {
    pub path: SourcePath,
    pub ref_count: usize,
    pub pending_reload: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum WatchRegistrationReason {
    SeenFile,
    SeenDirectory,
    Config,
    PackageRoot,
    StdlibRoot,
    GeneratedArtifact,
    FailedLookup,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WatchRegistration {
    pub glob: String,
    pub ref_count: usize,
    pub reasons: Vec<WatchRegistrationReason>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SifrBuildInfoSource {
    pub path: SourcePath,
    pub source_hash: SourceHash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SifrBuildInfoCandidate {
    pub path: SourcePath,
    pub compiler: CompilerFingerprint,
    pub package_config_identity: WorkspacePackageConfigIdentity,
    pub sources: Vec<SifrBuildInfoSource>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedSifrBuildInfo {
    pub path: SourcePath,
    pub compiler: CompilerFingerprint,
    pub package_config_identity: WorkspacePackageConfigIdentity,
    pub source_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SifrBuildInfoRejection {
    CompilerFingerprintMismatch,
    PackageConfigMismatch,
    MissingSource,
    SourceHashMismatch,
    ExtraSource,
    WorkspaceNotLoaded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SifrBuildInfoVerification {
    Verified(VerifiedSifrBuildInfo),
    Rejected {
        path: SourcePath,
        reasons: Vec<SifrBuildInfoRejection>,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkspaceResidencySnapshot {
    pub projects: Vec<ProjectResidencyEntry>,
    pub configs: Vec<ConfigRegistryEntry>,
    pub watchers: Vec<WatchRegistration>,
    pub build_info: Option<VerifiedSifrBuildInfo>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct WorkspaceResidencyState {
    projects: Vec<ProjectResidencyEntry>,
    configs: BTreeMap<PathBuf, ConfigRegistryEntry>,
    watchers: BTreeMap<String, WatchRegistration>,
    build_info: Option<VerifiedSifrBuildInfo>,
    retained_stdlib_roots: Vec<SourcePath>,
    retained_generated_artifacts: Vec<SourcePath>,
    pending_config_reloads: Vec<SourcePath>,
}

impl WorkspaceResidencyState {
    #[must_use]
    pub(crate) fn for_target(target: &WorkspaceSessionTarget) -> Self {
        let mut state = Self::default();
        state.reset_projects(target, false);
        state
    }

    pub(crate) fn refresh_after_reload(
        &mut self,
        target: &WorkspaceSessionTarget,
        overlays: &BTreeMap<PathBuf, OverlayDocument>,
        dependencies: &[SourceDependency],
        source_map: Option<&SourceMapView>,
        module_graph: Option<&ModuleGraphView>,
        package_identity: &WorkspacePackageConfigIdentity,
    ) {
        self.reset_projects(target, true);
        self.configs.clear();
        self.watchers.clear();
        self.register_identity(package_identity);
        for overlay in overlays.values() {
            self.retain_project(
                Some(overlay.path.clone()),
                ProjectResidencyKind::OpenFileOwner,
                true,
            );
            self.register_watch_path(overlay.path.as_path(), WatchRegistrationReason::SeenFile);
        }
        for dependency in dependencies {
            self.register_dependency(dependency);
        }
        if let Some(source_map) = source_map {
            for file in &source_map.files {
                self.register_watch_path(
                    file.canonical_path.as_path(),
                    WatchRegistrationReason::SeenFile,
                );
            }
        }
        if let Some(module_graph) = module_graph {
            for module in &module_graph.modules {
                self.register_watch_path(
                    module.canonical_path.as_path(),
                    WatchRegistrationReason::SeenFile,
                );
            }
        }
        for root in self.retained_stdlib_roots.clone() {
            self.register_watch_path(root.as_path(), WatchRegistrationReason::StdlibRoot);
        }
        for artifact in self.retained_generated_artifacts.clone() {
            self.register_watch_path(
                artifact.as_path(),
                WatchRegistrationReason::GeneratedArtifact,
            );
        }
    }

    pub(crate) fn release_open_file_project(&mut self, path: &Path) {
        let path = SourcePath::new(path.to_path_buf());
        self.projects.retain(|project| {
            !(project.kind == ProjectResidencyKind::OpenFileOwner
                && project.root.as_ref() == Some(&path))
        });
    }

    pub(crate) fn mark_config_pending_reload(&mut self, path: SourcePath) {
        if !self.pending_config_reloads.contains(&path) {
            self.pending_config_reloads.push(path.clone());
        }
        let key = path.as_path().to_path_buf();
        let entry = self.configs.entry(key).or_insert(ConfigRegistryEntry {
            path,
            ref_count: 1,
            pending_reload: true,
        });
        entry.pending_reload = true;
        entry.ref_count = entry.ref_count.max(1);
        let watch_path = entry.path.as_path().to_path_buf();
        self.register_watch_path(&watch_path, WatchRegistrationReason::Config);
    }

    pub(crate) fn retain_stdlib_root(&mut self, path: SourcePath) {
        let should_retain = !self.retained_stdlib_roots.contains(&path);
        self.register_watch_path(path.as_path(), WatchRegistrationReason::StdlibRoot);
        if should_retain {
            self.retained_stdlib_roots.push(path);
        }
    }

    pub(crate) fn retain_generated_artifact(&mut self, path: SourcePath) {
        let should_retain = !self.retained_generated_artifacts.contains(&path);
        self.register_watch_path(path.as_path(), WatchRegistrationReason::GeneratedArtifact);
        if should_retain {
            self.retained_generated_artifacts.push(path);
        }
    }

    pub(crate) fn verify_build_info(
        &mut self,
        candidate: SifrBuildInfoCandidate,
        source_map: Option<&SourceMapView>,
        package_identity: &WorkspacePackageConfigIdentity,
    ) -> SifrBuildInfoVerification {
        let mut reasons = Vec::new();
        let current_compiler = CompilerFingerprint::current();
        if candidate.compiler != current_compiler {
            reasons.push(SifrBuildInfoRejection::CompilerFingerprintMismatch);
        }
        if &candidate.package_config_identity != package_identity {
            reasons.push(SifrBuildInfoRejection::PackageConfigMismatch);
        }
        let Some(source_map) = source_map else {
            reasons.push(SifrBuildInfoRejection::WorkspaceNotLoaded);
            return self.reject_build_info(candidate.path, reasons);
        };
        for file in &source_map.files {
            match candidate
                .sources
                .iter()
                .find(|source| source.path == file.canonical_path)
            {
                Some(source) if source.source_hash == file.source_hash => {}
                Some(_) => push_rejection(&mut reasons, SifrBuildInfoRejection::SourceHashMismatch),
                None => push_rejection(&mut reasons, SifrBuildInfoRejection::MissingSource),
            }
        }
        for source in &candidate.sources {
            if !source_map
                .files
                .iter()
                .any(|file| file.canonical_path == source.path)
            {
                push_rejection(&mut reasons, SifrBuildInfoRejection::ExtraSource);
            }
        }
        if !reasons.is_empty() {
            return self.reject_build_info(candidate.path, reasons);
        }
        let verified = VerifiedSifrBuildInfo {
            path: candidate.path.clone(),
            compiler: current_compiler,
            package_config_identity: package_identity.clone(),
            source_count: source_map.files.len(),
        };
        self.build_info = Some(verified.clone());
        self.retain_generated_artifact(candidate.path);
        SifrBuildInfoVerification::Verified(verified)
    }

    #[must_use]
    pub(crate) fn snapshot(&self) -> Arc<WorkspaceResidencySnapshot> {
        Arc::new(WorkspaceResidencySnapshot {
            projects: self.projects.clone(),
            configs: self.configs.values().cloned().collect(),
            watchers: self.watchers.values().cloned().collect(),
            build_info: self.build_info.clone(),
        })
    }

    fn reject_build_info(
        &mut self,
        path: SourcePath,
        reasons: Vec<SifrBuildInfoRejection>,
    ) -> SifrBuildInfoVerification {
        self.build_info = None;
        SifrBuildInfoVerification::Rejected { path, reasons }
    }

    fn reset_projects(&mut self, target: &WorkspaceSessionTarget, loaded: bool) {
        self.projects.clear();
        match target {
            WorkspaceSessionTarget::Project(root) => {
                self.retain_project(
                    Some(root.root.clone()),
                    if loaded {
                        ProjectResidencyKind::ExplicitApiOpen
                    } else {
                        ProjectResidencyKind::Evictable
                    },
                    loaded,
                );
                self.retain_project(
                    Some(root.entrypoint.clone()),
                    ProjectResidencyKind::OpenFileOwner,
                    loaded,
                );
            }
            WorkspaceSessionTarget::SingleFile(target) => {
                self.retain_project(
                    Some(target.path.clone()),
                    ProjectResidencyKind::OpenFileOwner,
                    loaded,
                );
            }
        }
    }

    fn retain_project(
        &mut self,
        root: Option<SourcePath>,
        kind: ProjectResidencyKind,
        loaded: bool,
    ) {
        if let Some(project) = self
            .projects
            .iter_mut()
            .find(|project| project.root == root && project.kind == kind)
        {
            project.ref_count += 1;
            project.loaded |= loaded;
            return;
        }
        self.projects.push(ProjectResidencyEntry {
            root,
            kind,
            ref_count: 1,
            loaded,
        });
        self.projects.sort_by(|left, right| {
            left.kind.cmp(&right.kind).then_with(|| {
                display_path(left.root.as_ref()).cmp(&display_path(right.root.as_ref()))
            })
        });
    }

    fn register_identity(&mut self, identity: &WorkspacePackageConfigIdentity) {
        if let Some(root) = &identity.workspace_root {
            let config = SourcePath::new(root.as_path().join("sifr.toml"));
            self.register_config(config);
            self.register_watch_path(root.as_path(), WatchRegistrationReason::PackageRoot);
        }
        if let Some(entrypoint) = &identity.entrypoint {
            self.register_watch_path(entrypoint.as_path(), WatchRegistrationReason::SeenFile);
        }
    }

    fn register_config(&mut self, path: SourcePath) {
        let key = path.as_path().to_path_buf();
        let pending_reload = self.pending_config_reloads.contains(&path);
        if let Some(entry) = self.configs.get_mut(&key) {
            entry.ref_count += 1;
            entry.pending_reload |= pending_reload;
        } else {
            self.configs.insert(
                key.clone(),
                ConfigRegistryEntry {
                    path,
                    ref_count: 1,
                    pending_reload,
                },
            );
        }
        self.register_watch_path(&key, WatchRegistrationReason::Config);
    }

    fn register_dependency(&mut self, dependency: &SourceDependency) {
        let reason = match dependency.kind {
            SourceDependencyKind::FileRead
            | SourceDependencyKind::FileProbe { .. }
            | SourceDependencyKind::Canonicalize => WatchRegistrationReason::SeenFile,
            SourceDependencyKind::DirectoryRead | SourceDependencyKind::DirectoryProbe { .. } => {
                WatchRegistrationReason::SeenDirectory
            }
            SourceDependencyKind::FailedLookup => WatchRegistrationReason::FailedLookup,
        };
        self.register_watch_path(&dependency.path, reason);
    }

    fn register_watch_path(&mut self, path: &Path, reason: WatchRegistrationReason) {
        let glob = watch_glob(path, reason);
        let entry = self
            .watchers
            .entry(glob.clone())
            .or_insert(WatchRegistration {
                glob,
                ref_count: 0,
                reasons: Vec::new(),
            });
        entry.ref_count += 1;
        if !entry.reasons.contains(&reason) {
            entry.reasons.push(reason);
            entry.reasons.sort();
        }
    }
}

fn watch_glob(path: &Path, reason: WatchRegistrationReason) -> String {
    let rendered = path.display().to_string();
    match reason {
        WatchRegistrationReason::SeenDirectory
        | WatchRegistrationReason::PackageRoot
        | WatchRegistrationReason::StdlibRoot => format!("{rendered}/**"),
        WatchRegistrationReason::FailedLookup => path
            .parent()
            .map_or(rendered, |parent| format!("{}/**", parent.display())),
        WatchRegistrationReason::Config
        | WatchRegistrationReason::GeneratedArtifact
        | WatchRegistrationReason::SeenFile => rendered,
    }
}

fn display_path(path: Option<&SourcePath>) -> String {
    path.map_or_else(String::new, |path| path.as_path().display().to_string())
}

fn push_rejection(rejections: &mut Vec<SifrBuildInfoRejection>, rejection: SifrBuildInfoRejection) {
    if !rejections.contains(&rejection) {
        rejections.push(rejection);
    }
}
