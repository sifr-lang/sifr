#![allow(clippy::print_stdout, clippy::print_stderr)]

use serde_json::Value;
use sifr_diagnostics::RenderedDiagnostic;
use sifr_driver::{check_package_project, check_project, check_single_file, PackageEntrypoint};
use sifr_package::{
    derive_package_graph, parse_metadata_json, CargoCommandPlan, CargoLockMode, PackageSourceMap,
};
use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const PARSER_FIXTURES: &[(&str, &str)] = &[
    ("parser_bad_indent", "SIFR-PARSE-0002"),
    ("parser_unterminated_string", "SIFR-PARSE-0003"),
    ("parser_invalid_call_order", "SIFR-PARSE-0006"),
    ("parser_empty_declaration", "SIFR-PARSE-0007"),
    ("parser_invalid_declaration", "SIFR-PARSE-0002"),
    ("parser_invalid_match_pattern", "SIFR-PARSE-0008"),
    ("parser_unsupported_syntax", "SIFR-PARSE-0009"),
];

const PROJECT_FIXTURES: &[(&str, &str, &[&str])] = &[
    (
        "workspace_missing_import_canonical",
        "SIFR-IMPORT-0002",
        &["resolution_scope", "tried_paths"],
    ),
    (
        "bare_stdlib_import_canonical",
        "SIFR-IMPORT-0008",
        &[
            "bare_module",
            "suggested_module",
            "imported_names",
            "resolution_scope",
            "tried_paths",
        ],
    ),
    (
        "workspace_namespace_collision_canonical",
        "SIFR-IMPORT-0006",
        &["resolved_path", "parent_path"],
    ),
    (
        "workspace_bare_stdlib_import",
        "SIFR-IMPORT-0008",
        &[
            "bare_module",
            "suggested_module",
            "imported_names",
            "resolution_scope",
            "tried_paths",
        ],
    ),
];

const CYCLE_FIXTURES: &[(&str, &str, &[&str])] = &[(
    "import_cycle_source_spans",
    "SIFR-IMPORT-0007",
    &["cycle", "cycle_edges"],
)];

const PACKAGE_FIXTURES: &[(&str, &str, &[&str])] = &[
    (
        "package_missing_import_canonical",
        "SIFR-IMPORT-0002",
        &[
            "resolution_scope",
            "tried_paths",
            "written_module_path",
            "package_import_origin",
        ],
    ),
    (
        "package_ambiguous_import_canonical",
        "SIFR-IMPORT-0005",
        &[
            "resolution_scope",
            "candidate_paths",
            "written_module_path",
            "package_import_origin",
        ],
    ),
    (
        "package_bare_stdlib_import_canonical",
        "SIFR-IMPORT-0008",
        &[
            "bare_module",
            "suggested_module",
            "imported_names",
            "resolution_scope",
            "tried_paths",
            "written_module_path",
            "package_import_origin",
        ],
    ),
];

const PACKAGE_FATAL_FIXTURES: &[(&str, &str, &[&str])] = &[(
    "package_fatal_source_map_no_import_ambiguity",
    "SIFR-PACKAGE-0713",
    &["origin_kind", "manifest_path", "manifest_key"],
)];

fn main() {
    if let Err(error) = run() {
        eprintln!("diagnostic rendering harness: FAIL: {error}");
        std::process::exit(1);
    }
    println!("diagnostic rendering harness: PASS");
}

fn run() -> Result<(), String> {
    let repo_root = repo_root()?;
    match HarnessArgs::parse()? {
        HarnessArgs::All => {
            check_parser_runtime_rules(&repo_root)?;
            check_project_runtime_rules(&repo_root)?;
            check_cycle_runtime_rules(&repo_root)?;
            check_package_runtime_rules(&repo_root)?;
        }
        HarnessArgs::Target {
            target_id,
            seed_path,
        } => match target_id.as_str() {
            "diagnostic_renderer_entrypoint" => {
                check_parser_seed_runtime_rules(&repo_root, &seed_path)?;
            }
            "package_project_manifest_entrypoint" => {
                check_project_seed_runtime_rules(&repo_root, &seed_path)?;
            }
            _ => return Err(format!("unknown diagnostic rendering target: {target_id}")),
        },
    }
    Ok(())
}

enum HarnessArgs {
    All,
    Target {
        target_id: String,
        seed_path: PathBuf,
    },
}

impl HarnessArgs {
    fn parse() -> Result<Self, String> {
        let mut args = env::args().skip(1);
        let Some(first) = args.next() else {
            return Ok(Self::All);
        };
        if first != "--target" {
            return Err(format!("unexpected argument: {first}"));
        }
        let target_id = args
            .next()
            .ok_or_else(|| "--target requires a target id".to_string())?;
        if args.next().as_deref() != Some("--seed") {
            return Err("--target requires --seed <path>".to_string());
        }
        let seed_path = args
            .next()
            .map(PathBuf::from)
            .ok_or_else(|| "--seed requires a path".to_string())?;
        if let Some(extra) = args.next() {
            return Err(format!("unexpected trailing argument: {extra}"));
        }
        Ok(Self::Target {
            target_id,
            seed_path,
        })
    }
}

fn repo_root() -> Result<PathBuf, String> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "sifr_driver should live under repo_root/crates/sifr_driver".to_string())
}

fn check_parser_runtime_rules(root: &Path) -> Result<(), String> {
    let base = root.join("verification/areas/diagnostics/fixtures/diagnostics");
    for (fixture, code) in PARSER_FIXTURES {
        check_parser_fixture(&base, fixture, code)?;
    }
    Ok(())
}

fn check_parser_seed_runtime_rules(root: &Path, seed_path: &Path) -> Result<(), String> {
    let base = root.join("verification/areas/diagnostics/fixtures/diagnostics");
    let seed = root.join(seed_path);
    let fixture = fixture_name_for_seed(&base, &seed, FixtureLayout::Flat)?;
    let (_, code) = PARSER_FIXTURES
        .iter()
        .find(|(candidate, _)| *candidate == fixture)
        .ok_or_else(|| format!("seed is not a parser rules fixture: {}", seed.display()))?;
    check_parser_fixture(&base, &fixture, code)
}

fn check_parser_fixture(base: &Path, fixture: &str, code: &str) -> Result<(), String> {
    let entry = base.join(fixture).join("main.sifr");
    let source = std::fs::read_to_string(&entry)
        .map_err(|err| format!("failed to read {}: {err}", entry.display()))?;
    let diagnostics = check_single_file(&source, &entry);
    assert_rules(&diagnostics, code, fixture, &[], true, true)?;
    assert_text_formats(&diagnostics, code, &entry)
}

fn check_project_runtime_rules(root: &Path) -> Result<(), String> {
    let base = root.join("verification/areas/project_workspace/fixtures/project");
    for (fixture, code, required_args) in PROJECT_FIXTURES {
        check_project_fixture(&base, fixture, code, required_args)?;
    }
    Ok(())
}

fn check_project_seed_runtime_rules(root: &Path, seed_path: &Path) -> Result<(), String> {
    let base = root.join("verification/areas/project_workspace/fixtures/project");
    let seed = root.join(seed_path);
    let fixture = fixture_name_for_seed(&base, &seed, FixtureLayout::SourceRoot)?;
    if let Some((_, code, required_args)) = PROJECT_FIXTURES
        .iter()
        .find(|(candidate, _, _)| *candidate == fixture)
    {
        return check_project_fixture(&base, &fixture, code, required_args);
    }
    if let Some((_, code, required_args)) = CYCLE_FIXTURES
        .iter()
        .find(|(candidate, _, _)| *candidate == fixture)
    {
        return check_project_fixture(&base, &fixture, code, required_args);
    }
    Err(format!(
        "seed is not a project rules fixture: {}",
        seed.display()
    ))
}

fn check_project_fixture(
    base: &Path,
    fixture: &str,
    code: &str,
    required_args: &[&str],
) -> Result<(), String> {
    let entry = base.join(fixture).join("src/main.sifr");
    let mut provider = sifr_frontend::DiskSourceProvider::new();
    let diagnostics = check_project(&entry, &mut provider);
    assert_rules(&diagnostics, code, fixture, required_args, true, true)?;
    assert_no_prefix(&diagnostics, fixture, "SIFR-WORKSPACE-01")?;
    assert_text_formats(&diagnostics, code, &entry)
}

fn check_cycle_runtime_rules(root: &Path) -> Result<(), String> {
    let base = root.join("verification/areas/project_workspace/fixtures/project");
    for (fixture, code, required_args) in CYCLE_FIXTURES {
        check_project_fixture(&base, fixture, code, required_args)?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum FixtureLayout {
    Flat,
    SourceRoot,
}

fn fixture_name_for_seed(
    base: &Path,
    seed: &Path,
    layout: FixtureLayout,
) -> Result<String, String> {
    if seed.file_name().and_then(|name| name.to_str()) != Some("main.sifr") {
        return Err(format!(
            "seed must point at a main.sifr fixture: {}",
            seed.display()
        ));
    }
    let parent = seed
        .parent()
        .ok_or_else(|| format!("seed has no parent fixture: {}", seed.display()))?;
    let fixture_root = match layout {
        FixtureLayout::Flat => parent,
        FixtureLayout::SourceRoot => {
            if parent.file_name().and_then(|name| name.to_str()) != Some("src") {
                return Err(format!(
                    "seed must use the canonical src/main.sifr layout: {}",
                    seed.display()
                ));
            }
            parent
                .parent()
                .ok_or_else(|| format!("seed has no parent fixture: {}", seed.display()))?
        }
    };
    fixture_root
        .strip_prefix(base)
        .ok()
        .and_then(|relative| {
            let mut components = relative.components();
            let first = components.next()?;
            if components.next().is_none() {
                Some(first.as_os_str().to_string_lossy().into_owned())
            } else {
                None
            }
        })
        .ok_or_else(|| format!("seed is outside expected fixture root: {}", seed.display()))
}

fn check_package_runtime_rules(root: &Path) -> Result<(), String> {
    let base = root.join("verification/areas/package_management/fixtures/package");
    for (fixture, code, required_args) in PACKAGE_FIXTURES {
        let package = base.join(fixture);
        let diagnostics = package_diagnostics(&package)?;
        assert_rules(&diagnostics, code, fixture, required_args, true, true)?;
        assert_no_prefix(&diagnostics, fixture, "SIFR-WORKSPACE-01")?;
        assert_no_prefix(&diagnostics, fixture, "SIFR-PACKAGE-")?;
        assert_text_formats(&diagnostics, code, &package)?;
    }
    for (fixture, code, required_args) in PACKAGE_FATAL_FIXTURES {
        let package = base.join(fixture);
        let diagnostics = package_diagnostics(&package)?;
        assert_rules(&diagnostics, code, fixture, required_args, false, true)?;
        assert_no_prefix(&diagnostics, fixture, "SIFR-IMPORT-")?;
    }
    Ok(())
}

fn package_diagnostics(package: &Path) -> Result<Vec<RenderedDiagnostic>, String> {
    let plan = CargoCommandPlan::metadata(package.to_path_buf(), CargoLockMode::Normal);
    let output = Command::new(&plan.program)
        .args(&plan.args)
        .current_dir(&plan.current_dir)
        .output()
        .map_err(|err| {
            format!(
                "failed to run cargo metadata for {}: {err}",
                package.display()
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed for {}: {}",
            package.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let metadata = parse_metadata_json(&stdout).map_err(|err| {
        format!(
            "failed to parse cargo metadata for {}: {err:?}",
            package.display()
        )
    })?;
    let mut provider = sifr_frontend::DiskSourceProvider::new();
    let graph = match derive_package_graph(metadata, &mut provider) {
        Ok(graph) => graph,
        Err(errors) => {
            return Ok(errors
                .into_iter()
                .map(sifr_driver::render_package_diagnostic)
                .collect());
        }
    };
    let source_map = match PackageSourceMap::build(&graph, &mut provider) {
        Ok(source_map) => source_map,
        Err(errors) => {
            return Ok(errors
                .into_iter()
                .map(sifr_driver::render_package_diagnostic)
                .collect());
        }
    };
    let package_id = graph
        .packages
        .iter()
        .find(|(_, metadata)| metadata.package_root == package)
        .map(|(id, _)| id.clone())
        .ok_or_else(|| format!("could not find package id for {}", package.display()))?;
    let entry = find_package_entry(package)?;
    let entrypoint = PackageEntrypoint {
        main_file: entry,
        package_id,
        graph,
        source_map,
        python_runtime: None,
        lock_mode: sifr_package::CargoLockMode::Normal,
    };
    Ok(check_package_project(&entrypoint, &mut provider))
}

fn find_package_entry(package: &Path) -> Result<PathBuf, String> {
    for candidate in ["src/main.sifr", "src1/main.sifr", "src_a/main.sifr"] {
        let path = package.join(candidate);
        if path.is_file() {
            return Ok(path);
        }
    }
    Err(format!(
        "package fixture missing main.sifr under {}",
        package.display()
    ))
}

fn assert_rules(
    diagnostics: &[RenderedDiagnostic],
    expected_code: &str,
    case_id: &str,
    required_args: &[&str],
    require_span: bool,
    render_json: bool,
) -> Result<(), String> {
    if diagnostics.is_empty() {
        return Err(format!("{case_id}: diagnostic payload is empty"));
    }
    let codes = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect::<BTreeSet<_>>();
    if !codes.contains(expected_code) {
        return Err(format!(
            "{case_id}: expected {expected_code}, got {codes:?}"
        ));
    }
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == expected_code)
        .ok_or_else(|| format!("{case_id}: expected diagnostic disappeared"))?;
    if require_span {
        assert_primary_span(diagnostic)?;
    }
    for arg in required_args {
        if !diagnostic.args.contains_key(*arg) {
            return Err(format!("{expected_code} missing JSON arg: {arg}"));
        }
    }
    if render_json {
        let json = sifr_diagnostics::render_json_diagnostics(diagnostics)
            .map_err(|err| format!("{case_id}: JSON rendering failed: {err}"))?;
        let payload: Value =
            serde_json::from_str(&json).map_err(|err| format!("{case_id}: invalid JSON: {err}"))?;
        if payload.as_array().is_none_or(std::vec::Vec::is_empty) {
            return Err(format!("{case_id}: rendered JSON payload is empty"));
        }
    }
    Ok(())
}

fn assert_primary_span(diagnostic: &RenderedDiagnostic) -> Result<(), String> {
    let span = diagnostic
        .spans
        .iter()
        .find(|span| span.is_primary)
        .or_else(|| diagnostic.spans.first())
        .ok_or_else(|| format!("{} has no spans", diagnostic.code))?;
    if span.file.as_deref().is_none_or(|file| file == "<unknown>") {
        return Err(format!(
            "{} primary span file is <unknown>",
            diagnostic.code
        ));
    }
    if span.line.is_none_or(|line| line < 1) || span.column.is_none_or(|column| column < 1) {
        return Err(format!(
            "{} primary span location is not 1-based",
            diagnostic.code
        ));
    }
    if span.lines.is_empty() {
        return Err(format!(
            "{} primary span missing snippet lines",
            diagnostic.code
        ));
    }
    Ok(())
}

fn assert_text_formats(
    diagnostics: &[RenderedDiagnostic],
    expected_code: &str,
    entry: &Path,
) -> Result<(), String> {
    let human = sifr_diagnostics::render_human_diagnostics(diagnostics);
    if !human.contains(expected_code) || !human.contains("-->") || human.contains("<unknown>") {
        return Err(format!(
            "human output failed source rules for {}",
            entry.display()
        ));
    }
    let compact = sifr_diagnostics::render_compact_diagnostics(diagnostics);
    if !compact.contains(&format!("E {expected_code} ")) || compact.contains("<unknown>") {
        return Err(format!(
            "compact output failed source rules for {}",
            entry.display()
        ));
    }
    Ok(())
}

fn assert_no_prefix(
    diagnostics: &[RenderedDiagnostic],
    case_id: &str,
    forbidden_prefix: &str,
) -> Result<(), String> {
    for diagnostic in diagnostics {
        if diagnostic.code.starts_with(forbidden_prefix) {
            return Err(format!(
                "{case_id}: forbidden diagnostic family leaked: {}",
                diagnostic.code
            ));
        }
    }
    Ok(())
}
