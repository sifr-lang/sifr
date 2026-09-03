use sifr_sysroot::ResolvedSysroot;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

/// Canonical public stdlib module source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StdlibSource {
    pub module: &'static str,
    pub source: &'static str,
}

/// Stdlib module loaded from the resolved sysroot source tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedStdlibSource {
    pub module: String,
    pub source: String,
    pub path: PathBuf,
    pub kind: LoadedStdlibSourceKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadedStdlibSourceKind {
    Public,
    PrivateDeclaration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StdlibSourceInventoryError {
    pub message: String,
    pub path: Option<PathBuf>,
}

struct BootstrapSource<'a> {
    module: &'a str,
    source: &'a str,
    path: Option<&'a Path>,
}

impl StdlibSourceInventoryError {
    fn new(message: impl Into<String>, path: Option<PathBuf>) -> Self {
        Self {
            message: message.into(),
            path,
        }
    }
}

impl fmt::Display for StdlibSourceInventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.path {
            Some(path) => write!(f, "{}: {}", path.display(), self.message),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for StdlibSourceInventoryError {}

pub const PRIVATE_STDLIB_MODULES: &[&str] = &[
    "_sifr.calendar",
    "_sifr.compress",
    "_sifr.crypto",
    "_sifr.datetime",
    "_sifr.encoding",
    "_sifr.fs",
    "_sifr.html",
    "_sifr.http",
    "_sifr.i18n",
    "_sifr.json",
    "_sifr.logging",
    "_sifr.math",
    "_sifr.net",
    "_sifr.platform",
    "_sifr.process",
    "_sifr.python",
    "_sifr.regex",
    "_sifr.runtime",
    "_sifr.signal",
    "_sifr.sys",
    "_sifr.time",
    "_sifr.tls",
    "_sifr.toml",
    "_sifr.unicode",
    "_sifr.url",
    "_sifr.uuid",
];

pub const STDLIB_SOURCES: &[StdlibSource] = &[
    StdlibSource {
        module: "sifr.sql",
        source: include_str!("../../../stdlib/sifr/sql.sifr"),
    },
    StdlibSource {
        module: "sifr.sql.migration",
        source: include_str!("../../../stdlib/sifr/sql.migration.sifr"),
    },
    StdlibSource {
        module: "sifr.meta",
        source: include_str!("../../../stdlib/sifr/meta.sifr"),
    },
    StdlibSource {
        module: "sifr.test",
        source: include_str!("../../../stdlib/sifr/test.sifr"),
    },
    StdlibSource {
        module: "sifr.env",
        source: include_str!("../../../stdlib/sifr/env.sifr"),
    },
    StdlibSource {
        module: "sifr.encoding",
        source: include_str!("../../../stdlib/sifr/encoding.sifr"),
    },
    StdlibSource {
        module: "sifr.unicode",
        source: include_str!("../../../stdlib/sifr/unicode.sifr"),
    },
    StdlibSource {
        module: "sifr.i18n",
        source: include_str!("../../../stdlib/sifr/i18n.sifr"),
    },
    StdlibSource {
        module: "sifr.base64",
        source: include_str!("../../../stdlib/sifr/base64.sifr"),
    },
    StdlibSource {
        module: "sifr.math",
        source: include_str!("../../../stdlib/sifr/math.sifr"),
    },
    StdlibSource {
        module: "sifr.hashlib",
        source: include_str!("../../../stdlib/sifr/hashlib.sifr"),
    },
    StdlibSource {
        module: "sifr.io",
        source: include_str!("../../../stdlib/sifr/io.sifr"),
    },
    StdlibSource {
        module: "sifr.os",
        source: include_str!("../../../stdlib/sifr/os.sifr"),
    },
    StdlibSource {
        module: "sifr.json",
        source: include_str!("../../../stdlib/sifr/json.sifr"),
    },
    StdlibSource {
        module: "sifr.time",
        source: include_str!("../../../stdlib/sifr/time.sifr"),
    },
    StdlibSource {
        module: "sifr.random",
        source: include_str!("../../../stdlib/sifr/random.sifr"),
    },
    StdlibSource {
        module: "sifr.re",
        source: include_str!("../../../stdlib/sifr/re.sifr"),
    },
    StdlibSource {
        module: "sifr.collections",
        source: include_str!("../../../stdlib/sifr/collections.sifr"),
    },
    StdlibSource {
        module: "sifr.sync",
        source: include_str!("../../../stdlib/sifr/sync.sifr"),
    },
    StdlibSource {
        module: "sifr.task",
        source: include_str!("../../../stdlib/sifr/task.sifr"),
    },
    StdlibSource {
        module: "sifr.parallel",
        source: include_str!("../../../stdlib/sifr/parallel.sifr"),
    },
    StdlibSource {
        module: "sifr.process",
        source: include_str!("../../../stdlib/sifr/process.sifr"),
    },
    StdlibSource {
        module: "sifr.python_core",
        source: include_str!("../../../stdlib/sifr/python_core.sifr"),
    },
    StdlibSource {
        module: "sifr.python",
        source: include_str!("../../../stdlib/sifr/python.sifr"),
    },
    StdlibSource {
        module: "sifr.net",
        source: include_str!("../../../stdlib/sifr/net.sifr"),
    },
    StdlibSource {
        module: "sifr.tls",
        source: include_str!("../../../stdlib/sifr/tls.sifr"),
    },
    StdlibSource {
        module: "sifr.url",
        source: include_str!("../../../stdlib/sifr/url.sifr"),
    },
    StdlibSource {
        module: "sifr.http",
        source: include_str!("../../../stdlib/sifr/http.sifr"),
    },
    StdlibSource {
        module: "sifr.resource",
        source: include_str!("../../../stdlib/sifr/resource.sifr"),
    },
    StdlibSource {
        module: "sifr.signal",
        source: include_str!("../../../stdlib/sifr/signal.sifr"),
    },
    StdlibSource {
        module: "sifr.runtime",
        source: include_str!("../../../stdlib/sifr/runtime.sifr"),
    },
    StdlibSource {
        module: "sifr.ipc",
        source: include_str!("../../../stdlib/sifr/ipc.sifr"),
    },
    StdlibSource {
        module: "sifr.string",
        source: include_str!("../../../stdlib/sifr/string.sifr"),
    },
    StdlibSource {
        module: "sifr.bisect",
        source: include_str!("../../../stdlib/sifr/bisect.sifr"),
    },
    StdlibSource {
        module: "sifr.functools",
        source: include_str!("../../../stdlib/sifr/functools.sifr"),
    },
    StdlibSource {
        module: "sifr.secrets",
        source: include_str!("../../../stdlib/sifr/secrets.sifr"),
    },
    StdlibSource {
        module: "sifr.graphlib",
        source: include_str!("../../../stdlib/sifr/graphlib.sifr"),
    },
    StdlibSource {
        module: "sifr.uuid",
        source: include_str!("../../../stdlib/sifr/uuid.sifr"),
    },
    StdlibSource {
        module: "sifr.platform",
        source: include_str!("../../../stdlib/sifr/platform.sifr"),
    },
    StdlibSource {
        module: "sifr.pathlib",
        source: include_str!("../../../stdlib/sifr/pathlib.sifr"),
    },
    StdlibSource {
        module: "sifr.logging",
        source: include_str!("../../../stdlib/sifr/logging.sifr"),
    },
    StdlibSource {
        module: "sifr.heapq",
        source: include_str!("../../../stdlib/sifr/heapq.sifr"),
    },
    StdlibSource {
        module: "sifr.itertools",
        source: include_str!("../../../stdlib/sifr/itertools.sifr"),
    },
    StdlibSource {
        module: "sifr.textwrap",
        source: include_str!("../../../stdlib/sifr/textwrap.sifr"),
    },
    StdlibSource {
        module: "sifr.csv",
        source: include_str!("../../../stdlib/sifr/csv.sifr"),
    },
    StdlibSource {
        module: "sifr.argparse",
        source: include_str!("../../../stdlib/sifr/argparse.sifr"),
    },
    StdlibSource {
        module: "sifr.fnmatch",
        source: include_str!("../../../stdlib/sifr/fnmatch.sifr"),
    },
    StdlibSource {
        module: "sifr.shutil",
        source: include_str!("../../../stdlib/sifr/shutil.sifr"),
    },
    StdlibSource {
        module: "sifr.tempfile",
        source: include_str!("../../../stdlib/sifr/tempfile.sifr"),
    },
    StdlibSource {
        module: "sifr.difflib",
        source: include_str!("../../../stdlib/sifr/difflib.sifr"),
    },
    StdlibSource {
        module: "sifr.ipaddress",
        source: include_str!("../../../stdlib/sifr/ipaddress.sifr"),
    },
    StdlibSource {
        module: "sifr.timeit",
        source: include_str!("../../../stdlib/sifr/timeit.sifr"),
    },
    StdlibSource {
        module: "sifr.tomllib",
        source: include_str!("../../../stdlib/sifr/tomllib.sifr"),
    },
    StdlibSource {
        module: "sifr.datetime",
        source: include_str!("../../../stdlib/sifr/datetime.sifr"),
    },
    StdlibSource {
        module: "sifr.operator",
        source: include_str!("../../../stdlib/sifr/operator.sifr"),
    },
    StdlibSource {
        module: "sifr.calendar",
        source: include_str!("../../../stdlib/sifr/calendar.sifr"),
    },
    StdlibSource {
        module: "sifr.html",
        source: include_str!("../../../stdlib/sifr/html.sifr"),
    },
    StdlibSource {
        module: "sifr.sys",
        source: include_str!("../../../stdlib/sifr/sys.sifr"),
    },
    StdlibSource {
        module: "sifr.gzip",
        source: include_str!("../../../stdlib/sifr/gzip.sifr"),
    },
    StdlibSource {
        module: "sifr.zipfile",
        source: include_str!("../../../stdlib/sifr/zipfile.sifr"),
    },
    StdlibSource {
        module: "sifr.configparser",
        source: include_str!("../../../stdlib/sifr/configparser.sifr"),
    },
    StdlibSource {
        module: "sifr.statistics",
        source: include_str!("../../../stdlib/sifr/statistics.sifr"),
    },
    StdlibSource {
        module: "sifr.glob",
        source: include_str!("../../../stdlib/sifr/glob.sifr"),
    },
];

pub fn load_stdlib_sources_from_sysroot(
    sysroot: &ResolvedSysroot,
) -> Result<Vec<LoadedStdlibSource>, StdlibSourceInventoryError> {
    validate_stdlib_source_inventory(sysroot)?;
    let sources = STDLIB_SOURCES
        .iter()
        .map(|source| {
            let path = public_module_path(&sysroot.paths.stdlib_public_sources, source.module);
            let loaded = std::fs::read_to_string(&path).map_err(|error| {
                StdlibSourceInventoryError::new(
                    format!("failed to read stdlib module {}: {error}", source.module),
                    Some(path.clone()),
                )
            })?;
            Ok(LoadedStdlibSource {
                module: source.module.to_string(),
                source: loaded,
                path,
                kind: LoadedStdlibSourceKind::Public,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_loaded_public_stdlib_bootstrap_order(&sources)?;
    Ok(sources)
}

pub fn load_stdlib_tooling_sources_from_sysroot(
    sysroot: &ResolvedSysroot,
) -> Result<Vec<LoadedStdlibSource>, StdlibSourceInventoryError> {
    validate_stdlib_source_inventory(sysroot)?;
    let mut sources = Vec::new();
    for module in PRIVATE_STDLIB_MODULES {
        let path = private_module_path(&sysroot.paths.stdlib_private_sources, module);
        let loaded = std::fs::read_to_string(&path).map_err(|error| {
            StdlibSourceInventoryError::new(
                format!("failed to read private stdlib module {module}: {error}"),
                Some(path.clone()),
            )
        })?;
        sources.push(LoadedStdlibSource {
            module: (*module).to_string(),
            source: loaded,
            path,
            kind: LoadedStdlibSourceKind::PrivateDeclaration,
        });
    }
    validate_loaded_private_stdlib_declarations_are_dependency_free(&sources)?;
    sources.extend(load_stdlib_sources_from_sysroot(sysroot)?);
    Ok(sources)
}

pub fn validate_stdlib_source_inventory(
    sysroot: &ResolvedSysroot,
) -> Result<(), StdlibSourceInventoryError> {
    validate_unique_modules(
        STDLIB_SOURCES.iter().map(|source| source.module),
        "public stdlib source",
    )?;
    validate_unique_modules(
        PRIVATE_STDLIB_MODULES.iter().copied(),
        "private stdlib source",
    )?;
    validate_static_public_stdlib_bootstrap_order()?;
    validate_module_files(
        &sysroot.paths.stdlib_public_sources,
        STDLIB_SOURCES.iter().map(|source| source.module),
        "sifr",
    )?;
    validate_module_files(
        &sysroot.paths.stdlib_private_sources,
        PRIVATE_STDLIB_MODULES.iter().copied(),
        "_sifr",
    )
}

fn validate_unique_modules<'a>(
    modules: impl Iterator<Item = &'a str>,
    label: &str,
) -> Result<(), StdlibSourceInventoryError> {
    let mut seen = BTreeSet::new();
    for module in modules {
        if !seen.insert(module) {
            return Err(StdlibSourceInventoryError::new(
                format!("duplicate {label} module {module}"),
                None,
            ));
        }
    }
    Ok(())
}

fn validate_module_files<'a>(
    root: &Path,
    modules: impl Iterator<Item = &'a str>,
    prefix: &str,
) -> Result<(), StdlibSourceInventoryError> {
    let expected = modules.map(str::to_string).collect::<BTreeSet<_>>();
    let mut actual = BTreeSet::new();
    let entries = std::fs::read_dir(root).map_err(|error| {
        StdlibSourceInventoryError::new(
            format!("failed to read stdlib source directory: {error}"),
            Some(root.to_path_buf()),
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            StdlibSourceInventoryError::new(
                format!("failed to inspect stdlib source entry: {error}"),
                Some(root.to_path_buf()),
            )
        })?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("sifr") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            return Err(StdlibSourceInventoryError::new(
                "stdlib source filename is not valid UTF-8",
                Some(path),
            ));
        };
        actual.insert(format!("{prefix}.{stem}"));
    }
    if let Some(module) = expected.difference(&actual).next() {
        return Err(StdlibSourceInventoryError::new(
            format!("missing stdlib source module {module}"),
            Some(module_path(root, module, prefix)),
        ));
    }
    if let Some(module) = actual.difference(&expected).next() {
        return Err(StdlibSourceInventoryError::new(
            format!("stale stdlib source module {module}"),
            Some(module_path(root, module, prefix)),
        ));
    }
    Ok(())
}

fn validate_static_public_stdlib_bootstrap_order() -> Result<(), StdlibSourceInventoryError> {
    validate_public_stdlib_bootstrap_order(STDLIB_SOURCES.iter().map(|source| BootstrapSource {
        module: source.module,
        source: source.source,
        path: None,
    }))
}

fn validate_loaded_public_stdlib_bootstrap_order(
    sources: &[LoadedStdlibSource],
) -> Result<(), StdlibSourceInventoryError> {
    validate_public_stdlib_bootstrap_order(sources.iter().map(|source| BootstrapSource {
        module: &source.module,
        source: &source.source,
        path: Some(source.path.as_path()),
    }))
}

fn validate_public_stdlib_bootstrap_order<'a>(
    sources: impl IntoIterator<Item = BootstrapSource<'a>>,
) -> Result<(), StdlibSourceInventoryError> {
    let sources = sources.into_iter().collect::<Vec<_>>();
    let module_indices = sources
        .iter()
        .enumerate()
        .map(|(index, source)| (source.module, index))
        .collect::<BTreeMap<_, _>>();
    for (index, source) in sources.iter().enumerate() {
        for imported_module in stdlib_imports(source.source, "sifr.") {
            let Some(imported_index) = module_indices.get(imported_module) else {
                return Err(StdlibSourceInventoryError::new(
                    format!(
                        "public stdlib module {} imports unknown stdlib module {imported_module}",
                        source.module
                    ),
                    source.path.map(Path::to_path_buf),
                ));
            };
            if *imported_index > index {
                return Err(StdlibSourceInventoryError::new(
                    format!(
                        "public stdlib module {} imports {imported_module} before it is available in bootstrap order",
                        source.module
                    ),
                    source.path.map(Path::to_path_buf),
                ));
            }
        }
    }
    Ok(())
}

fn validate_loaded_private_stdlib_declarations_are_dependency_free(
    sources: &[LoadedStdlibSource],
) -> Result<(), StdlibSourceInventoryError> {
    for source in sources {
        if let Some(imported_module) = stdlib_imports(&source.source, "_sifr.")
            .into_iter()
            .chain(stdlib_imports(&source.source, "sifr."))
            .next()
        {
            return Err(StdlibSourceInventoryError::new(
                format!(
                    "private stdlib declaration {} imports {imported_module}; private declarations must be dependency-free",
                    source.module
                ),
                Some(source.path.clone()),
            ));
        }
    }
    Ok(())
}

fn stdlib_imports<'a>(source: &'a str, prefix: &str) -> BTreeSet<&'a str> {
    let mut imports = BTreeSet::new();
    for line in source.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if let Some(module) = line
            .strip_prefix("from ")
            .and_then(|rest| rest.split_once(" import ").map(|(module, _)| module))
        {
            if module.starts_with(prefix) {
                imports.insert(module);
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("import ") {
            for item in rest.split(',') {
                let module = item.split_whitespace().next().unwrap_or("");
                if module.starts_with(prefix) {
                    imports.insert(module);
                }
            }
        }
    }
    imports
}

fn public_module_path(root: &Path, module: &str) -> PathBuf {
    module_path(root, module, "sifr")
}

fn private_module_path(root: &Path, module: &str) -> PathBuf {
    module_path(root, module, "_sifr")
}

fn module_path(root: &Path, module: &str, prefix: &str) -> PathBuf {
    let filename = module
        .strip_prefix(prefix)
        .and_then(|tail| tail.strip_prefix('.'))
        .unwrap_or(module);
    root.join(format!("{filename}.sifr"))
}

#[cfg(test)]
mod tests {
    use super::{
        PRIVATE_STDLIB_MODULES, STDLIB_SOURCES, load_stdlib_sources_from_sysroot,
        load_stdlib_tooling_sources_from_sysroot, validate_stdlib_source_inventory,
    };
    use crate::test_support::{TestExpectErr as _, TestUnwrap as _};
    use sifr_sysroot::{ResolvedSysroot, SysrootManifest, SysrootPaths};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempRoot {
        path: PathBuf,
    }

    impl TempRoot {
        fn new(label: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .test_unwrap("system clock should be monotonic enough for test paths")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "sifr_stdlib_sources_{label}_{}_{}",
                std::process::id(),
                unique
            ));
            fs::create_dir_all(&path).test_unwrap("temp root should be created");
            Self { path }
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn load_stdlib_sources_reads_physical_sysroot_files() {
        let root = complete_source_tree("physical_sources");
        let sysroot = resolved_sysroot(&root.path);
        let json_path = root.path.join("stdlib/sifr/json.sifr");
        fs::write(&json_path, "def physical_marker() -> int:\n    return 1\n")
            .test_unwrap("overwrite physical json source");

        let sources =
            load_stdlib_sources_from_sysroot(&sysroot).test_unwrap("inventory should load");
        let json = sources
            .iter()
            .find(|source| source.module == "sifr.json")
            .test_unwrap("sifr.json should load");

        assert_eq!(json.path, json_path);
        assert!(json.source.contains("physical_marker"));
    }

    #[test]
    fn source_tree_inventory_matches_static_public_and_private_modules() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .test_unwrap("crate should live under crates/<name>");
        let sysroot = resolved_sysroot(workspace_root);

        validate_stdlib_source_inventory(&sysroot)
            .test_unwrap("source tree inventory should validate");
    }

    #[test]
    fn source_inventory_rejects_missing_public_modules() {
        let root = complete_source_tree("missing_public");
        let sysroot = resolved_sysroot(&root.path);
        fs::remove_file(root.path.join("stdlib/sifr/json.sifr"))
            .test_unwrap("remove public source");

        let error = validate_stdlib_source_inventory(&sysroot)
            .test_expect_err("missing public source should fail");

        assert!(
            error
                .message
                .contains("missing stdlib source module sifr.json")
        );
        assert_eq!(error.path, Some(root.path.join("stdlib/sifr/json.sifr")));
    }

    #[test]
    fn source_inventory_rejects_missing_private_modules() {
        let root = complete_source_tree("missing_private");
        let sysroot = resolved_sysroot(&root.path);
        fs::remove_file(root.path.join("stdlib/_sifr/fs.sifr"))
            .test_unwrap("remove private source");

        let error = validate_stdlib_source_inventory(&sysroot)
            .test_expect_err("missing private source should fail");

        assert!(
            error
                .message
                .contains("missing stdlib source module _sifr.fs")
        );
        assert_eq!(error.path, Some(root.path.join("stdlib/_sifr/fs.sifr")));
    }

    #[test]
    fn source_inventory_rejects_stale_private_modules() {
        let root = complete_source_tree("stale_private");
        let sysroot = resolved_sysroot(&root.path);
        fs::write(root.path.join("stdlib/_sifr/stale.sifr"), "# stale\n")
            .test_unwrap("write stale private source");

        let error = validate_stdlib_source_inventory(&sysroot)
            .test_expect_err("stale private source should fail");

        assert!(
            error
                .message
                .contains("stale stdlib source module _sifr.stale")
        );
        assert_eq!(error.path, Some(root.path.join("stdlib/_sifr/stale.sifr")));
    }

    #[test]
    fn load_stdlib_sources_rejects_forward_public_bootstrap_imports() {
        let root = complete_source_tree("forward_public_import");
        let sysroot = resolved_sysroot(&root.path);
        let test_path = root.path.join("stdlib/sifr/test.sifr");
        fs::write(&test_path, "from sifr.json import JsonValue\n")
            .test_unwrap("write forward import source");

        let error = load_stdlib_sources_from_sysroot(&sysroot)
            .test_expect_err("forward public stdlib import should fail");

        assert!(
            error.message.contains(
                "public stdlib module sifr.test imports sifr.json before it is available"
            )
        );
        assert_eq!(error.path, Some(test_path));
    }

    #[test]
    fn load_stdlib_sources_rejects_unknown_public_bootstrap_imports() {
        let root = complete_source_tree("unknown_public_import");
        let sysroot = resolved_sysroot(&root.path);
        let test_path = root.path.join("stdlib/sifr/test.sifr");
        fs::write(&test_path, "from sifr.missing import Missing\n")
            .test_unwrap("write unknown import source");

        let error = load_stdlib_sources_from_sysroot(&sysroot)
            .test_expect_err("unknown public stdlib import should fail");

        assert!(
            error.message.contains(
                "public stdlib module sifr.test imports unknown stdlib module sifr.missing"
            )
        );
        assert_eq!(error.path, Some(test_path));
    }

    #[test]
    fn load_stdlib_sources_rejects_public_import_cycles() {
        let root = complete_source_tree("cycle_public_import");
        let sysroot = resolved_sysroot(&root.path);
        let encoding_path = root.path.join("stdlib/sifr/encoding.sifr");
        let unicode_path = root.path.join("stdlib/sifr/unicode.sifr");
        fs::write(&encoding_path, "from sifr.unicode import Unicode\n")
            .test_unwrap("write forward half of cycle");
        fs::write(&unicode_path, "from sifr.encoding import Encoding\n")
            .test_unwrap("write backward half of cycle");

        let error = load_stdlib_sources_from_sysroot(&sysroot)
            .test_expect_err("public stdlib import cycle should fail");

        assert!(error.message.contains(
            "public stdlib module sifr.encoding imports sifr.unicode before it is available"
        ));
        assert_eq!(error.path, Some(encoding_path));
    }

    #[test]
    fn load_stdlib_tooling_sources_rejects_private_declaration_imports() {
        let root = complete_source_tree("private_declaration_import");
        let sysroot = resolved_sysroot(&root.path);
        let math_path = root.path.join("stdlib/_sifr/math.sifr");
        fs::write(&math_path, "from _sifr.fs import open\n").test_unwrap("write private import");

        let error = load_stdlib_tooling_sources_from_sysroot(&sysroot)
            .test_expect_err("private stdlib declaration import should fail");

        assert!(
            error
                .message
                .contains("private stdlib declaration _sifr.math imports _sifr.fs")
        );
        assert_eq!(error.path, Some(math_path));
    }

    #[test]
    fn load_stdlib_tooling_sources_rejects_private_declaration_public_imports() {
        let root = complete_source_tree("private_declaration_public_import");
        let sysroot = resolved_sysroot(&root.path);
        let math_path = root.path.join("stdlib/_sifr/math.sifr");
        fs::write(&math_path, "from sifr.encoding import Encoding\n")
            .test_unwrap("write public import");

        let error = load_stdlib_tooling_sources_from_sysroot(&sysroot)
            .test_expect_err("private stdlib declaration public import should fail");

        assert!(
            error
                .message
                .contains("private stdlib declaration _sifr.math imports sifr.encoding")
        );
        assert_eq!(error.path, Some(math_path));
    }

    fn complete_source_tree(label: &str) -> TempRoot {
        let root = TempRoot::new(label);
        let public_root = root.path.join("stdlib/sifr");
        let private_root = root.path.join("stdlib/_sifr");
        fs::create_dir_all(&public_root).test_unwrap("public stdlib source root");
        fs::create_dir_all(&private_root).test_unwrap("private stdlib source root");
        for source in STDLIB_SOURCES {
            fs::write(
                public_module_path(&public_root, source.module),
                source.source,
            )
            .test_unwrap("write public source");
        }
        for module in PRIVATE_STDLIB_MODULES {
            fs::write(
                private_module_path(&private_root, module),
                "# private declaration\n",
            )
            .test_unwrap("write private source");
        }
        root
    }

    fn resolved_sysroot(root: &Path) -> ResolvedSysroot {
        ResolvedSysroot {
            root: root.to_path_buf(),
            manifest: SysrootManifest {
                schema_version: 1,
                sifr_version: "0.0.0-dev".to_string(),
                target_triple: "test-target".to_string(),
                built_by_compiler_commit: "test".to_string(),
                sysroot_content_sha256:
                    "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                cargo_lock_sha256:
                    "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            },
            paths: SysrootPaths::from_root(root),
            cargo_lock_content_sha256: "2".repeat(64),
        }
    }

    fn public_module_path(root: &Path, module: &str) -> PathBuf {
        module_path(root, module, "sifr")
    }

    fn private_module_path(root: &Path, module: &str) -> PathBuf {
        module_path(root, module, "_sifr")
    }

    fn module_path(root: &Path, module: &str, prefix: &str) -> PathBuf {
        let filename = module
            .strip_prefix(prefix)
            .and_then(|tail| tail.strip_prefix('.'))
            .test_unwrap("test module should use expected prefix");
        root.join(format!("{filename}.sifr"))
    }
}
